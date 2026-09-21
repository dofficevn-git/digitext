# Copyright (c) 2026 DigiText. All rights reserved.
"""LƯỚI AN TOÀN phát hiện dòng theo NÉT MỰC — không dùng mô hình.

DBNet (PaddleOCR) học trên chữ in: gặp chữ viết tay bút bi nét mảnh nó chỉ bắt từng mẩu
của dòng, có dòng mất hẳn (đo 21/9/2026 trên trang "Bảng tổng hợp ý kiến": chỉ 80% nét mực
nằm trong khung; chỉnh ngưỡng/độ phân giải DBNet chỉ sinh khung vụn, vẫn sót). Chữ đã
không có khung thì không bao giờ vào được lớp text PDF/A, VLM đọc đúng cũng không có chỗ đặt.

Cách làm (sau khi VietOCR đã đọc các khung DBNet):
  1. Mặt nạ mực = điểm tối HOẶC mực xanh, trừ mực đỏ (dấu), trừ chấm dòng kẻ, trừ đường kẻ dài.
  2. Khung DBNet đọc TIN CẬY CAO (chữ in) được tin: xoá mực của chúng khỏi mặt nạ.
  3. Mực còn lại → nối ngang thành dòng; cụm cao (dòng tay dính nhau qua nét móc) được cắt
     theo thung lũng của biểu đồ chiếu ngang. Mỗi dòng thành một khung mới.
  4. Khung DBNet tin cậy THẤP đã bị khung mới phủ thì bỏ (chúng là mẩu vụn của dòng tay).
  5. Đo ĐỘ PHỦ MỰC = % nét mực nằm trong một khung nào đó — thước đo "đã bắt hết chưa".

Ưu tiên của lớp này là KHÔNG SÓT; nhận nhầm vệt bút/chữ ký thành dòng là chấp nhận được.
Tắt: OCR_INK_LINES=false.
"""
from __future__ import annotations

import os

import cv2
import numpy as np


def _env_f(name: str, default: float) -> float:
    try:
        return float(os.getenv(name, "") or default)
    except ValueError:
        return default


def enabled() -> bool:
    return os.getenv("OCR_INK_LINES", "true").strip().lower() in ("1", "true", "yes")


def describe() -> dict:
    return {"enabled": enabled(), "trust_conf": _env_f("OCR_INK_TRUST_CONF", 0.85)}


def ink_mask(img_bgr: np.ndarray, med_h: float) -> np.ndarray:
    """Mặt nạ 0/255 của nét mực dùng để TÁCH DÒNG và ĐO ĐỘ PHỦ (đã bỏ chấm nhỏ, đường kẻ)."""
    gray = cv2.cvtColor(img_bgr, cv2.COLOR_BGR2GRAY)
    hsv = cv2.cvtColor(img_bgr, cv2.COLOR_BGR2HSV)
    h, s, v = cv2.split(hsv)
    dark = gray < int(_env_f("OCR_INK_DARK", 140))
    # Ảnh xám/đen trắng: s ≈ 0 ở mọi nơi nên hai điều kiện màu tự tắt, chỉ còn `dark`.
    blue = (h > 95) & (h < 140) & (s > 60) & (v < 200)
    red = ((h < 12) | (h > 165)) & (s > 60)
    ink = ((dark | blue) & ~red).astype(np.uint8) * 255

    # Đường kẻ bảng / dòng kẻ liền: dài và thẳng — chữ không có nét nào như vậy.
    hk = cv2.getStructuringElement(cv2.MORPH_RECT, (max(40, int(4 * med_h)), 1))
    vk = cv2.getStructuringElement(cv2.MORPH_RECT, (1, max(40, int(3 * med_h))))
    rules = cv2.morphologyEx(ink, cv2.MORPH_OPEN, hk) | cv2.morphologyEx(ink, cv2.MORPH_OPEN, vk)
    ink = cv2.bitwise_and(ink, cv2.bitwise_not(cv2.dilate(rules, np.ones((3, 3), np.uint8))))

    # Chấm của dòng kẻ chấm + nhiễu giấy: thành phần nhỏ. (Dấu nặng/dấu chấm i cũng rơi ở
    # đây — không sao, mặt nạ chỉ để tách dòng; khung dòng được đệm lại sau.)
    n, lab, st, _ = cv2.connectedComponentsWithStats(ink, 8)
    min_dim = max(6.0, 0.15 * med_h)
    min_area = max(12.0, 0.004 * med_h * med_h)
    ok = np.zeros(n, dtype=bool)
    ok[1:] = (st[1:, cv2.CC_STAT_AREA] >= min_area) & (
        np.maximum(st[1:, cv2.CC_STAT_WIDTH], st[1:, cv2.CC_STAT_HEIGHT]) >= min_dim)
    return (ok[lab].astype(np.uint8)) * 255


