"""So các cấu hình OCR trên CÙNG trang và CÙNG dòng tham chiếu Qwen.

Đọc textlayer_agreement.jsonl (doc, page, qwen). Với mỗi (doc, page): lấy ảnh trang
từ PDF 2 lớp trong MinIO, gửi nguyên trang tới từng endpoint, ghép text, rồi với mỗi
dòng tham chiếu tìm cửa sổ khớp nhất trong text trang → CER. Cùng trang, cùng
tham chiếu nên chênh lệch giữa endpoint là do cấu hình, không do mẫu.

    python pad_eval.py name1=http://host:5002 name2=http://host:5003 ...
"""
import base64, csv, io, json, os, re, subprocess, sys, unicodedata, urllib.request
from collections import defaultdict

import fitz
from PIL import Image

sys.path.insert(0, r"D:\DigiText\ml\bench\harness")
from metrics import normalize_vi, levenshtein  # noqa: E402

KEY = [l.split("=", 1)[1].strip().strip('"') for l in open(r"D:\DigiText\crates\digitext-ocr\.env", encoding="utf-8")
       if l.startswith("DIGITEXT_OCR_HTTP_API_KEY=")][0]
PSQL = ["docker", "exec", "-i", "digitext-dev-postgres-1", "psql", "-U", "digitext", "-d", "digitext", "--csv", "-c"]


def q(sql):
    r = subprocess.run(PSQL + [sql], capture_output=True, text=True, encoding="utf-8")
    return list(csv.DictReader(io.StringIO(r.stdout)))


def minio_get(key):
    return subprocess.run(["docker", "exec", "digitext-dev-minio-1", "mc", "cat", f"local/pages/{key}"],
                          capture_output=True).stdout


def N(s):
    s = normalize_vi(s or "")
    return re.sub(r"\s+", " ", s).strip()


def ocr_page(endpoint, png):
    body = json.dumps({"image": base64.b64encode(png).decode()}).encode()
    req = urllib.request.Request(endpoint.rstrip("/") + "/api/ocr/tile", data=body,
                                 headers={"Content-Type": "application/json", "X-API-Key": KEY})
    r = json.load(urllib.request.urlopen(req, timeout=180))
    return [N(t) for t in r["texts"]]


def best_line_cer(ref, lines):
    """CER nhỏ nhất của ref so với từng dòng OCR (và với cặp dòng liền nhau ghép lại)."""
    best = len(ref)
    cands = lines + [a + " " + b for a, b in zip(lines, lines[1:])]
    for c in cands:
        if abs(len(c) - len(ref)) > max(6, len(ref) // 2):
            continue
        best = min(best, levenshtein(c, ref))
        if best == 0:
            break
    return best


def main():
    eps = dict(a.split("=", 1) for a in sys.argv[1:])
    refs = defaultdict(list)
    for l in open(os.path.join(os.path.dirname(__file__), "textlayer_agreement.jsonl"), encoding="utf-8"):
        r = json.loads(l)
        refs[(r["doc"], r["page"])].append(N(r["qwen"]))
    keys = {d["filename"]: d["pdfa_key"] for d in q("select filename, pdfa_key from documents")}
    tot = {n: [0, 0, 0] for n in eps}  # edit, chars, exact
    nang = {n: 0 for n in eps}
    n_lines = 0
    for (doc, page), rl in sorted(refs.items()):
        pdf = fitz.open(stream=minio_get(keys[doc]), filetype="pdf")
        ex = pdf.extract_image(pdf[page - 1].get_images(full=True)[0][0])
        im = Image.open(io.BytesIO(ex["image"])).convert("RGB")
        buf = io.BytesIO(); im.save(buf, "PNG"); png = buf.getvalue()
        out = {n: ocr_page(u, png) for n, u in eps.items()}
        for ref in rl:
            n_lines += 1
            for n in eps:
                e = best_line_cer(ref, out[n])
                tot[n][0] += e; tot[n][1] += len(ref); tot[n][2] += int(e == 0)
        print(f"{doc} p{page}: " + " · ".join(f"{n} {100*(1-tot[n][0]/max(tot[n][1],1)):.2f}%" for n in eps), flush=True)
    print(f"\n{n_lines} dòng tham chiếu Qwen, {len(refs)} trang")
    for n in eps:
        e, c, x = tot[n]
        print(f"  {n:12s}: đồng thuận ký tự {100*(1-e/c):.2f}%  · dòng khớp tuyệt đối {100*x/n_lines:.1f}%")


if __name__ == "__main__":
    main()
