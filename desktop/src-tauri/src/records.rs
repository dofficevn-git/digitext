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
}
#[tauri::command]
pub async fn records_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Vec<RecordRow>, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::RecordView,
        move |r_2ad, r_2ae| async move {
            client::f_1eo::<Vec<RecordRow>>(&r_2ad, &r_2ae, "/records").await
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3f {
    #[serde(default)]
    pub project_id: Option<String>,
    pub kind: String,
    pub payload: serde_json::Value,
}
#[tauri::command]
pub async fn record_create(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T3f,
) -> Result<RecordRow, AppError> {
    let body = serde_json::json!(
        { "projectId" : args.project_id, "kind" : args.kind, "payload" : args.payload, }
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
    .map_err(|e| AppError::internal(format!("bHXhu5NuZyBsxrB1IGZpbGUgbOG7l2k6IA=={e}")))?
    .map_err(|e| AppError::internal(format!("bMawdSBmaWxlIGzhu5dpOiA={e}")))?;
    tracing::info!(
        record = % args.id, "cmVjb3JkX2Rvd25sb2FkOiDEkcOjIGzGsHUgUERGIGJpw6puIGLhuqNu"
    );
    Ok(r_2al)
}
