#!/usr/bin/env bash
set -uo pipefail
ROOT=/home/do/DigiText
IMG=pytorch/pytorch:2.4.1-cuda12.1-cudnn9-runtime

docker run --rm --gpus all -v "$ROOT":/work -w /work/ml/finetune/kie \
  -e HOME=/tmp -e PYTHONDONTWRITEBYTECODE=1 "$IMG" bash -lc '
  set -e
  echo "· cài vietocr…"
  pip install -q vietocr 2>/tmp/pip.log || { echo "PIP_FAIL:"; tail -20 /tmp/pip.log; exit 1; }
  echo "· cài lib hệ thống cho cv2 (libxcb/libGL/glib)…"
  apt-get update -qq >/tmp/apt.log 2>&1 && apt-get install -y -qq libxcb1 libgl1 libglib2.0-0 >>/tmp/apt.log 2>&1 \
    || { echo "· apt lỗi → thử opencv-headless"; pip install -q --force-reinstall opencv-python-headless 2>>/tmp/apt.log; }
  echo "· pin numpy<2 cho imgaug…"
  pip install -q "numpy<2" 2>>/tmp/apt.log
  python3 -c "import cv2,numpy;print(\"· cv2\",cv2.__version__,\"| numpy\",numpy.__version__,\"OK\")" \
    || { echo "CV2_FAIL:"; tail -10 /tmp/apt.log; exit 1; }
  python3 -c "from imgaug import augmenters" || { echo "IMGAUG_FAIL"; exit 1; }
  python3 -c "import torch;print(\"· torch\",torch.__version__,\"cuda\",torch.cuda.is_available())"
  mkdir -p output
  echo "=== EVAL BASE (TRƯỚC) trên 13 crop hard held-out ==="
  python3 eval_vietocr_crops.py data/rec_dataset/eval_hard.jsonl > data/vietocr_before.json 2>/tmp/ev0.log \
    || { echo "EVAL_BASE_FAIL:"; tail -20 /tmp/ev0.log; exit 1; }
  cat data/vietocr_before.json
  echo "· lọc cặp train theo VOCAB VietOCR (bỏ dòng có ký tự ngoài charset, vd ² → KeyError)…"
  python3 - <<PYEOF
from vietocr.tool.config import Cfg
vocab=set(Cfg.load_config_from_name("vgg_transformer")["vocab"])
for f in ("data/rec_train/train.txt","data/rec_train/val.txt"):
    keep=[]; drop=0
    for ln in open(f,encoding="utf-8"):
        if "\t" not in ln: continue
        txt=ln.split("\t",1)[1].rstrip("\n")
        if txt and all(c in vocab for c in txt): keep.append(ln.rstrip("\n"))
        else: drop+=1
    open(f,"w",encoding="utf-8").write("\n".join(keep)+"\n")
    print(f"  {f}: giữ {len(keep)} / bỏ {drop} (ngoài vocab)")
PYEOF
  echo "· xoá lmdb cache cũ (train_data/valid_data)…"
  rm -rf train_data valid_data
  echo "=== FINE-TUNE (2000 iters, batch 16) ==="
  python3 ../vietocr/train_vietocr.py --train data/rec_train/train.txt --val data/rec_train/val.txt \
    --train-root data/rec_train --out output/vietocr_vi --epochs 2 --batch 16 2>&1 | tail -25
  echo "=== EVAL FINE-TUNED (SAU) ==="
  python3 eval_vietocr_crops.py data/rec_dataset/eval_hard.jsonl --weights output/vietocr_vi.pth \
    > data/vietocr_after.json 2>/tmp/ev1.log || { echo "EVAL_FT_FAIL:"; tail -20 /tmp/ev1.log; exit 1; }
  cat data/vietocr_after.json
  echo "=== PILOT XONG ==="
'
