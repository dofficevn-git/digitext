"""DBNet (en_PP-OCRv3_det) qua ONNX Runtime — dự phòng cho card Blackwell nếu paddlepaddle-gpu không có kernel sm_120
(30/10/2026). Tái tạo ĐÚNG tiền/hậu xử lý của PaddleOCR 2.x để hộp trả về trùng với bản Paddle (golden set không lệch):

  DetResizeForTest(limit_side_len=960, limit_type='max') → NormalizeImage(1/255, mean/std ImageNet, HWC)
  → ToCHW → model → DBPostProcess(thresh=0.3, box_thresh=0.6, max_candidates=1000, unclip_ratio=1.5, score_mode='fast')
  → sorted_boxes (trên xuống, trái sang).

Tệp ONNX xuất bằng paddle2onnx từ /models/.paddleocr/whl/det/en/en_PP-OCRv3_det_infer (opset 16, đầu vào động
'x' [N,3,H,W], đầu ra 'sigmoid_0.tmp_0' [N,1,H,W]); SHA-256 072ce94a…44eff. Đặt ở OCR_DET_ONNX (mặc định
/models/det/en_PP-OCRv3_det.onnx, bind-mount ocr/models/det như vietocr). KHÔNG tái dùng glue Triton
(triton/models/ocr_pipeline — resize cứng 1024², unclip tự chế → hộp khác).

Dùng: det = OnnxDetector(path, use_gpu); det.ocr(img_bgr, det=True, rec=False, cls=False) → [[quad, ...]] như PaddleOCR 2.x
"""
from __future__ import annotations

import os

import cv2
import numpy as np
import pyclipper
from shapely.geometry import Polygon

_MEAN = np.array([0.485, 0.456, 0.406], dtype=np.float32)
_STD = np.array([0.229, 0.224, 0.225], dtype=np.float32)


def _resize(img: np.ndarray, limit_side_len: int = 960) -> tuple[np.ndarray, float, float]:
    h, w = img.shape[:2]
    ratio = 1.0
    if max(h, w) > limit_side_len:
        ratio = limit_side_len / float(max(h, w))
    rh, rw = int(h * ratio), int(w * ratio)
    rh = max(int(round(rh / 32) * 32), 32)
    rw = max(int(round(rw / 32) * 32), 32)
    out = cv2.resize(img, (rw, rh))
    return out, rh / float(h), rw / float(w)


def _box_score_fast(bitmap: np.ndarray, box: np.ndarray) -> float:
    h, w = bitmap.shape[:2]
    xmin = int(np.clip(np.floor(box[:, 0].min()), 0, w - 1))
    xmax = int(np.clip(np.ceil(box[:, 0].max()), 0, w - 1))
    ymin = int(np.clip(np.floor(box[:, 1].min()), 0, h - 1))
    ymax = int(np.clip(np.ceil(box[:, 1].max()), 0, h - 1))
    mask = np.zeros((ymax - ymin + 1, xmax - xmin + 1), dtype=np.uint8)
    pts = box.copy()
    pts[:, 0] -= xmin
    pts[:, 1] -= ymin
    cv2.fillPoly(mask, pts.reshape(1, -1, 2).astype(np.int32), 1)
    return float(cv2.mean(bitmap[ymin:ymax + 1, xmin:xmax + 1], mask)[0])


def _unclip(box: np.ndarray, ratio: float) -> np.ndarray | None:
    poly = Polygon(box)
    if poly.length <= 0:
        return None
    distance = poly.area * ratio / poly.length
    offset = pyclipper.PyclipperOffset()
    offset.AddPath(box.tolist(), pyclipper.JT_ROUND, pyclipper.ET_CLOSEDPOLYGON)
    expanded = offset.Execute(distance)
    if not expanded:
        return None
    return np.array(expanded[0])


def _min_area_rect(contour: np.ndarray) -> tuple[np.ndarray, float]:
    rect = cv2.minAreaRect(contour)
    points = sorted(cv2.boxPoints(rect), key=lambda p: p[0])
    i1, i2 = (0, 1) if points[1][1] > points[0][1] else (1, 0)
    i3, i4 = (2, 3) if points[3][1] > points[2][1] else (3, 2)
    box = np.array([points[i1], points[i3], points[i4], points[i2]], dtype=np.float32)
    return box, min(rect[1])


