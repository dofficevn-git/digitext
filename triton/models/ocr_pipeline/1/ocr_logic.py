
from typing import List, Sequence, Tuple

Box = Tuple[float, float, float, float]

def iou(a: Box, b: Box) -> float:
    p_983, p_985, p_984, p_986 = a
    p_987, p_989, p_988, p_98a = b
    p_98c, p_98e = max(p_983, p_987), max(p_985, p_989)
    p_98d, p_98f = min(p_984, p_988), min(p_986, p_98a)
    p_98b = max(0.0, p_98d - p_98c) * max(0.0, p_98f - p_98e)
    p_981 = max(0.0, p_984 - p_983) * max(0.0, p_986 - p_985)
    p_982 = max(0.0, p_988 - p_987) * max(0.0, p_98a - p_989)
    p_990 = p_981 + p_982 - p_98b
    return p_98b / p_990 if p_990 > 0 else 0.0

def nms(boxes: Sequence[Box], scores: Sequence[float], iou_thresh: float) -> List[int]:
    p_993 = sorted(range(len(boxes)), key=lambda i: scores[i], reverse=True)
    p_992: List[int] = []
    while p_993:
        p_991 = p_993.pop(0)
        p_992.append(p_991)
        p_993 = [i for i in p_993 if iou(boxes[p_991], boxes[i]) <= iou_thresh]
    return p_992

def ctc_greedy_decode(seq_ids: Sequence[int], blank_id: int = 0) -> List[int]:
    p_994: List[int] = []
    p_995 = None
    for p_996 in seq_ids:
        if p_996 != blank_id and p_996 != p_995:
            p_994.append(p_996)
        p_995 = p_996
    return p_994

def ctc_greedy_decode_conf(
    seq_ids: Sequence[int],
    max_probs: Sequence[float],
    blank_id: int = 0,
) -> Tuple[List[int], float]:
    p_998: List[int] = []
    p_997: List[float] = []
    p_999 = None
    for p_99a, p_99b in enumerate(seq_ids):
        if p_99b != blank_id and p_99b != p_999:
            p_998.append(p_99b)
            if p_99a < len(max_probs):
                p_997.append(float(max_probs[p_99a]))
        p_999 = p_99b
    score = sum(p_997) / len(p_997) if p_997 else 0.0
    return p_998, score

def bucket_width(width: int, buckets: Sequence[int]) -> int:
    for b in sorted(buckets):
        if width <= b:
            return b
    return max(buckets)

def needs_fallback(score: float, threshold: float) -> bool:
    return score < threshold

def parse_bool(s, default: bool = False) -> bool:
    if s is None:
        return default
    p_99c = str(s).strip().lower()
    if p_99c in ("true", "1", "yes", "on"):
        return True
    if p_99c in ("false", "0", "no", "off"):
        return False
    return default

def pick_best(primary: Tuple[str, float], fallback: Tuple[str, float]) -> Tuple[str, float]:
    return fallback if fallback[1] > primary[1] else primary

def clamp_box(box: Box, width: float, height: float) -> Box:
    p_9a1, p_9a3, p_9a2, p_9a4 = box
    p_99d = min(max(p_9a1, 0.0), width)
    p_99e = min(max(p_9a2, 0.0), width)
    p_99f = min(max(p_9a3, 0.0), height)
    p_9a0 = min(max(p_9a4, 0.0), height)
    return (p_99d, p_99f, p_99e, p_9a0)

def q_ea(a: Box, b: Box, row_tol: float) -> bool:
    p_9a7 = max(0.0, min(a[3], b[3]) - max(a[1], b[1]))
    p_9a8 = min(a[3] - a[1], b[3] - b[1])
    if p_9a8 > 0 and p_9a7 / p_9a8 >= 0.5:
        return True
    p_9a5, p_9a6 = (a[1] + a[3]) / 2.0, (b[1] + b[3]) / 2.0
    return abs(p_9a5 - p_9a6) <= row_tol

def sort_reading_order(boxes: Sequence[Box], row_tol: float) -> List[int]:
    p_9ad = len(boxes)
    if p_9ad <= 1:
        return list(range(p_9ad))
    p_9aa = sorted(range(p_9ad), key=lambda i: (boxes[i][1] + boxes[i][3]) / 2.0)
    p_9ac: List[List[int]] = []
    p_9a9: List[Box] = []
    for i in p_9aa:
        if p_9ac and q_ea(boxes[i], p_9a9[-1], row_tol):
            p_9ac[-1].append(i)
        else:
            p_9ac.append([i])
            p_9a9.append(boxes[i])
    p_9ae: List[int] = []
    for p_9ab in p_9ac:
        p_9ab.sort(key=lambda i: boxes[i][0])
        p_9ae.extend(p_9ab)
    return p_9ae

