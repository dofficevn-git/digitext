// Copyright (c) 2026 DigiText. All rights reserved.
// Cơ quan nộp lưu vào lưu trữ lịch sử (0040, Digifile D11 / D12): danh mục, tài khoản gắn cơ quan, khoá API.
// Quyền: xem = submission.view; sửa / cấp khoá = submission.review (cán bộ lưu trữ lịch sử); đều trong phạm vi dự án.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Ts};
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use digitext_core::Permission;
use digitext_db::{audit, co_quan_nop};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/co-quan-nop", get(list).post(tao))
        .route("/co-quan-nop/{id}", get(chi_tiet).put(sua).delete(xoa))
        .route("/co-quan-nop/{id}/api-key", post(cap_api_key).delete(thu_hoi_api_key))
        .route("/co-quan-nop/{id}/tai-khoan", get(tai_khoan))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Q {
    pub project_id: Option<Uuid>,
}

/// SHA-256 hex của khoá API — chỉ băm được lưu; khoá thật trả về đúng một lần lúc cấp.
pub fn bam_api_key(key: &str) -> String {
    let mut h = Sha256::new();
    h.update(key.as_bytes());
    format!("{:x}", h.finalize())
}

async fn du_an_cua(st: &AppState, ts: &Ts, project_id: Option<Uuid>) -> Result<Uuid, ApiError> {
    let pid = project_id.or(ts.user.project_id).ok_or_else(|| ApiError::bad_request("thiếu projectId"))?;
    let scope = f_29a(&st.pool, &ts.user).await?;
    if !ts.can_touch_project(&scope, pid) {
        return Err(ApiError::forbidden());
    }
    Ok(pid)
}

async fn nap(st: &AppState, ts: &Ts, id: Uuid) -> Result<co_quan_nop::CoQuanNopRow, ApiError> {
    let row = co_quan_nop::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    du_an_cua(st, ts, Some(row.project_id)).await?;
    Ok(row)
}

async fn list(State(st): State<AppState>, ts: Ts, Query(q): Query<Q>) -> Result<Json<Vec<co_quan_nop::CoQuanNopRow>>, ApiError> {
    ts.require_perm(Permission::SubmissionView)?;
    let pid = du_an_cua(&st, &ts, q.project_id).await?;
    Ok(Json(co_quan_nop::list(&st.pool, pid).await?))
}

async fn chi_tiet(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<co_quan_nop::CoQuanNopRow>, ApiError> {
    ts.require_perm(Permission::SubmissionView)?;
    Ok(Json(nap(&st, &ts, id).await?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaoBody {
    pub project_id: Option<Uuid>,
    #[serde(flatten)]
    pub moi: co_quan_nop::Moi,
}

fn kiem(m: &co_quan_nop::Moi) -> Result<(), ApiError> {
    if m.ma.trim().is_empty() || m.ten.trim().is_empty() {
        return Err(ApiError::bad_request("thiếu mã hoặc tên cơ quan nộp lưu"));
    }
    if let Some(e) = m.email.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        if !e.contains('@') {
            return Err(ApiError::bad_request("email không hợp lệ"));
        }
    }
    Ok(())
}

async fn tao(State(st): State<AppState>, ts: Ts, Json(b): Json<TaoBody>) -> Result<(axum::http::StatusCode, Json<co_quan_nop::CoQuanNopRow>), ApiError> {
    ts.require_perm(Permission::SubmissionReview)?;
    let pid = du_an_cua(&st, &ts, b.project_id).await?;
    kiem(&b.moi)?;
    let row = co_quan_nop::tao(&st.pool, pid, &b.moi, ts.user.id).await.map_err(|e| {
        if digitext_db::f_1jn(&e) { ApiError::conflict("mã cơ quan nộp lưu đã có trong kho này") } else { e.into() }
    })?;
    ghi(&st, &ts, "co_quan_nop.tao", row.id, serde_json::json!({ "ma": row.ma, "ten": row.ten })).await?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

async fn sua(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(m): Json<co_quan_nop::Moi>) -> Result<Json<co_quan_nop::CoQuanNopRow>, ApiError> {
    ts.require_perm(Permission::SubmissionReview)?;
    nap(&st, &ts, id).await?;
    kiem(&m)?;
    let row = co_quan_nop::sua(&st.pool, id, &m)
        .await
        .map_err(|e| if digitext_db::f_1jn(&e) { ApiError::conflict("mã cơ quan nộp lưu đã có trong kho này") } else { e.into() })?
        .ok_or_else(ApiError::not_found)?;
    ghi(&st, &ts, "co_quan_nop.sua", id, serde_json::json!({ "ma": row.ma })).await?;
    Ok(Json(row))
}

async fn xoa(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<axum::http::StatusCode, ApiError> {
    ts.require_perm(Permission::SubmissionReview)?;
    let row = nap(&st, &ts, id).await?;
    if row.so_dot_nop > 0 {
        return Err(ApiError::conflict("cơ quan đã có đợt nộp — không xoá, chỉ bỏ tài khoản / thu hồi khoá API"));
    }
    co_quan_nop::xoa(&st.pool, id).await?;
    ghi(&st, &ts, "co_quan_nop.xoa", id, serde_json::json!({ "ma": row.ma })).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// Cấp (hoặc cấp lại) khoá API — trả khoá thật đúng một lần. Cơ quan cần có ít nhất một tài khoản đang hoạt động để khoá có danh tính.
async fn cap_api_key(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, ApiError> {
    ts.require_perm(Permission::SubmissionReview)?;
    let row = nap(&st, &ts, id).await?;
    if row.so_tai_khoan == 0 {
        return Err(ApiError::bad_request("gắn ít nhất một tài khoản đang hoạt động cho cơ quan trước khi cấp khoá API (khoá mang danh tính tài khoản đó)"));
    }
    use rand::RngCore;
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    use base64::Engine as _;
    let key = format!("dtk_{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b));
    co_quan_nop::dat_api_key(&st.pool, id, Some(&bam_api_key(&key))).await?;
    ghi(&st, &ts, "co_quan_nop.api_key", id, serde_json::json!({ "ma": row.ma, "capLai": row.co_api_key })).await?;
    Ok(Json(serde_json::json!({ "id": id, "apiKey": key, "huongDan": "Gửi kèm header X-Api-Key cho các đường /api/v1/submissions*. Khoá chỉ hiện một lần." })))
}

async fn thu_hoi_api_key(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<axum::http::StatusCode, ApiError> {
    ts.require_perm(Permission::SubmissionReview)?;
    let row = nap(&st, &ts, id).await?;
    co_quan_nop::dat_api_key(&st.pool, id, None).await?;
    ghi(&st, &ts, "co_quan_nop.thu_hoi_api_key", id, serde_json::json!({ "ma": row.ma })).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn tai_khoan(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Vec<co_quan_nop::TaiKhoan>>, ApiError> {
    ts.require_perm(Permission::SubmissionView)?;
    nap(&st, &ts, id).await?;
    Ok(Json(co_quan_nop::tai_khoan(&st.pool, id).await?))
}

async fn ghi(st: &AppState, ts: &Ts, action: &str, id: Uuid, detail: serde_json::Value) -> Result<(), ApiError> {
    audit::append(
        &st.pool,
        audit::Tp { actor_id: Some(ts.user.id), device_id: ts.claims.device_id, action, target: Some(format!("co_quan_nop:{id}")), detail: Some(detail), ip: ts.ip() },
    )
    .await?;
    Ok(())
}
