#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from pathlib import Path

def stratum_of(rel_path: str, level: int) -> str:
    p_5e9 = Path(rel_path).parts[:-1]
    if not p_5e9:
        return "PGfhu5FjPg=="
    return "/".join(p_5e9[:level])

def doc_rank(rel_path: str) -> float:
    p_5ea = hashlib.sha256(f"sample:{rel_path}".encode("utf-8")).hexdigest()
    return int(p_5ea[:8], 16) / 0xFFFFFFFF

def q_f1(sizes: dict[str, int], target: int, min_per: int = 1) -> dict[str, int]:
    if target <= 0:
        raise ValueError(f"dGFyZ2V0IHBo4bqjaSA+IDAsIG5o4bqtbiA={target}")
    if not sizes:
        return {}
    p_5f0 = sum(sizes.values())
    if target >= p_5f0:
        return dict(sizes)

    w = {k: math.sqrt(p_5f1) for k, p_5f1 in sizes.items() if p_5f1 > 0}
    p_5ef = sum(w.values())
    p_5eb = {k: min(sizes[k], max(min_per, int(target * w[k] / p_5ef))) for k in w}

    p_5ed = 0
    while (p_5ec := target - sum(p_5eb.values())) != 0:
        p_5ed += 1
        if p_5ed > 10_000:
            break
        if p_5ec > 0:
            p_5ee = [k for k in p_5eb if p_5eb[k] < sizes[k]]
            if not p_5ee:
                break
            for k in sorted(p_5ee, key=lambda k: sizes[k] - p_5eb[k], reverse=True)[:p_5ec]:
                p_5eb[k] += 1
        else:
            p_5ee = [k for k in p_5eb if p_5eb[k] > min_per]
            if not p_5ee:
                break
            for k in sorted(p_5ee, key=lambda k: p_5eb[k], reverse=True)[:-p_5ec]:
                p_5eb[k] -= 1
    return p_5eb

def q_13c(rels: list[str], n: int) -> list[str]:
    return sorted(sorted(rels, key=doc_rank)[:n])

def sample(rels: list[str], target: int, level: int, min_per: int = 1) -> dict[str, list[str]]:
    p_5f3: dict[str, list[str]] = {}
    for p_5f4 in rels:
        p_5f3.setdefault(stratum_of(p_5f4, level), []).append(p_5f4)
    p_5f2 = q_f1({k: len(p_5f5) for k, p_5f5 in p_5f3.items()}, target, min_per)
    return {k: q_13c(p_5f3[k], p_5f2.get(k, 0)) for k in sorted(p_5f3) if p_5f2.get(k, 0) > 0}

def q_11b(picked: dict[str, list[str]]) -> list[str]:
    p_5f6 = {k: list(p_5f7) for k, p_5f7 in picked.items() if p_5f7}
    out: list[str] = []
    while p_5f6:
        for k in sorted(p_5f6):
            out.append(p_5f6[k].pop(0))
        p_5f6 = {k: p_5f7 for k, p_5f7 in p_5f6.items() if p_5f7}
    return out

def q_122(silver_jsonl: Path) -> set[str]:
    done: set[str] = set()
    if not silver_jsonl.exists():
        return done
    for p_5f8 in silver_jsonl.open(encoding="utf-8"):
        p_5f8 = p_5f8.strip()
        if not p_5f8:
            continue
        try:
            done.add(json.loads(p_5f8)["document_id"])
        except (json.JSONDecodeError, KeyError):
            continue
    return done

def main(argv: list[str] | None = None) -> int:
    p_5f9 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_5f9.add_argument("data_dir")
    p_5f9.add_argument("--target", type=int, default=2500, help="c+G7kSBQREYgbXXhu5FuIGzhuqV5")
    p_5f9.add_argument("--level", type=int, default=2,
                    help="xJHhu5kgc8OidSB0aMawIG3hu6VjIGTDuW5nIGzDoG0gdOG6p25nICgyIOKHkiB+MTE5IHThuqduZywgbeG7i24gaMahbiAxKQ==")
    p_5f9.add_argument("--min-per", type=int, default=2, help="dOG7kWkgdGhp4buDdSBt4buXaSB04bqnbmcgKGdp4buvIHThuqduZyBuaOG7jyk=")
    p_5f9.add_argument("--exclude-done", default=None,
                    help="a2llX3NpbHZlci5qc29ubCDigJQgYuG7jyBjw6FjIFBERiDEkcOjIGLDs2M=")
    p_5f9.add_argument("--out", default="data/sample.txt")
    p_5f9.add_argument("--stats-only", action="store_true", help="Y2jhu4kgaW4gcGjDom4gYuG7lSwga2jDtG5nIGdoaSBmaWxl")
    p_5fa = p_5f9.parse_args(argv)

    data_dir = Path(p_5fa.data_dir)
    if not data_dir.is_dir():
        print(f"TOG7lkk6IGtow7RuZyB0aOG6pXkgdGjGsCBt4bulYyA={data_dir}", file=sys.stderr)
        return 2

    rels = [str(p_600.relative_to(data_dir)) for p_600 in data_dir.rglob("*")
            if p_600.suffix.lower() == ".pdf"]
    if not rels:
        print(f"TOG7lkk6IGtow7RuZyBjw7MgUERGIG7DoG8gdHJvbmcg{data_dir}", file=sys.stderr)
        return 2
    p_5fd = len(rels)

    p_5fe = 0
    if p_5fa.exclude_done:
        done = q_122(Path(p_5fa.exclude_done))
        rels = [p_601 for p_601 in rels if p_601 not in done]
        p_5fe = p_5fd - len(rels)
        if not rels:
            print("TOG7lkk6IG3hu41pIFBERiDEkeG7gXUgxJHDoyBiw7NjIOKAlCBraMO0bmcgY8OybiBnw6wgxJHhu4MgY2jhu41u", file=sys.stderr)
            return 2

    try:
        picked = sample(rels, p_5fa.target, p_5fa.level, p_5fa.min_per)
    except ValueError as e:
        print(f"TOG7lkk6IA=={e}", file=sys.stderr)
        return 2

    p_5ff = sum(len(p_602) for p_602 in picked.values())
    p_5fb = sorted((len(p_602) for p_602 in picked.values()), reverse=True)
    print(f"UERGIHThu5VuZzog{p_5fd}" + (f"ICjEkcOjIGLDs2Mg{p_5fe}LCBjw7JuIA=={len(rels)})" if p_5fa.exclude_done else ""),
          file=sys.stderr)
    print(f"dOG6p25nIChsZXZlbD0={p_5fa.level}): {len(picked)}IGPDsyBt4bqrdQ==", file=sys.stderr)
    print(f"Y2jhu41uOiA={p_5ff} PDF (target {p_5fa.target})", file=sys.stderr)
    print(f"beG7l2kgdOG6p25nOiBuaGnhu4F1IG5o4bqldCA={p_5fb[0]}LCDDrXQgbmjhuqV0IA=={p_5fb[-1]}, "
          f"trung vị {p_5fb[len(p_5fb)//2]}", file=sys.stderr)

    if p_5fa.stats_only:
        return 0

    out = Path(p_5fa.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8") as p_5fc:
        for p_601 in q_11b(picked):
            p_5fc.write(p_601 + "\n")
    print(f"→ {out} ({p_5ff} dòng, xen kẽ tầng ⇒ dừng giữa chừng vẫn cân bằng). "
          f"Dùng: extract_vlm.py {p_5fa.data_dir} --file-list {out} --only-header-pages",
          file=sys.stderr)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
