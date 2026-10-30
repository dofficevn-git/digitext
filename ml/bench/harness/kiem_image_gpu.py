"""Kiểm image OCR vừa build (chạy TRONG container, không cần GPU trống): phiên bản torch/paddle, có kernel sm_120,
DBNet CPU trên ảnh trắng (tải sẵn trọng số vào volume), VietOCR import, và một phép dò CUDA nhỏ (bỏ qua nếu VRAM
đang bị chiếm). Chuẩn bị chuyển máy sang OCR PC, 30/10/2026.

  docker run --rm --gpus all -v digitext-ocr-fulltext_paddlecache:/models -v <repo>/ml/bench/harness/kiem_image_gpu.py:/tmp/k.py:ro \
      digitext-ocr-fulltext-ocr python /tmp/k.py
"""
import sys

import numpy as np
import torch

print("torch", torch.__version__, "cuda", torch.version.cuda, "arch", torch.cuda.get_arch_list())
ok = "sm_120" in torch.cuda.get_arch_list()
print("sm_120 trong torch:", ok)

import paddle  # noqa: E402

print("paddle", paddle.__version__, "cuda", paddle.device.is_compiled_with_cuda(), paddle.version.cuda(), "cudnn", paddle.version.cudnn())

import paddleocr  # noqa: E402
from paddleocr import PaddleOCR  # noqa: E402

print("paddleocr", paddleocr.__version__)
det = PaddleOCR(lang="en", use_angle_cls=False, show_log=False, use_gpu=False)
r = det.ocr(np.full((64, 256, 3), 255, np.uint8), det=True, rec=False, cls=False)
print("DBNet CPU ok:", r)

import einops, vietocr  # noqa: E402,F401

print("vietocr ok")

try:
    print("cuda avail", torch.cuda.is_available(), torch.cuda.get_device_name(0))
    x = torch.ones(256, 256, device="cuda")
    print("torch matmul cuda ok", float((x @ x).sum()))
    del x
    torch.cuda.empty_cache()
except Exception as e:  # noqa: BLE001
    print("torch CUDA probe:", type(e).__name__, str(e)[:200])
try:
    paddle.set_device("gpu:0")
    y = paddle.ones([256, 256])
    print("paddle matmul gpu ok", float((y @ y).sum()))
except Exception as e:  # noqa: BLE001
    print("paddle GPU probe:", type(e).__name__, str(e)[:200])
sys.exit(0 if ok else 1)
