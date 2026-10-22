// Copyright (c) 2026 DigiText. All rights reserved.
//! Dashboard 3 góc nhìn — lệnh desktop gọi thẳng API /dashboard/* (22/10/2026). Máy chủ kiểm quyền và phạm vi;
//! ở đây chỉ kiểm hình thức tham số và chọn quyền tối thiểu để không gọi khi chắc chắn bị từ chối.
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use serde::Deserialize;
use tauri::State;

type J = serde_json::Value;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashArgs {
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub days: Option<i64>,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}

fn is_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 40 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}
fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10 && b[4] == b'-' && b[7] == b'-' && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

fn query(a: &DashArgs, require_project: bool) -> Result<String, AppError> {
    let mut q: Vec<String> = Vec::new();
    match a.project_id.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
        Some(p) if is_id(p) => q.push(format!("projectId={p}")),
        Some(_) => return Err(AppError::validation("mã dự án không hợp lệ")),
        None if require_project => return Err(AppError::validation("thiếu mã dự án")),
        None => {}
    }
    if let Some(d) = a.days {
        q.push(format!("days={}", d.clamp(1, 365)));
    }
    for (k, v) in [("from", &a.from), ("to", &a.to)] {
        if let Some(s) = v.as_deref().map(str::trim).filter(|s| is_date(s)) {
            q.push(format!("{k}={s}"));
        }
    }
    Ok(if q.is_empty() { String::new() } else { format!("?{}", q.join("&")) })
}

async fn get(db: &State<'_, Db>, auth: &State<'_, AuthState>, perm: Permission, path: String) -> Result<J, AppError> {
    f_2r8(db, auth, perm, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<J>(&base, &token, &path).await }
    })
    .await
}

#[tauri::command]
pub async fn dashboard_overview(db: State<'_, Db>, auth: State<'_, AuthState>, args: Option<DashArgs>) -> Result<J, AppError> {
    let path = format!("/dashboard/overview{}", query(&args.unwrap_or_default(), false)?);
    get(&db, &auth, Permission::StatsViewProject, path).await
}

#[tauri::command]
pub async fn dashboard_project(db: State<'_, Db>, auth: State<'_, AuthState>, args: DashArgs) -> Result<J, AppError> {
    let path = format!("/dashboard/project{}", query(&args, true)?);
    get(&db, &auth, Permission::StatsViewProject, path).await
}

#[tauri::command]
pub async fn dashboard_staff(db: State<'_, Db>, auth: State<'_, AuthState>, args: DashArgs) -> Result<J, AppError> {
    let path = format!("/dashboard/staff{}", query(&args, true)?);
    get(&db, &auth, Permission::StatsViewProject, path).await
}

#[tauri::command]
pub async fn dashboard_me(db: State<'_, Db>, auth: State<'_, AuthState>, args: Option<DashArgs>) -> Result<J, AppError> {
    let path = format!("/dashboard/me{}", query(&args.unwrap_or_default(), false)?);
    get(&db, &auth, Permission::StatsView, path).await
}

#[tauri::command]
pub async fn dashboard_kpi_settings_get(db: State<'_, Db>, auth: State<'_, AuthState>, project_id: String) -> Result<J, AppError> {
    let p = project_id.trim().to_owned();
    if !is_id(&p) {
        return Err(AppError::validation("mã dự án không hợp lệ"));
    }
    get(&db, &auth, Permission::StatsViewProject, format!("/dashboard/kpi-settings?projectId={p}")).await
}

#[tauri::command]
pub async fn dashboard_kpi_settings_set(db: State<'_, Db>, auth: State<'_, AuthState>, project_id: String, body: J) -> Result<J, AppError> {
    let p = project_id.trim().to_owned();
    if !is_id(&p) {
        return Err(AppError::validation("mã dự án không hợp lệ"));
    }
    let path = format!("/dashboard/kpi-settings?projectId={p}");
    f_2r8(&db, &auth, Permission::ProjectUpdate, move |base, token| {
        let (path, body) = (path.clone(), body.clone());
        async move { client::f_24x::<J>(&base, &token, &path, &body).await }
    })
    .await
}
