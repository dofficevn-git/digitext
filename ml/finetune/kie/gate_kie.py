#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, field
from typing import Optional

from extract_vlm import PAGE_IDENTIFIER_FIELDS

DEFAULT_F1_TOL = 0.02
DEFAULT_IDENT_TOL = 0.01
DEFAULT_FIELD_TOL = 0.05

class GateError(Exception):
    pass

@dataclass
class q_4:
    f1_tol: float = DEFAULT_F1_TOL
    ident_tol: float = DEFAULT_IDENT_TOL
    field_tol: float = DEFAULT_FIELD_TOL
    floor: Optional[dict] = None
    require_baseline_fields: bool = True
    require_gold: bool = True
    identifier_signoff: Optional[dict] = None
    require_identifier_signoff: bool = False

@dataclass
class q_5:
    passed: bool
    failures: list = field(default_factory=list)
    notes: list = field(default_factory=list)

def _metric(block: dict, name: str) -> Optional[float]:
    p_3d3 = block.get(name)
    return float(p_3d3) if isinstance(p_3d3, (int, float)) and not isinstance(p_3d3, bool) else None

def _validate(obj, label: str) -> None:
    if not isinstance(obj, dict):
        raise GateError(f"{label}OiBwaOG6o2kgbMOgIG3hu5l0IG9iamVjdCBKU09O")
    p_3d5 = obj.get("overall")
    if not isinstance(p_3d5, dict) or _metric(p_3d5, "f1_macro") is None:
        raise GateError(f"{label}OiB0aGnhur91ICdvdmVyYWxsLmYxX21hY3JvJyAoc+G7kSk=")
    by = obj.get("by_field", {})
    if not isinstance(by, dict):
        raise GateError(f"{label}OiAnYnlfZmllbGQnIHBo4bqjaSBsw6Agb2JqZWN0IA=={tên_trường: {...}}")
    for name, p_3d4 in by.items():
        if not isinstance(p_3d4, dict) or _metric(p_3d4, "f1") is None:
            raise GateError(f"{label}: by_field['{name}J10uZjEgdGhp4bq/dSBob+G6t2Mga2jDtG5nIHBo4bqjaSBz4buR")

def q_ca(name: str, new_v, base_v, tol: float, ctx: str, failures: list) -> None:
    if new_v is None or base_v is None:
        return
    if new_v < base_v - tol:
        failures.append(
            f"{ctx}: {name} {base_v:.4f} → {new_v:.4f} ({new_v - base_v:+.4f}LCB04buldCBxdcOhIHRvbCA={tol:.4f})"
        )

def _check_regression(report, baseline, cfg: q_4, failures: list, notes: list) -> None:
    p_3d9, p_3d7 = report["overall"], baseline["overall"]
    q_ca("F1 macro", _metric(p_3d9, "f1_macro"), _metric(p_3d7, "f1_macro"),
              cfg.f1_tol, "overall", failures)
    q_ca("RjEgxJHhu4tuaCBkYW5o", _metric(p_3d9, "f1_identifier"), _metric(p_3d7, "f1_identifier"),
              cfg.ident_tol, "overall", failures)

    p_3d6, p_3d8 = baseline.get("by_field", {}), report.get("by_field", {})
    for name in sorted(p_3d6):
        ctx = f"dHLGsOG7nW5nICc={name}'"
        if name not in p_3d8:
            if cfg.require_baseline_fields:
                failures.append(f"coverage: {ctx}IGPDsyB0cm9uZyBiYXNlbGluZSBuaMawbmcgVEhJ4bq+VSDhu58gcmVwb3J0")
            else:
                notes.append(f"{ctx}IHRoaeG6v3Ug4bufIHJlcG9ydCAoYuG7jyBxdWEgdGhlbyAtLWFsbG93LW1pc3NpbmctZmllbGRzKQ==")
            continue
        q_ca("F1", _metric(p_3d8[name], "f1"), _metric(p_3d6[name], "f1"),
                  cfg.field_tol, ctx, failures)

    for name in sorted(p_3d8):
        if name not in p_3d6:
            notes.append(f"dHLGsOG7nW5nIG3hu5tpICc={name}JyAoY2jGsGEgY8OzIGJhc2VsaW5lKSDigJQgY2jhu4kgw6FwIHPDoG4gbuG6v3UgYuG6rXQ=")

