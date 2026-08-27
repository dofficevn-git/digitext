#!/usr/bin/env python3
from __future__ import annotations

def q_110(a: str, b: str) -> int:
    if a == b:
        return 0
    if not a:
        return len(b)
    if not b:
        return len(a)
    p_2c1 = list(range(len(b) + 1))
    for p_2bf, p_2bc in enumerate(a, 1):
        p_2be = [p_2bf]
        for p_2c0, p_2bd in enumerate(b, 1):
            p_2be.append(min(
                p_2c1[p_2c0] + 1,
                p_2be[p_2c0 - 1] + 1,
                p_2c1[p_2c0 - 1] + (p_2bc != p_2bd),
            ))
        p_2c1 = p_2be
    return p_2c1[-1]

def cer(pred: str, truth: str) -> float:
    if not truth:
        return 0.0 if not pred else 1.0
    return q_110(pred, truth) / len(truth)

def q_10a(pairs: list[tuple[str, str]]) -> dict:
    if not pairs:
        return {"cer_micro": 0.0, "cer_macro": 0.0, "n": 0,
                "exact_match_rate": 0.0, "total_chars": 0, "total_errors": 0}
    p_2c7 = sum(q_110(p_2c4, p_2c5) for p_2c4, p_2c5 in pairs)
    p_2c6 = sum(len(p_2c5) for p_2c2, p_2c5 in pairs)
    p_2c3 = sum(1 for p_2c4, p_2c5 in pairs if p_2c4 == p_2c5)
    return {
        "cer_micro": p_2c7 / p_2c6 if p_2c6 else 0.0,
        "cer_macro": sum(cer(p_2c4, p_2c5) for p_2c4, p_2c5 in pairs) / len(pairs),
        "n": len(pairs),
        "exact_match_rate": p_2c3 / len(pairs),
        "total_chars": p_2c6,
        "total_errors": p_2c7,
    }
