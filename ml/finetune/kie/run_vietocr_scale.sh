#!/usr/bin/env bash
set -uo pipefail
ROOT=/home/do/DigiText
IMG=pytorch/pytorch:2.4.1-cuda12.1-cudnn9-runtime

docker run --rm --gpus all -v "$ROOT":/work -w /work/ml/finetune/kie \
  -e HOME=/tmp -e PYTHONDONTWRITEBYTECODE=1 "$IMG" bash -lc '
  set -e
  echo "· deps (vietocr + libs + numpy<2)…"
  pip install -q vietocr 2>/tmp/pip.log || { echo "PIP_FAIL"; tail -20 /tmp/pip.log; exit 1; }
  apt-get update -qq >/tmp/apt.log 2>&1 && apt-get install -y -qq libxcb1 libgl1 libglib2.0-0 >>/tmp/apt.log 2>&1 || true
  pip install -q "numpy<2" 2>>/tmp/apt.log
  python3 -c "import cv2,torch;print(\"· cv2\",cv2.__version__,\"torch\",torch.__version__,torch.cuda.is_available())"

  echo "· MIX: VLM 2000tr (90/10) + crop người-train, giữ body-eval…"
  python3 - <<PYEOF
import hashlib, json, shutil
from pathlib import Path
kie=Path(".")
tr=Path("data/rec_train2"); img=tr/"img"; img.mkdir(parents=True, exist_ok=True)
def h(s): return int(hashlib.md5(s.encode()).hexdigest(),16)%100
vlm=[l.rstrip("\n") for l in open("data/rec_distill2/pairs.txt",encoding="utf-8") if "\t" in l]
train=[]; val=[]; body=[]
for ln in vlm:
    rel,truth=ln.split("\t",1); name=Path(rel).name; b=h(name)
    src=Path("data/rec_distill2")/rel
    if b<80:   dst=img/name; shutil.copy(src,dst); train.append(f"img/{name}\t{truth}")
    elif b<90: dst=img/name; shutil.copy(src,dst); val.append(f"img/{name}\t{truth}")
    else:      body.append({"image":str(src),"truth":truth})   # held-out, KHÔNG copy vào train
# crop người-train (easy/fix_span) + val
for f,acc in (("data/rec_dataset_big/train.txt",train),("data/rec_dataset_big/val.txt",val)):
    for ln in open(f,encoding="utf-8"):
        if "\t" not in ln: continue
        rel,truth=ln.rstrip("\n").split("\t",1); name=Path(rel).name
        shutil.copy(Path("data/rec_dataset_big")/rel, img/name); acc.append(f"img/{name}\t{truth}")
open(tr/"train.txt","w",encoding="utf-8").write("\n".join(train)+"\n")
open(tr/"val.txt","w",encoding="utf-8").write("\n".join(val)+"\n")
open("data/rec_body_eval.jsonl","w",encoding="utf-8").write("\n".join(json.dumps(x,ensure_ascii=False) for x in body)+"\n")
print(f"  train {len(train)} | val {len(val)} | body-eval held-out {len(body)}")
PYEOF

  echo "· lọc vocab + xoá lmdb cache…"
  python3 - <<PYEOF
from vietocr.tool.config import Cfg
vocab=set(Cfg.load_config_from_name("vgg_transformer")["vocab"])
for f in ("data/rec_train2/train.txt","data/rec_train2/val.txt"):
    keep=[l.rstrip("\n") for l in open(f,encoding="utf-8") if "\t" in l and l.split("\t",1)[1].strip() and all(c in vocab for c in l.split("\t",1)[1].rstrip("\n"))]
    open(f,"w",encoding="utf-8").write("\n".join(keep)+"\n"); print(f"  {f}: giữ {len(keep)}")
PYEOF
  rm -rf train_data valid_data

  echo "=== EVAL BASE — 58 hard định danh ==="
  python3 eval_vietocr_crops.py data/rec_dataset_big/eval_hard.jsonl > data/scale_before_hard.json; cat data/scale_before_hard.json
  echo "=== EVAL BASE — body held-out ==="
  python3 eval_vietocr_crops.py data/rec_body_eval.jsonl > data/scale_before_body.json; cat data/scale_before_body.json

  echo "=== FINE-TUNE (train_root=rec_train2, 3000 iters, batch 16) ==="
  python3 ../vietocr/train_vietocr.py --train data/rec_train2/train.txt --val data/rec_train2/val.txt \
    --train-root data/rec_train2 --out output/vietocr_vi2 --epochs 3 --batch 16 2>&1 | tail -20

  echo "=== EVAL FINE-TUNED — 58 hard định danh ==="
  python3 eval_vietocr_crops.py data/rec_dataset_big/eval_hard.jsonl --weights output/vietocr_vi2.pth > data/scale_after_hard.json; cat data/scale_after_hard.json
  echo "=== EVAL FINE-TUNED — body held-out ==="
  python3 eval_vietocr_crops.py data/rec_body_eval.jsonl --weights output/vietocr_vi2.pth > data/scale_after_body.json; cat data/scale_after_body.json
  echo "=== SCALE XONG ==="
'
