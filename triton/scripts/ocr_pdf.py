#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys

def q_138(spec: str | None, n: int) -> list[int]:
    if not spec:
        return list(range(n))
    p_a4f: list[int] = []
    for p_a51 in spec.split(","):
        p_a51 = p_a51.strip()
        if not p_a51:
            continue
        if "-" in p_a51:
            p_a4d, p_a4e = p_a51.split("-", 1)
            p_a4f.extend(range(int(p_a4d) - 1, int(p_a4e)))
        else:
            p_a4f.append(int(p_a51) - 1)
    return [p_a50 for p_a50 in p_a4f if 0 <= p_a50 < n]

def q_149(boxes, texts, confs, y_tol: float = 12.0):
    p_a56 = []
    for p_a53, t, p_a54 in zip(boxes, texts, confs):
        p_a55 = (float(p_a53[1]) + float(p_a53[3])) / 2.0
        p_a56.append((p_a55, float(p_a53[0]), t, float(p_a54)))
    p_a56.sort(key=lambda it: it[0])
    p_a59: list[list] = []
    for p_a55, p_a5a, t, p_a54 in p_a56:
        if p_a59 and abs(p_a55 - p_a59[-1][0][0]) <= y_tol:
            p_a59[-1].append((p_a55, p_a5a, t, p_a54))
        else:
            p_a59.append([(p_a55, p_a5a, t, p_a54)])
    p_a57 = []
    for p_a58 in p_a59:
        p_a58.sort(key=lambda it: it[1])
        p_a57.extend((t, p_a54) for p_a52, p_a52, t, p_a54 in p_a58)
    return p_a57

def _as_text(t) -> str:
    if isinstance(t, bytes):
        return t.decode("utf-8", "replace")
    return str(t)

def _render(pdf_path: str, dpi: int):
    import fitz
    import numpy as np

    p_a5c = fitz.open(pdf_path)
    p_a60 = dpi / 72.0
    p_a5d = fitz.Matrix(p_a60, p_a60)
    pages = []
    for p_a5e in p_a5c:
        p_a5f = p_a5e.get_pixmap(matrix=p_a5d, alpha=False)
        p_a5b = np.frombuffer(p_a5f.samples, dtype=np.uint8).reshape(p_a5f.height, p_a5f.width, p_a5f.n)
        if p_a5f.n == 4:
            p_a5b = p_a5b[:, :, :3]
        pages.append(np.ascontiguousarray(p_a5b[:, :, :3]))
    return pages

def q_db(client, model: str, image):
    import numpy as np
    import tritonclient.grpc as grpcclient

    tile = np.ascontiguousarray(image, dtype=np.uint8)
    p_a62 = grpcclient.InferInput("tile", list(tile.shape), "UINT8")
    p_a62.set_data_from_numpy(tile)
    p_a63 = [grpcclient.InferRequestedOutput(n) for n in ("boxes", "texts", "confidences")]
    p_a64 = client.infer(model_name=model, inputs=[p_a62], outputs=p_a63)
    boxes = p_a64.as_numpy("boxes")
    texts = p_a64.as_numpy("texts")
    confs = p_a64.as_numpy("confidences")
    boxes = [] if boxes is None else boxes.tolist()
    texts = [] if texts is None else [_as_text(t) for t in texts]
    confs = [] if confs is None else [float(p_a61) for p_a61 in confs]
    return boxes, texts, confs

def main(argv=None) -> int:
    p_a65 = argparse.ArgumentParser(description="OCR một PDF qua ocr_pipeline (xem output model)")
    p_a65.add_argument("pdf", help="đường dẫn file PDF")
    p_a65.add_argument("--url", default="localhost:8001", help="Triton gRPC (loopback)")
    p_a65.add_argument("--model", default="ocr_pipeline")
    p_a65.add_argument("--dpi", type=int, default=200, help="độ phân giải render trang")
    p_a65.add_argument("--pages", default=None, help="ví dụ 1-3,5 (1-based); mặc định tất cả")
    p_a65.add_argument("--min-conf", type=float, default=0.0, help="ẩn dòng dưới ngưỡng conf")
    p_a66 = p_a65.parse_args(argv)

    import tritonclient.grpc as grpcclient

    p_a6a = _render(p_a66.pdf, p_a66.dpi)
    p_a69 = q_138(p_a66.pages, len(p_a6a))
    client = grpcclient.InferenceServerClient(url=p_a66.url)

    p_a6e = 0
    p_a68 = 0.0
    for p_a6d in p_a69:
        boxes, texts, confs = q_db(client, p_a66.model, p_a6a[p_a6d])
        p_a6c = q_149(boxes, texts, confs)
        print(f"\n===== TRANG {p_a6d + 1} — {len(p_a6c)} dòng (dpi={p_a66.dpi}) =====")
        for t, p_a67 in p_a6c:
            if p_a67 < p_a66.min_conf:
                continue
            print(f"[{p_a67:5.3f}] {t}")
            p_a6e += 1
            p_a68 += p_a67

    p_a6b = p_a68 / p_a6e if p_a6e else 0.0
    print(f"\n----- TỔNG: {p_a6e} dòng · conf TB {p_a6b:.3f} · "
          f"engine=rec_vietocr v4 (det_dbnet→cls_angle→rec_vietocr) -----")
    return 0

if __name__ == "__main__":
    sys.exit(main())
