"""Tải bù bản KÝ SỐ còn thiếu / lỗi ra ổ ngoài, đúng cây Phông/Hộp/Hồ sơ (16/10/2026: lượt tải trên app lỗi ~130 tệp
"lỗi hệ thống" sau khi đổi CSDL). Chỉ ĐỌC kho, chỉ GHI tệp còn thiếu — tệp đã có đúng kích thước thì bỏ qua.

    python -I dev/tai-bu-ban-ky.py --du-an LTLSBN --ra "F:/2026/PDFa/LTLSBN · LTLSBNinh" [--chay-thu]

Dò tệp đã có giống dev/xoa-pdfa-da-ky.py (cả thư mục lồng tên dự án và đường dẫn bị app rút còn 3 cấp).
Tệp mới ghi vào <ra>/<rel_path>/<tên>, ghi .part rồi đổi tên, kiểm kích thước.
"""
from __future__ import annotations

import argparse
import csv
import io
import os
import re
import subprocess
import sys
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import boto3

REPO = Path(__file__).resolve().parent.parent
PG = ["docker", "exec", "-i", "digitext-dev-postgres-1", "psql", "-U", "digitext", "-d", "digitext", "-v", "ON_ERROR_STOP=1", "-q"]
KY_TU_CAM = re.compile(r'[<>:"|?*\x00-\x1f]')


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


def an_toan(phan: str) -> str:
    return KY_TU_CAM.sub("_", phan).rstrip(" .") or "_"


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--du-an", required=True)
    ap.add_argument("--ra", required=True)
    ap.add_argument("--chay-thu", action="store_true")
    ap.add_argument("--luong", type=int, default=4)
    a = ap.parse_args()
    goc = Path(a.ra)
    if not goc.is_dir():
        print(f"Không thấy {goc}")
        return 2
    ma = a.du_an.replace("'", "''")
    pid = psql_csv(f"SELECT id FROM projects WHERE code = '{ma}'")[0][0]
    docs = psql_csv(
        "SELECT d.id, COALESCE(d.rel_path,''), d.filename, "
        "COALESCE((SELECT s.signed_key FROM sign_sessions s WHERE s.document_id = d.id AND s.status = 'signed' "
        "AND s.signed_key IS NOT NULL ORDER BY s.updated_at DESC LIMIT 1), '') "
        f"FROM documents d WHERE d.project_id = '{pid}' AND d.status = 'signed'"
    )
    env = doc_env(REPO / "crates" / "digitext-api" / ".env")
    s3 = boto3.client("s3", endpoint_url=env["DIGITEXT_MINIO_ENDPOINT"], aws_access_key_id=env["DIGITEXT_MINIO_ACCESS_KEY"],
                      aws_secret_access_key=env["DIGITEXT_MINIO_SECRET_KEY"], region_name=env.get("DIGITEXT_MINIO_REGION", "us-east-1"))
    bucket = env.get("DIGITEXT_PDF_STORE_BUCKET") or env["DIGITEXT_MINIO_BUCKET"]
    kho: dict[str, int] = {}
    for pg in s3.get_paginator("list_objects_v2").paginate(Bucket=bucket, Prefix=f"{pid}-warm/"):
        for o in pg.get("Contents", []):
            if o["Key"].endswith(".signed.pdf"):
                kho[o["Key"]] = o["Size"]

    theo_duong: dict[str, int] = {}
    theo_ten: dict[tuple[str, int], int] = defaultdict(int)
    for dp, _, fn in os.walk(goc):
        for f in fn:
            if f.lower().endswith(".pdf"):
                p = os.path.join(dp, f)
                n = os.stat(p).st_size
                theo_duong[os.path.relpath(p, goc).replace("\\", "/").lower()] = n
                theo_ten[(f.lower(), n)] += 1
    long = goc.name.lower()

    viec = []
    for did, rel, fn, sk in docs:
        if sk not in kho:
            continue
        n = kho[sk]
        phan = [x for x in rel.replace("\\", "/").split("/") if x.strip()]
        ung_vien = ["/".join(phan + [fn]), "/".join([long] + phan + [fn]), "/".join(phan[:3] + [fn]), "/".join([long] + phan[:3] + [fn])]
        if any(theo_duong.get(u.lower()) == n for u in ung_vien) or theo_ten.get((fn.lower(), n), 0) == 1:
            continue
        dich = goc.joinpath(*[an_toan(x.strip()) for x in phan], an_toan(fn))
        viec.append((sk, n, dich))
    print(f"{a.du_an}: {len(docs):,} tài liệu đã ký; trên ổ ngoài {len(theo_duong):,} tệp; CẦN TẢI BÙ {len(viec):,} tệp "
          f"({sum(v[1] for v in viec) / 1e9:,.2f} GB).")
    for sk, n, dich in viec[:10]:
        print("  ", dich)
    if a.chay_thu or not viec:
        return 0

    def mot(sk: str, n: int, dich: Path) -> str | None:
        try:
            dich.parent.mkdir(parents=True, exist_ok=True)
            tam = dich.with_name(dich.name + ".part")
            s3.download_file(bucket, sk, str(tam))
            if os.stat(tam).st_size != n:
                return f"{dich}: sai kích thước"
            os.replace(tam, dich)
            return None
        except Exception as e:  # noqa: BLE001
            return f"{dich}: {type(e).__name__}: {e}"

    loi = []
    with ThreadPoolExecutor(a.luong) as ex:
        for i, f in enumerate(as_completed([ex.submit(mot, *v) for v in viec]), 1):
            if (r := f.result()):
                loi.append(r)
            if i % 50 == 0 or i == len(viec):
                print(f"  {i:,}/{len(viec):,} · lỗi {len(loi)}", flush=True)
    for r in loi[:20]:
        print("LỖI", r)
    print(f"Xong: tải bù {len(viec) - len(loi):,}, lỗi {len(loi)}.")
    return 1 if loi else 0


if __name__ == "__main__":
    sys.exit(main())
