// Copyright (c) 2026 DigiText. All rights reserved.
//! Lưu trữ hiện hành (Điều 19, 26/11/2026) — chuyển tiếp tới máy chủ `routes/nop_luu.rs` và `routes/danh_muc_ho_so.rs`.
//! Hai lệnh thay vì vài chục: `lt_get` (đọc, nằm trong danh sách lệnh chỉ đọc của live/topics.ts) và `lt_call` (ghi — sau
//! mỗi lần ghi xin đồng bộ chỉnh lý sớm vì trạng thái nộp lưu nằm trên cl_ho_so và trôi về máy trạm qua pull).
//! Chỉ nhận đường dẫn dưới `/nop-luu` và `/danh-muc-ho-so`; quyền thật kiểm ở máy chủ (luutru.*, danhmuc.*).
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use crate::chinhly::sync;
use digitext_core::Permission;
use serde_json::{json, Value};
use tauri::{AppHandle, State};

fn hop_le(path: &str) -> Result<(), AppError> {
    if (path.starts_with("/nop-luu") || path.starts_with("/danh-muc-ho-so")) && !path.contains("..") {
        Ok(())
    } else {
        Err(AppError::validation("đường dẫn không hợp lệ"))
    }
}

#[tauri::command]
pub async fn lt_get(db: State<'_, Db>, auth: State<'_, AuthState>, path: String) -> Result<Value, AppError> {
    hop_le(&path)?;
    f_2r8(&db, &auth, Permission::DossierRead, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<Value>(&base, &token, &path).await }
    })
    .await
}

#[tauri::command]
pub async fn lt_call(app: AppHandle, db: State<'_, Db>, auth: State<'_, AuthState>, method: String, path: String, body: Option<Value>) -> Result<Value, AppError> {
    hop_le(&path)?;
    let m = method.to_uppercase();
    let body = body.unwrap_or(json!({}));
    let out = f_2r8(&db, &auth, Permission::DossierRead, move |base, token| {
        let (m, path, body) = (m.clone(), path.clone(), body.clone());
        async move {
            match m.as_str() {
                "PUT" => client::f_24x::<Value>(&base, &token, &path, &body).await,
                "DELETE" => client::f_13z::<Value>(&base, &token, &path).await,
                // Nhập danh mục hàng nghìn dòng, tiếp nhận phiếu lớn dựng biên bản: cho thời gian dài.
                _ => client::post_long::<Value>(&base, &token, &path, &body).await,
            }
        }
    })
    .await?;
    sync::request_soon(&app);
    Ok(out)
}
