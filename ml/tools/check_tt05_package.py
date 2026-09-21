#!/usr/bin/env python3
"""Kiểm ĐỘC LẬP một gói tin hồ sơ (AIP/SIP) theo bố cục TT 05/2025/TT-BNV mà DigiText sinh.

Cố ý không dùng crate digitext-package: cùng một code vừa dựng vừa kiểm thì lỗi hiểu
sai chuẩn sẽ qua cả hai. Script này đọc lại gói từ đĩa và kiểm những gì người nhận
gói (Lưu trữ lịch sử) sẽ kiểm:

  1. ZIP mở được, có đúng MỘT thư mục gốc; XML nào cũng well-formed.
  2. Mọi tham chiếu trong METS (gốc + rep1) trỏ tới tệp CÓ THẬT, đúng SIZE và
     SHA-256; không có tệp mồ côi ngoài METS.
  3. EAD hồ sơ / EAD văn bản hợp lệ theo XSD đi kèm trong schemas/.
  4. Giá trị mã nằm trong bảng mã TT05; ngày dd/mm/yyyy; arcDocCode = arcFileCode.STT7;
     tên tệp dữ liệu = arcDocCode + đuôi; totalDoc = số tệp dữ liệu = số EAD văn bản.
  5. PREMIS có đối tượng cho từng tệp dữ liệu với SHA-256 khớp.
  6. Tệp PDF mang dấu PDF/A (XMP pdfaid) — kiểm chuẩn đầy đủ thì chạy veraPDF riêng.

    python check_tt05_package.py <gói.zip | thư mục gói đã giải nén>
Mã thoát 0 = không có lỗi (cảnh báo không tính).
"""
from __future__ import annotations

import hashlib
import os
import re
import sys
import tempfile
import zipfile

from lxml import etree

NS = {"m": "http://www.loc.gov/METS/", "x": "http://www.w3.org/1999/xlink",
      "p": "http://www.loc.gov/premis/v3"}
CODES = {"maintenance": 7, "mode": 3, "language": 11, "typeName": 32, "confidenceLevel": 3, "format": 3}
DATE_FIELDS = ("startDate", "endDate", "issuedDate")
DATE_RE = re.compile(r"^(0[1-9]|[12]\d|3[01])/(0[1-9]|1[0-2])/(1[89]|20)\d\d$")

errors: list[str] = []
warns: list[str] = []
passed: list[str] = []


def err(m): errors.append(m)
def warn(m): warns.append(m)
def ok(m): passed.append(m)


