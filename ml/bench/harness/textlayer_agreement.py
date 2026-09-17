"""Ước lượng độ chính xác lớp chữ trên THÂN văn bản của tài liệu DigiText.

Không có người gõ → dùng Qwen2.5-VL đọc lại từng crop dòng làm tham chiếu, rồi đo
CER của VietOCR (nền và fine-tune) so với Qwen. Qwen cũng sai, nên đây là "độ
đồng thuận", không phải độ chính xác tuyệt đối: CER thật của VietOCR ≤ khoảng
CER này + sai số của Qwen. Dùng để biết KHOẢNG CÁCH tới 98%, không để chứng nhận.

    python textlayer_agreement.py --docs 30 --lines-per-page 8
"""
import argparse, base64, csv, io, json, random, re, subprocess, sys, unicodedata, urllib.request

import fitz  # PyMuPDF
from PIL import Image

PSQL = ["docker", "exec", "-i", "digitext-dev-postgres-1", "psql", "-U", "digitext", "-d", "digitext", "--csv", "-c"]
QWEN = "http://192.168.0.205:8003/v1/chat/completions"
MODEL = "Qwen2.5-VL-7B-Instruct-AWQ"


def q(sql):
    r = subprocess.run(PSQL + [sql], capture_output=True, text=True, encoding="utf-8")
    if r.returncode:
        raise SystemExit(r.stderr)
    csv.field_size_limit(10**9)
    return list(csv.DictReader(io.StringIO(r.stdout)))


def minio_get(key):
    r = subprocess.run(["docker", "exec", "digitext-dev-minio-1", "mc", "cat", f"local/pages/{key}"], capture_output=True)
    if r.returncode:
        raise RuntimeError(r.stderr.decode(errors="replace"))
    return r.stdout


def norm(s):
    s = unicodedata.normalize("NFC", s or "")
    s = s.replace("uỷ", "ủy").replace("Uỷ", "Ủy").replace("UỶ", "ỦY").replace("oà", "òa").replace("Oà", "Òa").replace("OÀ", "ÒA")
    return re.sub(r"\s+", " ", s).strip()


def lev(a, b):
    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        cur = [i]
        for j, cb in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (ca != cb)))
        prev = cur
    return prev[-1]


def qwen_read(im):
    w, h = im.size
    s = max(1.0, 64 / min(w, h))
    if s > 1:
        im = im.resize((int(w * s), int(h * s)), Image.LANCZOS)
    if max(im.size) > 1400:
        k = 1400 / max(im.size)
        im = im.resize((int(im.width * k), int(im.height * k)), Image.LANCZOS)
    buf = io.BytesIO(); im.save(buf, "PNG")
    body = {"model": MODEL, "temperature": 0.0, "max_tokens": 256, "messages": [{"role": "user", "content": [
        {"type": "image_url", "image_url": {"url": "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode()}},
        {"type": "text", "text": "Chép lại CHÍNH XÁC dòng chữ trong ảnh (tiếng Việt có dấu). Chỉ trả về đúng dòng chữ, không giải thích."}]}]}
    req = urllib.request.Request(QWEN, data=json.dumps(body).encode(), headers={"Content-Type": "application/json"})
    r = json.load(urllib.request.urlopen(req, timeout=120))
    return r["choices"][0]["message"]["content"].strip().strip('"')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--docs", type=int, default=30)
    ap.add_argument("--pages-per-doc", type=int, default=2)
    ap.add_argument("--lines-per-page", type=int, default=8)
    ap.add_argument("--min-chars", type=int, default=20)
    ap.add_argument("--base", default="vietocr-vgg-transformer-http")
    ap.add_argument("--ft", default="vietocr-vgg-transformer-ft-v1")
    ap.add_argument("--out", default="textlayer_agreement.jsonl")
    a = ap.parse_args()
    random.seed(20260917)

    docs = q("select id::text id, filename, pdfa_key, page_count from documents where pdfa_key is not null order by filename")
    docs = random.sample(docs, min(a.docs, len(docs)))
    rows = []
    for d in docs:
        pdf = fitz.open(stream=minio_get(d["pdfa_key"]), filetype="pdf")
        res = q(f"""select e.name engine, r.page_no, r.boxes::text boxes from ocr_results r
                    join engine_versions e on e.id=r.engine_version_id
                    where r.document_id='{d['id']}' and r.text_source='ocr_raw' and e.name in ('{a.base}','{a.ft}')""")
        by = {(r["engine"], int(r["page_no"])): json.loads(r["boxes"] or "[]") for r in res}
        pages = list(range(1, min(int(d["page_count"] or 1), len(pdf)) + 1))
        for pno in random.sample(pages, min(a.pages_per_doc, len(pages))):
            ft = by.get((a.ft, pno), []); base = {(round(b["x"]), round(b["y"])): b["t"] for b in by.get((a.base, pno), [])}
            cand = [b for b in ft if len(b["t"]) >= a.min_chars and (round(b["x"]), round(b["y"])) in base]
            if not cand:
                continue
            page = pdf[pno - 1]
            imgs = page.get_images(full=True)
            if not imgs:
                continue
            ex = pdf.extract_image(imgs[0][0])
            im = Image.open(io.BytesIO(ex["image"])).convert("RGB")
            for b in random.sample(cand, min(a.lines_per_page, len(cand))):
                x, y, w, h = b["x"], b["y"], b["w"], b["h"]
                crop = im.crop((max(0, int(x) - 3), max(0, int(y) - 2), min(im.width, int(x + w) + 3), min(im.height, int(y + h) + 2)))
                try:
                    ref = norm(qwen_read(crop))
                except Exception as e:  # noqa: BLE001
                    print("qwen lỗi:", e, file=sys.stderr); continue
                rows.append({"doc": d["filename"], "page": pno, "qwen": ref, "ft": norm(b["t"]),
                             "base": norm(base[(round(b["x"]), round(b["y"]))])})
        print(f"{d['filename']}: tổng {len(rows)} dòng", flush=True)

    with open(a.out, "w", encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")

    def agg(key):
        ed = sum(lev(r[key], r["qwen"]) for r in rows); n = sum(len(r["qwen"]) for r in rows)
        ex = sum(r[key] == r["qwen"] for r in rows)
        return ed / max(n, 1), ex / max(len(rows), 1)
    same = sum(r["ft"] == r["base"] for r in rows)
    print(f"\n{len(rows)} dòng thân văn bản từ {len(docs)} tài liệu (≥{a.min_chars} ký tự)")
    for k, name in (("base", "VietOCR nền     "), ("ft", "VietOCR fine-tune")):
        c, e = agg(k)
        print(f"  {name}: CER so với Qwen = {c:.4f}  (đồng thuận ký tự {100*(1-c):.2f}%) · dòng khớp tuyệt đối {100*e:.1f}%")
    print(f"  nền và fine-tune đọc giống hệt nhau: {100*same/max(len(rows),1):.1f}% số dòng")
    # Khi cả hai engine VietOCR cùng đọc giống nhau mà khác Qwen → nhiều khả năng Qwen sai.
    agree_both = [r for r in rows if r["ft"] == r["base"] and r["ft"] != r["qwen"]]
    print(f"  dòng mà hai VietOCR giống nhau nhưng khác Qwen: {len(agree_both)} — xem tay vài dòng trong {a.out}")


if __name__ == "__main__":
    main()
