"""Xuất bản PDF/A + bản ký số của MỘT dự án ra ổ ngoài, xếp theo Phông/Hộp/Hồ sơ (docs/KE-HOACH-CHUYEN-KHO-MINIO.md mục 0).

    python -I dev/xuat-kho-theo-ho-so.py --du-an LTLSBN --ra F:/DigiText-XuatKho [--luong 4] [--gioi-han 20]

- Đọc danh sách tài liệu từ Postgres (docker exec psql), tệp từ MinIO qua S3 (khoá trong crates/digitext-api/.env).
- Ghi <ra>/<mã dự án>/PDF-A/<rel_path>/<tên> và .../Ky-so/<rel_path>/<tên>; ghi .part rồi đổi tên,
  ĐỌC LẠI từ đĩa đích so SHA-256 với lúc tải → cột khop của manifest.csv. Chỉ dòng khop=1 mới được phép xoá khỏi kho.
- Chạy lại được: tệp đã có trong manifest với khop=1 và đúng kích thước thì bỏ qua.
- Chỉ ĐỌC kho và CSDL — không xoá gì.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
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
PG = ["docker", "exec", "-i", "digitext-dev-postgres-1", "psql", "-U", "digitext", "-d", "digitext", "-v", "ON_ERROR_STOP=1", "-q"]
COT = ["document_id", "loai", "khoa_kho", "duong_dan", "kich_thuoc", "sha256", "khop", "luc", "loi"]
KY_TU_CAM = re.compile(r'[<>:"|?*\x00-\x1f]')


def doc_env(p: Path) -> dict[str, str]:
    kv = {}
    for line in p.read_text(encoding="utf-8").splitlines():
        m = re.match(r"\s*([A-Z0-9_]+)\s*=\s*(.*?)\s*$", line)
        if m:
            kv[m.group(1)] = m.group(2).strip('"')
    return kv


def danh_sach(ma: str) -> list[dict]:
    sql = f"""COPY (
      SELECT d.id, COALESCE(d.rel_path, '') AS rel_path, d.filename, COALESCE(d.pdfa_key, '') AS pdfa_key,
             COALESCE((SELECT s.signed_key FROM sign_sessions s WHERE s.document_id = d.id AND s.status = 'signed'
                       AND s.signed_key IS NOT NULL ORDER BY s.updated_at DESC LIMIT 1), '') AS signed_key
      FROM documents d JOIN projects p ON p.id = d.project_id
      WHERE p.code = '{ma.replace("'", "''")}' ORDER BY d.rel_path, d.filename
    ) TO STDOUT WITH CSV HEADER"""
    out = subprocess.run(PG + ["-c", sql], capture_output=True, check=True).stdout.decode("utf-8")
    return list(csv.DictReader(io.StringIO(out)))


def an_toan(phan: str) -> str:
    phan = KY_TU_CAM.sub("_", phan).rstrip(" .")
    return phan or "_"


def duong_dan_dich(goc: Path, ma: str, loai: str, rel_path: str, filename: str) -> Path:
    p = goc / an_toan(ma) / loai
    for phan in rel_path.replace("\\", "/").split("/"):
        if phan.strip() and phan not in (".", ".."):
            p = p / an_toan(phan.strip())
    return p / an_toan(filename)


def sha256_tep(p: Path) -> str:
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for khoi in iter(lambda: f.read(4 << 20), b""):
            h.update(khoi)
    return h.hexdigest()


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--du-an", required=True)
    ap.add_argument("--ra", required=True)
    ap.add_argument("--luong", type=int, default=4)
    ap.add_argument("--gioi-han", type=int, default=0, help="chỉ xuất N tài liệu đầu (thử)")
    a = ap.parse_args()

    goc = Path(a.ra)
    if not goc.anchor or not Path(goc.anchor).exists():
        print(f"Không thấy ổ đích {goc.anchor} — cắm ổ ngoài rồi chạy lại.")
        return 2
    thu_muc = goc / an_toan(a.du_an)
    thu_muc.mkdir(parents=True, exist_ok=True)
    manifest = thu_muc / "manifest.csv"

    env = doc_env(REPO / "crates" / "digitext-api" / ".env")
    s3 = boto3.client(
        "s3",
        endpoint_url=env["DIGITEXT_MINIO_ENDPOINT"],
        aws_access_key_id=env["DIGITEXT_MINIO_ACCESS_KEY"],
        aws_secret_access_key=env["DIGITEXT_MINIO_SECRET_KEY"],
        region_name=env.get("DIGITEXT_MINIO_REGION", "us-east-1"),
        config=Config(retries={"max_attempts": 8, "mode": "standard"}, read_timeout=300, max_pool_connections=a.luong * 2),
    )
    bucket = env.get("DIGITEXT_PDF_STORE_BUCKET") or env["DIGITEXT_MINIO_BUCKET"]

    da_xong: dict[tuple[str, str], tuple[str, int]] = {}
    if manifest.exists():
        with open(manifest, encoding="utf-8-sig", newline="") as f:
            for r in csv.DictReader(f):
                if r["khop"] == "1":
                    da_xong[(r["document_id"], r["loai"])] = (r["duong_dan"], int(r["kich_thuoc"]))

    docs = danh_sach(a.du_an)
    if a.gioi_han:
        docs = docs[: a.gioi_han]
    viec = []
    for d in docs:
        for loai, khoa in (("PDF-A", d["pdfa_key"]), ("Ky-so", d["signed_key"])):
            if not khoa:
                continue
            dich = duong_dan_dich(goc, a.du_an, loai, d["rel_path"], d["filename"])
            cu = da_xong.get((d["id"], loai))
            if cu and Path(cu[0]).exists() and Path(cu[0]).stat().st_size == cu[1]:
                continue
            viec.append((d["id"], loai, khoa, dich))
    print(f"{a.du_an}: {len(docs):,} tài liệu, {len(viec):,} tệp cần xuất ({len(da_xong):,} đã khớp từ trước) → {thu_muc}")

    khoa_ghi = threading.Lock()
    moi = not manifest.exists()
    fm = open(manifest, "a", encoding="utf-8-sig" if moi else "utf-8", newline="")
    w = csv.DictWriter(fm, fieldnames=COT)
    if moi:
        w.writeheader()

    def mot(doc_id: str, loai: str, khoa: str, dich: Path) -> dict:
        dong = {"document_id": doc_id, "loai": loai, "khoa_kho": khoa, "duong_dan": str(dich), "kich_thuoc": 0, "sha256": "", "khop": 0, "luc": time.strftime("%Y-%m-%d %H:%M:%S"), "loi": ""}
        try:
            dich.parent.mkdir(parents=True, exist_ok=True)
            tam = dich.with_name(dich.name + ".part")
            obj = s3.get_object(Bucket=bucket, Key=khoa)
            h = hashlib.sha256()
            n = 0
            with open(tam, "wb") as f:
                for khoi in obj["Body"].iter_chunks(4 << 20):
                    h.update(khoi)
                    f.write(khoi)
                    n += len(khoi)
            if n != obj["ContentLength"]:
                raise IOError(f"tải thiếu: {n} / {obj['ContentLength']} byte")
            os.replace(tam, dich)
            tai = h.hexdigest()
            doc_lai = sha256_tep(dich)
            dong.update(kich_thuoc=n, sha256=tai, khop=int(tai == doc_lai))
            if tai != doc_lai:
                dong["loi"] = f"đọc lại từ đĩa đích khác mã băm ({doc_lai})"
        except Exception as e:  # noqa: BLE001 — ghi lỗi vào manifest, chạy tiếp tệp khác
            dong["loi"] = f"{type(e).__name__}: {e}"
        return dong

    t0 = time.time()
    xong = loi = byte = 0
    with ThreadPoolExecutor(a.luong) as ex:
        tuong_lai = [ex.submit(mot, *v) for v in viec]
        for f in as_completed(tuong_lai):
            dong = f.result()
            with khoa_ghi:
                w.writerow(dong)
                xong += 1
                byte += int(dong["kich_thuoc"])
                if dong["khop"] != 1:
                    loi += 1
                if xong % 200 == 0 or xong == len(viec):
                    fm.flush()
                    dt = max(time.time() - t0, 1)
                    con = (len(viec) - xong) * dt / xong
                    print(f"  {xong:,}/{len(viec):,} tệp · {byte / 1e9:,.1f} GB · {byte / 1e6 / dt:,.0f} MB/s · lỗi {loi} · còn ~{con / 3600:,.1f} giờ", flush=True)
    fm.close()
    print(f"Xong: {xong:,} tệp, {byte / 1e9:,.1f} GB, lỗi {loi}. Manifest: {manifest}")
    return 1 if loi else 0


if __name__ == "__main__":
    sys.exit(main())
