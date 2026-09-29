// Copyright (c) 2026 DigiText. All rights reserved.
use super::client::{self, ClientError, Tl4, Tl6, Tl7};
use super::commands;
use super::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::{Perm, Permission};
use serde::Deserialize;
use std::future::Future;
use tauri::State;
pub(crate) fn map_err(r_be: ClientError) -> AppError {
    match r_be {
        ClientError::Unreachable(detail) => {
            // Ghi lỗi thật: "Không kết nối được" còn che cả hết giờ chờ lẫn lỗi đọc phản hồi.
            tracing::warn!(error = % detail, "server_unreachable");
            AppError::new("server_unreachable", format!("Không kết nối được máy chủ ({detail})."))
        }
        ClientError::Api {
            code,
            message: r_bf,
            ..
        } => {
            let r_bd: &'static str = match code.as_str() {
                "forbidden" => "forbidden",
                "conflict" => "conflict",
                "not_found" => "not_found",
                "bad_request" => "bad_request",
                "revoked" => "revoked",
                "auth_failed" => "auth_failed",
                "unauthorized" => "unauthorized",
                _ => "server_error",
            };
            AppError::new(r_bd, r_bf)
        }
    }
}
async fn f_qj(
    r_bh: &State<'_, Db>,
    r_bg: &State<'_, AuthState>,
    r_bm: &str,
    r_bj: Option<&str>,
) -> Result<String, AppError> {
    let _guard = r_bg.lock_refresh().await;
    if let Some(r_bk) = r_bg.access_token() {
        if r_bj != Some(r_bk.as_str()) {
            return Ok(r_bk);
        }
    }
    let r_bi = r_bg
        .snapshot()
        .ok_or_else(|| AppError::new("unauthorized", "Chưa đăng nhập."))?;
    let r_bn = r_bg.wrap_key().ok_or_else(|| {
        AppError::new(
            "locked",
            "Mở khoá bằng mật khẩu trước khi quản trị.",
        )
    })?;
    let (_s, r_bl) = commands::f_26h(r_bh, r_bg, r_bn, r_bm, &r_bi.user_id).await?;
    Ok(r_bl)
}
pub(crate) async fn f_2r7<T, F, Fut>(
    r_bq: &State<'_, Db>,
    r_bo: &State<'_, AuthState>,
    r_br: Perm,
    r_bp: F,
) -> Result<T, AppError>
where
    F: Fn(String, String) -> Fut,
    Fut: Future<Output = Result<T, ClientError>>,
{
    r_bo.require(r_br)?;
    f_2r9(r_bq, r_bo, r_bp).await
}
pub(crate) async fn f_2r8<T, F, Fut>(
    r_bu: &State<'_, Db>,
    r_bs: &State<'_, AuthState>,
    r_bv: Permission,
    r_bt: F,
) -> Result<T, AppError>
where
    F: Fn(String, String) -> Fut,
    Fut: Future<Output = Result<T, ClientError>>,
{
    r_bs.require_perm(r_bv)?;
    f_2r9(r_bu, r_bs, r_bt).await
}
/// Như `f_2r8` nhưng chỉ cần MỘT trong các quyền (vd. document.list HOẶC work.execute).
pub(crate) async fn f_2r8_any<T, F, Fut>(
    db: &State<'_, Db>,
    auth: &State<'_, AuthState>,
    perms: &[Permission],
    f: F,
) -> Result<T, AppError>
where
    F: Fn(String, String) -> Fut,
    Fut: Future<Output = Result<T, ClientError>>,
{
    let mut last = None;
    for p in perms {
        match auth.require_perm(*p) {
            Ok(_) => return f_2r9(db, auth, f).await,
            Err(e) => last = Some(e),
        }
    }
    Err(last.unwrap_or_else(|| AppError::validation("thiếu quyền")))
}
async fn f_2r9<T, F, Fut>(
    r_by: &State<'_, Db>,
    r_bw: &State<'_, AuthState>,
    r_bx: F,
) -> Result<T, AppError>
where
    F: Fn(String, String) -> Fut,
    Fut: Future<Output = Result<T, ClientError>>,
{
    let r_c0 = commands::server_url(r_by)?;
    let token = match r_bw.access_token() {
        Some(r_bz) => r_bz,
        None => f_qj(r_by, r_bw, &r_c0, None).await?,
    };
    match r_bx(r_c0.clone(), token.clone()).await {
        Ok(r_c1) => Ok(r_c1),
        Err(e) if e.is_unauthorized() => {
            let token = f_qj(r_by, r_bw, &r_c0, Some(&token)).await?;
            r_bx(r_c0, token).await.map_err(map_err)
        }
        Err(e) => Err(map_err(e)),
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tb0 {
    #[serde(default)]
    pub project_id: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3h {
    pub username: String,
    pub password: String,
    pub display_name: String,
    #[serde(default)]
    pub email: Option<String>,
    pub role: String,
    #[serde(default)]
    pub project_id: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tp6 {
    pub id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tla {
    pub id: String,
    pub active: bool,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tj0 {
    pub id: String,
    pub new_password: String,
    #[serde(default)]
    pub old_password: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3d {
    pub code: String,
    pub name: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tlg {
    pub project_id: String,
    pub admin_ids: Vec<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thn {
    pub project_id: String,
    pub admin_id: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tav {
    #[serde(default)]
    pub limit: Option<i64>,
}
fn validate_password(r_c2: &str) -> Result<(), AppError> {
    if r_c2.chars().count() < 8 {
        return Err(AppError::validation(
            "Mật khẩu phải tối thiểu 8 ký tự.",
        ));
    }
    Ok(())
}
#[tauri::command]
pub async fn users_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tb0,
) -> Result<Vec<Tl6>, AppError> {
    let path = match args
        .project_id
        .as_deref()
        .map(str::trim)
        .filter(|r_c4| !r_c4.is_empty())
    {
        Some(pid) => format!("/users?projectId={pid}"),
        None => "/users".to_string(),
    };
    f_2r8(&db, &auth, Permission::UserView, move |r_c3, r_c5| {
        let path = path.clone();
        async move { client::f_1eo::<Vec<Tl6>>(&r_c3, &r_c5, &path).await }
    })
    .await
}
#[tauri::command]
pub async fn user_create(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T3h,
) -> Result<Tl6, AppError> {
    if args.username.trim().is_empty() {
        return Err(AppError::validation("Thiếu tên đăng nhập."));
    }
    if args.display_name.trim().is_empty() {
        return Err(AppError::validation("Thiếu tên hiển thị."));
    }
    validate_password(&args.password)?;
    let mut body = serde_json::json!(
        { "username" : args.username.trim(), "password" : args.password, "displayName" :
        args.display_name.trim(), "role" : args.role, }
    );
    if let Some(r_c7) = args
        .email
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        body["email"] = serde_json::Value::String(r_c7.to_string());
    }
    if let Some(r_c8) = args
        .project_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        body["projectId"] = serde_json::Value::String(r_c8.to_string());
    }
    f_2r8(&db, &auth, Permission::UserCreate, move |r_c6, r_c9| {
        let body = body.clone();
        async move { client::f_23d::<Tl6>(&r_c6, &r_c9, "/users", &body).await }
    })
    .await
}
#[tauri::command]
pub async fn user_update(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tp6,
) -> Result<Tl6, AppError> {
    let id = args.id.clone();
    let mut body = serde_json::json!({});
    if let Some(r_cb) = args
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|r_cd| !r_cd.is_empty())
    {
        body["displayName"] = serde_json::Value::String(r_cb.to_string());
    }
    if let Some(r_cc) = args.email.as_deref().map(str::trim) {
        body["email"] = serde_json::Value::String(r_cc.to_string());
    }
    f_2r8(&db, &auth, Permission::UserUpdate, move |r_ca, r_ce| {
        let (id, body) = (id.clone(), body.clone());
        async move { client::f_20e::<Tl6>(&r_ca, &r_ce, &format!("/users/{id}"), &body).await }
    })
    .await
}
#[tauri::command]
pub async fn user_set_active(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tla,
) -> Result<(), AppError> {
    let id = args.id.clone();
    let active = args.active;
    f_2r8(&db, &auth, Permission::UserRevoke, move |r_cf, r_cg| {
        let id = id.clone();
        async move {
            if active {
                client::f_20e::<Tl6>(
                    &r_cf,
                    &r_cg,
                    &format!("/users/{id}"),
                    &serde_json::json!({ "isActive" : true }),
                )
                .await
                .map(|_| ())
            } else {
                client::f_23c(
                    &r_cf,
                    &r_cg,
                    &format!("/users/{id}/revoke"),
                    &serde_json::json!({}),
                )
                .await
            }
        }
    })
    .await
}
#[tauri::command]
pub async fn user_reset_password(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tj0,
) -> Result<(), AppError> {
    validate_password(&args.new_password)?;
    let id = args.id.clone();
    let mut body = serde_json::json!({ "newPassword" : args.new_password });
    if let Some(r_ci) = args.old_password.clone() {
        body["oldPassword"] = serde_json::Value::String(r_ci);
    }
    f_2r8(
            &db,
            &auth,
            Permission::UserResetPassword,
            move |r_ch, r_cj| {
                let (id, body) = (id.clone(), body.clone());
                async move {
                    client::f_23c(&r_ch, &r_cj, &format!("/users/{id}/password"), &body)
                        .await
                }
            },
        )
        .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tpg {
    pub id: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tll {
    pub id: String,
    #[serde(default)]
    pub grants: Vec<String>,
    #[serde(default)]
    pub denies: Vec<String>,
}
#[tauri::command]
pub async fn user_permissions_get(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tpg,
) -> Result<Tl7, AppError> {
    let id = args.id.clone();
    f_2r8(
            &db,
            &auth,
            Permission::UserGrantPermissions,
            move |r_ck, r_cl| {
                let id = id.clone();
                async move {
                    client::f_1eo::<
                        Tl7,
                    >(&r_ck, &r_cl, &format!("/users/{id}/permissions"))
                        .await
                }
            },
        )
        .await
}
#[tauri::command]
pub async fn user_permissions_set(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tll,
) -> Result<Tl7, AppError> {
    let id = args.id.clone();
    let body = serde_json::json!({ "grants" : args.grants, "denies" : args.denies });
    f_2r8(
        &db,
        &auth,
        Permission::UserGrantPermissions,
        move |r_cm, r_cn| {
            let (id, body) = (id.clone(), body.clone());
            async move {
                client::f_24x::<Tl7>(&r_cm, &r_cn, &format!("/users/{id}/permissions"), &body).await
            }
        },
    )
    .await
}
#[tauri::command]
pub async fn projects_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Vec<Tl4>, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::ProjectView,
        move |r_co, r_cp| async move { client::f_1eo::<Vec<Tl4>>(&r_co, &r_cp, "/projects").await },
    )
    .await
}
#[tauri::command]
pub async fn project_create(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T3d,
) -> Result<Tl4, AppError> {
    if args.code.trim().is_empty() || args.name.trim().is_empty() {
        return Err(AppError::validation(
            "Thiếu mã hoặc tên dự án.",
        ));
    }
    let body = serde_json::json!(
        { "code" : args.code.trim(), "name" : args.name.trim() }
    );
    f_2r8(&db, &auth, Permission::ProjectCreate, move |r_cq, r_cr| {
        let body = body.clone();
        async move { client::f_23d::<Tl4>(&r_cq, &r_cr, "/projects", &body).await }
    })
    .await
}
#[tauri::command]
pub async fn project_set_admins(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tlg,
) -> Result<Tl4, AppError> {
    let pid = args.project_id.clone();
    let body = serde_json::json!({ "adminIds" : args.admin_ids });
    f_2r8(
        &db,
        &auth,
        Permission::ProjectAssignAdmin,
        move |r_cs, r_ct| {
            let (pid, body) = (pid.clone(), body.clone());
            async move {
                client::f_24x::<Tl4>(&r_cs, &r_ct, &format!("/projects/{pid}/admins"), &body).await
            }
        },
    )
    .await
}
#[tauri::command]
pub async fn project_add_admin(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Thn,
) -> Result<Tl4, AppError> {
    let pid = args.project_id.clone();
    let body = serde_json::json!({ "adminId" : args.admin_id });
    f_2r8(
        &db,
        &auth,
        Permission::ProjectAssignAdmin,
        move |r_cu, r_cv| {
            let (pid, body) = (pid.clone(), body.clone());
            async move {
                client::f_23d::<Tl4>(&r_cu, &r_cv, &format!("/projects/{pid}/admins"), &body).await
            }
        },
    )
    .await
}
#[tauri::command]
pub async fn project_remove_admin(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Thn,
) -> Result<Tl4, AppError> {
    let pid = args.project_id.clone();
    let aid = args.admin_id.clone();
    f_2r8(
        &db,
        &auth,
        Permission::ProjectAssignAdmin,
        move |r_cw, r_cx| {
            let (pid, aid) = (pid.clone(), aid.clone());
            async move {
                client::f_13z::<Tl4>(&r_cw, &r_cx, &format!("/projects/{pid}/admins/{aid}")).await
            }
        },
    )
    .await
}
#[tauri::command]
pub async fn upload_batches_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tav,
) -> Result<Vec<client::Tp9>, AppError> {
    let path = match args.limit {
        Some(n) => format!("/upload-batches?limit={n}"),
        None => "/upload-batches".to_string(),
    };
    f_2r8(&db, &auth, Permission::DocumentList, move |r_cy, r_cz| {
        let path = path.clone();
        async move { client::f_1eo::<Vec<client::Tp9>>(&r_cy, &r_cz, &path).await }
    })
    .await
}
#[tauri::command]
pub async fn upload_batches_matrix(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<serde_json::Value, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::DocumentList,
        move |r_d0, r_d1| async move {
            client::f_1eo::<serde_json::Value>(&r_d0, &r_d1, "/upload-batches/matrix").await
        },
    )
    .await
}
