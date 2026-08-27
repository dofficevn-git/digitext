#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

from extract_vlm import PAGE_IDENTIFIER_FIELDS

PRIORITY: dict[str, str] = {
    "ocr_wrong": "high",
    "fix_span": "med",
    "vlm_wrong": "med",
    "missing": "med",
    "ok": "low",
}
PRIORITY_RANK: dict[str, int] = {"high": 0, "med": 1, "low": 2}

def q_178(
    decisions: list[dict], fields: frozenset[str], crop_index: dict[tuple[str, str], dict]
) -> list[dict]:
    out: list[dict] = []
    for p_40d in decisions:
        if not p_40d.get("reviewed"):
            continue
        p_40f = p_40d.get("id")
        for p_40e, p_410 in p_40d.get("fields", {}).items():
            if p_40e not in fields:
                continue
            verdict = p_410.get("verdict", "ok")
            p_40c = crop_index.get((p_40f, p_40e))
            out.append({
                "id": p_40f,
                "field": p_40e,
                "verdict": verdict,
                "priority": PRIORITY.get(verdict, "med"),
                "crop_image": (p_40c or {}).get("image"),
                "truth_filled": bool((p_40c or {}).get("truth")) if p_40c else None,
            })
    out.sort(key=lambda e: (PRIORITY_RANK.get(e["priority"], 1), e["field"], e["id"] or ""))
    return out

def summarize(entries: list[dict]) -> dict:
    p_412 = Counter(e["priority"] for e in entries)
    p_411 = Counter(f"{e['field']}/{e['priority']}" for e in entries)
    total = len(entries)
    p_413 = sum(1 for e in entries if e["priority"] != "low")
    return {
        "total": total,
        "auto_accept": p_412.get("low", 0),
        "human_queue": p_413,
        "human_pct": round(100 * p_413 / total, 1) if total else 0.0,
        "by_priority": dict(sorted(p_412.items())),
        "by_field_priority": dict(sorted(p_411.items())),
    }

def load_jsonl(path: Path) -> list[dict]:
    out = []
    for p_414 in path.open(encoding="utf-8"):
        p_414 = p_414.strip()
        if p_414:
            try:
                out.append(json.loads(p_414))
            except json.JSONDecodeError:
                continue
    return out

def q_f9(crops: list[dict]) -> dict[tuple[str, str], dict]:
    return {(p_415.get("id"), p_415.get("field")): p_415 for p_415 in crops}

def main(argv: list[str] | None = None) -> int:
    p_416 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_416.add_argument("decisions_jsonl")
    p_416.add_argument("--crops", default="data/ocr_crops/crops.jsonl",
                    help="crops.jsonl để đính ảnh + biết truth đã gõ chưa (tuỳ, có thì tốt)")
    p_416.add_argument("--fields", default=",".join(sorted(PAGE_IDENTIFIER_FIELDS)),
                    help="trường soát (mặc định soKyHieu,ngayThang)")
    p_416.add_argument("--out", default="data/gold_review_queue.jsonl")
    p_417 = p_416.parse_args(argv)

    p_419 = Path(p_417.decisions_jsonl)
    if not p_419.is_file():
        print(f"LỖI: không thấy {p_419}", file=sys.stderr)
        return 2
    fields = frozenset(p_41a.strip() for p_41a in p_417.fields.split(",") if p_41a.strip())
    p_418 = Path(p_417.crops)
    crop_index = q_f9(load_jsonl(p_418)) if p_418.is_file() else {}

    entries = q_178(load_jsonl(p_419), fields, crop_index)
    p_41c = Path(p_417.out)
    p_41c.parent.mkdir(parents=True, exist_ok=True)
    with p_41c.open("w", encoding="utf-8") as p_41b:
        for e in entries:
            p_41b.write(json.dumps(e, ensure_ascii=False) + "\n")

    p_41d = summarize(entries)
    print(f"hàng đợi soát → {p_41c} ({p_41d['total']} quyết định định danh)", file=sys.stderr)
    print(f"  AUTO-nhận (ok):     {p_41d['auto_accept']}", file=sys.stderr)
    print(f"  NGƯỜI soát (đã sửa): {p_41d['human_queue']}  ({p_41d['human_pct']}%)", file=sys.stderr)
    print(f"  theo ưu tiên: {p_41d['by_priority']}", file=sys.stderr)
    print(f"  theo trường/ưu tiên: {p_41d['by_field_priority']}", file=sys.stderr)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
