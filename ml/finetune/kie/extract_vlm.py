#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import base64
import json
import math
import re
import struct
import subprocess
import sys
import tempfile
import time
import unicodedata
import urllib.error
import urllib.request
from pathlib import Path

FIELD_SPECS: list[tuple[str, str]] = [
    ("thuTu", "U+G7kSB0aOG7qSB04buxIGPhu6dhIHbEg24gYuG6o24gdHJvbmcgaOG7kyBzxqEgKGNo4buJIGNo4buvIHPhu5Ep"),
    ("loaiVanBan", "TG/huqFpIHbEg24gYuG6o24sIHbDrSBk4bulOiBRdXnhur90IMSR4buLbmgsIEPDtG5nIHbEg24sIFThu50gdHLDrG5oLCBCw6FvIGPDoW8gdMOgaSBjaMOtbmg="),
    ("soKyHieu", "U+G7kSB2w6Aga8O9IGhp4buHdSB2xINuIGLhuqNuLCB2w60gZOG7pTogOTk5OS9RxJAtVUJORA=="),
    ("ngayThang", "TmfDoHkgdGjDoW5nIGJhbiBow6BuaCwgxJHhu4tuaCBk4bqhbmcgZGQvbW0veXl5eQ=="),
    ("coQuanBanHanh", "VMOqbiBjxqEgcXVhbiBiYW4gaMOgbmggdsSDbiBi4bqjbg=="),
    ("trichYeu", "VHLDrWNoIHnhur91IG7hu5lpIGR1bmcgdsSDbiBi4bqjbiAoY8OzIHRo4buDIG5oaeG7gXUgZMOybmcp"),
    ("toSo", "VOG7nSBz4buRIChjaOG7iSBjaOG7ryBz4buRKQ=="),
    ("soTrang", "U+G7kSB0cmFuZyAoY2jhu4kgY2jhu68gc+G7kSk="),
    ("trangSo", "VHJhbmcgc+G7kSAoY2jhu4kgY2jhu68gc+G7kSk="),
    ("ngonNgu", "TmfDtG4gbmfhu68gY+G7p2EgdsSDbiBi4bqjbiwgdsOtIGThu6U6IFRp4bq/bmcgVmnhu4d0"),
    ("cheDoSuDung", "Q2jhur8gxJHhu5kgc+G7rSBk4bulbmcgdMOgaSBsaeG7h3U="),
    ("doTinCay", "xJDhu5kgdGluIGPhuq15IGPhu6dhIHTDoGkgbGnhu4d1"),
    ("butTich", "QsO6dCB0w61jaCB0csOqbiB2xINuIGLhuqNu"),
    ("tinhTrangVatLy", "VMOsbmggdHLhuqFuZyB24bqtdCBsw70gY+G7p2EgdMOgaSBsaeG7h3U="),
    ("taiLieuSo", "VGjDtG5nIHRpbiB0w6BpIGxp4buHdSBz4buR"),
    ("ghiChu", "R2hpIGNow7o="),
]
FIELD_KEYS: tuple[str, ...] = tuple(k for k, _ in FIELD_SPECS)

IDENTIFIER_FIELDS: frozenset[str] = frozenset(
    {"thuTu", "soKyHieu", "ngayThang", "toSo", "soTrang", "trangSo"}
)

PAGE_IDENTIFIER_FIELDS: frozenset[str] = frozenset({"soKyHieu", "ngayThang"})

GUIDED_SCHEMA: dict = {
    "type": "object",
    "properties": {k: {"type": "string", "description": d} for k, d in FIELD_SPECS},
    "required": list(FIELD_KEYS),
    "additionalProperties": False,
}

