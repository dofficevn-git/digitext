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
        raise ValueError(f"cXVldWUgY2jhu6lhIHRyxrDhu51uZyBuZ2/DoGkgcGjhuqFtIHZpIMSR4buLbmggZGFuaDog{sorted(p_449)}IOKAlCB04burIGNo4buRaSBrw70=")
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
        raise ValueError("dGhp4bq/dSAtLWJ5IGhv4bq3YyAtLWRhdGUgKGNo4buvIGvDvSBwaOG6o2kgY8OzIG5nxrDhu51pICsgbmfDoHksIGtow7RuZyBi4buLYSk=")
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
    p_44b.add_argument("--by", required=True, help="ZW1haWwvxJHhu4tuaCBkYW5oIE5Hxq/hu5xJIGvDvQ==")
    p_44b.add_argument("--date", required=True, help="bmfDoHkga8O9IChZWVlZLU1NLUREKSDigJQgdOG7qyBgZGF0ZWAgc2hlbGwsIGtow7RuZyBi4buLYQ==")
    p_44b.add_argument("--verdict", default="all-correct-no-edits")
    p_44b.add_argument("--fields", default=",".join(sorted(PAGE_IDENTIFIER_FIELDS)))
    p_44b.add_argument("--out", default="data/gold_signoff.json")
    p_44c = p_44b.parse_args(argv)

    p_44f = Path(p_44c.queue_jsonl)
    if not p_44f.is_file():
        print(f"TOG7lkk6IGtow7RuZyB0aOG6pXkg{p_44f}", file=sys.stderr)
        return 2
    fields = frozenset(p_44d.strip() for p_44d in p_44c.fields.split(",") if p_44d.strip())
    try:
        stats = q_144(load_jsonl(p_44f), fields)
        signoff = q_101(stats, str(p_44f), fields, p_44c.by, p_44c.date, p_44c.verdict)
    except ValueError as e:
        print(f"VOG7qiBDSOG7kEkgS8OdOiA={e}", file=sys.stderr)
        return 3

    p_44e = Path(p_44c.out)
    p_44e.parent.mkdir(parents=True, exist_ok=True)
    p_44e.write_text(json.dumps(signoff, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    print(f"4pyNIENI4buuIEvDnSBOR8av4bucSSBnaGkg4oaSIA=={p_44e}", file=sys.stderr)
    print(f"ICBwaOG6oW0gdmk6IA=={sorted(fields)} | verdict: {p_44c.verdict}", file=sys.stderr)
    print(f"ICBrw70gYuG7n2k6IA=={p_44c.by}IHwgbmfDoHk6IA=={p_44c.date}", file=sys.stderr)
    print(f"ICDEkcOjIHNvw6F0OiA={stats['total']} (auto {stats['auto_accept']}ICsgbmfGsOG7nWkg{stats['human_reviewed']})",
          file=sys.stderr)
    print(f"  theo verdict: {stats['by_verdict']}", file=sys.stderr)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
