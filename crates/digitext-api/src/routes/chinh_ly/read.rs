// Copyright (c) 2026 DigiText. All rights reserved.
//! Route đọc: danh mục, danh sách / chi tiết hồ sơ, văn bản, lịch sử.

use super::read_access;
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use digitext_db::chinh_ly::{self as cl, ho_so, lich_su};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cl/catalog", get(catalog))
        .route("/cl/ho-so", get(list_ho_so))
        .route("/cl/ho-so/{id}", get(get_ho_so))
        .route("/cl/ho-so/{id}/van-ban", get(van_ban_of))
        .route("/cl/van-ban", get(search_van_ban))
        .route("/cl/lich-su", get(list_lich_su))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectQ {
    project_id: Uuid,
}

async fn catalog(State(st): State<AppState>, ts: Ts, Query(q): Query<ProjectQ>) -> Result<Json<Value>, ApiError> {
    read_access(&st, &ts, q.project_id).await?;
    Ok(Json(Value::Object(cl::danh_muc::catalog(&st.pool, q.project_id).await?)))
}

fn csv_uuids(s: &Option<String>) -> Vec<Uuid> {
    s.as_deref().unwrap_or("").split(',').filter_map(|p| Uuid::parse_str(p.trim()).ok()).collect()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HoSoQ {
    project_id: Uuid,
    #[serde(default)]
    phong_id: Option<Uuid>,
    #[serde(default)]
    phong_ban_id: Option<Uuid>,
    #[serde(default)]
    mang_id: Option<Uuid>,
    /// CSV id THBQ; `0` = mọi loại có thời hạn; `-1`/rỗng = tất cả.
    #[serde(default)]
    thbq: Option<String>,
    /// VV | GL | HET
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    trung_thua: Option<bool>,
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    kho_id: Option<Uuid>,
    #[serde(default)]
    gia_id: Option<Uuid>,
    #[serde(default)]
    has_hop: Option<bool>,
    #[serde(default)]
    has_ho_so_so: Option<bool>,
    #[serde(default)]
    nam: Option<i32>,
    #[serde(default)]
    hop_id: Option<Uuid>,
    #[serde(default)]
    ids: Option<String>,
    #[serde(default)]
    include_deleted: bool,
    #[serde(default)]
    sort: Option<String>,
    #[serde(default)]
    desc: bool,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}
fn default_limit() -> i64 {
    200
}

async fn list_ho_so(State(st): State<AppState>, ts: Ts, Query(q): Query<HoSoQ>) -> Result<Json<Value>, ApiError> {
    read_access(&st, &ts, q.project_id).await?;
    let thbq_raw = q.thbq.as_deref().unwrap_or("").trim().to_owned();
    let f = ho_so::Filter {
        project_id: q.project_id,
        phong_id: q.phong_id,
        phong_ban_id: q.phong_ban_id,
        mang_id: q.mang_id,
        thbq_ids: if thbq_raw == "0" || thbq_raw == "-1" { vec![] } else { csv_uuids(&q.thbq) },
        thbq_co_thoi_han: thbq_raw == "0",
        status: q.status.filter(|s| matches!(s.as_str(), "VV" | "GL" | "HET")),
        trung_thua: q.trung_thua,
        q: q.q,
        kho_id: q.kho_id,
        gia_id: q.gia_id,
        has_hop: q.has_hop,
        has_ho_so_so: q.has_ho_so_so,
        nam: q.nam,
        hop_id: q.hop_id,
        ids: csv_uuids(&q.ids),
        include_deleted: q.include_deleted,
        sort: q.sort,
        desc: q.desc,
        limit: q.limit,
        offset: q.offset,
    };
    let (rows, total) = ho_so::list(&st.pool, &f).await?;
    Ok(Json(json!({"rows": rows, "total": total})))
}

async fn get_ho_so(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    let mut row = ho_so::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    let project = row.get("project_id").and_then(Value::as_str).and_then(|s| Uuid::parse_str(s).ok()).ok_or_else(ApiError::not_found)?;
    read_access(&st, &ts, project).await?;
    let vb = ho_so::van_ban_of(&st.pool, id, false).await?;
    let loan = ho_so::open_loan(&st.pool, id).await?;
    if let Some(o) = row.as_object_mut() {
        o.insert("van_ban".into(), Value::Array(vb));
        o.insert("phieu_muon".into(), loan.unwrap_or(Value::Null));
    }
    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VbOfQ {
    #[serde(default)]
    include_deleted: bool,
}

async fn van_ban_of(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Query(q): Query<VbOfQ>) -> Result<Json<Vec<Value>>, ApiError> {
    let row = ho_so::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    let project = row.get("project_id").and_then(Value::as_str).and_then(|s| Uuid::parse_str(s).ok()).ok_or_else(ApiError::not_found)?;
    read_access(&st, &ts, project).await?;
    Ok(Json(ho_so::van_ban_of(&st.pool, id, q.include_deleted).await?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VbQ {
    project_id: Uuid,
    #[serde(default)]
    phong_id: Option<Uuid>,
    #[serde(default)]
    phong_ban_id: Option<Uuid>,
    #[serde(default)]
    ho_so_id: Option<Uuid>,
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    loai_vb: Option<String>,
    #[serde(default)]
    co_quan: Option<String>,
    #[serde(default)]
    tu_ngay: Option<NaiveDate>,
    #[serde(default)]
    den_ngay: Option<NaiveDate>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

async fn search_van_ban(State(st): State<AppState>, ts: Ts, Query(q): Query<VbQ>) -> Result<Json<Value>, ApiError> {
    read_access(&st, &ts, q.project_id).await?;
    let f = ho_so::VanBanFilter {
        project_id: q.project_id,
        phong_id: q.phong_id,
        phong_ban_id: q.phong_ban_id,
        ho_so_id: q.ho_so_id,
        q: q.q,
        loai_vb: q.loai_vb,
        co_quan: q.co_quan,
        tu_ngay: q.tu_ngay,
        den_ngay: q.den_ngay,
        limit: q.limit,
        offset: q.offset,
    };
    let (rows, total) = ho_so::search_van_ban(&st.pool, &f).await?;
    Ok(Json(json!({"rows": rows, "total": total})))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LichSuQ {
    project_id: Uuid,
    #[serde(default)]
    entity: Option<String>,
    #[serde(default)]
    ho_so_id: Option<Uuid>,
    #[serde(default)]
    phong_id: Option<Uuid>,
    #[serde(default)]
    actor_id: Option<Uuid>,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    from: Option<DateTime<Utc>>,
    #[serde(default)]
    to: Option<DateTime<Utc>>,
    #[serde(default)]
    q: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

async fn list_lich_su(State(st): State<AppState>, ts: Ts, Query(q): Query<LichSuQ>) -> Result<Json<Value>, ApiError> {
    read_access(&st, &ts, q.project_id).await?;
    let f = lich_su::Filter {
        project_id: q.project_id,
        entity: q.entity,
        ho_so_id: q.ho_so_id,
        phong_id: q.phong_id,
        actor_id: q.actor_id,
        action: q.action,
        from: q.from,
        to: q.to,
        q: q.q,
        limit: q.limit,
        offset: q.offset,
    };
    let (rows, total) = lich_su::list(&st.pool, &f).await?;
    Ok(Json(json!({"rows": rows, "total": total})))
}