def sha256(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def parse(path: str):
    try:
        return etree.parse(path)
    except etree.XMLSyntaxError as e:
        err(f"XML hỏng: {os.path.relpath(path)}: {e}")
        return None


def check_mets(mets_path: str, root: str, referenced: set[str]) -> None:
    doc = parse(mets_path)
    if doc is None:
        return
    base = os.path.dirname(mets_path)
    n = bad = 0
    for el in doc.iter("{%s}mdRef" % NS["m"], "{%s}file" % NS["m"]):
        if el.tag.endswith("file"):
            loc = el.find("m:FLocat", NS)
            href = loc.get("{%s}href" % NS["x"]) if loc is not None else None
        else:
            href = el.get("{%s}href" % NS["x"])
        if not href:
            err(f"{os.path.relpath(mets_path, root)}: phần tử {el.get('ID')} thiếu href"); continue
        target = os.path.normpath(os.path.join(base, href))
        referenced.add(target)
        n += 1
        if not os.path.isfile(target):
            err(f"{os.path.relpath(mets_path, root)}: tham chiếu '{href}' không tồn tại"); bad += 1; continue
        size, cks = el.get("SIZE"), el.get("CHECKSUM")
        if el.get("CHECKSUMTYPE") != "SHA-256":
            err(f"{href}: CHECKSUMTYPE phải là SHA-256, đang là {el.get('CHECKSUMTYPE')!r}"); bad += 1
        if size is None or int(size) != os.path.getsize(target):
            err(f"{href}: SIZE {size} ≠ thực {os.path.getsize(target)}"); bad += 1
        if cks is None or cks.lower() != sha256(target):
            err(f"{href}: CHECKSUM lệch"); bad += 1
    for mptr in doc.iter("{%s}mptr" % NS["m"]):
        href = mptr.get("{%s}href" % NS["x"])
        target = os.path.normpath(os.path.join(base, href or ""))
        referenced.add(target)
        if not os.path.isfile(target):
            err(f"{os.path.relpath(mets_path, root)}: mptr '{href}' không tồn tại")
    if not bad:
        ok(f"{os.path.relpath(mets_path, root)}: {n} tham chiếu — tồn tại, đúng SIZE, đúng SHA-256")


def fields(doc) -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for el in doc.getroot():
        out.setdefault(etree.QName(el).localname, []).append((el.text or "").strip())
    return out


def check_codes(name: str, f: dict[str, list[str]]) -> None:
    for k, mx in CODES.items():
        for v in f.get(k, []):
            if not (re.fullmatch(r"\d\d", v) and 1 <= int(v) <= mx):
                err(f"{name}: {k}={v!r} ngoài bảng mã 01..{mx:02d}")
    for k in DATE_FIELDS:
        for v in f.get(k, []):
            if not DATE_RE.match(v):
                warn(f"{name}: {k}={v!r} không theo dd/mm/yyyy")
    for v in f.get("riskRecovery", []):
        if v not in ("0", "1"):
            err(f"{name}: riskRecovery={v!r} (chỉ 0/1)")


def main() -> int:
    # Console Windows mặc định cp1252 → in tiếng Việt/ký hiệu ─ ✓ ✗ là nổ UnicodeEncodeError.
    for _st in (sys.stdout, sys.stderr):
        try:
            _st.reconfigure(encoding="utf-8", errors="replace")
        except Exception:  # noqa: BLE001
            pass
    src = sys.argv[1]
    tmp = None
    if os.path.isfile(src):
        tmp = tempfile.TemporaryDirectory()
        try:
            with zipfile.ZipFile(src) as z:
                bad = z.testzip()
                if bad:
                    err(f"ZIP hỏng ở mục {bad}")
                z.extractall(tmp.name)
        except zipfile.BadZipFile as e:
            print(f"✗ không mở được ZIP: {e}"); return 1
        ok(f"ZIP mở được, CRC mọi mục đúng · SHA-256 gói: {sha256(src)}")
        top = tmp.name
    else:
        top = src
    roots = [d for d in os.listdir(top) if os.path.isdir(os.path.join(top, d))]
    if len(roots) != 1 or [f for f in os.listdir(top) if os.path.isfile(os.path.join(top, f))]:
        err(f"gói phải có đúng MỘT thư mục gốc, đang có: {os.listdir(top)}")
        if not roots:
            return report()
    root = os.path.join(top, roots[0])

    required = ["METS.xml", "representations/rep1/METS.xml", "schemas/METS.xsd"]
    for r in required:
        if not os.path.isfile(os.path.join(root, r)):
            err(f"thiếu tệp bắt buộc: {r}")
    all_files = {os.path.normpath(os.path.join(dp, f)) for dp, _, fs in os.walk(root) for f in fs}
    xmls = [f for f in all_files if f.lower().endswith((".xml", ".xsd"))]
    n_bad = sum(1 for f in xmls if parse(f) is None)
    if not n_bad:
        ok(f"{len(xmls)} tệp XML/XSD đều well-formed")

    referenced: set[str] = set()
    for m in ("METS.xml", "representations/rep1/METS.xml"):
        p = os.path.join(root, m)
        if os.path.isfile(p):
            check_mets(p, root, referenced)
    orphans = all_files - referenced - {os.path.normpath(os.path.join(root, "METS.xml"))}
    if orphans:
        for o in sorted(orphans)[:10]:
            err(f"tệp mồ côi (không METS nào tham chiếu): {os.path.relpath(o, root)}")
    else:
        ok(f"không có tệp mồ côi — cả {len(all_files)} tệp đều được METS tham chiếu")

    # METS gốc: OBJID ↔ tên thư mục, loại gói
    mdoc = parse(os.path.join(root, "METS.xml"))
    pkg_type = None
    if mdoc is not None:
        objid = mdoc.getroot().get("OBJID", "")
        if objid.replace(":", "_") != roots[0]:
            err(f"tên thư mục gốc {roots[0]!r} ≠ OBJID {objid!r} (thay ':' bằng '_')")
        else:
            ok(f"OBJID {objid} khớp tên thư mục gốc")
        hdr = mdoc.getroot().find("m:metsHdr", NS)
        pkg_type = hdr.get("{https://DILCIS.eu/XML/METS/CSIPExtensionMETS}OAISPACKAGETYPE") if hdr is not None else None
        ok(f"loại gói OAIS: {pkg_type} · PROFILE: {mdoc.getroot().get('PROFILE')}")

    # EAD hồ sơ
    ead_p = next((os.path.join(root, "metadata/descriptive", n) for n in ("EAD.xml", "EAD_TL.xml")
                  if os.path.isfile(os.path.join(root, "metadata/descriptive", n))), None)
    hs: dict[str, list[str]] = {}
    if not ead_p:
        err("thiếu metadata/descriptive/EAD.xml")
    else:
        d = parse(ead_p)
        if d is not None:
            hs = fields(d)
            check_codes("EAD hồ sơ", hs)
            xsd_p = os.path.join(root, "schemas", os.path.basename(ead_p).replace(".xml", ".xsd"))
            if os.path.isfile(xsd_p):
                schema = etree.XMLSchema(etree.parse(xsd_p))
                if schema.validate(d):
                    ok(f"{os.path.basename(ead_p)} hợp lệ theo schemas/{os.path.basename(xsd_p)}")
                else:
                    err(f"{os.path.basename(ead_p)} KHÔNG hợp lệ XSD: {schema.error_log.last_error}")
            for k in ("title", "maintenance", "mode", "language", "totalDoc", "confidenceLevel"):
                if not hs.get(k) or not hs[k][0]:
                    err(f"EAD hồ sơ thiếu trường bắt buộc: {k}")
    file_code = (hs.get("arcFileCode") or hs.get("fileCode") or [""])[0]
    if not file_code:
        err("EAD hồ sơ thiếu arcFileCode/fileCode")

    # EAD văn bản ↔ tệp dữ liệu
    ddir = os.path.join(root, "representations/rep1/data")
    edir = os.path.join(root, "representations/rep1/metadata/descriptive")
    data = sorted(os.listdir(ddir)) if os.path.isdir(ddir) else []
    eads = sorted(f for f in os.listdir(edir)) if os.path.isdir(edir) else []
    doc_schema = None
    xsd_doc = os.path.join(root, "schemas/EAD_doc.xsd")
    if os.path.isfile(xsd_doc):
        doc_schema = etree.XMLSchema(etree.parse(xsd_doc))
    codes_seen, n_xsd_bad, missing_subject = set(), 0, 0
    for e in eads:
        d = parse(os.path.join(edir, e))
        if d is None:
            continue
        f = fields(d)
        check_codes(e, f)
        code = (f.get("arcDocCode") or f.get("docCode") or [""])[0]
        if not code:
            err(f"{e}: thiếu arcDocCode/docCode"); continue
        if code in codes_seen:
            err(f"{e}: mã văn bản trùng {code}")
        codes_seen.add(code)
        if file_code and not re.fullmatch(re.escape(file_code) + r"\.\d{7}", code):
            err(f"{e}: mã văn bản {code!r} không theo khuôn <mã hồ sơ>.<STT 7 số>")
        if not any(x.rsplit(".", 1)[0] == code for x in data):
            err(f"{e}: không có tệp dữ liệu tên {code}.*")
        if e.startswith("EAD_doc_") and doc_schema is not None and not doc_schema.validate(d):
            n_xsd_bad += 1
            err(f"{e} KHÔNG hợp lệ EAD_doc.xsd: {doc_schema.error_log.last_error}")
        if not (f.get("subject") or [""])[0]:
            missing_subject += 1
    if eads and not n_xsd_bad and doc_schema is not None:
        ok(f"{len(eads)} EAD văn bản hợp lệ theo schemas/EAD_doc.xsd")
    if missing_subject:
        warn(f"{missing_subject} văn bản không có trích yếu (subject)")
    total = int((hs.get("totalDoc") or ["-1"])[0] or -1)
    if not (total == len(data) == len(eads)):
        err(f"totalDoc={total} · tệp dữ liệu={len(data)} · EAD văn bản={len(eads)} — phải bằng nhau")
    else:
        ok(f"totalDoc = số tệp dữ liệu = số EAD văn bản = {total}; mã văn bản duy nhất, đúng khuôn")
    sum_pages = 0
    for e in eads:
        d = parse(os.path.join(edir, e))
        if d is not None:
            sum_pages += int((fields(d).get("numberOfPage") or ["0"])[0] or 0)
    hs_pages = int((hs.get("numberOfPage") or ["0"])[0] or 0)
    if hs_pages and sum_pages and hs_pages != sum_pages:
        warn(f"numberOfPage hồ sơ ({hs_pages}) ≠ tổng số trang các văn bản ({sum_pages})")
    elif hs_pages:
        ok(f"numberOfPage hồ sơ = tổng số trang các văn bản = {hs_pages}")

    # PREMIS
    for prem in ("metadata/preservation/PREMIS.xml", "representations/rep1/metadata/preservation/PREMIS_rep1.xml"):
        pp = os.path.join(root, prem)
        if not os.path.isfile(pp):
            (err if pkg_type == "AIP" else warn)(f"thiếu {prem}" + (" (AIP bắt buộc PREMIS)" if pkg_type == "AIP" else ""))
            continue
        d = parse(pp)
        if d is None:
            continue
        digests = {x.text.strip().lower() for x in d.iter() if etree.QName(x).localname == "messageDigest" and x.text}
        names = {x.text.strip() for x in d.iter() if etree.QName(x).localname == "originalName" and x.text}
        if prem.endswith("PREMIS_rep1.xml"):
            miss = [x for x in data if x not in names]
            badsum = [x for x in data if sha256(os.path.join(ddir, x)) not in digests]
            if miss:
                err(f"PREMIS_rep1 thiếu đối tượng cho {len(miss)} tệp dữ liệu, vd {miss[:2]}")
            if badsum:
                err(f"PREMIS_rep1: {len(badsum)} tệp dữ liệu không có messageDigest SHA-256 khớp, vd {badsum[:2]}")
            if not miss and not badsum:
                ok(f"PREMIS_rep1: đủ {len(data)} đối tượng, SHA-256 khớp tệp")
        events = [x.text for x in d.iter() if etree.QName(x).localname == "eventType"]
        ok(f"{prem}: eventType = {events}")

    # PDF/A
    pdfs = [x for x in data if x.lower().endswith(".pdf")]
    try:
        import fitz

        no_mark, no_text, signed = [], [], 0
        for x in pdfs:
            with fitz.open(os.path.join(ddir, x)) as pdf:
                xmp = pdf.get_xml_metadata() or ""
                if "<pdfaid:part>" not in xmp:
                    no_mark.append(x)
                if not any(pdf[i].get_text().strip() for i in range(len(pdf))):
                    no_text.append(x)
                if pdf.get_sigflags() > 0:
                    signed += 1
        if no_mark:
            err(f"{len(no_mark)} PDF không mang dấu PDF/A (XMP pdfaid), vd {no_mark[:2]}")
        else:
            ok(f"{len(pdfs)} PDF đều mang dấu PDF/A trong XMP (chạy veraPDF để kiểm chuẩn đầy đủ)")
        if no_text:
            err(f"{len(no_text)} PDF không có lớp chữ, vd {no_text[:2]}")
        else:
            ok(f"{len(pdfs)} PDF đều có lớp chữ tìm kiếm được")
        if signed < len(pdfs):
            warn(f"{len(pdfs) - signed}/{len(pdfs)} PDF CHƯA ký số — TT05 Điều 8 yêu cầu tài liệu số hoá có chữ ký số của cơ quan")
        else:
            ok(f"{signed} PDF đều có chữ ký số")
    except ImportError:
        warn("không có PyMuPDF — bỏ qua kiểm PDF")
    if tmp:
        tmp.cleanup()
    return report()


def report() -> int:
    print(f"\n── ĐẠT ({len(passed)})")
    for m in passed:
        print("  ✓", m)
    if warns:
        print(f"\n── CẢNH BÁO ({len(warns)})")
        for m in warns:
            print("  !", m)
    print(f"\n── LỖI ({len(errors)})")
    for m in errors[:40]:
        print("  ✗", m)
    if len(errors) > 40:
        print(f"  … và {len(errors) - 40} lỗi nữa")
    print("\nKẾT LUẬN:", "KHÔNG ĐẠT" if errors else ("ĐẠT (có cảnh báo)" if warns else "ĐẠT"))
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
