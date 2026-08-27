// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Datelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RecordRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub created_by: Option<Uuid>,
    pub kind: String,
    pub so_bien_ban: String,
    pub title: String,
    pub submission_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub pdf_key: String,
    pub created_at: DateTime<Utc>,
}
const COLS: &str =
    "id, project_id, created_by, kind, so_bien_ban, title, submission_id, payload, pdf_key, \
     created_at";
pub fn f_1m5(kind: &str) -> &'static str {
    match kind {
        "kiem_tra" => "KT",
        "nhat_ky_sao_luu" => "NK",
        "sao_luu" => "SL",
        "su_co" => "SC",
        "thu_nop" => "TN",
        "thu_nop_truc_tiep" => "TT",
        "giao_nhan" => "GN",
        "tra_lai" => "TL",
        _ => "BB",
    }
}
pub struct Tdz<'a> {
    pub project_id: Uuid,
    pub created_by: Option<Uuid>,
    pub kind: &'a str,
    pub title: &'a str,
    pub submission_id: Option<Uuid>,
    pub payload: &'a serde_json::Value,
    pub pdf_key: &'a str,
}
pub async fn create(r_2b1: &PgPool, r_2b2: Tdz<'_>) -> sqlx::Result<RecordRow> {
    let mut r_2b5 = r_2b1.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))")
        .bind(r_2b2.project_id)
        .execute(&mut *r_2b5)
        .await?;
    let year = Utc::now().year();
    let r_2b4: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM records \
         WHERE project_id = $1 AND date_part('year', created_at) = $2",
    )
    .bind(r_2b2.project_id)
    .bind(year as f64)
    .fetch_one(&mut *r_2b5)
    .await?;
    let so_bien_ban = format!("{:02}/BB-{}.{}", r_2b4 + 1, f_1m5(r_2b2.kind), year);
    let r_2b3 = sqlx::query_as::<_, RecordRow>(&format!(
        "INSERT INTO records (id, project_id, created_by, kind, so_bien_ban, title, \
             submission_id, payload, pdf_key) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING {COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(r_2b2.project_id)
    .bind(r_2b2.created_by)
    .bind(r_2b2.kind)
    .bind(&so_bien_ban)
    .bind(r_2b2.title)
    .bind(r_2b2.submission_id)
    .bind(r_2b2.payload)
    .bind(r_2b2.pdf_key)
    .fetch_one(&mut *r_2b5)
    .await?;
    r_2b5.commit().await?;
    Ok(r_2b3)
}
pub async fn get(r_2b6: &PgPool, id: Uuid) -> sqlx::Result<Option<RecordRow>> {
    sqlx::query_as::<_, RecordRow>(&format!("SELECT {COLS} FROM records WHERE id = $1"))
        .bind(id)
        .fetch_optional(r_2b6)
        .await
}
pub async fn list(
    r_2b9: &PgPool,
    r_2ba: Option<&[Uuid]>,
    r_2b7: i64,
    r_2b8: i64,
) -> sqlx::Result<Vec<RecordRow>> {
    sqlx::query_as::<_, RecordRow>(&format!(
        "SELECT {COLS} FROM records \
         WHERE ($1::uuid[] IS NULL OR project_id = ANY($1)) \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(r_2ba)
    .bind(r_2b7)
    .bind(r_2b8)
    .fetch_all(r_2b9)
    .await
}
pub async fn list_for_submission(
    r_2bb: &PgPool,
    submission_id: Uuid,
) -> sqlx::Result<Vec<RecordRow>> {
    sqlx::query_as::<_, RecordRow>(&format!(
        "SELECT {COLS} FROM records WHERE submission_id = $1 ORDER BY created_at"
    ))
    .bind(submission_id)
    .fetch_all(r_2bb)
    .await
}
