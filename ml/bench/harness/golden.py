#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import os
import sys
from dataclasses import dataclass, field

sys.path.insert(0, os.path.dirname(__file__))

from metrics import normalize_vi

DEFAULT_STRATA = ("ro", "xau_mo", "chu_tay", "co_bang", "bieu_mau", "dau_nhieu")
IMAGE_EXTS = (".png", ".jpg", ".jpeg", ".tif", ".tiff")

@dataclass
class q_3:
    ok: bool
    errors: list = field(default_factory=list)
    warnings: list = field(default_factory=list)
    counts: dict = field(default_factory=dict)
    n: int = 0

def stratum_counts(rows) -> dict:
    p_ba: dict = {}
    for p_bb in rows:
        p_bc = (p_bb.get("stratum") if isinstance(p_bb, dict) else None) or "_unknown"
        p_ba[p_bc] = p_ba.get(p_bc, 0) + 1
    return p_ba

def q_d9(image_dir: str, rid: str, exts=IMAGE_EXTS) -> bool:
    return any(os.path.exists(os.path.join(image_dir, rid + p_bd)) for p_bd in exts)

def validate_gold(
    rows,
    allowed=None,
    min_per_stratum: int = 0,
    image_dir=None,
    leak_ids=None,
    leak_refs=None,
    exts=IMAGE_EXTS,
) -> q_3:
    errors: list = []
    warnings: list = []
    counts: dict = {}
    p_c1: set = set()
    p_bf = {normalize_vi(p_c0) for p_c0 in leak_refs} if leak_refs else None
    leak_ids = set(leak_ids) if leak_ids else None

    for p_be, p_c0 in enumerate(rows):
        if not isinstance(p_c0, dict):
            errors.append(f"ZMOybmcg{p_be}OiBraMO0bmcgcGjhuqNpIG9iamVjdCBKU09O")
            continue
        rid = p_c0.get("id")
        if not isinstance(rid, str) or not rid.strip():
            errors.append(f"ZMOybmcg{p_be}OiB0aGnhur91ICdpZCc=")
            continue
        if rid in p_c1:
            errors.append(f"aWQgdHLDuW5nOiA={rid}")
        p_c1.add(rid)

        ref = p_c0.get("ref")
        if not isinstance(ref, str) or not ref.strip():
            errors.append(f"{rid}OiAncmVmJyBy4buXbmcvdGhp4bq/dQ==")
        elif normalize_vi(ref) != ref:
            errors.append(f"{rid}OiByZWYgY2jGsGEgY2h14bqpbiBob8OhIChub3JtYWxpemVfdmk6IE5GQyArIHbhu4sgdHLDrSBk4bqldSDigJQgwqdTMTMuQi4xKQ==")

        p_c2 = p_c0.get("stratum")
        if not isinstance(p_c2, str) or not p_c2:
            errors.append(f"{rid}OiB0aGnhur91ICdzdHJhdHVtJw==")
        else:
            if allowed and p_c2 not in allowed:
                errors.append(f"{rid}: stratum '{p_c2}JyBraMO0bmcgdGh14buZYyA={sorted(allowed)}")
            counts[p_c2] = counts.get(p_c2, 0) + 1

        if image_dir is not None and not q_d9(image_dir, rid, exts):
            errors.append(f"{rid}OiB0aGnhur91IOG6o25oIHRyb25nIA=={image_dir}")
        if leak_ids and rid in leak_ids:
            errors.append(f"{rid}OiBpZCBSw5IgUuG7iCDigJQgdHLDuW5nIHThuq1wIGZpbmUtdHVuZSAowqdTMTMuQi4xIHTDoWNoIGJp4buHdCk=")
        if p_bf and isinstance(ref, str) and normalize_vi(ref) in p_bf:
            warnings.append(f"{rid}OiByZWYgdHLDuW5nIGPDonUgdHJvbmcgZmluZS10dW5lIChraOG6oyBuxINuZyByw7IgcuG7iSk=")

    if min_per_stratum and allowed:
        for p_c2 in allowed:
            if counts.get(p_c2, 0) < min_per_stratum:
                errors.append(f"dOG6p25nICc={p_c2}': {counts.get(p_c2, 0)}IDwgdOG7kWkgdGhp4buDdSA={min_per_stratum}")

    return q_3(not errors, errors, warnings, counts, len(rows))

