#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))

import metrics

def q_e2() -> dict:
    return {"ce": 0, "cn": 0, "we": 0, "wn": 0, "nde": 0, "ndn": 0, "n": 0, "conf_pairs": []}

def q_b9(acc: dict, ref: str, hyp: str, conf=None) -> None:
    ce, cn = metrics.char_counts(ref, hyp)
    we, wn = metrics.word_counts(ref, hyp)
    nde, ndn = metrics.char_counts_no_diac(ref, hyp)
    acc["ce"] += ce
    acc["cn"] += cn
    acc["we"] += we
    acc["wn"] += wn
    acc["nde"] += nde
    acc["ndn"] += ndn
    acc["n"] += 1
    if isinstance(conf, (int, float)) and not isinstance(conf, bool):
        acc["conf_pairs"].append((float(conf), metrics.normalize_vi(ref) == metrics.normalize_vi(hyp)))

def q_d5(acc: dict) -> dict:
    cer = acc["ce"] / acc["cn"] if acc["cn"] else 0.0
    wer = acc["we"] / acc["wn"] if acc["wn"] else 0.0
    p_149 = acc["nde"] / acc["ndn"] if acc["ndn"] else 0.0
    out = {
        "cer": round(cer, 6),
        "wer": round(wer, 6),
        "cer_no_diac": round(p_149, 6),
        "delta_dau": round(cer - p_149, 6),
        "n_lines": acc["n"],
    }
    if acc["conf_pairs"]:
        out["ece"] = round(metrics.ece(acc["conf_pairs"]), 6)
        out["n_conf"] = len(acc["conf_pairs"])
    return out

def score(gold: list, preds: dict, meta: dict | None = None, corrector=None) -> dict:
    overall = q_e2()
    p_151: dict = {}
    skipped_empty_ref = 0
    missing_pred = 0
    tokens_changed = 0
    p_14d = set()

    for p_14b in gold:
        p_14c = p_14b["id"]
        p_14d.add(p_14c)
        ref = p_14b.get("ref", "") or ""
        stratum = p_14b.get("stratum") or "_unknown"
        if not metrics.normalize_vi(ref).strip():
            skipped_empty_ref += 1
            continue
        p_150 = preds.get(p_14c)
        if p_150 is None:
            missing_pred += 1
            hyp, conf = "", None
        elif isinstance(p_150, str):
            hyp, conf = p_150, None
        else:
            hyp, conf = p_150.get("hyp", ""), p_150.get("conf")
        if corrector is not None:
            hyp, p_14a = corrector(hyp, conf if conf is not None else 0.0)
            tokens_changed += p_14a
        q_b9(overall, ref, hyp, conf)
        q_b9(p_151.setdefault(stratum, q_e2()), ref, hyp, conf)

    extra_pred = sum(1 for p_14e in preds if p_14e not in p_14d)
    p_14f = {
        **(meta or {}),
        "n_gold": len(gold),
        "n_scored": overall["n"],
        "skipped_empty_ref": skipped_empty_ref,
        "missing_pred": missing_pred,
        "extra_pred": extra_pred,
    }
    if corrector is not None:
        p_14f["corrected"] = True
        p_14f["tokens_changed"] = tokens_changed
    return {
        "overall": q_d5(overall),
        "by_stratum": {p_14e: q_d5(p_152) for p_14e, p_152 in sorted(p_151.items())},
        "meta": p_14f,
    }

def _load_jsonl(path: str, label: str) -> list:
    p_156 = []
    try:
        with open(path, encoding="utf-8") as p_153:
            for p_155, p_154 in enumerate(p_153, 1):
                p_154 = p_154.strip()
                if not p_154:
                    continue
                try:
                    p_156.append(json.loads(p_154))
                except json.JSONDecodeError as e:
                    raise SystemExit(f"❌ {label}: JSON hỏng dòng {p_155}: {e}")
    except FileNotFoundError:
        raise SystemExit(f"❌ {label}: không thấy file: {path}")
    return p_156

def main(argv=None) -> int:
    p_157 = argparse.ArgumentParser(description="Chấm CER/WER/Δdấu theo tầng → report.json (§S13.B.2)")
    p_157.add_argument("--gold", required=True, help="gold.jsonl {id, ref, stratum}")
    p_157.add_argument("--pred", required=True, help="pred.jsonl {id, hyp} (run_bench.py sinh)")
    p_157.add_argument("--out", required=True, help="report.json đầu ra (cho gate.py)")
    p_157.add_argument("--engine-version", default=None, help="ghi vào meta để truy vết")
    p_157.add_argument("--golden-version", default=None, help="ghi vào meta để truy vết")
    p_157.add_argument("--lexicon", default=None, help="bật sửa confusables/lexicon (§S13.A) TRƯỚC khi chấm")
    p_157.add_argument("--gazetteer", default=None, help="địa danh/tên riêng cần bảo vệ (dùng với --lexicon)")
    p_157.add_argument("--conf-gate", type=float, default=0.85, help="conf ≥ ngưỡng này thì không sửa")
    p_157.add_argument("--max-cost", type=float, default=0.6, help="chi phí sửa tối đa nhận từ lexicon")
    p_158 = p_157.parse_args(argv)

    gold = _load_jsonl(p_158.gold, "gold")
    p_15f = _load_jsonl(p_158.pred, "pred")
    preds = {p_160["id"]: p_160 for p_160 in p_15f}

    meta = {}
    if p_158.engine_version:
        meta["engine_version"] = p_158.engine_version
    if p_158.golden_version:
        meta["golden_version"] = p_158.golden_version

    corrector = None
    if p_158.lexicon:
        import correct

        p_15c = correct.load_lexicon(p_158.lexicon)
        p_15b = correct.load_lexicon(p_158.gazetteer) if p_158.gazetteer else frozenset()
        corrector = lambda h, c: correct.correct_line(
            h, c, p_15c, p_15b, p_158.conf_gate, p_158.max_cost
        )

    report = score(gold, preds, meta, corrector)
    with open(p_158.out, "w", encoding="utf-8") as p_15a:
        json.dump(report, p_15a, ensure_ascii=False, indent=2)
        p_15a.write("\n")

    p_15d = report["meta"]
    p_15e = report["overall"]
    p_159 = f" · ECE {p_15e['ece']:.4f}" if "ece" in p_15e else ""
    print(
        f"✅ {p_158.out}: {p_15d['n_scored']}/{p_15d['n_gold']} dòng · "
        f"CER {p_15e['cer']:.4f} · WER {p_15e['wer']:.4f} · Δdấu {p_15e['delta_dau']:.4f}{p_159}"
    )
    if p_15d["missing_pred"]:
        print(f"⚠ thiếu prediction cho {p_15d['missing_pred']} dòng (tính lỗi).")
    if p_15d["skipped_empty_ref"]:
        print(f"⚠ bỏ {p_15d['skipped_empty_ref']} dòng gold ref rỗng.")
    if p_15d["extra_pred"]:
        print(f"⚠ {p_15d['extra_pred']} prediction thừa (không có trong gold) — bỏ qua.")
    if p_15d.get("corrected"):
        print(f"✎ đã sửa {p_15d['tokens_changed']} token (confusables/lexicon §S13.A).")
    if p_15d["n_scored"] == 0:
        print("⚠ KHÔNG chấm được dòng nào — report có thể vô nghĩa.", file=sys.stderr)
    return 0

if __name__ == "__main__":
    sys.exit(main())
