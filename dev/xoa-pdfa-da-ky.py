"""Xoá bản PDF/A (chưa ký) của tài liệu ĐÃ KÝ khỏi kho máy chủ, giữ bản ký số — anh Hiếu chốt 16/10/2026.

    python -I dev/xoa-pdfa-da-ky.py --du-an LTLSBN --ban-ky "F:/2026/PDFa/LTLSBN · LTLSBNinh"           ← CHẠY THỬ
    python -I dev/xoa-pdfa-da-ky.py --du-an LTLSBN --ban-ky "F:/2026/PDFa/LTLSBN · LTLSBNinh" --that    ← xoá thật

Chỉ xoá PDF/A của tài liệu thoả TẤT CẢ:
  - đã ký (status = signed), bản ký số còn trong kho;
  - bản ký số có trên ổ ngoài (--ban-ky, dò cả thư mục lồng tên dự án và đường dẫn bị app rút còn 3 cấp)
    và ĐÚNG kích thước bản ký trong kho.
Trước khi xoá ghi DANH SÁCH (mã tài liệu, phông/hộp/hồ sơ, tên tệp, khoá + kích thước + ETag PDF/A, bản ký, đường dẫn
trên ổ ngoài) ra <ban-ky>/../danh-sach-pdfa-da-xoa-<mã>.csv và logs/. Đánh dấu documents.pdfa_offloaded_at (migration 0019).
Bản PDF/A KHÔNG khôi phục được từ bản ký (16/10: máy chủ ghi lại cả tệp khi ký — 0/50 trùng phần đầu).
"""
from __future__ import annotations

import argparse
import csv
import io
import os
import re
import subprocess
import sys
import time
from collections import defaultdict
from pathlib import Path

import boto3

REPO = Path(__file__).resolve().parent.parent
PG = ["docker", "exec", "-i", "digitext-dev-postgres-1", "psql", "-U", "digitext", "-d", "digitext", "-v", "ON_ERROR_STOP=1", "-q"]


def doc_env(p: Path) -> dict[str, str]:
    kv = {}
    for line in p.read_text(encoding="utf-8").splitlines():
        m = re.match(r"\s*([A-Z0-9_]+)\s*=\s*(.*?)\s*$", line)
        if m:
            kv[m.group(1)] = m.group(2).strip('"')
    return kv