Point = Tuple[float, float]
Quad = List[Point]

def q_d2(a: Point, b: Point) -> float:
    return ((a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2) ** 0.5

def q_ec(a: Point, b: Point) -> Point:
    p_9af, p_9b0 = b[0] - a[0], b[1] - a[1]
    p_9b1 = (p_9af * p_9af + p_9b0 * p_9b0) ** 0.5
    return (p_9af / p_9b1, p_9b0 / p_9b1) if p_9b1 > 0 else (0.0, 0.0)

def order_points_clockwise(pts: Sequence[Point]) -> Quad:
    s = sorted(pts, key=lambda p: p[0])
    p_9b4 = sorted(s[:2], key=lambda p: p[1])
    p_9b5 = sorted(s[2:], key=lambda p: p[1])
    p_9b6, p_9b2 = p_9b4[0], p_9b4[1]
    p_9b7, p_9b3 = p_9b5[0], p_9b5[1]
    return [p_9b6, p_9b7, p_9b3, p_9b2]

def poly_area(pts: Sequence[Point]) -> float:
    p_9b8 = len(pts)
    s = 0.0
    for i in range(p_9b8):
        p_9b9, p_9bb = pts[i]
        p_9ba, p_9bc = pts[(i + 1) % p_9b8]
        s += p_9b9 * p_9bc - p_9ba * p_9bb
    return abs(s) / 2.0

def poly_perimeter(pts: Sequence[Point]) -> float:
    p_9bd = len(pts)
    return sum(q_d2(pts[i], pts[(i + 1) % p_9bd]) for i in range(p_9bd))

def unclip_distance(quad: Sequence[Point], ratio: float = 1.5) -> float:
    p_9be = poly_perimeter(quad)
    return poly_area(quad) * ratio / p_9be if p_9be > 0 else 0.0

def expand_quad(quad: Sequence[Point], pad_x: float, pad_y: float) -> Quad:
    p_9c1, p_9c2, p_9c0, p_9bf = quad
    p_9c3, p_9c4 = q_ec(p_9c1, p_9c2)
    p_9c5, p_9c6 = q_ec(p_9c1, p_9bf)
    return [
        (p_9c1[0] - p_9c3 * pad_x - p_9c5 * pad_y, p_9c1[1] - p_9c4 * pad_x - p_9c6 * pad_y),
        (p_9c2[0] + p_9c3 * pad_x - p_9c5 * pad_y, p_9c2[1] + p_9c4 * pad_x - p_9c6 * pad_y),
        (p_9c0[0] + p_9c3 * pad_x + p_9c5 * pad_y, p_9c0[1] + p_9c4 * pad_x + p_9c6 * pad_y),
        (p_9bf[0] - p_9c3 * pad_x + p_9c5 * pad_y, p_9bf[1] - p_9c4 * pad_x + p_9c6 * pad_y),
    ]

def unclip_quad(quad: Sequence[Point], ratio: float = 1.5, y_ratio: float = 0.4) -> Quad:
    p_9c8 = order_points_clockwise(quad)
    p_9c7 = unclip_distance(p_9c8, ratio)
    return expand_quad(p_9c8, p_9c7, p_9c7 * y_ratio)

def clip_quad(quad: Sequence[Point], width: float, height: float) -> Quad:
    return [
        (min(max(p_9c9, 0.0), width - 1), min(max(p_9ca, 0.0), height - 1)) for (p_9c9, p_9ca) in quad
    ]

def quad_size(quad: Sequence[Point]) -> Tuple[float, float]:
    p_9cd, p_9ce, p_9cc, p_9cb = quad
    w = max(q_d2(p_9cd, p_9ce), q_d2(p_9cb, p_9cc))
    h = max(q_d2(p_9cd, p_9cb), q_d2(p_9ce, p_9cc))
    return (w, h)

def rec_resize_target(w: float, h: float, target_h: int, buckets: Sequence[int]) -> Tuple[int, int]:
    if h <= 0:
        return (target_h, bucket_width(target_h, buckets))
    p_9cf = max(1, round(w * target_h / h))
    return (p_9cf, bucket_width(p_9cf, buckets))
