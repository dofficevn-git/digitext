"""OCR toàn văn — nhận ảnh, trả về từng DÒNG chữ kèm toạ độ hộp.

Module này CỐ Ý không import database.py. Nó phải chạy được khi không có
SQL Server, vì DigiText gọi vào đây thuần tuý để nhận dạng chữ.

KIẾN TRÚC HAI TẦNG — và vì sao:
    phát hiện  : PaddleOCR (DBNet)
    nhận dạng  : VietOCR (transformer chuyên tiếng Việt)

Ban đầu dùng PaddleOCR cho cả hai. Đo trên 20 văn bản hành chính thật cho kết
quả 17,8%: `lang='vi'` KHÔNG nạp model tiếng Việt mà tải `latin_PP-OCRv3_rec`
— model Latin chung, bảng ký tự không có dấu thanh. Đầu ra mất sạch dấu và dính
chữ: "CHỦ TỊCH" thành "CHUTICH", "Lê Văn Lâm" thành "Le Van Lam".

Nhưng phần PHÁT HIỆN thì tốt — hộp đúng vị trí, tách dòng chuẩn. Nên chỉ thay
khâu đọc chữ. Đây đúng là cách DigiText nguyên bản làm: det_dbnet phát hiện,
rec_vietocr nhận dạng.

Hợp đồng đầu ra bám sát `TileOcr` của DigiText (crates/digitext-ocr/src/infer.rs):
ba mảng `boxes` / `texts` / `confidences` LUÔN bằng độ dài, hộp dạng
`[x1, y1, x2, y2]` chữ nhật thẳng trục, pixel, gốc trên-trái.

Vì sao phải thẳng trục: lớp text vô hình của PDF hai lớp
(crates/digitext-pdf/src/lib.rs:103-156) chỉ phát `Tm 1 0 0 1 tx ty` — không có
phép xoay. Hộp nghiêng sẽ đặt chữ sai vị trí.
"""
from __future__ import annotations

import os
import threading
import time

import cv2
import numpy as np


class OcrOutOfMemory(RuntimeError):
    """Hết bộ nhớ — DigiText sẽ tự chia đôi ô rồi thử lại."""


class OcrNotReady(RuntimeError):
    """Chưa nạp được model."""


def _is_oom(e: Exception) -> bool:
    m = str(e).lower()
    return "out of memory" in m or ("cuda" in m and "memory" in m)


def _env_flag(name: str, default: str = "false") -> bool:
    return os.getenv(name, default).strip().lower() in ("1", "true", "yes")


def use_gpu() -> bool:
    return _env_flag("OCR_USE_GPU")


# Trần số suy luận chạy ĐỒNG THỜI. waitress có OCR_THREADS luồng nhận request,
# nhưng mỗi suy luận trên GPU chiếm VRAM = batch × model; N luồng cùng chạy thì
# đỉnh VRAM nhân N mà không có trần nào (mem_limit của compose chỉ bó RAM).
# Semaphore giữ HTTP vẫn nhận request (xếp hàng) mà VRAM có trần.
_INFER_SEM = threading.BoundedSemaphore(max(1, int(os.getenv("OCR_GPU_CONCURRENCY", "2"))))


def _release_cuda() -> None:
    """Sau OOM: trả cache CUDA để lần thử lại (ô nhỏ hơn) có chỗ."""
    try:
        import torch

        if torch.cuda.is_available():
            torch.cuda.empty_cache()
    except Exception:  # noqa: BLE001
        pass


