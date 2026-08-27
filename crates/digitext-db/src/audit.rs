// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug)]
pub struct Tp<'a> {
    pub actor_id: Option<Uuid>,
    pub device_id: Option<Uuid>,
    pub action: &'a str,
    pub target: Option<String>,
    pub detail: Option<serde_json::Value>,
    pub ip: Option<String>,
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tq {
    pub id: i64,
    pub actor_id: Option<Uuid>,
    pub device_id: Option<Uuid>,
    pub action: String,
    pub target: Option<String>,
    pub detail: Option<serde_json::Value>,
    pub ip: Option<String>,
    pub at: DateTime<Utc>,
}
pub async fn append(r_21h: &PgPool, r_21g: Tp<'_>) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO audit_log (actor_id, device_id, action, target, detail, ip) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(r_21g.actor_id)
    .bind(r_21g.device_id)
    .bind(r_21g.action)
    .bind(r_21g.target)
    .bind(r_21g.detail)
    .bind(r_21g.ip)
    .execute(r_21h)
    .await?;
    Ok(())
}
pub async fn f_1o8(r_21j: &PgPool, r_21i: i64) -> sqlx::Result<Vec<Tq>> {
    sqlx::query_as::<_, Tq>("SELECT * FROM audit_log ORDER BY id DESC LIMIT $1")
        .bind(r_21i)
        .fetch_all(r_21j)
        .await
}
