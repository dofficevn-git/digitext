#!/usr/bin/env bash
set -euo pipefail

ONNX_DIR="${ONNX_DIR:-triton/build/onnx}"
MODELS_DIR="${MODELS_DIR:-triton/models}"
CALIB_DIR="${CALIB_DIR:-}"

command -v trtexec >/dev/null 2>&1 || { echo "❌ thiếu trtexec (TensorRT)"; exit 1; }

{
  echo "date_utc=$(date -u +%FT%TZ)"
  echo "trtexec=$(trtexec --version 2>&1 | head -1 || true)"
  command -v nvidia-smi >/dev/null 2>&1 && nvidia-smi --query-gpu=name,driver_version --format=csv,noheader
} > "$MODELS_DIR/BUILD_PROVENANCE.txt"

build_fp16() {
  local onnx="$1" model="$2" ver="${3:-1}"
  local dst="$MODELS_DIR/$model/$ver"
  mkdir -p "$dst"
  echo "▶ FP16 $onnx → $dst/model.plan"
  trtexec --onnx="$ONNX_DIR/$onnx" --saveEngine="$dst/model.plan" --fp16 --skipInference
}

build_int8_rec() {
  local onnx="$1" model="$2" ver="${3:-1}"
  local dst="$MODELS_DIR/$model/$ver"
  [ -n "$CALIB_DIR" ] || { echo "❌ INT8 cần CALIB_DIR (calibration set tiếng Việt)"; exit 1; }
  mkdir -p "$dst"
  echo "▶ INT8 $onnx → $dst/model.plan (calib $CALIB_DIR)"
  trtexec --onnx="$ONNX_DIR/$onnx" --saveEngine="$dst/model.plan" \
    --int8 --fp16 --calib="$CALIB_DIR/rec_svtr_vi.calib" --skipInference
  echo "  ⚠ Sau build: đo CER trên golden-set; nếu tụt > 0.5% thì rơi về FP16 (§S12.4)."
}

build_fp16 det_dbnet.onnx  det_dbnet    1
build_fp16 cls_angle.onnx  cls_angle    1
build_int8_rec rec_svtr_vi.onnx rec_svtr_vi 1

echo "✅ Engine đã đặt vào $MODELS_DIR/*/1/model.plan (KHÔNG commit — .gitignore)."
