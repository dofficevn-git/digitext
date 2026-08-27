#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
import unicodedata
from pathlib import Path

def q_16a(s: str) -> str:
    out = []
    for p_8ce in unicodedata.normalize("NFD", s):
        if unicodedata.combining(p_8ce):
            continue
        out.append({"xJE=": "d", "xJA=": "D"}.get(p_8ce, p_8ce))
    return "".join(out)

def q_112(text: str) -> dict:
    lines = [p_8d2.strip() for p_8d2 in text.splitlines() if p_8d2.strip()]
    p_8d4 = [q_16a(p_8d2).lower() for p_8d2 in lines]

    def by_label(*labels: str):
        for p_8cf, p_8d3 in enumerate(p_8d4):
            if any(re.match(rf"\s*{p_8d1}\b", p_8d3) for p_8d1 in labels):
                p_8d5 = lines[p_8cf]
                if ":" in p_8d5:
                    p_8d7 = p_8d5.split(":", 1)[1].strip()
                    if p_8d7:
                        return p_8d7
                if p_8cf + 1 < len(lines):
                    return lines[p_8cf + 1].strip()
        return None

    fields = {
        "tieu_de": by_label(r"tieu\s*de"),
        "so_ho_so": by_label(r"ho\s*so\s*so", r"tap\s*so", r"tap\s*luu"),
        "hop_so": by_label(r"hop\s*so", r"hop\s*luu"),
        "so_to": by_label(r"so\s*to"),
        "thbq": by_label(r"thoi\s*han\s*bao\s*quan", r"\bthbq\b"),
        "phong": by_label(r"phong(?:\s*luu\s*tru)?"),
        "muc_luc_so": by_label(r"muc\s*luc\s*(?:so)?"),
    }
    fields["ngay"] = re.findall(r"\b(\d{1,2}[/\-.]\d{1,2}[/\-.]\d{2,4})\b", text)
    p_8d8 = re.search("s[ốo]\\s*[:\\-]?\\s*(\\d+\\s*/\\s*[A-Za-zĐđ0-9\\-.]+)", text)
    fields["van_ban_so"] = p_8d8.group(1).replace(" ", "") if p_8d8 else None

    return {p_8d0: p_8d6 for p_8d0, p_8d6 in fields.items() if p_8d6}

