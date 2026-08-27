#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
EPOCHS="${EPOCHS:-30}"
PADDLEOCR_REF="${PADDLEOCR_REF:-release/2.8}"
REC_PRETRAIN_URL="${REC_PRETRAIN_URL:-https://paddleocr.bj.bcebos.com/PP-OCRv4/chinese/ch_PP-OCRv4_rec_train.tar}"

PADDLE_IMAGE="${PADDLE_IMAGE:-paddlepaddle/paddle:2.6.2-gpu-cuda12.0-cudnn8.9-trt8.6}"
docker rm -f svtr_train >/dev/null 2>&1 || true
docker run -d --name svtr_train --gpus all --shm-size=8g \
  -v "${REPO_ROOT}:/work" -v ocr_pipcache:/root/.cache/pip -w /work "$PADDLE_IMAGE" bash -lc '
  set -e
  echo "· apt (git, libgl)…"; apt-get -qq update >/dev/null 2>&1
  apt-get -qq install -y git libgl1 libglib2.0-0 wget >/dev/null 2>&1 || true
  python3 -c "import paddle; print(\"· paddle\", paddle.__version__, \"gpu?\", paddle.is_compiled_with_cuda())"
  echo "· clone PaddleOCR '"$PADDLEOCR_REF"'…"
  [ -d /paddleocr ] || git clone --depth 1 -b '"$PADDLEOCR_REF"' https://github.com/PaddlePaddle/PaddleOCR /paddleocr >/tmp/git.log 2>&1
  echo "· deps PaddleOCR (giữ numpy<2 để không vỡ paddle)…"
  pip install -q -r /paddleocr/requirements.txt >/tmp/req.log 2>&1 || { echo REQ_FAIL; tail -25 /tmp/req.log; exit 1; }
  pip install -q "numpy<2" >/tmp/np.log 2>&1
  python3 -c "import paddle,numpy; print(\"· paddle\", paddle.__version__, \"| numpy\", numpy.__version__, \"| gpu\", paddle.device.cuda.device_count())"

  echo "· tải pretrained PP-OCRv4 rec…"
  cd /work/ml/finetune/svtr
  mkdir -p pretrain/rec_svtr_lcnet
  if [ ! -f pretrain/rec_svtr_lcnet/best_accuracy.pdparams ]; then
    [ -f pretrain/rec.tar ] || wget -q -O pretrain/rec.tar "'"$REC_PRETRAIN_URL"'"
    tar -xf pretrain/rec.tar -C pretrain/
    src=$(find pretrain -name student.pdparams ! -name "._*" | head -1)
    [ -n "$src" ] || src=$(find pretrain -name best_accuracy.pdparams ! -name "._*" | head -1)
    [ -n "$src" ] || { echo "PRETRAIN_FAIL: không thấy .pdparams"; find pretrain; exit 1; }
    cp "$src" pretrain/rec_svtr_lcnet/best_accuracy.pdparams
    rm -f pretrain/rec.tar
  fi
  ls -la pretrain/rec_svtr_lcnet/

  CFG=$(ls /paddleocr/configs/rec/PP-OCRv4/ch_PP-OCRv4_rec.yml 2>/dev/null \
        || find /paddleocr/configs/rec -iname "*PP-OCRv4*rec*.yml" ! -iname "*distill*" ! -iname "*hgnet*" | head -1)
  echo "· CONFIG: $CFG"
  O=/work/ml/finetune/svtr/output/rec_svtr_vi
  if [ -f "$O/latest.pdparams" ]; then
    echo "· RESUME từ $O/latest (train tiếp)"; RESUME="Global.checkpoints=$O/latest"
  else
    echo "· Fresh từ pretrained PP-OCRv4 rec"; RESUME="Global.pretrained_model=/work/ml/finetune/svtr/pretrain/rec_svtr_lcnet/best_accuracy"
  fi
  echo "· TRAIN (epochs='"$EPOCHS"')…"
  D=/work/ml/finetune/svtr/train_data/rec
  export FLAGS_allocator_strategy=auto_growth
  export FLAGS_cudnn_exhaustive_search=0
  export FLAGS_conv_workspace_size_limit=512
  python3 /paddleocr/tools/train.py -c "$CFG" -o \
      Train.sampler.first_bs=32 \
      Global.epoch_num='"$EPOCHS"' Global.print_batch_step=100 Global.save_epoch_step=1 \
      Global.eval_batch_step=[0,2000] Global.use_space_char=false \
      Global.character_dict_path=/work/triton/models/ocr_pipeline/1/dict_VI.txt \
      "$RESUME" \
      Global.save_model_dir=/work/ml/finetune/svtr/output/rec_svtr_vi \
      Global.save_inference_dir=/work/ml/finetune/svtr/inference/rec_svtr_vi \
      Train.dataset.data_dir="$D/" Train.dataset.label_file_list=["$D/train_list.txt"] \
      Train.loader.batch_size_per_card=32 Train.loader.num_workers=8 \
      Eval.dataset.data_dir="$D/" Eval.dataset.label_file_list=["$D/val_list.txt"] \
      Eval.loader.batch_size_per_card=32 Eval.loader.num_workers=4
  echo "✅ TRAIN XONG — checkpoint ở ml/finetune/svtr/output/rec_svtr_vi/"
' >/dev/null
echo "container svtr_train: $(docker ps --filter name=svtr_train --format '{{.Status}}')"
echo "Theo dõi:  docker logs -f svtr_train"
