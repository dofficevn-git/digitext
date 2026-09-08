#!/usr/bin/env python3
"""Dựng dữ liệu huấn luyện nhận dạng từ ảnh tài liệu KHÔNG có nhãn từng dòng.

VẤN ĐỀ: train_vietocr.py cần `train.txt` dạng `<ảnh dòng>\\t<chữ của dòng>`.
Nhưng 878 ảnh trong doc_extract_pla/exports chỉ có nhãn 8 TRƯỜNG KIE, không có
nhãn từng dòng. Gán tay 878 trang là không khả thi.

CÁCH LÀM — chưng cất từ VLM (cùng ý tưởng với ml/finetune/kie/distill_vlm_rec.py,
đã chạy thật trên ~5000 trang theo MEMORY.md của bản gốc):

    ảnh trang → dịch vụ OCR (/api/ocr/tile) cắt ra từng DÒNG
              → mỗi dòng gửi cho VLM đọc lại
              → VLM đọc gì thì lấy làm NHÃN

VLM (Qwen2.5-VL 7B) đọc chữ tốt hơn hẳn model nhận dạng nhỏ, nhưng chậm và
không trả toạ độ — nên nó không thay được engine lúc chạy thật. Dùng nó làm
"thầy" để dạy lại VietOCR thì đúng vai.

BỘ LỌC BẤT ĐỒNG: nếu VietOCR và VLM đọc ra hai thứ QUÁ khác nhau (CER vượt
ngưỡng), thường là ô cắt hỏng — dòng cụt, dính hai dòng, hoặc vùng không phải
chữ. Học từ những cặp đó sẽ đầu độc model. distill_vlm_rec.py cũng chặn theo
cách này. Ngưỡng mặc định 0.6 nới hơn bản gốc (0.3) vì ở đây ta CỐ Ý muốn giữ
những dòng VietOCR đọc sai — đó chính là thứ cần dạy.

TÁCH TẬP THEO TRANG, không theo dòng: các dòng cùng một trang có cùng phông
chữ, cùng chất lượng quét. Trộn lẫn chúng vào cả train lẫn val sẽ cho điểm val
đẹp giả tạo.

    # trên máy có GPU (vLLM ở :8003) và dịch vụ OCR ở :5002
    python build_rec_dataset.py \\
        --images /duong/dan/exports/tu/images \\
        --out    data/rec_vi \\
        --limit  878
"""
from __future__ import annotations

import argparse
import base64
import io
import json
import os
import random
import sys
import unicodedata
import urllib.error
import urllib.request

from PIL import Image

LINE_PROMPT = (
    "Đọc CHÍNH XÁC dòng chữ tiếng Việt trong ảnh. Chỉ trả về đúng văn bản trên dòng, "
    "giữ nguyên dấu và chữ hoa/thường, KHÔNG giải thích, KHÔNG thêm dấu nháy."
)


def lev(a: str, b: str) -> int:
    if a == b:
        return 0
    if not a:
        return len(b)
    if not b:
        return len(a)
    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        cur = [i]
        for j, cb in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (ca != cb)))
        prev = cur
    return prev[-1]


def cer(pred: str, truth: str) -> float:
    if not truth:
        return 0.0 if not pred else 1.0
    return lev(pred, truth) / len(truth)


def norm(s: str) -> str:
    return unicodedata.normalize("NFC", " ".join((s or "").split()))


def post_json(url: str, payload: dict, headers: dict, timeout: int) -> dict:
    req = urllib.request.Request(
        url, data=json.dumps(payload).encode(),
        headers={"Content-Type": "application/json", **headers}, method="POST",
    )
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read().decode())


def ocr_page(endpoint: str, api_key: str, img_bytes: bytes, timeout: int) -> dict:
    return post_json(
        endpoint.rstrip("/") + "/api/ocr/tile",
        {"image": base64.b64encode(img_bytes).decode()},
        {"X-API-Key": api_key} if api_key else {},
        timeout,
    )


