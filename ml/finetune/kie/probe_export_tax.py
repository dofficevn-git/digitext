#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import unicodedata
from pathlib import Path

REC_H, REC_W, MAX_LEN = 32, 256, 128
PAD, SOS, EOS = 0, 1, 2

def norm(s: str) -> str:
    return unicodedata.normalize("NFC", (s or "").strip())

def attribute(ladder: list[tuple[str, float]]) -> list[dict]:
    p_4fa: list[dict] = []
    p_4fb: float | None = None
    for p_4f9, cer in ladder:
        p_4fa.append({"buoc": p_4f9, "cer": round(cer, 4),
                    "delta": None if p_4fb is None else round(cer - p_4fb, 4)})
        p_4fb = cer
    return p_4fa

def q_162(rows: list[dict]) -> list[dict]:
    p_4fd = sum(abs(p_4fc["delta"]) for p_4fc in rows if p_4fc["delta"] is not None)
    for p_4fc in rows:
        p_4fc["phan_tram"] = None if p_4fc["delta"] is None or p_4fd == 0 else round(
            100.0 * p_4fc["delta"] / p_4fd, 1)
    return rows

def q_bc(bgr, np, cv2):
    p_501 = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB).astype(np.float32)
    p_4ff, p_502 = p_501.shape[:2]
    p_4fe = np.full((REC_H, REC_W, 3), 255.0, dtype=np.float32)
    p_500 = max(1, min(int(round(p_502 * REC_H / float(p_4ff))), REC_W)) if p_4ff > 0 and p_502 > 0 else 0
    if p_500 > 0:
        p_4fe[:, :p_500] = cv2.resize(p_501, (p_500, REC_H))
    return (p_4fe / 255.0).transpose(2, 0, 1)[None]

def q_cd(model, x, torch, vocab):
    with torch.no_grad():
        p_504 = model.transformer.forward_encoder(model.cnn(x))
        p_508 = [SOS]
        for p_503 in range(MAX_LEN - 1):
            p_507 = torch.LongTensor([p_508]).transpose(0, 1)
            p_506, p_503 = model.transformer.forward_decoder(p_507, p_504)
            p_505 = int(p_506[0, -1].argmax())
            if p_505 == EOS:
                break
            p_508.append(p_505)
    return q_c1(p_508[1:], vocab)

def q_ce(model, x, torch, vocab):
    with torch.no_grad():
        p_50b = model.transformer.forward_encoder(model.cnn(x))
        p_50f = [SOS]
        for p_509 in range(MAX_LEN - 1):
            p_50a = p_50f + [PAD] * (MAX_LEN - len(p_50f))
            p_50e = torch.LongTensor([p_50a]).transpose(0, 1)
            p_50d, p_509 = model.transformer.forward_decoder(p_50e, p_50b)
            p_50c = int(p_50d[0, len(p_50f) - 1].argmax())
            if p_50c == EOS:
                break
            p_50f.append(p_50c)
    return q_c1(p_50f[1:], vocab)

def q_c1(ids, vocab):
    return norm("".join(vocab.i2c.get(p_510, "") for p_510 in ids if p_510 > EOS + 1))

def main(argv: list[str] | None = None) -> int:
    import cv2
    import numpy as np
    import torch
    from cer import q_10a
    from vietocr.tool.config import Cfg
    from vietocr.tool.predictor import Predictor
    from PIL import Image

    p_512 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_512.add_argument("rows_jsonl")
    p_512.add_argument("--weights", required=True)
    p_512.add_argument("--limit", type=int, default=400, help="số dòng (giải mã torch rất chậm)")
    p_511 = p_512.parse_args(argv)

    rows = [json.loads(p_514) for p_514 in open(p_511.rows_jsonl, encoding="utf-8") if p_514.strip()]
    rows = rows[: p_511.limit]

    p_513 = Cfg.load_config_from_name("vgg_transformer")
    p_513["device"] = "cpu"
    p_513["predictor"]["beamsearch"] = False
    p_513["weights"] = p_511.weights
    p_519 = Predictor(p_513)
    model, vocab = p_519.model.eval(), p_519.vocab

    p_515, p_516, p_517 = [], [], []
    for p_51a in rows:
        p_518 = Path(p_51a["image"])
        truth = norm(p_51a["truth"])
        p_515.append((norm(p_519.predict(Image.open(p_518).convert("RGB"))), truth))
        x = torch.from_numpy(q_bc(cv2.imread(str(p_518), cv2.IMREAD_COLOR), np, cv2))
        p_516.append((q_cd(model, x, torch, vocab), truth))
        p_517.append((q_ce(model, x, torch, vocab), truth))

    ladder = [("1_predictor_goc", q_10a(p_515)["cer_micro"]),
              ("2_tienxuly_modelpy", q_10a(p_516)["cer_micro"]),
              ("3_giaima_cuaso_tinh", q_10a(p_517)["cer_micro"])]
    print(json.dumps({"n": len(rows), "thang": q_162(attribute(ladder))},
                     ensure_ascii=True, indent=2))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
