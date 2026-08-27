#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_REL="triton/models/rec_vietocr/1"

echo "▶ Export VietOCR → ONNX trong container python:3.10 (CPU)…"
docker run --rm -i \
  -v "${REPO_ROOT}:/work" \
  python:3.10 bash -s <<'INCONTAINER'
set -euo pipefail
echo "· apt libs (opencv runtime)…"
apt-get -qq update >/dev/null 2>&1
apt-get -qq install -y libgl1 libglib2.0-0 >/dev/null 2>&1
echo "· pip torch CPU + vietocr…"
pip install --no-cache-dir -q torch==2.2.2 torchvision==0.17.2 \
    --index-url https://download.pytorch.org/whl/cpu >/tmp/pip1.log 2>&1 || { tail -20 /tmp/pip1.log; exit 1; }
pip install --no-cache-dir -q vietocr onnx onnxruntime >/tmp/pip2.log 2>&1 || { tail -30 /tmp/pip2.log; exit 1; }
# torch 2.2 cần numpy<2 (vietocr/onnx hay kéo numpy 2.x → "Numpy is not available").
pip install --no-cache-dir -q "numpy<2" >/tmp/pip3.log 2>&1 || { tail -20 /tmp/pip3.log; exit 1; }
echo "· chạy export_onnx.py…"
python3 /work/ml/finetune/vietocr/export_onnx.py --out /work/triton/models/rec_vietocr/1
INCONTAINER

echo "▶ Kết quả:"
ls -la "${REPO_ROOT}/${OUT_REL}/" | grep -E 'encoder|decoder|vocab' || true