def fit_for_vlm(png: bytes, max_side: int, max_px: int, min_side: int) -> bytes:
    """Ép ảnh vào khoảng kích thước mà VLM xử lý được. CẢ HAI đầu đều quan trọng.

    QUÁ LỚN: một ô cắt hỏng bao trùm cả trang thành hàng chục nghìn token thị
    giác, đủ để hạ máy chủ vLLM — đã xảy ra. digitext-ocr chặn đúng chỗ này:
    vlm.rs ép cạnh dài về 1400px trước khi gọi.

    QUÁ NHỎ: đây là chỗ tôi bỏ sót ban đầu, và nó gây hỏng 81/92 lời gọi. Ảnh
    một DÒNG CHỮ rất dẹt — rộng vài trăm pixel nhưng cao chỉ 15-25. Qwen2.5-VL
    chia ảnh thành ô 28×28; ảnh cao 20px không đủ MỘT ô theo chiều dọc và máy
    chủ trả 500 không kèm thông báo. Dòng cao (tiêu đề) thì lọt, nên lỗi trông
    như ngẫu nhiên.

    Phóng to bằng LANCZOS chứ không đệm viền: model đọc chữ, thêm viền trắng
    chỉ tổ làm loãng vùng có nội dung.
    """
    im = Image.open(io.BytesIO(png)).convert("RGB")
    w, h = im.size

    # Phóng lên trước, nếu cạnh ngắn chưa đủ vài ô patch.
    up = max(1.0, min_side / max(1, min(w, h)))
    w, h = int(w * up), int(h * up)

    # Rồi mới thu về trong trần — thứ tự này quan trọng: thu trước thì bước
    # phóng sẽ đẩy ngược ra ngoài trần.
    down = min(1.0, max_side / max(w, h), (max_px / max(1, w * h)) ** 0.5)
    w, h = max(min_side, int(w * down)), max(min_side, int(h * down))

    if (w, h) == im.size:
        return png
    buf = io.BytesIO()
    im.resize((w, h), Image.LANCZOS).save(buf, "PNG")
    return buf.getvalue()


