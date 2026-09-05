# Dịch vụ OCR toàn văn cho DigiText — chỉ chạy serve_ocr.py.
#
# CỐ Ý không cài pyodbc / torch / ultralytics / PyMuPDF: nhánh OCR không đụng
# CSDL và không chạy YOLO. Muốn chạy giao diện web/KIE cũ (app.py) thì dùng
# requirements.txt bản đầy đủ, không phải image này.
#
# Build context là thư mục `ocr/` ở gốc repo.

FROM python:3.10-slim

# libgomp1: PaddlePaddle cần OpenMP.
# libglib2.0-0: opencv-python-headless vẫn cần, dù không kéo theo X11.
RUN apt-get update \
 && apt-get install -y --no-install-recommends libgomp1 libglib2.0-0 \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cài phụ thuộc trước, tách khỏi mã nguồn — sửa code không phải cài lại từ đầu.
COPY requirements-ocr.txt .
RUN pip install --no-cache-dir -r requirements-ocr.txt

# Chỉ chép những gì dịch vụ OCR thực sự dùng.
COPY ocr_fulltext.py ocr_routes.py serve_ocr.py ./

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
HEALTHCHECK --interval=15s --timeout=5s --retries=20 --start-period=180s \
  CMD python -c "import urllib.request,sys; sys.exit(0 if urllib.request.urlopen('http://127.0.0.1:5002/api/ocr/health', timeout=4).status==200 else 1)" \
      || exit 1

CMD ["python", "serve_ocr.py"]
