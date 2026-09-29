// Copyright (c) 2026 DigiText. All rights reserved.
//! CHỈNH LÝ tài liệu giấy — HTTP (kế hoạch 29/9/2026). Mọi ghi đi qua `digitext_db::chinh_ly::apply_op`
//! (một luồng cho REST online lẫn push offline). Dòng trả về là JSON snake_case đúng tên cột cl_*.
//!
//! Route (tiền tố /api/v1):
//! - `GET  /cl/catalog?projectId`                       — mọi danh mục của dự án
//! - `GET  /cl/rows/{table}/{id}`                       — một dòng bất kỳ
//! - `POST /cl/rows/{table}` `{projectId, new}`          — thêm (id có thể client sinh)
//! - `PUT  /cl/rows/{table}/{id}` `{projectId, ifVer|base, new}` — sửa (409 kèm dòng máy chủ khi lệch)
//! - `DELETE /cl/rows/{table}/{id}?projectId&ifVer`     — xoá mềm (+ con)
//! - `POST /cl/rows/{table}/{id}/restore?projectId`     — khôi phục
//! - `GET  /cl/ho-so?…`, `GET /cl/ho-so/{id}`, `GET /cl/ho-so/{id}/van-ban`, `GET /cl/van-ban?…`
//! - `POST /cl/ho-so/import`, `POST /cl/ho-so/{id}/van-ban/import` — nhập theo dòng, lỗi từng dòng
//! - `GET  /cl/lich-su?…`
//! - `GET  /cl/sync?projectId&since&limit`, `POST /cl/sync/push` — đồng bộ offline

pub mod he_thong_hoa;
pub mod import;
pub mod read;
pub mod rows;
pub mod sync;
pub mod thong_ke;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Ts};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use digitext_core::{Permission, Role};
use digitext_db::chinh_ly::{OpCtx, OpResult};
use serde_json::json;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().merge(read::router()).merge(rows::router()).merge(import::router()).merge(sync::router()).merge(he_thong_hoa::router()).merge(thong_ke::router())
}

/// Người gọi phải thuộc dự án (admin quản dự án, thành viên, hoặc tài khoản gắn dự án).
pub(crate) async fn project_access(st: &AppState, ts: &Ts, project: Uuid) -> Result<(), ApiError> {
    let scope = f_29a(&st.pool, &ts.user).await?;
    if ts.can_touch_project(&scope, project) || digitext_db::work_packages::is_project_user(&st.pool, project, ts.user.id).await? {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}

/// Đọc: cần dossier.read hoặc catalog.read + thuộc dự án.
pub(crate) async fn read_access(st: &AppState, ts: &Ts, project: Uuid) -> Result<(), ApiError> {
    if !(ts.has_perm(Permission::DossierRead) || ts.has_perm(Permission::CatalogRead)) {
        return Err(ApiError::forbidden());
    }
    project_access(st, ts, project).await
}

pub(crate) struct CtxParts<'a> {
    pub has_perm: Box<dyn Fn(Permission) -> bool + Send + Sync + 'a>,
}

pub(crate) fn op_ctx<'a>(ts: &'a Ts, project: Uuid, source: &'a str, parts: &'a CtxParts<'a>) -> OpCtx<'a> {
    OpCtx {
        project_id: project,
        user_id: ts.user.id,
        user_name: &ts.user.display_name,
        device_id: ts.claims.device_id,
        source,
        has_perm: &*parts.has_perm,
        is_super: matches!(ts.user.role, Role::SuperAdmin),
    }
}

pub(crate) fn ctx_parts(ts: &Ts) -> CtxParts<'_> {
    CtxParts { has_perm: Box::new(move |p| ts.has_perm(p)) }
}

/// Mã HTTP theo kết quả op: applied/merged/no_change 200 · conflict 409 · rejected 400/403/404.
pub(crate) fn op_response(res: OpResult) -> Response {
    let status = match &res {
        OpResult::Applied { .. } | OpResult::Merged { .. } | OpResult::NoChange { .. } | OpResult::Duplicate { .. } => StatusCode::OK,
        OpResult::Conflict { .. } => StatusCode::CONFLICT,
        OpResult::Rejected { reason, .. } => {
            if reason == "forbidden" {
                StatusCode::FORBIDDEN
            } else if reason == "missing" || reason == "deleted_on_server" {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            }
        }
    };
    let mut body = serde_json::to_value(&res).unwrap_or_else(|_| json!({"status": "rejected", "reason": "internal"}));
    if status != StatusCode::OK {
        // Giữ dạng {code, message} quen thuộc của ApiError để desktop hiện lỗi.
        if let Some(o) = body.as_object_mut() {
            let msg = o.get("message").cloned().unwrap_or(json!("Xung đột dữ liệu — tải lại rồi thử lại"));
            let code = o.get("status").cloned().unwrap_or(json!("conflict"));
            o.entry("code").or_insert(code);
            o.insert("message".into(), msg);
        }
    }
    (status, Json(body)).into_response()
}

pub(crate) fn audit(ts: &Ts, action: &'static str, target: String, detail: serde_json::Value) -> digitext_db::audit::Tp<'static> {
    digitext_db::audit::Tp { actor_id: Some(ts.user.id), device_id: ts.claims.device_id, action, target: Some(target), detail: Some(detail), ip: ts.ip() }
}

/// `ho-so` / `ho_so` / `cl_ho_so` → `cl_ho_so`.
pub(crate) fn table_name(s: &str) -> String {
    let t = s.trim().to_lowercase().replace('-', "_");
    if t.starts_with("cl_") {
        t
    } else {
        format!("cl_{t}")
    }
}
