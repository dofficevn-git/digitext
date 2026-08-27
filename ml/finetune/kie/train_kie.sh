#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
KIE_DIR="ml/finetune/kie"
EPOCHS="${EPOCHS:-10}"
BATCH_SIZE="${BATCH_SIZE:-4}"
BIO_JSONL="${BIO_JSONL:-data/bio.jsonl}"
OUT_DIR="${OUT_DIR:-output/kie}"
TORCH_IMAGE="${TORCH_IMAGE:-pytorch/pytorch:2.4.1-cuda12.1-cudnn9-runtime}"
CONTAINER="${CONTAINER:-kie_train}"

if [[ ! -f "${REPO_ROOT}/${KIE_DIR}/${BIO_JSONL}" ]]; then
  echo "LỖI: không thấy ${KIE_DIR}/${BIO_JSONL} — chạy build_bio.py trước." >&2
  exit 2
fi

FREE_MIB="$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits | head -1)"
NEED_MIB="${NEED_MIB:-5000}"
if (( FREE_MIB < NEED_MIB )); then
  echo "LỖI: VRAM trống ${FREE_MIB}MiB < ${NEED_MIB}MiB cần thiết." >&2
  echo "      Tiến trình đang giữ GPU:" >&2
  nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv >&2
  echo "      Tắt vLLM rồi chạy lại:" >&2
  echo "        docker compose -f infra/compose/vllm/docker-compose.yml down" >&2
  exit 3
fi
echo "· VRAM trống ${FREE_MIB}MiB — đủ."

docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
docker run --rm --name "$CONTAINER" --gpus all --shm-size=8g \
  -v "${REPO_ROOT}:/work" \
  -v kie_pipcache:/root/.cache/pip \
  -v kie_hfcache:/root/.cache/huggingface \
  -w "/work/${KIE_DIR}" \
  -e EPOCHS="$EPOCHS" -e BATCH_SIZE="$BATCH_SIZE" \
  -e BIO_JSONL="$BIO_JSONL" -e OUT_DIR="$OUT_DIR" \
  "$TORCH_IMAGE" bash -lc '
  set -euo pipefail
  echo "· cài transformers…"
  pip install -q "transformers==4.44.2" "accelerate>=0.26,<1.0" "tokenizers<0.20" "numpy<2" "safetensors" 2>&1 | tail -2
  python -c "import torch; print(f\"· torch {torch.__version__} cuda={torch.cuda.is_available()}\")"
  echo "· thống kê dữ liệu trước khi train:"
  python train_kie.py "$BIO_JSONL" --dry-run
  echo "· train…"
  python train_kie.py "$BIO_JSONL" --out "$OUT_DIR" --epochs "$EPOCHS" --batch-size "$BATCH_SIZE"
'
echo "· xong → ${KIE_DIR}/${OUT_DIR}/train_report.json"