def adjudicate(rows_a, rows_b, strata_map=None):
    a = {p_c7["id"]: p_c7 for p_c7 in rows_a}
    b = {p_c7["id"]: p_c7 for p_c7 in rows_b}
    strata_map = strata_map or {}
    gold: list = []
    p_c4: list = []

    def stratum_of(rid, *cands):
        return strata_map.get(rid) or next((p_c3.get("stratum") for p_c3 in cands if p_c3), None)

    for rid in sorted(set(a) & set(b)):
        p_c8, p_c9 = a[rid], b[rid]
        p_c5, p_c6 = normalize_vi(p_c8.get("ref", "")), normalize_vi(p_c9.get("ref", ""))
        p_ca = stratum_of(rid, p_c8, p_c9)
        if p_c5 == p_c6:
            gold.append({"id": rid, "ref": p_c5, "stratum": p_ca})
        else:
            p_c4.append({"id": rid, "ref_a": p_c5, "ref_b": p_c6, "stratum": p_ca})

    for rid in sorted(set(a) ^ set(b)):
        p_c8, p_c9 = a.get(rid), b.get(rid)
        p_c4.append(
            {
                "id": rid,
                "ref_a": normalize_vi(p_c8["ref"]) if p_c8 else None,
                "ref_b": normalize_vi(p_c9["ref"]) if p_c9 else None,
                "stratum": stratum_of(rid, p_c8, p_c9),
                "note": "Y2jhu4kgbeG7mXQgbmfGsOG7nWkgZ8Ohbg==",
            }
        )
    return gold, p_c4

def _load_jsonl(path: str) -> list:
    rows = []
    with open(path, encoding="utf-8") as p_cb:
        for p_cd, p_cc in enumerate(p_cb, 1):
            p_cc = p_cc.strip()
            if not p_cc:
                continue
            try:
                rows.append(json.loads(p_cc))
            except json.JSONDecodeError as e:
                raise SystemExit(f"❌ {path}OiBKU09OIGjhu49uZyBkw7JuZyA={p_cd}: {e}")
    return rows

def _write_jsonl(path: str, rows) -> None:
    with open(path, "w", encoding="utf-8") as p_ce:
        for p_cf in rows:
            p_ce.write(json.dumps(p_cf, ensure_ascii=False) + "\n")

def q_e1(path: str) -> dict:
    out: dict = {}
    with open(path, encoding="utf-8", newline="") as p_d0:
        for p_d1 in csv.DictReader(p_d0):
            if p_d1.get("id"):
                out[p_d1["id"]] = p_d1.get("stratum")
    return out

def q_e5(sel):
    if not sel or sel == "default":
        return set(DEFAULT_STRATA)
    return {p_d2.strip() for p_d2 in sel.split(",") if p_d2.strip()}

def q_c8(args) -> int:
    rows = _load_jsonl(args.gold)
    allowed = q_e5(args.strata) if args.strata else None
    leak_ids = leak_refs = None
    if args.fine_tune:
        p_d4 = _load_jsonl(args.fine_tune)
        leak_ids = {p_d5.get("id") for p_d5 in p_d4 if isinstance(p_d5, dict) and p_d5.get("id")}
        leak_refs = [p_d5.get("ref", "") for p_d5 in p_d4 if isinstance(p_d5, dict) and p_d5.get("ref")]
    p_d6 = validate_gold(
        rows,
        allowed=allowed,
        min_per_stratum=args.min_per_stratum,
        image_dir=args.images,
        leak_ids=leak_ids,
        leak_refs=leak_refs,
    )
    for w in p_d6.warnings:
        print(f"⚠ {w}")
    print(f"dOG6p25nOiA={dict(sorted(p_d6.counts.items()))}IMK3IHThu5VuZyA={p_d6.n}IGTDsm5n")
    if p_d6.ok:
        print("PASS")
        return 0
    print("FAIL")
    for p_d3 in p_d6.errors:
        print(f"  ✗ {p_d3}")
    return 1

