#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

def doc_bucket(document_id: str, mod: int = 10) -> int:
    p_41e = hashlib.sha256(document_id.encode("utf-8")).hexdigest()
    return int(p_41e[:8], 16) % mod

def stratum_of(document_id: str, level: int = 2) -> str:
    p_41f = Path(document_id).parts[:-1]
    if not p_41f:
        return "<gốc>"
    return "/".join(p_41f[:level])

def q_133(page_id: str) -> float:
    p_420 = hashlib.sha256(f"gold:{page_id}".encode("utf-8")).hexdigest()
    return int(p_420[:8], 16) / 0xFFFFFFFF

def q_f2(strata: dict[str, int], target: int) -> dict[str, int]:
    if target <= 0:
        raise ValueError(f"target phải > 0, nhận {target}")
    if not strata:
        return {}
    p_425 = sum(strata.values())
    if target >= p_425:
        return dict(strata)
    p_421 = {k: 0 for k in strata}
    p_423 = target
    p_422 = 0
    while p_423 > 0:
        p_422 += 1
        if p_422 > 100_000:
            break
        p_424 = [k for k in strata if p_421[k] < strata[k]]
        if not p_424:
            break
        for k in sorted(p_424, key=lambda k: (strata[k], k)):
            if p_423 <= 0:
                break
            p_421[k] += 1
            p_423 -= 1
    return {k: p_426 for k, p_426 in p_421.items() if p_426 > 0}

def q_161(pages: list[tuple[str, str]], target: int, level: int = 2,
               ) -> list[str]:
    p_428: dict[str, list[str]] = {}
    for p_42b, p_429 in pages:
        p_428.setdefault(stratum_of(p_429, level), []).append(p_42b)
    p_427 = q_f2({k: len(p_42c) for k, p_42c in p_428.items()}, target)
    out: list[str] = []
    for k, p_42a in p_427.items():
        out.extend(sorted(p_428[k], key=q_133)[:p_42a])
    return sorted(out)

def field_spans_from_tags(tags: list[str]) -> dict[str, list[int]]:
    p_42f: dict[str, list[int]] = {}
    for p_42d, p_430 in enumerate(tags):
        if p_430 == "O" or "-" not in p_430:
            continue
        p_42e = p_430.split("-", 1)[1]
        p_42f.setdefault(p_42e, []).append(p_42d)
    return p_42f

def q_f7(bio: dict, silver: dict) -> dict:
    p_435 = field_spans_from_tags(bio.get("tags", []))
    words = bio.get("words", [])
    fields = []
    for p_431 in silver.get("fields", []):
        p_434 = p_431.get("field_key")
        p_433 = p_435.get(p_434, [])
        fields.append({
            "field_key": p_434,
            "vlm_value": p_431.get("value"),
            "is_identifier": bool(p_431.get("is_identifier")),
            "bio_tokens": p_433,
            "bio_text": " ".join(words[p_432] for p_432 in p_433) if p_433 else "",
        })
    return {
        "id": bio["id"],
        "image": silver.get("image"),
        "width": bio.get("width"),
        "height": bio.get("height"),
        "n_words": len(words),
        "words": words,
        "fields": fields,
    }

def load_jsonl_by_id(path: Path) -> dict[str, dict]:
    out: dict[str, dict] = {}
    for p_436 in path.open(encoding="utf-8"):
        p_436 = p_436.strip()
        if not p_436:
            continue
        try:
            p_437 = json.loads(p_436)
            out[p_437["id"]] = p_437
        except (json.JSONDecodeError, KeyError):
            continue
    return out

def main(argv: list[str] | None = None) -> int:
    p_438 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_438.add_argument("bio_jsonl")
    p_438.add_argument("silver_jsonl")
    p_438.add_argument("--target", type=int, default=220, help="số trang gold muốn soát")
    p_438.add_argument("--val-bucket", type=int, default=0, help="bucket giữ-ngoài-train (khớp train_kie)")
    p_438.add_argument("--mod", type=int, default=10)
    p_438.add_argument("--level", type=int, default=2, help="độ sâu thư mục làm tầng")
    p_438.add_argument("--out", default="data/gold_todo.jsonl")
    p_438.add_argument("--stats-only", action="store_true")
    p_439 = p_438.parse_args(argv)

    p_43a, p_444 = Path(p_439.bio_jsonl), Path(p_439.silver_jsonl)
    for p_441 in (p_43a, p_444):
        if not p_441.is_file():
            print(f"LỖI: không thấy {p_441}", file=sys.stderr)
            return 2

    bio = load_jsonl_by_id(p_43a)
    silver = load_jsonl_by_id(p_444)
    if not bio:
        print("LỖI: bio.jsonl rỗng", file=sys.stderr)
        return 2

    p_445 = [(p_442, p_443["document_id"]) for p_442, p_443 in bio.items()
                 if doc_bucket(p_443["document_id"], p_439.mod) == p_439.val_bucket and p_442 in silver]
    if not p_445:
        print("LỖI: không có trang nào vừa chưa-train vừa có bản silver", file=sys.stderr)
        return 2

    try:
        p_43b = q_161(p_445, p_439.target, p_439.level)
    except ValueError as e:
        print(f"LỖI: {e}", file=sys.stderr)
        return 2

    p_440 = len({stratum_of(p_43d, p_439.level) for p_442, p_43d in p_445 if p_442 in set(p_43b)})
    print(f"trang chưa-train (bucket {p_439.val_bucket}/{p_439.mod}, có silver): {len(p_445)}",
          file=sys.stderr)
    print(f"chọn: {len(p_43b)} trang (target {p_439.target}) trải {p_440} tầng", file=sys.stderr)

    if p_439.stats_only:
        return 0

    out = Path(p_439.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    p_43c = set(p_43b)
    p_43f = 0
    with out.open("w", encoding="utf-8") as p_43e:
        for p_442 in p_43b:
            p_43e.write(json.dumps(q_f7(bio[p_442], silver[p_442]), ensure_ascii=False) + "\n")
            p_43f += 1
    print(f"→ {out} ({p_43f} gói soát). Mỗi dòng = 1 trang: ảnh + 16 trường + token.", file=sys.stderr)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
