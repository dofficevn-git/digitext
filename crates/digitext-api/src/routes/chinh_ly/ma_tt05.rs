// Copyright (c) 2026 DigiText. All rights reserved.
// Bảng mã TT05 (0040, Digifile B03): chế độ sử dụng, mức độ tin cậy, tình trạng vật lý, tình trạng dự phòng.
// GET cho mọi người đọc được danh mục (catalog.read); thêm / sửa / xoá mã riêng của dự án cần catalog.update;
// dòng dùng chung (project_id NULL) chỉ super admin sửa.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Path, Query, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use digitext_core::{Permission, Role};
use digitext_db::ma_tt05;
use serde::Deserialize;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cl/ma-tt05", get(list).post(them))
        .route("/cl/ma-tt05/{id}", put(sua).delete(xoa))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Q {
    pub project_id: Option<Uuid>,
    pub nhom: Option<String>,
}

async fn list(State(st): State<AppState>, ts: Ts, Query(q): Query<Q>) -> Result<Json<serde_json::Value>, ApiError> {
    ts.require_perm(Permission::CatalogRead)?;
    let pid = q.project_id.or(ts.user.project_id).ok_or_else(|| ApiError::bad_request("thiếu projectId"))?;
    super::project_access(&st, &ts, pid).await?;
    if let Some(n) = q.nhom.as_deref() {
        if !ma_tt05::nhom_hop_le(n) {
            return Err(ApiError::bad_request(format!("nhom nhận {}", ma_tt05::NHOM.join(" | "))));
        }
    }
    let rows = ma_tt05::list(&st.pool, pid, q.nhom.as_deref()).await?;
    Ok(Json(serde_json::json!({ "nhom": ma_tt05::NHOM, "rows": rows })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemBody {
    pub project_id: Option<Uuid>,
    /// true = dòng dùng chung cho mọi kho (chỉ super admin).
    #[serde(default)]
    pub dung_chung: bool,
    #[serde(flatten)]
    pub moi: ma_tt05::Moi,
}

fn kiem(m: &ma_tt05::Moi) -> Result<(), ApiError> {
    if !ma_tt05::nhom_hop_le(&m.nhom) {
        return Err(ApiError::bad_request(format!("nhom nhận {}", ma_tt05::NHOM.join(" | "))));
    }
    if m.ma.trim().is_empty() || m.ten.trim().is_empty() {
        return Err(ApiError::bad_request("thiếu mã hoặc tên"));
    }
    Ok(())
}

async fn them(State(st): State<AppState>, ts: Ts, Json(b): Json<ThemBody>) -> Result<Json<ma_tt05::MaRow>, ApiError> {
    ts.require_perm(Permission::CatalogUpdate)?;
    kiem(&b.moi)?;
    let pid = if b.dung_chung {
        if ts.user.role != Role::SuperAdmin {
            return Err(ApiError::forbidden());
        }
        None
    } else {
        let pid = b.project_id.or(ts.user.project_id).ok_or_else(|| ApiError::bad_request("thiếu projectId"))?;
        super::project_access(&st, &ts, pid).await?;
        Some(pid)
    };
    let row = ma_tt05::them(&st.pool, pid, &b.moi, ts.user.id)
        .await
        .map_err(|e| if digitext_db::f_1jn(&e) { ApiError::conflict("mã đã có trong nhóm này") } else { e.into() })?;
    Ok(Json(row))
}

async fn quyen_sua(st: &AppState, ts: &Ts, id: Uuid) -> Result<ma_tt05::MaRow, ApiError> {
    ts.require_perm(Permission::CatalogUpdate)?;
    let row = ma_tt05::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    match row.project_id {
        None if ts.user.role != Role::SuperAdmin => return Err(ApiError::forbidden()),
        None => {}
        Some(p) => {
            super::project_access(st, ts, p).await?;
        }
    }
    Ok(row)
}

async fn sua(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(m): Json<ma_tt05::Moi>) -> Result<Json<ma_tt05::MaRow>, ApiError> {
    let cu = quyen_sua(&st, &ts, id).await?;
    if m.nhom != cu.nhom {
        return Err(ApiError::bad_request("không đổi nhóm của mã đã có"));
    }
    kiem(&m)?;
    let row = ma_tt05::sua(&st.pool, id, &m)
        .await
        .map_err(|e| if digitext_db::f_1jn(&e) { ApiError::conflict("mã đã có trong nhóm này") } else { e.into() })?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(row))
}

async fn xoa(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<axum::http::StatusCode, ApiError> {
    quyen_sua(&st, &ts, id).await?;
    ma_tt05::xoa(&st.pool, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
