#!/usr/bin/env python3
"""Dựng cặp huấn luyện VietOCR từ CHÍNH tài liệu trong DigiText, crop ĐÃ NỚI KHUNG.

Khác build_rec_dataset.py ở hai điểm, cả hai rút ra từ đo đạc 17/9/2026:
- Crop nới khung đúng như dịch vụ :5002 làm lúc chạy (OCR_CROP_PAD_Y/X). Model
  ft_v1 học trên crop CẮT SÁT nên khi phục vụ với khung nới nó đọc "." thành ",",
  rơi "/." cuối câu — lệch phân phối giữa train và chạy thật.
- Nguồn là thân văn bản của loại tài liệu đang số hoá, không phải trang tiêu đề
  của bộ khác.

Nhãn do VLM đọc crop → vẫn là nhãn máy. Lọc: bỏ dòng VLM dài/ngắn bất thường so
với OCR (VLM hay bịa phần đuôi), bỏ dòng lệch > max-cer, bỏ ký tự ngoài vocab.
Tài liệu trong --holdout KHÔNG được dùng (để dành đo).

Chạy trên máy có CSDL + MinIO của DigiText (docker) và tới được :5002, :8003.
"""
from __future__ import annotations

import argparse
import base64
import concurrent.futures as cf
import csv
import io
import json
import os
import random
import re
import subprocess
import sys
import unicodedata
import urllib.request

import fitz  # PyMuPDF
from PIL import Image

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "bench", "harness"))
from metrics import levenshtein, normalize_vi  # noqa: E402

PSQL = ["docker", "exec", "-i", "digitext-dev-postgres-1", "psql", "-U", "digitext", "-d", "digitext", "--csv", "-c"]


def q(sql):
    r = subprocess.run(PSQL + [sql], capture_output=True, text=True, encoding="utf-8")
    if r.returncode:
        raise SystemExit(r.stderr)
    return list(csv.DictReader(io.StringIO(r.stdout)))


def minio_get(key):
    return subprocess.run(["docker", "exec", "digitext-dev-minio-1", "mc", "cat", f"local/pages/{key}"],
                          capture_output=True).stdout


def N(s):
    return re.sub(r"\s+", " ", normalize_vi(unicodedata.normalize("NFC", s or ""))).strip()


def post(url, body, headers, timeout):
    req = urllib.request.Request(url, data=json.dumps(body).encode(),
                                 headers={"Content-Type": "application/json", **headers})
    return json.load(urllib.request.urlopen(req, timeout=timeout))


