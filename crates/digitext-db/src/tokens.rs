// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tij {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Option<Uuid>,
    pub token_hash: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub rotated_from: Option<Uuid>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
}
impl Tij {
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        self.revoked_at.is_none() && self.expires_at > now
    }
}
pub async fn insert(
    r_2e2: &PgPool,
    user_id: Uuid,
    device_id: Option<Uuid>,
    token_hash: &str,
    expires_at: DateTime<Utc>,
    rotated_from: Option<Uuid>,
) -> sqlx::Result<Tij> {
    sqlx::query_as::<
        _,
        Tij,
    >(
            "INSERT INTO refresh_tokens (id, user_id, device_id, token_hash, expires_at, rotated_from) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        )
        .bind(Uuid::now_v7())
        .bind(user_id)
        .bind(device_id)
        .bind(token_hash)
        .bind(expires_at)
        .bind(rotated_from)
        .fetch_one(r_2e2)
        .await
}
pub async fn find_by_hash(r_2e3: &PgPool, token_hash: &str) -> sqlx::Result<Option<Tij>> {
    sqlx::query_as::<_, Tij>("SELECT * FROM refresh_tokens WHERE token_hash = $1")
        .bind(token_hash)
        .fetch_optional(r_2e3)
        .await
}
/// Token này đã sinh token kế tiếp (xoay xong trọn vẹn) chưa? `false` = đã đánh dấu `rotated` mà
/// chưa cấp được token mới (CSDL nghẽn giữa hai bước, 19/10/2026) — client vẫn giữ đúng token này.
pub async fn has_child(pool: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM refresh_tokens WHERE rotated_from = $1)")
        .bind(id)
        .fetch_one(pool)
        .await
}
pub async fn revoke(r_2e4: &PgPool, id: Uuid, r_2e5: &str) -> sqlx::Result<bool> {
    let r_2e6 = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = now(), revoked_reason = $2 \
         WHERE id = $1 AND revoked_at IS NULL",
    )
    .bind(id)
    .bind(r_2e5)
    .execute(r_2e4)
    .await?;
    Ok(r_2e6.rows_affected() > 0)
}
pub async fn f_29z(r_2e7: &PgPool, user_id: Uuid, r_2e8: &str) -> sqlx::Result<u64> {
    let r_2e9 = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = now(), revoked_reason = $2 \
         WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user_id)
    .bind(r_2e8)
    .execute(r_2e7)
    .await?;
    Ok(r_2e9.rows_affected())
}
pub async fn f_29y<'e, E>(r_2ea: E, device_id: Uuid, r_2eb: &str) -> sqlx::Result<u64>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let r_2ec = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = now(), revoked_reason = $2 \
         WHERE device_id = $1 AND revoked_at IS NULL",
    )
    .bind(device_id)
    .bind(r_2eb)
    .execute(r_2ea)
    .await?;
    Ok(r_2ec.rows_affected())
}
pub async fn f_2a1(
    r_2ed: &PgPool,
    user_id: Uuid,
    device_id: Option<Uuid>,
    r_2ee: &str,
) -> sqlx::Result<u64> {
    let r_2ef = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = now(), revoked_reason = $3 \
         WHERE user_id = $1 AND revoked_at IS NULL \
           AND ($2::uuid IS NULL OR device_id = $2)",
    )
    .bind(user_id)
    .bind(device_id)
    .bind(r_2ee)
    .execute(r_2ed)
    .await?;
    Ok(r_2ef.rows_affected())
}
