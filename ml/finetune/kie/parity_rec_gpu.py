#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import sys
import unicodedata
from pathlib import Path

REC_H, REC_W, WIDE_REC_W, MAX_LEN = 32, 256, 512, 128
PAD, SOS, EOS = 0, 1, 2

def q_de(a: str, b: str) -> int:
    if a == b:
        return 0
    if len(a) < len(b):
        a, b = b, a
    p_4c2 = list(range(len(b) + 1))
    for p_4c0, p_4bd in enumerate(a, 1):
        p_4bf = [p_4c0]
        for p_4c1, p_4be in enumerate(b, 1):
            p_4bf.append(min(p_4c2[p_4c1] + 1, p_4bf[p_4c1 - 1] + 1, p_4c2[p_4c1 - 1] + (p_4bd != p_4be)))
        p_4c2 = p_4bf
    return p_4c2[-1]

def q_103(ref: str, hyp: str) -> float:
    if not ref:
        return 0.0 if not hyp else 1.0
    return q_de(ref, hyp) / len(ref)

def q_108(cpu_texts: list[str], gpu_texts: list[str]) -> dict:
    n = min(len(cpu_texts), len(gpu_texts))
    if n == 0:
        return {"n": 0, "safe_to_deploy": False}
    p_4c3 = [q_103(cpu_texts[p_4c5], gpu_texts[p_4c5]) for p_4c5 in range(n)]
    p_4c4 = sum(1 for p_4c5 in range(n) if cpu_texts[p_4c5] == gpu_texts[p_4c5])
    p_4c6 = sum(p_4c3) / n
    return {
        "n": n,
        "exact_match": p_4c4,
        "exact_match_rate": round(p_4c4 / n, 4),
        "mismatch_lines": n - p_4c4,
        "mean_char_disagreement": round(p_4c6, 5),
        "max_char_disagreement": round(max(p_4c3), 4),
        "safe_to_deploy": p_4c4 / n >= 0.95 and p_4c6 < 0.005,
    }

def q_bf(w, h):
    if w <= 0 or h <= 0:
        return REC_W
    return WIDE_REC_W if w / float(h) > 12.0 else REC_W

def _preprocess(bgr):
    import cv2
    import numpy as np

    a = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB).astype(np.float32)
    h, w = a.shape[:2]
    rec_w = q_bf(w, h)
    p_4c7 = np.full((REC_H, rec_w, 3), 255.0, dtype=np.float32)
    p_4c8 = max(1, min(int(round(w * REC_H / float(h))), rec_w)) if (w > 0 and h > 0) else 0
    if p_4c8 > 0:
        p_4c7[:, :p_4c8] = cv2.resize(a, (p_4c8, REC_H))
    x = (p_4c7 / 255.0).transpose(2, 0, 1)[None]
    return np.ascontiguousarray(x, dtype="float32"), rec_w

def _decode(sessions, x, rec_w, vocab):
    import numpy as np

    p_4cc, p_4cb = sessions[rec_w]
    memory = p_4cc.run(None, {"image": x})[0]
    tokens = [SOS]
    for p_4c9 in range(MAX_LEN - 1):
        p_4ca = np.array([q_e4(tokens)], dtype=np.int64)
        p_4cd = int(p_4cb.run(None, {"memory": memory, "tgt": p_4ca})[0][0, len(tokens) - 1].argmax())
        if p_4cd == EOS:
            break
        tokens.append(p_4cd)
    return unicodedata.normalize("NFC", "".join(vocab[p_4ce] for p_4ce in tokens[1:] if 4 <= p_4ce < len(vocab)))

def q_e4(tokens):
    p_4cf = list(tokens[:MAX_LEN])
    return p_4cf + [PAD] * (MAX_LEN - len(p_4cf))

def q_e0(md, providers):
    import onnxruntime as ort

    p_4d2 = {}
    for w, p_4d3 in ((REC_W, "w256"), (WIDE_REC_W, "w512")):
        p_4d0 = md / p_4d3
        p_4d2[w] = (
            ort.InferenceSession(str(p_4d0 / "encoder.onnx"), providers=providers),
            ort.InferenceSession(str(p_4d0 / "decoder.onnx"), providers=providers),
        )
    p_4d1 = p_4d2[REC_W][0].get_providers()[0]
    return p_4d2, p_4d1

def q_cc(md, providers, xs, vocab):
    p_4d5, p_4d4 = q_e0(md, providers)
    return [_decode(p_4d5, x, rec_w, vocab) for x, rec_w in xs], p_4d4

def main(argv: list[str] | None = None) -> int:
    p_4d7 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_4d7.add_argument("eval_jsonl")
    p_4d7.add_argument("--model-dir", default="/work/triton/models/rec_vietocr/3")
    p_4d7.add_argument("--limit", type=int, default=60)
    a = p_4d7.parse_args(argv)

    p_4d9, md = Path(a.eval_jsonl), Path(a.model_dir)
    if not p_4d9.is_file():
        print(f"TOG7lkk6IGtow7RuZyB0aOG6pXkg{p_4d9}", file=sys.stderr)
        return 2
    if not (md / "w256" / "encoder.onnx").is_file():
        print(f"TOG7lkk6IGtow7RuZyB0aOG6pXkgdzI1Ni9lbmNvZGVyLm9ubnggdHJvbmcg{md}", file=sys.stderr)
        return 2

    import cv2

    vocab = [p_4d8.rstrip("\n") for p_4d8 in (md / "vocab.txt").open(encoding="utf-8")]
    p_4dc = [json.loads(p_4d8) for p_4d8 in p_4d9.open(encoding="utf-8") if p_4d8.strip()][: a.limit]
    xs = [_preprocess(bgr) for p_4da in p_4dc if (bgr := cv2.imread(p_4da["image"], cv2.IMREAD_COLOR)) is not None]
    if not xs:
        print("TOG7lkk6IGtow7RuZyDEkeG7jWMgxJHGsOG7o2Mg4bqjbmggY3JvcCBuw6Bv", file=sys.stderr)
        return 2

    cpu_texts, p_4d6 = q_cc(md, ["CPUExecutionProvider"], xs, vocab)
    gpu_texts, p_4db = q_cc(md, ["CUDAExecutionProvider", "CPUExecutionProvider"], xs, vocab)
    p_4dd = q_108(cpu_texts, gpu_texts)
    p_4dd["gpu_provider"] = p_4db
    print(json.dumps(p_4dd, ensure_ascii=True, indent=2))
    if p_4db != "CUDAExecutionProvider":
        print(f"4pqgIEdQVSBy4bubdCB24buBIA=={p_4db}IOKAlCBLSMOUTkcga2nhu4NtIMSRxrDhu6NjIHBhcml0eSAoY+G6p24gb25ueHJ1bnRpbWUtZ3B1ICsgQ1VEQSBsaWJzKQ==",
              file=sys.stderr)
        return 3
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
