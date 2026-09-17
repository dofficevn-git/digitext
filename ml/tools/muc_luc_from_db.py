#!/usr/bin/env python3
"""Sinh mục lục hồ sơ (khuôn TT 02 mà `digitext tt02` nhận) từ CSDL DigiText.

Dùng khi tài liệu đã số hoá + bóc trường (v2) nhưng CHƯA chỉnh lý trên desktop: gom
các tài liệu cùng tiền tố mã hồ sơ trong tên tệp (…<mã hồ sơ>.<STT 7 số>.pdf), lấy
trường văn bản từ ocr_fields (ưu tiên giá trị người đã Duyệt, rồi tới engine *-kie).

Trường CẤP HỒ SƠ mà máy không biết (tiêu đề, thời hạn bảo quản, số tờ, mã hồ sơ
giấy) lấy từ tham số; không truyền thì điền giá trị TẠM và liệt kê ra để người chỉnh
lý thay — KHÔNG im lặng bịa.

    python muc_luc_from_db.py --project TEST --out muc_luc.json \
        --psql "docker exec -i digitext-dev-postgres-1 psql -U digitext -d digitext"
    digitext tt02 muc_luc.json --fond <mã phông> --org <mã CQ lưu trữ> --out wire/
"""
from __future__ import annotations

import argparse
import csv
import io
import json
import re
import shlex
import subprocess
import sys
import unicodedata
from collections import defaultdict

LOAI_CHUAN = ["nghị quyết", "quyết định", "chỉ thị", "quy chế", "quy định", "thông cáo", "thông báo",
              "hướng dẫn", "chương trình", "kế hoạch", "phương án", "đề án", "dự án", "báo cáo",
              "tờ trình", "giấy ủy quyền", "phiếu gửi", "phiếu chuyển", "phiếu báo", "biên bản",
              "hợp đồng", "công văn", "công điện", "bản ghi nhớ", "bản thỏa thuận", "giấy mời",
              "giấy giới thiệu", "giấy nghỉ phép", "thư công", "bản đồ", "bản vẽ kỹ thuật"]


def q(psql: str, sql: str) -> list[dict]:
    r = subprocess.run(shlex.split(psql) + ["--csv", "-v", "ON_ERROR_STOP=1", "-c", sql],
                       capture_output=True, text=True, encoding="utf-8")
    if r.returncode:
        raise SystemExit(f"❌ psql: {r.stderr.strip()}")
    csv.field_size_limit(10**9)
    return list(csv.DictReader(io.StringIO(r.stdout)))


def nfc(s: str | None) -> str:
    return " ".join(unicodedata.normalize("NFC", s or "").split())


def norm_date(s: str) -> tuple[str | None, bool]:
    """→ (dd/mm/yyyy, đã chuẩn hoá được?)"""
    s = nfc(s)
    m = re.search(r"(\d{1,2})\s*[/.\-]\s*(\d{1,2})\s*[/.\-]\s*(\d{4})", s) or \
        re.search(r"(\d{1,2})\s*tháng\s*(\d{1,2})\s*năm\s*(\d{4})", s, re.I)
    if not m:
        return (s or None), False
    d, mo, y = int(m.group(1)), int(m.group(2)), int(m.group(3))
    if not (1 <= d <= 31 and 1 <= mo <= 12 and 1900 <= y <= 2100):
        return s, False
    return f"{d:02d}/{mo:02d}/{y}", True


def split_so_ky_hieu(s: str) -> tuple[str | None, str | None]:
    s = nfc(s)
    s = re.sub(r"^(số|so)\s*[:.]?\s*", "", s, flags=re.I)
    m = re.match(r"^(\d+[a-zA-Z]?)\s*/\s*(.+)$", s)
    if m:
        return m.group(1), re.sub(r"\s*-\s*", "-", m.group(2)).strip()
    return None, (s or None)


