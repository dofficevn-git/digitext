"""Đo chất lượng nhận dạng của dịch vụ OCR trên dữ liệu thật có nhãn.

Kế hoạch xếp "chất lượng nhận dạng chưa đo được" là rủi ro số một: chuỗi Triton
cũ có rec_svtr_vi fine-tune riêng cho tiếng Việt, còn PaddleOCR lang='vi' là
model chung. Script này biến câu hỏi đó thành con số.

CÁCH ĐO — và giới hạn của nó:
Nhãn trong val.jsonl là 8 trường KIE, không phải văn bản từng dòng kèm toạ độ.
Nên đây KHÔNG phải phép đo CER/WER đúng nghĩa. Ta đo thứ yếu hơn nhưng vẫn có
giá trị: các giá trị trường (số văn bản, cơ quan, trích yếu...) đều là chữ CÓ
THẬT trên ảnh, nên nếu OCR đọc đúng thì chúng phải xuất hiện trong toàn văn.
Tỷ lệ tìm thấy là cận dưới của độ chính xác.

Chỉ dùng thư viện chuẩn — chạy được bằng Python 3.13 trên Windows, không cần
numpy/paddle (những thứ đó nằm trong container).

    python eval_ocr_quality.py --n 30
    python eval_ocr_quality.py --n 30 --endpoint http://127.0.0.1:5002
"""
from __future__ import annotations

import argparse
import base64
import json
import os
import re
import sys
import unicodedata
import urllib.error
import urllib.request

DEFAULT_DATA = r"D:\doc_extract_pla\exports\tu"
FIELDS = [
    "document_type",
    "document_number_and_symbol",
    "issue_date",
    "supervising_authority",
    "issuing_authority",
    "signer_name",
    "summary",
]


def norm(s: str) -> str:
    """Chuẩn hoá để so khớp: bỏ dấu, gộp khoảng trắng, thường hoá.

    Bỏ dấu là CỐ Ý — ta muốn tách bạch 'đọc sai chữ' với 'đọc sai dấu'. Sai dấu
    vẫn tệ với văn bản lưu trữ, nhưng là loại lỗi khác và nhẹ hơn nhiều.
    """
    s = unicodedata.normalize("NFD", s)
    s = "".join(c for c in s if unicodedata.category(c) != "Mn")
    s = re.sub(r"[^0-9a-zA-Z]+", " ", s)
    return " ".join(s.lower().split())


def match_field(field: str, gt: str, hay: str) -> bool:
    """So khớp một trường. Ba trường hợp cần đối xử khác nhau.

    Phiên bản đầu chỉ dùng "chuỗi con nguyên văn" cho mọi trường, cho ra 0% ở
    issue_date và 0% ở summary. Cả hai đều là LỖI CỦA PHÉP ĐO chứ không phải
    của OCR:

      - issue_date: nhãn chuẩn hoá về 23/05/2011, còn tài liệu viết
        "ngày 23 tháng 5 năm 2011". Không đời nào khớp nguyên văn.
      - summary: trích yếu dài hàng chục từ; sai một ký tự là trượt cả trường,
        nên nó đo "có hoàn hảo không" chứ không đo "đọc được bao nhiêu".
    """
    n = norm(gt)
    if not n:
        return False

    if field == "issue_date":
        # Đủ ngày + tháng + năm xuất hiện là tính đúng, bất kể định dạng.
        parts = re.findall(r"\d+", gt)
        if len(parts) >= 3:
            d, m, y = parts[0].lstrip("0"), parts[1].lstrip("0"), parts[2]
            toks = set(hay.split())
            return all(t in toks for t in (d, m, y))
        return n in hay

    if field == "summary":
        # Tỷ lệ TỪ của trích yếu tìm thấy trong toàn văn; ≥70% coi là đọc được.
        want = [w for w in n.split() if len(w) > 1]
        if not want:
            return False
        toks = set(hay.split())
        return sum(w in toks for w in want) / len(want) >= 0.7

    return n in hay


