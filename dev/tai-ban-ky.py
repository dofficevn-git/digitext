"""Tải bản ký số của tài liệu đã ký (chưa chuyển PDF/A) từ kho MinIO ra ổ ngoài, giữ cấu trúc <rel_path>/<tên tệp> — 24/10/2026.

    python -I dev/tai-ban-ky.py --du-an LTLSBN --dich "F:/2026/PDFa/LTLSBN · LTLSBNinh"
    python -I dev/tai-ban-ky.py --du-an LTLSBN --ban pdfa --dich "F:/2026/PDFa/LTLSBN · LTLSBNinh chua ky"   ← bản PDF/A chưa ký

Ghi <tệp>.part rồi đổi tên; tệp đã có đúng kích thước thì bỏ qua; có sẵn mà khác kích thước thì đổi tên cũ thành .cu.
Xong chạy dev/xoa-pdfa-da-ky.py (cùng --ban-ky) để xoá PDF/A chưa ký khỏi kho.
"""
from __future__ import annotations

import argparse
import csv
import io
import os
import re
import subprocess
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor
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
    ap.add_argument("--dich", required=True)
    ap.add_argument("--luong", type=int, default=6)
    ap.add_argument("--ban", choices=["ky", "pdfa"], default="ky", help="ky: bản ký số; pdfa: bản PDF/A chưa ký")
    ap.add_argument("--ca-da-chuyen", action="store_true", help="--ban ky: lấy cả tài liệu đã chuyển PDF/A ra ổ ngoài (pdfa_offloaded_at có giá trị)")
    ap.add_argument("--ca-chua-ky", action="store_true", help="--ban pdfa: lấy cả tài liệu CHƯA ký (chỉ sao lưu, không xoá)")
    a = ap.parse_args()
    dich = Path(a.dich)
    if not dich.is_dir():
        print(f"Không thấy {dich}")
        return 2
    ma = a.du_an.replace("'", "''")
    pid = psql_csv(f"SELECT id FROM projects WHERE code = '{ma}'")[0][0]
    docs = psql_csv(
        "SELECT COALESCE(d.rel_path,''), d.filename, "
        + ("d.pdfa_key " if a.ban == "pdfa" else "(SELECT s.signed_key FROM sign_sessions s WHERE s.document_id = d.id "
           "AND s.status = 'signed' AND s.signed_key IS NOT NULL ORDER BY s.updated_at DESC LIMIT 1) ")
        + 
        f"FROM documents d WHERE d.project_id = '{pid}' "
        + ("" if a.ban == "ky" and a.ca_da_chuyen else "AND d.pdfa_offloaded_at IS NULL ")
        + ("AND d.pdfa_key IS NOT NULL" if a.ban == "pdfa" and a.ca_chua_ky else "AND d.status = 'signed'")
    )
    docs = [x for x in docs if x[2]]
    env = doc_env(REPO / "crates" / "digitext-api" / ".env")
    s3 = boto3.client("s3", endpoint_url=env["DIGITEXT_MINIO_ENDPOINT"], aws_access_key_id=env["DIGITEXT_MINIO_ACCESS_KEY"],
                      aws_secret_access_key=env["DIGITEXT_MINIO_SECRET_KEY"], region_name=env.get("DIGITEXT_MINIO_REGION", "us-east-1"),
                      config=Config(max_pool_connections=a.luong * 2, retries={"max_attempts": 5}))
    bucket = env.get("DIGITEXT_PDF_STORE_BUCKET") or env["DIGITEXT_MINIO_BUCKET"]
    print(f"{a.du_an}: {len(docs):,} {'bản PDF/A chưa ký' if a.ban == 'pdfa' else 'bản ký'} cần có trên {dich}", flush=True)

    khoa = threading.Lock()
    dem = {"xong": 0, "bo_qua": 0, "loi": 0, "byte": 0, "cu": 0}
    loi: list[str] = []
    t0 = time.time()

    def mot(x: list[str]) -> None:
        rel, fn, sk = x
        phan = [p for p in rel.replace("\\", "/").split("/") if p.strip()]
        dst = dich.joinpath(*phan, fn)
        try:
            n = s3.head_object(Bucket=bucket, Key=sk)["ContentLength"]
            if dst.exists():
                if dst.stat().st_size == n:
                    with khoa:
                        dem["bo_qua"] += 1
                    return
                os.replace(dst, dst.with_name(dst.name + ".cu"))
                with khoa:
                    dem["cu"] += 1
            dst.parent.mkdir(parents=True, exist_ok=True)
            tam = dst.with_name(dst.name + ".part")
            s3.download_file(bucket, sk, str(tam))
            if tam.stat().st_size != n:
                raise IOError(f"sai kích thước {tam.stat().st_size} ≠ {n}")
            os.replace(tam, dst)
            with khoa:
                dem["xong"] += 1
                dem["byte"] += n
        except Exception as e:  # noqa: BLE001
            with khoa:
                dem["loi"] += 1
                loi.append(f"{rel}/{fn}\t{e}")

    with ThreadPoolExecutor(a.luong) as ex:
        for i, _ in enumerate(ex.map(mot, docs), 1):
            if i % 500 == 0 or i == len(docs):
                dt = time.time() - t0
                print(f"  {i:,}/{len(docs):,}  tải {dem['xong']:,} ({dem['byte'] / 1e9:,.1f} GB, {dem['byte'] / 1e6 / max(dt, 1):.0f} MB/s)"
                      f"  có sẵn {dem['bo_qua']:,}  lỗi {dem['loi']:,}  {dt / 60:.0f} phút", flush=True)
    if loi:
        p = REPO / "logs" / f"tai-ban-ky-loi-{time.strftime('%Y%m%d-%H%M')}.txt"
        p.write_text("\n".join(loi), encoding="utf-8")
        print(f"Lỗi: {len(loi):,} — {p}")
    print(f"Xong: tải {dem['xong']:,}, có sẵn {dem['bo_qua']:,}, đổi tên tệp cũ khác cỡ {dem['cu']:,}, lỗi {dem['loi']:,}.")
    return 1 if loi else 0


if __name__ == "__main__":
    sys.exit(main())
