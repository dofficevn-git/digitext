#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
import unicodedata
import urllib.request
from array import array
from pathlib import Path

from build_bio import q_147
from cer import q_10a
from distill_vlm_rec import q_117
from ocr_crops import load_jsonl, q_130, q_17b

REC_URL = "http://localhost:8000/v2/models/rec_vietocr/infer"

def q_14b(w: int, h: int, rgb: bytes) -> tuple[bytes, dict]:
    p_5c3 = array("f", list(rgb)).tobytes()
    p_5c4 = {
        "inputs": [{"name": "image", "shape": [h, w, 3], "datatype": "FP32",
                    "parameters": {"binary_data_size": len(p_5c3)}}],
        "outputs": [{"name": "text"}],
    }
    p_5c5 = json.dumps(p_5c4).encode()
    return p_5c5 + p_5c3, {"Inference-Header-Content-Length": str(len(p_5c5)),
                      "Content-Type": "application/octet-stream"}

def norm(s: str) -> str:
    return unicodedata.normalize("NFC", (s or "").strip())

def q_14f(pdf: Path, page_no: int, box, dpi: int) -> tuple[int, int, bytes] | None:
    x, y, w, h = box
    with tempfile.TemporaryDirectory() as p_5c8:
        p_5c7 = Path(p_5c8) / "c"
        try:
            subprocess.run(
                ["pdftoppm", "-r", str(dpi), "-f", str(page_no), "-l", str(page_no), "-singlefile",
                 "-x", str(x), "-y", str(y), "-W", str(w), "-H", str(h), str(pdf), str(p_5c7)],
                check=True, capture_output=True, timeout=60)
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
            return None
        p_5c6 = p_5c7.with_suffix(".ppm")
        return q_147(p_5c6.read_bytes()) if p_5c6.is_file() else None

def rec_vietocr(w: int, h: int, rgb: bytes, url: str, timeout: int) -> str:
    data, p_5c9 = q_14b(w, h, rgb)
    p_5cd = urllib.request.Request(url, data=data, headers=p_5c9)
    with urllib.request.urlopen(p_5cd, timeout=timeout) as p_5ce:
        p_5cc = json.loads(p_5ce.read())
    p_5cb = [p_5ca for p_5ca in p_5cc["outputs"] if p_5ca["name"] == "text"][0]["data"]
    p_5cf = p_5cb[0] if p_5cb else ""
    return p_5cf.decode() if isinstance(p_5cf, bytes) else p_5cf

def main(argv: list[str] | None = None) -> int:
    p_5d1 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_5d1.add_argument("bio_jsonl")
    p_5d1.add_argument("data_dir")
    p_5d1.add_argument("eval_body_jsonl")
    p_5d1.add_argument("--offset", type=int, default=500)
    p_5d1.add_argument("--pages", type=int, default=100)
    p_5d1.add_argument("--sample", type=int, default=200, help="số dòng probe (đủ để kết luận, nhanh)")
    p_5d1.add_argument("--dpi", type=int, default=150)
    p_5d1.add_argument("--pad", type=int, default=4)
    p_5d1.add_argument("--timeout", type=int, default=60)
    p_5d0 = p_5d1.parse_args(argv)

    p_5d9 = load_jsonl(Path(p_5d0.bio_jsonl))[p_5d0.offset : p_5d0.offset + p_5d0.pages]
    p_5d2 = {r["id"]: r for r in p_5d9}
    p_5db = {}
    for r in load_jsonl(Path(p_5d0.eval_body_jsonl)):
        p_5db[Path(r["image"]).stem] = r["truth"]

    p_5d8 = []
    n_fail = 0
    for r in p_5d9:
        if len(p_5d8) >= p_5d0.sample:
            break
        pdf = Path(p_5d0.data_dir) / r["document_id"]
        for p_5d7, p_5d5 in enumerate(q_117(r["bboxes"])):
            p_5d6 = f"{r['id']}__L{p_5d7:03d}"
            truth = p_5db.get(p_5d6)
            if not truth or len(p_5d8) >= p_5d0.sample:
                continue
            p_5dc = q_17b(r["bboxes"], p_5d5)
            if p_5dc is None:
                continue
            box = q_130(p_5dc, p_5d0.pad, r["width"], r["height"])
            p_5d3 = q_14f(pdf, r["page_no"], box, p_5d0.dpi)
            if p_5d3 is None:
                n_fail += 1
                continue
            w, h, rgb = p_5d3
            try:
                p_5d4 = rec_vietocr(w, h, rgb, REC_URL, p_5d0.timeout)
            except Exception:
                n_fail += 1
                continue
            p_5d8.append((norm(p_5d4), norm(truth)))

    if not p_5d8:
        print("LỖI: không probe được crop nào", file=sys.stderr)
        return 2
    p_5da = q_10a(p_5d8)
    print(json.dumps({
        "n_probed": p_5da["n"], "n_fail": n_fail,
        "rec_vietocr_onnx_on_SAME_crops": {
            "cer_micro": round(p_5da["cer_micro"], 4),
            "cer_macro": round(p_5da["cer_macro"], 4),
            "exact": round(p_5da["exact_match_rate"], 3)},
        "reference": {"torch_base_same_crops": 0.0848, "triton_page_pipeline_joined": 0.1253},
    }, ensure_ascii=True, indent=2))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
