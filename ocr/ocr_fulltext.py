"""OCR toàn văn — nhận ảnh, trả về từng DÒNG chữ kèm toạ độ hộp.

Module này CỐ Ý không import database.py. Nó phải chạy được khi không có
SQL Server, vì DigiText gọi vào đây thuần tuý để nhận dạng chữ.

Hợp đồng đầu ra bám sát `TileOcr` của DigiText (crates/digitext-ocr/src/infer.rs):
ba mảng `boxes` / `texts` / `confidences` LUÔN bằng độ dài, hộp dạng
`[x1, y1, x2, y2]` chữ nhật thẳng trục, pixel, gốc trên-trái.

Vì sao phải thẳng trục: lớp text vô hình của PDF hai lớp
(crates/digitext-pdf/src/lib.rs:103-156) chỉ phát `Tm 1 0 0 1 tx ty` — không có
phép xoay. Hộp nghiêng sẽ đặt chữ sai vị trí. PaddleOCR trả quad 4 điểm nên phải
quy về min/max.
"""
from __future__ import annotations

import os
import threading

import numpy as np


class OcrOutOfMemory(RuntimeError):
    """Hết bộ nhớ GPU — DigiText sẽ tự chia đôi ô rồi thử lại."""


class OcrNotReady(RuntimeError):
    """Chưa nạp được model."""


class OcrEngineManager:
    """Singleton thread-safe, theo đúng mẫu YOLOModelManager trong main.py:23-41.

    Nạp PaddleOCR MỘT lần cho cả tiến trình. Khởi tạo lại mỗi request sẽ giết
    hiệu năng — model detection + recognition mất vài giây để nạp.
    """

    _engine = None
    _lock = threading.Lock()
    _init_error: str | None = None

    @classmethod
    def get_engine(cls):
        if cls._engine is not None:
            return cls._engine
        with cls._lock:
            if cls._engine is not None:
                return cls._engine
            try:
                from paddleocr import PaddleOCR

                lang = os.getenv("OCR_LANG", "vi")
                use_gpu = os.getenv("OCR_USE_GPU", "false").lower() in ("1", "true", "yes")
                kwargs = dict(lang=lang, use_angle_cls=True, show_log=False)
                # `use_gpu` bị bỏ ở PaddleOCR 3.x — truyền vào sẽ nổ TypeError.
                if use_gpu:
                    kwargs["use_gpu"] = True
                try:
                    cls._engine = PaddleOCR(**kwargs)
                except TypeError:
                    kwargs.pop("show_log", None)
                    kwargs.pop("use_gpu", None)
                    cls._engine = PaddleOCR(**kwargs)
                cls._init_error = None
                print(f"✅ Đã nạp PaddleOCR (lang={lang}, gpu={use_gpu})")
            except Exception as e:  # noqa: BLE001 — muốn nuốt mọi lỗi nạp model
                cls._init_error = str(e)
                print(f"❌ Không nạp được PaddleOCR: {e}")
                cls._engine = None
        return cls._engine

    @classmethod
    def is_ready(cls) -> bool:
        return cls.get_engine() is not None

    @classmethod
    def init_error(cls) -> str | None:
        cls.get_engine()
        return cls._init_error


def quad_to_aabb(quad) -> list[float]:
    """Quad 4 điểm của PaddleOCR → chữ nhật thẳng trục [x1, y1, x2, y2]."""
    xs = [float(p[0]) for p in quad]
    ys = [float(p[1]) for p in quad]
    return [min(xs), min(ys), max(xs), max(ys)]


def _normalize_paddle_output(raw) -> list[tuple[list, str, float]]:
    """Gom về [(quad, text, conf)] — PaddleOCR 2.x và 3.x trả khác nhau.

    2.x: ocr.ocr(img) -> [[ [quad, (text, conf)], ... ]]   (một list cho mỗi ảnh)
    3.x: ocr.predict(img) -> [{'dt_polys': [...], 'rec_texts': [...], 'rec_scores': [...]}]
    """
    out: list[tuple[list, str, float]] = []
    if not raw:
        return out

    first = raw[0]

    # dạng 3.x
    if isinstance(first, dict):
        for page in raw:
            polys = page.get("dt_polys") or page.get("boxes") or []
            texts = page.get("rec_texts") or []
            scores = page.get("rec_scores") or []
            for i in range(min(len(polys), len(texts), len(scores))):
                out.append((polys[i], texts[i], float(scores[i])))
        return out

    # dạng 2.x — raw[0] là list các span của ảnh đầu tiên
    pages = raw if isinstance(first, list) and first and isinstance(first[0], (list, tuple)) else [raw]
    for page in pages:
        if not page:
            continue
        for span in page:
            try:
                quad, rec = span[0], span[1]
                text, conf = rec[0], float(rec[1])
            except (IndexError, TypeError, ValueError):
                continue
            out.append((quad, text, conf))
    return out


def recognize(image_bgr: np.ndarray) -> dict:
    """Nhận dạng một ảnh, trả về dict ba mảng CÙNG độ dài.

    Ảnh vào là mảng numpy BGR (quy ước của OpenCV, cũng là thứ PaddleOCR nhận).
    """
    engine = OcrEngineManager.get_engine()
    if engine is None:
        raise OcrNotReady(OcrEngineManager.init_error() or "PaddleOCR chưa sẵn sàng")

    try:
        if hasattr(engine, "ocr"):
            raw = engine.ocr(image_bgr, cls=True)
        else:
            raw = engine.predict(image_bgr)
    except Exception as e:  # noqa: BLE001
        msg = str(e).lower()
        if "out of memory" in msg or "cuda" in msg and "memory" in msg:
            raise OcrOutOfMemory(str(e)) from e
        raise

    boxes: list[list[float]] = []
    texts: list[str] = []
    confidences: list[float] = []

    for quad, text, conf in _normalize_paddle_output(raw):
        # Mỗi span PHẢI là một dòng đơn — ký tự xuống dòng làm hỏng phép giãn
        # ngang Tz khi dựng lớp text vô hình.
        text = " ".join(str(text).split())
        if not text:
            continue  # bỏ đồng thời ở CẢ BA mảng, giữ bất biến độ dài
        box = quad_to_aabb(quad)
        if box[2] <= box[0] or box[3] <= box[1]:
            continue  # hộp suy biến, không dựng được span
        boxes.append(box)
        texts.append(text)
        confidences.append(max(0.0, min(1.0, float(conf))))

    assert len(boxes) == len(texts) == len(confidences)
    return {"boxes": boxes, "texts": texts, "confidences": confidences}
