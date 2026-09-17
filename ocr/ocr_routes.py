"""Blueprint OCR toàn văn — bề mặt HTTP cho DigiText gọi vào.

Đây là giao diện MÁY GỌI MÁY: không có phiên đăng nhập, xác thực bằng API key.
Các route ở đây được miễn trừ khỏi `require_login` trong app.py.

MÃ LỖI HTTP LÀ MỘT PHẦN CỦA HỢP ĐỒNG, không phải chi tiết cài đặt.
Phía DigiText (crates/digitext-ocr) ánh xạ chúng thành `WorkerError` và dùng để
quyết định thử lại hay đánh hỏng trang:

    507  → GpuOom            tạm thời, VÀ kích hoạt cơ chế tự chia đôi ô
    504  → Timeout           tạm thời, thử lại
    502  → TritonUnavailable tạm thời, thử lại
    500  → TritonUnavailable tạm thời, thử lại
    400  → CorruptInput      VĨNH VIỄN, trang bị đánh hỏng
    501  → ConfigInvalid     VĨNH VIỄN

Trả sai mã ở đây sẽ khiến job lặp vô hạn, hoặc trang tốt bị đánh hỏng oan.
"""
from __future__ import annotations

import base64
import binascii
import os

import cv2
import numpy as np
from flask import Blueprint, jsonify, request

from ocr_fulltext import (
    OcrEngineManager,
    OcrNotReady,
    OcrOutOfMemory,
    device_info,
    recognize,
)

ocr_bp = Blueprint("ocr", __name__)

_API_KEY = os.getenv("OCR_API_KEY", "").strip()
if not _API_KEY:
    print("⚠  OCR_API_KEY chưa đặt — endpoint /api/ocr/* đang MỞ, chỉ dùng cho dev")


def _check_api_key():
    """None nếu hợp lệ, ngược lại trả response 401."""
    if not _API_KEY:
        return None
    if request.headers.get("X-API-Key", "") == _API_KEY:
        return None
    return jsonify({"error": "api key không hợp lệ"}), 401


@ocr_bp.route("/api/ocr/health", methods=["GET"])
def health():
    """DigiText dùng cho /readyz. `ready=false` khi model chưa nạp được."""
    ready = OcrEngineManager.is_ready()
    body = {"ready": ready}
    if not ready:
        body["error"] = OcrEngineManager.init_error() or "chưa nạp được model"
    # Cho biết đang chạy GPU hay CPU — cấu hình rơi về CPU phải nhìn thấy được.
    body.update(device_info())
    return jsonify(body), (200 if ready else 503)


@ocr_bp.route("/api/ocr/tile", methods=["POST"])
def tile():
    """Nhận một ô ảnh, trả về từng dòng chữ kèm hộp bao.

    Vào:  {"image": "<base64 PNG hoặc JPEG>"}
    Ra:   {"boxes": [[x1,y1,x2,y2], …], "texts": [...], "confidences": [...]}

    Toạ độ là pixel TRONG Ô, gốc trên-trái — DigiText tự dịch về toạ độ trang.
    """
    denied = _check_api_key()
    if denied:
        return denied

    payload = request.get_json(silent=True)
    if not isinstance(payload, dict):
        return jsonify({"error": "thân yêu cầu phải là JSON"}), 400

    b64 = payload.get("image")
    if not isinstance(b64, str) or not b64:
        return jsonify({"error": "thiếu trường 'image' (base64)"}), 400

    try:
        blob = base64.b64decode(b64, validate=True)
    except (binascii.Error, ValueError):
        return jsonify({"error": "'image' không phải base64 hợp lệ"}), 400

    buf = np.frombuffer(blob, dtype=np.uint8)
    img = cv2.imdecode(buf, cv2.IMREAD_COLOR)
    if img is None or img.size == 0:
        return jsonify({"error": "không giải mã được ảnh"}), 400

    try:
        result = recognize(img)
    except OcrOutOfMemory as e:
        # 507 là tín hiệu để DigiText chia đôi ô rồi thử lại — KHÔNG đổi mã này.
        return jsonify({"error": f"hết bộ nhớ: {e}"}), 507
    except OcrNotReady as e:
        return jsonify({"error": f"model chưa sẵn sàng: {e}"}), 501
    except Exception as e:  # noqa: BLE001
        return jsonify({"error": f"lỗi nhận dạng: {e}"}), 500

    return jsonify(result), 200
