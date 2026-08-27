// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
pub const C_I3: &str = "low_confidence";
pub const C_I1: &str = "heuristic_flag";
pub const C_I2: &str = "llm_disagreement";
pub const C_I4: &str = "page_failed";
pub const C_I5: &str = "vlm_fallback";
pub const C_I0: &str = "empty_page";
pub const C_I6: &str = "vlm_primary";
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tj7 {
    pub id: Uuid,
    pub document_id: Uuid,
    pub page_no: i32,
    pub reason: String,
    pub status: String,
    pub assigned_to: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}
const COLS: &str = "id, document_id, page_no, reason, status, assigned_to, created_at, resolved_at";
pub async fn enqueue(
    r_2bc: &PgPool,
    document_id: Uuid,
    page_no: i32,
    reason: &str,
) -> sqlx::Result<bool> {
    let r_2bd = sqlx::query(
        "INSERT INTO review_queue (id, document_id, page_no, reason) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (document_id, page_no) WHERE status IN ('open', 'in_review') DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(document_id)
    .bind(page_no)
    .bind(reason)
    .execute(r_2bc)
    .await?;
    Ok(r_2bd.rows_affected() > 0)
}
pub async fn f_1o4(r_2bf: &PgPool, r_2be: i64) -> sqlx::Result<Vec<Tj7>> {
    sqlx::query_as::<_, Tj7>(&format!(
        "SELECT {COLS} FROM review_queue WHERE status = 'open' ORDER BY created_at LIMIT $1"
    ))
    .bind(r_2be)
    .fetch_all(r_2bf)
    .await
}
pub async fn claim(r_2bg: &PgPool, id: Uuid, r_2bi: Uuid) -> sqlx::Result<bool> {
    let r_2bh = sqlx::query(
        "UPDATE review_queue SET status = 'in_review', assigned_to = $2 \
         WHERE id = $1 AND status = 'open'",
    )
    .bind(id)
    .bind(r_2bi)
    .execute(r_2bg)
    .await?;
    Ok(r_2bh.rows_affected() > 0)
}
pub async fn close(r_2bj: &PgPool, id: Uuid, resolved: bool) -> sqlx::Result<bool> {
    let status = if resolved { "resolved" } else { "skipped" };
    let r_2bk = sqlx::query(
        "UPDATE review_queue SET status = $2, resolved_at = now() \
         WHERE id = $1 AND status IN ('open', 'in_review')",
    )
    .bind(id)
    .bind(status)
    .execute(r_2bj)
    .await?;
    Ok(r_2bk.rows_affected() > 0)
}
pub async fn count_by_status(r_2bl: &PgPool) -> sqlx::Result<Vec<(String, i64)>> {
    sqlx::query_as::<_, (String, i64)>("SELECT status, count(*) FROM review_queue GROUP BY status")
        .fetch_all(r_2bl)
        .await
}
