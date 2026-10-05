// Copyright (c) 2026 DigiText. All rights reserved.
//! Quản lý kho ví dụ "dạy bằng ví dụ" cho bóc trường (5/10/2026): xem, lọc, bật / tắt.
//! Người có quyền duyệt tài liệu (document.review) — chính là người sinh ra ví dụ khi duyệt.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::extract::{Path, Query, State};
use axum::routing::{get, patch};
use axum::{Json, Router};
use digitext_core::Permission;
use digitext_db::kie_examples::{self as ke, ListFilter, ListItem};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/kie/examples", get(list))
        .route("/kie/examples/{id}", patch(set_active))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    project_id: Option<Uuid>,
    /// all (mặc định) | project (chỉ ví dụ của dự án) | general (chỉ ví dụ chung, vd plakie)
    #[serde(default)]
    scope: String,
    active: Option<bool>,
    q: Option<String>,
    /// used (mặc định) | recent
    #[serde(default)]
    order: String,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}
fn default_limit() -> i64 {
    50
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListResp {
    total: i64,
    items: Vec<ListItem>,
}

async fn list(State(st): State<AppState>, ts: Ts, Query(q): Query<ListQuery>) -> Result<Json<ListResp>, ApiError> {
    ts.require_perm(Permission::DocumentReview)?;
    let scope = f_29a(&st.pool, &ts.user).await?;
    let projects: Option<Vec<Uuid>> = match (&scope, q.project_id) {
        (s, Some(pid)) => {
            if !s.allows_project(pid) {
                return Err(ApiError::forbidden());
            }
            Some(vec![pid])
        }
        (Scope::All, None) => None,
        (Scope::Projects(p), None) => Some(p.clone()),
    };
    let only_general = q.scope == "general";
    let f = ListFilter {
        projects: projects.as_deref(),
        // Ví dụ chung (không thuộc dự án) dùng cho mọi dự án → ai duyệt cũng thấy, trừ khi lọc
        // "chỉ ví dụ của dự án".
        include_general: q.scope != "project",
        only_general,
        active: q.active,
        q: q.q.as_deref(),
        order: &q.order,
        limit: q.limit.clamp(1, 200),
        offset: q.offset.max(0),
    };
    let (total, items) = ke::list(&st.pool, &f).await?;
    Ok(Json(ListResp { total, items }))
}

#[derive(Debug, Deserialize)]
struct SetActiveReq {
    active: bool,
}

#[derive(Debug, Serialize)]
struct SetActiveResp {
    id: Uuid,
    active: bool,
}

async fn set_active(
    State(st): State<AppState>,
    ts: Ts,
    Path(id): Path<Uuid>,
    Json(r): Json<SetActiveReq>,
) -> Result<Json<SetActiveResp>, ApiError> {
    ts.require_perm(Permission::DocumentReview)?;
    let project = ke::project_of(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    let scope = f_29a(&st.pool, &ts.user).await?;
    // Ví dụ chung ảnh hưởng MỌI dự án → chỉ người có phạm vi toàn hệ thống được tắt / bật.
    let allowed = match project {
        Some(pid) => scope.allows_project(pid),
        None => matches!(scope, Scope::All),
    };
    if !allowed {
        return Err(ApiError::forbidden());
    }
    if !ke::set_active(&st.pool, id, r.active).await? {
        return Err(ApiError::not_found());
    }
    digitext_db::audit::append(
        &st.pool,
        digitext_db::audit::Tp {
            actor_id: Some(ts.user.id),
            device_id: ts.claims.device_id,
            action: if r.active { "kie.example.enable" } else { "kie.example.disable" },
            target: Some(format!("kie_example:{id}")),
            detail: None,
            ip: ts.ip(),
        },
    )
    .await?;
    Ok(Json(SetActiveResp { id, active: r.active }))
}
