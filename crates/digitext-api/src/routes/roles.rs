// Copyright (c) 2026 DigiText. All rights reserved.
//! Vai trò và bộ quyền (migration 0022): xem, tạo vai trò tuỳ chỉnh, sửa bộ quyền, khôi phục mặc định, xoá.
//! Vai trò gốc (`base_role`) quyết định tầng và phạm vi; bộ quyền không được vượt tầng của vai trò gốc
//! (digitext_core::allowed_for_base). Chỉ người phạm vi toàn hệ thống (Quản lý / Quản trị hệ thống) được sửa vai trò
//! vì nó ảnh hưởng mọi dự án.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::{allowed_for_base, is_role_code, Permission, Role};
use digitext_db::roles;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/roles", get(list).post(create))
        .route("/roles/{code}", get(get_one).put(update).delete(remove))
        .route("/roles/{code}/reset", post(reset))
}

// ---------------------------------------------------------------------------------------------------------------
// Cache bộ quyền theo mã vai trò (mọi request đều cần; vai trò hiếm đổi). Xoá khi có ghi.
// ---------------------------------------------------------------------------------------------------------------
type PresetCache = HashMap<String, (std::time::Instant, Option<BTreeSet<Permission>>)>;
static PRESETS: std::sync::OnceLock<std::sync::Mutex<PresetCache>> = std::sync::OnceLock::new();
const PRESET_TTL: std::time::Duration = std::time::Duration::from_secs(20);

pub fn cache_clear() {
    if let Ok(mut c) = PRESETS.get_or_init(Default::default).lock() {
        c.clear();
    }
}

