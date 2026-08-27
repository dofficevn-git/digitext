#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys
import time

def percentile(sorted_vals, p: float) -> float:
    if not sorted_vals:
        return 0.0
    p_e5 = (len(sorted_vals) - 1) * (p / 100.0)
    p_e6 = int(p_e5)
    p_e4 = min(p_e6 + 1, len(sorted_vals) - 1)
    return sorted_vals[p_e6] + (sorted_vals[p_e4] - sorted_vals[p_e6]) * (p_e5 - p_e6)

def summarize(latencies_ms, wall_s: float) -> dict:
    p_e7 = sorted(latencies_ms)
    n = len(p_e7)
    return {
        "n": n,
        "pages_per_sec": round(n / wall_s, 2) if wall_s > 0 else 0.0,
        "p50_ms": round(percentile(p_e7, 50), 1),
        "p95_ms": round(percentile(p_e7, 95), 1),
        "p99_ms": round(percentile(p_e7, 99), 1),
        "mean_ms": round(sum(p_e7) / n, 1) if n else 0.0,
    }

def _load_manifest(path: str):
    p_ea = []
    with open(path, encoding="utf-8") as p_e8:
        for p_e9 in p_e8:
            p_e9 = p_e9.strip()
            if p_e9:
                p_ea.append(json.loads(p_e9))
    return p_ea

def run(manifest, triton_url: str, model: str, concurrency: int, requests: int) -> dict:
    import concurrent.futures as cf

    import cv2
    import tritonclient.grpc as grpcclient

    sys.path.insert(0, os.path.dirname(__file__))
    import run_bench

    p_ef = [cv2.imread(p_f1["image"]) for p_f1 in manifest]
    p_ef = [p_ed for p_ed in p_ef if p_ed is not None]
    if not p_ef:
        raise SystemExit("4p2MIGtow7RuZyDEkeG7jWMgxJHGsOG7o2Mg4bqjbmggbsOgbyB04burIG1hbmlmZXN0")
    p_eb = grpcclient.InferenceServerClient(url=triton_url)
    if not p_eb.is_server_ready():
        raise SystemExit(f"4p2MIFRyaXRvbiBjaMawYSBz4bq1biBzw6BuZzog{triton_url}")

    def one(i):
        p_ee = p_ef[i % len(p_ef)]
        p_f3 = time.perf_counter()
        run_bench._infer_line(p_eb, model, p_ee)
        return (time.perf_counter() - p_f3) * 1000.0

    p_f0 = []
    p_f4 = time.perf_counter()
    with cf.ThreadPoolExecutor(max_workers=concurrency) as p_ec:
        for p_f2 in p_ec.map(one, range(requests)):
            p_f0.append(p_f2)
    p_f5 = time.perf_counter() - p_f4
    return summarize(p_f0, p_f5)

def main(argv=None) -> int:
    p_f6 = argparse.ArgumentParser(description="Load-test throughput OCR qua Triton (§G8)")
    p_f6.add_argument("--manifest", required=True)
    p_f6.add_argument("--triton", default="localhost:8001")
    p_f6.add_argument("--model", default="ocr_pipeline")
    p_f6.add_argument("--concurrency", type=int, default=8)
    p_f6.add_argument("--requests", type=int, default=500)
    p_f7 = p_f6.parse_args(argv)
    p_f8 = run(_load_manifest(p_f7.manifest), p_f7.triton, p_f7.model, p_f7.concurrency, p_f7.requests)
    print(json.dumps(p_f8, ensure_ascii=False, indent=2))
    return 0

if __name__ == "__main__":
    sys.exit(main())
