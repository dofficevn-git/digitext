// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use crate::password;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::{f_18s, Permission, PermissionOverrides, Role};
use digitext_db::users::{self, User};
use digitext_db::{audit, permissions, tokens};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(f_1oo).post(f_12g))
        .route("/users/{id}", get(f_1f0).patch(f_20i))
        .route("/users/{id}/password", post(f_xv))
        .route("/users/{id}/revoke", post(f_2a3))
        .route("/users/{id}/permissions", get(f_1f1).put(f_24z))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub display_name: String,
    pub role: Role,
    pub project_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
impl From<User> for UserDto {
    fn from(r_w0: User) -> Self {
        UserDto {
            id: r_w0.id,
            username: r_w0.username,
            email: r_w0.email,
            display_name: r_w0.display_name,
            role: r_w0.role,
            project_id: r_w0.project_id,
            is_active: r_w0.is_active,
            created_at: r_w0.created_at,
        }
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3i {
    pub username: String,
    pub password: String,
    pub display_name: String,
    #[serde(default)]
    pub email: Option<String>,
    pub role: Role,
    #[serde(default)]
    pub project_id: Option<Uuid>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tgt {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T2n {
    #[serde(default)]
    pub old_password: Option<String>,
    pub new_password: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tb1 {
    #[serde(default)]
    pub project_id: Option<Uuid>,
    #[serde(default)]
    pub role: Option<Role>,
}
fn f_x0(r_w1: &User, r_w3: &User, r_w2: &Scope) -> bool {
    match r_w1.role {
        Role::SuperAdmin => true,
        // Quản lý không đụng tới tài khoản super_admin / quản lý khác.
        Role::Manager => !matches!(r_w3.role, Role::SuperAdmin | Role::Manager),
        Role::Admin => r_w3.role == Role::User && r_w2.allows_opt(r_w3.project_id),
        _ => false,
    }
}
fn validate_password(r_w4: &str) -> Result<(), ApiError> {
    if r_w4.chars().count() < 8 {
        return Err(ApiError::bad_request(
            "mật khẩu phải tối thiểu 8 ký tự",
        ));
    }
    Ok(())
}
async fn f_1ox(r_w5: &AppState, id: Uuid) -> Result<User, ApiError> {
    users::f_1eg(&r_w5.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)
}
async fn f_1oo(
    State(r_wb): State<AppState>,
    r_w6: Ts,
    Query(r_w9): Query<Tb1>,
) -> Result<Json<Vec<UserDto>>, ApiError> {
    r_w6.require_perm(Permission::UserView)?;
    let r_wa = f_29a(&r_wb.pool, &r_w6.user).await?;
    let r_w8 = match r_wa {
        Scope::All => users::list(&r_wb.pool, r_w9.project_id, r_w9.role).await?,
        Scope::Projects(r_w7) => users::list_in_projects(&r_wb.pool, &r_w7, r_w9.role).await?,
    };
    Ok(Json(r_w8.into_iter().map(UserDto::from).collect()))
}
async fn f_12g(
    State(r_wk): State<AppState>,
    r_wc: Ts,
    Json(r_wi): Json<T3i>,
) -> Result<(StatusCode, Json<UserDto>), ApiError> {
    r_wc.require_perm(Permission::UserCreate)?;
    validate_password(&r_wi.password)?;
    let r_wj = f_29a(&r_wk.pool, &r_wc.user).await?;
    if r_wc.user.role == Role::Manager && matches!(r_wi.role, Role::SuperAdmin | Role::Manager) {
        return Err(ApiError::forbidden());
    }
    // Quản trị dự án / nhân viên dự án luôn gắn đúng MỘT dự án (15/10/2026).
    if matches!(r_wi.role, Role::Admin | Role::User) && r_wi.project_id.is_none() {
        return Err(ApiError::bad_request("chọn dự án cho quản trị dự án / nhân viên dự án"));
    }
    let project_id = match r_wj {
        Scope::All => r_wi.project_id,
        Scope::Projects(r_wf) => {
            if r_wi.role != Role::User {
                return Err(ApiError::forbidden());
            }
            match r_wi.project_id {
                Some(r_wg) if r_wf.contains(&r_wg) => Some(r_wg),
                Some(_) => return Err(ApiError::forbidden()),
                None => None,
            }
        }
    };
    let r_wh = password::f_1g0(&r_wi.password).await?;
    let r_wd = users::create(
        &r_wk.pool,
        users::Te1 {
            username: &r_wi.username,
            email: r_wi.email.as_deref(),
            display_name: &r_wi.display_name,
            pwd_hash: &r_wh,
            role: r_wi.role,
            project_id,
            created_by: Some(r_wc.user.id),
        },
    )
    .await
    .map_err(|r_we| {
        if digitext_db::f_1jn(&r_we) {
            ApiError::conflict("username hoặc email đã tồn tại")
        } else {
            r_we.into()
        }
    })?;
    audit::append(
        &r_wk.pool,
        audit::Tp {
            actor_id: Some(r_wc.user.id),
            device_id: r_wc.claims.device_id,
            action: "user.create",
            target: Some(format!("user:{}", r_wd.id)),
            detail: Some(serde_json::json!(
                { "username" : r_wd.username, "role" : r_wd.role, }
            )),
            ip: r_wc.ip(),
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(r_wd.into())))
}
async fn f_1f0(
    State(r_wn): State<AppState>,
    r_wl: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<UserDto>, ApiError> {
    let r_wo = f_1ox(&r_wn, id).await?;
    let r_wm = f_29a(&r_wn.pool, &r_wl.user).await?;
    let r_wp =
        r_wl.user.id == r_wo.id || matches!(r_wm, Scope::All) || r_wm.allows_opt(r_wo.project_id);
    if !r_wp {
        return Err(ApiError::forbidden());
    }
    Ok(Json(r_wo.into()))
}
async fn f_20i(
    State(r_wv): State<AppState>,
    r_wq: Ts,
    Path(id): Path<Uuid>,
    Json(r_wt): Json<Tgt>,
) -> Result<Json<UserDto>, ApiError> {
    let r_ww = f_1ox(&r_wv, id).await?;
    let r_wu = f_29a(&r_wv.pool, &r_wq.user).await?;
    let r_wr = r_wq.user.id == r_ww.id;
    let r_ws = f_x0(&r_wq.user, &r_ww, &r_wu);
    if !r_wr && !r_ws {
        return Err(ApiError::forbidden());
    }
    if !r_wr {
        r_wq.require_perm(Permission::UserUpdate)?;
    }
    if r_wr && r_wt.is_active.is_some() && !r_ws {
        return Err(ApiError::forbidden());
    }
    let r_wx = users::update(
        &r_wv.pool,
        r_ww.id,
        users::Tph {
            display_name: r_wt.display_name.as_deref(),
            email: r_wt.email.as_deref(),
            is_active: r_wt.is_active,
        },
    )
    .await?
    .ok_or_else(ApiError::not_found)?;
    audit::append(
        &r_wv.pool,
        audit::Tp {
            actor_id: Some(r_wq.user.id),
            device_id: r_wq.claims.device_id,
            action: "user.update",
            target: Some(format!("user:{}", r_ww.id)),
            detail: None,
            ip: r_wq.ip(),
        },
    )
    .await?;
    Ok(Json(r_wx.into()))
}
async fn f_xv(
    State(r_x4): State<AppState>,
    r_wy: Ts,
    Path(id): Path<Uuid>,
    Json(r_x2): Json<T2n>,
) -> Result<StatusCode, ApiError> {
    let r_x5 = f_1ox(&r_x4, id).await?;
    validate_password(&r_x2.new_password)?;
    if r_wy.user.id == r_x5.id {
        let r_x0 = r_x2
            .old_password
            .as_deref()
            .ok_or_else(|| ApiError::bad_request("thiếu mật khẩu cũ"))?;
        let (_, r_x1) = users::f_1ej(&r_x4.pool, &r_x5.username)
            .await?
            .ok_or_else(ApiError::not_found)?;
        if !password::f_2pn(&r_x1, r_x0).await {
            return Err(ApiError::auth_failed());
        }
    } else {
        r_wy.require_perm(Permission::UserResetPassword)?;
        let r_x3 = f_29a(&r_x4.pool, &r_wy.user).await?;
        if !f_x0(&r_wy.user, &r_x5, &r_x3) {
            return Err(ApiError::forbidden());
        }
    }
    let r_wz = password::f_1g0(&r_x2.new_password).await?;
    users::f_2e8(&r_x4.pool, r_x5.id, &r_wz)
        .await?
        .ok_or_else(ApiError::not_found)?;
    tokens::f_29z(&r_x4.pool, r_x5.id, "password_changed").await?;
    audit::append(
        &r_x4.pool,
        audit::Tp {
            actor_id: Some(r_wy.user.id),
            device_id: r_wy.claims.device_id,
            action: "user.password",
            target: Some(format!("user:{}", r_x5.id)),
            detail: None,
            ip: r_wy.ip(),
        },
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn f_2a3(
    State(r_x8): State<AppState>,
    r_x6: Ts,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    r_x6.require_perm(Permission::UserRevoke)?;
    let r_x9 = f_1ox(&r_x8, id).await?;
    let r_x7 = f_29a(&r_x8.pool, &r_x6.user).await?;
    if !f_x0(&r_x6.user, &r_x9, &r_x7) {
        return Err(ApiError::forbidden());
    }
    if r_x6.user.id == r_x9.id {
        return Err(ApiError::bad_request(
            "không thể tự thu hồi tài khoản mình",
        ));
    }
    users::update(
        &r_x8.pool,
        r_x9.id,
        users::Tph {
            is_active: Some(false),
            ..Default::default()
        },
    )
    .await?;
    users::f_wi(&r_x8.pool, r_x9.id).await?;
    tokens::f_29z(&r_x8.pool, r_x9.id, "user.revoke").await?;
    audit::append(
        &r_x8.pool,
        audit::Tp {
            actor_id: Some(r_x6.user.id),
            device_id: r_x6.claims.device_id,
            action: "user.revoke",
            target: Some(format!("user:{}", r_x9.id)),
            detail: None,
            ip: r_x6.ip(),
        },
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tpi {
    pub effective: Vec<Permission>,
    pub grants: Vec<Permission>,
    pub denies: Vec<Permission>,
    pub role_defaults: Vec<Permission>,
    pub grantable: Vec<Permission>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tlf {
    #[serde(default)]
    pub grants: Vec<Permission>,
    #[serde(default)]
    pub denies: Vec<Permission>,
}
fn f_vt(r_xa: &Ts, role: Role, r_xb: &PermissionOverrides) -> Tpi {
    Tpi {
        effective: f_18s(role, r_xb).into_iter().collect(),
        grants: r_xb.grants.iter().copied().collect(),
        denies: r_xb.denies.iter().copied().collect(),
        role_defaults: Permission::ALL
            .into_iter()
            .filter(|p| p.granted_by_default(role))
            .collect(),
        grantable: Permission::ALL
            .into_iter()
            .filter(|p| r_xa.has_perm(*p))
            .collect(),
    }
}
async fn f_sw(r_xe: &AppState, r_xc: &Ts, r_xf: &User) -> Result<(), ApiError> {
    r_xc.require_perm(Permission::UserGrantPermissions)?;
    if r_xc.user.id == r_xf.id {
        return Err(ApiError::bad_request(
            "không thể tự sửa quyền của mình",
        ));
    }
    let r_xd = f_29a(&r_xe.pool, &r_xc.user).await?;
    if !f_x0(&r_xc.user, r_xf, &r_xd) {
        return Err(ApiError::forbidden());
    }
    Ok(())
}
async fn f_1f1(
    State(r_xi): State<AppState>,
    r_xg: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<Tpi>, ApiError> {
    let r_xj = f_1ox(&r_xi, id).await?;
    f_sw(&r_xi, &r_xg, &r_xj).await?;
    let r_xh = permissions::f_1o5(&r_xi.pool, r_xj.id).await?;
    Ok(Json(f_vt(&r_xg, r_xj.role, &r_xh)))
}
async fn f_24z(
    State(r_xo): State<AppState>,
    r_xk: Ts,
    Path(id): Path<Uuid>,
    Json(r_xn): Json<Tlf>,
) -> Result<Json<Tpi>, ApiError> {
    let r_xp = f_1ox(&r_xo, id).await?;
    f_sw(&r_xo, &r_xk, &r_xp).await?;
    for r_xm in &r_xn.grants {
        if !r_xk.has_perm(*r_xm) {
            return Err(ApiError::forbidden());
        }
    }
    let r_xl = PermissionOverrides {
        grants: r_xn.grants.into_iter().collect(),
        denies: r_xn.denies.into_iter().collect(),
    }
    .sanitized();
    permissions::f_2e7(&r_xo.pool, r_xp.id, &r_xl, Some(r_xk.user.id)).await?;
    audit::append(
        &r_xo.pool,
        audit::Tp {
            actor_id: Some(r_xk.user.id),
            device_id: r_xk.claims.device_id,
            action: "user.permissions",
            target: Some(format!("user:{}", r_xp.id)),
            detail: Some(serde_json::json!(
                { "grants" : r_xl.grants.iter().map(| r_xm | r_xm.as_str())
                .collect::< Vec < _ >> (), "denies" : r_xl.denies.iter().map(|
                r_xm | r_xm.as_str()).collect::< Vec < _ >> (), }
            )),
            ip: r_xk.ip(),
        },
    )
    .await?;
    Ok(Json(f_vt(&r_xk, r_xp.role, &r_xl)))
}
