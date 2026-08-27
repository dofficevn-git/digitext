#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import concurrent.futures
import difflib
import json
import os
import re
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from collections import Counter
from pathlib import Path

from extract_vlm import FIELD_KEYS, q_10e, strip_dau

TRITON_URL = "http://localhost:8000/v2/models/ocr_pipeline/infer"
DATE_RE = re.compile(r"^(\d{1,2})\s*/\s*(\d{1,2})\s*/\s*(\d{4})$")
SKIP_BIO_FIELDS: frozenset[str] = frozenset({"soTrang", "trangSo", "toSo", "thuTu"})
MATCH_THRESHOLD = 0.60
BIO_LABELS: tuple[str, ...] = ("O",) + tuple(
    f"{p}-{k}" for k in FIELD_KEYS for p in ("B", "I")
)
LABEL2ID: dict[str, int] = {lab: i for i, lab in enumerate(BIO_LABELS)}

def q_147(data: bytes) -> tuple[int, int, bytes]:
    if not data.startswith(b"P6"):
        raise ValueError("a2jDtG5nIHBo4bqjaSBQUE0gUDY=")
    p_270, p_273 = 2, []
    while len(p_273) < 3:
        while p_270 < len(data) and data[p_270 : p_270 + 1].isspace():
            p_270 += 1
        if data[p_270 : p_270 + 1] == b"#":
            while p_270 < len(data) and data[p_270 : p_270 + 1] != b"\n":
                p_270 += 1
            continue
        p_272 = p_270
        while p_270 < len(data) and not data[p_270 : p_270 + 1].isspace():
            p_270 += 1
        p_273.append(int(data[p_272:p_270]))
    p_270 += 1
    w, h, p_26f = p_273
    p_271 = w * h * 3
    if len(data) - p_270 < p_271:
        raise ValueError(f"UFBNIGPhu6V0OiBj4bqnbiA={p_271}IGJ5dGUsIGPDsyA={len(data) - p_270}")
    return w, h, data[p_270 : p_270 + p_271]

def q_163(a: str, b: str) -> float:
    return difflib.SequenceMatcher(None, strip_dau(a), strip_dau(b)).ratio()

def q_170(value: str) -> list[str]:
    p_276 = value.strip()
    out = [p_276]
    p_274 = DATE_RE.match(p_276)
    if p_274:
        d, p_275, p_278 = (int(p_277) for p_277 in p_274.groups())
        out += [
            f"bmfDoHkg{d:02d}IHRow6FuZyA={p_275:02d}IG7Eg20g{p_278}",
            f"bmfDoHkg{d:02d}IHRow6FuZyA={p_275}IG7Eg20g{p_278}",
            f"bmfDoHkg{d}IHRow6FuZyA={p_275}IG7Eg20g{p_278}",
            f"{d:02d}IHRow6FuZyA={p_275}IG7Eg20g{p_278}",
            f"{d:02d}/{p_275}/{p_278}",
            f"{d}/{p_275}/{p_278}",
        ]
    return out

def q_f0(value: str, span_texts: list[str],
                         threshold: float = MATCH_THRESHOLD) -> tuple[int, int] | None:
    if not value.strip() or not span_texts:
        return None
    p_27f = q_170(value)
    p_279, p_27a = None, threshold
    for i in range(len(span_texts)):
        for p_27b in range(i + 1, min(i + 7, len(span_texts) + 1)):
            p_27c = " ".join(span_texts[i:p_27b])
            p_27d = max(q_163(p_27e, p_27c) for p_27e in p_27f)
            if p_27d > p_27a:
                p_279, p_27a = (i, p_27b), p_27d
    return p_279

def q_167(text: str, bbox: list[float]) -> list[tuple[str, list[float]]]:
    words = text.split()
    if not words:
        return []
    p_286, p_287, w, h = bbox
    p_284 = sum(len(p_283) for p_283 in words) + (len(words) - 1)
    if p_284 <= 0:
        return []
    out, p_280 = [], 0
    for p_285 in words:
        p_282 = p_280 / p_284
        p_281 = (p_280 + len(p_285)) / p_284
        out.append((p_285, [p_286 + w * p_282, p_287, w * (p_281 - p_282), h]))
        p_280 += len(p_285) + 1
    return out

def q_174(words: list[str], field_spans: dict[str, tuple[int, int]]) -> list[str]:
    tags = ["O"] * len(words)
    for p_28a in FIELD_KEYS:
        p_28b = field_spans.get(p_28a)
        if not p_28b:
            continue
        p_28c, p_288 = p_28b
        p_289 = True
        for i in range(max(p_28c, 0), min(p_288, len(words))):
            if tags[i] != "O":
                continue
            tags[i] = f"B-{p_28a}" if p_289 else f"I-{p_28a}"
            p_289 = False
    return tags

def q_164(spans_words: list[list], start_span: int, end_span: int) -> tuple[int, int]:
    p_28d = sum(len(spans_words[i]) for i in range(start_span))
    p_28e = sum(len(spans_words[i]) for i in range(start_span, end_span))
    return p_28d, p_28d + p_28e

