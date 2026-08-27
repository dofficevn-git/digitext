#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from typing import Optional

DEFAULT_CER_TOL = 0.005

class GateError(Exception):
    pass

@dataclass
class GateConfig:
    cer_tol: float = DEFAULT_CER_TOL
    wer_tol: Optional[float] = None
    dau_tol: Optional[float] = None
    ece_tol: Optional[float] = None
    slo: Optional[dict] = None
    require_baseline_strata: bool = True

@dataclass
class q_2:
    passed: bool
    failures: list
    notes: list

def _metric(block: dict, name: str) -> Optional[float]:
    p_a6 = block.get(name)
    return float(p_a6) if isinstance(p_a6, (int, float)) and not isinstance(p_a6, bool) else None

def q_d0(block: dict) -> Optional[float]:
    p_a7 = _metric(block, "delta_dau")
    if p_a7 is not None:
        return p_a7
    cer, p_a8 = _metric(block, "cer"), _metric(block, "cer_no_diac")
    return cer - p_a8 if cer is not None and p_a8 is not None else None

def _validate(obj, label: str) -> None:
    if not isinstance(obj, dict):
        raise GateError(f"{label}: phải là một object JSON")
    p_aa = obj.get("overall")
    if not isinstance(p_aa, dict) or _metric(p_aa, "cer") is None:
        raise GateError(f"{label}: thiếu 'overall.cer' (số)")
    strata = obj.get("by_stratum", {})
    if not isinstance(strata, dict):
        raise GateError(f"{label}: 'by_stratum' phải là object {tên_tầng: {...}}")
    for name, p_a9 in strata.items():
        if not isinstance(p_a9, dict) or _metric(p_a9, "cer") is None:
            raise GateError(f"{label}: by_stratum['{name}'].cer thiếu hoặc không phải số")

def q_c9(name: str, new_v, base_v, tol: float, ctx: str, failures: list) -> None:
    if new_v is None or base_v is None:
        return
    if new_v > base_v + tol:
        failures.append(
            f"{ctx}: {name} {base_v:.4f} → {new_v:.4f} (+{new_v - base_v:.4f} > tol {tol:.4f})"
        )

def _check_regression(report, baseline, cfg: GateConfig, failures: list, notes: list) -> None:
    p_ad, p_ab = report["overall"], baseline["overall"]
    q_c9("CER", _metric(p_ad, "cer"), _metric(p_ab, "cer"), cfg.cer_tol, "overall", failures)
    if cfg.wer_tol is not None:
        q_c9("WER", _metric(p_ad, "wer"), _metric(p_ab, "wer"), cfg.wer_tol, "overall", failures)
    if cfg.dau_tol is not None:
        q_c9("Δdấu", q_d0(p_ad), q_d0(p_ab), cfg.dau_tol, "overall", failures)
    if cfg.ece_tol is not None:
        q_c9("ECE", _metric(p_ad, "ece"), _metric(p_ab, "ece"), cfg.ece_tol, "overall", failures)

    p_ac = baseline.get("by_stratum", {})
    p_ae = report.get("by_stratum", {})
    for name in sorted(p_ac):
        ctx = f"tầng '{name}'"
        if name not in p_ae:
            if cfg.require_baseline_strata:
                failures.append(f"coverage: {ctx} có trong baseline nhưng THIẾU ở report")
            else:
                notes.append(f"{ctx} thiếu ở report (bỏ qua theo --allow-missing-strata)")
            continue
        q_c9("CER", _metric(p_ae[name], "cer"), _metric(p_ac[name], "cer"), cfg.cer_tol, ctx, failures)
        if cfg.wer_tol is not None:
            q_c9("WER", _metric(p_ae[name], "wer"), _metric(p_ac[name], "wer"), cfg.wer_tol, ctx, failures)
        if cfg.dau_tol is not None:
            q_c9("Δdấu", q_d0(p_ae[name]), q_d0(p_ac[name]), cfg.dau_tol, ctx, failures)
        if cfg.ece_tol is not None:
            q_c9("ECE", _metric(p_ae[name], "ece"), _metric(p_ac[name], "ece"), cfg.ece_tol, ctx, failures)

    for name in sorted(p_ae):
        if name not in p_ac:
            notes.append(f"tầng mới '{name}' (chưa có baseline) — chỉ áp trần SLO nếu bật")

