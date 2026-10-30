# Dịch vụ OCR toàn văn cho DigiText — chỉ chạy serve_ocr.py.
#
# CỐ Ý không cài pyodbc / torch / ultralytics / PyMuPDF: nhánh OCR không đụng
# CSDL và không chạy YOLO. Muốn chạy giao diện web/KIE cũ (app.py) thì dùng
# requirements.txt bản đầy đủ, không phải image này.
#
# Build context là thư mục `ocr/` ở gốc repo.

# Hai biến thể từ CÙNG một Dockerfile, chọn bằng build-arg:
#   CPU (mặc định): BASE_IMAGE=python:3.10-slim, torch +cpu, requirements-ocr.txt
#   GPU:            xem infra/compose/ocr/docker-compose.gpu.yml — base CUDA,
#                   torch cu121, requirements-ocr-gpu.txt, OCR_USE_GPU=true
ARG BASE_IMAGE=python:3.10-slim
FROM ${BASE_IMAGE}
ARG TORCH_INDEX=https://download.pytorch.org/whl/cpu
ARG TORCH_VERSION=2.4.1+cpu
ARG TORCHVISION_VERSION=0.19.1+cpu
ARG OCR_REQS=requirements-ocr.txt
ARG OCR_USE_GPU=false

# libgomp1:     PaddlePaddle cần OpenMP.
# libglib2.0-0: OpenCV cần.
# libgl1:       BẮT BUỘC dù ta đã ghim opencv-python-headless — paddleocr khai
#               `opencv-python` (bản đầy đủ) làm phụ thuộc riêng, pip cài đè lên,
#               và bản đó liên kết libGL. Thiếu nó container crash-loop với
#               "ImportError: libGL.so.1".
# python3/pip:  image CUDA của NVIDIA (Ubuntu 22.04) không có sẵn Python; trên
#               python:3.10-slim các gói này đã có, apt chỉ xác nhận.
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      libgomp1 libglib2.0-0 libgl1 python3 python3-pip python-is-python3 \
 && rm -rf /var/lib/apt/lists/* \
 && python -m pip install --no-cache-dir --upgrade pip

WORKDIR /app

# torch cài TRƯỚC và từ index riêng của PyTorch. Nếu để vietocr tự kéo torch từ
# PyPI, pip lấy bản mặc định kèm CUDA — thêm ~2,5GB vô ích cho biến thể CPU.
# Hai điều dễ sai ở bước này:
#  - PHẢI có --extra-index-url PyPI. Index của PyTorch chỉ chứa torch; các phụ
#    thuộc của nó (filelock, sympy, jinja2...) nằm ở PyPI. Dùng --index-url một
#    mình sẽ hỏng với "Could not find a version that satisfies flit_core".
#  - Biến thể CPU PHẢI ghim hậu tố +cpu; biến thể GPI dùng index cu121 và không
#    hậu tố. Không ghim thì pip có thể lấy bản khác từ PyPI vì hai index được
#    gộp chứ không xét theo thứ tự ưu tiên.
RUN pip install --no-cache-dir \
      torch==${TORCH_VERSION} torchvision==${TORCHVISION_VERSION} \
      --index-url ${TORCH_INDEX} \
      --extra-index-url https://pypi.org/simple

# Chặn ngay lúc build nếu pip lấy nhầm torch từ PyPI (30/10/2026, chuẩn bị card Blackwell): yêu cầu
# "2.7.1+cu128" mà về "2.7.1" (cu126, không có sm_120) thì bề ngoài vẫn chạy, tới request mới lỗi.
# TORCH_VERSION không hậu tố (2.4.1 trên Alienware) thì chỉ so phần số.
RUN python -c "import sys,torch; v=torch.__version__; w='${TORCH_VERSION}'; ok=(v==w) if '+' in w else (v.split('+')[0]==w); print('torch',v,'cuda',torch.version.cuda,'- yeu cau',w); sys.exit(0 if ok else 1)"

# Cài phụ thuộc trước, tách khỏi mã nguồn — sửa code không phải cài lại từ đầu.
COPY requirements-ocr.txt requirements-ocr-gpu.txt requirements-ocr-gpu-cu128.txt ./
RUN pip install --no-cache-dir -r ${OCR_REQS}

# vietocr cài RIÊNG với --no-deps.
#
# Lý do: paddleocr ghim albumentations==1.4.10 còn vietocr ghim ==1.4.2, pip
# không hoà giải được (ResolutionImpossible). Nhưng albumentations chỉ dùng để
# tăng cường dữ liệu LÚC HUẤN LUYỆN — đường suy luận của vietocr
# (tool.predictor → model.transformerocr) không chạm tới nó. Bản 1.4.10 mà
# paddleocr mang theo nằm im ở đó, vô hại.
#
# Trong 8 phụ thuộc vietocr khai, chỉ einops thực sự cần khi suy luận — nó nằm
# trong chính kiến trúc model (model/backbone/vgg.py, model/seqmodel/transformer.py).
# Số còn lại chỉ dùng lúc huấn luyện: albumentations+imgaug ở loader/aug.py,
# lmdb+prefetch_generator ở loader/dataloader*.py; scikit-image không được
# import ở đâu cả. torch/torchvision cài ở trên; Pillow, numpy, pyyaml theo
# paddleocr; gdown khai trong requirements.
RUN pip install --no-cache-dir --no-deps vietocr==0.3.13 \
 && pip install --no-cache-dir einops==0.8.0

# Chỉ chép những gì dịch vụ OCR thực sự dùng.
COPY ocr_fulltext.py ocr_routes.py serve_ocr.py handwriting.py inklines.py vietocr_fast.py ./
# Cau hinh VietOCR gop san — de container khong phai goi ra vocr.vn
# (chung chi TLS cua may chu do da het han).
COPY vietocr_config ./vietocr_config

# PaddleOCR tải trọng số về thư mục này ở lần chạy đầu (~100MB). Gắn volume
# vào đây, nếu không mỗi lần dựng lại container là tải lại.
ENV PADDLE_OCR_BASE_DIR=/models/paddleocr
ENV HOME=/models

# OCR_DET_LANG (không phải OCR_LANG — biến đó không có code nào đọc): ngôn ngữ
# model PHÁT HIỆN của PaddleOCR; 'en' là DBNet chung, tách dòng tốt cho tiếng Việt.
# PYTHONUNBUFFERED: log không bị đệm — bẫy "log trống mà tiến trình vẫn chạy".
ENV OCR_HOST=0.0.0.0 \
    OCR_PORT=5002 \
    OCR_THREADS=4 \
    OCR_DET_LANG=en \
    OCR_USE_GPU=${OCR_USE_GPU} \
    PYTHONUNBUFFERED=1

# Paddle dlopen thư viện CUDA theo TÊN KHÔNG PHIÊN BẢN (libcudnn.so, …) còn image
# runtime của NVIDIA chỉ có libcudnn.so.8 — lần chạy GPU đầu tiên (17/9/2026)
# nạp model xong nhưng mọi request trả 500 "Cannot load cudnn shared library".
# Tạo symlink cho những lib thiếu; trên image CPU (python:3.10-slim) vòng lặp
# không thấy file nào nên vô hại. Đặt cuối để không phá cache các layer pip.
# libcudnn.so.9 / libnvJitLink.so.12: image CUDA 12.9 + cuDNN 9 cho card Blackwell (30/10/2026); trên image
# cuDNN 8 hai tên đó không có nên bỏ qua.
RUN cd /usr/lib/x86_64-linux-gnu 2>/dev/null && for f in libcudnn.so.8 libcudnn.so.9 libcublas.so.12 \
      libcublasLt.so.12 libcurand.so.10 libcusolver.so.11 libcusparse.so.12 \
      libcufft.so.11 libnvrtc.so.12 libnvJitLink.so.12; do \
      if [ -e "$f" ] && [ ! -e "${f%%.so.*}.so" ]; then ln -s "$f" "${f%%.so.*}.so"; fi; \
    done; true

EXPOSE 5002

# python thay vì curl: image slim không có curl, thêm vào chỉ để healthcheck là thừa.
# start-period dài vì lần chạy đầu phải tải cả trọng số PaddleOCR lẫn VietOCR (+ warm-up GPU; trên WSL2
# volume đi qua 9p và Defender quét → 600 s, 30/10/2026).
HEALTHCHECK --interval=15s --timeout=5s --retries=30 --start-period=600s \
  CMD python -c "import urllib.request,sys; sys.exit(0 if urllib.request.urlopen('http://127.0.0.1:5002/api/ocr/health', timeout=4).status==200 else 1)" \
      || exit 1

CMD ["python", "serve_ocr.py"]
