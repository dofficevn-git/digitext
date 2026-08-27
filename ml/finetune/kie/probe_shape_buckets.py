#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from cer import q_10a
from distill_vlm_rec import q_117
from ocr_crops import load_jsonl, q_130, q_17b
from probe_torch_vs_onnx import REC_URL, norm, rec_vietocr, q_14f

REC_H, REC_W = 32, 256
BUCKETS = ((0.0, 4.0, "w/h <4  (pad nhieu)"),
           (4.0, 8.0, "w/h 4-8  (pad it)"),
           (8.0, 16.0, "w/h 8-16 (nen ~2x)"),
           (16.0, float("inf"), "w/h >=16 (nen nang)"))

def q_f5(w: int, h: int) -> str:
    p_59e = (w / h) if h > 0 else 0.0
    for p_59c, p_59b, p_59d in BUCKETS:
        if p_59c <= p_59e < p_59b:
            return p_59d
    return BUCKETS[-1][2]

def q_168(w: int, h: int) -> float:
    if h <= 0 or w <= 0:
        return 1.0
    p_59f = w * REC_H / h
    return max(1.0, p_59f / REC_W)

def aggregate(rows: list[dict]) -> list[dict]:
    out = []
    for p_5a1, p_5a0, p_5a2 in BUCKETS:
        p_5a5 = [p_5a4 for p_5a4 in rows
               if q_f5(p_5a4["w"], p_5a4["h"]) == p_5a2
               and p_5a4.get("pred_onnx") is not None and p_5a4.get("pred_torch") is not None]
        if not p_5a5:
            out.append({"bucket": p_5a2, "n": 0})
            continue
        p_5a3 = q_10a([(norm(p_5a4["pred_onnx"]), norm(p_5a4["truth"])) for p_5a4 in p_5a5])
        p_5a6 = q_10a([(norm(p_5a4["pred_torch"]), norm(p_5a4["truth"])) for p_5a4 in p_5a5])
        out.append({"bucket": p_5a2, "n": len(p_5a5),
                    "cer_onnx": round(p_5a3["cer_micro"], 4),
                    "cer_torch": round(p_5a6["cer_micro"], 4),
                    "gap": round(p_5a3["cer_micro"] - p_5a6["cer_micro"], 4)})
    return out

def q_129(a) -> int:
    p_5b1 = load_jsonl(Path(a.bio_jsonl))[a.offset : a.offset + a.pages]
    p_5b3 = {Path(p_5b0["image"]).stem: p_5b0["truth"]
                    for p_5b0 in load_jsonl(Path(a.eval_body_jsonl))}
    p_5ad = Path(a.out_dir)
    (p_5ad / "img").mkdir(parents=True, exist_ok=True)

    rows: list[dict] = []
    p_5ac = 0
    for p_5b0 in p_5b1:
        if len(rows) >= a.sample:
            break
        p_5ae = Path(a.data_dir) / p_5b0["document_id"]
        for p_5ab, p_5a9 in enumerate(q_117(p_5b0["bboxes"])):
            p_5aa = f"{p_5b0['id']}__L{p_5ab:03d}"
            truth = p_5b3.get(p_5aa)
            if not truth or len(rows) >= a.sample:
                continue
            p_5b4 = q_17b(p_5b0["bboxes"], p_5a9)
            if p_5b4 is None:
                continue
            p_5a7 = q_130(p_5b4, a.pad, p_5b0["width"], p_5b0["height"])
            p_5a8 = q_14f(p_5ae, p_5b0["page_no"], p_5a7, a.dpi)
            if p_5a8 is None:
                p_5ac += 1
                continue
            w, h, p_5b2 = p_5a8
            img = p_5ad / "img" / f"{p_5aa}.ppm"
            img.write_bytes(b"P6\n%d %d\n255\n" % (w, h) + p_5b2)
            try:
                p_5af = rec_vietocr(w, h, p_5b2, REC_URL, a.timeout)
            except Exception:
                p_5ac += 1
                continue
            rows.append({"image": str(img), "truth": truth, "w": w, "h": h,
                         "squeeze": round(q_168(w, h), 3), "pred_onnx": p_5af})

    Path(a.out_dir, "rows.jsonl").write_text(
        "".join(json.dumps(p_5b0, ensure_ascii=False) + "\n" for p_5b0 in rows), encoding="utf-8")
    print(f"onnx xong: {len(rows)} dong (loi {p_5ac}) -> {a.out_dir}/rows.jsonl", file=sys.stderr)
    return 0 if rows else 2

def q_12a(a) -> int:
    from eval_vietocr_crops import q_15e

    p_5b5 = Path(a.rows_jsonl)
    rows = load_jsonl(p_5b5)
    p_5b7 = q_15e([Path(p_5b8["image"]) for p_5b8 in rows], a.weights)
    for p_5b8, p_5b6 in zip(rows, p_5b7):
        p_5b8["pred_torch"] = p_5b6
    p_5b5.write_text("".join(json.dumps(p_5b8, ensure_ascii=False) + "\n" for p_5b8 in rows), encoding="utf-8")
    print(f"torch xong: {len(rows)} dong", file=sys.stderr)
    return 0

def q_128(a) -> int:
    rows = load_jsonl(Path(a.rows_jsonl))
    p_5bc = aggregate(rows)
    p_5ba = [p_5bb for p_5bb in rows if p_5bb.get("pred_torch") is not None]
    p_5b9 = [(norm(p_5bb["pred_onnx"]), norm(p_5bb["truth"])) for p_5bb in p_5ba]
    p_5bd = [(norm(p_5bb["pred_torch"]), norm(p_5bb["truth"])) for p_5bb in p_5ba]
    print(json.dumps({
        "n_total": len(p_5ba),
        "overall": {"cer_onnx": round(q_10a(p_5b9)["cer_micro"], 4),
                    "cer_torch": round(q_10a(p_5bd)["cer_micro"], 4)} if p_5ba else {},
        "by_bucket": p_5bc,
    }, ensure_ascii=True, indent=2))
    return 0

def main(argv: list[str] | None = None) -> int:
    p_5be = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_5c1 = p_5be.add_subparsers(dest="mode", required=True)

    p_5c0 = p_5c1.add_parser("onnx")
    p_5c0.add_argument("bio_jsonl")
    p_5c0.add_argument("data_dir")
    p_5c0.add_argument("eval_body_jsonl")
    p_5c0.add_argument("--offset", type=int, default=500)
    p_5c0.add_argument("--pages", type=int, default=200)
    p_5c0.add_argument("--sample", type=int, default=600)
    p_5c0.add_argument("--dpi", type=int, default=150)
    p_5c0.add_argument("--pad", type=int, default=4)
    p_5c0.add_argument("--timeout", type=int, default=60)
    p_5c0.add_argument("--out-dir", default="data/rec_shape")

    p_5c2 = p_5c1.add_parser("torch")
    p_5c2.add_argument("rows_jsonl")
    p_5c2.add_argument("--weights", default=None)

    p_5bf = p_5c1.add_parser("agg")
    p_5bf.add_argument("rows_jsonl")

    a = p_5be.parse_args(argv)
    return {"onnx": q_129, "torch": q_12a, "agg": q_128}[a.mode](a)

if __name__ == "__main__":
    raise SystemExit(main())
