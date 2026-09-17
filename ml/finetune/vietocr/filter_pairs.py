#!/usr/bin/env python3
"""Lọc cặp huấn luyện VietOCR theo độ đồng thuận với model đang phục vụ.

Vì sao: nhãn do VLM sinh từ crop dòng. Nhìn 36 crop (17/9/2026) thấy lỗi chính
không phải VLM đọc sai chữ, mà crop DÍNH MÉP dòng bên cạnh và VLM chép luôn phần
dính ("- Như trên Lưu: TC"). Huấn luyện trên đó dạy model bịa thêm chữ. Dòng mà
VietOCR hiện tại và VLM lệch nhau quá nhiều gần như chắc là kiểu này (VietOCR chỉ
đọc dòng chính) — bỏ. Lệch vừa (dấu, một hai ký tự) mới là thứ đáng học — giữ.

Cái giá: cũng bỏ mất những dòng VietOCR đọc sai nặng thật. Chấp nhận, vì không
có người kiểm; ghi số dòng bỏ theo lý do để biết mình bỏ bao nhiêu.

    python filter_pairs.py --pairs data/rec_vi/train.txt --pred pred_all.jsonl \
        --config ../../../ocr/vietocr_config/vgg_transformer.yml --out data/rec_vi_ft/train.txt
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import unicodedata

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "bench", "harness"))
from metrics import levenshtein  # noqa: E402


def norm(s: str) -> str:
    return " ".join(unicodedata.normalize("NFC", s or "").split())


def main(argv=None) -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--pairs", required=True, help="<ảnh>\\t<nhãn VLM>")
    p.add_argument("--pred", required=True, help="pred.jsonl của run_bench.py (id = tên file crop)")
    p.add_argument("--config", required=True, help="yml VietOCR đang phục vụ (lấy vocab)")
    p.add_argument("--img-root", default=None, help="gốc ảnh để đo tỉ lệ rộng/ký tự (bỏ = không kiểm)")
    p.add_argument("--out", required=True)
    p.add_argument("--path-prefix", default="", help="tiền tố ghi vào cột ảnh của --out, vd ../rec_vi/")
    p.add_argument("--max-cer", type=float, default=0.35)
    p.add_argument("--min-len", type=int, default=2)
    p.add_argument("--max-len", type=int, default=120)
    p.add_argument("--ratio-lo", type=float, default=0.18)
    p.add_argument("--ratio-hi", type=float, default=1.2)
    a = p.parse_args(argv)

    import yaml

    vocab = set(yaml.safe_load(open(a.config, encoding="utf-8"))["vocab"])
    hyp = {}
    for l in open(a.pred, encoding="utf-8"):
        if l.strip():
            r = json.loads(l)
            hyp[r["id"]] = norm(r.get("hyp", ""))

    keep, why = [], {"no_pred": 0, "vocab": 0, "len": 0, "ratio": 0, "cer": 0}
    cers = []
    for l in open(a.pairs, encoding="utf-8"):
        if "\t" not in l:
            continue
        rel, text = l.rstrip("\n").split("\t", 1)
        ref = norm(text)
        name = rel.split("/")[-1]
        if name not in hyp:
            why["no_pred"] += 1
            continue
        if any(c not in vocab for c in ref):
            why["vocab"] += 1
            continue
        if not (a.min_len <= len(ref) <= a.max_len):
            why["len"] += 1
            continue
        if a.img_root:
            from PIL import Image

            w, h = Image.open(os.path.join(a.img_root, rel)).size
            ratio = (w / max(h, 1)) / len(ref)
            if not (a.ratio_lo <= ratio <= a.ratio_hi):
                why["ratio"] += 1
                continue
        cer = levenshtein(hyp[name], ref) / max(len(ref), 1)
        cers.append(cer)
        if cer > a.max_cer:
            why["cer"] += 1
            continue
        keep.append(f"{a.path_prefix}{rel}\t{ref}")

    os.makedirs(os.path.dirname(os.path.abspath(a.out)), exist_ok=True)
    with open(a.out, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(keep) + ("\n" if keep else ""))
    total = len(keep) + sum(why.values())
    cers.sort()
    med = cers[len(cers) // 2] if cers else 0
    print(f"✅ {a.out}: giữ {len(keep)}/{total} · bỏ {why} · CER(VietOCR↔VLM) median {med:.3f}, "
          f"trung bình {sum(cers)/max(len(cers),1):.3f} trên {len(cers)} dòng có dự đoán")
    return 0


if __name__ == "__main__":
    sys.exit(main())
