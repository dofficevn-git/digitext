// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::Permission;
use digitext_db::stats::{self, StatRow, Tm0};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new().route("/stats", post(f_28a).get(f_1od))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tiw {
    #[serde(default)]
    pub pages_scanned: i64,
    #[serde(default)]
    pub ho_so_count: i64,
    #[serde(default)]
    pub pdf_count: i64,
    #[serde(default)]
    pub pdf_bytes: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatDto {
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
impl From<StatRow> for StatDto {
    fn from(r_rz: StatRow) -> Self {
        StatDto {
            user_id: r_rz.user_id,
            username: r_rz.username,
            display_name: r_rz.display_name,
            project_id: r_rz.project_id,
            pages_scanned: r_rz.pages_scanned,
            ho_so_count: r_rz.ho_so_count,
            pdf_count: r_rz.pdf_count,
            pdf_bytes: r_rz.pdf_bytes,
            files_uploaded: r_rz.files_uploaded,
            bytes_uploaded: r_rz.bytes_uploaded,
            reported_at: r_rz.reported_at,
        }
    }
}
async fn f_28a(
    State(r_s2): State<AppState>,
    r_s0: Ts,
    Json(r_s1): Json<Tiw>,
) -> Result<StatusCode, ApiError> {
    r_s0.require_perm(Permission::StatsView)?;
    stats::report(
        &r_s2.pool,
        r_s0.user.id,
        r_s0.user.project_id,
        Tm0 {
            pages_scanned: r_s1.pages_scanned.max(0),
            ho_so_count: r_s1.ho_so_count.max(0),
            pdf_count: r_s1.pdf_count.max(0),
            pdf_bytes: r_s1.pdf_bytes.max(0),
        },
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn f_1od(State(r_s6): State<AppState>, r_s3: Ts) -> Result<Json<Vec<StatDto>>, ApiError> {
    r_s3.require_perm(Permission::StatsViewProject)?;
    let r_s5 = match f_29a(&r_s6.pool, &r_s3.user).await? {
        Scope::All => stats::list_all(&r_s6.pool).await?,
        Scope::Projects(r_s4) => stats::list_for_projects(&r_s6.pool, &r_s4).await?,
    };
    Ok(Json(r_s5.into_iter().map(StatDto::from).collect()))
}
