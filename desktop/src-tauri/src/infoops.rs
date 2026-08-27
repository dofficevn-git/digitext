// Copyright (c) 2026 DigiText. All rights reserved.
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use serde::Deserialize;
use tauri::State;
#[tauri::command]
pub async fn info_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<serde_json::Value, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::AccessView,
        move |r_1wj, r_1wk| async move {
            client::f_1eo::<serde_json::Value>(&r_1wj, &r_1wk, "/info").await
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyArgs {
    pub body: serde_json::Value,
}
#[tauri::command]
pub async fn info_create(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: BodyArgs,
) -> Result<serde_json::Value, AppError> {
    let body = args.body;
    f_2r8(
        &db,
        &auth,
        Permission::AccessRequest,
        move |r_1wl, r_1wm| {
            let body = body.clone();
            async move { client::f_23d::<serde_json::Value>(&r_1wl, &r_1wm, "/info", &body).await }
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T9t {
    pub id: String,
    pub action: String,
    pub note: Option<String>,
    pub bien_tap: Option<String>,
}
#[tauri::command]
pub async fn info_action(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T9t,
) -> Result<serde_json::Value, AppError> {
    if !matches!(
        args.action.as_str(),
        "approve" | "refuse" | "pay" | "generate" | "fulfill"
    ) {
        return Err(AppError::validation(format!(
            "hành động không hợp lệ: {}",
            args.action
        )));
    }
    let T9t {
        id,
        action,
        note,
        bien_tap,
    } = args;
    f_2r8(&db, &auth, Permission::AccessReview, move |r_1wn, r_1wo| {
        let (id, action, note, bien_tap) =
            (id.clone(), action.clone(), note.clone(), bien_tap.clone());
        async move {
            client::f_23d::<serde_json::Value>(
                &r_1wn,
                &r_1wo,
                &format!("/info/{id}/{action}"),
                &serde_json::json!({ "note" : note, "bienTap" : bien_tap }),
            )
            .await
        }
    })
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadArgs {
    pub id: String,
    pub file_name: String,
}
#[tauri::command]
pub async fn info_download(
    app: tauri::AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: DownloadArgs,
) -> Result<String, AppError> {
    let id = args.id.clone();
    let r_1wq = f_2r8(&db, &auth, Permission::AccessView, move |r_1wp, r_1wx| {
        let id = id.clone();
        async move { client::f_1ei(&r_1wp, &r_1wx, &format!("/info/{id}/download")).await }
    })
    .await?;
    let r_1wu = crate::archive::paths::f_29d(&app, crate::archive::paths::f_112(&db).as_deref())?;
    let r_1wr = r_1wu.join("_khai_thac");
    let file_name = args.file_name.clone();
    let r_1wt = tauri::async_runtime::spawn_blocking(move || -> std::io::Result<String> {
        std::fs::create_dir_all(&r_1wr)?;
        let r_1ww = std::path::PathBuf::from(&file_name)
            .file_name()
            .map(|r_1wv| r_1wv.to_string_lossy().into_owned())
            .unwrap_or_else(|| "cung-cap-thong-tin.pdf".into());
        let r_1ws = r_1wr.join(r_1ww);
        std::fs::write(&r_1ws, &r_1wq)?;
        Ok(r_1ws.to_string_lossy().into_owned())
    })
    .await
    .map_err(|e| AppError::internal(format!("luồng lưu file lỗi: {e}")))?
    .map_err(|e| AppError::internal(format!("lưu file lỗi: {e}")))?;
    tracing::info!(
        request = % args.id, "info_download: đã lưu kết quả"
    );
    Ok(r_1wt)
}
#[tauri::command]
pub async fn ops_integrity_report(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<serde_json::Value, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::BkView,
        move |r_1wy, r_1wz| async move {
            client::f_1eo::<serde_json::Value>(&r_1wy, &r_1wz, "/ops/integrity").await
        },
    )
    .await
}
#[tauri::command]
pub async fn ops_integrity_run(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<serde_json::Value, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::BkManage,
        move |r_1x0, r_1x1| async move {
            client::f_23d::<serde_json::Value>(
                &r_1x0,
                &r_1x1,
                "/ops/integrity/run",
                &serde_json::json!({}),
            )
            .await
        },
    )
    .await
}
#[tauri::command]
pub async fn ops_backups(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<serde_json::Value, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::BkView,
        move |r_1x2, r_1x3| async move {
            client::f_1eo::<serde_json::Value>(&r_1x2, &r_1x3, "/ops/backups").await
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T11 {
    pub loai: String,
    pub nguon: String,
    pub phuong_tien: Option<String>,
    pub ket_qua: String,
    pub chi_tiet: Option<String>,
}
#[tauri::command]
pub async fn ops_backup_report(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T11,
) -> Result<serde_json::Value, AppError> {
    let T11 {
        loai,
        nguon,
        phuong_tien,
        ket_qua,
        chi_tiet,
    } = args;
    f_2r8(&db, &auth, Permission::BkManage, move |r_1x4, r_1x5| {
        let (loai, nguon, phuong_tien, ket_qua, chi_tiet) = (
            loai.clone(),
            nguon.clone(),
            phuong_tien.clone(),
            ket_qua.clone(),
            chi_tiet.clone(),
        );
        async move {
            client::f_23d::<serde_json::Value>(
                &r_1x4,
                &r_1x5,
                "/ops/backups",
                &serde_json::json!(
                    { "loai" : loai, "nguon" : nguon, "phuongTien" :
                    phuong_tien, "ketQua" : ket_qua, "chiTiet" : chi_tiet, }
                ),
            )
            .await
        }
    })
    .await
}
