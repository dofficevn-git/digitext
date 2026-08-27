// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T4m {
    pub id: Uuid,
    pub name: String,
    pub platform: String,
    pub app_version: Option<String>,
    pub project_id: Option<Uuid>,
    pub registered_by: Option<Uuid>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}
impl T4m {
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }
}
#[derive(Debug)]
pub struct Til<'a> {
    pub id: Option<Uuid>,
    pub name: &'a str,
    pub platform: &'a str,
    pub app_version: Option<&'a str>,
    pub project_id: Option<Uuid>,
    pub registered_by: Option<Uuid>,
}
pub async fn register(r_21u: &PgPool, r_21v: Til<'_>) -> sqlx::Result<T4m> {
    sqlx::query_as::<
        _,
        T4m,
    >(
            "INSERT INTO devices (id, name, platform, app_version, project_id, registered_by, last_seen) \
         VALUES ($1, $2, $3, $4, $5, $6, now()) \
         ON CONFLICT (id) DO UPDATE SET \
            name = EXCLUDED.name, \
            platform = EXCLUDED.platform, \
            app_version = EXCLUDED.app_version, \
            project_id = EXCLUDED.project_id, \
            last_seen = now() \
         RETURNING *",
        )
        .bind(r_21v.id.unwrap_or_else(Uuid::now_v7))
        .bind(r_21v.name)
        .bind(r_21v.platform)
        .bind(r_21v.app_version)
        .bind(r_21v.project_id)
        .bind(r_21v.registered_by)
        .fetch_one(r_21u)
        .await
}
pub async fn get(r_21w: &PgPool, id: Uuid) -> sqlx::Result<Option<T4m>> {
    sqlx::query_as::<_, T4m>("SELECT * FROM devices WHERE id = $1")
        .bind(id)
        .fetch_optional(r_21w)
        .await
}
pub async fn list(r_21x: &PgPool, project_id: Option<Uuid>) -> sqlx::Result<Vec<T4m>> {
    sqlx::query_as::<_, T4m>(
        "SELECT * FROM devices WHERE ($1::uuid IS NULL OR project_id = $1) ORDER BY first_seen",
    )
    .bind(project_id)
    .fetch_all(r_21x)
    .await
}
pub async fn list_in_projects(r_21y: &PgPool, r_21z: &[Uuid]) -> sqlx::Result<Vec<T4m>> {
    sqlx::query_as::<_, T4m>("SELECT * FROM devices WHERE project_id = ANY($1) ORDER BY first_seen")
        .bind(r_21z)
        .fetch_all(r_21y)
        .await
}
pub async fn touch(r_220: &PgPool, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE devices SET last_seen = now() WHERE id = $1")
        .bind(id)
        .execute(r_220)
        .await?;
    Ok(())
}
pub async fn revoke<'e, E>(r_221: E, id: Uuid) -> sqlx::Result<Option<T4m>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, T4m>("UPDATE devices SET revoked_at = now() WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_optional(r_221)
        .await
}
