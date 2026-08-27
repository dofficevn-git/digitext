#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TRITON_IMAGE="${TRITON_IMAGE:-nvcr.io/nvidia/tritonserver:24.08-py3}"
BUILD_DIR="${REPO_ROOT}/triton/build"
PADDLE_DIR="${BUILD_DIR}/paddle"
ONNX_DIR="${BUILD_DIR}/onnx"

DET_URL="${DET_URL:-https://paddleocr.bj.bcebos.com/PP-OCRv4/chinese/ch_PP-OCRv4_det_infer.tar}"
CLS_URL="${CLS_URL:-https://paddleocr.bj.bcebos.com/dygraph_v2.0/ch/ch_ppocr_mobile_v2.0_cls_infer.tar}"

echo "▶ [1/3] Tải model PaddleOCR pretrained (host)…"
mkdir -p "$PADDLE_DIR" "$ONNX_DIR"
fetch() {
  local url="$1" name; name="$(basename "$url")"
  if [[ ! -d "$PADDLE_DIR/${name%.tar}" ]]; then
    echo "  ↓ $name"
    curl -fSL --retry 3 -o "$PADDLE_DIR/$name" "$url"
    tar -xf "$PADDLE_DIR/$name" -C "$PADDLE_DIR"
    rm -f "$PADDLE_DIR/$name"
  else
    echo "  ✓ đã có ${name%.tar}"
  fi
}
fetch "$DET_URL"
fetch "$CLS_URL"
DET_SUB="$(basename "${DET_URL%.tar}")"
CLS_SUB="$(basename "${CLS_URL%.tar}")"

echo "▶ [2/3] Convert ONNX + build TensorRT trong container Triton…"
docker run --rm -i --gpus all \
  -v "${REPO_ROOT}/triton:/work/triton" \
  -e DET_SUB="$DET_SUB" -e CLS_SUB="$CLS_SUB" \
  "$TRITON_IMAGE" bash -s <<'INCONTAINER'
set -euo pipefail
export PATH="/usr/src/tensorrt/bin:$PATH"   # trtexec sống ở đây trong image Triton
cd /work/triton/build
echo "  · pip paddle2onnx onnx…"
pip install --no-cache-dir -q "paddle2onnx==1.0.6" "onnx==1.14.1" packaging >/tmp/pip.log 2>&1 || { echo "pip FAIL"; tail -20 /tmp/pip.log; exit 1; }

PADDLE=/work/triton/build/paddle
ONNX=/work/triton/build/onnx
mkdir -p "$ONNX"

# --- paddle → onnx (input var name của PaddleOCR vốn là 'x') ---
conv() {  # subdir  out.onnx
  local sub="$1" out="$2"
  echo "  · paddle2onnx $sub"
  paddle2onnx --model_dir "$PADDLE/$sub" \
    --model_filename inference.pdmodel --params_filename inference.pdiparams \
    --opset_version 16 --save_file "$ONNX/$out"
}
conv "$DET_SUB" det_dbnet.onnx
conv "$CLS_SUB" cls_angle.onnx

# --- Chuẩn hóa tên I/O cho khớp config.pbtxt (Triton map binding theo TÊN) ---
python3 - <<'PY'
import onnx
def fix(path, in_name, out_name):
    m = onnx.load(path); g = m.graph
    old_in = g.input[0].name
    if old_in != in_name:
        for n in g.node:
            n.input[:] = [in_name if x==old_in else x for x in n.input]
        g.input[0].name = in_name
    old_out = g.output[0].name
    if old_out != out_name:
        for n in g.node:
            n.output[:] = [out_name if x==old_out else x for x in n.output]
        g.output[0].name = out_name
    onnx.save(m, path)
    print(f"    {path}: in='{in_name}' out='{out_name}' (was in='{old_in}' out='{old_out}')")
fix("/work/triton/build/onnx/det_dbnet.onnx", "x", "sigmoid_0.tmp_0")
fix("/work/triton/build/onnx/cls_angle.onnx", "x", "softmax")
PY

# --- ONNX → TensorRT FP16 với optimization profile (paddle input động → cố định shape) ---
trt() {  # onnx  model_dir  min  opt  max
  local onnx="$1" model="$2" mn="$3" op="$4" mx="$5"
  local dst="/work/triton/models/$model/1"
  mkdir -p "$dst"
  echo "  · trtexec FP16 $onnx → $model/1/model.plan"
  trtexec --onnx="$ONNX/$onnx" --saveEngine="$dst/model.plan" --fp16 \
    --minShapes=x:$mn --optShapes=x:$op --maxShapes=x:$mx --skipInference
}
trt det_dbnet.onnx det_dbnet 1x3x1024x1024 8x3x1024x1024 16x3x1024x1024
trt cls_angle.onnx cls_angle 1x3x48x192      32x3x48x192   128x3x48x192

# provenance (engine khóa theo TRT/driver — ghi để tái lập).
{
  echo "date_utc=$(date -u +%FT%TZ)"
  trtexec --version 2>&1 | head -1
  echo "det_src=$DET_SUB  cls_src=$CLS_SUB (PaddleOCR pretrained, KHÔNG fine-tune)"
} > /work/triton/models/BUILD_PROVENANCE.txt
echo "  ✓ Xong trong container."
INCONTAINER

echo "▶ [3/3] Kết quả:"
ls -la "${REPO_ROOT}/triton/models/det_dbnet/1/" "${REPO_ROOT}/triton/models/cls_angle/1/" 2>/dev/null | grep -E 'model.plan|total' || true
echo "✅ det_dbnet + cls_angle engine đã đặt vào triton/models/*/1/model.plan"
echo "   (rec_svtr_vi vẫn cần fine-tune §P-M5; rec_vietocr build ở script riêng.)"