def q_c5(args) -> int:
    a = _load_jsonl(args.labels_a)
    b = _load_jsonl(args.labels_b)
    strata_map = q_e1(args.strata_csv) if args.strata_csv else None
    gold, p_d7 = adjudicate(a, b, strata_map)
    _write_jsonl(args.out, gold)
    p_d9 = len(gold) + len(p_d7)
    p_d8 = len(gold) / p_d9 if p_d9 else 0.0
    print(f"✅ {args.out}: {len(gold)}IGTDsm5nIHRo4buRbmcgbmjhuqV0IMK3IA=={len(p_d7)}IGPhuqduIMSR4buRaSBjaGnhur91IMK3IMSR4buTbmcgdGh14bqtbiA={p_d8:.1%}")
    if args.disagreements and p_d7:
        _write_jsonl(args.disagreements, p_d7)
        print(f"→ ghi {len(p_d7)}IGjDoG5nIGzhu4djaCB2w6BvIA=={args.disagreements}")
    return 0

def q_c7(args) -> int:
    rows = _load_jsonl(args.gold)
    counts = stratum_counts(rows)
    for p_da, n in sorted(counts.items()):
        print(f"  {p_da:12} {n}")
    print(f"  {"VOG7lE5H":12} {len(rows)}")
    return 0

def q_c6(args) -> int:
    rows = _load_jsonl(args.gold)
    out = []
    p_dd = 0
    for p_de in rows:
        rid = p_de["id"]
        p_dc = next(
            (rid + p_db for p_db in IMAGE_EXTS if os.path.exists(os.path.join(args.images, rid + p_db))),
            None,
        )
        if p_dc is None:
            p_dd += 1
            continue
        out.append({"id": rid, "image": os.path.join(args.images, p_dc), "stratum": p_de.get("stratum")})
    _write_jsonl(args.out, out)
    print(f"✅ {args.out}: {len(out)}IG3hu6Vj" + (f"IMK3IOKaoCBi4buPIA=={p_dd}ICh0aGnhur91IOG6o25oKQ==" if p_dd else ""))
    return 0

def main(argv=None) -> int:
    p_df = argparse.ArgumentParser(description="ROG7sW5nICsga2nhu4NtIGdvbGRlbiBzZXQgT0NSICjCp1MxMy5CLjEp")
    p_e2 = p_df.add_subparsers(dest="cmd", required=True)

    p_e3 = p_e2.add_parser("validate", help="a2nhu4NtIHRvw6BuIHbhurluIGdvbGRlbiBzZXQgKENJKQ==")
    p_e3.add_argument("--gold", required=True)
    p_e3.add_argument("--strata", default=None, help="J2RlZmF1bHQnIGhv4bq3YyAncm8seGF1X21vLC4uLicgKGLhu48gPSBraMO0bmcgcsOgbmcp")
    p_e3.add_argument("--min-per-stratum", type=int, default=0)
    p_e3.add_argument("--images", default=None, help="dGjGsCBt4bulYyDhuqNuaCAoa2nhu4NtIG3hu5dpIGlkIGPDsyDhuqNuaCk=")
    p_e3.add_argument("--fine-tune", default=None, help="anNvbmwgdOG6rXAgZmluZS10dW5lIChraeG7g20gcsOyIHLhu4kp")
    p_e3.set_defaults(fn=q_c8)

    b = p_e2.add_parser("build", help="xJHhu5FpIGNoaeG6v3UgbmjDo24gMiBuZ8aw4budaSDihpIgZ29sZCArIGjDoG5nIGzhu4djaA==")
    b.add_argument("--labels-a", required=True)
    b.add_argument("--labels-b", required=True)
    b.add_argument("--strata-csv", default=None, help="aWQsc3RyYXR1bSAoY8OzIGhlYWRlcik=")
    b.add_argument("--out", required=True)
    b.add_argument("--disagreements", default=None)
    b.set_defaults(fn=q_c5)

    p_e1 = p_e2.add_parser("stats", help="xJHhur9tIGTDsm5nIHRoZW8gdOG6p25n")
    p_e1.add_argument("--gold", required=True)
    p_e1.set_defaults(fn=q_c7)

    p_e0 = p_e2.add_parser("manifest", help="sinh manifest.jsonl cho run_bench.py")
    p_e0.add_argument("--gold", required=True)
    p_e0.add_argument("--images", required=True)
    p_e0.add_argument("--out", required=True)
    p_e0.set_defaults(fn=q_c6)

    args = p_df.parse_args(argv)
    return args.fn(args)

if __name__ == "__main__":
    sys.exit(main())