def vlm_read(args, im: Image.Image) -> str:
    w, h = im.size
    s = max(1.0, 64 / min(w, h))
    if s > 1:
        im = im.resize((int(w * s), int(h * s)), Image.LANCZOS)
    if max(im.size) > 1400:
        k = 1400 / max(im.size)
        im = im.resize((int(im.width * k), int(im.height * k)), Image.LANCZOS)
    buf = io.BytesIO(); im.save(buf, "PNG")
    body = {"model": args.vlm_model, "temperature": 0.0, "max_tokens": 200, "messages": [{"role": "user", "content": [
        {"type": "image_url", "image_url": {"url": "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode()}},
        {"type": "text", "text": "Chép lại CHÍNH XÁC dòng chữ chính giữa ảnh (tiếng Việt có dấu, giữ nguyên dấu câu). "
                                 "Bỏ qua mép chữ của dòng trên/dưới nếu có. Chỉ trả về đúng dòng chữ."}]}]}
    r = post(args.vlm_endpoint, body, {}, 120)
    return r["choices"][0]["message"]["content"].strip().strip('"')


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--out", required=True)
    ap.add_argument("--holdout", required=True, help="jsonl có khoá 'doc' — các tài liệu để dành đo")
    ap.add_argument("--ocr-endpoint", default="http://192.168.0.205:5002")
    ap.add_argument("--ocr-key", default=os.environ.get("OCR_API_KEY", ""))
    ap.add_argument("--vlm-endpoint", default="http://192.168.0.205:8003/v1/chat/completions")
    ap.add_argument("--vlm-model", default="Qwen2.5-VL-7B-Instruct-AWQ")
    ap.add_argument("--config", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..",
                                                     "ocr", "vietocr_config", "vgg_transformer.yml"))
    ap.add_argument("--pad-y", type=float, default=0.12)
    ap.add_argument("--pad-x", type=float, default=0.15)
    ap.add_argument("--min-chars", type=int, default=4)
    ap.add_argument("--max-cer", type=float, default=0.30)
    ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--val-ratio", type=float, default=0.05)
    ap.add_argument("--limit-docs", type=int, default=0)
    a = ap.parse_args(argv)

    try:
        import yaml
        vocab = set(yaml.safe_load(open(a.config, encoding="utf-8"))["vocab"])
    except ModuleNotFoundError:
        # Không có PyYAML: vocab là một dòng `vocab: '…'` (nháy đơn, '' = một dấu nháy).
        m = re.search(r"^vocab:\s*'(.*)'\s*$", open(a.config, encoding="utf-8").read(), re.M)
        if not m:
            raise SystemExit(f"❌ không đọc được vocab từ {a.config} (cài PyYAML?)")
        vocab = set(m.group(1).replace("''", "'"))
    hold = {json.loads(l)["doc"] for l in open(a.holdout, encoding="utf-8") if l.strip()}
    docs = [d for d in q("select id::text id, filename, pdfa_key, page_count from documents "
                         "where pdfa_key is not null order by filename") if d["filename"] not in hold]
    if a.limit_docs:
        docs = docs[:a.limit_docs]
    os.makedirs(os.path.join(a.out, "crops"), exist_ok=True)
    print(f"{len(docs)} tài liệu huấn luyện · {len(hold)} tài liệu để dành đo", flush=True)

    items = []  # (rel, crop, ocr_text)
    for d in docs:
        pdf = fitz.open(stream=minio_get(d["pdfa_key"]), filetype="pdf")
        stem = d["filename"].rsplit(".", 2)[-2] if d["filename"].count(".") >= 2 else d["filename"]
        for pno in range(len(pdf)):
            imgs = pdf[pno].get_images(full=True)
            if not imgs:
                continue
            im = Image.open(io.BytesIO(pdf.extract_image(imgs[0][0])["image"])).convert("RGB")
            buf = io.BytesIO(); im.save(buf, "PNG")
            try:
                r = post(a.ocr_endpoint.rstrip("/") + "/api/ocr/tile",
                         {"image": base64.b64encode(buf.getvalue()).decode()}, {"X-API-Key": a.ocr_key}, 180)
            except Exception as e:  # noqa: BLE001
                print(f"  OCR lỗi {d['filename']} p{pno+1}: {e}", file=sys.stderr); continue
            for i, (t, b) in enumerate(zip(r["texts"], r["boxes"])):
                if len(t.strip()) < a.min_chars:
                    continue
                x1, y1, x2, y2 = b; h = y2 - y1
                crop = im.crop((max(0, int(x1 - a.pad_x * h)), max(0, int(y1 - a.pad_y * h)),
                                min(im.width, int(x2 + a.pad_x * h)), min(im.height, int(y2 + a.pad_y * h))))
                if crop.width < 8 or crop.height < 8:
                    continue
                items.append((f"crops/{stem}_p{pno+1:02d}_{i:03d}.png", crop, t))
        print(f"  {d['filename']}: {len(items)} dòng", flush=True)

    def label(it):
        rel, crop, ocr_t = it
        try:
            return rel, crop, ocr_t, vlm_read(a, crop)
        except Exception as e:  # noqa: BLE001
            return rel, crop, ocr_t, None

    def upper_ratio(s: str) -> float:
        letters = [c for c in s if c.isalpha()]
        return sum(c.isupper() for c in letters) / max(len(letters), 1)

    def _skeleton(s: str) -> str:
        s = unicodedata.normalize("NFD", s.lower()).replace("đ", "d")
        return "".join(c for c in s if c.isalnum() and not unicodedata.combining(c))

    why = {"vlm_err": 0, "vocab": 0, "len_ratio": 0, "case": 0, "lead": 0, "anchor": 0, "cer": 0}
    kept, raw = [], []
    with cf.ThreadPoolExecutor(max_workers=a.workers) as ex:
        for n, (rel, crop, ocr_t, vl) in enumerate(ex.map(label, items), 1):
            if n % 500 == 0:
                print(f"  VLM {n}/{len(items)} · giữ {len(kept)}", flush=True)
            if vl is None:
                why["vlm_err"] += 1; continue
            o, v = N(ocr_t), N(vl)
            raw.append({"path": rel, "ocr": o, "vlm": v})
            if not v or any(c not in vocab for c in v):
                why["vocab"] += 1; continue
            if not (0.85 * len(o) - 2 <= len(v) <= 1.12 * len(o) + 2):
                why["len_ratio"] += 1; continue
            # VLM hay "làm đẹp": đổi DÒNG CHỮ HOA thành chữ thường, bỏ "- " hay dấu nháy
            # đầu dòng. Học theo là hỏng lớp chữ → bỏ các nhãn đó.
            if abs(upper_ratio(o) - upper_ratio(v)) > 0.2:
                why["case"] += 1; continue
            if o[:1] != v[:1] and not (o[:1].isalnum() and v[:1].isalnum()):
                why["lead"] += 1; continue
            # NEO ĐẦU/CUỐI. VLM hay bỏ từ đầu dòng ("Căn cứ ", "11. ", "Điều 3. ") mà
            # vẫn lọt lọc độ dài. Lần huấn luyện v2 (17/9/2026) không có neo này: model
            # học bỏ đầu dòng, đồng thuận tụt 97,84% → 96,00%. So 3 ký tự chữ-số đầu và
            # cuối sau khi bỏ dấu — VietOCR hiếm khi sai cả hai đầu, VLM cắt thì lệch ngay.
            so, sv = _skeleton(o), _skeleton(v)
            if len(so) < 3 or len(sv) < 3 or so[:3] != sv[:3] or so[-3:] != sv[-3:]:
                why["anchor"] += 1; continue
            if levenshtein(o, v) / max(len(v), 1) > a.max_cer:
                why["cer"] += 1; continue
            crop.save(os.path.join(a.out, rel))
            kept.append((rel, v, o))

    with open(os.path.join(a.out, "pairs_raw.jsonl"), "w", encoding="utf-8") as f:
        for r in raw:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    random.seed(42)
    by_doc = sorted({k[0].split("_p")[0] for k in kept})
    val_docs = set(random.sample(by_doc, max(1, int(len(by_doc) * a.val_ratio))))
    tr = [k for k in kept if k[0].split("_p")[0] not in val_docs]
    va = [k for k in kept if k[0].split("_p")[0] in val_docs]
    for name, rows in (("train.txt", tr), ("val.txt", va)):
        with open(os.path.join(a.out, name), "w", encoding="utf-8", newline="\n") as f:
            f.write("\n".join(f"{p}\t{t}" for p, t, _ in rows) + "\n")
    changed = sum(1 for _, v, o in kept if v != o)
    print(f"✅ {a.out}: giữ {len(kept)}/{len(items)} (train {len(tr)}, val {len(va)}) · bỏ {why}")
    print(f"   trong số giữ: {changed} dòng nhãn KHÁC kết quả OCR hiện tại ({100*changed/max(len(kept),1):.1f}%) — đó là phần model sẽ học")
    return 0


if __name__ == "__main__":
    sys.exit(main())