PROMPT = (
    "Bạn là hệ thống bóc tách thông tin văn bản hành chính Việt Nam.\n"
    "Hãy đọc ảnh trang văn bản và trích xuất 16 trường sau:\n\n"
    + "\n".join(f"- {k}: {d}" for k, d in FIELD_SPECS)
    + "\n\nQUY TẮC BẮT BUỘC:\n"
    "1. CHỈ trích xuất chữ THỰC SỰ nhìn thấy trên trang. TUYỆT ĐỐI KHÔNG suy đoán, không bịa.\n"
    "2. Trường nào không xuất hiện trên trang thì trả về chuỗi rỗng \"\". Không tự suy diễn.\n"
    "3. Giữ NGUYÊN VĂN tiếng Việt có dấu đúng như trên trang.\n"
    "4. Với số, ký hiệu, ngày tháng: chép chính xác từng ký tự, không chuẩn hoá lại.\n"
    "5. BỎ QUA hoàn toàn dấu chữ ký số / con dấu ở MÉP trang (dòng bắt đầu bằng"
    " \"Ký bởi:\", \"Thời gian:\", tên cơ quan quét/lưu trữ). Đó KHÔNG phải nội dung văn bản.\n"
    "\nQUY TẮC RIÊNG TỪNG TRƯỜNG (đọc kỹ — hay nhầm nhất):\n"
    "- coQuanBanHanh: LẤY cơ quan ở GÓC TRÊN BÊN TRÁI (tiêu đề văn bản). TUYỆT ĐỐI KHÔNG lấy"
    " cơ quan sau \"Kính gửi:\" (đó là NƠI NHẬN), KHÔNG lấy \"CỘNG HOÀ XÃ HỘI CHỦ NGHĨA VIỆT NAM\""
    " (đó là quốc hiệu), KHÔNG lấy tên cơ quan trong dấu chữ ký số ở mép, KHÔNG lấy cơ quan ban"
    " hành MẪU BIỂU (vd \"của Bộ Tài chính\" trong ô Mẫu số). Đơn/hợp đồng cá nhân không có cơ"
    " quan ban hành thì để rỗng.\n"
    "- soKyHieu: LẤY số ở đầu văn bản, ngay dưới cơ quan ban hành (vd \"Số: 123/QĐ-UBND\")."
    " KHÔNG lấy số của văn bản KHÁC được nhắc trong thân bài (\"Căn cứ Công văn số...\","
    " \"Thực hiện văn bản số...\").\n"
    "- ngayThang: LẤY ngày trên dòng \"...., ngày ... tháng ... năm ...\" ở ĐẦU văn bản (ngày ban"
    " hành). KHÔNG lấy ngày trong dấu \"CV ĐẾN / CÔNG VĂN ĐẾN\" (ngày nhận), KHÔNG lấy ngày trong"
    " thân bài, KHÔNG lấy ngày sinh, ngày cấp CMND, ngày kết thúc năm tài chính, hay thời gian"
    " chữ ký số ở mép.\n"
    "- loaiVanBan: LẤY từ DÒNG TIÊU ĐỀ in đậm giữa trang (QUYẾT ĐỊNH, TỜ TRÌNH, BÁO CÁO, THÔNG"
    " BÁO, KẾ HOẠCH, GIẤY PHÉP, GIẤY CHỨNG NHẬN, HỢP ĐỒNG, BIÊN BẢN, CÔNG VĂN...). Nếu không có"
    " dòng tiêu đề, suy từ ký hiệu (CV=Công văn, QĐ=Quyết định, TTr=Tờ trình, BC=Báo cáo,"
    " TB=Thông báo, KH=Kế hoạch, GP=Giấy phép).\n"
    "- trichYeu: LẤY dòng \"V/v ...\" hoặc \"Về việc ...\" ngay dưới số/ký hiệu (hoặc dòng phụ đề"
    " ngay dưới tiêu đề). KHÔNG lấy câu đầu THÂN BÀI (\"Căn cứ...\", \"Thực hiện...\", \"Kính"
    " gửi...\", tên cơ quan/công ty). Nếu văn bản không có dòng trích yếu thì để rỗng.\n"
    "\nChỉ trả về duy nhất một đối tượng JSON."
)

DEFAULT_ENDPOINT = "http://127.0.0.1:8003/v1/chat/completions"
DEFAULT_MODEL = "Qwen2.5-VL-7B-Instruct-AWQ"

QWEN_PATCH = 28
MODEL_MAX_LEN = 4096
PROMPT_TEXT_TOKENS = 1122
SAFETY_MARGIN = 64
MAX_IMAGE_TOKENS = 1850