def q_c4(report, slo: dict, failures: list) -> None:
    p_b2 = slo.get("overall", {})
    p_b0 = p_b2.get("cer") if isinstance(p_b2, dict) else None
    cer = _metric(report["overall"], "cer")
    if isinstance(p_b0, (int, float)) and not isinstance(p_b0, bool) and cer is not None and cer > p_b0:
        failures.append(f"SLO overall: CER {cer:.4f} > trần {float(p_b0):.4f}")

    p_b3 = report.get("by_stratum", {})
    for name in sorted(slo.get("by_stratum", {})):
        p_af = slo["by_stratum"][name]
        p_b0 = p_af.get("cer") if isinstance(p_af, dict) else None
        p_b1 = p_b3.get(name)
        p_b4 = _metric(p_b1, "cer") if isinstance(p_b1, dict) else None
        if isinstance(p_b0, (int, float)) and not isinstance(p_b0, bool) and p_b4 is not None and p_b4 > p_b0:
            failures.append(f"SLO tầng '{name}': CER {p_b4:.4f} > trần {float(p_b0):.4f}")

def evaluate(report: dict, baseline: Optional[dict], cfg: GateConfig) -> q_2:
    failures: list = []
    notes: list = []
    if baseline is None:
        notes.append("Không có baseline → bỏ kiểm hồi quy (lần phát hành đầu).")
    else:
        _check_regression(report, baseline, cfg, failures, notes)
    if cfg.slo:
        q_c4(report, cfg.slo, failures)
    return q_2(passed=not failures, failures=failures, notes=notes)

def _load_json(path: str, label: str) -> dict:
    try:
        with open(path, encoding="utf-8") as p_b5:
            return json.load(p_b5)
    except FileNotFoundError as e:
        raise GateError(f"{label}: không thấy file: {path}") from e
    except json.JSONDecodeError as e:
        raise GateError(f"{label}: JSON hỏng ({e})") from e

def run(args: argparse.Namespace) -> int:
    try:
        report = _load_json(args.report, "report")
        _validate(report, "report")
        baseline = None
        if args.baseline:
            baseline = _load_json(args.baseline, "baseline")
            _validate(baseline, "baseline")
        slo = _load_json(args.slo, "slo") if args.slo else None
        cfg = GateConfig(
            cer_tol=args.tol,
            wer_tol=args.wer_tol,
            dau_tol=args.dau_tol,
            ece_tol=args.ece_tol,
            slo=slo,
            require_baseline_strata=not args.allow_missing_strata,
        )
        p_b8 = evaluate(report, baseline, cfg)
    except GateError as e:
        print(f"LỖI: {e}", file=sys.stderr)
        return 2

    for p_b7 in p_b8.notes:
        print(f"• {p_b7}")
    if p_b8.passed:
        print("PASS")
        return 0
    print("FAIL")
    for p_b6 in p_b8.failures:
        print(f"  ✗ {p_b6}")
    return 1

def main(argv=None) -> int:
    p_b9 = argparse.ArgumentParser(
        description="Cổng hồi quy chất lượng OCR (CER/WER/Δdấu) — §S13.B.3"
    )
    p_b9.add_argument("--report", required=True, help="report.json bản mới (score.py sinh ra)")
    p_b9.add_argument("--baseline", default=None, help="baseline.json production (bỏ trống = lần đầu)")
    p_b9.add_argument("--slo", default=None, help="slo.json trần CER tuyệt đối (tuỳ chọn)")
    p_b9.add_argument("--tol", type=float, default=DEFAULT_CER_TOL, help="dung sai CER (mặc định 0.005)")
    p_b9.add_argument("--wer-tol", type=float, default=None, help="bật kiểm WER với dung sai này")
    p_b9.add_argument("--dau-tol", type=float, default=None, help="bật kiểm Δdấu với dung sai này")
    p_b9.add_argument("--ece-tol", type=float, default=None, help="bật kiểm ECE (calibration) với dung sai này")
    p_b9.add_argument(
        "--allow-missing-strata",
        action="store_true",
        help="KHÔNG fail khi tầng trong baseline thiếu ở report (mặc định = fail để chặn né tầng khó)",
    )
    return run(p_b9.parse_args(argv))

if __name__ == "__main__":
    sys.exit(main())
