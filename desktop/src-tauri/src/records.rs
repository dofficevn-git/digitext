// Copyright (c) 2026 DigiText. All rights reserved.
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordRow {
    pub id: String,
    pub project_id: String,
    pub kind: String,
    pub so_bien_ban: String,
    pub title: String,
    pub submission_id: Option<String>,
    pub created_at: String,
    /// Khoá phụ (0024): mã phông / ngày / lần bàn giao.
    #[serde(default, rename = "ref")]
    pub ref_: Option<String>,
}
/// Lọc sổ biên bản (0024): `kinds` rỗng = mọi loại; `ref_` khoá phụ; `project_id` một dự án; máy chủ cũ bỏ qua tham số lạ.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListArgs {
    #[serde(default)]
    pub kinds: Vec<String>,
    #[serde(default, rename = "ref")]
    pub ref_: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}
/// Mã hoá phần trăm tối thiểu cho tham số truy vấn (chữ số, chữ cái ASCII, "-_.~" giữ nguyên).
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
#[tauri::command]
pub async fn records_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Option<ListArgs>,
) -> Result<Vec<RecordRow>, AppError> {
    let a = args.unwrap_or_default();
    let mut q: Vec<String> = vec![format!("limit={}", a.limit.unwrap_or(200).clamp(1, 200))];
    if !a.kinds.is_empty() {
        q.push(format!("kind={}", a.kinds.join(",")));
    }
    if let Some(r) = a.ref_.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        q.push(format!("ref={}", pct(r)));
    }
    if let Some(p) = a.project_id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        q.push(format!("projectId={p}"));
    }
    let path = format!("/records?{}", q.join("&"));
    f_2r8(&db, &auth, Permission::RecordView, move |r_2ad, r_2ae| {
        let path = path.clone();
        async move { client::f_1eo::<Vec<RecordRow>>(&r_2ad, &r_2ae, &path).await }
    })
    .await
}
/// Chi tiết biên bản kèm payload (xem lại / nhân bản).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDetail {
    #[serde(flatten)]
    pub row: RecordRow,
    pub payload: serde_json::Value,
}
#[tauri::command]
pub async fn record_get(db: State<'_, Db>, auth: State<'_, AuthState>, id: String) -> Result<RecordDetail, AppError> {
    let id = id.trim().to_string();
    if !id.chars().all(|c| c.is_ascii_hexdigit() || c == '-') || id.len() > 40 {
        return Err(AppError::validation("mã biên bản không hợp lệ"));
    }
    let path = format!("/records/{id}");
    f_2r8(&db, &auth, Permission::RecordView, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<RecordDetail>(&base, &token, &path).await }
    })
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3f {
    #[serde(default)]
    pub project_id: Option<String>,
    pub kind: String,
    pub payload: serde_json::Value,
    #[serde(default, rename = "ref")]
    pub ref_: Option<String>,
}
#[tauri::command]
pub async fn record_create(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T3f,
) -> Result<RecordRow, AppError> {
    let body = serde_json::json!(
        { "projectId" : args.project_id, "kind" : args.kind, "payload" : args.payload, "ref": args.ref_ }
    );
    f_2r8(&db, &auth, Permission::RecordCreate, move |r_2af, r_2ag| {
        let body = body.clone();
        async move { client::f_23d::<RecordRow>(&r_2af, &r_2ag, "/records", &body).await }
    })
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T58 {
    pub id: String,
    pub file_name: String,
}
#[tauri::command]
pub async fn record_download(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T58,
) -> Result<String, AppError> {
    let id = args.id.clone();
    let r_2ai = f_2r8(&db, &auth, Permission::RecordView, move |r_2ah, r_2ap| {
        let id = id.clone();
        async move { client::f_1ei(&r_2ah, &r_2ap, &format!("/records/{id}/download")).await }
    })
    .await?;
    let r_2am = crate::archive::paths::f_29d(&app, crate::archive::paths::f_112(&db).as_deref())?;
    let r_2aj = r_2am.join("_bien_ban");
    let file_name = args.file_name.clone();
    let r_2al = tauri::async_runtime::spawn_blocking(move || -> std::io::Result<String> {
        std::fs::create_dir_all(&r_2aj)?;
        let r_2ao = std::path::PathBuf::from(&file_name)
            .file_name()
            .map(|r_2an| r_2an.to_string_lossy().into_owned())
            .unwrap_or_else(|| "bien-ban.pdf".into());
        let r_2ak = r_2aj.join(r_2ao);
        std::fs::write(&r_2ak, &r_2ai)?;
        Ok(r_2ak.to_string_lossy().into_owned())
    })
    .await
    .map_err(|e| AppError::internal(format!("luồng lưu file lỗi: {e}")))?
    .map_err(|e| AppError::internal(format!("lưu file lỗi: {e}")))?;
    tracing::info!(
        record = % args.id, "record_download: đã lưu PDF biên bản"
    );
    Ok(r_2al)
}
