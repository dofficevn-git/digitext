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
/// Băm một dòng nhật ký nối với dòng trước (0040, Digifile H08): sửa / xoá một dòng là đứt chuỗi.
pub fn bam_dong(prev: Option<&str>, actor: Option<Uuid>, device: Option<Uuid>, action: &str, target: Option<&str>, detail: Option<&serde_json::Value>, ip: Option<&str>, at: &DateTime<Utc>) -> String {
    let mut h = blake3::Hasher::new();
    for phan in [
        prev.unwrap_or(""),
        &actor.map(|u| u.to_string()).unwrap_or_default(),
        &device.map(|u| u.to_string()).unwrap_or_default(),
        action,
        target.unwrap_or(""),
        &detail.map(|d| d.to_string()).unwrap_or_default(),
        ip.unwrap_or(""),
        &at.to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
    ] {
        h.update(phan.as_bytes());
        h.update(b"|");
    }
    h.finalize().to_hex().to_string()
}
pub async fn append(r_21h: &PgPool, r_21g: Tp<'_>) -> sqlx::Result<()> {
    let mut tx = r_21h.begin().await?;
    // Khoá nhẹ để hai dòng cùng lúc không cùng nối vào một prev_hash.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext('audit_log_chain'))").execute(&mut *tx).await?;
    let prev: Option<String> = sqlx::query_scalar("SELECT hash FROM audit_log WHERE hash IS NOT NULL ORDER BY id DESC LIMIT 1")
        .fetch_optional(&mut *tx)
        .await?
        .flatten();
    let at = Utc::now();
    let hash = bam_dong(prev.as_deref(), r_21g.actor_id, r_21g.device_id, r_21g.action, r_21g.target.as_deref(), r_21g.detail.as_ref(), r_21g.ip.as_deref(), &at);
    sqlx::query(
        "INSERT INTO audit_log (actor_id, device_id, action, target, detail, ip, at, prev_hash, hash) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(r_21g.actor_id)
    .bind(r_21g.device_id)
    .bind(r_21g.action)
    .bind(r_21g.target)
    .bind(r_21g.detail)
    .bind(r_21g.ip)
    .bind(at)
    .bind(prev)
    .bind(hash)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KetQuaChuoi {
    pub da_kiem: i64,
    pub chua_bam: i64,
    pub dong_loi: Option<i64>,
    pub hash_cuoi: Option<String>,
}
/// Kiểm lại chuỗi băm từ dòng có hash đầu tiên: trả dòng đầu tiên không khớp (đã bị sửa / chen / xoá) nếu có.
pub async fn kiem_chuoi(pool: &PgPool, limit: i64) -> sqlx::Result<KetQuaChuoi> {
    #[derive(sqlx::FromRow)]
    struct D { id: i64, actor_id: Option<Uuid>, device_id: Option<Uuid>, action: String, target: Option<String>, detail: Option<serde_json::Value>, ip: Option<String>, at: DateTime<Utc>, prev_hash: Option<String>, hash: Option<String> }
    let rows = sqlx::query_as::<_, D>(
        "SELECT id, actor_id, device_id, action, target, detail, ip, at, prev_hash, hash FROM audit_log \
         WHERE id >= COALESCE((SELECT min(id) FROM audit_log WHERE hash IS NOT NULL), 0) ORDER BY id LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    let chua_bam: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_log WHERE hash IS NULL").fetch_one(pool).await?;
    let mut prev: Option<String> = None;
    let mut da_kiem = 0i64;
    for r in &rows {
        let Some(h) = r.hash.as_deref() else { continue };
        let tinh = bam_dong(prev.as_deref(), r.actor_id, r.device_id, &r.action, r.target.as_deref(), r.detail.as_ref(), r.ip.as_deref(), &r.at);
        if tinh != h || r.prev_hash != prev {
            return Ok(KetQuaChuoi { da_kiem, chua_bam, dong_loi: Some(r.id), hash_cuoi: prev });
        }
        prev = Some(h.to_string());
        da_kiem += 1;
    }
    Ok(KetQuaChuoi { da_kiem, chua_bam, dong_loi: None, hash_cuoi: prev })
}
pub async fn f_1o8(r_21j: &PgPool, r_21i: i64) -> sqlx::Result<Vec<Tq>> {
    sqlx::query_as::<_, Tq>("SELECT * FROM audit_log ORDER BY id DESC LIMIT $1")
        .bind(r_21i)
        .fetch_all(r_21j)
        .await
}
