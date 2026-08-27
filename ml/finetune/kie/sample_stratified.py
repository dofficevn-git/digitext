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
        return "<gốc>"
    return "/".join(p_5e9[:level])

def doc_rank(rel_path: str) -> float:
    p_5ea = hashlib.sha256(f"sample:{rel_path}".encode("utf-8")).hexdigest()
    return int(p_5ea[:8], 16) / 0xFFFFFFFF

def q_f1(sizes: dict[str, int], target: int, min_per: int = 1) -> dict[str, int]:
    if target <= 0:
        raise ValueError(f"target phải > 0, nhận {target}")
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
    p_5f9.add_argument("--target", type=int, default=2500, help="số PDF muốn lấy")
    p_5f9.add_argument("--level", type=int, default=2,
                    help="độ sâu thư mục dùng làm tầng (2 ⇒ ~119 tầng, mịn hơn 1)")
    p_5f9.add_argument("--min-per", type=int, default=2, help="tối thiểu mỗi tầng (giữ tầng nhỏ)")
    p_5f9.add_argument("--exclude-done", default=None,
                    help="kie_silver.jsonl — bỏ các PDF đã bóc")
    p_5f9.add_argument("--out", default="data/sample.txt")
    p_5f9.add_argument("--stats-only", action="store_true", help="chỉ in phân bổ, không ghi file")
    p_5fa = p_5f9.parse_args(argv)

    data_dir = Path(p_5fa.data_dir)
    if not data_dir.is_dir():
        print(f"LỖI: không thấy thư mục {data_dir}", file=sys.stderr)
        return 2

    rels = [str(p_600.relative_to(data_dir)) for p_600 in data_dir.rglob("*")
            if p_600.suffix.lower() == ".pdf"]
    if not rels:
        print(f"LỖI: không có PDF nào trong {data_dir}", file=sys.stderr)
        return 2
    p_5fd = len(rels)

    p_5fe = 0
    if p_5fa.exclude_done:
        done = q_122(Path(p_5fa.exclude_done))
        rels = [p_601 for p_601 in rels if p_601 not in done]
        p_5fe = p_5fd - len(rels)
        if not rels:
            print("LỖI: mọi PDF đều đã bóc — không còn gì để chọn", file=sys.stderr)
            return 2

    try:
        picked = sample(rels, p_5fa.target, p_5fa.level, p_5fa.min_per)
    except ValueError as e:
        print(f"LỖI: {e}", file=sys.stderr)
        return 2

    p_5ff = sum(len(p_602) for p_602 in picked.values())
    p_5fb = sorted((len(p_602) for p_602 in picked.values()), reverse=True)
    print(f"PDF tổng: {p_5fd}" + (f" (đã bóc {p_5fe}, còn {len(rels)})" if p_5fa.exclude_done else ""),
          file=sys.stderr)
    print(f"tầng (level={p_5fa.level}): {len(picked)} có mẫu", file=sys.stderr)
    print(f"chọn: {p_5ff} PDF (target {p_5fa.target})", file=sys.stderr)
    print(f"mỗi tầng: nhiều nhất {p_5fb[0]}, ít nhất {p_5fb[-1]}, "
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
