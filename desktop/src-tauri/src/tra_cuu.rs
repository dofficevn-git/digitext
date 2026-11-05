// Copyright (c) 2026 DigiText. All rights reserved.
//! Tra cứu tài liệu lưu trữ (bản Lưu trữ GĐ4, 5/11/2026) — chuyển tiếp tới máy chủ `routes/tra_cuu.rs`: tìm không dấu trên
//! mục lục và lớp chữ, bộ lọc, bản đọc có watermark (trả base64 như `access_read`), nhật ký khai thác.
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use base64::Engine;
use digitext_core::Permission;
use serde::Deserialize;
use serde_json::Value;
use tauri::State;

fn pct(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(*b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// `{khoá: giá trị}` → `k=v&…` (bỏ giá trị rỗng / null).
fn query_string(q: &serde_json::Map<String, Value>) -> String {
    q.iter()
        .filter_map(|(k, v)| {
            let s = match v {
                Value::Null => return None,
                Value::String(s) if s.trim().is_empty() => return None,
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            Some(format!("{}={}", pct(k), pct(&s)))
        })
        .collect::<Vec<_>>()
        .join("&")
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimArgs {
    /// projectId, q, noiDung, phongId, hoSoId, thbqId, loaiVb, namTu, namDen, cheDo, daHuy, limit, offset
    pub query: serde_json::Map<String, Value>,
}

#[tauri::command]
pub async fn tra_cuu_tim(db: State<'_, Db>, auth: State<'_, AuthState>, args: TimArgs) -> Result<Value, AppError> {
    let path = format!("/tra-cuu?{}", query_string(&args.query));
    f_2r8(&db, &auth, Permission::AccessRequest, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<Value>(&base, &token, &path).await }
    })
    .await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuAnArgs {
    pub project_id: String,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[tauri::command]
pub async fn tra_cuu_bo_loc(db: State<'_, Db>, auth: State<'_, AuthState>, args: DuAnArgs) -> Result<Value, AppError> {
    let path = format!("/tra-cuu/bo-loc?projectId={}", pct(&args.project_id));
    f_2r8(&db, &auth, Permission::AccessRequest, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<Value>(&base, &token, &path).await }
    })
    .await
}

#[tauri::command]
pub async fn tra_cuu_nhat_ky(db: State<'_, Db>, auth: State<'_, AuthState>, args: DuAnArgs) -> Result<Value, AppError> {
    let path = format!("/tra-cuu/nhat-ky?projectId={}&limit={}", pct(&args.project_id), args.limit.unwrap_or(500));
    f_2r8(&db, &auth, Permission::AccessReview, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<Value>(&base, &token, &path).await }
    })
    .await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocArgs {
    pub document_id: String,
}

/// Bản đọc có watermark (base64) — máy chủ quyết định được đọc thẳng hay phải gửi yêu cầu khai thác.
#[tauri::command]
pub async fn tra_cuu_doc(db: State<'_, Db>, auth: State<'_, AuthState>, args: DocArgs) -> Result<String, AppError> {
    let path = format!("/tra-cuu/tai-lieu/{}/doc", pct(&args.document_id));
    let bytes = f_2r8(&db, &auth, Permission::AccessRequest, move |base, token| {
        let path = path.clone();
        async move { client::f_1ei(&base, &token, &path).await }
    })
    .await?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_bo_rong_va_ma_hoa() {
        let q = serde_json::json!({ "projectId": "p1", "q": "Quyết định", "noiDung": true, "phongId": null, "loaiVb": " ", "limit": 50 });
        let s = query_string(q.as_object().unwrap());
        assert!(s.contains("projectId=p1"));
        assert!(s.contains("q=Quy%E1%BA%BFt%20%C4%91%E1%BB%8Bnh"));
        assert!(s.contains("noiDung=true"));
        assert!(s.contains("limit=50"));
        assert!(!s.contains("phongId") && !s.contains("loaiVb"));
    }
}