def device_info() -> dict:
    """Cho /api/ocr/health: máy đang chạy trên gì. Trước đây health chỉ nói
    'ready', không phân biệt được cấu hình GPU bị rơi về CPU."""
    info: dict = {"device": "cuda:0" if use_gpu() else "cpu", "gpu": False}
    try:
        import torch

        if use_gpu() and torch.cuda.is_available():
            info["gpu"] = True
            info["gpu_name"] = torch.cuda.get_device_name(0)
            info["torch_mem_mb"] = round(torch.cuda.memory_allocated(0) / 1e6, 1)
            info["torch_reserved_mb"] = round(torch.cuda.memory_reserved(0) / 1e6, 1)
    except Exception as e:  # noqa: BLE001
        info["gpu_error"] = str(e)
    info["infer_concurrency"] = int(os.getenv("OCR_GPU_CONCURRENCY", "2"))
    info["rec_batch"] = int(os.getenv("OCR_REC_BATCH", "8"))
    return info


class DetectorManager:
    """PaddleOCR dùng RIÊNG cho phát hiện vùng chữ. Singleton thread-safe,
    theo đúng mẫu YOLOModelManager trong main.py:23-41."""

    _engine = None
    _lock = threading.Lock()
    _error: str | None = None

    @classmethod
    def get(cls):
        if cls._engine is not None:
            return cls._engine
        with cls._lock:
            if cls._engine is not None:
                return cls._engine
            try:
                from paddleocr import PaddleOCR

                gpu = use_gpu()
                # use_gpu/gpu_mem là API PaddleOCR 2.x (bản đang ghim). Trước đây
                # KHÔNG truyền nên dù OCR_USE_GPU=true thì DBNet vẫn chạy CPU —
                # chỉ VietOCR lên GPU. gpu_mem là trần MB paddle được giữ.
                kwargs = dict(lang=os.getenv("OCR_DET_LANG", "en"),
                              use_angle_cls=False, show_log=False,
                              use_gpu=gpu,
                              gpu_mem=int(os.getenv("OCR_DET_GPU_MEM_MB", "1024")))
                for attempt in range(3):
                    try:
                        cls._engine = PaddleOCR(**kwargs)
                        break
                    except TypeError:
                        # Bản khác API: rụng dần tham số, lần cuối chuyển sang
                        # kiểu 3.x `device=`.
                        if "show_log" in kwargs:
                            kwargs.pop("show_log")
                        elif "use_gpu" in kwargs:
                            kwargs.pop("use_gpu")
                            kwargs.pop("gpu_mem", None)
                            kwargs["device"] = "gpu" if gpu else "cpu"
                        else:
                            raise
                cls._error = None
                print(f"✅ Đã nạp PaddleOCR (chỉ dùng để PHÁT HIỆN vùng chữ, "
                      f"{'GPU' if gpu else 'CPU'})", flush=True)
            except Exception as e:  # noqa: BLE001
                cls._error = str(e)
                print(f"❌ Không nạp được PaddleOCR: {e}")
        return cls._engine

    @classmethod
    def error(cls) -> str | None:
        return cls._error


