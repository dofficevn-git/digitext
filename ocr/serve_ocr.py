"""Điểm vào GỌN cho dịch vụ OCR toàn văn — chỉ phục vụ DigiText.

Vì sao không dùng app.py: app.py khởi tạo DatabaseManager ở mức module
(app.py:174) và gọi init_database() trước khi serve (app.py:2171), nên chạy nó
BẮT BUỘC phải có SQL Server. Nhánh OCR thì không đụng CSDL — không có lý do gì
để kéo theo phụ thuộc đó.

Tiến trình này chỉ nạp PaddleOCR và mở hai endpoint. Không DB, không phiên đăng
nhập, không giao diện web, không YOLO, không torch.

    python serve_ocr.py

Giao diện web/KIE cũ vẫn chạy bằng app.py như trước, độc lập hoàn toàn.
"""
from __future__ import annotations

import os

from dotenv import load_dotenv
from flask import Flask, jsonify
from waitress import serve

load_dotenv()

from ocr_fulltext import OcrEngineManager  # noqa: E402 — phải sau load_dotenv
from ocr_routes import ocr_bp  # noqa: E402

app = Flask(__name__)
app.register_blueprint(ocr_bp)


@app.route("/", methods=["GET"])
def root():
    return jsonify(
        {
            "service": "digitext-ocr-fulltext",
            "endpoints": ["GET /api/ocr/health", "POST /api/ocr/tile"],
        }
    )


def main() -> None:
    host = os.getenv("OCR_HOST", "127.0.0.1")
    port = int(os.getenv("OCR_PORT", "5002"))
    threads = int(os.getenv("OCR_THREADS", "4"))

    # Nạp model NGAY lúc khởi động thay vì đợi request đầu tiên: nạp mất vài
    # giây, và ta muốn /api/ocr/health nói thật ngay từ đầu.
    print("→ đang nạp engine OCR...")
    ready = OcrEngineManager.is_ready()
    if ready:
        print(f"✅ sẵn sàng — nghe tại http://{host}:{port}")
    else:
        # KHÔNG thoát: /api/ocr/health trả 503 để bên gọi biết, và tiến trình
        # vẫn sống để container không rơi vào vòng lặp khởi động lại.
        print(f"⚠  engine CHƯA sẵn sàng: {OcrEngineManager.init_error()}")
        print(f"   vẫn nghe tại http://{host}:{port} — /api/ocr/health sẽ trả 503")

    serve(app, host=host, port=port, threads=threads, channel_timeout=300)


if __name__ == "__main__":
    main()
