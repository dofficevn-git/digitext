# Copyright (c) 2026 DigiText. All rights reserved.
"""Lớp dự phòng CHỮ VIẾT TAY cho dịch vụ OCR.

VietOCR (vgg_transformer) học trên chữ in; gặp chữ tay điền vào biểu mẫu (tên, ngày,
địa danh trên hàng chấm) nó bịa ra chữ in gần giống ("Việt Yên" → "Di lệt, vịcm") với
độ tin cậy thấp (đo 18/9/2026: 0,74–0,83, trong khi chữ in thường ≥ 0,9). Mô hình
thị giác-ngôn ngữ (Qwen2.5-VL, đang phục vụ bóc trường) đọc chữ tay tiếng Việt tốt hơn
hẳn (địa danh/tên trường/tháng năm đúng, còn sai lẻ ở chữ số viết tay).

Cách làm: sau khi VietOCR xong, dòng nào độ tin cậy < OCR_HW_CONF được gửi crop (đã đệm
khung như crop VietOCR nhận) sang VLM với prompt "chỉ chép đúng MỘT dòng". Kết quả chỉ
được nhận khi qua kiểm hợp lệ (một dòng, độ dài tương xứng, không ký tự lạ); khi đó
text thay bằng bản VLM, độ tin cậy đặt OCR_HW_ACCEPT_CONF (mặc định 0,6 — vẫn dưới
ngưỡng soát để người kiểm ưu tiên), và `sources[i] = "hw"`.

Tắt: để trống OCR_HW_VLM_URL. Mọi lỗi mạng/VLM đều nuốt và giữ kết quả VietOCR —
lớp này chỉ được phép làm tốt hơn, không được làm hỏng.
"""
from __future__ import annotations

import base64
import concurrent.futures as cf
import io
import json
import os
import re
import time
import urllib.request

BLOCK_PROMPT = (
    "Đây là một phần văn bản hành chính tiếng Việt, gồm chữ in và nhiều chữ VIẾT TAY. Chép lại "
    "toàn bộ nội dung, MỖI DÒNG TRONG ẢNH LÀ MỘT DÒNG VĂN BẢN, đúng thứ tự từ trên xuống. "
    "Tiếng Việt có dấu. Không giải thích, không đánh số, không thêm chữ không có trong ảnh."
)

PROMPT = (
    "Ảnh là MỘT dòng chữ cắt từ văn bản hành chính Việt Nam, có phần chữ in và phần chữ "
    "VIẾT TAY điền vào chỗ trống. Chép lại chính xác đúng một dòng này (không thêm dòng "
    "khác, không suy đoán thêm chữ). Tiếng Việt có dấu. Chỉ trả về dòng chữ."
)


def _env_f(name: str, default: float) -> float:
    try:
        return float(os.getenv(name, "") or default)
    except ValueError:
        return default


def _env_i(name: str, default: int) -> int:
    try:
        return int(os.getenv(name, "") or default)
    except ValueError:
        return default


