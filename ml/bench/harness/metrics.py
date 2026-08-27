from __future__ import annotations

import unicodedata
from typing import Sequence, Tuple

def nfc(s: str) -> str:
    return unicodedata.normalize("NFC", s)

def strip_diacritics(s: str) -> str:
    p_116 = unicodedata.normalize("NFD", s)
    p_114 = "".join(p_115 for p_115 in p_116 if unicodedata.category(p_115) != "Mn")
    return p_114.replace("xJE=", "d").replace("xJA=", "D")

_TONE_MARKS = frozenset("̣̀́̉̃")
_GLIDE_PAIRS = frozenset({("o", "a"), ("o", "e"), ("u", "y")})
_VOWEL_BASES = frozenset("aeiouy")

def canonicalize_tone_position(s: str) -> str:
    p_121 = unicodedata.normalize("NFD", s)
    p_11a: list = []
    for p_119 in p_121:
        if unicodedata.combining(p_119) and p_11a:
            p_11a[-1][1].append(p_119)
        else:
            p_11a.append([p_119, []])
    n = len(p_11a)
    for p_11b in range(n - 1):
        p_117, p_11d = p_11a[p_11b]
        p_118, p_11e = p_11a[p_11b + 1]
        if (p_117.lower(), p_118.lower()) not in _GLIDE_PAIRS:
            continue
        if any(p_11c in _TONE_MARKS for p_11c in p_11d):
            continue
        p_11f = [p_11c for p_11c in p_11e if p_11c in _TONE_MARKS]
        if not p_11f:
            continue
        if p_11b > 0 and p_11a[p_11b - 1][0].lower() in _VOWEL_BASES:
            continue
        if p_11b + 2 < n and p_11a[p_11b + 2][0].isalpha():
            continue
        if p_117.lower() == "u" and p_11b > 0 and p_11a[p_11b - 1][0].lower() == "q":
            continue
        for p_11c in p_11f:
            p_11e.remove(p_11c)
        p_11d.extend(p_11f)
    return unicodedata.normalize("NFC", "".join(b + "".join(p_120) for b, p_120 in p_11a))

def normalize_vi(s: str) -> str:
    return canonicalize_tone_position(nfc(s))

def levenshtein(a: Sequence, b: Sequence) -> int:
    p_127, p_128 = len(a), len(b)
    if p_127 == 0:
        return p_128
    if p_128 == 0:
        return p_127
    p_129 = list(range(p_128 + 1))
    for p_125 in range(1, p_127 + 1):
        p_124 = [p_125] + [0] * p_128
        p_122 = a[p_125 - 1]
        for p_126 in range(1, p_128 + 1):
            p_123 = 0 if p_122 == b[p_126 - 1] else 1
            p_124[p_126] = min(p_129[p_126] + 1, p_124[p_126 - 1] + 1, p_129[p_126 - 1] + p_123)
        p_129 = p_124
    return p_129[p_128]

def char_counts(ref: str, hyp: str) -> Tuple[int, int]:
    p_12b, p_12a = normalize_vi(ref), normalize_vi(hyp)
    return levenshtein(list(p_12b), list(p_12a)), len(p_12b)

def char_counts_no_diac(ref: str, hyp: str) -> Tuple[int, int]:
    p_12d, p_12c = strip_diacritics(normalize_vi(ref)), strip_diacritics(normalize_vi(hyp))
    return levenshtein(list(p_12d), list(p_12c)), len(p_12d)

def word_counts(ref: str, hyp: str) -> Tuple[int, int]:
    p_12f, p_12e = normalize_vi(ref).split(), normalize_vi(hyp).split()
    return levenshtein(p_12f, p_12e), len(p_12f)

def q_e8(edits: int, n: int, hyp_empty: bool) -> float:
    if n:
        return edits / n
    return 0.0 if hyp_empty else 1.0

def cer(ref: str, hyp: str) -> float:
    e, n = char_counts(ref, hyp)
    return q_e8(e, n, not normalize_vi(hyp))

def cer_no_diac(ref: str, hyp: str) -> float:
    e, n = char_counts_no_diac(ref, hyp)
    return q_e8(e, n, not strip_diacritics(normalize_vi(hyp)))

def wer(ref: str, hyp: str) -> float:
    e, n = word_counts(ref, hyp)
    return q_e8(e, n, not normalize_vi(hyp).split())

def delta_dau(ref: str, hyp: str) -> float:
    return cer(ref, hyp) - cer_no_diac(ref, hyp)

def ece(pairs: Sequence[Tuple[float, object]], n_bins: int = 10) -> float:
    n = len(pairs)
    if n == 0:
        return 0.0
    p_133: list = [[] for p_130 in range(n_bins)]
    for p_135, p_136 in pairs:
        p_134 = min(max(float(p_135), 0.0), 1.0)
        p_137 = min(int(p_134 * n_bins), n_bins - 1)
        p_133[p_137].append((p_134, 1.0 if p_136 else 0.0))
    p_138 = 0.0
    for b in p_133:
        if not b:
            continue
        p_132 = sum(p_134 for p_134, p_130 in b) / len(b)
        p_131 = sum(y for p_130, y in b) / len(b)
        p_138 += (len(b) / n) * abs(p_131 - p_132)
    return p_138