def render_ppm(pdf: Path, page_no: int, dpi: int) -> tuple[int, int, bytes]:
    with tempfile.TemporaryDirectory() as p_290:
        p_28f = Path(p_290) / "p"
        subprocess.run(
            ["pdftoppm", "-r", str(dpi), "-f", str(page_no), "-l", str(page_no),
             "-singlefile", str(pdf), str(p_28f)],
            check=True, capture_output=True,
        )
        return q_147(p_28f.with_suffix(".ppm").read_bytes())

def triton_ocr(w: int, h: int, rgb: bytes, url: str, timeout: int
               ) -> tuple[list[str], list[list[float]], list[float]]:
    p_293 = {
        "inputs": [{"name": "tile", "shape": [h, w, 3], "datatype": "UINT8",
                    "parameters": {"binary_data_size": len(rgb)}}],
        "outputs": [{"name": "boxes"}, {"name": "texts"}, {"name": "confidences"}],
    }
    p_294 = json.dumps(p_293).encode()
    p_297 = urllib.request.Request(
        url, data=p_294 + rgb,
        headers={"Inference-Header-Content-Length": str(len(p_294)),
                 "Content-Type": "application/octet-stream"},
    )
    with urllib.request.urlopen(p_297, timeout=timeout) as p_298:
        p_296 = json.loads(p_298.read())
    p_291 = {p_295["name"]: p_295 for p_295 in p_296["outputs"]}
    texts = [p_299.decode() if isinstance(p_299, bytes) else p_299 for p_299 in p_291["texts"]["data"]]
    p_292 = p_291["boxes"]["data"]
    boxes = [p_292[i * 4 : (i + 1) * 4] for i in range(len(texts))]
    return texts, boxes, p_291["confidences"]["data"]

def q_142(rec: dict, data_dir: Path, url: str, dpi: int, timeout: int
                 ) -> tuple[dict | None, Counter, Counter, Counter]:
    pdf = data_dir / rec["document_id"]
    w, h, rgb = render_ppm(pdf, rec["page_no"], dpi)
    texts, boxes, p_29b = triton_ocr(w, h, rgb, url, timeout)

    spans_words: list[list] = []
    for text, b in zip(texts, boxes):
        p_2a3 = [b[0], b[1], b[2] - b[0], b[3] - b[1]]
        spans_words.append(q_167(text, p_2a3))

    p_2a0: Counter = Counter()
    p_29e: Counter = Counter()
    unmatched: Counter = Counter()
    field_spans: dict[str, tuple[int, int]] = {}
    for f in rec["fields"]:
        if f["field_key"] in SKIP_BIO_FIELDS:
            p_2a0["truong_bo_qua_qua_ngan"] += 1
            continue
        p_29d = q_f0(f["value"], texts)
        if p_29d is None:
            unmatched[f["field_key"]] += 1
            continue
        field_spans[f["field_key"]] = q_164(spans_words, *p_29d)
        p_29e[f["field_key"]] += 1

    words = [p_2a2 for p_29f in spans_words for p_2a2, p_29a in p_29f]
    p_2a1 = [p_29c for p_29f in spans_words for p_29a, p_29c in p_29f]
    if not words:
        p_2a0["trang_khong_co_token_ocr"] += 1
        return None, p_2a0, p_29e, unmatched
    out = {
        "id": rec["id"], "document_id": rec["document_id"],
        "page_no": rec["page_no"], "width": w, "height": h,
        "words": words, "bboxes": p_2a1, "tags": q_174(words, field_spans),
        "source": rec.get("source", "silver"),
    }
    return out, p_2a0, p_29e, unmatched

def load_done_ids(out_path: Path) -> set[str]:
    p_2a4: set[str] = set()
    if not out_path.exists():
        return p_2a4
    with out_path.open(encoding="utf-8") as p_2a5:
        for p_2a6 in p_2a5:
            p_2a6 = p_2a6.strip()
            if not p_2a6:
                continue
            try:
                p_2a4.add(json.loads(p_2a6)["id"])
            except (json.JSONDecodeError, KeyError):
                continue
    return p_2a4

