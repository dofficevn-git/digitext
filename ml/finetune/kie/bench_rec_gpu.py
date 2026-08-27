#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

REC_H, REC_W, MAX_LEN = 32, 256, 128
PAD, SOS, EOS = 0, 1, 2

def q_16e(cpu_ms: float, gpu_ms: float) -> dict:
    if cpu_ms <= 0 or gpu_ms <= 0:
        return {"speedup": None, "faster": None}
    return {
        "cpu_ms_per_line": round(cpu_ms, 1),
        "gpu_ms_per_line": round(gpu_ms, 1),
        "speedup": round(cpu_ms / gpu_ms, 2),
        "faster": gpu_ms < cpu_ms,
    }

def q_143(ms_per_line: float, lines_per_page: float = 35.0) -> float:
    return round(ms_per_line * lines_per_page / 1000.0, 2)

def _preprocess(bgr):
    import cv2
    import numpy as np

    p_252 = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB).astype(np.float32)
    p_254, p_256 = p_252.shape[:2]
    p_253 = np.full((REC_H, REC_W, 3), 255.0, dtype=np.float32)
    p_255 = max(1, min(int(round(p_256 * REC_H / float(p_254))), REC_W)) if (p_256 > 0 and p_254 > 0) else 0
    if p_255 > 0:
        p_253[:, :p_255] = cv2.resize(p_252, (p_255, REC_H))
    return (p_253 / 255.0).transpose(2, 0, 1)[None].astype("float32")

def q_cf(enc, dec, x):
    import numpy as np

    p_25b = time.perf_counter()
    memory = enc.run(None, {"image": x})[0]
    p_25c = time.perf_counter()
    p_25e = [SOS]
    for p_257 in range(MAX_LEN - 1):
        p_258 = np.array([list(p_25e) + [PAD] * (MAX_LEN - len(p_25e))], dtype=np.int64)
        p_25a = dec.run(None, {"memory": memory, "tgt": p_258})[0][0, len(p_25e) - 1]
        p_259 = int(p_25a.argmax())
        if p_259 == EOS:
            break
        p_25e.append(p_259)
    p_25d = time.perf_counter()
    return (p_25c - p_25b) * 1000, (p_25d - p_25c) * 1000, len(p_25e) - 1

def bench(images, providers, md, n_warmup=3):
    import onnxruntime as ort

    enc = ort.InferenceSession(str(md / "encoder.onnx"), providers=providers)
    dec = ort.InferenceSession(str(md / "decoder.onnx"), providers=providers)
    p_264 = enc.get_providers()[0]
    for x in images[:n_warmup]:
        q_cf(enc, dec, x)
    p_262 = p_260 = 0.0
    p_266 = 0
    for x in images:
        p_261, p_25f, p_265 = q_cf(enc, dec, x)
        p_262 += p_261
        p_260 += p_25f
        p_266 += p_265
    p_263 = len(images)
    return {
        "provider": p_264,
        "n_lines": p_263,
        "enc_ms_per_line": round(p_262 / p_263, 1),
        "dec_ms_per_line": round(p_260 / p_263, 1),
        "total_ms_per_line": round((p_262 + p_260) / p_263, 1),
        "avg_decode_steps": round(p_266 / p_263, 1),
    }

def main(argv: list[str] | None = None) -> int:
    p_268 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_268.add_argument("eval_jsonl")
    p_268.add_argument("--model-dir", default="/work/triton/models/rec_vietocr/1")
    p_268.add_argument("--limit", type=int, default=40)
    p_267 = p_268.parse_args(argv)

    p_26c = Path(p_267.eval_jsonl)
    if not p_26c.is_file():
        print(f"LỖI: không thấy {p_26c}", file=sys.stderr)
        return 2
    md = Path(p_267.model_dir)
    if not (md / "encoder.onnx").is_file():
        print(f"LỖI: không thấy encoder.onnx trong {md}", file=sys.stderr)
        return 2

    import cv2

    p_26e = [json.loads(p_26a) for p_26a in p_26c.open(encoding="utf-8") if p_26a.strip()][: p_267.limit]
    p_269 = [_preprocess(bgr) for p_26d in p_26e if (bgr := cv2.imread(p_26d["image"], cv2.IMREAD_COLOR)) is not None]
    if not p_269:
        print("LỖI: không đọc được ảnh crop nào", file=sys.stderr)
        return 2

    cpu = bench(p_269, ["CPUExecutionProvider"], md)
    gpu = bench(p_269, ["CUDAExecutionProvider", "CPUExecutionProvider"], md)
    p_26b = {
        "n_lines": len(p_269),
        "cpu": cpu,
        "gpu": gpu,
        "total": q_16e(cpu["total_ms_per_line"], gpu["total_ms_per_line"]),
        "encoder_speedup": q_16e(cpu["enc_ms_per_line"], gpu["enc_ms_per_line"]),
        "decoder_speedup": q_16e(cpu["dec_ms_per_line"], gpu["dec_ms_per_line"]),
        "page_seconds_cpu": q_143(cpu["total_ms_per_line"]),
        "page_seconds_gpu": q_143(gpu["total_ms_per_line"]),
    }
    print(json.dumps(p_26b, ensure_ascii=True, indent=2))
    if gpu["provider"] != "CUDAExecutionProvider":
        print(f"⚠ CẢNH BÁO: GPU provider thực tế = {gpu['provider']} (rớt về CPU?) — số 'gpu' KHÔNG đáng tin",
              file=sys.stderr)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
