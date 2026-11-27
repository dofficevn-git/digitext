// Copyright (c) 2026 DigiText. All rights reserved.
//! Thống kê chỉnh lý (danh sách phiếu mượn chuyển sang `muon_tra.rs` từ 0034).

use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use digitext_core::Permission;
use digitext_db::chinh_ly::thong_ke as tk;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cl/thong-ke/chinh-ly", get(chinh_ly))
        .route("/cl/thong-ke/nhan-vien", get(nhan_vien))
        .route("/cl/thong-ke/phan-mem", get(phan_mem))
        .route("/cl/thong-ke/muon-tra", get(muon_tra))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Q {
    project_id: Uuid,
    #[serde(default)]
    phong_id: Option<Uuid>,
    /// Ngày VN dd/MM/yyyy hoặc yyyy-MM-dd.
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
}

/// Ngày VN → mốc UTC (đầu ngày +07:00); `to` là ngày kết thúc bao gồm → +1 ngày.
fn vn(s: Option<&str>, end: bool) -> Result<Option<DateTime<Utc>>, ApiError> {
    let Some(s) = s.map(str::trim).filter(|x| !x.is_empty()) else { return Ok(None) };
    let d = NaiveDate::parse_from_str(s, "%d/%m/%Y").or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d")).map_err(|_| ApiError::bad_request(format!("Ngày {s:?} không hợp lệ (dd/MM/yyyy)")))?;
    let d = if end { d.succ_opt().unwrap_or(d) } else { d };
    let t = d.and_hms_opt(0, 0, 0).unwrap().and_utc() - chrono::Duration::hours(7);
    Ok(Some(t))
}

async fn stats_access(st: &AppState, ts: &Ts, project: Uuid) -> Result<(), ApiError> {
    if !(ts.has_perm(Permission::StatsView) || ts.has_perm(Permission::StatsViewProject)) {
        return Err(ApiError::forbidden());
    }
    super::project_access(st, ts, project).await
}

async fn chinh_ly(State(st): State<AppState>, ts: Ts, Query(q): Query<Q>) -> Result<Json<Vec<Value>>, ApiError> {
    stats_access(&st, &ts, q.project_id).await?;
    Ok(Json(tk::chinh_ly(&st.pool, q.project_id, q.phong_id).await?))
}

async fn nhan_vien(State(st): State<AppState>, ts: Ts, Query(q): Query<Q>) -> Result<Json<Vec<Value>>, ApiError> {
    stats_access(&st, &ts, q.project_id).await?;
    Ok(Json(tk::nhan_vien(&st.pool, q.project_id, q.phong_id, vn(q.from.as_deref(), false)?, vn(q.to.as_deref(), true)?).await?))
}

async fn phan_mem(State(st): State<AppState>, ts: Ts, Query(q): Query<Q>) -> Result<Json<Vec<Value>>, ApiError> {
    stats_access(&st, &ts, q.project_id).await?;
    Ok(Json(tk::phan_mem(&st.pool, q.project_id, vn(q.from.as_deref(), false)?, vn(q.to.as_deref(), true)?).await?))
}

async fn muon_tra(State(st): State<AppState>, ts: Ts, Query(q): Query<Q>) -> Result<Json<Value>, ApiError> {
    stats_access(&st, &ts, q.project_id).await?;
    Ok(Json(tk::muon_tra(&st.pool, q.project_id, vn(q.from.as_deref(), false)?, vn(q.to.as_deref(), true)?).await?))
}