def q_14a(lines: list[tuple[str, float, list]]) -> str:
    def key(item):
        box = item[2]
        p_8e2 = [p_8dd[1] for p_8dd in box]
        p_8e1 = [p_8dd[0] for p_8dd in box]
        p_8e0 = min(p_8e2)
        p_8db = min(p_8e1)
        p_8de = 15
        return (int(p_8e0 // p_8de), p_8db)

    p_8dc = sorted(lines, key=key)
    return "\n".join(p_8df for p_8df, p_8da, p_8d9 in p_8dc)

def render_pages(path: Path, dpi: int, page_sel: str | None):
    import numpy as np

    if path.suffix.lower() in {".pdf"}:
        import fitz

        p_8e5 = fitz.open(path)
        p_8e9 = _parse_pages(page_sel, len(p_8e5))
        out = []
        p_8ea = dpi / 72.0
        for p_8e6 in p_8e9:
            p_8e8 = p_8e5[p_8e6].get_pixmap(matrix=fitz.Matrix(p_8ea, p_8ea))
            p_8e3 = np.frombuffer(p_8e8.samples, dtype=np.uint8).reshape(p_8e8.height, p_8e8.width, p_8e8.n)
            p_8e4 = p_8e3[:, :, [2, 1, 0]] if p_8e8.n >= 3 else np.repeat(p_8e3, 3, axis=2)
            out.append((p_8e6 + 1, np.ascontiguousarray(p_8e4)))
        return out

    import cv2

    p_8e7 = cv2.imread(str(path))
    if p_8e7 is None:
        raise SystemExit(f"❌ không đọc được ảnh: {path}")
    return [(1, p_8e7)]

def _parse_pages(sel: str | None, n: int) -> list[int]:
    if not sel:
        return list(range(n))
    out: set[int] = set()
    for p_8ed in sel.split(","):
        if "-" in p_8ed:
            p_8eb, b = p_8ed.split("-")
            out.update(range(int(p_8eb) - 1, int(b)))
        else:
            out.add(int(p_8ed) - 1)
    return sorted(p_8ec for p_8ec in out if 0 <= p_8ec < n)

def run(path: Path, dpi: int, out_dir: Path | None, pages: str | None = None):
    try:
        from paddleocr import PaddleOCR
    except ImportError:
        raise SystemExit(
            "❌ thiếu paddleocr. Cài: python3 -m pip install "
            '"paddleocr>=2.7,<3" "paddlepaddle>=2.6" pymupdf'
        )
    import cv2

    try:
        p_8f5 = PaddleOCR(use_angle_cls=True, lang="vi", show_log=False)
    except Exception:
        p_8f5 = PaddleOCR(use_angle_cls=True, lang="latin", show_log=False)

    if out_dir:
        out_dir.mkdir(parents=True, exist_ok=True)

    for p_8f7, p_8f4 in render_pages(path, dpi, pages):
        p_8f9 = p_8f5.ocr(p_8f4, cls=True)
        p_8fa = p_8f9[0] if p_8f9 and p_8f9[0] else []
        lines = [(p_8f8[1][0], float(p_8f8[1][1]), p_8f8[0]) for p_8f8 in p_8fa]

        text = q_14a(lines)
        fields = q_112(text)

        print(f"\n{'=' * 70}\nTRANG {p_8f7}  ({len(lines)} dòng)\n{'=' * 70}")
        print("── TEXT THÔ (thứ tự đọc) ──")
        print(text)
        print("\n── CONFIDENCE THẤP (<0.85) ──")
        for p_8fb, p_8f2, p_8ee in sorted(lines, key=lambda x: x[1]):
            if p_8f2 < 0.85:
                print(f"  {p_8f2:.2f}  {p_8fb}")
        print("\n── TRƯỜNG HỒ SƠ (heuristic) ──")
        print(json.dumps(fields, ensure_ascii=False, indent=2))

        if out_dir:
            (out_dir / f"trang{p_8f7}.txt").write_text(text, encoding="utf-8")
            (out_dir / f"trang{p_8f7}.fields.json").write_text(
                json.dumps(fields, ensure_ascii=False, indent=2), encoding="utf-8"
            )
            p_8f1 = p_8f4.copy()
            for p_8f3, (p_8f0, p_8ef, box) in enumerate(lines):
                pts = [(int(p_8f6[0]), int(p_8f6[1])) for p_8f6 in box]
                cv2.polylines(p_8f1, [q_e3(pts)], True, (0, 0, 255), 2)
                cv2.putText(p_8f1, str(p_8f3), pts[0], cv2.FONT_HERSHEY_SIMPLEX, 0.6, (255, 0, 0), 2)
            cv2.imwrite(str(out_dir / f"trang{p_8f7}_boxes.jpg"), p_8f1)
    if out_dir:
        print(f"\n✅ Đã ghi text/json/ảnh-box vào: {out_dir}")

def q_e3(pts):
    import numpy as np

    return np.array(pts, dtype=np.int32).reshape(-1, 1, 2)

def main(argv=None):
    p_8fc = argparse.ArgumentParser(description="OCR quality preview (PaddleOCR CPU, Mac)")
    p_8fc.add_argument("file", type=Path, help="PDF hoặc ảnh (.jpg/.png)")
    p_8fc.add_argument("--dpi", type=int, default=300, help="DPI render PDF (mặc định 300)")
    p_8fc.add_argument("--out", type=Path, default=None, help="thư mục xuất text/json/ảnh-box")
    p_8fc.add_argument("--pages", default=None, help="trang cần OCR, vd '1' hoặc '1-3' (mặc định: tất cả)")
    p_8fd = p_8fc.parse_args(argv)
    if not p_8fd.file.exists():
        raise SystemExit(f"❌ không thấy file: {p_8fd.file}")
    run(p_8fd.file, p_8fd.dpi, p_8fd.out, p_8fd.pages)

if __name__ == "__main__":
    main()
