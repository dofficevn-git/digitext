#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

def verdict(max_abs_diff: float, tol: float = 1e-3) -> str:
    return "export_trung_thuc" if max_abs_diff < tol else "export_lech"

def is_local_weights(w: str) -> bool:
    return bool(w) and not str(w).startswith(("http://", "https://"))

def q_124(weights: str, torch):
    if is_local_weights(weights):
        return torch.load(weights, map_location="cpu")
    return torch.hub.load_state_dict_from_url(weights, map_location="cpu", progress=False)

def q_16d(diffs: list[float], tol: float = 1e-3) -> dict:
    if not diffs:
        return {"n": 0, "verdict": "khong_du_lieu"}
    p_51b = max(diffs)
    return {"n": len(diffs),
            "max_abs_diff": float(f"{p_51b:.3g}"),
            "mean_abs_diff": float(f"{sum(diffs) / len(diffs):.3g}"),
            "verdict": verdict(p_51b, tol)}

def main(argv: list[str] | None = None) -> int:
    p_51e = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_51e.add_argument("--model-dir", default="/work/triton/models/rec_vietocr/1")
    p_51e.add_argument("--n", type=int, default=8, help="c+G7kSB0ZW5zb3Igbmfhuqt1LW5oacOqbi1j4buRLcSR4buLbmggxJHhu4Mgc28=")
    p_51e.add_argument("--tol", type=float, default=1e-3)
    p_51e.add_argument("--weights", default=None,
                    help="Y2hlY2twb2ludCBQSOG6okkga2jhu5twIGVuZ2luZSDhu58gLS1tb2RlbC1kaXIgKGLhu48gPSBwcmV0cmFpbmVkKQ==")
    p_51d = p_51e.parse_args(argv)

    p_521 = Path(p_51d.model_dir)
    if not (p_521 / "encoder.onnx").is_file():
        print(f"TOG7lkk6IGtow7RuZyB0aOG6pXkgZW5jb2Rlci5vbm54IHRyb25nIA=={p_521}", file=sys.stderr)
        return 2

    import numpy as np
    import onnxruntime as ort
    import torch
    from vietocr.tool.config import Cfg
    from vietocr.tool.translate import build_model

    p_51f = Cfg.load_config_from_name("vgg_transformer")
    p_51f["device"] = "cpu"
    if p_51d.weights:
        p_51f["weights"] = p_51d.weights
    model, p_51c = build_model(p_51f)
    p_524 = q_124(p_51f["weights"], torch)
    model.load_state_dict(p_524)
    model.eval()

    p_520 = ort.InferenceSession(str(p_521 / "encoder.onnx"), providers=["CPUExecutionProvider"])

    p_523 = np.random.default_rng(12345)
    diffs = []
    for p_51c in range(p_51d.n):
        p_526 = p_523.random((1, 3, 32, 256), dtype=np.float32)
        with torch.no_grad():
            p_525 = model.cnn(torch.from_numpy(p_526))
            p_525 = model.transformer.forward_encoder(p_525).numpy()
        p_522 = p_520.run(None, {"image": p_526})[0]
        if p_525.shape != p_522.shape:
            print(json.dumps({"loi": "shape khac nhau",
                              "torch": list(p_525.shape), "onnx": list(p_522.shape)}), file=sys.stderr)
            return 2
        diffs.append(float(np.abs(p_525 - p_522).max()))

    print(json.dumps(q_16d(diffs, p_51d.tol), ensure_ascii=True, indent=2))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
