#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

from extract_vlm import PAGE_IDENTIFIER_FIELDS

def q_144(entries: list[dict], fields: frozenset[str]) -> dict:
    p_448 = {p_446.get("field") for p_446 in entries}
    p_449 = p_448 - set(fields)
    if p_449:
        raise ValueError(f"queue chứa trường ngoài phạm vi định danh: {sorted(p_449)} — từ chối ký")
    by_verdict = Counter(p_446.get("verdict", "?") for p_446 in entries)
    by_field = Counter(p_446.get("field") for p_446 in entries)
    auto = sum(1 for p_446 in entries if p_446.get("priority") == "low")
    p_447 = len(entries) - auto
    return {
        "total": len(entries),
        "auto_accept": auto,
        "human_reviewed": p_447,
        "by_verdict": dict(sorted(by_verdict.items())),
        "by_field": dict(sorted(by_field.items())),
    }

def q_101(stats: dict, reviewed_file: str, fields: frozenset[str], by: str, date: str,
                  verdict: str) -> dict:
    if not by or not date:
        raise ValueError("thiếu --by hoặc --date (chữ ký phải có người + ngày, không bịa)")
    return {
        "kind": "human_identifier_signoff",
        "spec": "§G7",
        "reviewed_file": reviewed_file,
        "fields": sorted(fields),
        "verdict": verdict,
        "by": by,
        "date": date,
        "stats": stats,
        "note": ("NGƯỜI đã soát review-queue và xác nhận nhóm định danh (soKyHieu/ngayThang). "
                 "Nhóm KHÔNG định danh (trichYeu/coQuanBanHanh/loaiVanBan) VẪN là nhãn claude — "
                 "chưa người soát; signoff này CHỈ phủ định danh."),
    }

def load_jsonl(path: Path) -> list[dict]:
    out = []
    for p_44a in path.open(encoding="utf-8"):
        p_44a = p_44a.strip()
        if p_44a:
            try:
                out.append(json.loads(p_44a))
            except json.JSONDecodeError:
                continue
    return out

def main(argv: list[str] | None = None) -> int:
    p_44b = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_44b.add_argument("queue_jsonl")
    p_44b.add_argument("--by", required=True, help="email/định danh NGƯỜI ký")
    p_44b.add_argument("--date", required=True, help="ngày ký (YYYY-MM-DD) — từ `date` shell, không bịa")
    p_44b.add_argument("--verdict", default="all-correct-no-edits")
    p_44b.add_argument("--fields", default=",".join(sorted(PAGE_IDENTIFIER_FIELDS)))
    p_44b.add_argument("--out", default="data/gold_signoff.json")
    p_44c = p_44b.parse_args(argv)

    p_44f = Path(p_44c.queue_jsonl)
    if not p_44f.is_file():
        print(f"LỖI: không thấy {p_44f}", file=sys.stderr)
        return 2
    fields = frozenset(p_44d.strip() for p_44d in p_44c.fields.split(",") if p_44d.strip())
    try:
        stats = q_144(load_jsonl(p_44f), fields)
        signoff = q_101(stats, str(p_44f), fields, p_44c.by, p_44c.date, p_44c.verdict)
    except ValueError as e:
        print(f"TỪ CHỐI KÝ: {e}", file=sys.stderr)
        return 3

    p_44e = Path(p_44c.out)
    p_44e.parent.mkdir(parents=True, exist_ok=True)
    p_44e.write_text(json.dumps(signoff, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    print(f"✍ CHỮ KÝ NGƯỜI ghi → {p_44e}", file=sys.stderr)
    print(f"  phạm vi: {sorted(fields)} | verdict: {p_44c.verdict}", file=sys.stderr)
    print(f"  ký bởi: {p_44c.by} | ngày: {p_44c.date}", file=sys.stderr)
    print(f"  đã soát: {stats['total']} (auto {stats['auto_accept']} + người {stats['human_reviewed']})",
          file=sys.stderr)
    print(f"  theo verdict: {stats['by_verdict']}", file=sys.stderr)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