def load_labels(data_dir: str, split: str) -> list[dict]:
    path = os.path.join(data_dir, f"{split}.jsonl")
    rows = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            rec = json.loads(line)
            gt = None
            for turn in rec.get("conversations", []):
                if turn.get("from") == "gpt":
                    try:
                        gt = json.loads(re.sub(r"^```json|```$", "", turn["value"].strip(),
                                               flags=re.M).strip())
                    except json.JSONDecodeError:
                        gt = None
                    break
            if gt:
                rows.append({"image": rec["image"], "gt": gt})
    return rows


def ocr_tile(endpoint: str, api_key: str, img_path: str, timeout: int) -> dict:
    with open(img_path, "rb") as fh:
        b64 = base64.b64encode(fh.read()).decode()
    req = urllib.request.Request(
        endpoint.rstrip("/") + "/api/ocr/tile",
        data=json.dumps({"image": b64}).encode(),
        headers={"Content-Type": "application/json", "X-API-Key": api_key},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode())


def read_api_key(repo_root: str) -> str:
    env = os.path.join(repo_root, "infra", "compose", "ocr", ".env")
    if os.path.exists(env):
        for line in open(env, encoding="utf-8"):
            if line.startswith("OCR_API_KEY="):
                return line.split("=", 1)[1].strip()
    return os.getenv("OCR_API_KEY", "")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=DEFAULT_DATA)
    ap.add_argument("--split", default="val")
    ap.add_argument("--n", type=int, default=30)
    ap.add_argument("--endpoint", default="http://127.0.0.1:5002")
    ap.add_argument("--timeout", type=int, default=180)
    args = ap.parse_args()

    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    api_key = read_api_key(repo_root)

    rows = load_labels(args.data, args.split)[: args.n]
    if not rows:
        print(f"Không đọc được nhãn nào từ {args.data}/{args.split}.jsonl")
        return 1
    print(f"Đo trên {len(rows)} mẫu — {args.endpoint}\n")

    per_field = {f: [0, 0] for f in FIELDS}   # [tìm thấy, tổng]
    n_ok = n_err = 0
    total_spans = 0
    conf_sum = 0.0

    for i, row in enumerate(rows, 1):
        img = os.path.join(args.data, "images", row["image"])
        if not os.path.exists(img):
            continue
        try:
            res = ocr_tile(args.endpoint, api_key, img, args.timeout)
        except urllib.error.HTTPError as e:
            print(f"  [{i}] HTTP {e.code}: {e.read()[:120].decode(errors='replace')}")
            n_err += 1
            continue
        except Exception as e:  # noqa: BLE001
            print(f"  [{i}] lỗi: {e}")
            n_err += 1
            continue

        n_ok += 1
        total_spans += len(res.get("texts", []))
        conf_sum += sum(res.get("confidences", []))
        hay = norm(" ".join(res.get("texts", [])))

        for f in FIELDS:
            val = str(row["gt"].get(f, "") or "").strip()
            if len(val) < 3:
                continue
            per_field[f][1] += 1
            if match_field(f, val, hay):
                per_field[f][0] += 1

        if i % 10 == 0:
            print(f"  ...{i}/{len(rows)}")

    print(f"\n{'='*58}")
    print(f"ảnh đọc được       : {n_ok}   lỗi: {n_err}")
    if n_ok:
        print(f"dòng chữ / ảnh     : {total_spans / n_ok:.1f}")
        if total_spans:
            print(f"độ tin cậy trung bình: {conf_sum / total_spans:.3f}")
    print(f"\n{'trường':-<30} {'khớp':>8} {'tổng':>6} {'tỷ lệ':>7}")
    tot_hit = tot_all = 0
    for f in FIELDS:
        hit, all_ = per_field[f]
        tot_hit += hit
        tot_all += all_
        pct = f"{100*hit/all_:.0f}%" if all_ else "-"
        print(f"{f:-<30} {hit:>8} {all_:>6} {pct:>7}")
    pct = f"{100*tot_hit/tot_all:.1f}%" if tot_all else "-"
    print(f"{'CỘNG':-<30} {tot_hit:>8} {tot_all:>6} {pct:>7}")
    print("\nLưu ý: vẫn là cận DƯỚI. Đã bỏ dấu trước khi so nên lỗi dấu KHÔNG bị")
    print("tính — muốn đo cả dấu thì phải sửa norm(). Các trường thường (không")
    print("phải date/summary) vẫn so khớp nguyên văn nên sai một ký tự là trượt.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
