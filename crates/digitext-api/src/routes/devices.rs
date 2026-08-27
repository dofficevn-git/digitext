// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::Permission;
use digitext_db::devices::{self, T4m};
use digitext_db::{audit, tokens};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/devices", get(list_devices))
        .route("/devices/register", post(f_26o))
        .route("/devices/{id}/revoke", post(f_2a0))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDto {
    pub id: Uuid,
    pub name: String,
    pub platform: String,
    pub app_version: Option<String>,
    pub project_id: Option<Uuid>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}
impl From<T4m> for DeviceDto {
    fn from(r_bh: T4m) -> Self {
        DeviceDto {
            id: r_bh.id,
            name: r_bh.name,
            platform: r_bh.platform,
            app_version: r_bh.app_version,
            project_id: r_bh.project_id,
            first_seen: r_bh.first_seen,
            last_seen: r_bh.last_seen,
            revoked_at: r_bh.revoked_at,
        }
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tim {
    #[serde(default)]
    pub device_id: Option<Uuid>,
    pub name: String,
    pub platform: String,
    #[serde(default)]
    pub app_version: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Taw {
    #[serde(default)]
    pub project_id: Option<Uuid>,
}
pub(crate) async fn f_153(
    r_bl: &sqlx::PgPool,
    r_bm: &digitext_db::users::User,
    device_id: Option<Uuid>,
) -> Result<bool, ApiError> {
    let Some(id) = device_id else { return Ok(true) };
    let Some(r_bi) = devices::get(r_bl, id).await? else {
        return Ok(true);
    };
    let r_bk = r_bi.registered_by == Some(r_bm.id);
    let r_bj = r_bm.project_id.is_some() && r_bi.project_id == r_bm.project_id;
    Ok(r_bk || r_bj || f_29a(r_bl, r_bm).await?.allows_opt(r_bi.project_id))
}
pub(crate) async fn f_19w(
    r_bn: &sqlx::PgPool,
    r_bo: &digitext_db::users::User,
    device_id: Option<Uuid>,
) -> Result<(), ApiError> {
    if f_153(r_bn, r_bo, device_id).await? {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}
async fn list_devices(
    State(r_bt): State<AppState>,
    r_bp: Ts,
    Query(r_bs): Query<Taw>,
) -> Result<Json<Vec<DeviceDto>>, ApiError> {
    r_bp.require_perm(Permission::DeviceView)?;
    let r_br = match f_29a(&r_bt.pool, &r_bp.user).await? {
        Scope::All => devices::list(&r_bt.pool, r_bs.project_id).await?,
        Scope::Projects(r_bq) => devices::list_in_projects(&r_bt.pool, &r_bq).await?,
    };
    Ok(Json(r_br.into_iter().map(DeviceDto::from).collect()))
}
async fn f_26o(
    State(r_bw): State<AppState>,
    r_bu: Ts,
    Json(r_bv): Json<Tim>,
) -> Result<Json<DeviceDto>, ApiError> {
    f_19w(&r_bw.pool, &r_bu.user, r_bv.device_id).await?;
    let device = devices::register(
        &r_bw.pool,
        devices::Til {
            id: r_bv.device_id,
            name: &r_bv.name,
            platform: &r_bv.platform,
            app_version: r_bv.app_version.as_deref(),
            project_id: r_bu.user.project_id,
            registered_by: Some(r_bu.user.id),
        },
    )
    .await?;
    if device.is_revoked() {
        return Err(ApiError::device_revoked());
    }
    Ok(Json(device.into()))
}
async fn f_2a0(
    State(r_by): State<AppState>,
    r_bx: Ts,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    r_bx.require_perm(Permission::DeviceRevoke)?;
    let device = devices::get(&r_by.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !f_29a(&r_by.pool, &r_bx.user)
        .await?
        .allows_opt(device.project_id)
    {
        return Err(ApiError::forbidden());
    }
    let mut r_bz = r_by.pool.begin().await?;
    devices::revoke(&mut *r_bz, id).await?;
    tokens::f_29y(&mut *r_bz, id, "device_revoked").await?;
    r_bz.commit().await?;
    audit::append(
        &r_by.pool,
        audit::Tp {
            actor_id: Some(r_bx.user.id),
            device_id: r_bx.claims.device_id,
            action: "device.revoke",
            target: Some(format!("device:{id}")),
            detail: None,
            ip: r_bx.ip(),
        },
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
