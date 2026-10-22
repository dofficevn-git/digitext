// Copyright (c) 2026 DigiText. All rights reserved.
//! Vai trò và bộ quyền — lệnh desktop gọi thẳng API /roles (22/10/2026). Máy chủ kiểm quyền và tầng vai trò gốc.
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use tauri::State;

type J = serde_json::Value;

fn code_ok(c: &str) -> Result<String, AppError> {
    let c = c.trim().to_owned();
    if !digitext_core::is_role_code(&c) {
        return Err(AppError::validation("mã vai trò không hợp lệ"));
    }
    Ok(c)
}

#[tauri::command]
pub async fn roles_list(db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<J, AppError> {
    f_2r8(&db, &auth, Permission::UserView, move |base, token| async move { client::f_1eo::<J>(&base, &token, "/roles").await }).await
}

#[tauri::command]
pub async fn role_create(db: State<'_, Db>, auth: State<'_, AuthState>, body: J) -> Result<J, AppError> {
    f_2r8(&db, &auth, Permission::UserGrantPermissions, move |base, token| {
        let body = body.clone();
        async move { client::f_23d::<J>(&base, &token, "/roles", &body).await }
    })
    .await
}

#[tauri::command]
pub async fn role_update(db: State<'_, Db>, auth: State<'_, AuthState>, code: String, body: J) -> Result<J, AppError> {
    let path = format!("/roles/{}", code_ok(&code)?);
    f_2r8(&db, &auth, Permission::UserGrantPermissions, move |base, token| {
        let (path, body) = (path.clone(), body.clone());
        async move { client::f_24x::<J>(&base, &token, &path, &body).await }
    })
    .await
}

#[tauri::command]
pub async fn role_reset(db: State<'_, Db>, auth: State<'_, AuthState>, code: String) -> Result<J, AppError> {
    let path = format!("/roles/{}/reset", code_ok(&code)?);
    f_2r8(&db, &auth, Permission::UserGrantPermissions, move |base, token| {
        let path = path.clone();
        async move { client::f_23d::<J>(&base, &token, &path, &serde_json::json!({})).await }
    })
    .await
}

#[tauri::command]
pub async fn role_delete(db: State<'_, Db>, auth: State<'_, AuthState>, code: String) -> Result<J, AppError> {
    let path = format!("/roles/{}", code_ok(&code)?);
    f_2r8(&db, &auth, Permission::UserGrantPermissions, move |base, token| {
        let path = path.clone();
        async move { client::f_13z::<J>(&base, &token, &path).await }
    })
    .await
}
