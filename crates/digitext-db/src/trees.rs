// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Ths {
    pub project_id: Uuid,
    pub version: i32,
    pub payload: Value,
    pub published_by: Option<Uuid>,
    pub published_at: DateTime<Utc>,
}
pub async fn upsert(
    r_2eg: &PgPool,
    project_id: Uuid,
    payload: &Value,
    published_by: Option<Uuid>,
) -> sqlx::Result<i32> {
    sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_trees (project_id, version, payload, published_by) \
         VALUES ($1, 1, $2, $3) \
         ON CONFLICT (project_id) DO UPDATE SET \
            version = project_trees.version + 1, \
            payload = EXCLUDED.payload, \
            published_by = EXCLUDED.published_by, \
            published_at = now() \
         RETURNING version",
    )
    .bind(project_id)
    .bind(payload)
    .bind(published_by)
    .fetch_one(r_2eg)
    .await
}
pub async fn get(r_2eh: &PgPool, project_id: Uuid) -> sqlx::Result<Option<Ths>> {
    sqlx::query_as::<_, Ths>("SELECT * FROM project_trees WHERE project_id = $1")
        .bind(project_id)
        .fetch_optional(r_2eh)
        .await
}
