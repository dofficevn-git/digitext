# Dịch vụ OCR toàn văn cho DigiText — chỉ chạy serve_ocr.py.
#
# CỐ Ý không cài pyodbc / torch / ultralytics / PyMuPDF: nhánh OCR không đụng
# CSDL và không chạy YOLO. Muốn chạy giao diện web/KIE cũ (app.py) thì dùng
# requirements.txt bản đầy đủ, không phải image này.
#
# Build context là thư mục `ocr/` ở gốc repo.

FROM python:3.10-slim

# libgomp1:     PaddlePaddle cần OpenMP.
# libglib2.0-0: OpenCV cần.
# libgl1:       BẮT BUỘC dù ta đã ghim opencv-python-headless — paddleocr khai
#               `opencv-python` (bản đầy đủ) làm phụ thuộc riêng, pip cài đè lên,
#               và bản đó liên kết libGL. Thiếu nó container crash-loop với
#               "ImportError: libGL.so.1".
RUN apt-get update \
 && apt-get install -y --no-install-recommends libgomp1 libglib2.0-0 libgl1 \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# torch bản CPU, cài TRƯỚC và từ index riêng của PyTorch. Nếu để vietocr tự kéo
# torch từ PyPI, pip lấy bản mặc định kèm CUDA — thêm ~2,5GB vô ích vì dịch vụ
# này chạy CPU. Cài trước thì bước sau thấy đã thoả và bỏ qua.
# Hai điều dễ sai ở bước này:
#  - PHẢI có --extra-index-url PyPI. Index của PyTorch chỉ chứa torch; các phụ
#    thuộc của nó (filelock, sympy, jinja2...) nằm ở PyPI. Dùng --index-url một
#    mình sẽ hỏng với "Could not find a version that satisfies flit_core".
#  - PHẢI ghim hậu tố +cpu. Không có nó, pip có thể lấy bản CUDA từ PyPI vì hai
#    index được gộp chứ không xét theo thứ tự ưu tiên.
RUN pip install --no-cache-dir \
      torch==2.4.1+cpu torchvision==0.19.1+cpu \
      --index-url https://download.pytorch.org/whl/cpu \
      --extra-index-url https://pypi.org/simple

# Cài phụ thuộc trước, tách khỏi mã nguồn — sửa code không phải cài lại từ đầu.
COPY requirements-ocr.txt .
RUN pip install --no-cache-dir -r requirements-ocr.txt

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
COPY ocr_fulltext.py ocr_routes.py serve_ocr.py ./
# Cau hinh VietOCR gop san — de container khong phai goi ra vocr.vn
# (chung chi TLS cua may chu do da het han).
COPY vietocr_config ./vietocr_config

# PaddleOCR tải trọng số về thư mục này ở lần chạy đầu (~100MB). Gắn volume
# vào đây, nếu không mỗi lần dựng lại container là tải lại.
ENV PADDLE_OCR_BASE_DIR=/models/paddleocr
ENV HOME=/models

ENV OCR_HOST=0.0.0.0 \
    OCR_PORT=5002 \
    OCR_THREADS=4 \
    OCR_LANG=vi \
    OCR_USE_GPU=false

EXPOSE 5002

# python thay vì curl: image slim không có curl, thêm vào chỉ để healthcheck là thừa.
# start-period dài vì lần chạy đầu phải tải cả trọng số PaddleOCR lẫn VietOCR.
HEALTHCHECK --interval=15s --timeout=5s --retries=30 --start-period=420s \
  CMD python -c "import urllib.request,sys; sys.exit(0 if urllib.request.urlopen('http://127.0.0.1:5002/api/ocr/health', timeout=4).status==200 else 1)" \
      || exit 1

CMD ["python", "serve_ocr.py"]