def norm_loai(s: str, notes: list[str], stt: int) -> str | None:
    v = nfc(s).lower()
    if not v:
        return None
    for l in sorted(LOAI_CHUAN, key=len, reverse=True):
        if v == l or v.startswith(l + " "):
            if v != l:
                notes.append(f"STT {stt}: loại văn bản {s!r} → {l!r} (bảng mã TT05)")
            return l.capitalize()
    return nfc(s)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--psql", default="psql", help="lệnh psql (vd docker exec -i <ct> psql -U digitext -d digitext)")
    ap.add_argument("--project", required=True, help="mã dự án (projects.code)")
    ap.add_argument("--out", required=True)
    ap.add_argument("--tieu-de", default=None, help="tiêu đề hồ sơ (chỉ dùng khi dự án có đúng 1 hồ sơ)")
    ap.add_argument("--thoi-han", default=None, help='thời hạn bảo quản, vd "Vĩnh viễn", "70 năm"')
    ap.add_argument("--so-luong-to", type=int, default=None)
    ap.add_argument("--ma-ho-so-giay", default=None)
    a = ap.parse_args(argv)

    docs = q(a.psql, f"""select d.id::text id, d.filename, d.page_count from documents d
                         join projects p on p.id=d.project_id
                         where p.code={"'" + a.project.replace("'", "''") + "'"} and d.status<>'disposed'
                         order by d.filename""")
    if not docs:
        raise SystemExit("❌ dự án không có tài liệu")
    fields = q(a.psql, """select f.document_id::text d, f.field_key k, f.value v, f.verified ver,
                                 f.page_no p, e.name engine
                          from ocr_fields f join engine_versions e on e.id=f.engine_version_id
                          order by f.document_id, f.field_key, f.verified desc,
                                   (e.name like '%-kie') desc, f.page_no asc, f.updated_at desc""")
    best: dict[str, dict[str, tuple[str, bool]]] = defaultdict(dict)
    for r in fields:
        if r["k"] not in best[r["d"]] and nfc(r["v"]):
            best[r["d"]][r["k"]] = (nfc(r["v"]), r["ver"] in ("t", "true", "True"))

    groups: dict[str, list] = defaultdict(list)
    skipped = []
    for d in docs:
        m = re.match(r"^(.+)\.(\d{5,9})\.pdf$", d["filename"], re.I)
        if not m:
            skipped.append(d["filename"]); continue
        groups[m.group(1)].append((int(m.group(2)), d))

    out, notes, tam = [], [], []
    for ma, items in sorted(groups.items()):
        items.sort(key=lambda x: x[0])
        vb, dates, loai_count = [], [], defaultdict(int)
        n_verified = 0
        for stt, d in items:
            f = best.get(d["id"], {})
            g = lambda k: f.get(k, (None, False))[0]
            n_verified += sum(1 for v in f.values() if v[1])
            so, kh = split_so_ky_hieu(g("soKyHieu") or "")
            ngay, ok = norm_date(g("ngayThang") or "")
            if ngay and ok:
                dd, mm, yy = ngay.split("/"); dates.append((yy, mm, dd))
            elif ngay:
                notes.append(f"STT {stt}: ngày tháng {ngay!r} không chuẩn hoá được — giữ nguyên")
            loai = norm_loai(g("loaiVanBan") or "", notes, stt)
            if loai:
                loai_count[loai] += 1
            if not g("trichYeu"):
                notes.append(f"STT {stt}: thiếu trích yếu")
            if not (so or kh):
                notes.append(f"STT {stt}: thiếu số/ký hiệu")
            vb.append({k: v for k, v in {
                "stt": stt, "tenLoai": loai, "soVanBan": so, "kyHieu": kh, "ngayThang": ngay,
                "coQuanBanHanh": g("coQuanBanHanh"), "trichYeu": g("trichYeu"),
                "ngonNgu": [g("ngonNgu")] if g("ngonNgu") else [],
                "soTrang": int(d["page_count"]) if d["page_count"] else None,
                "butTich": g("butTich"), "ghiChu": g("ghiChu"),
                "documentId": d["id"], "tepTin": d["filename"],
            }.items() if v not in (None, [], "")})
        pages = sum(int(d["page_count"] or 0) for _, d in items)
        dates.sort()
        fmt = lambda t: f"{t[2]}/{t[1]}/{t[0]}"
        main_loai = max(loai_count, key=loai_count.get) if loai_count else "văn bản"
        organ = next((x.get("coQuanBanHanh") for x in vb if x.get("coQuanBanHanh")), "")
        year = dates[0][0] if dates else ""
        tieu_de = a.tieu_de if (a.tieu_de and len(groups) == 1) else None
        if not tieu_de:
            tieu_de = f"Tập {main_loai} năm {year} ({len(vb)} văn bản)".replace("  ", " ")
            tam.append(f"{ma}: tieuDe = {tieu_de!r} (máy đặt từ loại VB chiếm đa số; cơ quan ban hành đọc được: {organ!r})")
        thoi_han = a.thoi_han or "Vĩnh viễn"
        if not a.thoi_han:
            tam.append(f"{ma}: thoiHanBaoQuan = 'Vĩnh viễn' (chưa có thông tin chỉnh lý)")
        so_to = a.so_luong_to if a.so_luong_to is not None else pages
        if a.so_luong_to is None:
            tam.append(f"{ma}: soLuongTo = {pages} (lấy bằng số trang quét; tờ in hai mặt thì ít hơn)")
        giay = a.ma_ho_so_giay or ma
        if not a.ma_ho_so_giay:
            tam.append(f"{ma}: maHoSoGiay = mã hồ sơ số (chưa có mã hồ sơ giấy riêng)")
        out.append({
            "maHoSo": ma, "tieuDe": tieu_de, "thoiHanBaoQuan": thoi_han, "cheDoSuDung": "Công khai",
            "ngonNgu": ["Tiếng Việt"],
            "thoiGianBatDau": fmt(dates[0]) if dates else None,
            "thoiGianKetThuc": fmt(dates[-1]) if dates else None,
            "tongSoVanBan": len(vb), "soLuongTo": so_to, "soLuongTrang": pages,
            "tinhTrangVatLy": "Bình thường", "maHoSoGiay": giay, "vanBan": vb,
        })
        print(f"── {ma}: {len(vb)} văn bản, {pages} trang, {fmt(dates[0]) if dates else '?'} → "
              f"{fmt(dates[-1]) if dates else '?'}, trường đã Duyệt: {n_verified}")

    with open(a.out, "w", encoding="utf-8") as fh:
        json.dump(out if len(out) > 1 else out[0], fh, ensure_ascii=False, indent=1)
    print(f"✅ {a.out}: {len(out)} hồ sơ" + (f" · bỏ {len(skipped)} tệp không theo khuôn tên" if skipped else ""))
    if tam:
        print("\n⚠ GIÁ TRỊ TẠM — người chỉnh lý phải xác nhận/thay trước khi nộp lưu:")
        for t in tam:
            print("   •", t)
    if notes:
        print(f"\nGhi chú dữ liệu ({len(notes)}):")
        for n in notes[:25]:
            print("   •", n)
        if len(notes) > 25:
            print(f"   … và {len(notes) - 25} ghi chú nữa")
    return 0


if __name__ == "__main__":
    sys.exit(main())
