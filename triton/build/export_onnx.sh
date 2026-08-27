#!/usr/bin/env bash
set -euo pipefail

PADDLE_MODELS_DIR="${PADDLE_MODELS_DIR:?đặt PADDLE_MODELS_DIR trỏ tới thư mục model Paddle inference}"
OUT_DIR="${OUT_DIR:-triton/build/onnx}"
OPSET="${OPSET:-16}"

command -v paddle2onnx >/dev/null 2>&1 || { echo "❌ thiếu paddle2onnx (pip install paddle2onnx)"; exit 1; }
mkdir -p "$OUT_DIR"

{
  echo "date_utc=$(date -u +%FT%TZ)"
  echo "paddle2onnx=$(paddle2onnx --version 2>&1 | head -1)"
  echo "opset=$OPSET"
} > "$OUT_DIR/PROVENANCE.txt"

export_one() {
  local sub="$1" out="$2"
  echo "▶ export $sub → $out.onnx (opset $OPSET)"
  paddle2onnx \
    --model_dir "$PADDLE_MODELS_DIR/$sub" \
    --model_filename inference.pdmodel \
    --params_filename inference.pdiparams \
    --opset_version "$OPSET" \
    --save_file "$OUT_DIR/$out.onnx"
}

export_one det   det_dbnet
export_one cls   cls_angle
export_one rec   rec_svtr_vi

echo "✅ ONNX ở $OUT_DIR (xác minh shape bằng: polygraphy inspect model $OUT_DIR/det_dbnet.onnx)"