QUOC_HIEU_MARKERS: tuple[re.Pattern, ...] = (
    re.compile(r"cong\s*hoa"),
    re.compile(r"hanh\s*phuc"),
    re.compile(r"doc\s*lap\s*[-–—]?\s*tu\s*do"),
)

def strip_dau(s: str) -> str:
    d = unicodedata.normalize("NFD", s)
    d = "".join(p_36a for p_36a in d if not unicodedata.combining(p_36a))
    d = unicodedata.normalize("NFC", d).lower()
    return d.replace("xJE=", "d")

def has_quoc_hieu(page_text_norm: str) -> bool:
    return any(p_36b.search(page_text_norm) for p_36b in QUOC_HIEU_MARKERS)

def q_160(page_texts: list[str]) -> set[int]:
    p_36c = {p_36d for p_36d, p_36e in enumerate(page_texts, 1) if has_quoc_hieu(strip_dau(p_36e))}
    if page_texts:
        p_36c.add(1)
    return p_36c

def page_id(pdf_rel: str, page_no: int) -> str:
    p_36f = pdf_rel.replace("/", "_").replace(".pdf", "").replace(".PDF", "")
    return f"{p_36f}_p{page_no:04d}"

def q_10e(pdf_rel: str) -> str:
    return pdf_rel

def q_146(path: Path) -> list[str]:
    out: list[str] = []
    for p_370 in path.read_text(encoding="utf-8").splitlines():
        p_370 = p_370.strip()
        if not p_370 or p_370.startswith("#"):
            continue
        out.append(str(Path(p_370)))
    return out

def q_114(pdfs: list[Path], data_dir: Path, wanted: list[str]
                   ) -> tuple[list[Path], list[str]]:
    p_374 = set(wanted)
    p_371 = [p_373 for p_373 in pdfs if str(p_373.relative_to(data_dir)) in p_374]
    p_372 = {str(p_373.relative_to(data_dir)) for p_373 in pdfs}
    return p_371, sorted(p_374 - p_372)

def q_11f(data: bytes) -> tuple[int, int]:
    p_376 = 2
    while p_376 < len(data) - 9:
        if data[p_376] != 0xFF:
            p_376 += 1
            continue
        p_377 = data[p_376 + 1]
        if p_377 in (0xC0, 0xC1, 0xC2, 0xC3):
            p_375, p_379 = struct.unpack(">HH", data[p_376 + 5 : p_376 + 9])
            return p_379, p_375
        if p_377 in (0xD8, 0xD9, 0x01) or 0xD0 <= p_377 <= 0xD7:
            p_376 += 2
            continue
        (p_378,) = struct.unpack(">H", data[p_376 + 2 : p_376 + 4])
        p_376 += 2 + p_378
    raise ValueError("a2jDtG5nIMSR4buNYyDEkcaw4bujYyBrw61jaCB0aMaw4bubYyBKUEVH")

def q_119(width: int, height: int) -> int:
    return int((width / QWEN_PATCH) * (height / QWEN_PATCH))

def q_116(width: int, height: int, budget: int = MAX_IMAGE_TOKENS) -> int | None:
    if width <= 0 or height <= 0:
        raise ValueError(f"a8OtY2ggdGjGsOG7m2Mg4bqjbmggdsO0IGzDvTog{width}x{height}")
    if q_119(width, height) <= budget:
        return None
    s = math.sqrt(0.99 * budget * (QWEN_PATCH**2) / (width * height))
    return max(1, int(s * max(width, height)))

def q_12f(img_tokens: int) -> int:
    budget = MODEL_MAX_LEN - img_tokens - PROMPT_TEXT_TOKENS - SAFETY_MARGIN
    if budget <= 0:
        raise ValueError(
            f"4bqjbmgg{img_tokens} token không còn chỗ cho đầu ra "
            f"(max_len={MODEL_MAX_LEN}KTsgcGjhuqNpIHRodSBuaOG7jyDhuqNuaCB0csaw4bubYw=="
        )
    return budget

