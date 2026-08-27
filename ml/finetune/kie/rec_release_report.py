#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import unicodedata
from pathlib import Path

def norm(s: str) -> str:
    return unicodedata.normalize("NFC", (s or "").strip())

def build_report(rows: list[dict], pred_key: str, engine: str,
                 truth_key: str = "truth", stratum_key: str = "field") -> dict:
    from cer import q_10a

    p_5e4 = [p_5e3 for p_5e3 in rows if pred_key in p_5e3]
    p_5e2 = [(norm(p_5e3[pred_key]), norm(p_5e3[truth_key])) for p_5e3 in p_5e4]

    p_5dd: dict[str, dict] = {}
    p_5df: dict[str, list] = {}
    for p_5e3 in p_5e4:
        p_5df.setdefault(str(p_5e3.get(stratum_key) or "khong_ro"), []).append(p_5e3)
    for p_5e1, p_5e0 in sorted(p_5df.items()):
        p_5de = q_10a([(norm(p_5e3[pred_key]), norm(p_5e3[truth_key])) for p_5e3 in p_5e0])
        p_5dd[p_5e1] = {"cer": round(p_5de["cer_micro"], 4), "n_lines": len(p_5e0)}

    overall = q_10a(p_5e2) if p_5e2 else {"cer_micro": 0.0}
    return {
        "meta": {"engine_version": engine, "pred_key": pred_key,
                 "n_skipped_missing_pred": len(rows) - len(p_5e4)},
        "overall": {"cer": round(overall["cer_micro"], 4), "n_lines": len(p_5e4)},
        "by_stratum": p_5dd,
    }

def main(argv: list[str] | None = None) -> int:
    p_5e6 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_5e6.add_argument("rows_jsonl")
    p_5e6.add_argument("--pred", required=True, help="khoá pred trong rows (vd pred_onnx_ft)")
    p_5e6.add_argument("--out", required=True)
    p_5e6.add_argument("--engine", required=True, help="nhãn version engine ghi vào meta")
    p_5e5 = p_5e6.parse_args(argv)

    rows = [json.loads(p_5e7) for p_5e7 in open(p_5e5.rows_jsonl, encoding="utf-8") if p_5e7.strip()]
    p_5e8 = build_report(rows, p_5e5.pred, p_5e5.engine)
    Path(p_5e5.out).write_text(json.dumps(p_5e8, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(p_5e8, ensure_ascii=True, indent=2))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
