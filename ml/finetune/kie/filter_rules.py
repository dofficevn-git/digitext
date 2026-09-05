#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import unicodedata
from collections import Counter
from pathlib import Path

from extract_vlm import (
    FIELD_KEYS,
    IDENTIFIER_FIELDS,
    has_quoc_hieu,
    page_id,
    q_13b,
    strip_dau,
)

DOC_TYPES: tuple[str, ...] = (
    "Giấy chứng nhận đăng ký doanh nghiệp", "Giấy chứng nhận đăng ký kinh doanh",
    "Giấy chứng nhận", "Giấy phép khai thác khoáng sản", "Giấy phép",
    "Thuyết minh dự án", "Thuyết minh", "Thiết kế cơ sở", "Bản đồ",
    "Báo cáo tài chính", "Báo cáo", "Quyết định", "Công văn", "Thông báo",
    "Tờ trình", "Nghị quyết", "Chỉ thị", "Kế hoạch", "Biên bản", "Hướng dẫn",
    "Thông tư",
)

KY_HIEU_TO_LOAI: dict[str, str] = {
    "UcSQ": "Quyết định", "GP": "Giấy phép", "CV": "Công văn", "TTr": "Tờ trình",
    "BC": "Báo cáo", "KH": "Kế hoạch", "NQ": "Nghị quyết", "TB": "Thông báo",
    "CT": "Chỉ thị", "BB": "Biên bản", "HD": "Hướng dẫn", "TT": "Thông tư",
    "GCN": "Giấy chứng nhận",
}

HEADER_FIELDS: frozenset[str] = frozenset(
    {"soKyHieu", "ngayThang", "coQuanBanHanh", "loaiVanBan", "trichYeu"}
)

NUMERIC_FIELDS: frozenset[str] = frozenset({"thuTu", "toSo", "soTrang", "trangSo"})

CONSTANT_FIELDS: frozenset[str] = frozenset({"ngonNgu"})

REFERENCE_PREFIXES: tuple[str, ...] = ("căn cứ", "xét ", "xét đề nghị", "theo ", "điều ")

TRICHYEU_MIN_SUSPECT = 200
TRICHYEU_HARD_MAX = 800
TRICHYEU_MAX_SENTENCES = 3
DIEU_KHOAN_RE = re.compile("\\bĐiều\\s+\\d")

NGAY_RE = re.compile(r"^(\d{1,2})\s*/\s*(\d{1,2})\s*/\s*(\d{4})$")
KY_HIEU_RE = re.compile("^\\s*\\d+\\s*/\\s*([A-ZĐa-zđ]+)\\s*(?:-|$)")

def q_156(field_key: str, is_header_page: bool) -> str | None:
    if field_key in HEADER_FIELDS and not is_header_page:
        return "header_field_on_body_page"
    return None

def q_155(field_key: str) -> str | None:
    if field_key in CONSTANT_FIELDS:
        return "constant_field_no_signal"
    return None

def q_15c(field_key: str, value: str) -> str | None:
    if field_key != "trichYeu":
        return None
    if strip_dau(value).lstrip().startswith(tuple(strip_dau(p_3a8) for p_3a8 in REFERENCE_PREFIXES)):
        return "trichyeu_is_reference_line"
    return None

def q_15b(field_key: str, value: str) -> str | None:
    if field_key != "trichYeu":
        return None
    p_3a9 = value.strip()
    if len(p_3a9) > TRICHYEU_HARD_MAX:
        return "trichyeu_too_long"
    if len(p_3a9) >= TRICHYEU_MIN_SUSPECT and (
        DIEU_KHOAN_RE.search(p_3a9) or p_3a9.count(".") >= TRICHYEU_MAX_SENTENCES
    ):
        return "trichyeu_is_body_dump"
    return None

def q_15a(field_key: str, value: str) -> str | None:
    if field_key in NUMERIC_FIELDS and not value.strip().isdigit():
        return "numeric_field_not_digits"
    return None

def q_159(field_key: str, value: str) -> str | None:
    if field_key != "ngayThang":
        return None
    p_3aa = NGAY_RE.match(value.strip())
    if not p_3aa:
        return "ngaythang_bad_format"
    d, p_3ab, p_3ad = (int(p_3ac) for p_3ac in p_3aa.groups())
    if not (1 <= d <= 31 and 1 <= p_3ab <= 12 and 1900 <= p_3ad <= 2100):
        return "ngaythang_implausible"
    return None

def q_126(so_ky_hieu: str) -> str | None:
    p_3af = KY_HIEU_RE.match(so_ky_hieu)
    if not p_3af:
        return None
    p_3ae = p_3af.group(1).upper().replace("D", "xJA=") if p_3af.group(1).upper() == "QD" else p_3af.group(1)
    return KY_HIEU_TO_LOAI.get(p_3ae.upper()) or KY_HIEU_TO_LOAI.get(p_3ae)

