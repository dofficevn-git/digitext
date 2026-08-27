#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import base64
import json
import re
import sys
import unicodedata
import urllib.request
from pathlib import Path

from cer import cer
from ocr_crops import load_jsonl, q_130, q_14d, q_17b

LINE_PROMPT = (
    "Đọc CHÍNH XÁC dòng chữ tiếng Việt trong ảnh. Chỉ trả về đúng văn bản trên dòng, "
    "giữ nguyên dấu và chữ hoa/thường, KHÔNG giải thích, KHÔNG thêm dấu nháy."
)
_DATE = re.compile("\\d{1,2}\\s*[/-]\\s*\\d{1,2}\\s*[/-]\\s*\\d{2,4}|ngày\\s+\\d|tháng\\s+\\d", re.I)

def q_117(bboxes: list[list[float]], y_tol_frac: float = 0.6) -> list[list[int]]:
    if not bboxes:
        return []
    p_2cb = sorted(p_2c8[3] for p_2c8 in bboxes)
    p_2cf = (p_2cb[len(p_2cb) // 2] or 1) * y_tol_frac
    p_2d0 = lambda i: bboxes[i][1] + bboxes[i][3] / 2
    p_2cd: list[list[int]] = []
    p_2c9: list[int] = []
    p_2ca = 0.0
    for i in sorted(range(len(bboxes)), key=p_2d0):
        if p_2c9 and abs(p_2d0(i) - p_2ca) > p_2cf:
            p_2cd.append(p_2c9)
            p_2c9 = []
        p_2c9.append(i)
        p_2ca = sum(p_2d0(p_2cc) for p_2cc in p_2c9) / len(p_2c9)
    if p_2c9:
        p_2cd.append(p_2c9)
    return [sorted(p_2ce, key=lambda i: bboxes[i][0]) for p_2ce in p_2cd]

def q_11c(text: str) -> bool:
    p_2d5 = (text or "").strip()
    if not p_2d5:
        return True
    p_2d4 = p_2d5.lower()
    if p_2d4.startswith("c+G7kQ==") or "c+G7kTo=" in p_2d4[:8]:
        return True
    if _DATE.search(p_2d4):
        return True
    p_2d3 = sum(p_2d2.isdigit() for p_2d2 in p_2d5)
    p_2d1 = sum(p_2d2.isalpha() for p_2d2 in p_2d5)
    return p_2d3 >= p_2d1

def q_17c(text: str, min_chars: int = 4, max_chars: int = 48) -> bool:
    p_2d7 = (text or "").strip()
    return min_chars <= len(p_2d7) <= max_chars and any(p_2d6.isalpha() for p_2d6 in p_2d7)

def q_120(vlm: str, triton: str, max_cer: float = 0.3) -> bool:
    return cer(unicodedata.normalize("NFC", vlm or ""),
               unicodedata.normalize("NFC", triton or "")) <= max_cer

def q_106(text: str) -> str | None:
    p_2d9 = (text or "").strip().splitlines()
    if not p_2d9:
        return None
    p_2d8 = p_2d9[0].strip().strip('"').strip("'").strip()
    p_2d8 = re.sub(r"\s+", " ", unicodedata.normalize("NFC", p_2d8))
    return p_2d8 or None

def q_177(image: bytes, endpoint: str, model: str, timeout: int) -> str | None:
    p_2da = base64.b64encode(image).decode()
    body = {
        "model": model,
        "messages": [{"role": "user", "content": [
            {"type": "image_url", "image_url": {"url": f"data:image/jpeg;base64,{p_2da}"}},
            {"type": "text", "text": LINE_PROMPT},
        ]}],
        "temperature": 0.0,
        "max_tokens": 96,
    }
    p_2dd = urllib.request.Request(endpoint, data=json.dumps(body).encode(),
                                headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(p_2dd, timeout=timeout) as p_2de:
            p_2dc = json.load(p_2de)
        p_2db = p_2dc["choices"][0]
        if p_2db.get("finish_reason") == "length":
            return None
        return q_106(p_2db["message"]["content"])
    except Exception:
        return None

def main(argv: list[str] | None = None) -> int:
    p_2e0 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_2e0.add_argument("bio_jsonl")
    p_2e0.add_argument("data_dir")
    p_2e0.add_argument("--endpoint", default="http://127.0.0.1:8003/v1/chat/completions")
    p_2e0.add_argument("--model", default="Qwen2.5-VL-7B-Instruct-AWQ")
    p_2e0.add_argument("--limit", type=int, default=500, help="số TRANG BIO xử lý (pilot 500)")
    p_2e0.add_argument("--offset", type=int, default=0,
                    help="BỎ QUA n trang đầu — dùng để lấy trang MỚI làm eval held-out (không nhiễm train)")
    p_2e0.add_argument("--dpi", type=int, default=150)
    p_2e0.add_argument("--pad", type=int, default=4)
    p_2e0.add_argument("--timeout", type=int, default=120)
    p_2e0.add_argument("--max-disagree", type=float, default=0.3,
                    help="giữ dòng nếu CER(VLM,Triton) ≤ ngưỡng (loại rác non-text 2 engine bất đồng)")
    p_2e0.add_argument("--out-dir", default="data/rec_distill")
    p_2e0.add_argument("--dry-run", action="store_true", help="chỉ gom dòng + lọc, KHÔNG gọi VLM (đếm)")
    p_2df = p_2e0.parse_args(argv)

    p_2e1, data_dir = Path(p_2df.bio_jsonl), Path(p_2df.data_dir)
    if not p_2e1.is_file():
        print(f"LỖI: không thấy {p_2e1}", file=sys.stderr)
        return 2
    p_2ec = load_jsonl(p_2e1)[p_2df.offset : p_2df.offset + p_2df.limit]
    p_2eb = Path(p_2df.out_dir)
    p_2e5 = p_2eb / "img"
    p_2e5.mkdir(parents=True, exist_ok=True)

    pairs = []
    p_2e8 = p_2e7 = p_2e9 = p_2ea = 0
    for p_2ee in p_2ec:
        words, bboxes = p_2ee.get("words", []), p_2ee.get("bboxes", [])
        p_2ed = data_dir / p_2ee["document_id"]
        for p_2e6, p_2e4 in enumerate(q_117(bboxes)):
            p_2e8 += 1
            text = " ".join(words[i] for i in p_2e4)
            if q_11c(text) or not q_17c(text):
                continue
            p_2e7 += 1
            if p_2df.dry_run:
                continue
            p_2f0 = q_17b(bboxes, p_2e4)
            if p_2f0 is None:
                continue
            p_2e2 = q_130(p_2f0, p_2df.pad, p_2ee["width"], p_2ee["height"])
            img = p_2e5 / f"{p_2ee['id']}__L{p_2e6:03d}.jpg"
            if not p_2ed.is_file() or not q_14d(p_2ed, p_2ee["page_no"], p_2e2, p_2df.dpi, img):
                p_2ea += 1
                continue
            with img.open("rb") as p_2e3:
                p_2ef = q_177(p_2e3.read(), p_2df.endpoint, p_2df.model, p_2df.timeout)
            if (not p_2ef or not q_17c(p_2ef, min_chars=2, max_chars=60)
                    or not q_120(p_2ef, text, p_2df.max_disagree)):
                img.unlink(missing_ok=True)
                p_2ea += 1
                continue
            pairs.append(f"{img.relative_to(p_2eb)}\t{p_2ef}")
            p_2e9 += 1

    if not p_2df.dry_run:
        (p_2eb / "pairs.txt").write_text("\n".join(pairs) + "\n", encoding="utf-8")

    print(f"trang {len(p_2ec)} | dòng {p_2e8} | body-text (sau lọc định danh) {p_2e7}", file=sys.stderr)
    if not p_2df.dry_run:
        print(f"  VLM transcribe OK {p_2e9} cặp (bỏ {p_2ea}) → {p_2eb}/pairs.txt", file=sys.stderr)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