class HandwritingFallback:
    def __init__(self) -> None:
        self.url = (os.getenv("OCR_HW_VLM_URL") or "").strip()
        self.model = os.getenv("OCR_HW_MODEL", "Qwen2.5-VL-7B-Instruct-AWQ")
        self.conf = _env_f("OCR_HW_CONF", 0.85)
        self.accept_conf = _env_f("OCR_HW_ACCEPT_CONF", 0.6)
        self.min_chars = _env_i("OCR_HW_MIN_CHARS", 3)
        self.max_per_page = _env_i("OCR_HW_MAX_PER_PAGE", 15)
        self.timeout = _env_f("OCR_HW_TIMEOUT_S", 25.0)
        self.workers = max(1, _env_i("OCR_HW_WORKERS", 2))
        self.min_height = _env_i("OCR_HW_MIN_HEIGHT", 64)
        # Chế độ KHỐI: chữ tay nối nét cần ngữ cảnh — cắt từng dòng thì VLM đọc "Cụm Dân chủ
        # phường Lê Lợi" thành "Cụng Disoal Chế Ph 2008"; cho nhìn cả vùng thì đọc đúng
        # (đo 18/9/2026). Vùng = cụm ≥ block_min dòng tin cậy thấp gần nhau theo chiều dọc.
        self.block_min = _env_i("OCR_HW_BLOCK_MIN_LINES", 3)
        self.block_gap = _env_f("OCR_HW_BLOCK_GAP", 2.5)  # tính theo chiều cao dòng trung vị
        self.max_blocks = _env_i("OCR_HW_MAX_BLOCKS", 3)
        self.block_timeout = _env_f("OCR_HW_BLOCK_TIMEOUT_S", 90.0)
        # Trần token trả lời. Dòng tiếng Việt thật ~25–40 token; trần cao chỉ để vòng lặp đốt GPU
        # (6/10/2026: 4% yêu cầu chạm trần 160 / 90×dòng ≈ 1/4–1/3 tổng token sinh). Đo lại tỉ lệ
        # finished_reason="length" của vLLM sau khi đổi — tăng vọt nghĩa là đang cắt cụt dòng thật.
        # Bị cắt ở trần thì phần cụt KHÔNG được nhận (giữ chữ VietOCR) — trần thấp chỉ làm mất
        # công đọc lại, không làm thiếu chữ.
        self.line_max_tokens = max(32, _env_i("OCR_HW_LINE_MAX_TOKENS", 96))
        self.block_tokens_per_line = max(20, _env_i("OCR_HW_BLOCK_TOKENS_PER_LINE", 65))
        # Cạnh dài tối đa của ảnh gửi VLM. Qwen2.5-VL tính theo mảnh 28×28 px nên ảnh nhỏ hơn
        # = ít token hơn = nhanh hơn (GPU máy OCR đã chạy 98% — 26/9/2026). Đổi thì phải đo
        # lại độ phủ + nội dung trên trang tay thật (dev/thu-hw-max-side).
        self.max_side = max(256, _env_i("OCR_HW_MAX_SIDE", 1600))
        self._pool = cf.ThreadPoolExecutor(max_workers=self.workers) if self.url else None
        self._warned = False

    @property
    def enabled(self) -> bool:
        return bool(self.url)

    def describe(self) -> dict:
        return {
            "enabled": self.enabled,
            "url": self.url or None,
            "model": self.model if self.enabled else None,
            "conf_threshold": self.conf,
            "max_side": self.max_side,
        }

    # ---- VLM ---------------------------------------------------------------
    def _read_line(self, crop) -> str | None:
        w, h = crop.size
        if h < self.min_height:
            crop = crop.resize((max(1, int(w * self.min_height / h)), self.min_height))
        text, cut = self._ask(crop, PROMPT, self.line_max_tokens, self.timeout)
        # Trả lời bị cắt ở trần token = dòng cụt đuôi. Không nhận: giữ chữ VietOCR đủ cả dòng,
        # để lớp chữ không bao giờ thiếu phần chữ có trên ảnh (6/10/2026, yêu cầu Ctrl+A).
        return None if cut else text

    def _ask(self, crop, prompt: str, max_tokens: int, timeout: float) -> tuple[str | None, bool]:
        """Trả (nội dung, bị_cắt_ở_trần_token)."""
        if max(crop.size) > self.max_side:
            k = self.max_side / max(crop.size)
            crop = crop.resize((max(1, int(crop.width * k)), max(1, int(crop.height * k))))
        buf = io.BytesIO()
        crop.save(buf, "PNG")
        body = {
            "model": self.model,
            "temperature": 0.0,
            "max_tokens": max_tokens,
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "image_url", "image_url": {
                        "url": "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode()}},
                    {"type": "text", "text": prompt},
                ],
            }],
        }
        req = urllib.request.Request(
            self.url, data=json.dumps(body).encode(), headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(req, timeout=timeout) as r:
            out = json.load(r)
        ch = out["choices"][0]
        return ch["message"]["content"], ch.get("finish_reason") == "length"

    # ---- chữ bịa ------------------------------------------------------------
    # VLM bịa nội dung không có trên giấy (đo 6/10/2026 trên phông Sở NN Hà Bắc thập niên 70):
    # "giấy chứng nhận … số 123456789 dated 01/01/2020", "tại xã X, huyện Y, tỉnh Z",
    # "Dấu mộc đỏ: CÔNG TY …", "Chữ viết tay bên trái: Xuan". Bị chặn thì giữ chữ VietOCR của
    # chính khung đó — khung không bao giờ mất chữ, chỉ không lấy bản VLM.
    _FAKE = re.compile(
        r"0?123456789|987654321|([1-9])\1{5,}"   # "1000000 đồng" là thật — không bắt chuỗi số 0
        r"|\b(dated|signed|signature|issued|certificate|stamp)\b"
        r"|\b(xã|huyện|tỉnh|phường|quận|ông|bà) [XYZ]\b",
        re.IGNORECASE)
    _DESCRIBE = re.compile(r"^\s*(chữ (viết tay|in|ký)|dấu (mộc|đỏ|tròn)|con dấu|chữ ký)[^:]{0,30}:",
                           re.IGNORECASE)
    # Số ký tự tối đa một khung chứa nổi: n ≤ K·(cạnh dài / cạnh ngắn) + C. Đo 6/10/2026 trên 2.030
    # dòng VietOCR tin cậy ≥ 0,9: p99,9 = 5,2·tỉ lệ, lớn nhất 5,6 → K=5, C=10 không chặn nhầm dòng
    # thật nào, chặn 67/364 dòng VLM (phần lớn là chữ bịa / đọc gộp cả hàng bảng vào một ô).
    # 6/10/2026 chiều: K 5 → 6 (bớt chặn oan tiêu đề bảng xuống dòng); lặp chữ đã có luật _REPEAT.
    _FIT_K = _env_f("OCR_HW_FIT_K", 6.0)
    # Cùng một từ/số (≥3 ký tự có chữ hoặc số, hoặc một chữ cái) lặp liền ≥4 lần trong MỘT khung:
    # "4.405,07 4.405,07 4.405,07 4.405,07", "x x x x x x" — VLM lặp, không phải chữ trên giấy.
    # Hàng chấm "… … … …" không có chữ/số nên không bị bắt.
    _REPEAT = re.compile(r"(?<!\S)((?=\S*[^\W_])\S{3,}|[^\W\d_])(?:\s+\1){3,}(?!\S)")
    _FIT_C = _env_f("OCR_HW_FIT_C", 10.0)

    _LOG_REJECTED = os.getenv("OCR_HW_LOG_REJECTED", "true").lower() in ("1", "true", "yes")

    @classmethod
    def rejected(cls, t: str, box) -> bool:
        """Bịa hoặc tràn khung? Ghi log từng dòng bị chặn (OCR_HW_LOG_REJECTED) để soi tỉ lệ chặn oan."""
        why = ("so-gia" if cls._FAKE.search(t) else "mo-ta" if cls._DESCRIBE.search(t)
               else "lap" if cls._REPEAT.search(t)
               else None if cls.fits(t, box) else "tran-khung")
        if why and cls._LOG_REJECTED:
            w, h = box[2] - box[0], box[3] - box[1]
            print(f"🚫 chặn [{why}] khung {w:.0f}x{h:.0f} n={len(t)}: {t[:120]}", flush=True)
        return why is not None

    @classmethod
    def made_up(cls, t: str) -> bool:
        return bool(cls._FAKE.search(t) or cls._DESCRIBE.search(t) or cls._REPEAT.search(t))

    @classmethod
    def fits(cls, t: str, box) -> bool:
        if box is None or cls._FIT_K <= 0:
            return True
        w, h = box[2] - box[0], box[3] - box[1]
        if min(w, h) < 4:
            return True
        return len(t) <= cls._FIT_K * max(w, h) / min(w, h) + cls._FIT_C

    # ---- kiểm hợp lệ --------------------------------------------------------
    @staticmethod
    def accept(ocr_text: str, vlm_text: str | None, loose: bool = False, box=None) -> str | None:
        """Trả về dòng đã làm sạch nếu đáng tin, None nếu giữ kết quả VietOCR."""
        if not vlm_text:
            return None
        t = vlm_text.strip().strip('"').strip("'").strip()
        # VLM đọc lấn sang dòng trên/dưới → trả nhiều dòng: bỏ, không đoán dòng nào đúng.
        if "\n" in t:
            return None
        t = re.sub(r"\s+", " ", t)
        if len(t) < 2:
            return None
        # Kẹt vòng lặp → không tin cả dòng, giữ chữ VietOCR (dòng vẫn còn trong lớp chữ).
        if HandwritingFallback._looping(t):
            return None
        # Từ chối câu trả lời kiểu hội thoại / từ chối ("Xin lỗi", "Không thể", "The image").
        low = t.lower()
        if low.startswith(("xin lỗi", "không thể", "tôi không", "the image", "i cannot", "i'm sorry")):
            return None
        # Ký tự lạ (CJK, emoji…) là dấu hiệu mô hình trượt ngôn ngữ.
        if re.search(r"[　-鿿\U0001F000-\U0001FFFF]", t):
            return None
        # Độ dài tương xứng với dòng VietOCR đọc — chữ tay có thể dài hơn chữ in bịa ra
        # (VietOCR hay rút gọn phần chấm chấm), nên nới trần rộng hơn sàn.
        # loose: dòng từ lưới nét mực — chữ VietOCR ở đó là rác/rỗng, không dùng làm thước đo.
        n = max(len(ocr_text), 1)
        if loose:
            if len(t) > 200:
                return None
        elif not (0.4 * n <= len(t) <= 3.0 * n + 12):
            return None
        if box is not None and HandwritingFallback.rejected(t, box):
            return None
        if box is None and HandwritingFallback.made_up(t):
            return None
        return t

    # ---- lặp vô hạn ------------------------------------------------------
    # VLM (giải mã tham lam) đôi khi kẹt vòng: "1 tháng lương la: 1 tháng lương la: …" ×200
    # (đo 26/9/2026). Vừa đưa rác vào lớp chữ, vừa đốt tới max_tokens (~18 s/khối).
    _LOOP = re.compile(r"(.{3,60}?)(?:\s*\1){3,}")

    @staticmethod
    def _wordy(unit: str) -> bool:
        # Chỉ coi là kẹt vòng khi đoạn lặp có chữ cái — hàng chấm "……", "0000", "! ! !" của
        # biểu mẫu/bảng là nội dung thật, giữ nguyên.
        return sum(c.isalpha() for c in unit) >= 2

    # Lặp KHÔNG có chữ (`" " " "…`, dấu câu) chỉ coi là kẹt vòng khi dài bất thường: hàng
    # chấm điền, dấu "như trên" của bảng không bao giờ dài tới mức này (26/9/2026: 2.262 ký tự).
    _JUNK_RUN = 200

    @classmethod
    def _bad(cls, m) -> bool:
        return cls._wordy(m.group(1)) or len(m.group(0)) > cls._JUNK_RUN

    @classmethod
    def _looping(cls, t: str) -> bool:
        return any(cls._bad(m) for m in cls._LOOP.finditer(t))

    @classmethod
    def _unloop(cls, t: str) -> str:
        """Gộp đoạn lặp ≥4 lần liên tiếp về một lần (giữ nội dung trước/sau vòng lặp); lặp
        không chữ mà quá dài thì cắt về 40 ký tự."""
        def fix(m):
            if cls._wordy(m.group(1)):
                return m.group(1)
            if len(m.group(0)) > cls._JUNK_RUN:
                return m.group(0)[:40]
            return m.group(0)
        return cls._LOOP.sub(fix, t)

    # ---- tiện ích ---------------------------------------------------------
    @staticmethod
    def _norm(t: str) -> str:
        import unicodedata
        t = unicodedata.normalize("NFD", t.lower()).replace("đ", "d")
        return "".join(c for c in t if c.isalnum() and not unicodedata.combining(c))

    @classmethod
    def _sim(cls, a: str, b: str) -> float:
        import difflib
        a, b = cls._norm(a), cls._norm(b)
        if not a or not b:
            return 0.0
        return difflib.SequenceMatcher(None, a, b).ratio()

    @staticmethod
    def _clean_lines(raw: str | None) -> list[str]:
        if not raw:
            return []
        out = []
        for ln in raw.replace("\r", "").split("\n"):
            ln = re.sub(r"\s+", " ", ln).strip().strip("`").strip()
            if not ln or re.fullmatch(r"[-–—_.·•\s]+", ln):
                continue
            if re.search(r"[　-鿿]", ln):
                continue
            out.append(ln)
        return out

    def _crop_line(self, page, box):
        x1, y1, x2, y2 = box
        h = max(1.0, y2 - y1)
        return page.crop((max(0, int(x1 - 0.15 * h)), max(0, int(y1 - 0.12 * h)),
                          min(page.width, int(x2 + 0.15 * h)), min(page.height, int(y2 + 0.12 * h))))

    def _blocks(self, boxes, low: set[int]) -> list[list[int]]:
        """Gom các dòng tin cậy thấp gần nhau theo chiều dọc thành khối; mỗi khối trả về MỌI
        chỉ số dòng (cả dòng in) nằm trong dải dọc của khối, xếp theo y."""
        if len(low) < self.block_min:
            return []
        hs = sorted(b[3] - b[1] for b in boxes)
        med = max(1.0, hs[len(hs) // 2])
        order = sorted(low, key=lambda i: (boxes[i][1] + boxes[i][3]) / 2)
        groups, cur = [], [order[0]]
        for i in order[1:]:
            if boxes[i][1] - boxes[cur[-1]][3] <= self.block_gap * med:
                cur.append(i)
            else:
                groups.append(cur)
                cur = [i]
        groups.append(cur)
        out = []
        for g in groups:
            if len(g) < self.block_min:
                continue
            top = min(boxes[i][1] for i in g) - 1.2 * med   # kèm ~1 dòng in trên/dưới làm mốc
            bot = max(boxes[i][3] for i in g) + 1.2 * med
            span = [i for i, b in enumerate(boxes) if top <= (b[1] + b[3]) / 2 <= bot]
            span.sort(key=lambda i: ((boxes[i][1] + boxes[i][3]) / 2, boxes[i][0]))
            out.append(span)
        out.sort(key=len, reverse=True)
        return out[: self.max_blocks]

    def _align(self, span, texts, confs, vlm_lines):
        """Gióng dòng VLM vào dòng đã phát hiện (Needleman–Wunsch). Dòng IN (tin cậy cao) là
        neo: điểm theo độ giống chữ. Dòng TAY: chữ VietOCR là rác nên cho điểm nền dương nhỏ,
        chỉ để giữ thứ tự. Trả ({chỉ số dòng: chữ VLM}, [(chỉ số dòng liền trước, chữ thừa)])."""
        n, m = len(span), len(vlm_lines)
        gap = -0.35
        sc = [[0.0] * (m + 1) for _ in range(n + 1)]
        bt = [[0] * (m + 1) for _ in range(n + 1)]
        for a in range(1, n + 1):
            sc[a][0], bt[a][0] = a * gap, 1
        for b in range(1, m + 1):
            sc[0][b], bt[0][b] = b * gap, 2
        for a in range(1, n + 1):
            i = span[a - 1]
            is_low = confs[i] < self.conf
            for b in range(1, m + 1):
                sim = self._sim(texts[i], vlm_lines[b - 1])
                pair = (0.25 + 0.5 * sim) if is_low else (2.0 * sim - 0.6)
                best, how = sc[a - 1][b - 1] + pair, 0
                if sc[a - 1][b] + gap > best:
                    best, how = sc[a - 1][b] + gap, 1
                if sc[a][b - 1] + gap > best:
                    best, how = sc[a][b - 1] + gap, 2
                sc[a][b], bt[a][b] = best, how
        a, b, pairs, extra = n, m, {}, []
        while a > 0 or b > 0:
            how = bt[a][b]
            if a > 0 and b > 0 and how == 0:
                pairs[span[a - 1]] = vlm_lines[b - 1]
                a -= 1
                b -= 1
            elif a > 0 and (b == 0 or how == 1):
                a -= 1
            else:
                extra.append((a, vlm_lines[b - 1]))   # a = số dòng span đã đi qua
                b -= 1
        extra.reverse()
        return pairs, [(span[k - 1] if k > 0 else span[0], t) for k, t in extra]

    def _refine_block(self, page, boxes, texts, confs, span) -> tuple[dict[int, str], int]:
        """Trả ({chỉ số dòng: chữ VLM nhận}, số dòng VLM bị chặn vì bịa / tràn khung)."""
        x1 = min(boxes[i][0] for i in span)
        y1 = min(boxes[i][1] for i in span)
        x2 = max(boxes[i][2] for i in span)
        y2 = max(boxes[i][3] for i in span)
        mx, my = 0.02 * page.width, 0.006 * page.height
        crop = page.crop((max(0, int(x1 - mx)), max(0, int(y1 - my)),
                          min(page.width, int(x2 + mx)), min(page.height, int(y2 + my))))
        # Trần token theo số dòng của khối: câu trả lời bình thường không chạm trần, vòng lặp
        # thì bị cắt sớm thay vì chạy 1200.
        raw, cut = self._ask(crop, BLOCK_PROMPT, min(1200, self.block_tokens_per_line * len(span) + 80),
                             self.block_timeout)
        lines = [self._unloop(ln) for ln in self._clean_lines(raw)]
        # Bị cắt ở trần token: dòng cuối có thể cụt nửa chừng → bỏ; các dòng khối không nhận
        # được bản VLM giữ nguyên chữ VietOCR (không dòng nào bị thiếu chữ).
        if cut and lines:
            lines = lines[:-1]
        if not lines or len(lines) > 3 * len(span) + 5:
            return {}, 0
        pairs, extra = self._align(span, texts, confs, lines)
        out: dict[int, str] = {}
        rejected = 0
        for i, t in pairs.items():
            if confs[i] < self.conf:            # chỉ thay dòng TAY; dòng in giữ chữ VietOCR
                # Bịa / tràn khung → không nhận; dòng rơi xuống bước đọc từng dòng.
                if self.rejected(t, boxes[i]):
                    rejected += 1
                    continue
                out[i] = t
        # Dòng VLM không có khung (bộ phát hiện bỏ sót chữ tay trên hàng chấm): nối vào dòng
        # tay liền trước để nội dung vẫn nằm trong lớp chữ và tìm kiếm được — trừ khi bịa hoặc
        # làm dòng đích tràn khung (đo 6/10/2026: chữ bịa hay vào lớp chữ đúng qua lối này).
        low_in_span = [i for i in span if confs[i] < self.conf]
        for anchor, t in extra:
            if not low_in_span:
                break
            pos = span.index(anchor)
            cand = [i for i in span[: pos + 1] if confs[i] < self.conf] or low_in_span[:1]
            tgt = cand[-1]
            merged = (out.get(tgt, "") + " " + t).strip()
            if self.made_up(t) or self.rejected(merged, boxes[tgt]):
                rejected += 1
                continue
            out[tgt] = merged
        return out, rejected

    # ---- áp vào kết quả một trang ----------------------------------------
    def refine(self, page, boxes, texts: list[str], confs: list[float],
               force: set[int] | None = None) -> tuple[list[str], list[float], list[str], int]:
        """page: PIL RGB cả trang/tile; boxes/texts/confs cùng độ dài.
        Trả (texts, confs, sources, số dòng đã đổi)."""
        sources = ["ocr"] * len(texts)
        if not self.enabled or not texts:
            return texts, confs, sources, 0
        # force: dòng do lưới an toàn nét mực sinh ra (DBNet không thấy → gần như chắc là chữ
        # tay) — luôn đọc lại, kể cả khi VietOCR trả chuỗi rỗng/ngắn.
        force = force or set()
        low = [i for i, (t, c) in enumerate(zip(texts, confs))
               if c < self.conf and (i in force or len(t.strip()) >= self.min_chars)]
        if not low:
            return texts, confs, sources, 0
        t0 = time.perf_counter()
        texts, confs = list(texts), list(confs)
        orig_confs = list(confs)
        changed, done = 0, set()

        def warn(e):
            if not self._warned:
                self._warned = True
                print(f"⚠  lớp chữ viết tay: VLM lỗi, giữ kết quả VietOCR ({e})", flush=True)

        # 1) KHỐI — vùng nhiều dòng tay liền nhau, đọc có ngữ cảnh.
        blocks = self._blocks(boxes, set(low))
        n_block_lines = 0
        n_made_up = 0
        if blocks:
            def work_block(span):
                try:
                    return span, *self._refine_block(page, boxes, texts, orig_confs, span)
                except Exception as e:  # noqa: BLE001
                    warn(e)
                    return span, {}, 0

            for span, repl, rej in list(self._pool.map(work_block, blocks)):
                n_made_up += rej
                for i, t in repl.items():
                    t = re.sub(r"\s+", " ", t).strip()
                    if len(t) >= 2 and t != texts[i]:
                        texts[i], confs[i], sources[i] = t, self.accept_conf, "hw-block"
                        changed += 1
                        n_block_lines += 1
                    done.add(i)

        # 2) TỪNG DÒNG — dòng tay lẻ không thuộc khối nào (hoặc khối không trả kết quả).
        rest = [i for i in low if i not in done]
        rest.sort(key=lambda i: orig_confs[i])
        rest = rest[: self.max_per_page]

        def work(i: int):
            try:
                return i, self._read_line(self._crop_line(page, boxes[i]))
            except Exception as e:  # noqa: BLE001
                warn(e)
                return i, None

        for i, raw in self._pool.map(work, rest):
            t = self.accept(texts[i], raw, loose=i in force, box=boxes[i])
            if t is None and raw and (self.made_up(raw) or not self.fits(raw.strip(), boxes[i])):
                n_made_up += 1
            if t is None or t == texts[i]:
                continue
            texts[i], confs[i], sources[i] = t, self.accept_conf, "hw"
            changed += 1
        ms = (time.perf_counter() - t0) * 1000.0
        print(f"✍  chữ viết tay: {len(low)} dòng <{self.conf:.2f} · khối {len(blocks)} (đổi {n_block_lines}) · "
              f"lẻ {len(rest)} · tổng đổi {changed} · chặn bịa {n_made_up} · {ms:.0f}ms", flush=True)
        return texts, confs, sources, changed


_FALLBACK: HandwritingFallback | None = None


def get() -> HandwritingFallback:
    global _FALLBACK
    if _FALLBACK is None:
        _FALLBACK = HandwritingFallback()
    return _FALLBACK
