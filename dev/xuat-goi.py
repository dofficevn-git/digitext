"""Xuất gói tin đã dựng từ MinIO trên máy chủ ra một thư mục (ổ cắm trực tiếp vào máy chủ).

Tải qua app phải đi vòng ra Internet rồi quay lại (~3 MB/s, 70 GB mất hơn 12 giờ). Chạy script này
ngay trên OCR PC thì đọc MinIO qua 127.0.0.1 và ghi thẳng ra ổ cắm, chỉ còn giới hạn của hai ổ đĩa.

    python -I dev/xuat-goi.py --thu-muc F:/Goi-AIP [--du-an <mã hoặc uuid>] [--loai aip_hoso] [--luong 4]

Tên tệp lấy đúng `zip_name` trong CSDL (urn_<mã phông>_uuid_<...>.zip). Chạy lại được: tệp đã có
đúng kích thước thì bỏ qua, nên cắm lại ổ chạy tiếp cũng không mất công.
Khoá MinIO đọc từ crates/digitext-api/.env; danh sách gói lấy qua `docker exec … psql` nên không
cần chuỗi kết nối CSDL.
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
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import boto3
from botocore.config import Config

REPO = Path(__file__).resolve().parent.parent
CONTAINER = "digitext-dev-postgres-1"


def doc_env(p: Path) -> dict[str, str]:
    kv = {}
    for line in p.read_text(encoding="utf-8").splitlines():
        m = re.match(r"\s*([A-Z0-9_]+)\s*=\s*(.*?)\s*$", line)
        if m:
            kv[m.group(1)] = m.group(2).strip('"')
    return kv


def danh_sach_goi(du_an: str | None, loai: str | None) -> list[tuple[str, str, str, int]]:
    """(project_id, package_id, zip_name, zip_size) của các gói đã dựng xong."""
    dk = ["status = 'built'", "zip_name IS NOT NULL"]
    if du_an:
        dk.append(
            "(project_id::text = '{0}' OR project_id IN (SELECT id FROM projects WHERE code = '{0}'))".format(
                du_an.replace("'", "''")
            )
        )
    if loai:
        dk.append("kind = '{0}'".format(loai.replace("'", "''")))
    sql = (
        "COPY (SELECT project_id, id, zip_name, zip_size FROM packages WHERE "
        + " AND ".join(dk)
        + " ORDER BY created_at) TO STDOUT WITH CSV"
    )
    r = subprocess.run(
        ["docker", "exec", CONTAINER, "psql", "-U", "digitext", "-d", "digitext", "-At", "-c", sql],
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    if r.returncode != 0:
        raise SystemExit(f"hỏi CSDL lỗi: {r.stderr.strip()[:400]}")
    ra = []
    for hang in csv.reader(io.StringIO(r.stdout)):
        if len(hang) >= 4 and hang[2]:
            ra.append((hang[0], hang[1], hang[2], int(hang[3] or 0)))
    return ra


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--thu-muc", required=True, help="thư mục đích, vd F:/Goi-AIP")
    ap.add_argument("--nguon", default="http://127.0.0.1:9000")
    ap.add_argument("--bucket", default="pages")
    ap.add_argument("--du-an", default=None, help="mã dự án hoặc uuid; bỏ trống = mọi dự án")
    ap.add_argument("--loai", default=None, help="aip_hoso | sip_hoso | … ; bỏ trống = mọi loại")
    ap.add_argument("--luong", type=int, default=4)
    ap.add_argument("--gioi-han", type=int, default=0, help="chỉ lấy N gói đầu (0 = lấy hết); dùng để chạy thử")
    a = ap.parse_args()

    env = doc_env(REPO / "crates" / "digitext-api" / ".env")
    s3 = boto3.client(
        "s3",
        endpoint_url=a.nguon,
        aws_access_key_id=env.get("DIGITEXT_MINIO_ACCESS_KEY") or env.get("MINIO_ROOT_USER"),
        aws_secret_access_key=env.get("DIGITEXT_MINIO_SECRET_KEY") or env.get("MINIO_ROOT_PASSWORD"),
        region_name=env.get("DIGITEXT_MINIO_REGION", "us-east-1"),
        config=Config(
            retries={"max_attempts": 10, "mode": "standard"},
            read_timeout=600,
            connect_timeout=20,
            max_pool_connections=a.luong * 2,
            signature_version="s3v4",
        ),
    )

    goi = danh_sach_goi(a.du_an, a.loai)
    if a.gioi_han > 0:
        goi = goi[: a.gioi_han]
    if not goi:
        print("không có gói nào đã dựng xong khớp điều kiện")
        return 1
    dich = Path(a.thu_muc)
    dich.mkdir(parents=True, exist_ok=True)
    tong = sum(g[3] for g in goi)
    print(f"{len(goi)} gói, {tong / 1024**3:.1f} GB → {dich}")

    khoa = threading.Lock()
    dem = {"xong": 0, "bo_qua": 0, "byte": 0, "loi": []}
    bat_dau = time.time()

    def mot(g: tuple[str, str, str, int]) -> None:
        project_id, package_id, zip_name, zip_size = g
        ten = Path(zip_name).name or f"{package_id}.zip"
        ra = dich / ten
        if ra.exists() and zip_size and ra.stat().st_size == zip_size:
            with khoa:
                dem["bo_qua"] += 1
            return
        key = f"{project_id}-packages/{package_id}.zip"
        tam = ra.with_suffix(ra.suffix + ".dang-tai")
        try:
            s3.download_file(a.bucket, key, str(tam))
            co = tam.stat().st_size
            if zip_size and co != zip_size:
                tam.unlink(missing_ok=True)
                raise OSError(f"tải cụt: {co} ≠ {zip_size} byte")
            os.replace(tam, ra)
            with khoa:
                dem["xong"] += 1
                dem["byte"] += co
        except Exception as e:  # noqa: BLE001 — một gói hỏng không được làm dừng cả đợt
            tam.unlink(missing_ok=True)
            with khoa:
                dem["loi"].append(f"{ten}: {e}")

    with ThreadPoolExecutor(max_workers=a.luong) as ex:
        viec = [ex.submit(mot, g) for g in goi]
        for i, _ in enumerate(as_completed(viec), 1):
            if i % 10 == 0 or i == len(goi):
                troi = max(time.time() - bat_dau, 0.001)
                mbps = dem["byte"] / 1024**2 / troi
                con = (tong - dem["byte"]) / 1024**2 / mbps / 60 if mbps > 0.01 else 0
                print(
                    f"  {i}/{len(goi)} · {dem['xong']} tải · {dem['bo_qua']} bỏ qua · "
                    f"{dem['byte'] / 1024**3:.1f} GB · {mbps:.0f} MB/giây · còn ~{con:.0f} phút",
                    flush=True,
                )

    troi = max(time.time() - bat_dau, 0.001)
    print(
        f"XONG: {dem['xong']} gói tải mới, {dem['bo_qua']} đã có sẵn, "
        f"{dem['byte'] / 1024**3:.1f} GB trong {troi / 60:.1f} phút ({dem['byte'] / 1024**2 / troi:.0f} MB/giây)"
    )
    if dem["loi"]:
        print(f"{len(dem['loi'])} gói lỗi (chạy lại script để lấy nốt):")
        for d in dem["loi"][:20]:
            print("  ", d)
    return 1 if dem["loi"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
