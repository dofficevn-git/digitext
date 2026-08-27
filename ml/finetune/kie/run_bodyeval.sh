#!/usr/bin/env bash
set -uo pipefail
ROOT=/home/do/DigiText
IMG=pytorch/pytorch:2.4.1-cuda12.1-cudnn9-runtime

docker run --rm --gpus all -v "$ROOT":/work -w /work/ml/finetune/kie \
  -e HOME=/tmp -e PYTHONDONTWRITEBYTECODE=1 "$IMG" bash -lc '
  set -e
  echo "· cài vietocr + lib cv2…"
  pip install -q vietocr 2>/tmp/pip.log || { echo "PIP_FAIL:"; tail -20 /tmp/pip.log; exit 1; }
  apt-get update -qq >/tmp/apt.log 2>&1 && apt-get install -y -qq libxcb1 libgl1 libglib2.0-0 >>/tmp/apt.log 2>&1 \
    || { echo "· apt lỗi → opencv-headless"; pip install -q --force-reinstall opencv-python-headless 2>>/tmp/apt.log; }
  pip install -q "numpy<2" 2>>/tmp/apt.log
  python3 -c "import cv2" 2>/dev/null || { echo "CV2_FAIL:"; tail -10 /tmp/apt.log; exit 1; }

  echo "=== BODY-EVAL BASE (TRƯỚC) ==="
  python3 eval_vietocr_crops.py data/rec_bodyeval/eval_body.jsonl > data/bodyeval_before.json 2>/tmp/b0.log \
    || { echo "BODYEVAL_BASE_FAIL:"; tail -20 /tmp/b0.log; exit 1; }
  cat data/bodyeval_before.json

  echo "=== BODY-EVAL FINE-TUNED (SAU) ==="
  python3 eval_vietocr_crops.py data/rec_bodyeval/eval_body.jsonl --weights output/vietocr_vi.pth \
    > data/bodyeval_after.json 2>/tmp/b1.log \
    || { echo "BODYEVAL_FT_FAIL:"; tail -20 /tmp/b1.log; exit 1; }
  cat data/bodyeval_after.json
  echo "=== BODYEVAL XONG ==="
'
