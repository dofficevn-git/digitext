#!/usr/bin/env bash
set -uo pipefail
cd /home/do/DigiText/ml/finetune/kie
while kill -0 365416 2>/dev/null; do sleep 60; done
echo "=== BƯỚC 1 XONG ==="; tail -2 data/extract2.log
echo "tổng: $(wc -l < data/kie_silver.jsonl) dòng"

echo; echo "=== BƯỚC 2: LỌC ==="
python3 filter_rules.py data/kie_silver.jsonl ../../data --out data/kie_filtered.jsonl 2>&1 | tail -25

echo; echo "=== BƯỚC 3: DỰNG BIO (Triton) ==="
python3 build_bio.py data/kie_filtered.jsonl ../../data --out data/bio.jsonl 2>&1 | tail -25
echo "=== BƯỚC 3 XONG: $(wc -l < data/bio.jsonl 2>/dev/null || echo 0) trang có BIO ==="
