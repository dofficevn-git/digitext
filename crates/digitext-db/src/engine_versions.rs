// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T6t {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub precision_mode: String,
    pub active: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
pub async fn insert(
    r_253: &PgPool,
    name: &str,
    role: &str,
    precision_mode: &str,
) -> sqlx::Result<T6t> {
    sqlx::query_as::<_, T6t>(
        "INSERT INTO engine_versions (id, name, role, precision_mode) \
         VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(Uuid::now_v7())
    .bind(name)
    .bind(role)
    .bind(precision_mode)
    .fetch_one(r_253)
    .await
}
pub async fn ensure(
    r_254: &PgPool,
    name: &str,
    role: &str,
    precision_mode: &str,
) -> sqlx::Result<T6t> {
    sqlx::query(
        "INSERT INTO engine_versions (id, name, role, precision_mode) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (name, precision_mode) DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(name)
    .bind(role)
    .bind(precision_mode)
    .execute(r_254)
    .await?;
    sqlx::query_as::<_, T6t>(
        "SELECT * FROM engine_versions WHERE name = $1 AND precision_mode = $2",
    )
    .bind(name)
    .bind(precision_mode)
    .fetch_one(r_254)
    .await
}
pub async fn f_ql(r_255: &PgPool, role: &str) -> sqlx::Result<Option<T6t>> {
    sqlx::query_as::<_, T6t>(
        "SELECT * FROM engine_versions WHERE role = $1 AND active = TRUE \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(role)
    .fetch_optional(r_255)
    .await
}
/// Tắt mọi dòng cùng `role` trừ `keep` — dùng khi chuyển engine tường minh.
pub async fn deactivate_others(r_256: &PgPool, role: &str, keep: Uuid) -> sqlx::Result<u64> {
    let r = sqlx::query(
        "UPDATE engine_versions SET active = FALSE WHERE role = $1 AND id <> $2 AND active",
    )
    .bind(role)
    .bind(keep)
    .execute(r_256)
    .await?;
    Ok(r.rows_affected())
}
pub async fn f_2dw(r_256: &PgPool, id: Uuid, active: bool) -> sqlx::Result<bool> {
    let r_257 = sqlx::query("UPDATE engine_versions SET active = $2 WHERE id = $1")
        .bind(id)
        .bind(active)
        .execute(r_256)
        .await?;
    Ok(r_257.rows_affected() > 0)
}
