// Copyright (c) 2026 DigiText. All rights reserved.
//! Gói việc & cộng tác viên xác nhận trường — lệnh desktop gọi thẳng API /work/* (kế hoạch 29/9/2026).
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use tauri::State;

type J = serde_json::Value;

fn uuid_ok(s: &str) -> Result<String, AppError> {
    let s = s.trim();
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        return Err(AppError::validation("mã không hợp lệ"));
    }
    Ok(s.to_owned())
}

#[tauri::command]
pub async fn work_list_packages(db: State<'_, Db>, auth: State<'_, AuthState>, project_id: Option<String>) -> Result<J, AppError> {
    let path = match project_id.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
        Some(p) => format!("/work/packages?projectId={}", uuid_ok(p)?),
        None => "/work/packages".to_owned(),
    };
    f_2r8(&db, &auth, Permission::WorkManage, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<J>(&base, &token, &path).await }
    })
    .await
}

#[tauri::command]
pub async fn work_create_package(db: State<'_, Db>, auth: State<'_, AuthState>, body: J) -> Result<J, AppError> {
    f_2r8(&db, &auth, Permission::WorkManage, move |base, token| {
        let body = body.clone();
        async move { client::f_23d::<J>(&base, &token, "/work/packages", &body).await }
    })
    .await
}

/// Chi tiết gói (quản trị: mọi gói trong phạm vi; CTV: gói mình là thành viên).
#[tauri::command]
pub async fn work_get_package(db: State<'_, Db>, auth: State<'_, AuthState>, id: String) -> Result<J, AppError> {
    let path = format!("/work/packages/{}", uuid_ok(&id)?);
    f_2r8(&db, &auth, Permission::WorkExecute, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<J>(&base, &token, &path).await }
    })
    .await
}

#[tauri::command]
pub async fn work_patch_package(db: State<'_, Db>, auth: State<'_, AuthState>, id: String, body: J) -> Result<J, AppError> {
    let path = format!("/work/packages/{}", uuid_ok(&id)?);
    f_2r8(&db, &auth, Permission::WorkManage, move |base, token| {
        let (path, body) = (path.clone(), body.clone());
        async move { client::f_20e::<J>(&base, &token, &path, &body).await }
    })
    .await
}

#[tauri::command]
pub async fn work_set_members(db: State<'_, Db>, auth: State<'_, AuthState>, id: String, user_ids: Vec<String>) -> Result<J, AppError> {
    let path = format!("/work/packages/{}/members", uuid_ok(&id)?);
    let body = serde_json::json!({ "userIds": user_ids });
    f_2r8(&db, &auth, Permission::WorkManage, move |base, token| {
        let (path, body) = (path.clone(), body.clone());
        async move { client::f_24x::<J>(&base, &token, &path, &body).await }
    })
    .await
}

#[tauri::command]
pub async fn work_review(db: State<'_, Db>, auth: State<'_, AuthState>, id: String, body: J) -> Result<J, AppError> {
    let path = format!("/work/packages/{}/review", uuid_ok(&id)?);
    f_2r8(&db, &auth, Permission::WorkManage, move |base, token| {
        let (path, body) = (path.clone(), body.clone());
        async move { client::f_23d::<J>(&base, &token, &path, &body).await }
    })
    .await
}

#[tauri::command]
pub async fn work_close_package(db: State<'_, Db>, auth: State<'_, AuthState>, id: String) -> Result<J, AppError> {
    let path = format!("/work/packages/{}/close", uuid_ok(&id)?);
    f_2r8(&db, &auth, Permission::WorkManage, move |base, token| {
        let path = path.clone();
        async move { client::f_23d::<J>(&base, &token, &path, &serde_json::json!({})).await }
    })
    .await
}

#[tauri::command]
pub async fn work_my_packages(db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<J, AppError> {
    f_2r8(&db, &auth, Permission::WorkExecute, |base, token| async move {
        client::f_1eo::<J>(&base, &token, "/work/my").await
    })
    .await
}

#[tauri::command]
pub async fn work_rejected_items(db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<J, AppError> {
    f_2r8(&db, &auth, Permission::WorkExecute, |base, token| async move {
        client::f_1eo::<J>(&base, &token, "/work/items/rejected").await
    })
    .await
}

#[tauri::command]
pub async fn work_next_item(db: State<'_, Db>, auth: State<'_, AuthState>, package_id: String) -> Result<J, AppError> {
    let path = format!("/work/packages/{}/next", uuid_ok(&package_id)?);
    f_2r8(&db, &auth, Permission::WorkExecute, move |base, token| {
        let path = path.clone();
        async move { client::f_23d::<J>(&base, &token, &path, &serde_json::json!({})).await }
    })
    .await
}

#[tauri::command]
pub async fn work_claim_item(db: State<'_, Db>, auth: State<'_, AuthState>, item_id: String) -> Result<J, AppError> {
    let path = format!("/work/items/{}/claim", uuid_ok(&item_id)?);
    f_2r8(&db, &auth, Permission::WorkExecute, move |base, token| {
        let path = path.clone();
        async move { client::f_23d::<J>(&base, &token, &path, &serde_json::json!({})).await }
    })
    .await
}

#[tauri::command]
pub async fn work_navigate(db: State<'_, Db>, auth: State<'_, AuthState>, item_id: String, direction: String) -> Result<J, AppError> {
    let path = format!("/work/items/{}/navigate", uuid_ok(&item_id)?);
    let body = serde_json::json!({ "direction": direction });
    f_2r8(&db, &auth, Permission::WorkExecute, move |base, token| {
        let (path, body) = (path.clone(), body.clone());
        async move { client::f_23d::<J>(&base, &token, &path, &body).await }
    })
    .await
}

#[tauri::command]
pub async fn work_release(db: State<'_, Db>, auth: State<'_, AuthState>, item_id: String) -> Result<J, AppError> {
    let path = format!("/work/items/{}/release", uuid_ok(&item_id)?);
    f_2r8(&db, &auth, Permission::WorkExecute, move |base, token| {
        let path = path.clone();
        async move { client::f_23d::<J>(&base, &token, &path, &serde_json::json!({})).await }
    })
    .await
}

#[tauri::command]
pub async fn work_done(db: State<'_, Db>, auth: State<'_, AuthState>, item_id: String) -> Result<J, AppError> {
    let path = format!("/work/items/{}/done", uuid_ok(&item_id)?);
    f_2r8(&db, &auth, Permission::WorkExecute, move |base, token| {
        let path = path.clone();
        async move { client::f_23d::<J>(&base, &token, &path, &serde_json::json!({})).await }
    })
    .await
}

#[tauri::command]
pub async fn work_check(db: State<'_, Db>, auth: State<'_, AuthState>, item_id: String, ok: bool, note: Option<String>) -> Result<J, AppError> {
    let path = format!("/work/items/{}/check", uuid_ok(&item_id)?);
    let body = serde_json::json!({ "ok": ok, "note": note });
    f_2r8(&db, &auth, Permission::WorkExecute, move |base, token| {
        let (path, body) = (path.clone(), body.clone());
        async move { client::f_23d::<J>(&base, &token, &path, &body).await }
    })
    .await
}