def vlm_read_line(endpoint: str, model: str, png: bytes, timeout: int) -> str:
    b64 = base64.b64encode(png).decode()
    body = {
        "model": model,
        "temperature": 0.0,
        "max_tokens": 128,
        "messages": [{
            "role": "user",
            "content": [
                {"type": "text", "text": LINE_PROMPT},
                {"type": "image_url", "image_url": {"url": f"data:image/png;base64,{b64}"}},
            ],
        }],
    }
    out = post_json(endpoint, body, {}, timeout)
    return norm(out["choices"][0]["message"]["content"])


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--images", required=True, help="thư mục ảnh trang/header")
    ap.add_argument("--out", default="data/rec_vi", help="thư mục kết quả")
    ap.add_argument("--ocr-endpoint", default="http://127.0.0.1:5002")
    ap.add_argument("--ocr-api-key", default=os.getenv("OCR_API_KEY", ""))
    ap.add_argument("--vlm-endpoint", default="http://127.0.0.1:8003/v1/chat/completions")
    ap.add_argument("--vlm-model", default="Qwen2.5-VL-7B-Instruct-AWQ")
    ap.add_argument("--limit", type=int, default=0, help="0 = tất cả")
    ap.add_argument("--pad", type=int, default=3, help="nới hộp vài pixel cho khỏi cụt nét")
    ap.add_argument("--min-h", type=int, default=10, help="bỏ dòng thấp hơn (nhiễu)")
    ap.add_argument("--min-chars", type=int, default=2)
    ap.add_argument("--min-side", type=int, default=64,
                    help="phóng ảnh sao cho cạnh NGẮN đạt mức này. Qwen2.5-VL chia ô "
                         "28px; ảnh dòng cao 15-25px không đủ một ô và máy chủ trả 500")
    ap.add_argument("--max-side", type=int, default=1400,
                    help="ép cạnh dài ảnh gửi VLM về mức này — CHẶN ô hỏng làm sập vLLM")
    ap.add_argument("--max-px", type=int, default=1_200_000,
                    help="trần tổng số điểm ảnh gửi VLM")
    ap.add_argument("--max-box-ratio", type=float, default=0.5,
                    help="bỏ hộp cao hơn tỉ lệ này của trang — chắc chắn không phải một dòng")
    ap.add_argument("--max-cer", type=float, default=0.6,
                    help="bất đồng vượt ngưỡng thì bỏ — ô cắt hỏng")
    ap.add_argument("--val-ratio", type=float, default=0.1)
    ap.add_argument("--timeout", type=int, default=180)
    ap.add_argument("--seed", type=int, default=42)
    ap.add_argument("--dry-run", action="store_true",
                    help="chỉ cắt dòng và đếm, KHÔNG gọi VLM")
    a = ap.parse_args()

    imgs = sorted(
        os.path.join(a.images, f) for f in os.listdir(a.images)
        if f.lower().endswith((".png", ".jpg", ".jpeg"))
    )
    if a.limit:
        imgs = imgs[: a.limit]
    if not imgs:
        print(f"không thấy ảnh nào trong {a.images}")
        return 1

    # Tách tập THEO TRANG trước khi cắt dòng — tránh rò rỉ.
    rnd = random.Random(a.seed)
    shuffled = imgs[:]
    rnd.shuffle(shuffled)
    n_val = max(1, int(len(shuffled) * a.val_ratio))
    val_set = set(shuffled[:n_val])

    crop_dir = os.path.join(a.out, "crops")
    os.makedirs(crop_dir, exist_ok=True)
    rows = {"train": [], "val": []}
    stat = {"trang": 0, "dong_phat_hien": 0, "bo_thap": 0, "bo_qua_to": 0,
            "bo_ngan": 0, "bo_bat_dong": 0, "loi_ocr": 0, "loi_vlm": 0, "giu": 0}
    cer_sum = 0.0

    print(f"{len(imgs)} trang ({len(val_set)} cho val) -> {a.out}")
    for n, path in enumerate(imgs, 1):
        try:
            blob = open(path, "rb").read()
            res = ocr_page(a.ocr_endpoint, a.ocr_api_key, blob, a.timeout)
        except Exception as e:  # noqa: BLE001
            stat["loi_ocr"] += 1
            print(f"  [{n}] OCR lỗi {os.path.basename(path)}: {e}")
            continue

        stat["trang"] += 1
        im = Image.open(io.BytesIO(blob)).convert("RGB")
        W, H = im.size
        split = "val" if path in val_set else "train"
        stem = os.path.splitext(os.path.basename(path))[0]

        for i, (box, txt) in enumerate(zip(res["boxes"], res["texts"])):
            stat["dong_phat_hien"] += 1
            x1, y1, x2, y2 = box
            if y2 - y1 < a.min_h:
                stat["bo_thap"] += 1
                continue
            # Hộp cao gần bằng cả trang không thể là MỘT dòng chữ — đó là ô cắt
            # hỏng. Gửi nó cho VLM vừa vô nghĩa vừa nguy hiểm về bộ nhớ.
            if (y2 - y1) > H * a.max_box_ratio:
                stat["bo_qua_to"] += 1
                continue
            c = im.crop((max(0, int(x1) - a.pad), max(0, int(y1) - a.pad),
                         min(W, int(x2) + a.pad), min(H, int(y2) + a.pad)))
            buf = io.BytesIO()
            c.save(buf, "PNG")
            png = buf.getvalue()

            if a.dry_run:
                stat["giu"] += 1
                continue

            try:
                small = fit_for_vlm(png, a.max_side, a.max_px, a.min_side)
                truth = vlm_read_line(a.vlm_endpoint, a.vlm_model, small, a.timeout)
            except Exception as e:  # noqa: BLE001
                stat["loi_vlm"] += 1
                # In NGUYÊN thân lỗi cho 3 lần đầu — nuốt lỗi khiến lần trước
                # phải mò mẫm mới biết vLLM trả 500 vì cái gì.
                if stat["loi_vlm"] <= 3:
                    detail = ""
                    if isinstance(e, urllib.error.HTTPError):
                        try:
                            detail = " | " + e.read().decode(errors="replace")[:300]
                        except Exception:  # noqa: BLE001
                            pass
                    print(f"  VLM lỗi: {e}{detail}")
                continue

            if len(truth) < a.min_chars:
                stat["bo_ngan"] += 1
                continue
            d = cer(norm(txt), truth)
            if d > a.max_cer:
                stat["bo_bat_dong"] += 1
                continue

            rel = f"crops/{stem}_{i:03d}.png"
            with open(os.path.join(a.out, rel), "wb") as fh:
                fh.write(png)
            rows[split].append(f"{rel}\t{truth}")
            cer_sum += d
            stat["giu"] += 1

        if n % 25 == 0:
            print(f"  {n}/{len(imgs)} trang — giữ {stat['giu']} dòng")

    for split in ("train", "val"):
        p = os.path.join(a.out, f"{split}.txt")
        with open(p, "w", encoding="utf-8", newline="\n") as fh:
            fh.write("\n".join(rows[split]) + ("\n" if rows[split] else ""))
        print(f"{p}: {len(rows[split])} dòng")

    print("\n=== thống kê ===")
    for k, v in stat.items():
        print(f"  {k:16s} {v}")
    if stat["giu"] and not a.dry_run:
        print(f"\nCER trung bình VietOCR-hiện-tại so với VLM: {cer_sum/stat['giu']:.3f}")
        print("  (càng cao = VietOCR càng đọc sai nhiều = càng đáng fine-tune)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