def q_135(raw: dict, model_tag: str) -> list[dict]:
    out: list[dict] = []
    for p_37a in FIELD_KEYS:
        value = raw.get(p_37a)
        if not isinstance(value, str):
            continue
        value = value.strip()
        if not value:
            continue
        out.append(
            {
                "field_key": p_37a,
                "value": value,
                "bbox": None,
                "confidence": None,
                "verified": False,
                "is_identifier": p_37a in IDENTIFIER_FIELDS,
            }
        )
    return out

def q_fe(
    *,
    pdf_rel: str,
    page_no: int,
    image_rel: str,
    width: int,
    height: int,
    fields: list[dict],
    model_tag: str,
    latency_ms: int,
) -> dict:
    return {
        "id": page_id(pdf_rel, page_no),
        "document_id": q_10e(pdf_rel),
        "page_no": page_no,
        "image": image_rel,
        "width": width,
        "height": height,
        "source": "silver",
        "labeler": f"vlm:{model_tag}",
        "fields": fields,
        "meta": {"latency_ms": latency_ms},
    }

def load_done_ids(out_path: Path) -> set[str]:
    p_37b: set[str] = set()
    if not out_path.exists():
        return p_37b
    with out_path.open(encoding="utf-8") as p_37c:
        for p_37d in p_37c:
            p_37d = p_37d.strip()
            if not p_37d:
                continue
            try:
                p_37b.add(json.loads(p_37d)["id"])
            except (json.JSONDecodeError, KeyError):
                continue
    return p_37b

def q_13b(pdf: Path) -> list[str]:
    p_37e = subprocess.run(
        ["pdftotext", str(pdf), "-"], capture_output=True, text=True, check=False
    ).stdout
    pages = p_37e.split("\f")
    return pages[:-1] if len(pages) > 1 else pages

def q_13a(pdf: Path) -> int:
    out = subprocess.run(
        ["pdfinfo", str(pdf)], capture_output=True, text=True, check=True
    ).stdout
    for p_37f in out.splitlines():
        if p_37f.startswith("Pages:"):
            return int(p_37f.split()[1])
    raise ValueError(f"cGRmaW5mbyBraMO0bmcgY2hvIHPhu5EgdHJhbmc6IA=={pdf}")

def q_e9(pdf: Path, page_no: int, out_prefix: Path, *,
                  dpi: int | None = None, scale_to: int | None = None) -> bytes:
    p_380 = ["pdftoppm", "-jpeg", "-jpegopt", "quality=85",
           "-f", str(page_no), "-l", str(page_no), "-singlefile"]
    if scale_to is not None:
        p_380 += ["-scale-to", str(scale_to)]
    else:
        p_380 += ["-r", str(dpi)]
    p_380 += [str(pdf), str(out_prefix)]
    subprocess.run(p_380, check=True, capture_output=True)
    return out_prefix.with_suffix(".jpg").read_bytes()

def q_14e(pdf: Path, page_no: int, dest: Path, dpi: int) -> bytes:
    dest.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as p_383:
        p_382 = Path(p_383) / "pg"
        data = q_e9(pdf, page_no, p_382, dpi=dpi)
        p_384, p_381 = q_11f(data)
        scale_to = q_116(p_384, p_381)
        if scale_to is not None:
            data = q_e9(pdf, page_no, p_382, scale_to=scale_to)
    dest.write_bytes(data)
    return data