def psql_csv(sql: str) -> list[list[str]]:
    out = subprocess.run(PG + ["-c", f"COPY ({sql}) TO STDOUT WITH CSV"], capture_output=True, check=True).stdout.decode("utf-8")
    return list(csv.reader(io.StringIO(out)))


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--du-an", required=True)
    ap.add_argument("--ban-ky", required=True, help="thư mục bản ký số đã tải ra ổ ngoài")
    ap.add_argument("--that", action="store_true", help="xoá thật (mặc định chỉ chạy thử)")
    ap.add_argument("--da-doi-chieu", action="store_true",
                    help="ổ ngoài không cắm vào máy chủ: dùng kết quả đối chiếu trước (16/10 10:45: 160.375/160.375 bản ký khớp) — "
                         "chỉ kiểm bản ký còn trong kho")
    a = ap.parse_args()
    goc = Path(a.ban_ky)
    if not goc.is_dir() and not a.da_doi_chieu:
        print(f"Không thấy {goc} — cắm ổ ngoài rồi chạy lại.")
        return 2

    ma = a.du_an.replace("'", "''")
    pid = psql_csv(f"SELECT id FROM projects WHERE code = '{ma}'")[0][0]
    docs = psql_csv(
        "SELECT d.id, COALESCE(d.rel_path,''), d.filename, COALESCE(d.pdfa_key,''), "
        "COALESCE((SELECT s.signed_key FROM sign_sessions s WHERE s.document_id = d.id AND s.status = 'signed' "
        "AND s.signed_key IS NOT NULL ORDER BY s.updated_at DESC LIMIT 1), '') "
        f"FROM documents d WHERE d.project_id = '{pid}' AND d.status = 'signed' AND d.pdfa_offloaded_at IS NULL"
    )
    print(f"{a.du_an}: {len(docs):,} tài liệu đã ký, chưa chuyển PDF/A.")

    env = doc_env(REPO / "crates" / "digitext-api" / ".env")
    s3 = boto3.client("s3", endpoint_url=env["DIGITEXT_MINIO_ENDPOINT"], aws_access_key_id=env["DIGITEXT_MINIO_ACCESS_KEY"],
                      aws_secret_access_key=env["DIGITEXT_MINIO_SECRET_KEY"], region_name=env.get("DIGITEXT_MINIO_REGION", "us-east-1"))
    bucket = env.get("DIGITEXT_PDF_STORE_BUCKET") or env["DIGITEXT_MINIO_BUCKET"]
    kho: dict[str, tuple[int, str]] = {}
    for pg in s3.get_paginator("list_objects_v2").paginate(Bucket=bucket, Prefix=f"{pid}-warm/"):
        for o in pg.get("Contents", []):
            kho[o["Key"]] = (o["Size"], o["ETag"].strip('"'))
    print(f"Kho: {len(kho):,} đối tượng trong {pid}-warm/.")

    # Bản ký trên ổ ngoài: theo đường dẫn tương đối (gốc + thư mục lồng tên dự án) và theo (tên tệp, kích thước).
    theo_duong: dict[str, tuple[str, int]] = {}
    theo_ten: dict[tuple[str, int], list[str]] = defaultdict(list)
    for dp, _, fn in ([] if a.da_doi_chieu else os.walk(goc)):
        for f in fn:
            if not f.lower().endswith(".pdf"):
                continue
            p = os.path.join(dp, f)
            n = os.stat(p).st_size
            rel = os.path.relpath(p, goc).replace("\\", "/")
            theo_duong.setdefault(rel.lower(), (p, n))
            theo_ten[(f.lower(), n)].append(p)
    print(f"Ổ ngoài: {len(theo_duong):,} tệp PDF dưới {goc}.")
    long = goc.name.lower()

    duoc, bo = [], defaultdict(int)
    for did, rel, fn, pk, sk in docs:
        if not pk or pk not in kho:
            bo["PDF/A không còn trong kho"] += 1
            continue
        if not sk or sk not in kho:
            bo["bản ký không có trong kho"] += 1
            continue
        n_ky = kho[sk][0]
        phan = [x for x in rel.replace("\\", "/").split("/") if x.strip()]
        ung_vien = ["/".join(phan + [fn]), "/".join([long] + phan + [fn]), "/".join(phan[:3] + [fn]), "/".join([long] + phan[:3] + [fn])]
        tim = None
        for u in ung_vien:
            v = theo_duong.get(u.lower())
            if v and v[1] == n_ky:
                tim = v[0]
                break
        if a.da_doi_chieu:
            tim = f"(đã đối chiếu 16/10/2026 10:45) {goc}/{rel}/{fn}"
        if not tim:
            cung = theo_ten.get((fn.lower(), n_ky), [])
            if len(cung) == 1:
                tim = cung[0]
        if not tim:
            bo["bản ký trên ổ ngoài thiếu / sai kích thước"] += 1
            continue
        duoc.append((did, rel, fn, pk, kho[pk][0], kho[pk][1], sk, n_ky, tim))

    tong = sum(x[4] for x in duoc)
    print(f"Đủ điều kiện xoá: {len(duoc):,} PDF/A ({tong / 1e9:,.1f} GB). Bỏ qua: {dict(bo) or 0}")
    if not a.that:
        print("CHẠY THỬ — chưa xoá gì. Thêm --that để xoá.")
        return 0

    ten_ds = f"danh-sach-pdfa-da-xoa-{a.du_an}-{time.strftime('%Y%m%d-%H%M')}.csv"
    cot = ["document_id", "rel_path", "filename", "pdfa_key", "pdfa_bytes", "pdfa_etag_md5", "signed_key", "signed_bytes", "ban_ky_o_ngoai"]
    for noi in ([] if a.da_doi_chieu else [goc.parent / ten_ds]) + [REPO / "logs" / ten_ds]:
        with open(noi, "w", encoding="utf-8-sig", newline="") as f:
            w = csv.writer(f)
            w.writerow(cot)
            w.writerows(duoc)
        print(f"Danh sách: {noi}")

    ghi_chu = f"Bản ký số tại {goc}; danh sách {ten_ds}".replace("'", "''")
    for i in range(0, len(duoc), 1000):
        lo = duoc[i : i + 1000]
        s3.delete_objects(Bucket=bucket, Delete={"Objects": [{"Key": x[3]} for x in lo], "Quiet": True})
        ids = ",".join(f"'{x[0]}'" for x in lo)
        sql = f"SET lock_timeout = '5s'; UPDATE documents SET pdfa_offloaded_at = now(), pdfa_offload_note = '{ghi_chu}' WHERE id IN ({ids});"
        subprocess.run(PG, input=sql.encode("utf-8"), check=True)
        print(f"  đã xoá {min(i + 1000, len(duoc)):,}/{len(duoc):,}", flush=True)
    print("Xong.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