class RecognizerManager:
    """VietOCR — đọc chữ tiếng Việt có dấu."""

    _predictor = None
    _lock = threading.Lock()
    _error: str | None = None

    @classmethod
    def get(cls):
        if cls._predictor is not None:
            return cls._predictor
        with cls._lock:
            if cls._predictor is not None:
                return cls._predictor
            try:
                from vietocr.tool.config import Cfg
                from vietocr.tool.predictor import Predictor

                # PHẢI dùng load_config_from_file, KHÔNG dùng
                # load_config_from_name: hàm kia tải cấu hình từ https://vocr.vn
                # mà chứng chỉ TLS của máy chủ đó đã hết hạn — dịch vụ sẽ không
                # khởi động nổi. Cấu hình gộp sẵn nằm trong repo.
                cfg_path = os.getenv(
                    "OCR_REC_CONFIG",
                    os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 "vietocr_config", "vgg_transformer.yml"),
                )
                if not os.path.exists(cfg_path):
                    raise FileNotFoundError(f"thiếu cấu hình VietOCR: {cfg_path}")
                cfg = Cfg.load_config_from_file(cfg_path)

                weights = os.getenv("OCR_REC_WEIGHTS", cfg.get("weights", ""))
                if not weights or not os.path.exists(weights):
                    raise FileNotFoundError(
                        f"thiếu trọng số VietOCR: {weights!r} — cấp phát khi triển khai "
                        "(xem infra/compose/ocr/docker-compose.yml)"
                    )
                cfg["weights"] = weights
                cfg["device"] = "cuda:0" if use_gpu() else "cpu"
                if cfg["device"] != "cpu":
                    import torch

                    if not torch.cuda.is_available():
                        # Không im lặng rơi về CPU: đó chính là lỗi "20 s/trang
                        # trên máy có GPU" mà không ai thấy.
                        raise RuntimeError(
                            "OCR_USE_GPU=true nhưng torch không thấy CUDA — "
                            "kiểm tra image (torch cu121?) và `--gpus`/device reservation"
                        )
                cfg["predictor"]["beamsearch"] = False  # nhanh hơn nhiều, sai lệch không đáng kể
                cls._predictor = Predictor(cfg)
                cls._error = None
                print(f"✅ Đã nạp VietOCR ({os.path.basename(cfg_path)}, {cfg['device']}, "
                      f"trọng số {os.path.basename(weights)})", flush=True)
            except Exception as e:  # noqa: BLE001
                cls._error = str(e)
                print(f"❌ Không nạp được VietOCR: {e}")
        return cls._predictor

    @classmethod
    def error(cls) -> str | None:
        return cls._error


class OcrEngineManager:
    """Mặt tiền chung cho hai tầng — health check hỏi qua đây."""

    @classmethod
    def is_ready(cls) -> bool:
        return DetectorManager.get() is not None and RecognizerManager.get() is not None

    @classmethod
    def init_error(cls) -> str | None:
        cls.is_ready()
        return DetectorManager.error() or RecognizerManager.error()


_warned_no_prob = False


def _warn_no_prob() -> None:
    global _warned_no_prob
    if not _warned_no_prob:
        _warned_no_prob = True
        print("⚠  VietOCR không trả xác suất — mọi span nhận confidence 0.9. "
              "DigiText sẽ KHÔNG đánh dấu được trang tin cậy thấp để soát.", flush=True)


def quad_to_aabb(quad) -> list[float]:
    """Quad 4 điểm của PaddleOCR → chữ nhật thẳng trục [x1, y1, x2, y2]."""
    xs = [float(p[0]) for p in quad]
    ys = [float(p[1]) for p in quad]
    return [min(xs), min(ys), max(xs), max(ys)]


def warp_crop(img: np.ndarray, quad) -> np.ndarray | None:
    """Cắt vùng chữ theo quad, nắn thẳng bằng phép biến đổi phối cảnh.

    Cắt theo hộp thẳng trục sẽ kéo theo nền và chữ của dòng bên cạnh khi trang
    scan bị nghiêng — nắn thẳng trước giúp VietOCR đọc chính xác hơn hẳn.
    """
    pts = np.array(quad, dtype=np.float32)
    if pts.shape != (4, 2):
        return None
    w = int(max(np.linalg.norm(pts[0] - pts[1]), np.linalg.norm(pts[2] - pts[3])))
    h = int(max(np.linalg.norm(pts[0] - pts[3]), np.linalg.norm(pts[1] - pts[2])))
    if w < 4 or h < 4:
        return None
    # ĐỆM khung trước khi cắt. Khung DBNet ôm sát thân chữ; dấu nặng (ạ ệ ị ộ ự)
    # nằm DƯỚI dòng kẻ và dấu trên chữ hoa (Ộ Ấ) nằm trên — cắt sát là mất dấu.
    # Đo 17/9/2026 trên 341 dòng thân văn bản: lỗi chủ đạo còn lại của VietOCR là
    # a→ạ, ê→ệ, Ô→Ộ, i→ị… Đệm tính theo CHIỀU CAO dòng, theo hướng của chính quad
    # (trang nghiêng vẫn đúng). Hộp trả về cho lớp chữ PDF vẫn là quad GỐC.
    pad_y = float(os.getenv("OCR_CROP_PAD_Y", "0") or 0)
    pad_x = float(os.getenv("OCR_CROP_PAD_X", "0") or 0)
    if pad_y > 0 or pad_x > 0:
        tl, tr, br, bl = pts

        def _unit(v):
            n = float(np.linalg.norm(v))
            return v / n if n > 1e-6 else v

        down_l, down_r = _unit(bl - tl), _unit(br - tr)
        right_t, right_b = _unit(tr - tl), _unit(br - bl)
        dy, dx = pad_y * h, pad_x * h
        pts = np.array([
            tl - down_l * dy - right_t * dx,
            tr - down_r * dy + right_t * dx,
            br + down_r * dy + right_b * dx,
            bl + down_l * dy - right_b * dx,
        ], dtype=np.float32)
        w = int(max(np.linalg.norm(pts[0] - pts[1]), np.linalg.norm(pts[2] - pts[3])))
        h = int(max(np.linalg.norm(pts[0] - pts[3]), np.linalg.norm(pts[1] - pts[2])))
    dst = np.array([[0, 0], [w, 0], [w, h], [0, h]], dtype=np.float32)
    m = cv2.getPerspectiveTransform(pts, dst)
    return cv2.warpPerspective(img, m, (w, h), borderMode=cv2.BORDER_REPLICATE)