/// Bộ quyền của vai trò mà người dùng mang (`role_code`, không có thì vai trò gốc). None → bảng roles chưa có dòng
/// (chưa áp 0022 / chưa chèn) → người gọi dùng mặc định trong code.
pub async fn preset_cached(pool: &sqlx::PgPool, user: &digitext_db::users::User) -> sqlx::Result<Option<BTreeSet<Permission>>> {
    let code = user.role_code.clone().unwrap_or_else(|| user.role.as_str().to_owned());
    if let Ok(c) = PRESETS.get_or_init(Default::default).lock() {
        if let Some((at, v)) = c.get(&code) {
            if at.elapsed() < PRESET_TTL {
                return Ok(v.clone());
            }
        }
    }
    let v = match roles::permissions_of(pool, &code).await {
        Ok(v) => v,
        // Bảng roles chưa có (migration chưa áp): coi như không có bộ quyền riêng, không làm hỏng đăng nhập.
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("42P01") => None,
        Err(e) => return Err(e),
    };
    if let Ok(mut c) = PRESETS.get_or_init(Default::default).lock() {
        c.insert(code, (std::time::Instant::now(), v.clone()));
    }
    Ok(v)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleDto {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub base_role: Role,
    pub base_role_name: &'static str,
    pub is_system: bool,
    pub sort_order: i32,
    pub permissions: Vec<Permission>,
    /// Quyền được phép có với vai trò gốc này (trần theo tầng).
    pub allowed: Vec<Permission>,
    pub user_count: i64,
    pub updated_at: DateTime<Utc>,
}
impl From<roles::RoleFull> for RoleDto {
    fn from(r: roles::RoleFull) -> Self {
        let base = r.row.base_role;
        RoleDto {
            code: r.row.code,
            name: r.row.name,
            description: r.row.description,
            base_role: base,
            base_role_name: base.display_name(),
            is_system: r.row.is_system,
            sort_order: r.row.sort_order,
            permissions: r.permissions.into_iter().collect(),
            allowed: Permission::ALL.into_iter().filter(|p| allowed_for_base(base, *p)).collect(),
            user_count: r.user_count,
            updated_at: r.row.updated_at,
        }
    }
}

async fn full(pool: &sqlx::PgPool, code: &str) -> Result<RoleDto, ApiError> {
    roles::list(pool).await?.into_iter().find(|r| r.row.code == code).map(RoleDto::from).ok_or_else(ApiError::not_found)
}

/// Ai được sửa vai trò: user.grant_permissions + phạm vi toàn hệ thống. Quản lý không đụng vai trò gốc
/// super_admin / manager (cùng luật với tài khoản).
async fn require_editor(st: &AppState, ts: &Ts, base: Role) -> Result<(), ApiError> {
    ts.require_perm(Permission::UserGrantPermissions)?;
    if !matches!(f_29a(&st.pool, &ts.user).await?, Scope::All) {
        return Err(ApiError::forbidden());
    }
    if ts.user.role == Role::Manager && matches!(base, Role::SuperAdmin | Role::Manager) {
        return Err(ApiError::forbidden());
    }
    Ok(())
}

fn check_perms(base: Role, perms: &[Permission]) -> Result<BTreeSet<Permission>, ApiError> {
    let set: BTreeSet<Permission> = perms.iter().copied().collect();
    if let Some(p) = set.iter().find(|p| !allowed_for_base(base, **p)) {
        return Err(ApiError::bad_request(format!("quyền «{p}» vượt tầng của vai trò gốc {}", base.display_name())));
    }
    Ok(set)
}

async fn list(State(st): State<AppState>, ts: Ts) -> Result<Json<Vec<RoleDto>>, ApiError> {
    ts.require_perm(Permission::UserView)?;
    Ok(Json(roles::list(&st.pool).await?.into_iter().map(RoleDto::from).collect()))
}

async fn get_one(State(st): State<AppState>, ts: Ts, Path(code): Path<String>) -> Result<Json<RoleDto>, ApiError> {
    ts.require_perm(Permission::UserView)?;
    Ok(Json(full(&st.pool, &code).await?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateReq {
    code: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    base_role: Role,
    #[serde(default)]
    permissions: Vec<Permission>,
}

async fn create(State(st): State<AppState>, ts: Ts, Json(b): Json<CreateReq>) -> Result<(axum::http::StatusCode, Json<RoleDto>), ApiError> {
    require_editor(&st, &ts, b.base_role).await?;
    let code = b.code.trim().to_owned();
    if !is_role_code(&code) {
        return Err(ApiError::bad_request("mã vai trò: chữ thường, số, gạch dưới, 2–40 ký tự, bắt đầu bằng chữ"));
    }
    if code.parse::<Role>().is_ok() {
        return Err(ApiError::bad_request("mã này là vai trò hệ thống"));
    }
    let name = b.name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(ApiError::bad_request("tên vai trò 1–80 ký tự"));
    }
    if b.base_role == Role::Customer {
        return Err(ApiError::bad_request("không tạo vai trò mới trên nền Khách hàng"));
    }
    let perms = check_perms(b.base_role, &b.permissions)?;
    let row = roles::create(&st.pool, &code, name, b.description.as_deref().map(str::trim).filter(|d| !d.is_empty()), b.base_role, &perms, Some(ts.user.id))
        .await
        .map_err(|e| if digitext_db::f_1jn(&e) { ApiError::conflict("mã vai trò đã tồn tại") } else { e.into() })?;
    cache_clear();
    audit(&st, &ts, "role.create", &row.code, serde_json::json!({ "name": row.name, "baseRole": row.base_role, "permissions": perms.len() })).await?;
    Ok((axum::http::StatusCode::CREATED, Json(full(&st.pool, &row.code).await?)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateReq {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    permissions: Vec<Permission>,
}

async fn update(State(st): State<AppState>, ts: Ts, Path(code): Path<String>, Json(b): Json<UpdateReq>) -> Result<Json<RoleDto>, ApiError> {
    let row = roles::get(&st.pool, &code).await?.ok_or_else(ApiError::not_found)?;
    require_editor(&st, &ts, row.base_role).await?;
    if row.code == Role::SuperAdmin.as_str() {
        return Err(ApiError::bad_request("Quản trị hệ thống luôn có toàn quyền — không sửa được"));
    }
    let name = b.name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(ApiError::bad_request("tên vai trò 1–80 ký tự"));
    }
    let perms = check_perms(row.base_role, &b.permissions)?;
    if !roles::update(&st.pool, &row.code, name, b.description.as_deref().map(str::trim).filter(|d| !d.is_empty()), &perms).await? {
        return Err(ApiError::not_found());
    }
    cache_clear();
    audit(&st, &ts, "role.update", &row.code, serde_json::json!({ "name": name, "permissions": perms.iter().map(|p| p.as_str()).collect::<Vec<_>>() })).await?;
    let _ = digitext_db::changes::note(&st.pool, "users").await;
    Ok(Json(full(&st.pool, &row.code).await?))
}

async fn reset(State(st): State<AppState>, ts: Ts, Path(code): Path<String>) -> Result<Json<RoleDto>, ApiError> {
    let row = roles::get(&st.pool, &code).await?.ok_or_else(ApiError::not_found)?;
    require_editor(&st, &ts, row.base_role).await?;
    if !row.is_system {
        return Err(ApiError::bad_request("chỉ vai trò hệ thống mới có bộ quyền mặc định để khôi phục"));
    }
    roles::reset_system(&st.pool, &row.code).await?;
    cache_clear();
    audit(&st, &ts, "role.reset", &row.code, serde_json::json!({})).await?;
    Ok(Json(full(&st.pool, &row.code).await?))
}

async fn remove(State(st): State<AppState>, ts: Ts, Path(code): Path<String>) -> Result<Json<serde_json::Value>, ApiError> {
    let row = roles::get(&st.pool, &code).await?.ok_or_else(ApiError::not_found)?;
    require_editor(&st, &ts, row.base_role).await?;
    if row.is_system {
        return Err(ApiError::bad_request("không xoá vai trò hệ thống"));
    }
    let n = roles::user_count(&st.pool, &row.code).await?;
    if n > 0 {
        return Err(ApiError::conflict(format!("còn {n} tài khoản đang dùng vai trò này — đổi vai trò cho họ trước")));
    }
    if !roles::delete(&st.pool, &row.code).await? {
        return Err(ApiError::not_found());
    }
    cache_clear();
    audit(&st, &ts, "role.delete", &row.code, serde_json::json!({ "name": row.name })).await?;
    Ok(Json(serde_json::json!({ "deleted": true, "code": row.code })))
}

async fn audit(st: &AppState, ts: &Ts, action: &str, code: &str, detail: serde_json::Value) -> Result<(), ApiError> {
    digitext_db::audit::append(
        &st.pool,
        digitext_db::audit::Tp { actor_id: Some(ts.user.id), device_id: ts.claims.device_id, action, target: Some(format!("role:{code}")), detail: Some(detail), ip: ts.ip() },
    )
    .await?;
    Ok(())
}
