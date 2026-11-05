// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tgc {
    pub id: Uuid,
    pub project_id: Uuid,
    pub requested_by: Option<Uuid>,
    pub kind: String,
    pub status: String,
    pub ho_so_ref: Option<String>,
    pub label: String,
    pub request_json: serde_json::Value,
    pub objid: Option<String>,
    pub zip_key: Option<String>,
    pub zip_name: Option<String>,
    pub zip_sha256: Option<String>,
    pub zip_size: Option<i64>,
    pub document_count: i32,
    pub error: Option<String>,
    pub virus_scan: Option<String>,
    pub attempts: i32,
    pub submission_id: Option<Uuid>,
    pub converted_from: Option<Uuid>,
    pub stt_from: Option<i32>,
    pub stt_to: Option<i32>,
    pub locked_by: Option<String>,
    pub lease_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
const COLS: &str = "id, project_id, requested_by, kind, status, ho_so_ref, label, request_json, \
     objid, zip_key, zip_name, zip_sha256, zip_size, document_count, error, virus_scan, \
     attempts, submission_id, converted_from, stt_from, stt_to, locked_by, lease_until, \
     created_at, updated_at";
pub struct Tdy<'a> {
    pub project_id: Uuid,
    pub requested_by: Option<Uuid>,
    pub kind: &'a str,
    pub ho_so_ref: Option<&'a str>,
    pub label: &'a str,
    pub request_json: &'a serde_json::Value,
    pub submission_id: Option<Uuid>,
    pub converted_from: Option<Uuid>,
    pub stt_range: Option<(i32, i32)>,
}
pub async fn create(r_29e: &PgPool, r_29d: Tdy<'_>) -> sqlx::Result<Tgc> {
    sqlx::query_as::<
        _,
        Tgc,
    >(
            &format!(
                "INSERT INTO packages (id, project_id, requested_by, kind, ho_so_ref, label, request_json, \
             submission_id, converted_from, stt_from, stt_to) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING {COLS}"
            ),
        )
        .bind(Uuid::now_v7())
        .bind(r_29d.project_id)
        .bind(r_29d.requested_by)
        .bind(r_29d.kind)
        .bind(r_29d.ho_so_ref)
        .bind(r_29d.label)
        .bind(r_29d.request_json)
        .bind(r_29d.submission_id)
        .bind(r_29d.converted_from)
        .bind(r_29d.stt_range.map(|(r_29b, _)| r_29b))
        .bind(r_29d.stt_range.map(|(_, r_29c)| r_29c))
        .fetch_one(r_29e)
        .await
}
pub async fn get(r_29f: &PgPool, id: Uuid) -> sqlx::Result<Option<Tgc>> {
    sqlx::query_as::<_, Tgc>(&format!("SELECT {COLS} FROM packages WHERE id = $1"))
        .bind(id)
        .fetch_optional(r_29f)
        .await
}
/// Dòng rút gọn cho màn Gói tin. Bỏ `request_json` (trung bình 24 KB/gói, cá biệt 70 KB —
/// danh sách vài nghìn gói sẽ kéo hàng chục MB mà máy trạm không dùng tới) và lấy sẵn mã/tên
/// phông nằm trong đó để lọc, sắp xếp, tải theo phông.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tgd {
    pub id: Uuid,
    pub project_id: Uuid,
    pub kind: String,
    pub status: String,
    pub ho_so_ref: Option<String>,
    pub label: String,
    pub fond_code: Option<String>,
    pub fond_name: Option<String>,
    pub objid: Option<String>,
    pub zip_name: Option<String>,
    pub zip_sha256: Option<String>,
    pub zip_size: Option<i64>,
    pub document_count: i32,
    pub error: Option<String>,
    pub virus_scan: Option<String>,
    pub submission_id: Option<Uuid>,
    pub converted_from: Option<Uuid>,
    pub stt_from: Option<i32>,
    pub stt_to: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
const COLS_GON: &str = "id, project_id, kind, status, ho_so_ref, label, \
     request_json->>'fondCode' AS fond_code, request_json->>'fondName' AS fond_name, \
     objid, zip_name, zip_sha256, zip_size, document_count, error, virus_scan, \
     submission_id, converted_from, stt_from, stt_to, created_at, updated_at";
