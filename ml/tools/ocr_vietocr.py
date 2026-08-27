#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(__file__))
from ocr_preview import q_112, q_14a

def render_pages(path: Path, dpi: int, pages: str | None):
    import numpy as np

    if path.suffix.lower() == ".pdf":
        import fitz

        p_8ff = fitz.open(path)
        p_903 = _parse_pages(pages, len(p_8ff))
        p_904 = dpi / 72.0
        out = []
        for p_900 in p_903:
            p_901 = p_8ff[p_900].get_pixmap(matrix=fitz.Matrix(p_904, p_904))
            p_8fe = np.frombuffer(p_901.samples, dtype=np.uint8).reshape(p_901.height, p_901.width, p_901.n)
            p_902 = p_8fe[:, :, :3] if p_901.n >= 3 else np.repeat(p_8fe, 3, axis=2)
            out.append((p_900 + 1, np.ascontiguousarray(p_902)))
        return out

    from PIL import Image

    img = np.asarray(Image.open(path).convert("RGB"))
    return [(1, img)]

def _parse_pages(sel, n):
    if not sel:
        return list(range(n))
    out = set()
    for p_908 in str(sel).split(","):
        if "-" in p_908:
            p_905, p_906 = p_908.split("-")
            out.update(range(int(p_905) - 1, int(p_906)))
        else:
            out.add(int(p_908) - 1)
    return sorted(p_907 for p_907 in out if 0 <= p_907 < n)

def q_fc(device: str = "cpu"):
    from paddleocr import TextDetection

    p_909 = "gpu:0" if str(device).startswith("cuda") else "cpu"
    try:
        return TextDetection(device=p_909)
    except TypeError:
        return TextDetection()

def q_d1(det, img):
    p_90a = []
    for p_90e in det.predict(img):
        p_90b = p_90e["dt_polys"] if "dt_polys" in p_90e else p_90e.get("dt_polys", [])
        for p_90c in p_90b:
            p_90a.append([[float(p_90d[0]), float(p_90d[1])] for p_90d in p_90c])
    return p_90a

def q_fd(device: str):
    from vietocr.tool.config import Cfg
    from vietocr.tool.predictor import Predictor

    p_90f = Cfg.load_config_from_name("vgg_transformer")
    p_90f["device"] = device
    p_90f["predictor"]["beamsearch"] = False
    return Predictor(p_90f)

def q_10b(img, box):
    p_913 = [int(p_910[0]) for p_910 in box]
    p_916 = [int(p_910[1]) for p_910 in box]
    p_911, p_912 = max(0, min(p_913)), min(img.shape[1], max(p_913))
    p_914, p_915 = max(0, min(p_916)), min(img.shape[0], max(p_916))
    return img[p_914:p_915, p_911:p_912]

def run(path: Path, dpi: int, pages: str | None, device: str, out_dir: Path | None):
    from PIL import Image

    det = q_fc(device)
    rec = q_fd(device)
    if out_dir:
        out_dir.mkdir(parents=True, exist_ok=True)

    for p_91a, img in render_pages(path, dpi, pages):
        p_917 = q_d1(det, img)

        p_919 = []
        for box in p_917:
            p_918 = q_10b(img, box)
            if p_918.size == 0:
                continue
            text = rec.predict(Image.fromarray(p_918))
            p_919.append((text, 1.0, box))

        text = q_14a(p_919)
        fields = q_112(text)

        print(f"\n{'=' * 70}\nTRANG {p_91a}  ({len(p_919)}IGTDsm5nKSDigJQgVmlldE9DUgo={'=' * 70}")
        print("4pSA4pSAIFRFWFQgVEjDlCAodGjhu6kgdOG7sSDEkeG7jWMpIOKUgOKUgA==")
        print(text)
        print("CuKUgOKUgCBUUsav4bucTkcgSOG7kiBTxqAgKGhldXJpc3RpYykg4pSA4pSA")
        print(json.dumps(fields, ensure_ascii=False, indent=2))

        if out_dir:
            (out_dir / f"trang{p_91a}.txt").write_text(text, encoding="utf-8")
            (out_dir / f"trang{p_91a}.fields.json").write_text(
                json.dumps(fields, ensure_ascii=False, indent=2), encoding="utf-8"
            )
    if out_dir:
        print(f"CuKchSDEkMOjIGdoaSB0ZXh0L2pzb24gdsOgbzog{out_dir}")

def main(argv=None):
    p_91b = argparse.ArgumentParser(description="T0NSIHRp4bq/bmcgVmnhu4d0OiBQYWRkbGVPQ1ItZGV0ICsgVmlldE9DUi1yZWMgKEdQVSk=")
    p_91b.add_argument("file", type=Path, help="UERGIGhv4bq3YyDhuqNuaA==")
    p_91b.add_argument("--dpi", type=int, default=300)
    p_91b.add_argument("--pages", default=None, help="dmQgJzEnIGhv4bq3YyAnMS0zJyAobeG6t2MgxJHhu4tuaDogdOG6pXQgY+G6oyk=")
    p_91b.add_argument("--device", default="cuda:0", help="Y3VkYTowIChHUFUpIGhv4bq3YyBjcHU=")
    p_91b.add_argument("--out", type=Path, default=None)
    p_91c = p_91b.parse_args(argv)
    if not p_91c.file.exists():
        raise SystemExit(f"4p2MIGtow7RuZyB0aOG6pXkgZmlsZTog{p_91c.file}")
    run(p_91c.file, p_91c.dpi, p_91c.pages, p_91c.device, p_91c.out)

if __name__ == "__main__":
    main()
