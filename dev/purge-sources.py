"""Xoá PDF gốc của tài liệu ĐÃ có bản PDF/A-2b trong kho (5/10/2026: kết quả cuối chỉ giữ
1 bản PDF/A + 1 bản ký số). Chạy lại được: chỉ lấy tài liệu source_purged_at IS NULL.
Mỗi tài liệu: HEAD bản PDF/A (phải còn, > 0 byte) -> xoá PDF gốc -> ghi source_purged_at (theo mẻ).
"""
import pathlib, subprocess, time
import boto3
from botocore.exceptions import ClientError

LOG = open(pathlib.Path(r"D:\DigiText\logs") / "purge.log", "a", encoding="utf-8")


def log(m):
    line = f"{time.strftime('%H:%M:%S')} {m}"
    print(line, flush=True)
    LOG.write(line + "\n"); LOG.flush()


def psql(sql: str) -> str:
    r = subprocess.run(["docker", "exec", "-i", "digitext-dev-postgres-1", "sh", "-c",
                        'psql -v ON_ERROR_STOP=1 -U ${POSTGRES_USER:-postgres} -d ${POSTGRES_DB:-digitext} -At -F "\t"'],
                       input=sql, capture_output=True, text=True, encoding="utf-8")
    if r.returncode != 0:
        raise RuntimeError(r.stderr)
    return r.stdout


env = {}
for line in open(r"D:\DigiText\crates\digitext-ocr\.env", encoding="utf-8"):
    if "=" in line and not line.lstrip().startswith("#"):
        k, v = line.strip().split("=", 1)
        env[k] = v
s3 = boto3.client("s3", endpoint_url=env["DIGITEXT_MINIO_ENDPOINT"],
                  aws_access_key_id=env["DIGITEXT_MINIO_ACCESS_KEY"],
                  aws_secret_access_key=env["DIGITEXT_MINIO_SECRET_KEY"],
                  region_name=env.get("DIGITEXT_MINIO_REGION", "us-east-1"))
bucket = env["DIGITEXT_MINIO_BUCKET"]


def size(key):
    try:
        return s3.head_object(Bucket=bucket, Key=key)["ContentLength"]
    except ClientError as e:
        if e.response["Error"]["Code"] in ("404", "NoSuchKey", "NotFound"):
            return None
        raise


total_deleted = total_bytes = kept = 0
while True:
    rows = [l.split("\t") for l in psql(
        "SELECT id, storage_path, pdfa_key FROM documents "
        "WHERE pdfa_key IS NOT NULL AND pdfa_2b AND source_purged_at IS NULL "
        "AND storage_path <> pdfa_key ORDER BY pdfa_at LIMIT 1000;").splitlines() if l]
    if not rows:
        break
    done_ids, skip_ids = [], []
    for doc_id, src, pdfa in rows:
        if not size(pdfa):
            kept += 1
            skip_ids.append(doc_id)
            log(f"GIỮ {doc_id}: không thấy bản PDF/A {pdfa}")
            continue
        sz = size(src)
        if sz is not None:
            s3.delete_object(Bucket=bucket, Key=src)
            total_bytes += sz
        done_ids.append(doc_id)
    if done_ids:
        ids = ",".join(f"'{i}'" for i in done_ids)
        psql(f"UPDATE documents SET source_purged_at = now() WHERE id IN ({ids});")
        total_deleted += len(done_ids)
    if skip_ids and not done_ids:
        log("mẻ chỉ còn tài liệu thiếu PDF/A — dừng")
        break
    log(f"đã xoá {total_deleted} tệp gốc · {total_bytes / 1e9:.1f} GB · giữ {kept}")
log(f"XONG: xoá {total_deleted} tệp gốc, giải phóng {total_bytes / 1e9:.1f} GB, giữ {kept}")
