"""Xoá tệp của dự án LTLSBNI (01a0cd0c-bb65-7501-bf10-134d44e05284) trong kho MinIO — anh Hiếu yêu cầu 16/10/2026.
Chỉ đụng 3 tiền tố mang mã dự án này. Chạy:  python -I dev\\xoa-du-an-ltlsbni.py
"""
import re
import sys

import boto3

sys.stdout.reconfigure(encoding="utf-8")
PID = "01a0cd0c-bb65-7501-bf10-134d44e05284"
env = {}
for l in open(r"D:\DigiText\crates\digitext-api\.env", encoding="utf-8"):
    m = re.match(r"\s*([A-Z0-9_]+)\s*=\s*(.*?)\s*$", l)
    if m:
        env[m.group(1)] = m.group(2).strip('"')
s3 = boto3.client("s3", endpoint_url=env["DIGITEXT_MINIO_ENDPOINT"], aws_access_key_id=env["DIGITEXT_MINIO_ACCESS_KEY"],
                  aws_secret_access_key=env["DIGITEXT_MINIO_SECRET_KEY"], region_name="us-east-1")
b = env["DIGITEXT_MINIO_BUCKET"]
tong = n = 0
for pre in (f"{PID}/", f"{PID}-warm/", f"{PID}-packages/"):
    keys = []
    for pg in s3.get_paginator("list_objects_v2").paginate(Bucket=b, Prefix=pre):
        keys += [(o["Key"], o["Size"]) for o in pg.get("Contents", [])]
    for i in range(0, len(keys), 1000):
        s3.delete_objects(Bucket=b, Delete={"Objects": [{"Key": k} for k, _ in keys[i : i + 1000]], "Quiet": True})
    n += len(keys)
    tong += sum(s for _, s in keys)
    print(f"{pre}: xoá {len(keys)} tệp")
print(f"TỔNG: {n} tệp, {tong / 1e6:.1f} MB")
