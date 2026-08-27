#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import sys
import unicodedata
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from cer import q_10a

def norm(s: str) -> str:
    return unicodedata.normalize("NFC", (s or "").strip())

def score(preds: list[str], truths: list[str]) -> dict:
    return q_10a([(norm(p_35c), norm(p_35d)) for p_35c, p_35d in zip(preds, truths)])

def q_16f(tag: str, vie_cer: float, tri_cer: float | None) -> str:
    s = f"# VietOCR {tag}: CER micro {vie_cer:.4f}"
    return s + (f"IHwgVHJpdG9uIG3hu5FjIA=={tri_cer:.4f}" if tri_cer is not None else " | (khong co moc Triton)")

def q_15e(images: list[Path], weights: str | None) -> list[str]:
    from PIL import Image
    from vietocr.tool.config import Cfg
    from vietocr.tool.predictor import Predictor

    p_35e = Cfg.load_config_from_name("vgg_transformer")
    if weights:
        p_35e["weights"] = weights
    p_35e["device"] = "cuda:0"
    p_35e["predictor"]["beamsearch"] = False
    p_35f = Predictor(p_35e)
    return [p_35f.predict(Image.open(p_360).convert("RGB")) for p_360 in images]

def main(argv: list[str] | None = None) -> int:
    p_362 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_362.add_argument("eval_jsonl", help="eval_hard.jsonl (image, truth, ocr_reading)")
    p_362.add_argument("--weights", default=None, help="d2VpZ2h0cyBWaWV0T0NSIGZpbmUtdHVuZSAoYuG7jyA9IGJhc2Up")
    p_362.add_argument("--img-root", default=".")
    p_361 = p_362.parse_args(argv)

    p_367 = [json.loads(p_364) for p_364 in open(p_361.eval_jsonl, encoding="utf-8") if p_364.strip()]
    if not p_367:
        print("TOG7lkk6IGV2YWwgcuG7l25n", file=sys.stderr)
        return 2
    root = Path(p_361.img_root)
    p_363 = [(Path(p_366["image"]) if Path(p_366["image"]).is_absolute() else root / p_366["image"]) for p_366 in p_367]
    truths = [p_366["truth"] for p_366 in p_367]

    preds = q_15e(p_363, p_361.weights)
    p_369 = score(preds, truths)

    p_365 = {
        "n": len(p_367),
        "vietocr_" + ("finetuned" if p_361.weights else "base"): {
            "cer_micro": round(p_369["cer_micro"], 4), "cer_macro": round(p_369["cer_macro"], 4),
            "exact": round(p_369["exact_match_rate"], 3)},
    }
    tri_cer = None
    if any(p_366.get("ocr_reading") for p_366 in p_367):
        p_368 = score([p_366.get("ocr_reading", "") for p_366 in p_367], truths)
        tri_cer = p_368["cer_micro"]
        p_365["triton_baseline"] = {"cer_micro": round(tri_cer, 4)}
    tag = f"fine-tuned ({p_361.weights})" if p_361.weights else "base"
    print(json.dumps(p_365, ensure_ascii=True, indent=2))
    print(q_16f(tag, p_369["cer_micro"], tri_cer), file=sys.stderr)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
