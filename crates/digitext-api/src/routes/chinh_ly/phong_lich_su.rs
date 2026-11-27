// Copyright (c) 2026 DigiText. All rights reserved.
//! Lịch sử đơn vị hình thành phông / lịch sử phông có phiên bản (0039, 27/11/2026 — đợt 5, Digifile B02).
//! `GET /cl/phong/{id}/lich-su` (đọc mục lục + trong phạm vi phông) · `POST` (quyền `catalog.update`) thêm phiên bản mới.
use super::{kiem_phong, pham_vi_phong, project_access, read_access};
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use digitext_core::Permission;
use digitext_db::phong_lich_su as db;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/cl/phong/{id}/lich-su", get(xem).post(them))
}

async fn du_an_cua_phong(st: &AppState, phong: Uuid) -> Result<Uuid, ApiError> {
    let p: Option<Uuid> = sqlx::query_scalar("SELECT project_id FROM cl_phong WHERE id = $1 AND deleted_at IS NULL").bind(phong).fetch_optional(&st.pool).await?;
    p.ok_or_else(ApiError::not_found)
}

async fn xem(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    let project = du_an_cua_phong(&st, id).await?;
    read_access(&st, &ts, project).await?;
    kiem_phong(pham_vi_phong(&st, &ts, project).await?.as_deref(), id)?;
    Ok(Json(json!({ "phongId": id, "rows": db::list(&st.pool, id).await? })))
}

async fn them(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<db::Moi>) -> Result<Json<db::LichSuRow>, ApiError> {
    ts.require_perm(Permission::CatalogUpdate)?;
    let project = du_an_cua_phong(&st, id).await?;
    project_access(&st, &ts, project).await?;
    kiem_phong(pham_vi_phong(&st, &ts, project).await?.as_deref(), id)?;
    let row = db::them(&st.pool, project, id, &b, ts.user.id).await?.map_err(ApiError::bad_request)?;
    digitext_db::audit::append(&st.pool, super::audit(&ts, "cl.phong_lich_su", id.to_string(), json!({ "phienBan": row.phien_ban }))).await?;
    Ok(Json(row))
}