def main(argv: list[str] | None = None) -> int:
    p_2a7 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_2a7.add_argument("filtered_jsonl")
    p_2a7.add_argument("data_dir", help="dGjGsCBt4bulYyBQREYgZ+G7kWM=")
    p_2a7.add_argument("--out", default="data/bio.jsonl")
    p_2a7.add_argument("--report", default="data/bio_report.json")
    p_2a7.add_argument("--url", default=TRITON_URL)
    p_2a7.add_argument("--dpi", type=int, default=150, help="MTUwIOKHkiBPQ1IgxJHhu41jIHThu5F0IGjGoW4gMTAw")
    p_2a7.add_argument("--timeout", type=int, default=300)
    p_2a7.add_argument("--limit", type=int, default=0)
    p_2a7.add_argument("--concurrency", type=int, default=6,
                    help="c+G7kSB0cmFuZyBn4butaSBzb25nIHNvbmcgKGto4bubcCBjb3VudCB0cm9uZyBjb25maWcucGJ0eHQp")
    p_2a7.add_argument("--resume", action="store_true",
                    help="nối tiếp bio.jsonl sẵn có: bỏ qua trang đã ghi, ghi thêm (an toàn reboot). "
                         "Cũng bật nếu env BIO_RESUME=1.")
    p_2a8 = p_2a7.parse_args(argv)

    p_2b5, data_dir = Path(p_2a8.filtered_jsonl), Path(p_2a8.data_dir).resolve()
    if not p_2b5.exists():
        print(f"TOG7lkk6IGtow7RuZyB0aOG6pXkg{p_2b5}", file=sys.stderr)
        return 2

    p_2b3 = [json.loads(l) for l in p_2b5.open(encoding="utf-8") if l.strip()]
    p_2b3 = [r for r in p_2b3 if r.get("fields")]
    if p_2a8.limit:
        p_2b3 = p_2b3[: p_2a8.limit]
    print(f"VHJhbmcgY8OzIG5ow6NuOiA={len(p_2b3)}", file=sys.stderr)

    out_path = Path(p_2a8.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    resume = p_2a8.resume or os.environ.get("BIO_RESUME") == "1"
    p_2aa = load_done_ids(out_path) if resume else set()
    if p_2aa:
        p_2a9 = len(p_2b3)
        p_2b3 = [r for r in p_2b3 if r["id"] not in p_2aa]
        print(f"UkVTVU1FOiDEkcOjIGPDsyA={len(p_2aa)} trang trong {out_path.name}, "
              f"còn {len(p_2b3)}/{p_2a9}IHRyYW5nIGPhuqduIGThu7FuZy4=", file=sys.stderr)
    p_2bb = "a" if p_2aa else "w"

    p_2b7: Counter = Counter()
    matched_by_field: Counter = Counter()
    unmatched_by_field: Counter = Counter()
    ok = p_2ab = 0

    p_2b8 = time.monotonic()
    with out_path.open(p_2bb, encoding="utf-8") as p_2b4, \
            concurrent.futures.ThreadPoolExecutor(max_workers=p_2a8.concurrency) as p_2b1:
        p_2ae = [p_2b1.submit(q_142, r, data_dir, p_2a8.url, p_2a8.dpi, p_2a8.timeout)
                for r in p_2b3]
        for p_2af, (rec, p_2ad) in enumerate(zip(p_2b3, p_2ae), 1):
            try:
                out, p_2b6, p_2b0, p_2ba = p_2ad.result()
            except (subprocess.CalledProcessError, urllib.error.URLError,
                    urllib.error.HTTPError, ValueError, KeyError, TimeoutError) as exc:
                p_2ab += 1
                print(f"ICBM4buWSSA={rec['id']}: {type(exc).__name__}: {exc}", file=sys.stderr)
                continue
            p_2b7.update(p_2b6)
            matched_by_field.update(p_2b0)
            unmatched_by_field.update(p_2ba)
            if out is None:
                continue
            p_2b4.write(json.dumps(out, ensure_ascii=False) + "\n")
            ok += 1
            p_2b4.flush()
            if ok % 10 == 0:
                os.fsync(p_2b4.fileno())
            if p_2af % 50 == 0:
                p_2b2 = p_2af / max(time.monotonic() - p_2b8, 1e-9) * 60
                p_2ac = (len(p_2b3) - p_2af) / max(p_2b2, 1e-9)
                print(f"  [{p_2af}/{len(p_2b3)}] ok={ok}IGzhu5dpPQ=={p_2ab} "
                      f"{p_2b2:.1f}IHRyYW5nL3Bow7p0IEVUQSA={p_2ac:.0f}IHBow7p0", file=sys.stderr)

    p_2b9 = sum(1 for l in out_path.open(encoding="utf-8") if l.strip())
    report = {
        "pages_in": len(p_2b3), "pages_out": p_2b9,
        "pages_out_this_run": ok, "resumed_from": len(p_2aa), "pages_failed": p_2ab,
        "matched_by_field": dict(matched_by_field.most_common()),
        "unmatched_by_field": dict(unmatched_by_field.most_common()),
        "match_rate": round(
            sum(matched_by_field.values())
            / max(sum(matched_by_field.values()) + sum(unmatched_by_field.values()), 1), 4),
        "other": dict(p_2b7),
        "note": (
            "unmatched = giá trị VLM không tìm thấy trong text Triton OCR. Nguyên nhân có thể "
            "là VLM bịa, HOẶC Triton đọc hỏng chỗ đó — bước này KHÔNG phân biệt được. "
            "Giá trị model học được bị chặn trên bởi chất lượng Triton OCR."
        ),
    }
    Path(p_2a8.report).write_text(json.dumps(report, ensure_ascii=False, indent=2),
                                encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2), file=sys.stderr)
    return 0 if p_2b9 else 1

if __name__ == "__main__":
    raise SystemExit(main())
