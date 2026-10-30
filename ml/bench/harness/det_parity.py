"""So hộp phát hiện DBNet: PaddleOCR 2.x (Paddle) với det_onnx.OnnxDetector (ONNX Runtime) trên cùng ảnh — 30/10/2026.
Chạy trong container OCR (có paddleocr) đã cài thêm onnxruntime + model ONNX:

  python det_parity.py --onnx /out/en_PP-OCRv3_det.onnx --anh /anh/*.png [--iou 0.95]

Mỗi ảnh: ghép hộp theo IoU tốt nhất; đạt khi số hộp bằng nhau và IoU trung bình ≥ ngưỡng. Thoát 1 nếu ảnh nào không đạt.
"""
from __future__ import annotations

import argparse
import glob
import os
import sys
import time

import cv2
import numpy as np
from shapely.geometry import Polygon


def iou(a, b) -> float:
    pa, pb = Polygon(a), Polygon(b)
    if not pa.is_valid or not pb.is_valid:
        return 0.0
    inter = pa.intersection(pb).area
    return inter / (pa.area + pb.area - inter + 1e-9)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--onnx", required=True)
    ap.add_argument("--anh", nargs="+", required=True)
    ap.add_argument("--iou", type=float, default=0.95)
    a = ap.parse_args()
    sys.path.insert(0, os.getenv("OCR_SRC", "/app"))
    from paddleocr import PaddleOCR
    import det_onnx

    paddle_det = PaddleOCR(lang="en", use_angle_cls=False, show_log=False, use_gpu=False)
    onnx_det = det_onnx.OnnxDetector(a.onnx, use_gpu=False)
    files = [f for p in a.anh for f in sorted(glob.glob(p))]
    hong = 0
    for f in files:
        img = cv2.imread(f)
        if img is None:
            print(f"{f}: không đọc được")
            continue
        t0 = time.time()
        bp = paddle_det.ocr(img, det=True, rec=False, cls=False)[0] or []
        t1 = time.time()
        bo = onnx_det.ocr(img, det=True, rec=False, cls=False)[0] or []
        t2 = time.time()
        ious = []
        for q in bp:
            ious.append(max((iou(q, r) for r in bo), default=0.0))
        tb = float(np.mean(ious)) if ious else (1.0 if not bo else 0.0)
        ok = len(bp) == len(bo) and tb >= a.iou
        hong += 0 if ok else 1
        print(f"{'ĐẠT ' if ok else 'LỆCH'} {os.path.basename(f)}: paddle {len(bp)} hộp {t1 - t0:.2f}s · onnx {len(bo)} hộp {t2 - t1:.2f}s · IoU TB {tb:.3f} · min {min(ious) if ious else 1:.3f}")
    print(f"{len(files) - hong}/{len(files)} ảnh đạt (IoU ≥ {a.iou}, cùng số hộp)")
    return 1 if hong else 0


if __name__ == "__main__":
    sys.exit(main())