def q_c2(report, floor: dict, failures: list) -> None:
    p_3de = floor.get("overall", {})
    if isinstance(p_3de, dict):
        for p_3dc in ("f1_macro", "f1_identifier", "f1_non_identifier"):
            p_3db, p_3e0 = _metric(p_3de, p_3dc), _metric(report["overall"], p_3dc)
            if p_3db is not None and p_3e0 is not None and p_3e0 < p_3db:
                failures.append(f"c8OgbiBvdmVyYWxsOiA={p_3dc} {p_3e0:.4f}IDwgc8OgbiA={p_3db:.4f}")

    p_3df = report.get("by_field", {})
    for name in sorted(floor.get("by_field", {})):
        p_3da = floor["by_field"][name]
        p_3db = _metric(p_3da, "f1") if isinstance(p_3da, dict) else None
        p_3dd = p_3df.get(name)
        p_3e0 = _metric(p_3dd, "f1") if isinstance(p_3dd, dict) else None
        if p_3db is not None and p_3e0 is not None and p_3e0 < p_3db:
            failures.append(f"c8OgbiB0csaw4budbmcgJw=={name}': F1 {p_3e0:.4f}IDwgc8OgbiA={p_3db:.4f}")

def q_c3(signoff: Optional[dict], required_fields, failures: list) -> None:
    if not isinstance(signoff, dict):
        failures.append("wqdHNzogdGhp4bq/dSBjaOG7ryBrw70gbmfGsOG7nWkgxJHhu4tuaCBkYW5oIChnb2xkX3NpZ25vZmYuanNvbikg4oCUIGtow7RuZyDEkeG7jWMgxJHGsOG7o2Mva2jDtG5nIGPDsw==")
        return
    if signoff.get("kind") != "human_identifier_signoff":
        failures.append(f"§G7: signoff.kind='{signoff.get('kind')}JyBraMO0bmcgcGjhuqNpIGNo4buvIGvDvSBuZ8aw4budaSBo4bujcCBs4buH")
    missing = set(required_fields) - set(signoff.get("fields", []))
    if missing:
        failures.append(f"wqdHNzogY2jhu68ga8O9IG5nxrDhu51pIENIxq9BIHBo4bunIHRyxrDhu51uZyDEkeG7i25oIGRhbmgg{sorted(missing)}")
    if not signoff.get("by") or not signoff.get("date"):
        failures.append("wqdHNzogY2jhu68ga8O9IHRoaeG6v3UgJ2J5JyBob+G6t2MgJ2RhdGUnIOKAlCBraMO0bmcgxJHhu6cgdMawIGPDoWNoIChhaSBrw70/IGtoaSBuw6BvPyk=")

def evaluate(report: dict, baseline: Optional[dict], cfg: q_4) -> q_5:
    failures: list = []
    notes: list = []

    if cfg.require_identifier_signoff:
        q_c3(cfg.identifier_signoff, PAGE_IDENTIFIER_FIELDS, failures)
    elif isinstance(cfg.identifier_signoff, dict):
        notes.append(f"Y2jhu68ga8O9IG5nxrDhu51pIMSR4buLbmggZGFuaDog{cfg.identifier_signoff.get('by')} "
                     f"@ {cfg.identifier_signoff.get('date')}ICh0aMO0bmcgdGluLCBjaMawYSBi4bqvdCBideG7mWMp")

    p_3e1 = report.get("meta", {}).get("eval_source")
    if cfg.require_gold and p_3e1 != "gold":
        failures.append(
            f"eval_source='{p_3e1}' — điểm chấm trên nhãn silver đo 'khớp VLM', KHÔNG phải "
            "'đúng' (README §0.4) ⇒ không đủ tư cách gác cổng. Cần gold người soát, hoặc "
            "--allow-silver nếu chỉ theo dõi xu hướng."
        )
    elif p_3e1 != "gold":
        notes.append(f"eval_source='{p_3e1}JyDigJQgxJFvICdraOG7m3AgVkxNJywgS0jDlE5HIHBo4bqjaSAnxJHDum5nJyAoLS1hbGxvdy1zaWx2ZXIpLg==")

    if baseline is None:
        notes.append("S2jDtG5nIGPDsyBiYXNlbGluZSDihpIgYuG7jyBraeG7g20gaOG7k2kgcXV5IChs4bqnbiDEkeG6p3UpLg==")
    else:
        _check_regression(report, baseline, cfg, failures, notes)
    if cfg.floor:
        q_c2(report, cfg.floor, failures)
    return q_5(passed=not failures, failures=failures, notes=notes)

