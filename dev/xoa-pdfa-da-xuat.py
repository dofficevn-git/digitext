"""Xoá bản PDF/A đã xuất ra ổ ngoài khỏi kho MinIO + đánh dấu CSDL (docs/KE-HOACH-CHUYEN-KHO-MINIO.md mục 0, bước 3).

    python -I dev/xoa-pdfa-da-xuat.py --manifest F:/DigiText-XuatKho/LTLSBN/manifest.csv --ghi-chu "Ổ WD Elements 4TB F:\\DigiText-XuatKho"
    python -I dev/xoa-pdfa-da-xuat.py ... --that      ← mặc định CHỈ CHẠY THỬ (đếm), có --that mới xoá

Chỉ xoá PDF/A của tài liệu thoả TẤT CẢ:
  - manifest có dòng PDF-A khop=1 VÀ dòng Ky-so khop=1;
  - tệp trên ổ ngoài còn đó, đúng kích thước manifest; mã băm tệp PDF-A tính lại khớp (--kiem-lai, mặc định bật);
  - trong kho: bản ký số còn (đúng kích thước manifest), bản PDF/A đúng kích thước manifest.
Cần migration 0019 (documents.pdfa_offloaded_at / pdfa_offload_note).
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

import boto3
from botocore.exceptions import ClientError

REPO = Path(__file__).resolve().parent.parent
PG = ["docker", "exec", "-i", "digitext-dev-postgres-1", "psql", "-U", "digitext", "-d", "digitext", "-v", "ON_ERROR_STOP=1", "-q"]


def doc_env(p: Path) -> dict[str, str]:
    kv = {}
    for line in p.read_text(encoding="utf-8").splitlines():
        m = re.match(r"\s*([A-Z0-9_]+)\s*=\s*(.*?)\s*$", line)
        if m:
            kv[m.group(1)] = m.group(2).strip('"')
    return kv


def sha256_tep(p: Path) -> str:
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for khoi in iter(lambda: f.read(4 << 20), b""):
            h.update(khoi)
    return h.hexdigest()


def kich_thuoc_kho(s3, bucket: str, key: str) -> int | None:
    try:
        return s3.head_object(Bucket=bucket, Key=key)["ContentLength"]
    except ClientError as e:
        if e.response.get("Error", {}).get("Code") in ("404", "NoSuchKey", "NotFound"):
            return None
        raise


def danh_dau(ids: list[str], ghi_chu: str) -> None:
    sql = (
        "SET lock_timeout = '5s';\n"
        "UPDATE documents SET pdfa_offloaded_at = now(), pdfa_offload_note = :'ghichu' "
        f"WHERE id IN ({','.join(repr(i) for i in ids)});\n"
    )
    subprocess.run(PG + ["-v", f"ghichu={ghi_chu}"], input=sql.encode("utf-8"), check=True)


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--manifest", required=True)
    ap.add_argument("--ghi-chu", required=True, help="nơi lưu, hiện cho người dùng khi mở PDF/A")
    ap.add_argument("--that", action="store_true", help="xoá thật (mặc định chỉ chạy thử)")
    ap.add_argument("--khong-kiem-lai", action="store_true", help="bỏ bước tính lại SHA-256 trên ổ ngoài")
    a = ap.parse_args()

    dong: dict[str, dict[str, dict]] = defaultdict(dict)
    with open(a.manifest, encoding="utf-8-sig", newline="") as f:
        for r in csv.DictReader(f):
            if r["khop"] == "1":
                dong[r["document_id"]][r["loai"]] = r  # dòng sau ghi đè dòng trước (lần xuất lại)
    du = {d: v for d, v in dong.items() if "PDF-A" in v and "Ky-so" in v}
    print(f"Manifest: {len(dong):,} tài liệu có tệp khớp; {len(du):,} có ĐỦ PDF/A + bản ký → ứng viên xoá PDF/A.")

    env = doc_env(REPO / "crates" / "digitext-api" / ".env")
    s3 = boto3.client("s3", endpoint_url=env["DIGITEXT_MINIO_ENDPOINT"], aws_access_key_id=env["DIGITEXT_MINIO_ACCESS_KEY"],
                      aws_secret_access_key=env["DIGITEXT_MINIO_SECRET_KEY"], region_name=env.get("DIGITEXT_MINIO_REGION", "us-east-1"))
    bucket = env.get("DIGITEXT_PDF_STORE_BUCKET") or env["DIGITEXT_MINIO_BUCKET"]

    duoc, bo_qua, lo = [], 0, []
    tong_byte = 0
    for i, (doc, v) in enumerate(du.items(), 1):
        pa, ks = v["PDF-A"], v["Ky-so"]
        ly_do = None
        tep_pa = Path(pa["duong_dan"])
        if not tep_pa.exists() or tep_pa.stat().st_size != int(pa["kich_thuoc"]):
            ly_do = "tệp PDF/A trên ổ ngoài thiếu / sai kích thước"
        elif not Path(ks["duong_dan"]).exists() or Path(ks["duong_dan"]).stat().st_size != int(ks["kich_thuoc"]):
            ly_do = "tệp ký số trên ổ ngoài thiếu / sai kích thước"
        elif kich_thuoc_kho(s3, bucket, ks["khoa_kho"]) != int(ks["kich_thuoc"]):
            ly_do = "bản ký số trong kho thiếu / khác kích thước"
        else:
            n = kich_thuoc_kho(s3, bucket, pa["khoa_kho"])
            if n is None:
                ly_do = "PDF/A đã không còn trong kho"
            elif n != int(pa["kich_thuoc"]):
                ly_do = "PDF/A trong kho khác kích thước manifest"
            elif not a.khong_kiem_lai and sha256_tep(tep_pa) != pa["sha256"]:
                ly_do = "SHA-256 tệp PDF/A trên ổ ngoài không còn khớp"
        if ly_do:
            bo_qua += 1
            if bo_qua <= 20:
                print(f"  bỏ qua {doc}: {ly_do}")
            continue
        duoc.append((doc, pa["khoa_kho"]))
        tong_byte += int(pa["kich_thuoc"])
        if i % 2000 == 0:
            print(f"  đã kiểm {i:,}/{len(du):,}…", flush=True)

    print(f"Đủ điều kiện xoá: {len(duoc):,} PDF/A ({tong_byte / 1e9:,.1f} GB); bỏ qua {bo_qua:,}.")
    if not a.that:
        print("CHẠY THỬ — chưa xoá gì. Thêm --that để xoá.")
        return 0
    for i in range(0, len(duoc), 500):
        lo = duoc[i : i + 500]
        for doc, key in lo:
            s3.delete_object(Bucket=bucket, Key=key)
        danh_dau([d for d, _ in lo], a.ghi_chu)
        print(f"  đã xoá {min(i + 500, len(duoc)):,}/{len(duoc):,}", flush=True)
    print("Xong.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
