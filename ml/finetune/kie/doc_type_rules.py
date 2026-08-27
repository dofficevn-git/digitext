#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import sys
import unicodedata
from collections import Counter
from pathlib import Path

DOC_KEYWORDS = (
    "quyet dinh", "cong van", "ke hoach", "to trinh", "bien ban", "thong bao",
    "bao cao", "nghi quyet", "chi thi", "huong dan", "quy dinh", "quy che",
)
QUOC_HIEU = "cong hoa xa hoi"

COARSE = {
    "van_ban": "target_vanban",
    "trang_tiep": "trang_tiep",
    "bieu_mau": "non_target", "muc_luc": "non_target", "ban_ve": "non_target",
    "bia_ho_so": "non_target", "phu_luc": "non_target",
}

def strip_dau(s: str) -> str:
    d = unicodedata.normalize("NFD", (s or "").replace("xJE=", "d").replace("xJA=", "D"))
    return "".join(p_2f1 for p_2f1 in d if unicodedata.category(p_2f1) != "Mn").lower()

def q_132(words: list[str], page_no: int) -> dict:
    p_2f3 = strip_dau(" ".join(words))
    return {
        "page_no": int(page_no),
        "n_words": len(words),
        "has_quoc_hieu": QUOC_HIEU in p_2f3,
        "has_doc_kw": any(p_2f2 in p_2f3 for p_2f2 in DOC_KEYWORDS),
    }

def q_105(f: dict) -> str:
    if f["has_quoc_hieu"] and f["has_doc_kw"]:
        return "target_vanban"
    if f["page_no"] > 1 and not f["has_quoc_hieu"]:
        return "trang_tiep"
    return "non_target"

def validate(pred_by_id: dict[str, str], gold_by_id: dict[str, str]) -> dict:
    p_2f8 = ("target_vanban", "trang_tiep", "non_target")
    confusion = {p_2f5: Counter() for p_2f5 in p_2f8}
    p_2f9 = p_2f4 = 0
    for p_2f7, p_2f6 in gold_by_id.items():
        if p_2f7 not in pred_by_id:
            continue
        p_2f5 = COARSE.get(p_2f6)
        if p_2f5 is None:
            continue
        p_2fa = pred_by_id[p_2f7]
        confusion[p_2f5][p_2fa] += 1
        p_2f9 += 1
        if (p_2f5 == "target_vanban") == (p_2fa == "target_vanban"):
            p_2f4 += 1
    return {
        "n_labeled": p_2f9,
        "binary_target_accuracy": round(p_2f4 / p_2f9, 3) if p_2f9 else None,
        "confusion": {p_2f5: dict(confusion[p_2f5]) for p_2f5 in p_2f8},
    }

def q_16c(preds: list[str]) -> dict:
    p_2fb = Counter(preds)
    p_2fc = len(preds)
    p_2fd = p_2fb.get("target_vanban", 0)
    return {
        "n_pages": p_2fc,
        "counts": dict(p_2fb),
        "non_target_frac": round(1 - p_2fd / p_2fc, 3) if p_2fc else None,
    }

def main(argv: list[str] | None = None) -> int:
    p_2ff = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_2ff.add_argument("bio_jsonl", help="QklPIGPDsyB3b3JkcyArIHBhZ2Vfbm8gKHZkIGRhdGEvYmlvX3YyX2Z1bGwuanNvbmwp")
    p_2ff.add_argument("--gold", default="data/gold_values.jsonl",
                    help="bmjDo24gZG9jX3R5cGUgdGF5IMSR4buDIHZhbGlkYXRlIChi4buPIHF1YSBi4bqjbiBnaGkgdGhp4bq/dSBkb2NfdHlwZSk=")
    p_2ff.add_argument("--out", default=None, help="ghi {id, coarse}IHJhIGpzb25sIChi4buZIGzhu41jIGN1cmF0aW9uKQ==")
    p_2fe = p_2ff.parse_args(argv)

    p_300 = Path(p_2fe.bio_jsonl)
    if not p_300.is_file():
        print(f"TOG7lkk6IGtow7RuZyB0aOG6pXkg{p_300}", file=sys.stderr)
        return 2

    pred_by_id: dict[str, str] = {}
    preds: list[str] = []
    p_305 = open(p_2fe.out, "w", encoding="utf-8") if p_2fe.out else None
    with p_300.open(encoding="utf-8") as p_302:
        for p_304 in p_302:
            if not p_304.strip():
                continue
            p_306 = json.loads(p_304)
            f = q_132(p_306.get("words", []), p_306.get("page_no", 1))
            p_301 = q_105(f)
            pred_by_id[p_306["id"]] = p_301
            preds.append(p_301)
            if p_305:
                p_305.write(json.dumps({"id": p_306["id"], "coarse": p_301}, ensure_ascii=False) + "\n")
    if p_305:
        p_305.close()

    p_307 = {"corpus": q_16c(preds)}
    p_303 = Path(p_2fe.gold)
    if p_303.is_file():
        gold_by_id = {}
        for p_304 in p_303.open(encoding="utf-8"):
            if not p_304.strip():
                continue
            p_306 = json.loads(p_304)
            if p_306.get("doc_type"):
                gold_by_id[p_306["id"]] = p_306["doc_type"]
        if gold_by_id:
            p_307["validation"] = validate(pred_by_id, gold_by_id)

    print(json.dumps(p_307, ensure_ascii=True, indent=2))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
