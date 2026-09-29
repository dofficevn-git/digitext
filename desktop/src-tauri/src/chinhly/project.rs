// Copyright (c) 2026 DigiText. All rights reserved.
//! "Dự án đang làm" của chỉnh lý. Tài khoản thường gắn sẵn một dự án (`session.project_id`); tài khoản quản trị
//! (super_admin) không gắn dự án nào nên phải chọn dự án ở màn Chỉnh lý — lựa chọn lưu trong `settings`
//! (`cl_project_id`) và dùng cho mọi lệnh `cl_*`, cầu nối bảng cũ và engine đồng bộ.

use crate::auth::admin::f_2r7;
use crate::auth::{client, AuthState};
use crate::db::{self, Db};
use crate::error::AppError;
use digitext_core::Perm;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

pub const KEY: &str = "cl_project_id";
const CACHE: &str = "cl_projects_cache";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItem {
    pub id: String,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    /// Dự án hiệu lực (None = chưa chọn).
    pub project_id: Option<String>,
    /// true = lấy từ tài khoản (không đổi được); false = tài khoản không gắn dự án, chọn tay.
    pub from_account: bool,
    pub projects: Vec<ProjectItem>,
    /// Danh sách lấy từ bộ nhớ đệm (không tới được máy chủ).
    pub cached: bool,
}

fn chosen(conn: &Connection) -> Option<String> {
    db::f_1ew(conn, KEY).ok().flatten().map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
}

/// Dự án hiệu lực từ phiên + lựa chọn đã lưu (cần kết nối SQLite đã khoá sẵn).
pub fn effective_with(conn: &Connection, auth: &AuthState) -> Option<(String, String)> {
    let s = auth.snapshot()?;
    let p = s.project_id.clone().filter(|p| !p.trim().is_empty()).or_else(|| chosen(conn))?;
    Some((p, s.user_id))
}

/// Dự án hiệu lực (khoá SQLite bên trong). Lỗi `no_project` khi chưa có.
pub fn resolve(db: &Db, auth: &AuthState) -> Result<(String, String), AppError> {
    let s = auth.snapshot().ok_or_else(|| AppError::new("unauthorized", "Chưa đăng nhập."))?;
    if let Some(p) = s.project_id.clone().filter(|p| !p.trim().is_empty()) {
        return Ok((p, s.user_id));
    }
    let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    match chosen(&g) {
        Some(p) => Ok((p, s.user_id)),
        None => Err(AppError::new("no_project", "Tài khoản không gắn dự án — chọn \"Dự án đang làm\" ở đầu màn Chỉnh lý.")),
    }
}

pub fn resolve_opt(db: &Db, auth: &AuthState) -> Option<(String, String)> {
    resolve(db, auth).ok()
}

/// Danh sách dự án cho tài khoản quản trị (`GET /projects`), lưu đệm để dùng khi ngoại tuyến.
pub async fn info(db: &State<'_, Db>, auth: &State<'_, AuthState>) -> Result<ProjectInfo, AppError> {
    let s = auth.snapshot().ok_or_else(|| AppError::new("unauthorized", "Chưa đăng nhập."))?;
    if let Some(p) = s.project_id.clone().filter(|p| !p.trim().is_empty()) {
        return Ok(ProjectInfo { project_id: Some(p), from_account: true, projects: Vec::new(), cached: false });
    }
    let fetched = f_2r7(db, auth, Perm::Admin, |base, token| async move { client::f_1eo::<Vec<client::Tl4>>(&base, &token, "/projects").await }).await;
    let (projects, cached) = match fetched {
        Ok(list) => {
            let items: Vec<ProjectItem> = list.into_iter().filter(|p| p.is_active).map(|p| ProjectItem { id: p.id, code: p.code, name: p.name }).collect();
            if let Ok(g) = db.0.lock() {
                let _ = db::f_2ei(&g, CACHE, Some(&serde_json::to_string(&items).unwrap_or_else(|_| "[]".into())));
            }
            (items, false)
        }
        Err(_) => {
            let items = db.0.lock().ok().and_then(|g| db::f_1ew(&g, CACHE).ok().flatten()).and_then(|s| serde_json::from_str::<Vec<ProjectItem>>(&s).ok()).unwrap_or_default();
            (items, true)
        }
    };
    let project_id = db.0.lock().ok().and_then(|g| chosen(&g));
    Ok(ProjectInfo { project_id, from_account: false, projects, cached })
}

pub fn set(db: &Db, auth: &AuthState, project_id: &str) -> Result<(), AppError> {
    let s = auth.snapshot().ok_or_else(|| AppError::new("unauthorized", "Chưa đăng nhập."))?;
    if s.project_id.as_deref().map_or(false, |p| !p.trim().is_empty()) {
        return Err(AppError::validation("Tài khoản đã gắn dự án, không đổi được."));
    }
    let p = project_id.trim();
    if p.is_empty() {
        return Err(AppError::validation("Chưa chọn dự án."));
    }
    let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    db::f_2ei(&g, KEY, Some(p))?;
    Ok(())
}