pub async fn list_gon(
    r_29i2: &PgPool,
    r_29j2: Option<&[Uuid]>,
    r_29g2: i64,
    r_29h2: i64,
) -> sqlx::Result<Vec<Tgd>> {
    sqlx::query_as::<_, Tgd>(&format!(
        "SELECT {COLS_GON} FROM packages \
         WHERE ($1::uuid[] IS NULL OR project_id = ANY($1)) \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(r_29j2)
    .bind(r_29g2)
    .bind(r_29h2)
    .fetch_all(r_29i2)
    .await
}
pub async fn list(
    r_29i: &PgPool,
    r_29j: Option<&[Uuid]>,
    r_29g: i64,
    r_29h: i64,
) -> sqlx::Result<Vec<Tgc>> {
    sqlx::query_as::<_, Tgc>(&format!(
        "SELECT {COLS} FROM packages \
         WHERE ($1::uuid[] IS NULL OR project_id = ANY($1)) \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(r_29j)
    .bind(r_29g)
    .bind(r_29h)
    .fetch_all(r_29i)
    .await
}
pub async fn claim_next(r_29l: &PgPool, r_29m: &str, r_29k: i64) -> sqlx::Result<Option<Tgc>> {
    // Worker hỏi mỗi vài giây; UPDATE không chạm dòng nào vẫn làm trigger phát sự kiện "packages" cho mọi máy trạm.
    let any: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM packages WHERE status = 'queued')").fetch_one(r_29l).await?;
    if !any {
        return Ok(None);
    }
    sqlx::query_as::<_, Tgc>(&format!(
        "UPDATE packages SET status = 'building', locked_by = $1, attempts = attempts + 1, \
             lease_until = now() + ($2 * interval '1 second'), updated_at = now() \
         WHERE id = ( \
             SELECT id FROM packages WHERE status = 'queued' \
             ORDER BY created_at FOR UPDATE SKIP LOCKED LIMIT 1 \
         ) RETURNING {COLS}"
    ))
    .bind(r_29m)
    .bind(r_29k)
    .fetch_optional(r_29l)
    .await
}
pub struct T1e<'a> {
    pub objid: &'a str,
    pub zip_key: &'a str,
    pub zip_name: &'a str,
    pub zip_sha256: &'a str,
    pub zip_size: i64,
    pub document_count: i32,
}
pub async fn f_1r5(r_29o: &PgPool, id: Uuid, r_29q: &str, r_29n: T1e<'_>) -> sqlx::Result<bool> {
    let r_29p = sqlx::query(
        "UPDATE packages SET status = 'built', objid = $3, zip_key = $4, zip_name = $5, \
             zip_sha256 = $6, zip_size = $7, document_count = $8, error = NULL, \
             locked_by = NULL, lease_until = NULL, updated_at = now() \
         WHERE id = $1 AND locked_by = $2 AND status = 'building'",
    )
    .bind(id)
    .bind(r_29q)
    .bind(r_29n.objid)
    .bind(r_29n.zip_key)
    .bind(r_29n.zip_name)
    .bind(r_29n.zip_sha256)
    .bind(r_29n.zip_size)
    .bind(r_29n.document_count)
    .execute(r_29o)
    .await?;
    Ok(r_29p.rows_affected() > 0)
}
pub async fn f_1rd(r_29r: &PgPool, id: Uuid, r_29t: &str, error: &str) -> sqlx::Result<bool> {
    let r_29s = sqlx::query(
        "UPDATE packages SET status = 'failed', error = $3, locked_by = NULL, \
             lease_until = NULL, updated_at = now() \
         WHERE id = $1 AND locked_by = $2 AND status = 'building'",
    )
    .bind(id)
    .bind(r_29t)
    .bind(error)
    .execute(r_29r)
    .await?;
    Ok(r_29s.rows_affected() > 0)
}
pub async fn f_2ek(r_29v: &PgPool, id: Uuid, r_29u: &str) -> sqlx::Result<bool> {
    let r_29w =
        sqlx::query("UPDATE packages SET virus_scan = $2, updated_at = now() WHERE id = $1")
            .bind(id)
            .bind(r_29u)
            .execute(r_29v)
            .await?;
    Ok(r_29w.rows_affected() > 0)
}
pub async fn f_w8(r_29x: &PgPool, submission_id: Uuid) -> sqlx::Result<Vec<Tgc>> {
    sqlx::query_as::<_, Tgc>(&format!(
        "SELECT {COLS} FROM packages \
         WHERE submission_id = $1 AND status = 'built' AND kind IN ('sip_hoso', 'sip_tailieu') \
         ORDER BY created_at"
    ))
    .bind(submission_id)
    .fetch_all(r_29x)
    .await
}
pub async fn requeue_expired(r_29z: &PgPool, r_29y: i32) -> sqlx::Result<u64> {
    // Trigger AFTER STATEMENT trên `packages` phát sự kiện "packages" kể cả khi UPDATE không chạm dòng nào;
    // worker gọi hàm này mỗi vài giây → mọi máy trạm nạp lại danh sách gói vô ích. Chỉ UPDATE khi thật có gói quá hạn.
    let any: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM packages WHERE status = 'building' AND lease_until < now())")
        .fetch_one(r_29z)
        .await?;
    if !any {
        return Ok(0);
    }
    let r_2a0 = sqlx::query(
            "UPDATE packages SET status = CASE WHEN attempts >= $1 THEN 'failed' ELSE 'queued' END, error = CASE WHEN attempts >= $1 THEN COALESCE(error, 'lease hết hạn — worker không phản hồi') ELSE error END, locked_by = NULL, lease_until = NULL, updated_at = now() WHERE status = 'building' AND lease_until < now()",
        )
        .bind(r_29y)
        .execute(r_29z)
        .await?;
    Ok(r_2a0.rows_affected())
}
