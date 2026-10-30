"""Chuyển bản ký số sang kho MinIO mới từ thư mục đã tải ra ổ ngoài (dev/tai-ban-ky.py) — không chép lại 800 GB.
Chuẩn bị chuyển máy chủ sang OCR PC, 30/10/2026.

  Máy cũ — xuất bảng ánh xạ tài liệu → khoá kho (cần Docker Postgres):
    python -I dev/nap-ban-ky.py xuat --du-an LTLSBN --bang F:/DigiText-ChuyenMay/ban-ky-LTLSBN.csv
  Máy mới — nạp từ thư mục vào MinIO (bỏ qua khoá đã có đúng kích thước; chạy lại được):
    python -I dev/nap-ban-ky.py nap --bang F:/DigiText-ChuyenMay/ban-ky-LTLSBN.csv \
        --thu-muc "F:/2026/PDFa/LTLSBN · LTLSBNinh" --dich http://127.0.0.1:9000
  Kiểm sau nạp (số đối tượng + tổng byte theo khoá, 300 mẫu MD5 so với tệp trên ổ):
    python -I dev/nap-ban-ky.py kiem --bang ... --thu-muc ... --dich http://127.0.0.1:9000

Bảng CSV: document_id,rel_path,filename,signed_key,size. Tệp trên ổ ở <thu-muc>/<rel_path>/<filename>.
Tải lên bằng put_object một phần (không multipart) để ETag = MD5 như kho cũ. Khoá truy cập: crates/digitext-api/.env.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import io
import random
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


def client(url: str, luong: int):
    env = doc_env(REPO / "crates" / "digitext-api" / ".env")
    cfg = Config(retries={"max_attempts": 10, "mode": "standard"}, read_timeout=300, connect_timeout=20, max_pool_connections=luong * 2)
    s3 = boto3.client("s3", endpoint_url=url, aws_access_key_id=env["DIGITEXT_MINIO_ACCESS_KEY"],
                      aws_secret_access_key=env["DIGITEXT_MINIO_SECRET_KEY"], region_name=env.get("DIGITEXT_MINIO_REGION", "us-east-1"), config=cfg)
    return s3, (env.get("DIGITEXT_PDF_STORE_BUCKET") or env["DIGITEXT_MINIO_BUCKET"])


def doc_bang(p: Path) -> list[dict]:
    with open(p, encoding="utf-8", newline="") as f:
        rows = list(csv.DictReader(f))
    for r in rows:
        r["size"] = int(r["size"])
    return rows


def tep_cua(goc: Path, r: dict) -> Path:
    phan = [p for p in r["rel_path"].replace("\\", "/").split("/") if p.strip()]
    return goc.joinpath(*phan, r["filename"])


def liet_ke(s3, bucket: str, prefix: str) -> dict[str, int]:
    co = {}
    for pg in s3.get_paginator("list_objects_v2").paginate(Bucket=bucket, Prefix=prefix):
        for o in pg.get("Contents", []):
            co[o["Key"]] = o["Size"]
    return co


def xuat(a) -> int:
    ma = a.du_an.replace("'", "''")
    pid = psql_csv(f"SELECT id FROM projects WHERE code = '{ma}'")[0][0]
    rows = psql_csv(
        "SELECT d.id, COALESCE(d.rel_path,''), d.filename, s.signed_key "
        "FROM documents d JOIN LATERAL (SELECT signed_key FROM sign_sessions s "
        "WHERE s.document_id = d.id AND s.status = 'signed' AND s.signed_key IS NOT NULL ORDER BY s.updated_at DESC LIMIT 1) s ON true "
        f"WHERE d.project_id = '{pid}' AND d.status = 'signed' ORDER BY d.rel_path, d.filename"
    )
    # Kích thước lấy từ kho (HEAD) — bảng sign_sessions không lưu cỡ bản ký.
    env = doc_env(REPO / "crates" / "digitext-api" / ".env")
    s3, bucket = client(env["DIGITEXT_MINIO_ENDPOINT"], 4)
    kho = liet_ke(s3, bucket, f"{pid}-warm/")
    out = Path(a.bang)
    out.parent.mkdir(parents=True, exist_ok=True)
    thieu = 0
    with open(out, "w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(["document_id", "rel_path", "filename", "signed_key", "size"])
        for did, rel, fn, sk in rows:
            n = kho.get(sk)
            if n is None:
                thieu += 1
                continue
            w.writerow([did, rel, fn, sk, n])
    print(f"{a.du_an}: {len(rows) - thieu:,} bản ký → {out}; không có trong kho: {thieu}")
    return 1 if thieu else 0


def nap(a) -> int:
    goc = Path(a.thu_muc)
    rows = doc_bang(Path(a.bang))
    s3, bucket = client(a.dich, a.luong)
    try:
        s3.head_bucket(Bucket=bucket)
    except Exception:  # noqa: BLE001
        s3.create_bucket(Bucket=bucket)
    prefixes = {r["signed_key"].split("/", 1)[0] + "/" for r in rows}
    co: dict[str, int] = {}
    for p in prefixes:
        co.update(liet_ke(s3, bucket, p))
    can = [r for r in rows if co.get(r["signed_key"]) != r["size"]]
    thieu_tep = [r for r in can if not tep_cua(goc, r).is_file() or tep_cua(goc, r).stat().st_size != r["size"]]
    if thieu_tep:
        for r in thieu_tep[:10]:
            print("THIẾU/SAI CỠ trên ổ:", tep_cua(goc, r))
        print(f"{len(thieu_tep):,} tệp thiếu hoặc sai kích thước trên {goc} — không nạp gì.")
        return 2
    tong = sum(r["size"] for r in can)
    print(f"Bảng {len(rows):,} bản ký; đích đã có đúng {len(rows) - len(can):,}; cần nạp {len(can):,} ({tong / 1e9:,.1f} GB).", flush=True)
    if not can:
        return 0
    khoa = threading.Lock()
    dem = {"xong": 0, "byte": 0}
    loi: list[str] = []
    t0 = time.time()

    def day(r: dict) -> None:
        p = tep_cua(goc, r)
        for lan in range(1, 9):
            try:
                with open(p, "rb") as f:
                    body = f.read()
                if len(body) != r["size"]:
                    raise IOError(f"đọc cụt {len(body)}/{r['size']}")
                md5 = hashlib.md5(body).hexdigest()
                et = s3.put_object(Bucket=bucket, Key=r["signed_key"], Body=body, ContentType="application/pdf")["ETag"].strip('"')
                if et != md5:
                    raise IOError(f"ETag {et} ≠ MD5 {md5}")
                with khoa:
                    dem["xong"] += 1
                    dem["byte"] += len(body)
                return
            except Exception as e:  # noqa: BLE001
                if lan == 8:
                    loi.append(f"{r['signed_key']}\t{p}\t{e}")
                    return
                time.sleep(min(30, 2 * lan))

    with ThreadPoolExecutor(a.luong) as ex:
        for i, _ in enumerate(ex.map(day, can), 1):
            if i % 1000 == 0 or i == len(can):
                dt = max(time.time() - t0, 1)
                print(f"{time.strftime('%H:%M:%S')}  {i:,}/{len(can):,} · {dem['byte'] / 1e9:,.1f}/{tong / 1e9:,.1f} GB · "
                      f"{dem['byte'] / 1e6 / dt:,.1f} MB/s · lỗi {len(loi)} · còn ~{(tong - dem['byte']) / max(dem['byte'] / dt, 1) / 3600:,.1f} giờ", flush=True)
    if loi:
        p = REPO / "logs" / f"nap-ban-ky-loi-{time.strftime('%Y%m%d-%H%M')}.tsv"
        p.parent.mkdir(exist_ok=True)
        p.write_text("\n".join(loi), encoding="utf-8")
        print(f"Lỗi {len(loi):,} — {p}")
    print(f"XONG nạp: {dem['xong']:,} tệp, {dem['byte'] / 1e9:,.1f} GB, lỗi {len(loi)}.", flush=True)
    return 1 if loi else 0


def kiem(a) -> int:
    goc = Path(a.thu_muc)
    rows = doc_bang(Path(a.bang))
    s3, bucket = client(a.dich, a.luong)
    prefixes = {r["signed_key"].split("/", 1)[0] + "/" for r in rows}
    co: dict[str, int] = {}
    for p in prefixes:
        co.update(liet_ke(s3, bucket, p))
    sai = [r for r in rows if co.get(r["signed_key"]) != r["size"]]
    tong_bang = sum(r["size"] for r in rows)
    tong_kho = sum(co[r["signed_key"]] for r in rows if r["signed_key"] in co)
    print(f"Bảng {len(rows):,} khoá / {tong_bang / 1e9:,.1f} GB; kho khớp {len(rows) - len(sai):,} / {tong_kho / 1e9:,.1f} GB; sai/thiếu {len(sai):,}")
    for r in sai[:10]:
        print("  ", r["signed_key"], co.get(r["signed_key"]), "≠", r["size"])
    mau = random.sample(rows, min(a.mau, len(rows)))
    lech = 0
    for r in mau:
        et = s3.head_object(Bucket=bucket, Key=r["signed_key"])["ETag"].strip('"')
        md5 = hashlib.md5(tep_cua(goc, r).read_bytes()).hexdigest()
        if et != md5:
            lech += 1
            print("  MD5 lệch:", r["signed_key"])
    print(f"Mẫu {len(mau)} tệp: MD5 lệch {lech}")
    return 1 if sai or lech else 0


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("viec", choices=["xuat", "nap", "kiem"])
    ap.add_argument("--du-an")
    ap.add_argument("--bang", required=True)
    ap.add_argument("--thu-muc")
    ap.add_argument("--dich")
    ap.add_argument("--luong", type=int, default=6)
    ap.add_argument("--mau", type=int, default=300)
    a = ap.parse_args()
    if a.viec == "xuat" and not a.du_an:
        ap.error("xuat cần --du-an")
    if a.viec in ("nap", "kiem") and not (a.thu_muc and a.dich):
        ap.error(f"{a.viec} cần --thu-muc và --dich")
    return {"xuat": xuat, "nap": nap, "kiem": kiem}[a.viec](a)


if __name__ == "__main__":
    sys.exit(main())