def _split_tall(sub: np.ndarray, pitch: float) -> list[tuple[int, int]]:
    """Cắt một cụm cao thành các dải dòng (y1, y2) theo thung lũng của biểu đồ chiếu ngang."""
    hh = sub.shape[0]
    prof = (sub > 0).sum(axis=1).astype(np.float32)
    k = max(3, int(0.12 * pitch) | 1)
    prof = np.convolve(prof, np.ones(k, np.float32) / k, mode="same")
    n_lines = max(2, int(round(hh / pitch)))
    cuts = []
    for j in range(1, n_lines):
        c = int(j * hh / n_lines)
        lo, hi = max(1, int(c - 0.35 * pitch)), min(hh - 1, int(c + 0.35 * pitch))
        if hi <= lo:
            continue
        cuts.append(lo + int(np.argmin(prof[lo:hi])))
    ys = [0] + sorted(set(cuts)) + [hh]
    return [(a, b) for a, b in zip(ys[:-1], ys[1:]) if b - a >= 0.3 * pitch]


def line_boxes(mask: np.ndarray, med_h: float) -> list[list[int]]:
    """Mặt nạ mực → khung dòng [x1, y1, x2, y2]."""
    H, W = mask.shape[:2]
    kw = max(15, int(1.1 * med_h))
    joined = cv2.morphologyEx(mask, cv2.MORPH_CLOSE, cv2.getStructuringElement(cv2.MORPH_RECT, (kw, 3)))
    # Cắt cầu nối mảnh giữa hai dòng (nét móc g, y chạm dòng dưới).
    ow, oh = max(5, int(0.3 * med_h)), max(3, int(0.1 * med_h))
    opened = cv2.morphologyEx(joined, cv2.MORPH_OPEN, cv2.getStructuringElement(cv2.MORPH_RECT, (ow, oh)))
    n, lab, st, _ = cv2.connectedComponentsWithStats(opened, 8)
    out: list[list[int]] = []
    min_w, min_h = 0.7 * med_h, 0.3 * med_h
    pitch = 1.25 * med_h
    for i in range(1, n):
        x, y, w, h, _area = (int(v) for v in st[i])
        if w < min_w or h < min_h:
            continue
        comp = (lab[y:y + h, x:x + w] == i)
        ink_here = (mask[y:y + h, x:x + w] > 0) & comp
        dens = float(ink_here.sum()) / float(w * h)
        if dens > 0.55 and h > 1.5 * med_h:
            continue  # mảng đặc (ảnh, vết mực loang) — không phải chữ
        if h <= 2.2 * med_h:
            out.append([x, y, x + w, y + h])
            continue
        for a, b in _split_tall(ink_here.astype(np.uint8), pitch):
            cols = np.where(ink_here[a:b].any(axis=0))[0]
            if cols.size == 0 or (cols[-1] - cols[0]) < min_w:
                continue
            out.append([x + int(cols[0]), y + a, x + int(cols[-1]) + 1, y + b])
    # Đệm nhẹ để lấy lại dấu thanh/dấu nặng đã bị lọc khỏi mặt nạ.
    px, py = int(0.12 * med_h), int(0.10 * med_h)
    return [[max(0, x1 - px), max(0, y1 - py), min(W, x2 + px), min(H, y2 + py)] for x1, y1, x2, y2 in out]


def coverage(mask: np.ndarray, boxes) -> float:
    """Tỷ lệ nét mực nằm trong ít nhất một khung. Trang không có mực → 1.0."""
    total = int((mask > 0).sum())
    if total == 0:
        return 1.0
    cov = np.zeros(mask.shape[:2], np.uint8)
    H, W = cov.shape
    for x1, y1, x2, y2 in boxes:
        cov[max(0, int(y1)):min(H, int(y2)), max(0, int(x1)):min(W, int(x2))] = 1
    return float(((mask > 0) & (cov > 0)).sum()) / float(total)


