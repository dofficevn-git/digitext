#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import math
import sys
import unicodedata
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from cer import q_10a

REC_H, REC_W, MAX_LEN = 32, 256, 128
PAD, SOS, EOS = 0, 1, 2
MODEL_DIR = Path("/work/triton/models/rec_vietocr/1")

def norm(s: str) -> str:
    return unicodedata.normalize("NFC", (s or "").strip())

def q_179(w: int, h: int, rec_h: int = REC_H, rec_w: int = REC_W) -> int:
    if w <= 0 or h <= 0:
        return 0
    return max(1, min(int(round(w * rec_h / float(h))), rec_w))

def q_175(w: int, h: int, rec_h: int = REC_H, min_w: int = 32,
                max_w: int = 512, round_to: int = 10) -> int:
    if w <= 0 or h <= 0:
        return 0
    p_559 = int(rec_h * float(w) / float(h))
    p_559 = math.ceil(p_559 / round_to) * round_to
    return min(max(p_559, min_w), max_w)

def summarize(variants: dict[str, list[str]], truths: list[str]) -> dict:
    p_55c = {}
    for p_55b, p_55e in variants.items():
        p_55a = q_10a([(norm(p_55d), norm(p_55f)) for p_55d, p_55f in zip(p_55e, truths)])
        p_55c[p_55b] = {"cer_micro": round(p_55a["cer_micro"], 4),
                     "exact": round(p_55a["exact_match_rate"], 3)}
    return p_55c

def q_be(bgr, interp):
    import cv2
    import numpy as np

    p_560 = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB).astype(np.float32)
    h, w = p_560.shape[:2]
    p_561 = np.full((REC_H, REC_W, 3), 255.0, dtype=np.float32)
    p_562 = q_179(w, h)
    if p_562 > 0:
        p_561[:, :p_562] = cv2.resize(p_560, (p_562, REC_H), interpolation=interp)
    return (p_561 / 255.0).transpose(2, 0, 1)[None].astype("float32")

def q_bd(bgr):
    import numpy as np
    from PIL import Image

    import cv2
    p_566 = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB)
    h, w = p_566.shape[:2]
    p_565 = min(q_175(w, h), REC_W)
    p_564 = Image.fromarray(p_566).resize((max(p_565, 1), REC_H), Image.LANCZOS)
    p_563 = np.full((REC_H, REC_W, 3), 255.0, dtype=np.float32)
    p_563[:, :p_565] = np.asarray(p_564, dtype=np.float32)
    return (p_563 / 255.0).transpose(2, 0, 1)[None].astype("float32")

def _decode(enc, dec, vocab, x):
    import numpy as np

    memory = enc.run(None, {"image": x})[0]
    p_56d = [SOS]
    for p_567 in range(MAX_LEN - 1):
        p_568 = np.array([list(p_56d[:MAX_LEN]) + [PAD] * (MAX_LEN - len(p_56d))], dtype=np.int64)
        p_56c = dec.run(None, {"memory": memory, "tgt": p_568})[0][0, len(p_56d) - 1]
        p_56b = int(p_56c.argmax())
        if p_56b == EOS:
            break
        p_56d.append(p_56b)
    p_569 = [vocab[p_56a - 4] for p_56a in p_56d[1:] if 4 <= p_56a < len(vocab) + 4]
    return unicodedata.normalize("NFC", "".join(p_569))

def main(argv: list[str] | None = None) -> int:
    p_56f = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_56f.add_argument("eval_jsonl")
    p_56f.add_argument("--model-dir", default=str(MODEL_DIR))
    p_56e = p_56f.parse_args(argv)

    p_574 = Path(p_56e.eval_jsonl)
    if not p_574.is_file():
        print(f"LỖI: không thấy {p_574}", file=sys.stderr)
        return 2
    p_576 = [json.loads(p_572) for p_572 in p_574.open(encoding="utf-8") if p_572.strip()]
    if not p_576:
        print("LỖI: eval rỗng", file=sys.stderr)
        return 2

    import cv2
    import onnxruntime as ort

    p_573 = Path(p_56e.model_dir)
    enc = ort.InferenceSession(str(p_573 / "encoder.onnx"), providers=["CPUExecutionProvider"])
    dec = ort.InferenceSession(str(p_573 / "decoder.onnx"), providers=["CPUExecutionProvider"])
    vocab = [p_572.rstrip("\n") for p_572 in (p_573 / "vocab.txt").open(encoding="utf-8")][4:]

    p_571 = [cv2.imread(p_575["image"], cv2.IMREAD_COLOR) for p_575 in p_576]
    variants = {
        "A_hien_tai_bilinear": [_decode(enc, dec, vocab, q_be(p_570, cv2.INTER_LINEAR))
                                for p_570 in p_571 if p_570 is not None],
        "B_doi_sang_INTER_AREA": [_decode(enc, dec, vocab, q_be(p_570, cv2.INTER_AREA))
                                  for p_570 in p_571 if p_570 is not None],
        "C_kieu_torch_LANCZOS": [_decode(enc, dec, vocab, q_bd(p_570))
                                 for p_570 in p_571 if p_570 is not None],
    }
    truths = [p_575["truth"] for p_575, p_570 in zip(p_576, p_571) if p_570 is not None]
    print(json.dumps({"n": len(truths), **summarize(variants, truths)},
                     ensure_ascii=True, indent=2))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
