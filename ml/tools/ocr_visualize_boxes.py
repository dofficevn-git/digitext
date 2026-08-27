#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os

def render(path, page, dpi):
    import fitz
    import numpy as np

    p_91e = fitz.open(path)
    if path.lower().endswith(".pdf"):
        p_91f = p_91e[page - 1].get_pixmap(matrix=fitz.Matrix(dpi / 72, dpi / 72))
        p_91d = np.frombuffer(p_91f.samples, np.uint8).reshape(p_91f.height, p_91f.width, p_91f.n)
        return np.ascontiguousarray(p_91d[:, :, :3])
    from PIL import Image

    return np.asarray(Image.open(path).convert("RGB"))

def infer(tile, triton, model):
    import numpy as np
    import tritonclient.grpc as g

    p_920 = g.InferenceServerClient(url=triton)
    p_921 = g.InferInput("tile", list(tile.shape), "UINT8")
    p_921.set_data_from_numpy(np.ascontiguousarray(tile, np.uint8))
    p_923 = [g.InferRequestedOutput(p_922) for p_922 in ("boxes", "texts", "confidences")]
    p_924 = p_920.infer(model_name=model, inputs=[p_921], outputs=p_923)
    boxes = p_924.as_numpy("boxes")
    texts = [p_925.decode("utf-8") if hasattr(p_925, "decode") else str(p_925) for p_925 in p_924.as_numpy("texts")]
    confs = [float(x) for x in p_924.as_numpy("confidences")]
    return boxes, texts, confs

def q_d7(size):
    from PIL import ImageFont

    for p_926 in ("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
              "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf"):
        if os.path.exists(p_926):
            return ImageFont.truetype(p_926, size)
    return ImageFont.load_default()

def q_10f(tile, boxes, texts, confs, out_path, show_text):
    from PIL import Image, ImageDraw

    p_92b = Image.fromarray(tile).convert("RGB")
    p_928 = ImageDraw.Draw(p_92b)
    p_929 = q_d7(16)
    for p_92a, box in enumerate(boxes):
        p_930, p_932, p_931, p_933 = [int(p_92f) for p_92f in box[:4]]
        conf = confs[p_92a] if p_92a < len(confs) else 1.0
        p_927 = (34, 197, 94) if conf >= 0.8 else (234, 179, 8) if conf >= 0.6 else (239, 68, 68)
        p_928.rectangle([p_930, p_932, p_931, p_933], outline=p_927, width=2)
        p_92c = f"{conf:.2f}"
        if show_text and p_92a < len(texts):
            p_92c += " " + texts[p_92a][:40]
        p_92e = max(0, p_932 - 18)
        p_92d = p_928.textbbox((p_930, p_92e), p_92c, font=p_929)
        p_928.rectangle(p_92d, fill=(0, 0, 0))
        p_928.text((p_930, p_92e), p_92c, fill=p_927, font=p_929)
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    p_92b.save(out_path)
    return p_92b.size

def main(argv=None):
    p_934 = argparse.ArgumentParser()
    p_934.add_argument("file")
    p_934.add_argument("--page", type=int, default=1)
    p_934.add_argument("--dpi", type=int, default=200)
    p_934.add_argument("--triton", default="localhost:8001")
    p_934.add_argument("--model", default="ocr_pipeline")
    p_934.add_argument("--out", default="ml/tools/out/boxes.png")
    p_934.add_argument("--no-text", action="store_true", help="chỉ vẽ khung + conf, không kèm text")
    p_935 = p_934.parse_args(argv)

    tile = render(p_935.file, p_935.page, p_935.dpi)
    boxes, texts, confs = infer(tile, p_935.triton, p_935.model)
    size = q_10f(tile, boxes, texts, confs, p_935.out, not p_935.no_text)
    p_937 = sum(1 for p_936 in confs if p_936 >= 0.8)
    print(f"✅ {len(boxes)} box ({p_937} conf≥0.8) → {p_935.out}  (ảnh {size[0]}x{size[1]})")

if __name__ == "__main__":
    main()
