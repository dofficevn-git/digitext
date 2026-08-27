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
from probe_rec_preproc import q_179

REC_H, REC_W = 32, 256

def norm(s: str) -> str:
    return unicodedata.normalize("NFC", (s or "").strip())

def q_131(w: int, h: int, rec_h: int = REC_H, rec_w: int = REC_W) -> float:
    p_527 = q_179(w, h, rec_h, rec_w)
    if p_527 <= 0:
        return 0.0
    return 1.0 - p_527 / rec_w

def attribute(cer_native: float, cer_padded: float, cer_onnx: float) -> dict:
    p_529 = cer_onnx - cer_native
    p_528 = max(0.0, cer_padded - cer_native)
    return {"gap_tong": round(p_529, 4),
            "do_dem": round(min(p_528, p_529) if p_529 > 0 else 0.0, 4),
            "chua_giai_thich": round(max(0.0, p_529 - p_528), 4)}

def q_e7(det, images, pad_to_frame: bool):
    from PIL import Image

    p_52d = []
    for p_52e in images:
        p_52b = Image.open(p_52e).convert("RGB")
        if pad_to_frame:
            w, h = p_52b.size
            p_52c = q_179(w, h)
            p_52a = Image.new("RGB", (REC_W, REC_H), (255, 255, 255))
            if p_52c > 0:
                p_52a.paste(p_52b.resize((p_52c, REC_H), Image.LANCZOS), (0, 0))
            p_52b = p_52a
        p_52d.append(det.predict(p_52b))
    return p_52d

def main(argv: list[str] | None = None) -> int:
    p_530 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_530.add_argument("eval_jsonl")
    p_52f = p_530.parse_args(argv)

    p_536 = Path(p_52f.eval_jsonl)
    if not p_536.is_file():
        print(f"LỖI: không thấy {p_536}", file=sys.stderr)
        return 2
    p_53a = [json.loads(p_534) for p_534 in p_536.open(encoding="utf-8") if p_534.strip()]
    if not p_53a:
        print("LỖI: eval rỗng", file=sys.stderr)
        return 2

    from vietocr.tool.config import Cfg
    from vietocr.tool.predictor import Predictor

    p_532 = Cfg.load_config_from_name("vgg_transformer")
    p_532["device"] = "cpu"
    p_532["predictor"]["beamsearch"] = False
    det = Predictor(p_532)

    p_533 = [p_538["image"] for p_538 in p_53a]
    p_53c = [p_538["truth"] for p_538 in p_53a]
    p_539 = {}
    for p_535, p_537 in (("torch_native_khong_dem", False), ("torch_ep_khung_256_co_dem", True)):
        p_531 = q_10a([(norm(p_53d), norm(p_53b))
                        for p_53d, p_53b in zip(q_e7(det, p_533, p_537), p_53c)])
        p_539[p_535] = {"cer_micro": round(p_531["cer_micro"], 4),
                     "exact": round(p_531["exact_match_rate"], 3)}

    print(json.dumps({"n": len(p_53a), **p_539}, ensure_ascii=True, indent=2))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
