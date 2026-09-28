"""Đo cách bóc trường: (A) như DigiText hiện tại — Qwen chỉ nhìn ảnh; (B) ảnh + chữ OCR;
(C) ảnh + chữ OCR + K ví dụ giống nhất lấy từ kho ví dụ đã gán nhãn ("dạy bằng ví dụ").

Dữ liệu: ml/finetune/kie/data/plakie (tạo bằng plakie_import.py). Đo trên 47 ảnh heldout,
ví dụ chỉ lấy từ phần train. Chữ OCR lấy từ dịch vụ OCR DigiText (/api/ocr/tile), lưu đệm ở
data/plakie/ocr.jsonl để lần sau không phải đọc lại.

  python fewshot_bench.py --ocr http://192.168.0.205:5002 --vlm http://192.168.0.205:8003
"""
import argparse
import base64
import concurrent.futures as cf
import json
import re
import unicodedata
import urllib.request
from pathlib import Path

D = Path(__file__).parent / "data" / "plakie"
FIELDS = [
    ("loaiVanBan", "Loại văn bản (Quyết định, Công văn, Tờ trình, Báo cáo…)"),
    ("soKyHieu", "Số và ký hiệu văn bản, vd 123/QĐ-UBND"),
    ("ngayThang", "Ngày tháng năm ban hành, dạng dd/mm/yyyy"),
    ("coQuanChuQuan", "Cơ quan chủ quản (dòng TRÊN tên cơ quan ban hành ở góc trái, nếu có)"),
    ("coQuanBanHanh", "Cơ quan ban hành (góc trên bên trái)"),
    ("trichYeu", "Trích yếu nội dung (dòng V/v…, Về việc…, hoặc dòng dưới tiêu đề)"),
    ("nguoiKy", "Họ tên người ký"),
    ("soVietTay", "Số viết tay ở góc trên bên phải"),
]
RULES = """QUY TẮC:
1. Chỉ chép chữ THỰC SỰ có trên ảnh; trường không có thì để "".
2. Giữ nguyên văn tiếng Việt có dấu; số, ký hiệu, ngày chép đúng từng ký tự.
3. coQuanBanHanh lấy ở góc trên bên trái, KHÔNG lấy quốc hiệu, KHÔNG lấy nơi nhận sau "Kính gửi".
4. soKyHieu là số ngay dưới cơ quan ban hành, KHÔNG lấy số văn bản khác nhắc trong thân bài.
Chỉ trả về một đối tượng JSON với đúng các khoá trên."""


def post(url: str, payload: dict, headers: dict | None = None, timeout: int = 180) -> dict:
    req = urllib.request.Request(url, data=json.dumps(payload).encode(), method="POST",
                                 headers={"Content-Type": "application/json", **(headers or {})})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read().decode())


def ocr_text(endpoint: str, key: str, img: bytes) -> str:
    r = post(endpoint.rstrip("/") + "/api/ocr/tile", {"image": base64.b64encode(img).decode()}, {"X-API-Key": key})
    items = []
    for box, text in zip(r.get("boxes", []), r.get("texts", [])):
        pts = box if isinstance(box[0], (list, tuple)) else [box[:2], box[2:4]]
        ys = [p[1] for p in pts]
        xs = [p[0] for p in pts]
        items.append(((min(ys) + max(ys)) / 2, min(xs), max(ys) - min(ys), text))
    items.sort()
    lines: list[list] = []
    for y, x, h, t in items:
        if lines and abs(lines[-1][0] - y) < max(8, h * 0.5):
            lines[-1][1].append((x, t))
        else:
            lines.append([y, [(x, t)]])
    return "\n".join("   ".join(t for _, t in sorted(parts)) for _, parts in lines)


def norm(s: str) -> str:
    s = unicodedata.normalize("NFC", " ".join((s or "").split())).lower()
    return s.strip(" .,:;\"'“”")


def lev(a: str, b: str) -> int:
    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        cur = [i]
        for j, cb in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (ca != cb)))
        prev = cur
    return prev[-1]


def trigrams(s: str) -> set:
    s = f"  {norm(s)}  "
    return {s[i:i + 3] for i in range(len(s) - 2)}


def prompt(ocr: str | None, examples: list[dict]) -> str:
    p = ["Bạn là hệ thống bóc tách thông tin văn bản hành chính Việt Nam. Đọc ảnh phần đầu văn bản và điền các trường:"]
    p += [f"- {k}: {d}" for k, d in FIELDS]
    p.append(RULES)
    for i, e in enumerate(examples, 1):
        ans = {k: e["fields"].get(k, "") for k, _ in FIELDS}
        p.append(f"\nVÍ DỤ {i} (văn bản khác đã được người kiểm tra):\nChữ OCR:\n{e['ocr'][:600]}\nKết quả đúng:\n{json.dumps(ans, ensure_ascii=False)}")
    if ocr is not None:
        p.append("\nCHỮ OCR CỦA ẢNH CẦN BÓC (máy đọc, có thể sai dấu — đối chiếu với ảnh):\n" + ocr[:1500])
    p.append("\nTrả JSON cho ẢNH đính kèm:")
    return "\n".join(p)


