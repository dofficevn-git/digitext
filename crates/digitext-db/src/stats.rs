// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, Copy)]
pub struct Tm0 {
    pub pages_scanned: i64,
    pub ho_so_count: i64,
    pub pdf_count: i64,
    pub pdf_bytes: i64,
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StatRow {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub project_id: Option<Uuid>,
    pub pages_scanned: i64,
    pub ho_so_count: i64,
    pub pdf_count: i64,
    pub pdf_bytes: i64,
    pub files_uploaded: i64,
    pub bytes_uploaded: i64,
    pub reported_at: Option<DateTime<Utc>>,
}
pub async fn report(
    r_2cb: &PgPool,
    user_id: Uuid,
    project_id: Option<Uuid>,
    s: Tm0,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO user_stats \
            (user_id, project_id, pages_scanned, ho_so_count, pdf_count, pdf_bytes, reported_at) \
         VALUES ($1, $2, $3, $4, $5, $6, now()) \
         ON CONFLICT (user_id) DO UPDATE SET \
            project_id = EXCLUDED.project_id, \
            pages_scanned = EXCLUDED.pages_scanned, \
            ho_so_count = EXCLUDED.ho_so_count, \
            pdf_count = EXCLUDED.pdf_count, \
            pdf_bytes = EXCLUDED.pdf_bytes, \
            reported_at = now()",
    )
    .bind(user_id)
    .bind(project_id)
    .bind(s.pages_scanned)
    .bind(s.ho_so_count)
    .bind(s.pdf_count)
    .bind(s.pdf_bytes)
    .execute(r_2cb)
    .await?;
    Ok(())
}
const STAT_SELECT: &str = "SELECT u.id AS user_id, u.username, u.display_name, u.project_id, \
     COALESCE(s.pages_scanned, 0) AS pages_scanned, \
     COALESCE(s.ho_so_count, 0) AS ho_so_count, \
     COALESCE(s.pdf_count, 0) AS pdf_count, \
     COALESCE(s.pdf_bytes, 0) AS pdf_bytes, \
     COALESCE(d.files, 0) AS files_uploaded, \
     COALESCE(d.bytes, 0) AS bytes_uploaded, \
     s.reported_at \
   FROM users u \
   LEFT JOIN user_stats s ON s.user_id = u.id \
   LEFT JOIN (SELECT uploaded_by, project_id, COUNT(*) AS files, \
                     COALESCE(SUM(size_bytes), 0)::BIGINT AS bytes \
              FROM documents GROUP BY uploaded_by, project_id) d \
          ON d.uploaded_by = u.id AND d.project_id = u.project_id \
   WHERE u.role = 'user' AND u.is_active \
     AND (s.user_id IS NOT NULL OR d.uploaded_by IS NOT NULL)";
pub async fn list_all(r_2cc: &PgPool) -> sqlx::Result<Vec<StatRow>> {
    sqlx::query_as::<_, StatRow>(&format!("{STAT_SELECT} ORDER BY u.username"))
        .fetch_all(r_2cc)
        .await
}
pub async fn list_for_projects(r_2cd: &PgPool, r_2ce: &[Uuid]) -> sqlx::Result<Vec<StatRow>> {
    sqlx::query_as::<_, StatRow>(&format!(
        "{STAT_SELECT} AND u.project_id = ANY($1) ORDER BY u.username"
    ))
    .bind(r_2ce)
    .fetch_all(r_2cd)
    .await
}