def _detect(img_bgr: np.ndarray) -> list:
    """-> danh sách quad 4 điểm."""
    det = DetectorManager.get()
    if det is None:
        raise OcrNotReady(DetectorManager.error() or "PaddleOCR chưa sẵn sàng")
    try:
        raw = det.ocr(img_bgr, det=True, rec=False, cls=False)
    except Exception as e:  # noqa: BLE001
        if _is_oom(e):
            _release_cuda()
            raise OcrOutOfMemory(str(e)) from e
        raise
    if not raw:
        return []
    first = raw[0]
    if not isinstance(first, (list, tuple)) or not len(first):
        return []
    # Hai dạng trả về, phân biệt bằng phần tử SÂU NHẤT chứ không bằng độ dài:
    #   2.x det-only : [[quad, quad, ...]]  -> first[0] là một quad, first[0][0] là ĐIỂM
    #   một số bản   : [quad, quad, ...]    -> first[0] là một ĐIỂM (hai số)
    inner = first[0]
    if isinstance(inner, (list, tuple)) and len(inner) and isinstance(
        inner[0], (list, tuple, np.ndarray)
    ):
        return list(first)
    return list(raw)


def recognize(image_bgr: np.ndarray) -> dict:
    """Nhận dạng một ảnh, trả về dict ba mảng CÙNG độ dài."""
    rec = RecognizerManager.get()
    if rec is None:
        raise OcrNotReady(RecognizerManager.error() or "VietOCR chưa sẵn sàng")
    t_wait = time.perf_counter()
    with _INFER_SEM:
        waited_ms = (time.perf_counter() - t_wait) * 1000.0
        t0 = time.perf_counter()
        result, det_ms = _recognize_locked(image_bgr, rec)
        total_ms = (time.perf_counter() - t0) * 1000.0
    if _env_flag("OCR_LOG_TIMING", "true"):
        h, w = image_bgr.shape[:2]
        print(f"⏱ tile {w}x{h} · dòng={len(result['texts'])} · det={det_ms:.0f}ms · "
              f"tổng={total_ms:.0f}ms · chờ={waited_ms:.0f}ms", flush=True)
    return result