def _boxes_from_bitmap(pred: np.ndarray, bitmap: np.ndarray, dest_w: int, dest_h: int, *, box_thresh: float,
                       max_candidates: int, unclip_ratio: float, min_size: int = 3) -> list[np.ndarray]:
    h, w = bitmap.shape
    contours, _ = cv2.findContours((bitmap * 255).astype(np.uint8), cv2.RETR_LIST, cv2.CHAIN_APPROX_SIMPLE)
    boxes: list[np.ndarray] = []
    for contour in contours[:max_candidates]:
        points, sside = _min_area_rect(contour)
        if sside < min_size:
            continue
        if _box_score_fast(pred, points.reshape(-1, 2)) < box_thresh:
            continue
        box = _unclip(points, unclip_ratio)
        if box is None:
            continue
        box, sside = _min_area_rect(box.reshape(-1, 1, 2).astype(np.int32))
        if sside < min_size + 2:
            continue
        box[:, 0] = np.clip(np.round(box[:, 0] / w * dest_w), 0, dest_w)
        box[:, 1] = np.clip(np.round(box[:, 1] / h * dest_h), 0, dest_h)
        boxes.append(box.astype(np.int16))
    return boxes


def _filter_and_sort(boxes: list[np.ndarray], h: int, w: int) -> list[list[list[int]]]:
    out = []
    for box in boxes:
        box = np.array(box, dtype=np.float32)
        box[:, 0] = np.clip(box[:, 0], 0, w - 1)
        box[:, 1] = np.clip(box[:, 1], 0, h - 1)
        rw = int(np.linalg.norm(box[0] - box[1]))
        rh = int(np.linalg.norm(box[0] - box[3]))
        if rw <= 3 or rh <= 3:
            continue
        out.append(box)
    # sorted_boxes của PaddleOCR: theo y rồi x, đổi chỗ cặp kề nhau cùng dòng (|Δy| < 10).
    out.sort(key=lambda b: (b[0][1], b[0][0]))
    for i in range(len(out) - 1):
        for j in range(i, -1, -1):
            if abs(out[j + 1][0][1] - out[j][0][1]) < 10 and out[j + 1][0][0] < out[j][0][0]:
                out[j], out[j + 1] = out[j + 1], out[j]
            else:
                break
    return [b.astype(int).tolist() for b in out]


class OnnxDetector:
    """Giao diện tối thiểu giống PaddleOCR 2.x: .ocr(img, det=True, rec=False, cls=False) → [[quad, ...]]."""

    def __init__(self, path: str | None = None, use_gpu: bool = False, *, thresh: float = 0.3, box_thresh: float = 0.6,
                 unclip_ratio: float = 1.5, max_candidates: int = 1000, limit_side_len: int = 960) -> None:
        import onnxruntime as ort

        self.path = path or os.getenv("OCR_DET_ONNX", "/models/det/en_PP-OCRv3_det.onnx")
        if not os.path.exists(self.path):
            raise FileNotFoundError(f"thiếu model DBNet ONNX: {self.path}")
        providers = ["CPUExecutionProvider"]
        if use_gpu:
            providers = [("CUDAExecutionProvider", {"device_id": 0, "arena_extend_strategy": "kSameAsRequested",
                                                    "gpu_mem_limit": int(os.getenv("OCR_DET_GPU_MEM_MB", "512")) * 1024 * 1024})] + providers
        so = ort.SessionOptions()
        so.intra_op_num_threads = int(os.getenv("OCR_DET_ONNX_THREADS", "2"))
        self.sess = ort.InferenceSession(self.path, so, providers=providers)
        self.provider = self.sess.get_providers()[0]
        if use_gpu and not self.provider.startswith("CUDA"):
            # Không im lặng rơi về CPU (cùng lý do như VietOCR): báo rõ để health thấy.
            raise RuntimeError(f"OCR_USE_GPU=true nhưng onnxruntime chỉ có {self.sess.get_providers()} — cần onnxruntime-gpu + CUDA/cuDNN khớp")
        self.input = self.sess.get_inputs()[0].name
        self.thresh, self.box_thresh, self.unclip_ratio = thresh, box_thresh, unclip_ratio
        self.max_candidates, self.limit_side_len = max_candidates, limit_side_len

    def ocr(self, img_bgr: np.ndarray, det: bool = True, rec: bool = False, cls: bool = False):
        if not det or rec:
            raise NotImplementedError("OnnxDetector chỉ phát hiện vùng chữ (det=True, rec=False)")
        h, w = img_bgr.shape[:2]
        resized, _, _ = _resize(img_bgr, self.limit_side_len)
        x = (resized.astype(np.float32) / 255.0 - _MEAN) / _STD
        x = x.transpose(2, 0, 1)[None]
        pred = self.sess.run(None, {self.input: x})[0][0, 0]
        bitmap = pred > self.thresh
        boxes = _boxes_from_bitmap(pred, bitmap, w, h, box_thresh=self.box_thresh, max_candidates=self.max_candidates,
                                   unclip_ratio=self.unclip_ratio)
        quads = _filter_and_sort(boxes, h, w)
        return [quads] if quads else [[]]
