#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import sys
from typing import FrozenSet, Optional, Tuple

sys.path.insert(0, os.path.dirname(__file__))

from metrics import normalize_vi, strip_diacritics

DIACRITIC_SUB = 0.2
HOMOGLYPH_SUB = 0.3
BASE_SUB = 1.0
INS_DEL = 1.0

_HOMOGLYPH_GROUPS = [{"O", "0"}, {"l", "1", "I"}, {"S", "5"}, {"B", "8"}]
_HOMOGLYPH: dict = {}
for _g in _HOMOGLYPH_GROUPS:
    for _ch in _g:
        _HOMOGLYPH[_ch] = _g

def q_dd(a: str, b: str) -> bool:
    return a in _HOMOGLYPH and b in _HOMOGLYPH[a]

def weighted_cost(a: str, b: str) -> float:
    if a == b:
        return 0.0
    if strip_diacritics(a).lower() == strip_diacritics(b).lower():
        return DIACRITIC_SUB
    if q_dd(a, b):
        return HOMOGLYPH_SUB
    return BASE_SUB

def weighted_levenshtein(a: str, b: str) -> float:
    p_8e, p_8f = len(a), len(b)
    if p_8e == 0:
        return p_8f * INS_DEL
    if p_8f == 0:
        return p_8e * INS_DEL
    p_90 = [p_8d * INS_DEL for p_8d in range(p_8f + 1)]
    for p_8c in range(1, p_8e + 1):
        p_8b = [p_8c * INS_DEL] + [0.0] * p_8f
        p_8a = a[p_8c - 1]
        for p_8d in range(1, p_8f + 1):
            p_8b[p_8d] = min(
                p_90[p_8d] + INS_DEL,
                p_8b[p_8d - 1] + INS_DEL,
                p_90[p_8d - 1] + weighted_cost(p_8a, b[p_8d - 1]),
            )
        p_90 = p_8b
    return p_90[p_8f]

def nearest_in_lexicon(
    token: str, lexicon: FrozenSet[str], max_cost: float = 0.6
) -> Optional[Tuple[str, float]]:
    p_91: Optional[str] = None
    p_92: Optional[float] = None
    for p_94 in sorted(lexicon):
        if abs(len(p_94) - len(token)) * INS_DEL > max_cost:
            continue
        p_93 = weighted_levenshtein(token, p_94)
        if p_93 <= max_cost and (p_92 is None or p_93 < p_92):
            p_91, p_92 = p_94, p_93
    return (p_91, p_92) if p_91 is not None else None

def correct_token(
    tok: str,
    conf: float,
    lexicon: FrozenSet[str],
    gazetteer: FrozenSet[str] = frozenset(),
    conf_gate: float = 0.85,
    max_cost: float = 0.6,
    protect_numeric: bool = True,
) -> Tuple[str, bool]:
    p_98 = normalize_vi(tok)
    if protect_numeric and any(p_95.isdigit() for p_95 in p_98):
        return p_98, False
    p_97 = p_98.lower()
    if conf >= conf_gate or p_97 in lexicon or p_97 in gazetteer:
        return p_98, False
    p_96 = nearest_in_lexicon(p_97, lexicon, max_cost)
    if p_96:
        return p_96[0], True
    return p_98, False

def correct_line(
    line: str,
    conf: float,
    lexicon: FrozenSet[str],
    gazetteer: FrozenSet[str] = frozenset(),
    conf_gate: float = 0.85,
    max_cost: float = 0.6,
    protect_numeric: bool = True,
) -> Tuple[str, int]:
    p_9c = []
    p_9b = 0
    for tok in normalize_vi(line).split():
        p_99, p_9a = correct_token(
            tok, conf, lexicon, gazetteer, conf_gate, max_cost, protect_numeric
        )
        p_9c.append(p_99)
        p_9b += 1 if p_9a else 0
    return " ".join(p_9c), p_9b

def load_lexicon(path: str) -> FrozenSet[str]:
    p_9f = set()
    with open(path, encoding="utf-8") as p_9d:
        for line in p_9d:
            p_9e = normalize_vi(line.strip()).lower()
            if p_9e:
                p_9f.add(p_9e)
    return frozenset(p_9f)

def main(argv=None) -> int:
    p_a0 = argparse.ArgumentParser(description="Sửa OCR theo confusables + lexicon (§S13.A.2/A.3)")
    p_a0.add_argument("--lexicon", required=True, help="file từ điển (mỗi dòng một từ, đã normalize)")
    p_a0.add_argument("--gazetteer", default=None, help="file địa danh/tên riêng cần bảo vệ (tuỳ chọn)")
    p_a0.add_argument("--text", required=True, help="dòng cần thử sửa")
    p_a0.add_argument("--conf", type=float, default=0.5, help="confidence của dòng (thấp → được sửa)")
    p_a0.add_argument("--conf-gate", type=float, default=0.85)
    p_a0.add_argument("--max-cost", type=float, default=0.6)
    p_a1 = p_a0.parse_args(argv)

    p_a4 = load_lexicon(p_a1.lexicon)
    p_a3 = load_lexicon(p_a1.gazetteer) if p_a1.gazetteer else frozenset()
    p_a2, p_a5 = correct_line(p_a1.text, p_a1.conf, p_a4, p_a3, p_a1.conf_gate, p_a1.max_cost)
    print(f"gốc:  {p_a1.text}")
    print(f"sửa:  {p_a2}   ({p_a5} token đổi)")
    return 0

if __name__ == "__main__":
    sys.exit(main())
