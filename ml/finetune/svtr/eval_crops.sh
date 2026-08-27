#!/usr/bin/env bash
set -euo pipefail
ROOT=/home/do/DigiText
IMG=paddlepaddle/paddle:2.6.2-gpu-cuda12.0-cudnn8.9-trt8.6
PADDLEOCR_REF=release/2.8

docker run --rm --gpus all -v "$ROOT":/work -w /work/ml/finetune/svtr \
  -e HOME=/tmp -e PYTHONDONTWRITEBYTECODE=1 "$IMG" bash -lc '
    set -e
    if [ ! -d /paddleocr ]; then
      echo "· clone PaddleOCR '"$PADDLEOCR_REF"'…"
      git clone --depth 1 -b '"$PADDLEOCR_REF"' https://github.com/PaddlePaddle/PaddleOCR /paddleocr >/tmp/git.log 2>&1
    fi
    echo "· deps (numpy<2 để không vỡ paddle)…"
    pip install -q "numpy<2" opencv-python-headless shapely pyclipper scikit-image lmdb 2>/tmp/pip.log || {
      echo "pip lỗi:"; tail -5 /tmp/pip.log; exit 1; }
    echo "· chạy eval SVTR trên crop…"
    python3 /work/ml/finetune/svtr/eval_crops.py \
      /work/ml/finetune/kie/data/ocr_crops/crops.jsonl \
      --img-root /work/ml/finetune/kie \
      --model-dir /work/ml/finetune/svtr/inference/rec_svtr_vi \
      --dict /work/triton/models/ocr_pipeline/1/dict_VI.txt \
      --out /work/ml/finetune/kie/data/svtr_crops_cer.json
  '
