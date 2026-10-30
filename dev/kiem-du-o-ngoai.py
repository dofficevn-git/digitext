"""Đối chiếu TỪNG tài liệu của dự án với bản sao trên ổ ngoài — 27/10/2026.

    python -I dev/kiem-du-o-ngoai.py --du-an LTLSBN --ban-ky "F:/2026/PDFa/LTLSBN · LTLSBNinh" \
        --ban-pdfa "F:/2026/PDFa/LTLSBN · LTLSBNinh chua ky" --kho-nguon "D:/DigiText-Kho/LTLSBN · LTLSBNinh" [--bo-sung]

- Tài liệu đã ký: <ban-ky>/<rel_path>/<tên> đúng kích thước bản ký trong kho.
- Mọi tài liệu: <ban-pdfa>/<rel_path>/<tên> đúng kích thước PDF/A (trong kho; PDF/A đã chuyển ra ngoài thì theo các
  logs/danh-sach-pdfa-da-xoa-*.csv); tài liệu không có PDF/A thì đúng kích thước bản gốc trong kho nguồn.
--bo-sung: tải/chép tệp còn thiếu (ghi .part rồi đổi tên). Tệp khác kích thước KHÔNG ghi đè — chỉ báo.
Danh sách thiếu/sai ghi ra logs/kiem-du-o-ngoai-<mã>-<ngày>.tsv.
"""
from __future__ import annotations

import argparse
import csv
import glob
import io
import os
import re
import shutil
import subprocess
import sys
import time
from collections import Counter
from pathlib import Path

import boto3
from botocore.config import Config

