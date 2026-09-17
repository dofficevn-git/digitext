#!/usr/bin/env python3
"""Biến tệp cặp <ảnh dòng>\t<text> (đầu ra build_rec_dataset.py / val.txt) thành
manifest.jsonl + gold.jsonl cho score.py/gate.py.

Đây là "bộ đồng thuận VLM": nhãn do Qwen đọc, không phải người gõ. Dùng để đo
xu hướng và hồi quy (CPU↔GPU, base↔fine-tune), KHÔNG dùng làm gate phát hành.
`meta.source` ghi rõ điều đó để report không bị hiểu là CER thật.

    python make_golden_from_pairs.py --pairs data/rec_vi/val.txt --img-root data/rec_vi \
        --out golden-set/vlm_pseudo --stratum vlm_pseudo
"""
from __future__ import annotations

import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from metrics import normalize_vi  # noqa: E402


def main(argv=None) -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--pairs", required=True, help="<ảnh>\\t<text> mỗi dòng")
    p.add_argument("--img-root", default=".", help="gốc đường dẫn ảnh trong --pairs")
    p.add_argument("--out", required=True, help="thư mục ghi manifest.jsonl + gold.jsonl")
    p.add_argument("--stratum", default="vlm_pseudo")
    p.add_argument("--source", default="qwen2.5-vl-pseudo", help="ghi vào meta.source của gold")
    p.add_argument("--limit", type=int, default=0)
    a = p.parse_args(argv)

    os.makedirs(a.out, exist_ok=True)
    n = skipped = 0
    with open(a.pairs, encoding="utf-8") as f, \
         open(os.path.join(a.out, "manifest.jsonl"), "w", encoding="utf-8") as mf, \
         open(os.path.join(a.out, "gold.jsonl"), "w", encoding="utf-8") as gf:
        for i, line in enumerate(f):
            line = line.rstrip("\n")
            if "\t" not in line:
                continue
            rel, text = line.split("\t", 1)
            img = os.path.abspath(os.path.join(a.img_root, rel))
            ref = normalize_vi(text)
            if not ref or not os.path.exists(img):
                skipped += 1
                continue
            rid = f"l{i:05d}"
            mf.write(json.dumps({"id": rid, "image": img, "stratum": a.stratum}, ensure_ascii=False) + "\n")
            gf.write(json.dumps({"id": rid, "ref": ref, "stratum": a.stratum,
                                 "meta": {"source": a.source}}, ensure_ascii=False) + "\n")
            n += 1
            if a.limit and n >= a.limit:
                break
    print(f"✅ {a.out}: {n} dòng (bỏ {skipped}) · nguồn nhãn: {a.source} — KHÔNG phải CER thật")
    return 0


if __name__ == "__main__":
    sys.exit(main())
