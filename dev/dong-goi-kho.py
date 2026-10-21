"""Đóng gói kho MinIO ra ổ ngoài thành tệp .tar lớn (ghi TUẦN TỰ — ổ USB SMR ghi tệp nhỏ kiểu MinIO chỉ ~14 MB/s) rồi
nạp lại vào MinIO mới. Chuyển kho 17/10/2026 (docs/KE-HOACH-CHUYEN-KHO-MINIO.md).

  Đóng gói (chạy lại được, chỉ lấy khoá chưa có / đổi kích thước):
    python -I dev/dong-goi-kho.py xuat --nguon http://127.0.0.1:9000 --thu-muc F:/DigiText-KhoTar
  Nạp lại (bỏ qua tệp đã có đúng kích thước ở đích):
    python -I dev/dong-goi-kho.py nap --dich http://127.0.0.1:9000 --thu-muc F:/DigiText-KhoTar

<thu-muc>/kho-NNNN.tar   mỗi tệp ~20 GB, tên mục trong tar = khoá đối tượng.
<thu-muc>/danh-sach.csv  khoa,kich_thuoc,sha256,tep_tar — chỉ ghi khi tệp .tar đã đóng và fsync xong, nên tar dở
                         (máy tắt giữa chừng) không bị coi là đã có; lần chạy sau bỏ tar dở đó, ghi phần mới.
Khoá truy cập: crates/digitext-api/.env (MinIO cũ và mới dùng chung tài khoản gốc).
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import io
import os
import queue
import re
import sys
import tarfile
import threading
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import boto3
from botocore.config import Config

REPO = Path(__file__).resolve().parent.parent
CO_TAR = 20 * 1000**3


def doc_env(p: Path) -> dict[str, str]:
    kv = {}
    for line in p.read_text(encoding="utf-8").splitlines():
        m = re.match(r"\s*([A-Z0-9_]+)\s*=\s*(.*?)\s*$", line)
        if m:
            kv[m.group(1)] = m.group(2).strip('"')
    return kv


def client(url: str, luong: int):
    env = doc_env(REPO / "crates" / "digitext-api" / ".env")
    cfg = Config(retries={"max_attempts": 10, "mode": "standard"}, read_timeout=300, connect_timeout=20, max_pool_connections=luong * 2)
    return boto3.client("s3", endpoint_url=url, aws_access_key_id=env["DIGITEXT_MINIO_ACCESS_KEY"],
                        aws_secret_access_key=env["DIGITEXT_MINIO_SECRET_KEY"], region_name="us-east-1", config=cfg)


def doc_danh_sach(p: Path) -> dict[str, tuple[int, str, str]]:
    out = {}
    if p.exists():
        with open(p, encoding="utf-8", newline="") as f:
            for r in csv.DictReader(f):
                out[r["khoa"]] = (int(r["kich_thuoc"]), r["sha256"], r["tep_tar"])
    return out


def xuat(a) -> int:
    goc = Path(a.thu_muc)
    goc.mkdir(parents=True, exist_ok=True)
    ds_p = goc / "danh-sach.csv"
    da = doc_danh_sach(ds_p)
    nguon = client(a.nguon, a.luong)
    co = {}
    for pg in nguon.get_paginator("list_objects_v2").paginate(Bucket=a.bucket):
        for o in pg.get("Contents", []):
            co[o["Key"]] = o["Size"]
    viec = sorted(k for k, n in co.items() if da.get(k, (None,))[0] != n)
    if a.gioi_han:
        viec = viec[: a.gioi_han]
    tong = sum(co[k] for k in viec)
    so_tar = max([int(t[1][4:8]) for t in [(0, v[2]) for v in da.values()] if t[1].startswith("kho-")] or [0])
    # Bỏ tar dở của lần trước (không có trong danh sách).
    for t in goc.glob("kho-*.tar"):
        if int(t.stem[4:]) > so_tar:
            t.unlink()
    print(f"Nguồn {len(co):,} tệp; đã đóng gói {len(da):,}; cần đóng gói {len(viec):,} tệp ({tong / 1e9:,.1f} GB).", flush=True)
    if not viec:
        return 0

    hang: queue.Queue = queue.Queue(maxsize=a.luong * 4)
    loi: list[str] = []

    def doc(k: str) -> None:
        n = co[k]
        for lan in range(1, 9):
            try:
                body = nguon.get_object(Bucket=a.bucket, Key=k)["Body"].read()
                if len(body) != n:
                    raise IOError(f"đọc cụt {len(body)}/{n}")
                hang.put((k, body))
                return
            except Exception as e:  # noqa: BLE001
                if lan == 8:
                    loi.append(f"{k}: {e}")
                    hang.put((k, None))
                    return
                time.sleep(min(30, 2 * lan))

    def doc_tat_ca() -> None:
        with ThreadPoolExecutor(a.luong) as ex:
            for f in as_completed([ex.submit(doc, k) for k in viec]):
                f.result()
        hang.put(None)

    threading.Thread(target=doc_tat_ca, daemon=True).start()
    moi = not ds_p.exists()
    fds = open(ds_p, "a", encoding="utf-8", newline="")
    w = csv.writer(fds)
    if moi:
        w.writerow(["khoa", "kich_thuoc", "sha256", "tep_tar"])
    tar = None
    tep = None
    cho_ghi: list[list] = []
    co_hien = 0
    xong = byte = 0
    t0 = time.time()

    def dong_tar() -> None:
        nonlocal tar, tep, cho_ghi
        if tar is None:
            return
        tar.close()
        with open(tep, "rb+") as f:
            os.fsync(f.fileno())
        w.writerows(cho_ghi)
        fds.flush()
        os.fsync(fds.fileno())
        tar, cho_ghi = None, []

    while (muc := hang.get()) is not None:
        k, body = muc
        if body is None:
            continue
        if tar is None or co_hien >= CO_TAR:
            dong_tar()
            so_tar += 1
            tep = goc / f"kho-{so_tar:04d}.tar"
            tar = tarfile.open(tep, "w", format=tarfile.PAX_FORMAT)
            co_hien = 0
        ti = tarfile.TarInfo(k)
        ti.size = len(body)
        ti.mtime = int(time.time())
        tar.addfile(ti, io.BytesIO(body))
        cho_ghi.append([k, len(body), hashlib.sha256(body).hexdigest(), tep.name])
        co_hien += len(body)
        xong += 1
        byte += len(body)
        if xong % 2000 == 0:
            dt = max(time.time() - t0, 1)
            print(f"{time.strftime('%H:%M:%S')}  {xong:,}/{len(viec):,} tệp · {byte / 1e9:,.1f}/{tong / 1e9:,.1f} GB · "
                  f"{byte / 1e6 / dt:,.1f} MB/s · lỗi {len(loi)} · còn ~{(tong - byte) / max(byte / dt, 1) / 3600:,.1f} giờ", flush=True)
    dong_tar()
    fds.close()
    for r in loi[:20]:
        print("LỖI", r)
    print(f"XONG đóng gói: {xong:,} tệp, {byte / 1e9:,.1f} GB, lỗi {len(loi)}.", flush=True)
    return 1 if loi else 0


def nap(a) -> int:
    goc = Path(a.thu_muc)
    da = doc_danh_sach(goc / "danh-sach.csv")
    dich = client(a.dich, a.luong)
    try:
        dich.head_bucket(Bucket=a.bucket)
    except Exception:  # noqa: BLE001
        dich.create_bucket(Bucket=a.bucket)
    co = {}
    for pg in dich.get_paginator("list_objects_v2").paginate(Bucket=a.bucket):
        for o in pg.get("Contents", []):
            co[o["Key"]] = o["Size"]
    can = {k for k, v in da.items() if co.get(k) != v[0]}
    tong = sum(da[k][0] for k in can)
    print(f"Danh sách {len(da):,} tệp; đích đã có {len(co):,}; cần nạp {len(can):,} tệp ({tong / 1e9:,.1f} GB).", flush=True)
    loi: list[str] = []
    xong = byte = 0
    t0 = time.time()
    khoa = threading.Lock()

    def day(k: str, body: bytes) -> None:
        nonlocal xong, byte
        for lan in range(1, 9):
            try:
                dich.put_object(Bucket=a.bucket, Key=k, Body=body, ContentType="application/pdf" if k.endswith(".pdf") else "application/octet-stream")
                if dich.head_object(Bucket=a.bucket, Key=k)["ContentLength"] != len(body):
                    raise IOError("đích sai kích thước")
                with khoa:
                    xong += 1
                    byte += len(body)
                return
            except Exception as e:  # noqa: BLE001
                if lan == 8:
                    loi.append(f"{k}: {e}")
                    return
                time.sleep(min(30, 2 * lan))

    # Ổ HDD SMR (19/10/2026): ghi nhanh tới khi vùng đệm đầy rồi tụt; --nghi-khi-cham X: tốc độ phút < X MB/s
    # liền 3 phút thì ngừng đẩy --nghi-phut phút cho ổ tự dọn, rồi chạy tiếp. Mỗi phút in một dòng tiến độ.
    het = threading.Event()
    nghi_den = [0.0]

    nghi_ds = [float(x) for x in str(a.nghi_phut).split(",") if x.strip()]

    def theo_doi() -> None:
        truoc, cham, tep_truoc = 0, 0, 0
        chu_ky, cky_t, cky_b, nghi_truoc = 0, t0, 0, 0.0
        while not het.wait(60):
            with khoa:
                b = byte
            mbs = (b - truoc) / 1e6 / 60
            truoc = b
            tep_phut, tep_truoc = xong - tep_truoc, xong
            dang_nghi = time.time() < nghi_den[0]
            print(f"{time.strftime('%H:%M:%S')}  {xong:,}/{len(can):,} tệp · {b / 1e9:,.1f}/{tong / 1e9:,.1f} GB · "
                  f"phút vừa qua {mbs:,.1f} MB/s{' · ĐANG NGHỈ' if dang_nghi else ''} · lỗi {len(loi)}", flush=True)
            # Chỉ xét phút đã chạy TRỌN sau khi hết nghỉ: phút vừa hết nghỉ gần như chưa ghi gì, xét nó thì nghỉ tiếp
            # mãi (21/10/2026, --so-phut-cham 1: 17:49–19:52 nghỉ liền 5 lượt, ghi 0,1 GB).
            if a.nghi_khi_cham and not dang_nghi and b > 0 and time.time() - t0 > 120 and time.time() - nghi_den[0] >= 55:
                # 22/10/2026: đoạn nhiều tệp nhỏ (~2 MB) ghi ít MB/s mà ổ vẫn khoẻ → chỉ coi là chậm khi CẢ số tệp/phút thấp.
                cham = cham + 1 if mbs < a.nghi_khi_cham and tep_phut < a.tep_khi_cham else 0
                if cham >= a.so_phut_cham:
                    phut = nghi_ds[chu_ky % len(nghi_ds)]
                    chay = (time.time() - cky_t) / 60
                    gb = (b - cky_b) / 1e9
                    print(f"{time.strftime('%H:%M:%S')}  CHU KỲ {chu_ky}: sau nghỉ {nghi_truoc:g} phút ghi được {gb:,.1f} GB "
                          f"trong {chay:,.0f} phút chạy · TB cả chu kỳ (kể cả nghỉ) "
                          f"{gb * 1e3 / ((chay + nghi_truoc) * 60):,.1f} MB/s", flush=True)
                    nghi_den[0] = time.time() + phut * 60
                    cham, chu_ky, nghi_truoc = 0, chu_ky + 1, phut
                    cky_t, cky_b = nghi_den[0], b
                    print(f"{time.strftime('%H:%M:%S')}  tốc độ thấp {a.so_phut_cham} phút liền → nghỉ {phut:g} phút cho ổ dọn vùng đệm", flush=True)

    threading.Thread(target=theo_doi, daemon=True).start()
    with ThreadPoolExecutor(a.luong) as ex:
        dang = []
        for tep in sorted(goc.glob("kho-*.tar")):
            with tarfile.open(tep, "r") as tar:
                for ti in tar:
                    if ti.name not in can:
                        continue
                    while time.time() < nghi_den[0]:
                        time.sleep(5)
                    body = tar.extractfile(ti).read()
                    if hashlib.sha256(body).hexdigest() != da[ti.name][1]:
                        loi.append(f"{ti.name}: SHA-256 trong {tep.name} không khớp danh sách")
                        continue
                    dang.append(ex.submit(day, ti.name, body))
                    if len(dang) >= a.luong * 4:
                        dang[0].result()
                        dang.pop(0)
        for f in dang:
            f.result()
    het.set()
    for r in loi[:20]:
        print("LỖI", r)
    print(f"XONG nạp: {xong:,} tệp, {byte / 1e9:,.1f} GB, lỗi {len(loi)}.", flush=True)
    return 1 if loi else 0


def kiem(a) -> int:
    """Đọc lại mọi .tar, tính SHA-256 từng mục, so với danh-sach.csv (trước khi xoá kho cũ)."""
    goc = Path(a.thu_muc)
    da = doc_danh_sach(goc / "danh-sach.csv")
    thay: set[str] = set()
    loi: list[str] = []
    byte = 0
    t0 = time.time()
    for tep in sorted(goc.glob("kho-*.tar")):
        with tarfile.open(tep, "r") as tar:
            for ti in tar:
                body = tar.extractfile(ti).read()
                byte += len(body)
                v = da.get(ti.name)
                if v is None:
                    loi.append(f"{ti.name}: có trong {tep.name} nhưng không có trong danh sách")
                elif v[2] != tep.name or v[0] != len(body) or hashlib.sha256(body).hexdigest() != v[1]:
                    loi.append(f"{ti.name}: không khớp danh sách ({tep.name})")
                thay.add(ti.name)
        print(f"{time.strftime('%H:%M:%S')}  đã kiểm {tep.name} · {len(thay):,}/{len(da):,} tệp · {byte / 1e9:,.1f} GB · "
              f"{byte / 1e6 / max(time.time() - t0, 1):,.0f} MB/s · lỗi {len(loi)}", flush=True)
    thieu = [k for k in da if k not in thay]
    for k in thieu[:10]:
        loi.append(f"{k}: có trong danh sách nhưng không thấy trong tệp .tar")
    for r in loi[:20]:
        print("LỖI", r)
    print(f"XONG kiểm: {len(thay):,}/{len(da):,} tệp đọc lại được và khớp SHA-256; thiếu {len(thieu)}; lỗi {len(loi)}.", flush=True)
    return 1 if loi else 0


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("viec", choices=["xuat", "nap", "kiem"])
    ap.add_argument("--nguon")
    ap.add_argument("--dich")
    ap.add_argument("--thu-muc", required=True)
    ap.add_argument("--bucket", default="pages")
    ap.add_argument("--luong", type=int, default=4)
    ap.add_argument("--gioi-han", type=int, default=0, help="chỉ đóng gói N tệp đầu (thử)")
    ap.add_argument("--nghi-khi-cham", type=float, default=0, help="nap: tốc độ phút < X MB/s liền 3 phút thì nghỉ")
    ap.add_argument("--tep-khi-cham", type=int, default=10**9,
                    help="nap: chỉ coi là chậm khi số tệp/phút cũng dưới mức này (mặc định: không xét)")
    ap.add_argument("--so-phut-cham", type=int, default=3,
                    help="nap: số phút chậm liền nhau thì nghỉ (21/10/2026: OCR đọc chung ổ — chậm 3 phút là OCR khựng 3 phút; dùng 1)")
    ap.add_argument("--nghi-phut", default="30", help="nap: số phút nghỉ cho ổ dọn vùng đệm; danh sách '10,5,15' thì luân phiên (thử)")
    a = ap.parse_args()
    return {"xuat": xuat, "nap": nap, "kiem": kiem}[a.viec](a)


if __name__ == "__main__":
    sys.exit(main())