def ask(vlm: str, model: str, img: bytes, text: str) -> dict:
    try:
        return _ask(vlm, model, img, text)
    except Exception as e:  # noqa: BLE001 — một ảnh lỗi thì tính là trả rỗng, không dừng cả đợt
        print("VLM lỗi:", str(e)[:120], flush=True)
        return {}


def _ask(vlm: str, model: str, img: bytes, text: str) -> dict:
    r = post(vlm.rstrip("/") + "/v1/chat/completions", {
        "model": model, "temperature": 0, "max_tokens": 700,
        "messages": [{"role": "user", "content": [
            {"type": "image_url", "image_url": {"url": "data:image/png;base64," + base64.b64encode(img).decode()}},
            {"type": "text", "text": text}]}],
    })
    out = r["choices"][0]["message"]["content"]
    m = re.search(r"\{.*\}", out, re.S)
    try:
        return json.loads(m.group(0)) if m else {}
    except json.JSONDecodeError:
        return {}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ocr", default="http://192.168.0.205:5002")
    ap.add_argument("--ocr-key", default="digitext-lan-dev-2026")
    ap.add_argument("--vlm", default="http://192.168.0.205:8003")
    ap.add_argument("--model", default="Qwen3-VL-8B-Instruct")
    ap.add_argument("-k", type=int, default=3)
    ap.add_argument("--workers", type=int, default=3)
    ap.add_argument("--only", default="", help="chỉ chạy các cách có tên bắt đầu bằng (vd C)")
    a = ap.parse_args()

    rows = [json.loads(l) for l in (D / "examples.jsonl").open(encoding="utf-8")]
    cache_f = D / "ocr.jsonl"
    cache = {}
    if cache_f.exists():
        cache = {r["image"]: r["ocr"] for r in map(json.loads, cache_f.open(encoding="utf-8"))}
    todo = [r for r in rows if r["image"] not in cache]
    if todo:
        print(f"OCR {len(todo)} ảnh…", flush=True)
        with cf.ThreadPoolExecutor(4) as ex, cache_f.open("a", encoding="utf-8") as w:
            futs = {ex.submit(ocr_text, a.ocr, a.ocr_key, (D / "images" / r["image"]).read_bytes()): r for r in todo}
            for f in cf.as_completed(futs):
                r = futs[f]
                try:
                    cache[r["image"]] = f.result()
                    w.write(json.dumps({"image": r["image"], "ocr": cache[r["image"]]}, ensure_ascii=False) + "\n")
                except Exception as e:  # noqa: BLE001
                    print("OCR lỗi", r["image"], e)
    for r in rows:
        r["ocr"] = cache.get(r["image"], "")
    bank = [r for r in rows if r["split"] == "train"]
    test = [r for r in rows if r["split"] == "heldout"]
    grams = [trigrams(r["ocr"]) for r in bank]

    def similar(r: dict) -> list[dict]:
        g = trigrams(r["ocr"])
        sc = sorted(((len(g & h) / (len(g | h) or 1), i) for i, h in enumerate(grams)), reverse=True)
        return [bank[i] for _, i in sc[: a.k]]

    variants = {
        "A_chi_anh": lambda r: prompt(None, []),
        "B_anh_ocr": lambda r: prompt(r["ocr"], []),
        f"C_anh_ocr_{a.k}vidu": lambda r: prompt(r["ocr"], similar(r)),
    }
    res: dict = {}
    for name, mk in variants.items():
        if a.only and not any(name.startswith(x) for x in a.only.split(",")):
            continue
        with cf.ThreadPoolExecutor(a.workers) as ex:
            preds = list(ex.map(lambda r: ask(a.vlm, a.model, (D / "images" / r["image"]).read_bytes(), mk(r)), test))
        per = {}
        for k, _ in FIELDS:
            ok = sim = 0.0
            for r, p in zip(test, preds):
                t, v = norm(r["fields"].get(k, "")), norm(str(p.get(k, "") or ""))
                ok += t == v
                sim += 1.0 if t == v else max(0.0, 1 - lev(v, t) / max(len(t), len(v), 1))
            per[k] = (round(100 * ok / len(test)), round(100 * sim / len(test)))
        res[name] = per
        avg = sum(v[0] for v in per.values()) / len(per)
        print(f"\n{name}: đúng hoàn toàn TB {avg:.0f}%", flush=True)
        for k, (e, s) in per.items():
            print(f"  {k:14s} đúng {e:3d}%   giống {s:3d}%")
    f_res = D / "bench_result.json"
    res = {**(json.loads(f_res.read_text(encoding="utf-8")) if f_res.exists() else {}), **res}
    (D / "bench_result.json").write_text(json.dumps(res, ensure_ascii=False, indent=1), encoding="utf-8")


if __name__ == "__main__":
    main()
