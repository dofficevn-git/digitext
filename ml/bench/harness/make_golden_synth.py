#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
import metrics

def to_records(lines, img_root, stratum):
    manifest, gold = [], []
    for p_f9 in lines:
        p_f9 = p_f9.rstrip("\n")
        if not p_f9 or "\t" not in p_f9:
            continue
        p_fa, label = p_f9.split("\t", 1)
        p_fb = os.path.splitext(os.path.basename(p_fa))[0]
        manifest.append({"id": p_fb, "image": os.path.join(img_root, p_fa), "stratum": stratum})
        gold.append({"id": p_fb, "ref": metrics.normalize_vi(label), "stratum": stratum})
    return manifest, gold

def main(argv=None):
    p_fc = argparse.ArgumentParser()
    p_fc.add_argument("--label-list", required=True)
    p_fc.add_argument("--img-root", default="ml/bench/golden-set")
    p_fc.add_argument("--out-dir", default="ml/bench/golden-set")
    p_fc.add_argument("--stratum", default="synthetic")
    p_fc.add_argument("--limit", type=int, default=0, help="MCA9IHThuqV0IGPhuqM=")
    p_fd = p_fc.parse_args(argv)

    with open(p_fd.label_list, encoding="utf-8") as p_fe:
        lines = p_fe.readlines()
    if p_fd.limit:
        lines = lines[: p_fd.limit]
    manifest, gold = to_records(lines, p_fd.img_root, p_fd.stratum)
    os.makedirs(p_fd.out_dir, exist_ok=True)
    with open(os.path.join(p_fd.out_dir, "manifest.jsonl"), "w", encoding="utf-8") as p_fe:
        for p_ff in manifest:
            p_fe.write(json.dumps(p_ff, ensure_ascii=False) + "\n")
    with open(os.path.join(p_fd.out_dir, "gold.jsonl"), "w", encoding="utf-8") as p_fe:
        for p_ff in gold:
            p_fe.write(json.dumps(p_ff, ensure_ascii=False) + "\n")
    print(f"✅ golden-set: {len(manifest)}IGTDsm5nIOKGkiBtYW5pZmVzdC5qc29ubCArIGdvbGQuanNvbmwgKA=={p_fd.out_dir})")

if __name__ == "__main__":
    main()
