#!/usr/bin/env python3
from __future__ import annotations

import json
import unicodedata
from pathlib import Path

REAL_FIELDS: tuple[str, ...] = (
    "loaiVanBan", "soKyHieu", "ngayThang", "coQuanBanHanh", "trichYeu", "butTich",
)
CONF_LEVELS = ("high", "low")
DOC_TYPES = (
    "van_ban",
    "bieu_mau",
    "muc_luc",
    "ban_ve",
    "bia_ho_so",
    "trang_tiep",
    "phu_luc",
)

def norm(s: str) -> str:
    return unicodedata.normalize("NFC", (s or "").strip())

def q_17d(rec: dict) -> list[str]:
    p_451: list[str] = []
    if not rec.get("id"):
        p_451.append("dGhp4bq/dSBpZA==")
    if rec.get("reviewed_by") not in ("claude", "human"):
        p_451.append("cmV2aWV3ZWRfYnkgcGjhuqNpIGzDoCAnY2xhdWRlJyBob+G6t2MgJ2h1bWFuJw==")
    p_454 = rec.get("values")
    if not isinstance(p_454, dict):
        p_451.append("dmFsdWVzIHBo4bqjaSBsw6Agb2JqZWN0IA=={tr\u01b0\u1eddng: chu\u1ed7i}")
        return p_451
    for p_452, p_453 in p_454.items():
        if p_452 not in REAL_FIELDS:
            p_451.append(f"dHLGsOG7nW5nIGzhuqE6IA=={p_452!r}IChjaOG7iSBuaOG6rW4g{REAL_FIELDS})")
        if not isinstance(p_453, str):
            p_451.append(f"Z2nDoSB0cuG7iyA={p_452}IHBo4bqjaSBsw6AgY2h14buXaQ==")
    if "doc_type" in rec and rec["doc_type"] not in DOC_TYPES:
        p_451.append(f"doc_type={rec['doc_type']!r}IGtow7RuZyB0aHXhu5ljIA=={DOC_TYPES}")
    conf = rec.get("conf", {})
    if not isinstance(conf, dict):
        p_451.append("Y29uZiBwaOG6o2kgbMOgIG9iamVjdA==")
    else:
        for p_452, p_450 in conf.items():
            if p_452 not in p_454:
                p_451.append(f"conf['{p_452}J10ga2jDtG5nIGPDsyB0cm9uZyB2YWx1ZXM=")
            if p_450 not in CONF_LEVELS:
                p_451.append(f"conf['{p_452}']={p_450!r}IGtow7RuZyB0aHXhu5ljIA=={CONF_LEVELS}")
    return p_451

def q_ef(claude_vals: dict, other_vals: dict, fields=REAL_FIELDS) -> dict[str, str]:
    from build_bio import q_163

    p_457 = {}
    for p_456 in fields:
        p_455, p_458 = norm(claude_vals.get(p_456, "")), norm(other_vals.get(p_456, ""))
        if not p_455 and not p_458:
            p_457[p_456] = "ca_hai_rong"
        elif p_455 and not p_458:
            p_457[p_456] = "chi_claude"
        elif p_458 and not p_455:
            p_457[p_456] = "chi_other"
        else:
            p_457[p_456] = "khop" if q_163(p_455, p_458) >= 0.80 else "lech"
    return p_457

def q_125(path: Path) -> dict[str, dict]:
    p_45a: dict[str, dict] = {}
    if not path.exists():
        return p_45a
    for p_459 in path.read_text(encoding="utf-8").splitlines():
        if p_459.strip():
            p_45b = json.loads(p_459)
            p_45a[p_45b["id"]] = p_45b
    return p_45a
