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
pub struct Thc {
    pub id: String,
    pub project_id: String,
    pub kind: String,
    pub status: String,
    pub ho_so_ref: Option<String>,
    pub label: String,
    pub objid: Option<String>,
    pub zip_name: Option<String>,
    pub zip_sha256: Option<String>,
    pub zip_size: Option<i64>,
    pub document_count: i32,
    pub error: Option<String>,
    pub virus_scan: Option<String>,
    pub submission_id: Option<String>,
    pub converted_from: Option<String>,
    pub stt_from: Option<i32>,
    pub stt_to: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}
#[tauri::command]
pub async fn packages_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Vec<Thc>, AppError> {
    f_2r8(
            &db,
            &auth,
            Permission::PackageView,
            move |r_2a0, r_2a1| async move {
                client::f_1eo::<Vec<Thc>>(&r_2a0, &r_2a1, "/packages").await
            },
        )
        .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3c {
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub ho_so_ref: Option<String>,
    #[serde(default)]
    pub submission_id: Option<String>,
    #[serde(default)]
    pub stt_from: Option<i32>,
    #[serde(default)]
    pub stt_to: Option<i32>,
    pub request: serde_json::Value,
}
#[tauri::command]
pub async fn package_create(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T3c,
) -> Result<Thc, AppError> {
    let body = serde_json::json!(
        { "projectId" : args.project_id, "hoSoRef" : args.ho_so_ref, "submissionId" :
        args.submission_id, "sttFrom" : args.stt_from, "sttTo" : args.stt_to, "request" :
        args.request, }
    );
    f_2r8(
        &db,
        &auth,
        Permission::PackageCreate,
        move |r_2a2, r_2a3| {
            let body = body.clone();
            async move { client::f_23d::<Thc>(&r_2a2, &r_2a3, "/packages", &body).await }
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T56 {
    pub id: String,
    pub zip_name: String,
}
#[tauri::command]
pub async fn package_download(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T56,
) -> Result<String, AppError> {
    let id = args.id.clone();
    let r_2a5 = f_2r8(
        &db,
        &auth,
        Permission::PackageDownload,
        move |r_2a4, r_2ac| {
            let id = id.clone();
            async move { client::f_1ei(&r_2a4, &r_2ac, &format!("/packages/{id}/download")).await }
        },
    )
    .await?;
    let r_2a9 = crate::archive::paths::f_29d(&app, crate::archive::paths::f_112(&db).as_deref())?;
    let r_2a6 = r_2a9.join("_goi_tin");
    let zip_name = args.zip_name.clone();
    let r_2a8 = tauri::async_runtime::spawn_blocking(move || -> std::io::Result<String> {
        std::fs::create_dir_all(&r_2a6)?;
        let r_2ab = std::path::PathBuf::from(&zip_name)
            .file_name()
            .map(|r_2aa| r_2aa.to_string_lossy().into_owned())
            .unwrap_or_else(|| "goi-tin.zip".into());
        let r_2a7 = r_2a6.join(r_2ab);
        std::fs::write(&r_2a7, &r_2a5)?;
        Ok(r_2a7.to_string_lossy().into_owned())
    })
    .await
    .map_err(|e| AppError::internal(format!("bHXhu5NuZyBsxrB1IGZpbGUgbOG7l2k6IA=={e}")))?
    .map_err(|e| AppError::internal(format!("bMawdSBmaWxlIGzhu5dpOiA={e}")))?;
    tracing::info!(
        package = % args.id, "cGFja2FnZV9kb3dubG9hZDogxJHDoyBsxrB1IFpJUCBnw7NpIHRpbg=="
    );
    Ok(r_2a8)
}
