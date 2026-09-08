# Giao diện web + KIE của doc_extract_pla (app.py).
#
# KHÁC với infra/compose/ocr/Dockerfile — cái đó chỉ chạy serve_ocr.py, một dịch
# vụ tối giản không đụng CSDL. Image này chạy ứng dụng ĐẦY ĐỦ: giao diện web,
# quản lý lô, gói việc, và nhánh KIE dùng YOLO + VLM.
#
# Hai thứ image kia không cần mà đây bắt buộc:
#   msodbcsql17  — pyodbc nói chuyện với SQL Server. Code viết cho Windows với
#                  "{ODBC Driver 17 for SQL Server}"; trên Linux phải cài driver
#                  của Microsoft, không có sẵn trong kho Debian.
#   poppler-utils — pdf2image gọi binary pdftoppm để render PDF (main.py:471).
#
# Build context là thư mục `ocr/` ở gốc repo.

FROM python:3.10-slim

# Driver ODBC của Microsoft nằm ở kho riêng, phải thêm khoá và nguồn.
RUN apt-get update \
 && apt-get install -y --no-install-recommends curl gnupg ca-certificates \
 && curl -fsSL https://packages.microsoft.com/keys/microsoft.asc \
      | gpg --dearmor -o /usr/share/keyrings/microsoft.gpg \
 && echo "deb [signed-by=/usr/share/keyrings/microsoft.gpg] https://packages.microsoft.com/debian/12/prod bookworm main" \
      > /etc/apt/sources.list.d/mssql-release.list \
 && apt-get update \
 && ACCEPT_EULA=Y apt-get install -y --no-install-recommends \
      msodbcsql17 unixodbc \
      poppler-utils \
      libgl1 libglib2.0-0 libgomp1 \
 && apt-get purge -y gnupg && apt-get autoremove -y \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# torch bản CPU: YOLO cắt vùng header chạy CPU thoải mái, và GPU trên máy này
# đã dành cho vLLM. Cài trước từ index riêng — xem ghi chú ở
# infra/compose/ocr/Dockerfile về hai bẫy của --index-url.
RUN pip install --no-cache-dir \
      torch==2.4.1+cpu torchvision==0.19.1+cpu \
      --index-url https://download.pytorch.org/whl/cpu \
      --extra-index-url https://pypi.org/simple

COPY requirements-web.txt .
RUN pip install --no-cache-dir -r requirements-web.txt

# Toàn bộ ứng dụng. Khác image kia ở chỗ cần cả templates/ và static/.
COPY . /app

ENV FLASK_HOST=0.0.0.0 \
    FLASK_PORT=5001

EXPOSE 5001

# app.py gọi db_manager.init_database() TRƯỚC khi mở cổng, và exit(1) nếu
# SQL Server chưa tới được (app.py:2180-2186). Nên cổng mở nghĩa là CSDL đã
# thông — đó chính là thứ healthcheck cần biết.
#
# Nhưng nó KHÔNG bảo đảm vLLM sống hay best.pt có mặt: DocumentKIE.__init__
# chỉ đọc biến môi trường, không hề kết nối, và nuốt lỗi nạp YOLO thành
# self.yolo_model = None. Hai thứ đó hỏng thì container vẫn "healthy" và chỉ
# đổ vỡ lúc xử lý tài liệu. Đừng tin healthcheck này quá mức nó nói.
HEALTHCHECK --interval=20s --timeout=5s --retries=15 --start-period=180s \
  CMD python -c "import socket,sys; s=socket.socket(); s.settimeout(4); sys.exit(s.connect_ex(('127.0.0.1',5001)))"

CMD ["python", "app.py"]