def _inter_frac(a, b) -> float:
    """Phần diện tích của a nằm trong b."""
    ix = max(0.0, min(a[2], b[2]) - max(a[0], b[0]))
    iy = max(0.0, min(a[3], b[3]) - max(a[1], b[1]))
    area = max(1.0, (a[2] - a[0]) * (a[3] - a[1]))
    return ix * iy / area


def _merge_row(boxes: list, med_h: float) -> list:
    """Gộp các mẩu CÙNG HÀNG nằm sát nhau thành một dòng (VLM đọc cả dòng tốt hơn đọc mẩu).
    Khe > 1,5 chiều cao dòng thì coi là hai cột khác nhau, không gộp."""
    boxes = sorted(boxes, key=lambda b: (b[1], b[0]))
    changed = True
    while changed:
        changed = False
        for i in range(len(boxes)):
            for j in range(i + 1, len(boxes)):
                a, b = boxes[i], boxes[j]
                ov = min(a[3], b[3]) - max(a[1], b[1])
                if ov < 0.5 * min(a[3] - a[1], b[3] - b[1]):
                    continue
                gap = max(a[0], b[0]) - min(a[2], b[2])
                if gap > 1.5 * med_h:
                    continue
                if max(a[3], b[3]) - min(a[1], b[1]) > 2.0 * med_h:
                    continue
                boxes[i] = [min(a[0], b[0]), min(a[1], b[1]), max(a[2], b[2]), max(a[3], b[3])]
                del boxes[j]
                changed = True
                break
            if changed:
                break
    return boxes


def complete(img_bgr: np.ndarray, det_boxes: list, det_confs: list) -> dict:
    """det_boxes: khung thẳng trục [x1,y1,x2,y2] của DBNet; det_confs: độ tin cậy VietOCR.

    Trả {"new": [khung mới], "drop": {chỉ số khung DBNet bị thay}, "before", "after": độ phủ}.
    """
    trust = _env_f("OCR_INK_TRUST_CONF", 0.85)
    hs = sorted((b[3] - b[1]) for b, c in zip(det_boxes, det_confs) if c >= trust) or \
        sorted((b[3] - b[1]) for b in det_boxes)
    H, W = img_bgr.shape[:2]
    med_h = float(hs[len(hs) // 2]) if hs else max(24.0, H / 45.0)
    med_h = min(max(med_h, 16.0), H / 8.0)

    mask = ink_mask(img_bgr, med_h)
    before = coverage(mask, det_boxes)

    rest = mask.copy()
    for b, c in zip(det_boxes, det_confs):
        if c >= trust:
            rest[max(0, int(b[1])):min(H, int(b[3])), max(0, int(b[0])):min(W, int(b[2]))] = 0
    new = line_boxes(rest, med_h)

    # Khung mới trùng gần khít với một khung DBNet tin cậy THẤP thì không thêm gì — giữ khung
    # DBNet (được nắn nghiêng theo quad).
    kept_new = []
    for nb in new:
        dup = any(c < trust and _inter_frac(nb, b) > 0.9 and _inter_frac(b, nb) > 0.7
                  for b, c in zip(det_boxes, det_confs))
        if not dup:
            kept_new.append(nb)
    kept_new = _merge_row(kept_new, med_h)
    drop = set()
    for i, (b, c) in enumerate(zip(det_boxes, det_confs)):
        if c >= trust:
            continue
        if sum(_inter_frac(b, nb) for nb in kept_new) >= 0.6:
            drop.add(i)
    final = [b for i, b in enumerate(det_boxes) if i not in drop] + kept_new
    return {"new": kept_new, "drop": drop, "before": before, "after": coverage(mask, final),
            "med_h": med_h, "mask": mask}


def reading_order(boxes: list) -> list[int]:
    """Thứ tự đọc: gom hàng theo tâm dọc, trong hàng xếp trái→phải."""
    if not boxes:
        return []
    hs = sorted(b[3] - b[1] for b in boxes)
    tol = 0.5 * max(1.0, hs[len(hs) // 2])
    idx = sorted(range(len(boxes)), key=lambda i: (boxes[i][1] + boxes[i][3]) / 2)
    rows, cur, cy = [], [idx[0]], (boxes[idx[0]][1] + boxes[idx[0]][3]) / 2
    for i in idx[1:]:
        c = (boxes[i][1] + boxes[i][3]) / 2
        if abs(c - cy) <= tol:
            cur.append(i)
        else:
            rows.append(cur)
            cur, cy = [i], c
    rows.append(cur)
    return [i for r in rows for i in sorted(r, key=lambda j: boxes[j][0])]
