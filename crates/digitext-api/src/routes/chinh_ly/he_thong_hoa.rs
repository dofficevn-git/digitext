// Copyright (c) 2026 DigiText. All rights reserved.
//! Hệ thống hoá (quyền dossier.systematize, chỉ online): xem trước / đánh số / gán hộp-bó-tập.

use super::{ctx_parts, op_ctx, project_access};
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use digitext_core::chinh_ly::SortOrder;
use digitext_core::Permission;
use digitext_db::chinh_ly::he_thong_hoa as hth;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cl/he-thong-hoa/preview", post(preview))
        .route("/cl/he-thong-hoa/apply", post(apply))
        .route("/cl/he-thong-hoa/boxing", post(boxing))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PreviewReq {
    project_id: Uuid,
    phong_id: Uuid,
    #[serde(default)]
    filter: hth::Filter,
    /// 1..6
    #[serde(default = "default_order")]
    sort_order: i32,
    /// 1 hộp / hồ sơ · 2 bó / tập
    #[serde(default = "default_ft")]
    file_type: i32,
    #[serde(default = "default_start")]
    start_at: i64,
    #[serde(default)]
    snapshot_ver: Option<i64>,
}
fn default_order() -> i32 {
    1
}
fn default_ft() -> i32 {
    1
}
fn default_start() -> i64 {
    1
}

fn order_of(n: i32) -> Result<SortOrder, ApiError> {
    SortOrder::from_i32(n).ok_or_else(|| ApiError::bad_request("Thứ tự sắp xếp phải từ 1 đến 6"))
}

async fn preview(State(st): State<AppState>, ts: Ts, Json(b): Json<PreviewReq>) -> Result<Json<hth::Preview>, ApiError> {
    ts.require_perm(Permission::DossierSystematize)?;
    project_access(&st, &ts, b.project_id).await?;
    let order = order_of(b.sort_order)?;
    Ok(Json(hth::preview(&st.pool, b.project_id, b.phong_id, &b.filter, order, b.file_type, b.start_at).await?))
}

async fn apply(State(st): State<AppState>, ts: Ts, Json(b): Json<PreviewReq>) -> Result<Json<hth::Preview>, ApiError> {
    ts.require_perm(Permission::DossierSystematize)?;
    project_access(&st, &ts, b.project_id).await?;
    let order = order_of(b.sort_order)?;
    let snapshot = b.snapshot_ver.ok_or_else(|| ApiError::bad_request("Thiếu snapshotVer — hãy xem trước rồi áp dụng"))?;
    let parts = ctx_parts(&ts);
    let ctx = op_ctx(&ts, b.project_id, "online", &parts);
    match hth::apply(&st.pool, &ctx, b.phong_id, &b.filter, order, b.file_type, b.start_at, snapshot).await? {
        Ok(p) => {
            digitext_db::audit::append(&st.pool, super::audit(&ts, "cl.systematize", b.phong_id.to_string(), json!({"rows": p.rows.len(), "order": b.sort_order, "fileType": b.file_type}))).await?;
            Ok(Json(p))
        }
        Err(hth::ApplyErr::Stale) => Err(ApiError::conflict("Tập hồ sơ đã thay đổi sau khi xem trước — hãy xem trước lại rồi áp dụng")),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BoxReq {
    project_id: Uuid,
    #[serde(default = "default_ft")]
    file_type: i32,
    rows: Vec<hth::BoxRow>,
}

async fn boxing(State(st): State<AppState>, ts: Ts, Json(b): Json<BoxReq>) -> Result<Json<hth::BoxResult>, ApiError> {
    ts.require_perm(Permission::DossierSystematize)?;
    project_access(&st, &ts, b.project_id).await?;
    if b.rows.len() > 5000 {
        return Err(ApiError::bad_request("Tối đa 5000 dòng mỗi lần"));
    }
    let parts = ctx_parts(&ts);
    let ctx = op_ctx(&ts, b.project_id, "online", &parts);
    let r = hth::boxing(&st.pool, &ctx, b.file_type, &b.rows).await?;
    digitext_db::audit::append(&st.pool, super::audit(&ts, "cl.boxing", b.project_id.to_string(), json!({"applied": r.applied, "hopsCreated": r.hops_created, "errors": r.errors.len()}))).await?;
    Ok(Json(r))
}
