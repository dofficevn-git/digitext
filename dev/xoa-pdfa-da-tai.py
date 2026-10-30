"""Xoá cuốn chiếu bản PDF/A (chưa ký) của tài liệu ĐÃ KÝ khi bản PDF/A đã tải ra ổ ngoài — anh Hiếu chốt 24/10/2026.

    python -I dev/xoa-pdfa-da-tai.py --du-an LTLSBN --ban-pdfa "F:/2026/PDFa/LTLSBN · LTLSBNinh chua ky"            ← CHẠY THỬ
    python -I dev/xoa-pdfa-da-tai.py --du-an LTLSBN --ban-pdfa "F:/2026/PDFa/LTLSBN · LTLSBNinh chua ky" --that --lap 120

Chạy song song với dev/tai-ban-ky.py --ban pdfa: mỗi vòng (--lap giây) xoá các PDF/A thoả TẤT CẢ:
  - tài liệu đã ký (status = signed), bản ký số còn trong kho;
  - <ban-pdfa>/<rel_path>/<tên tệp> có trên ổ ngoài, ĐÚNG kích thước và MD5 = ETag của PDF/A trong kho
    (ETag nhiều phần thì chỉ so kích thước).
Ghi danh sách vào logs/danh-sach-pdfa-da-xoa-<mã>-<ngày>.csv, đánh dấu documents.pdfa_offloaded_at (migration 0019).
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import io
import re
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import boto3
from botocore.config import Config

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


def md5_tep(p: Path) -> str:
    h = hashlib.md5()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(8 << 20), b""):
            h.update(b)
    return h.hexdigest()


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--du-an", required=True)
    ap.add_argument("--ban-pdfa", required=True, help="thư mục bản PDF/A chưa ký trên ổ ngoài")
    ap.add_argument("--that", action="store_true", help="xoá thật (mặc định chỉ chạy thử)")
    ap.add_argument("--lap", type=int, default=0, help="chạy lặp mỗi N giây tới khi hết tài liệu (0 = một vòng)")
    ap.add_argument("--luong", type=int, default=4, help="số luồng kiểm (MD5 + HEAD) song song")
    ap.add_argument("--nghi-khi-ranh", type=int, default=45, help="--lap: dừng sau N phút liền không xoá được tệp nào")
    a = ap.parse_args()
    goc = Path(a.ban_pdfa)
    if not goc.is_dir():
        print(f"Không thấy {goc}")
        return 2
    ma = a.du_an.replace("'", "''")
    pid = psql_csv(f"SELECT id FROM projects WHERE code = '{ma}'")[0][0]
    env = doc_env(REPO / "crates" / "digitext-api" / ".env")
    s3 = boto3.client("s3", endpoint_url=env["DIGITEXT_MINIO_ENDPOINT"], aws_access_key_id=env["DIGITEXT_MINIO_ACCESS_KEY"],
                      aws_secret_access_key=env["DIGITEXT_MINIO_SECRET_KEY"], region_name=env.get("DIGITEXT_MINIO_REGION", "us-east-1"),
                      config=Config(max_pool_connections=16))
    bucket = env.get("DIGITEXT_PDF_STORE_BUCKET") or env["DIGITEXT_MINIO_BUCKET"]
    ten_ds = REPO / "logs" / f"danh-sach-pdfa-da-xoa-{a.du_an}-{time.strftime('%Y%m%d-%H%M')}.csv"
    ghi_chu = f"Bản PDF/A chưa ký tại {goc}; danh sách {ten_ds.name}".replace("'", "''")
    tong_xoa = tong_byte = 0
    ranh_tu = time.time()

    while True:
        docs = psql_csv(
            "SELECT d.id, COALESCE(d.rel_path,''), d.filename, d.pdfa_key, "
            "(SELECT s.signed_key FROM sign_sessions s WHERE s.document_id = d.id AND s.status = 'signed' "
            "AND s.signed_key IS NOT NULL ORDER BY s.updated_at DESC LIMIT 1) "
            f"FROM documents d WHERE d.project_id = '{pid}' AND d.status = 'signed' AND d.pdfa_key IS NOT NULL "
            "AND d.pdfa_offloaded_at IS NULL"
        )
        duoc = []
        cho = 0
        vong_xoa = vong_byte = 0

        def xoa_lo(lo: list) -> None:
            nonlocal tong_xoa, tong_byte, vong_xoa, vong_byte, ranh_tu
            if not lo or not a.that:
                return
            moi = not ten_ds.exists()
            with open(ten_ds, "a", encoding="utf-8-sig", newline="") as f:
                w = csv.writer(f)
                if moi:
                    w.writerow(["document_id", "rel_path", "filename", "pdfa_key", "pdfa_bytes", "pdfa_etag_md5", "signed_key", "ban_pdfa_o_ngoai"])
                w.writerows(lo)
            s3.delete_objects(Bucket=bucket, Delete={"Objects": [{"Key": x[3]} for x in lo], "Quiet": True})
            ids = ",".join(f"'{x[0]}'" for x in lo)
            sql = f"SET lock_timeout = '5s'; UPDATE documents SET pdfa_offloaded_at = now(), pdfa_offload_note = '{ghi_chu}' WHERE id IN ({ids});"
            subprocess.run(PG, input=sql.encode("utf-8"), check=True)
            n = sum(x[4] for x in lo)
            tong_xoa += len(lo)
            tong_byte += n
            vong_xoa += len(lo)
            vong_byte += n
            ranh_tu = time.time()
            print(f"  {time.strftime('%H:%M:%S')}  xoá {len(lo):,} — cộng dồn {tong_xoa:,} ({tong_byte / 1e9:,.1f} GB)", flush=True)

        def kiem(x: list[str]) -> tuple | None:
            did, rel, fn, pk, sk = x
            phan = [p for p in rel.replace("\\", "/").split("/") if p.strip()]
            tep = goc.joinpath(*phan, fn)
            if not sk or not tep.is_file():
                return None
            try:
                h_pa = s3.head_object(Bucket=bucket, Key=pk)
                s3.head_object(Bucket=bucket, Key=sk)
            except Exception:  # noqa: BLE001
                return None
            n = h_pa["ContentLength"]
            etag = h_pa["ETag"].strip('"')
            if tep.stat().st_size != n:
                return None
            try:
                md5 = md5_tep(tep) if "-" not in etag else ""
            except OSError:
                return None
            if md5 and md5 != etag:
                print(f"  MD5 KHÁC kho — không xoá: {tep}", flush=True)
                return None
            return (did, rel, fn, pk, n, etag, sk, str(tep))

        with ThreadPoolExecutor(a.luong) as ex:
            for r in ex.map(kiem, docs):
                if r is None:
                    cho += 1
                    continue
                duoc.append(r)
                if len(duoc) >= 200 and a.that:
                    xoa_lo(duoc)
                    duoc = []
        if a.that:
            xoa_lo(duoc)
            duoc = []
        print(f"{time.strftime('%H:%M:%S')}  vòng xong: {'xoá ' + format(vong_xoa, ',') if a.that else 'đủ điều kiện ' + format(len(duoc), ',')}"
              f"  — cộng dồn {tong_xoa:,} ({tong_byte / 1e9:,.1f} GB), còn chờ {cho:,}", flush=True)
        if not a.that or not a.lap or cho == 0:
            break
        if time.time() - ranh_tu > a.nghi_khi_ranh * 60:
            print(f"{a.nghi_khi_ranh} phút không có tệp mới — dừng.")
            break
        time.sleep(a.lap)
    if a.that and tong_xoa:
        print(f"Danh sách: {ten_ds}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