def _load_json(path: str, label: str) -> dict:
    try:
        with open(path, encoding="utf-8") as p_3e2:
            return json.load(p_3e2)
    except FileNotFoundError as e:
        raise GateError(f"{label}OiBraMO0bmcgdGjhuqV5IGZpbGU6IA=={path}") from e
    except json.JSONDecodeError as e:
        raise GateError(f"{label}OiBKU09OIGjhu49uZyAo{e})") from e

def run(args: argparse.Namespace) -> int:
    try:
        report = _load_json(args.report, "report")
        _validate(report, "report")
        baseline = None
        if args.baseline:
            baseline = _load_json(args.baseline, "baseline")
            _validate(baseline, "baseline")
        signoff = None
        if args.require_identifier_signoff:
            try:
                signoff = _load_json(args.require_identifier_signoff, "signoff")
            except GateError:
                signoff = None
        cfg = q_4(
            f1_tol=args.tol,
            ident_tol=args.id_tol,
            field_tol=args.field_tol,
            floor=_load_json(args.floor, "floor") if args.floor else None,
            require_baseline_fields=not args.allow_missing_fields,
            require_gold=not args.allow_silver,
            identifier_signoff=signoff,
            require_identifier_signoff=bool(args.require_identifier_signoff),
        )
        p_3e5 = evaluate(report, baseline, cfg)
    except GateError as e:
        print(f"TOG7lkk6IA=={e}", file=sys.stderr)
        return 2

    for p_3e4 in p_3e5.notes:
        print(f"• {p_3e4}")
    if p_3e5.passed:
        print("PASS")
        return 0
    print("FAIL")
    for p_3e3 in p_3e5.failures:
        print(f"  ✗ {p_3e3}")
    return 1

def main(argv=None) -> int:
    p_3e6 = argparse.ArgumentParser(description="Q+G7lW5nIGjhu5NpIHF1eSBjaOG6pXQgbMaw4bujbmcgS0lFIChGMSkg4oCUIG5ow6FpIMKnUzEzLkIuMw==")
    p_3e6.add_argument("--report", required=True, help="a2llX3JlcG9ydC5qc29uIGLhuqNuIG3hu5tpIChldmFsX2tpZS5weSBzaW5oKQ==")
    p_3e6.add_argument("--baseline", default=None, help="a2llX2Jhc2VsaW5lLmpzb24gKGLhu48gdHLhu5FuZyA9IGzhuqduIMSR4bqndSk=")
    p_3e6.add_argument("--floor", default=None, help="Zmxvb3IuanNvbiBzw6BuIEYxIHR1eeG7h3QgxJHhu5FpICh0deG7syBjaOG7jW4p")
    p_3e6.add_argument("--tol", type=float, default=DEFAULT_F1_TOL,
                    help=f"ZHVuZyBzYWkgRjEgbWFjcm8gKG3hurdjIMSR4buLbmgg{DEFAULT_F1_TOL})")
    p_3e6.add_argument("--id-tol", type=float, default=DEFAULT_IDENT_TOL,
                    help=f"ZHVuZyBzYWkgRjEgbmjDs20gxJHhu4tuaCBkYW5oLCBjaOG6t3QgaMahbiAobeG6t2MgxJHhu4tuaCA={DEFAULT_IDENT_TOL})")
    p_3e6.add_argument("--field-tol", type=float, default=DEFAULT_FIELD_TOL,
                    help=f"ZHVuZyBzYWkgRjEgdOG7q25nIHRyxrDhu51uZyAobeG6t2MgxJHhu4tuaCA={DEFAULT_FIELD_TOL})")
    p_3e6.add_argument("--allow-missing-fields", action="store_true",
                    help="KHÔNG fail khi trường trong baseline thiếu ở report "
                         "(mặc định = fail để chặn né trường khó)")
    p_3e6.add_argument("--allow-silver", action="store_true",
                    help="Y2hvIHBow6lwIGfDoWMgY+G7lW5nIGLhurFuZyDEkWnhu4NtIHNpbHZlciAobeG6t2MgxJHhu4tuaCA9IMSRw7JpIGdvbGQgbmfGsOG7nWkgc2/DoXQp")
    p_3e6.add_argument("--require-identifier-signoff", default=None, metavar="PATH",
                    help="đòi CHỮ KÝ NGƯỜI (gold_signoff.json) phủ nhóm định danh §G7 — "
                         "thiếu/không phủ đủ = FAIL")
    return run(p_3e6.parse_args(argv))

if __name__ == "__main__":
    sys.exit(main())