def q_102(image: bytes, endpoint: str, model: str, timeout: int,
             max_tokens: int) -> dict:
    p_385 = base64.b64encode(image).decode()
    p_386 = {
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "image_url",
                     "image_url": {"url": f"data:image/jpeg;base64,{p_385}"}},
                    {"type": "text", "text": PROMPT},
                ],
            }
        ],
        "temperature": 0.0,
        "max_tokens": max_tokens,
        "guided_json": GUIDED_SCHEMA,
    }
    p_389 = urllib.request.Request(
        endpoint, data=json.dumps(p_386).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(p_389, timeout=timeout) as p_38a:
        p_388 = json.load(p_38a)
    p_387 = p_388["choices"][0]
    if p_387.get("finish_reason") == "length":
        raise ValueError(
            f"SlNPTiBi4buLIGPhuq90IGPhu6V0IChmaW5pc2hfcmVhc29uPWxlbmd0aCwgbWF4X3Rva2Vucz0={max_tokens}); "
            f"trang quá đặc chữ — tăng ngân sách hoặc thu nhỏ ảnh"
        )
    return json.loads(p_387["message"]["content"])

def main(argv: list[str] | None = None) -> int:
    p_38b = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_38b.add_argument("data_dir", help="dGjGsCBt4bulYyBjaOG7qWEgUERGICjEkeG7hyBxdXkp")
    p_38b.add_argument("--out", default="data/kie_silver.jsonl")
    p_38b.add_argument("--img-dir", default="data/img")
    p_38b.add_argument("--endpoint", default=DEFAULT_ENDPOINT)
    p_38b.add_argument("--model", default=DEFAULT_MODEL)
    p_38b.add_argument("--dpi", type=int, default=100,
                    help="MTAwIOKHkiB+MTI1MCBpbWFnZSB0b2tlbiwgduG7q2EgbWF4LW1vZGVsLWxlbiA0MDk2")
    p_38b.add_argument("--timeout", type=int, default=300)
    p_38b.add_argument("--limit", type=int, default=0, help="Z2nhu5tpIGjhuqFuIHPhu5EgVFJBTkcgKDAgPSB04bqldCBj4bqjKQ==")
    p_38b.add_argument("--only-header-pages", action="store_true",
                    help="CHỈ bóc trang đầu văn bản (có quốc hiệu) ∪ trang 1 của mỗi PDF. "
                         "5 trường header chỉ có ở đó ⇒ tiết kiệm ~83%% thời gian GPU")
    p_38b.add_argument("--file-list", default=None,
                    help="tệp danh sách đường dẫn PDF (tương đối với data_dir), 1 dòng 1 tệp — "
                         "do sample_stratified.py sinh. CHỈ bóc các tệp này")
    p_38b.add_argument("--dry-run", action="store_true", help="Y2jhu4kgbGnhu4d0IGvDqiB0cmFuZywga2jDtG5nIGfhu41pIFZMTQ==")
    p_38c = p_38b.parse_args(argv)

    data_dir = Path(p_38c.data_dir).resolve()
    if not data_dir.is_dir():
        print(f"TOG7lkk6IGtow7RuZyB0aOG6pXkgdGjGsCBt4bulYyA={data_dir}", file=sys.stderr)
        return 2
    out_path = Path(p_38c.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    p_394 = Path(p_38c.img_dir)

    pdfs = sorted(p_39a for p_39a in data_dir.rglob("*") if p_39a.suffix.lower() == ".pdf")
    if not pdfs:
        print(f"TOG7lkk6IGtow7RuZyBjw7MgUERGIG7DoG8gdHJvbmcg{data_dir}", file=sys.stderr)
        return 2

    if p_38c.file_list:
        try:
            wanted = q_146(Path(p_38c.file_list))
        except OSError as e:
            print(f"TOG7lkk6IGtow7RuZyDEkeG7jWMgxJHGsOG7o2MgLS1maWxlLWxpc3Q6IA=={e}", file=sys.stderr)
            return 2
        pdfs, p_396 = q_114(pdfs, data_dir, wanted)
        if p_396:
            print(f"TOG7lkk6IC0tZmlsZS1saXN0IGPDsyA={len(p_396)}IG3hu6VjIGtow7RuZyB0aOG6pXkgdHJvbmcg{data_dir}",
                  file=sys.stderr)
            return 2
        if not pdfs:
            print("TOG7lkk6IC0tZmlsZS1saXN0IGtow7RuZyBraOG7m3AgUERGIG7DoG8=", file=sys.stderr)
            return 2
        print(f"LS1maWxlLWxpc3Q6IGLDs2Mg{len(pdfs)} PDF", file=sys.stderr)

    p_3a4: list[tuple[Path, str, int]] = []
    p_398 = 0
    p_399 = 0
    for p_392, pdf in enumerate(pdfs, 1):
        p_39e = str(pdf.relative_to(data_dir))
        try:
            if p_38c.only_header_pages:
                p_3a5 = q_13b(pdf)
                p_398 += len(p_3a5)
                pages = sorted(q_160(p_3a5))
            else:
                p_397 = q_13a(pdf)
                p_398 += p_397
                pages = list(range(1, p_397 + 1))
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired, ValueError, OSError):
            p_399 += 1
            continue
        p_3a4.extend((pdf, p_39e, pg) for pg in pages)
        if p_38c.only_header_pages and p_392 % 200 == 0:
            print(f"ICBxdcOpdCB0cmFuZ+KApiA={p_392}/{len(pdfs)} PDF", file=sys.stderr)
    if p_399:
        print(f"Q+G6ok5IIELDgU86IGLhu48gcXVhIA=={p_399}/{len(pdfs)}IFBERiBraMO0bmcgxJHhu41jIMSRxrDhu6NjIChwZGZpbmZvL3BkZnRvdGV4dCBs4buXaSk=",
              file=sys.stderr)

    p_38d = load_done_ids(out_path)
    p_3a6 = [p_3a2 for p_3a2 in p_3a4 if page_id(p_3a2[1], p_3a2[2]) not in p_38d]
    if p_38c.limit:
        p_3a6 = p_3a6[: p_38c.limit]

    p_39f = (f"Y2jhu41uIGzhu41jIA=={len(p_3a4)}/{p_398}IHRyYW5nIChi4buPIA=={p_398 - len(p_3a4)}IHRyYW5nIHRow6JuIGLDoGkp"
             if p_38c.only_header_pages else f"dHJhbmcgdOG7lW5nOiA={p_398}")
    print(f"PDF: {len(pdfs)} | {p_39f}IHwgxJHDoyB4b25nOiA={len(p_38d)} | "
          f"sẽ bóc: {len(p_3a6)}", file=sys.stderr)
    if p_38c.dry_run:
        for _, p_39e, pg in p_3a6[:20]:
            print(f"  {page_id(p_39e, pg)}", file=sys.stderr)
        return 0

    ok = p_38f = 0
    p_3a1 = time.monotonic()
    with out_path.open("a", encoding="utf-8") as p_3a0:
        for p_393, (pdf, p_39e, pg) in enumerate(p_3a6, 1):
            p_39b = page_id(p_39e, pg)
            try:
                p_395 = p_394 / f"{p_39b}.jpg"
                data = q_14e(pdf, pg, p_395, p_38c.dpi)
                p_3a7, p_391 = q_11f(data)
                p_3a3 = time.monotonic()
                raw = q_102(data, p_38c.endpoint, p_38c.model, p_38c.timeout,
                               q_12f(q_119(p_3a7, p_391)))
                p_38e = int((time.monotonic() - p_3a3) * 1000)
                p_39d = q_fe(
                    pdf_rel=p_39e, page_no=pg, image_rel=str(p_395),
                    width=p_3a7, height=p_391, fields=q_135(raw, p_38c.model),
                    model_tag=p_38c.model, latency_ms=p_38e,
                )
                p_3a0.write(json.dumps(p_39d, ensure_ascii=False) + "\n")
                p_3a0.flush()
                ok += 1
            except (subprocess.CalledProcessError, urllib.error.URLError,
                    urllib.error.HTTPError, ValueError, KeyError, TimeoutError) as exc:
                p_38f += 1
                print(f"ICBM4buWSSA={p_39b}: {type(exc).__name__}: {exc}", file=sys.stderr)
            if p_393 % 10 == 0 or p_393 == len(p_3a6):
                p_39c = p_393 / max(time.monotonic() - p_3a1, 1e-9)
                p_390 = (len(p_3a6) - p_393) / p_39c if p_39c else 0
                print(f"  [{p_393}/{len(p_3a6)}] ok={ok}IGzhu5dpPQ=={p_38f} "
                      f"{p_39c*60:.1f}IHRyYW5nL3Bow7p0IEVUQSA={p_390/60:.0f}IHBow7p0", file=sys.stderr)

    print(f"XONG: ok={ok}IGzhu5dpPQ=={p_38f} → {out_path}", file=sys.stderr)
    return 0 if p_38f == 0 else 1

if __name__ == "__main__":
    raise SystemExit(main())
