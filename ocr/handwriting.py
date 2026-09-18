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
        }

    # ---- VLM ---------------------------------------------------------------
    def _read_line(self, crop) -> str | None:
        w, h = crop.size
        if h < self.min_height:
            crop = crop.resize((max(1, int(w * self.min_height / h)), self.min_height))
        if max(crop.size) > 1600:
            k = 1600 / max(crop.size)
            crop = crop.resize((max(1, int(crop.width * k)), max(1, int(crop.height * k))))
        buf = io.BytesIO()
        crop.save(buf, "PNG")
        body = {
            "model": self.model,
            "temperature": 0.0,
            "max_tokens": 160,
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "image_url", "image_url": {
                        "url": "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode()}},
                    {"type": "text", "text": PROMPT},
                ],
            }],
        }
        req = urllib.request.Request(
            self.url, data=json.dumps(body).encode(), headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(req, timeout=self.timeout) as r:
            out = json.load(r)
        return out["choices"][0]["message"]["content"]

    # ---- kiểm hợp lệ --------------------------------------------------------
    @staticmethod
    def accept(ocr_text: str, vlm_text: str | None) -> str | None:
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
        # Từ chối câu trả lời kiểu hội thoại / từ chối ("Xin lỗi", "Không thể", "The image").
        low = t.lower()
        if low.startswith(("xin lỗi", "không thể", "tôi không", "the image", "i cannot", "i'm sorry")):
            return None
        # Ký tự lạ (CJK, emoji…) là dấu hiệu mô hình trượt ngôn ngữ.
        if re.search(r"[　-鿿\U0001F000-\U0001FFFF]", t):
            return None
        # Độ dài tương xứng với dòng VietOCR đọc — chữ tay có thể dài hơn chữ in bịa ra
        # (VietOCR hay rút gọn phần chấm chấm), nên nới trần rộng hơn sàn.
        n = max(len(ocr_text), 1)
        if not (0.4 * n <= len(t) <= 3.0 * n + 12):
            return None
        return t

    # ---- áp vào kết quả một trang ----------------------------------------
    def refine(self, crops: list, texts: list[str], confs: list[float]) -> tuple[list[str], list[float], list[str], int]:
        """crops/texts/confs cùng độ dài. Trả (texts, confs, sources, số dòng đã đổi)."""
        sources = ["ocr"] * len(texts)
        if not self.enabled or not texts:
            return texts, confs, sources, 0
        idx = [i for i, (t, c) in enumerate(zip(texts, confs))
               if c < self.conf and len(t.strip()) >= self.min_chars]
        if not idx:
            return texts, confs, sources, 0
        # Ưu tiên dòng tin cậy thấp nhất khi trang có quá nhiều dòng xấu (ảnh mờ toàn trang
        # không phải chữ tay — không đáng đốt VLM cho cả trang).
        idx.sort(key=lambda i: confs[i])
        idx = idx[: self.max_per_page]
        t0 = time.perf_counter()

        def work(i: int):
            try:
                return i, self._read_line(crops[i])
            except Exception as e:  # noqa: BLE001
                if not self._warned:
                    self._warned = True
                    print(f"⚠  lớp chữ viết tay: VLM lỗi, giữ kết quả VietOCR ({e})", flush=True)
                return i, None

        texts, confs = list(texts), list(confs)
        changed = 0
        for i, raw in self._pool.map(work, idx):
            t = self.accept(texts[i], raw)
            if t is None or t == texts[i]:
                continue
            texts[i], confs[i], sources[i] = t, self.accept_conf, "hw"
            changed += 1
        ms = (time.perf_counter() - t0) * 1000.0
        print(f"✍  chữ viết tay: gửi {len(idx)} dòng (<{self.conf:.2f}) → đổi {changed} · {ms:.0f}ms", flush=True)
        return texts, confs, sources, changed


_FALLBACK: HandwritingFallback | None = None


def get() -> HandwritingFallback:
    global _FALLBACK
    if _FALLBACK is None:
        _FALLBACK = HandwritingFallback()
    return _FALLBACK