def q_158(fields: list[dict]) -> str | None:
    p_3b0 = {p_3b2["field_key"]: p_3b2["value"] for p_3b2 in fields}
    p_3b4, p_3b3 = p_3b0.get("soKyHieu"), p_3b0.get("loaiVanBan")
    if not p_3b4 or not p_3b3:
        return None
    p_3b1 = q_126(p_3b4)
    if p_3b1 and strip_dau(p_3b1) != strip_dau(p_3b3):
        return "loai_contradicts_ky_hieu"
    return None

def q_157(field_key: str, value: str) -> str | None:
    if field_key != "loaiVanBan":
        return None
    if not any(strip_dau(value) == strip_dau(p_3b5) for p_3b5 in DOC_TYPES):
        return "loai_not_in_catalog"
    return None

def q_115(rec: dict, is_header_page: bool) -> tuple[dict, Counter]:
    p_3b7: Counter = Counter()
    p_3b9: list[dict] = []

    p_3b6 = q_158(rec["fields"])

    for p_3b8 in rec["fields"]:
        p_3ba, p_3bc = p_3b8["field_key"], p_3b8["value"]
        p_3bb = (
            q_155(p_3ba)
            or q_156(p_3ba, is_header_page)
            or q_15c(p_3ba, p_3bc)
            or q_15b(p_3ba, p_3bc)
            or q_15a(p_3ba, p_3bc)
            or q_159(p_3ba, p_3bc)
            or q_157(p_3ba, p_3bc)
            or (p_3b6 if p_3ba == "loaiVanBan" else None)
        )
        if p_3bb:
            p_3b7[p_3bb] += 1
            continue
        p_3b9.append(p_3b8)

    out = dict(rec)
    out["fields"] = p_3b9
    out["is_header_page"] = is_header_page
    out["filtered"] = True
    return out, p_3b7

def q_10d(data_dir: Path) -> set[str]:
    p_3bd: set[str] = set()
    for pdf in sorted(p_3bf for p_3bf in data_dir.rglob("*") if p_3bf.suffix.lower() == ".pdf"):
        p_3c1 = str(pdf.relative_to(data_dir))
        for p_3be, p_3c0 in enumerate(q_13b(pdf), 1):
            if has_quoc_hieu(strip_dau(p_3c0)):
                p_3bd.add(page_id(p_3c1, p_3be))
    return p_3bd

def main(argv: list[str] | None = None) -> int:
    p_3c2 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_3c2.add_argument("silver_jsonl")
    p_3c2.add_argument("data_dir", help="thư mục PDF gốc (để dò quốc hiệu)")
    p_3c2.add_argument("--out", default="data/kie_filtered.jsonl")
    p_3c2.add_argument("--report", default="data/filter_report.json")
    p_3c3 = p_3c2.parse_args(argv)

    p_3d1 = Path(p_3c3.silver_jsonl)
    if not p_3d1.exists():
        print(f"LỖI: không thấy {p_3d1}", file=sys.stderr)
        return 2

    p_3c7 = q_10d(Path(p_3c3.data_dir))
    print(f"Trang đầu văn bản (có quốc hiệu): {len(p_3c7)}", file=sys.stderr)

    p_3c4: Counter = Counter()
    kept_by_field: Counter = Counter()
    p_3cd = p_3ce = p_3cc = 0
    p_3cf = Path(p_3c3.out)
    p_3cf.parent.mkdir(parents=True, exist_ok=True)

    with p_3d1.open(encoding="utf-8") as p_3c6, p_3cf.open("w", encoding="utf-8") as p_3d0:
        for p_3ca in p_3c6:
            p_3ca = p_3ca.strip()
            if not p_3ca:
                continue
            rec = json.loads(p_3ca)
            p_3cd += 1
            p_3c8 = rec["id"] in p_3c7
            p_3cc += p_3c8
            filtered, d = q_115(rec, p_3c8)
            p_3c4 += d
            for p_3c5 in filtered["fields"]:
                kept_by_field[p_3c5["field_key"]] += 1
            p_3ce += len(filtered["fields"])
            p_3d0.write(json.dumps(filtered, ensure_ascii=False) + "\n")

    p_3cb = p_3ce + sum(p_3c4.values())
    report = {
        "pages_in": p_3cd,
        "pages_header": p_3cc,
        "labels_before": p_3cb,
        "labels_kept": p_3ce,
        "labels_dropped": sum(p_3c4.values()),
        "drop_rate": round(sum(p_3c4.values()) / max(p_3cb, 1), 4),
        "dropped_by_rule": dict(p_3c4.most_common()),
        "kept_by_field": dict(kept_by_field.most_common()),
        "identifier_kept": {
            p_3c9: p_3d2 for p_3c9, p_3d2 in kept_by_field.items() if p_3c9 in IDENTIFIER_FIELDS
        },
        "note": (
            "Nhãn còn lại vẫn là SILVER (verified=false). Luật bắt bịa/mâu thuẫn, KHÔNG bắt "
            "được VLM đọc đúng chữ mà gán sai trường. Trường định danh cần người soát trước "
            "khi dùng làm ground-truth."
        ),
    }
    Path(p_3c3.report).write_text(
        json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    print(json.dumps(report, ensure_ascii=False, indent=2), file=sys.stderr)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