def _recognize_locked(image_bgr: np.ndarray, rec) -> tuple[dict, float]:
    from PIL import Image

    t_det = time.perf_counter()
    quads = _detect(image_bgr)
    det_ms = (time.perf_counter() - t_det) * 1000.0
    crops, keep, dropped = [], [], 0
    for q in quads:
        patch = warp_crop(image_bgr, q)
        if patch is None:
            dropped += 1
            continue
        crops.append(Image.fromarray(cv2.cvtColor(patch, cv2.COLOR_BGR2RGB)))
        keep.append(q)

    if not crops:
        # Ảnh trắng thì im lặng là đúng; nhưng phát hiện ra vùng chữ mà cắt hỏng
        # HẾT là dấu hiệu phân tích sai đầu ra của bộ phát hiện — từng xảy ra.
        if quads:
            print(f"⚠  phát hiện {len(quads)} vùng nhưng cắt hỏng cả {dropped} — "
                  "kiểm tra lại dạng dữ liệu trả về của bộ phát hiện", flush=True)
        return {"boxes": [], "texts": [], "confidences": []}, det_ms

    # Thứ tự ưu tiên: batch KÈM xác suất > batch trần > từng ảnh một.
    #
    # Xác suất không phải thứ trang trí: DigiText dùng nó tính low_conf_ratio để
    # quyết định trang nào phải đưa vào hàng soát của người
    # (crates/digitext-ocr/src/worker.rs:131-139). Gán cứng một giá trị đẹp sẽ
    # khiến KHÔNG trang nào bị đánh dấu — mất hẳn lưới an toàn đó.
    # CHIA MẺ. Đẩy cả trang vào một lần gọi là nguồn gốc của hai sự cố thật:
    # một trang 30 dòng nghĩa là 30 ảnh chạy qua VGG19 + transformer cùng lúc,
    # fp32 trên CPU — đỉnh bộ nhớ vọt lên hàng GB. Không có trần thì nó ăn RAM
    # máy chủ tới khi OOM killer nổ và kéo sập cả phiên; có trần thì container
    # tự chết và bên gọi nhận "connection reset by peer".
    #
    # Số dòng mỗi trang là do TÀI LIỆU quyết định, không phải ta — nên không
    # được để nó điều khiển lượng bộ nhớ dùng. Mẻ cố định thì đỉnh bộ nhớ có
    # trần, bất kể trang dày bao nhiêu.
    batch = max(1, int(os.getenv("OCR_REC_BATCH", "8")))
    try:
        texts, probs = [], []
        for i in range(0, len(crops), batch):
            chunk = crops[i:i + batch]
            done = False
            if hasattr(rec, "predict_batch"):
                try:
                    t, p = rec.predict_batch(chunk, return_prob=True)
                    texts.extend(t)
                    probs.extend(p)
                    done = True
                except TypeError:
                    t = rec.predict_batch(chunk)
                    texts.extend(t)
                    probs.extend([None] * len(t))
                    done = True
            if not done:
                for c in chunk:
                    t, p = rec.predict(c, return_prob=True)
                    texts.append(t)
                    probs.append(p)
    except Exception as e:  # noqa: BLE001
        if _is_oom(e):
            _release_cuda()
            raise OcrOutOfMemory(str(e)) from e
        raise

    boxes: list[list[float]] = []
    out_texts: list[str] = []
    confidences: list[float] = []

    for q, text, prob in zip(keep, texts, probs):
        # Mỗi span PHẢI là một dòng đơn — ký tự xuống dòng làm hỏng phép giãn
        # ngang Tz khi dựng lớp text vô hình.
        text = " ".join(str(text).split())
        if not text:
            continue  # bỏ đồng thời ở CẢ BA mảng, giữ bất biến độ dài
        box = quad_to_aabb(q)
        if box[2] <= box[0] or box[3] <= box[1]:
            continue
        boxes.append(box)
        out_texts.append(text)
        # Chỉ rơi vào 0.9 khi bản VietOCR này không trả nổi xác suất — lúc đó
        # tín hiệu low_conf_ratio coi như mất, ghi log một lần để biết.
        if prob is None:
            _warn_no_prob()
            confidences.append(0.9)
        else:
            confidences.append(max(0.0, min(1.0, float(prob))))

    assert len(boxes) == len(out_texts) == len(confidences)
    return {"boxes": boxes, "texts": out_texts, "confidences": confidences}, det_ms
