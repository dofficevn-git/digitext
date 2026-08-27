#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
PADDLE_IMAGE="${PADDLE_IMAGE:-paddlepaddle/paddle:2.6.2-gpu-cuda12.0-cudnn8.9-trt8.6}"
TRITON_IMAGE="${TRITON_IMAGE:-nvcr.io/nvidia/tritonserver:24.08-py3}"
CKPT="${CKPT:-/work/ml/finetune/svtr/output/rec_svtr_vi/best_accuracy}"

echo "▶ [1/2] export_model (checkpoint → inference) trong container paddle…"
docker rm -f svtr_export >/dev/null 2>&1 || true
docker run --rm --gpus all -v "${REPO_ROOT}:/work" -v ocr_pipcache:/root/.cache/pip \
  "$PADDLE_IMAGE" bash -lc '
  set -e
  [ -d /paddleocr ] || git clone --depth 1 -b release/2.8 https://github.com/PaddlePaddle/PaddleOCR /paddleocr >/dev/null 2>&1
  pip install -q -r /paddleocr/requirements.txt >/dev/null 2>&1
  pip install -q "numpy<2" "paddle2onnx==1.2.6" >/dev/null 2>&1
  CFG=$(ls /paddleocr/configs/rec/PP-OCRv4/ch_PP-OCRv4_rec.yml)
  cd /work/ml/finetune/svtr
  python3 /paddleocr/tools/export_model.py -c "$CFG" -o \
      Global.checkpoints='"$CKPT"' \
      Global.character_dict_path=/work/triton/models/ocr_pipeline/1/dict_VI.txt \
      Global.use_space_char=false \
      Global.save_inference_dir=/work/ml/finetune/svtr/inference/rec_svtr_vi
  echo "  ✓ inference model saved"
  mkdir -p /work/triton/build/onnx
  echo "· paddle2onnx (input động theo W)"
  paddle2onnx --model_dir /work/ml/finetune/svtr/inference/rec_svtr_vi \
      --model_filename inference.pdmodel --params_filename inference.pdiparams \
      --opset_version 13 --save_file /work/triton/build/onnx/rec_svtr_vi.onnx
  echo "  ✓ ONNX: triton/build/onnx/rec_svtr_vi.onnx"
'

echo "▶ [2/2] ONNX → rename I/O → TensorRT FP16 (container Triton, trtexec khớp runtime)…"
docker run --rm -i --gpus all -v "${REPO_ROOT}:/work" "$TRITON_IMAGE" bash -s <<'INCONTAINER'
set -euo pipefail
export PATH="/usr/src/tensorrt/bin:$PATH"
pip install --no-cache-dir -q "onnx==1.14.1" >/tmp/pip.log 2>&1 || { tail -20 /tmp/pip.log; exit 1; }
echo "· chuẩn hoá tên I/O khớp config.pbtxt + BLS"
python3 - <<'PY'
import onnx
p = "/work/triton/build/onnx/rec_svtr_vi.onnx"
m = onnx.load(p); g = m.graph
# LƯU Ý: "softmax_0.tmp_0" trong graph này là softmax ATTENTION nội bộ (SVTR neck, 8 heads) —
# KHÔNG phải CTC output. CTC output THẬT = graph output gốc (save_infer_model/scale_0.tmp_0),
# shape [B,T,219]. Đổi tên nó → "ctc_probs" (tên MỚI, không trùng) nhất quán ở mọi tham chiếu.
old_out, new_out = g.output[0].name, "ctc_probs"
for n in g.node:
    n.input[:]  = [new_out if i == old_out else i for i in n.input]
    n.output[:] = [new_out if o == old_out else o for o in n.output]
g.output[0].name = new_out
oin = g.input[0].name
if oin != "x":               # bảo đảm input tên 'x'
    for n in g.node:
        n.input[:] = ["x" if i == oin else i for i in n.input]
    g.input[0].name = "x"
onnx.checker.check_model(m)
onnx.save(m, p)
print(f"  I/O: in 'x' out 'ctc_probs' (CTC thật, was out={old_out}) — checker OK")
PY
dst=/work/triton/models/rec_svtr_vi/1; mkdir -p "$dst"
echo "· trtexec FP16 (W profile 160/320/640)"
trtexec --onnx=/work/triton/build/onnx/rec_svtr_vi.onnx --saveEngine="$dst/model.plan" --fp16 \
  --minShapes=x:1x3x48x160 --optShapes=x:1x3x48x320 --maxShapes=x:1x3x48x640 --skipInference
echo "  ✓ engine: rec_svtr_vi/1/model.plan"
echo "✅ rec_svtr_vi engine sẵn sàng"
INCONTAINER
echo "Bước cuối: đặt svtr_enabled=\"true\" trong ocr_pipeline/config.pbtxt rồi khởi lại Triton."