REPO = Path(__file__).resolve().parent.parent
PG = ["docker", "exec", "-i", "digitext-dev-postgres-1", "psql", "-U", "digitext", "-d", "digitext", "-q"]


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
    ap.add_argument("--ban-ky", required=True)
    ap.add_argument("--ban-pdfa", required=True)
    ap.add_argument("--kho-nguon", required=True)
    ap.add_argument("--bo-sung", action="store_true")
    a = ap.parse_args()
    ky_dir, pa_dir, nguon = Path(a.ban_ky), Path(a.ban_pdfa), Path(a.kho_nguon)
    for d in (ky_dir, pa_dir, nguon):
        if not d.is_dir():
            print(f"Không thấy {d}")
            return 2
    ma = a.du_an.replace("'", "''")
    pid = psql_csv(f"SELECT id FROM projects WHERE code = '{ma}'")[0][0]
    docs = psql_csv(
        "SELECT d.id, COALESCE(d.rel_path,''), d.filename, d.status, d.size_bytes, COALESCE(d.pdfa_key,''), "
        "COALESCE((SELECT s.signed_key FROM sign_sessions s WHERE s.document_id = d.id AND s.status = 'signed' "
        "AND s.signed_key IS NOT NULL ORDER BY s.updated_at DESC LIMIT 1),'') "
        f"FROM documents d WHERE d.project_id = '{pid}'"
    )
    env = doc_env(REPO / "crates" / "digitext-api" / ".env")
    s3 = boto3.client("s3", endpoint_url=env["DIGITEXT_MINIO_ENDPOINT"], aws_access_key_id=env["DIGITEXT_MINIO_ACCESS_KEY"],
                      aws_secret_access_key=env["DIGITEXT_MINIO_SECRET_KEY"], region_name=env.get("DIGITEXT_MINIO_REGION", "us-east-1"),
                      config=Config(read_timeout=300, retries={"max_attempts": 10}))
    bucket = env.get("DIGITEXT_PDF_STORE_BUCKET") or env["DIGITEXT_MINIO_BUCKET"]
    kho: dict[str, int] = {}
    for pg in s3.get_paginator("list_objects_v2").paginate(Bucket=bucket, Prefix=f"{pid}-warm/"):
        for o in pg.get("Contents", []):
            kho[o["Key"]] = o["Size"]
    da_xoa: dict[str, int] = {}
    for f in glob.glob(str(REPO / "logs" / f"danh-sach-pdfa-da-xoa-{a.du_an}-*.csv")):
        with open(f, encoding="utf-8-sig") as fh:
            for r in csv.DictReader(fh):
                if r.get("pdfa_key") and r.get("pdfa_bytes"):
                    da_xoa[r["pdfa_key"]] = int(r["pdfa_bytes"])
    print(f"{a.du_an}: {len(docs):,} tài liệu; kho {len(kho):,} đối tượng; danh sách đã chuyển {len(da_xoa):,} PDF/A.", flush=True)

    dem: Counter = Counter()
    van_de: list[tuple[str, str, str]] = []

    def can(dst: Path, n: int | None, nguon_tai, nhan: str, rel: str) -> None:
        if n is None:
            dem[f"{nhan}: không biết kích thước chuẩn"] += 1
            van_de.append((nhan, rel, "không biết kích thước chuẩn (không còn trong kho, không có trong danh sách)"))
            return
        if dst.is_file():
            if dst.stat().st_size == n:
                dem[f"{nhan}: đủ"] += 1
                return
            dem[f"{nhan}: SAI kích thước"] += 1
            van_de.append((nhan, rel, f"trên ổ {dst.stat().st_size} ≠ chuẩn {n}"))
            return
        if not a.bo_sung or nguon_tai is None:
            dem[f"{nhan}: THIẾU"] += 1
            van_de.append((nhan, rel, "thiếu"))
            return
        try:
            dst.parent.mkdir(parents=True, exist_ok=True)
            tam = dst.with_name(dst.name + ".part")
            nguon_tai(tam)
            if tam.stat().st_size != n:
                raise IOError(f"sai kích thước {tam.stat().st_size} ≠ {n}")
            os.replace(tam, dst)
            dem[f"{nhan}: vừa bổ sung"] += 1
        except Exception as e:  # noqa: BLE001
            dem[f"{nhan}: THIẾU (bổ sung lỗi)"] += 1
            van_de.append((nhan, rel, f"bổ sung lỗi: {e}"))

    for i, (did, rel, fn, st, sz, pk, sk) in enumerate(docs, 1):
        phan = [x for x in rel.replace("\\", "/").split("/") if x.strip()]
        ten = "/".join(phan + [fn])
        if st == "signed":
            n = kho.get(sk) if sk else None
            can(ky_dir.joinpath(*phan, fn), n, (lambda t, k=sk: s3.download_file(bucket, k, str(t))) if n is not None else None, "bản ký", ten)
        if pk:
            if pk in kho:
                can(pa_dir.joinpath(*phan, fn), kho[pk], lambda t, k=pk: s3.download_file(bucket, k, str(t)), "PDF/A", ten)
            else:
                can(pa_dir.joinpath(*phan, fn), da_xoa.get(pk), None, "PDF/A", ten)
        else:
            src = nguon.joinpath(*phan, fn)
            n = src.stat().st_size if src.is_file() else None
            can(pa_dir.joinpath(*phan, fn), n, (lambda t, s=src: shutil.copyfile(s, t)) if n is not None else None, "bản gốc (không có PDF/A)", ten)
        if i % 20000 == 0:
            print(f"  {i:,}/{len(docs):,}  {dict(dem)}", flush=True)

    print("KẾT QUẢ:")
    for k, v in sorted(dem.items()):
        print(f"  {k}: {v:,}")
    if van_de:
        p = REPO / "logs" / f"kiem-du-o-ngoai-{a.du_an}-{time.strftime('%Y%m%d-%H%M')}.tsv"
        p.write_text("\n".join("\t".join(x) for x in van_de), encoding="utf-8")
        print(f"Vấn đề: {len(van_de):,} — {p}")
        for x in van_de[:15]:
            print("  ", *x)
    else:
        print("Không còn tài liệu nào thiếu hoặc sai trên ổ ngoài.")
    return 1 if van_de else 0


if __name__ == "__main__":
    sys.exit(main())
