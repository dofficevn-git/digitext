// Copyright (c) 2026 DigiText. All rights reserved.
//! Bảo quản Điều 29–31 (0032, 26/11/2026) — chuyển tiếp tới máy chủ `routes/bao_quan.rs`. Như `luu_tru.rs`: hai lệnh `bq_get`
//! (chỉ đọc, nằm trong danh sách lệnh chỉ đọc của live/topics.ts) và `bq_call` (ghi). Chỉ nhận đường dẫn dưới `/bao-quan`;
//! quyền thật kiểm ở máy chủ (bk.view / bk.manage / bk.approve).
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use serde_json::{json, Value};
use tauri::State;

fn hop_le(path: &str) -> Result<(), AppError> {
    if path.starts_with("/bao-quan") && !path.contains("..") {
        Ok(())
    } else {
        Err(AppError::validation("đường dẫn không hợp lệ"))
    }
}

#[tauri::command]
pub async fn bq_get(db: State<'_, Db>, auth: State<'_, AuthState>, path: String) -> Result<Value, AppError> {
    hop_le(&path)?;
    f_2r8(&db, &auth, Permission::BkView, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<Value>(&base, &token, &path).await }
    })
    .await
}

#[tauri::command]
pub async fn bq_call(db: State<'_, Db>, auth: State<'_, AuthState>, method: String, path: String, body: Option<Value>) -> Result<Value, AppError> {
    hop_le(&path)?;
    let m = method.to_uppercase();
    let body = body.unwrap_or(json!({}));
    f_2r8(&db, &auth, Permission::BkView, move |base, token| {
        let (m, path, body) = (m.clone(), path.clone(), body.clone());
        async move {
            match m.as_str() {
                "PUT" => client::f_24x::<Value>(&base, &token, &path, &body).await,
                "DELETE" => client::f_13z::<Value>(&base, &token, &path).await,
                // Chạy một lô kiểm toàn vẹn / duyệt kế hoạch chốt hàng vạn tệp: cho thời gian dài.
                _ => client::post_long::<Value>(&base, &token, &path, &body).await,
            }
        }
    })
    .await
}
