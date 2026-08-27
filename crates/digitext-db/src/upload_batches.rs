// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tp8 {
    pub id: Uuid,
    pub forwarded_by: Option<Uuid>,
    pub device_id: Option<Uuid>,
    pub note: Option<String>,
    pub status: String,
    pub forwarded: i32,
    pub skipped: i32,
    pub failed: i32,
    pub rejected: i32,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T18 {
    pub id: Uuid,
    pub forwarded_by: Option<Uuid>,
    pub forwarded_by_name: Option<String>,
    pub device_id: Option<Uuid>,
    pub note: Option<String>,
    pub status: String,
    pub forwarded: i32,
    pub skipped: i32,
    pub failed: i32,
    pub rejected: i32,
    pub received: i64,
    pub project_count: i64,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
pub async fn open(
    r_2ei: &PgPool,
    forwarded_by: Option<Uuid>,
    device_id: Option<Uuid>,
    note: Option<&str>,
) -> sqlx::Result<Tp8> {
    sqlx::query_as::<_, Tp8>(
        "INSERT INTO upload_batches (id, forwarded_by, device_id, note) \
         VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(Uuid::now_v7())
    .bind(forwarded_by)
    .bind(device_id)
    .bind(note)
    .fetch_one(r_2ei)
    .await
}
pub async fn complete(
    r_2ej: &PgPool,
    id: Uuid,
    forwarded_by: Option<Uuid>,
    forwarded: i32,
    skipped: i32,
    failed: i32,
    rejected: i32,
) -> sqlx::Result<Option<Tp8>> {
    sqlx::query_as::<_, Tp8>(
        "UPDATE upload_batches SET \
            status = 'completed', forwarded = $3, skipped = $4, failed = $5, rejected = $6, \
            completed_at = now() \
         WHERE id = $1 AND forwarded_by IS NOT DISTINCT FROM $2 AND status = 'open' \
         RETURNING *",
    )
    .bind(id)
    .bind(forwarded_by)
    .bind(forwarded)
    .bind(skipped)
    .bind(failed)
    .bind(rejected)
    .fetch_optional(r_2ej)
    .await
}
pub async fn f_qf(r_2ek: &PgPool, r_2em: i32) -> sqlx::Result<u64> {
    let r_2el = sqlx::query(
        "UPDATE upload_batches b SET status = 'abandoned', completed_at = now() \
         WHERE b.status = 'open' \
           AND b.created_at < now() - make_interval(hours => $1) \
           AND NOT EXISTS ( \
                 SELECT 1 FROM documents d \
                 WHERE d.batch_id = b.id \
                   AND d.uploaded_at > now() - make_interval(hours => $1))",
    )
    .bind(r_2em)
    .execute(r_2ek)
    .await?;
    Ok(r_2el.rows_affected())
}
pub async fn f_1j9(r_2en: &PgPool, id: Uuid, forwarded_by: Option<Uuid>) -> sqlx::Result<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM upload_batches \
         WHERE id = $1 AND forwarded_by IS NOT DISTINCT FROM $2 AND status = 'open')",
    )
    .bind(id)
    .bind(forwarded_by)
    .fetch_one(r_2en)
    .await
}
const SELECT_ROW: &str = "SELECT b.id, b.forwarded_by, u.display_name AS forwarded_by_name, \
        b.device_id, b.note, b.status, b.forwarded, b.skipped, b.failed, b.rejected, \
        (SELECT count(*) FROM documents d WHERE d.batch_id = b.id) AS received, \
        (SELECT count(DISTINCT d.project_id) FROM documents d WHERE d.batch_id = b.id) \
            AS project_count, \
        b.created_at, b.completed_at \
     FROM upload_batches b LEFT JOIN users u ON u.id = b.forwarded_by";
pub async fn list(r_2eq: &PgPool, r_2ep: Option<Uuid>, r_2eo: i64) -> sqlx::Result<Vec<T18>> {
    sqlx::query_as::<_, T18>(&format!(
        "{SELECT_ROW} WHERE ($1::uuid IS NULL OR b.forwarded_by = $1) \
         ORDER BY b.created_at DESC LIMIT $2"
    ))
    .bind(r_2ep)
    .bind(r_2eo)
    .fetch_all(r_2eq)
    .await
}
pub async fn f_1ev(r_2er: &PgPool, id: Uuid) -> sqlx::Result<Option<T18>> {
    sqlx::query_as::<_, T18>(&format!("{SELECT_ROW} WHERE b.id = $1"))
        .bind(id)
        .fetch_optional(r_2er)
        .await
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Td3 {
    pub project_id: Uuid,
    pub project_code: String,
    pub project_name: String,
    pub admin_id: Uuid,
    pub admin_name: Option<String>,
    pub count: i64,
}
pub async fn matrix(r_2et: &PgPool, projects: Option<&[Uuid]>) -> sqlx::Result<Vec<Td3>> {
    let r_2es = projects.is_none();
    let r_2eu: &[Uuid] = projects.unwrap_or(&[]);
    sqlx::query_as::<_, Td3>(
        "SELECT d.project_id, p.code AS project_code, p.name AS project_name, \
                b.forwarded_by AS admin_id, u.display_name AS admin_name, \
                count(*) AS count \
         FROM documents d \
         JOIN upload_batches b ON b.id = d.batch_id \
         JOIN projects p ON p.id = d.project_id \
         LEFT JOIN users u ON u.id = b.forwarded_by \
         WHERE b.forwarded_by IS NOT NULL \
           AND ($1 OR d.project_id = ANY($2)) \
         GROUP BY d.project_id, p.code, p.name, b.forwarded_by, u.display_name \
         ORDER BY p.code, u.display_name",
    )
    .bind(r_2es)
    .bind(r_2eu)
    .fetch_all(r_2et)
    .await
}
pub async fn f_rf(r_2ev: &PgPool) -> sqlx::Result<Vec<Uuid>> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT b.forwarded_by FROM upload_batches b \
         WHERE b.forwarded_by IS NOT NULL AND b.status = 'completed' \
           AND b.forwarded > (SELECT count(*) FROM documents d WHERE d.batch_id = b.id)",
    )
    .fetch_all(r_2ev)
    .await
}
