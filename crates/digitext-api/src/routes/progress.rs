// Copyright (c) 2026 DigiText. All rights reserved.
//! Báo cáo tiến độ dự án (đề nghị chỉnh sửa mục 10).
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::extract::{Path, Query, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use digitext_core::Permission;
use digitext_db::progress::{self, Contract};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stats/project-progress", get(project_progress))
        .route("/projects/{id}/contract", put(put_contract))
        .route("/stats/staff", get(staff))
        .route("/stats/staff/targets", put(put_target))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProgressQuery {
    /// Từ ngày (YYYY-MM-DD, giờ Việt Nam, tính cả ngày này).
    #[serde(default)]
    from: Option<NaiveDate>,
    /// Đến ngày (YYYY-MM-DD, tính cả ngày này).
    #[serde(default)]
    to: Option<NaiveDate>,
    #[serde(default)]
    project_id: Option<Uuid>,
}

/// Ngày (giờ Việt Nam, UTC+7) → mốc UTC; `to` loại trừ nên cộng 1 ngày.
fn vn_range(from: Option<NaiveDate>, to: Option<NaiveDate>) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
    let at = |d: NaiveDate| {
        d.and_hms_opt(0, 0, 0)
            .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t - chrono::Duration::hours(7), Utc))
    };
    (from.and_then(at), to.and_then(|d| d.succ_opt()).and_then(at))
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ContractDto {
    signed_on: Option<NaiveDate>,
    end_on: Option<NaiveDate>,
    deadline_on: Option<NaiveDate>,
    contract_pages: Option<i64>,
    contract_files: Option<i64>,
    #[serde(default)]
    contract_boxes: Option<i64>,
    #[serde(default)]
    contract_dossiers: Option<i64>,
    #[serde(default)]
    contract_sheets: Option<i64>,
    manager_name: Option<String>,
    note: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressDto {
    project_id: Uuid,
    code: String,
    name: String,
    contract: ContractDto,
    files: i64,
    pages: i64,
    pdfa_files: i64,
    signed_files: i64,
    ocr_files: i64,
    approved_files: i64,
    rejected_files: i64,
}

async fn project_progress(
    State(st): State<AppState>,
    ts: Ts,
    Query(q): Query<ProgressQuery>,
) -> Result<Json<Vec<ProgressDto>>, ApiError> {
    ts.require_perm(Permission::StatsViewProject)?;
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
    let (from, to) = vn_range(q.from, q.to);
    let rows = progress::project_progress(&st.pool, projects.as_deref(), from, to).await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| ProgressDto {
                project_id: r.project_id,
                code: r.code,
                name: r.name,
                contract: ContractDto {
                    signed_on: r.signed_on,
                    end_on: r.end_on,
                    deadline_on: r.deadline_on,
                    contract_pages: r.contract_pages,
                    contract_files: r.contract_files,
                    contract_boxes: r.contract_boxes,
                    contract_dossiers: r.contract_dossiers,
                    contract_sheets: r.contract_sheets,
                    manager_name: r.manager_name,
                    note: r.note,
                },
                files: r.files,
                pages: r.pages,
                pdfa_files: r.pdfa_files,
                signed_files: r.signed_files,
                ocr_files: r.ocr_files,
                approved_files: r.approved_files,
                rejected_files: r.rejected_files,
            })
            .collect(),
    ))
}

async fn put_contract(
    State(st): State<AppState>,
    ts: Ts,
    Path(id): Path<Uuid>,
    Json(c): Json<ContractDto>,
) -> Result<Json<ContractDto>, ApiError> {
    ts.require_perm(Permission::ProjectUpdate)?;
    if !f_29a(&st.pool, &ts.user).await?.allows_project(id) {
        return Err(ApiError::forbidden());
    }
    if let (Some(a), Some(b)) = (c.signed_on, c.end_on) {
        if b < a {
            return Err(ApiError::bad_request("ngày kết thúc hợp đồng phải sau ngày ký"));
        }
    }
    if [c.contract_pages, c.contract_files, c.contract_boxes, c.contract_dossiers, c.contract_sheets].iter().any(|v| v.is_some_and(|n| n < 0)) {
        return Err(ApiError::bad_request("khối lượng hợp đồng không được âm"));
    }
    let clean = |s: Option<String>| s.map(|x| x.trim().to_string()).filter(|x| !x.is_empty());
    let c = ContractDto { manager_name: clean(c.manager_name), note: clean(c.note), ..c };
    progress::upsert_contract(
        &st.pool,
        id,
        &Contract {
            signed_on: c.signed_on,
            end_on: c.end_on,
            deadline_on: c.deadline_on,
            contract_pages: c.contract_pages,
            contract_files: c.contract_files,
            contract_boxes: c.contract_boxes,
            contract_dossiers: c.contract_dossiers,
            contract_sheets: c.contract_sheets,
            manager_name: c.manager_name.clone(),
            note: c.note.clone(),
        },
        ts.user.id,
    )
    .await?;
    digitext_db::audit::append(
        &st.pool,
        digitext_db::audit::Tp {
            actor_id: Some(ts.user.id),
            device_id: ts.claims.device_id,
            action: "project.contract_update",
            target: Some(format!("project:{id}")),
            detail: serde_json::to_value(&c).ok(),
            ip: ts.ip(),
        },
    )
    .await?;
    Ok(Json(c))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StaffDto {
    user_id: Uuid,
    username: String,
    display_name: String,
    task: String,
    done: i64,
    files: i64,
    fields: i64,
    days: i64,
    errors: i64,
    target: Option<i64>,
}

/// Báo cáo nhân viên (đề nghị chỉnh sửa mục 11): sản lượng từng người × công việc.
async fn staff(
    State(st): State<AppState>,
    ts: Ts,
    Query(q): Query<ProgressQuery>,
) -> Result<Json<Vec<StaffDto>>, ApiError> {
    ts.require_perm(Permission::StatsViewProject)?;
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
    let (from, to) = vn_range(q.from, q.to);
    let rows = progress::staff_report(&st.pool, projects.as_deref(), from, to).await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| StaffDto {
                user_id: r.user_id,
                username: r.username,
                display_name: r.display_name,
                task: r.task,
                done: r.done,
                files: r.files,
                fields: r.fields,
                days: r.days,
                errors: r.errors,
                target: r.target,
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetIn {
    project_id: Uuid,
    user_id: Uuid,
    task: String,
    /// None / null = xoá sản lượng giao.
    #[serde(default)]
    quantity: Option<i64>,
}

async fn put_target(
    State(st): State<AppState>,
    ts: Ts,
    Json(t): Json<TargetIn>,
) -> Result<Json<serde_json::Value>, ApiError> {
    ts.require_perm(Permission::ProjectUpdate)?;
    if !f_29a(&st.pool, &ts.user).await?.allows_project(t.project_id) {
        return Err(ApiError::forbidden());
    }
    if !progress::TASKS.contains(&t.task.as_str()) {
        return Err(ApiError::bad_request("công việc không hợp lệ"));
    }
    if t.quantity.is_some_and(|q| q < 0) {
        return Err(ApiError::bad_request("sản lượng giao không được âm"));
    }
    progress::set_target(&st.pool, t.project_id, t.user_id, &t.task, t.quantity, ts.user.id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
