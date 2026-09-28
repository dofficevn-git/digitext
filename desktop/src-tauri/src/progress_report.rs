// Copyright (c) 2026 DigiText. All rights reserved.
//! Báo cáo tiến độ dự án + báo cáo nhân viên lấy số từ MÁY CHỦ (đề nghị chỉnh sửa mục 10, 11) —
//! khác các báo cáo cũ chỉ đọc SQLite của riêng máy này.
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeArgs {
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

fn query(a: &RangeArgs) -> String {
    let mut q: Vec<String> = Vec::new();
    if let Some(p) = a.project_id.as_deref().map(str::trim).filter(|p| {
        !p.is_empty() && p.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
    }) {
        q.push(format!("projectId={p}"));
    }
    for (k, v) in [("from", &a.from), ("to", &a.to)] {
        if let Some(s) = v.as_deref().map(str::trim).filter(|s| is_date(s)) {
            q.push(format!("{k}={s}"));
        }
    }
    if q.is_empty() {
        String::new()
    } else {
        format!("?{}", q.join("&"))
    }
}

#[tauri::command]
pub async fn report_project_progress(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Option<RangeArgs>,
) -> Result<serde_json::Value, AppError> {
    let path = format!("/stats/project-progress{}", query(&args.unwrap_or_default()));
    f_2r8(&db, &auth, Permission::StatsViewProject, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<serde_json::Value>(&base, &token, &path).await }
    })
    .await
}

#[tauri::command]
pub async fn report_set_contract(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    project_id: String,
    contract: serde_json::Value,
) -> Result<serde_json::Value, AppError> {
    if !project_id.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        return Err(AppError::validation("mã dự án không hợp lệ"));
    }
    let path = format!("/projects/{project_id}/contract");
    f_2r8(&db, &auth, Permission::ProjectUpdate, move |base, token| {
        let (path, body) = (path.clone(), contract.clone());
        async move { client::f_24x::<serde_json::Value>(&base, &token, &path, &body).await }
    })
    .await
}

#[tauri::command]
pub async fn report_staff(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Option<RangeArgs>,
) -> Result<serde_json::Value, AppError> {
    let path = format!("/stats/staff{}", query(&args.unwrap_or_default()));
    f_2r8(&db, &auth, Permission::StatsViewProject, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<serde_json::Value>(&base, &token, &path).await }
    })
    .await
}

#[tauri::command]
pub async fn report_set_target(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    target: serde_json::Value,
) -> Result<serde_json::Value, AppError> {
    f_2r8(&db, &auth, Permission::ProjectUpdate, move |base, token| {
        let body = target.clone();
        async move { client::f_24x::<serde_json::Value>(&base, &token, "/stats/staff/targets", &body).await }
    })
    .await
}
