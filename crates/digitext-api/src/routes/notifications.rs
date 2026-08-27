// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_db::notifications::{self, Tec};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/notifications", get(f_1o3))
        .route("/notifications/{id}/read", post(f_25v))
        .route("/notifications/read-all", post(f_25m))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationDto {
    pub id: Uuid,
    pub submission_id: Option<Uuid>,
    pub kind: String,
    pub message: String,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
impl From<Tec> for NotificationDto {
    fn from(r_ka: Tec) -> Self {
        NotificationDto {
            id: r_ka.id,
            submission_id: r_ka.submission_id,
            kind: r_ka.kind,
            message: r_ka.message,
            read_at: r_ka.read_at,
            created_at: r_ka.created_at,
        }
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Teb {
    pub unread: i64,
    pub items: Vec<NotificationDto>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub limit: Option<i64>,
}
async fn f_1o3(
    State(r_kd): State<AppState>,
    r_kb: Ts,
    Query(r_kc): Query<ListQuery>,
) -> Result<Json<Teb>, ApiError> {
    let limit = r_kc.limit.unwrap_or(50).clamp(1, 200);
    let items = notifications::f_1nk(&r_kd.pool, r_kb.user.id, limit).await?;
    let unread = notifications::f_2n9(&r_kd.pool, r_kb.user.id).await?;
    Ok(Json(Teb {
        unread,
        items: items.into_iter().map(NotificationDto::from).collect(),
    }))
}
async fn f_25v(
    State(r_kf): State<AppState>,
    r_ke: Ts,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !notifications::f_1rg(&r_kf.pool, r_ke.user.id, id).await? {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}
async fn f_25m(State(r_kh): State<AppState>, r_kg: Ts) -> Result<StatusCode, ApiError> {
    notifications::f_1r4(&r_kh.pool, r_kg.user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
