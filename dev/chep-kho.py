"""Chép toàn bộ bucket giữa hai MinIO (chuyển kho 17/10/2026). Thay `mc mirror`: tệp lớn đọc cụt giữa chừng trên ổ HDD SMR
làm mc dừng cả đợt mỗi ~12 phút. Ở đây mỗi tệp đọc trọn vào RAM, kiểm đủ độ dài, thiếu thì thử lại riêng tệp đó.

    python -I dev/chep-kho.py --nguon http://127.0.0.1:9000 --dich http://127.0.0.1:9200 [--luong 4]

Khoá truy cập lấy từ crates/digitext-api/.env (hai MinIO dùng chung tài khoản gốc). Tệp đã có ở đích, đúng kích thước → bỏ qua.
In tiến độ ra stdout; kết thúc in số tệp lỗi (chạy lại để chép nốt).
"""
from __future__ import annotations

import argparse
import re
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import boto3
from botocore.config import Config

REPO = Path(__file__).resolve().parent.parent


def doc_env(p: Path) -> dict[str, str]:
    kv = {}
    for line in p.read_text(encoding="utf-8").splitlines():
        m = re.match(r"\s*([A-Z0-9_]+)\s*=\s*(.*?)\s*$", line)
        if m:
            kv[m.group(1)] = m.group(2).strip('"')
    return kv


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--nguon", required=True)
    ap.add_argument("--dich", required=True)
    ap.add_argument("--bucket", default="pages")
    ap.add_argument("--luong", type=int, default=4)
    a = ap.parse_args()
    env = doc_env(REPO / "crates" / "digitext-api" / ".env")
    cfg = Config(retries={"max_attempts": 10, "mode": "standard"}, read_timeout=300, connect_timeout=20, max_pool_connections=a.luong * 2)

    def kh(url: str):
        return boto3.client("s3", endpoint_url=url, aws_access_key_id=env["DIGITEXT_MINIO_ACCESS_KEY"],
                            aws_secret_access_key=env["DIGITEXT_MINIO_SECRET_KEY"], region_name="us-east-1", config=cfg)

    nguon, dich = kh(a.nguon), kh(a.dich)

    def liet_ke(c) -> dict[str, int]:
        out = {}
        for pg in c.get_paginator("list_objects_v2").paginate(Bucket=a.bucket):
            for o in pg.get("Contents", []):
                out[o["Key"]] = o["Size"]
        return out

    t0 = time.time()
    co = liet_ke(nguon)
    da = liet_ke(dich)
    viec = [(k, n) for k, n in co.items() if da.get(k) != n]
    tong_byte = sum(n for _, n in viec)
    print(f"Nguồn {len(co):,} tệp; đích đã có {len(da):,}; cần chép {len(viec):,} tệp ({tong_byte / 1e9:,.1f} GB). "
          f"Liệt kê mất {time.time() - t0:,.0f} s.", flush=True)

    khoa = threading.Lock()
    dem = {"xong": 0, "byte": 0, "loi": 0}
    loi: list[str] = []

    def mot(k: str, n: int) -> None:
        for lan in range(1, 9):
            try:
                obj = nguon.get_object(Bucket=a.bucket, Key=k)
                body = obj["Body"].read()
                if len(body) != n:
                    raise IOError(f"đọc cụt {len(body)}/{n} byte")
                dich.put_object(Bucket=a.bucket, Key=k, Body=body, ContentType=obj.get("ContentType") or "application/octet-stream")
                if dich.head_object(Bucket=a.bucket, Key=k)["ContentLength"] != n:
                    raise IOError("đích sai kích thước")
                with khoa:
                    dem["xong"] += 1
                    dem["byte"] += n
                return
            except Exception as e:  # noqa: BLE001 — thử lại riêng tệp này
                if lan == 8:
                    with khoa:
                        dem["loi"] += 1
                        loi.append(f"{k}: {type(e).__name__}: {e}")
                    return
                time.sleep(min(30, 2 * lan))

    t1 = time.time()
    with ThreadPoolExecutor(a.luong) as ex:
        tl = [ex.submit(mot, k, n) for k, n in viec]
        for i, _ in enumerate(as_completed(tl), 1):
            if i % 1000 == 0 or i == len(tl):
                dt = max(time.time() - t1, 1)
                with khoa:
                    con = (tong_byte - dem["byte"]) / max(dem["byte"] / dt, 1)
                    print(f"{time.strftime('%H:%M:%S')}  {i:,}/{len(tl):,} tệp · {dem['byte'] / 1e9:,.1f}/{tong_byte / 1e9:,.1f} GB · "
                          f"{dem['byte'] / 1e6 / dt:,.1f} MB/s · lỗi {dem['loi']} · còn ~{con / 3600:,.1f} giờ", flush=True)
    for r in loi[:20]:
        print("LỖI", r)
    print(f"XONG: chép {dem['xong']:,} tệp, lỗi {dem['loi']}.", flush=True)
    return 1 if dem["loi"] else 0


if __name__ == "__main__":
    sys.exit(main())
