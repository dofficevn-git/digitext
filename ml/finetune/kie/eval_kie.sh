#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
KIE_DIR="ml/finetune/kie"
BIO_JSONL="${BIO_JSONL:-data/bio.jsonl}"
GOLD_JSONL="${GOLD_JSONL:-data/kie_gold.jsonl}"
MODEL_DIR="${MODEL_DIR:-output/kie}"
OUT="${OUT:-data/kie_gold_report.json}"
TORCH_IMAGE="${TORCH_IMAGE:-pytorch/pytorch:2.4.1-cuda12.1-cudnn9-runtime}"
CONTAINER="${CONTAINER:-kie_eval}"

for f in "${GOLD_JSONL}" "${MODEL_DIR}/config.json"; do
  if [[ ! -e "${REPO_ROOT}/${KIE_DIR}/${f}" ]]; then
    echo "LỖI: không thấy ${KIE_DIR}/${f}" >&2
    exit 2
  fi
done

FREE_MIB="$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits | head -1)"
echo "· VRAM trống ${FREE_MIB}MiB"

docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
docker run --rm --name "$CONTAINER" --gpus all --shm-size=8g \
  -v "${REPO_ROOT}:/work" \
  -v kie_pipcache:/root/.cache/pip \
  -v kie_hfcache:/root/.cache/huggingface \
  -w "/work/${KIE_DIR}" \
  -e BIO_JSONL="$BIO_JSONL" -e GOLD_JSONL="$GOLD_JSONL" \
  -e MODEL_DIR="$MODEL_DIR" -e OUT="$OUT" \
  "$TORCH_IMAGE" bash -lc '
  set -euo pipefail
  echo "· cài transformers…"
  pip install -q "transformers==4.44.2" "accelerate>=0.26,<1.0" "tokenizers<0.20" "numpy<2" "safetensors" 2>&1 | tail -2
  python -c "import torch; print(f\"· torch {torch.__version__} cuda={torch.cuda.is_available()}\")"
  echo "· chấm trên GOLD…"
  python eval_kie.py "$BIO_JSONL" --model-dir "$MODEL_DIR" --gold "$GOLD_JSONL" --out "$OUT"
'
echo "· xong → ${KIE_DIR}/${OUT}"
