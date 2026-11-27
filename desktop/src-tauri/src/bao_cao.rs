// Copyright (c) 2026 DigiText. All rights reserved.
//! Báo cáo thống kê TT05 (27/11/2026, đợt 3): `bc_get` đọc JSON dưới `/bao-cao` (danh mục, số liệu), `bc_download` tải tệp
//! Excel / PDF máy chủ dựng (báo cáo `/bao-cao/{ma}?format=`, phiếu mượn `/cl/muon-tra/{id}/phieu.pdf`, hồ sơ theo kho
//! `/cl/kho-gia/ho-so?format=xlsx`) rồi mở hộp thoại Lưu. Quyền thật kiểm ở máy chủ (stats.view / loan.view / dossier.read).
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use serde_json::Value;
use tauri::{AppHandle, State};

fn hop_le(path: &str) -> Result<(), AppError> {
    let ok = (path.starts_with("/bao-cao") || path.starts_with("/cl/muon-tra/") || path.starts_with("/cl/kho-gia/")) && !path.contains("..");
    if ok {
        Ok(())
    } else {
        Err(AppError::validation("đường dẫn không hợp lệ"))
    }
}

#[tauri::command]
pub async fn bc_get(db: State<'_, Db>, auth: State<'_, AuthState>, path: String) -> Result<Value, AppError> {
    hop_le(&path)?;
    if !path.starts_with("/bao-cao") {
        return Err(AppError::validation("đường dẫn không hợp lệ"));
    }
    f_2r8(&db, &auth, Permission::StatsView, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<Value>(&base, &token, &path).await }
    })
    .await
}

/// Tải tệp máy chủ dựng rồi hỏi nơi lưu; trả đường dẫn đã ghi (None = người dùng huỷ). `default_name` phải có đuôi xlsx | pdf.
#[tauri::command]
pub async fn bc_download(app: AppHandle, db: State<'_, Db>, auth: State<'_, AuthState>, path: String, default_name: String) -> Result<Option<String>, AppError> {
    hop_le(&path)?;
    let ext = std::path::Path::new(&default_name).extension().and_then(|e| e.to_str()).map(str::to_lowercase).unwrap_or_default();
    if !matches!(ext.as_str(), "xlsx" | "pdf") {
        return Err(AppError::validation("Đuôi tệp phải là xlsx hoặc pdf."));
    }
    let perm = if path.starts_with("/bao-cao") { Permission::StatsView } else { Permission::DossierRead };
    let bytes = f_2r8(&db, &auth, perm, move |base, token| {
        let path = path.clone();
        async move { client::f_1ei(&base, &token, &path).await }
    })
    .await?;
    let ten = std::path::Path::new(&default_name).file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| format!("bao-cao.{ext}"));
    let ext2 = ext.clone();
    let chon = tauri::async_runtime::spawn_blocking(move || {
        use tauri_plugin_dialog::DialogExt;
        app.dialog().file().set_file_name(&ten).add_filter(ext2.to_uppercase(), &[ext2.as_str()]).blocking_save_file()
    })
    .await
    .map_err(|e| AppError::internal(format!("hộp thoại Lưu lỗi: {e}")))?;
    let Some(fp) = chon else { return Ok(None) };
    let path = fp.into_path().map_err(|e| AppError::internal(format!("đường dẫn không hợp lệ: {e}")))?;
    let p2 = path.clone();
    tauri::async_runtime::spawn_blocking(move || std::fs::write(&p2, &bytes))
        .await
        .map_err(|e| AppError::internal(format!("luồng ghi lỗi: {e}")))?
        .map_err(|e| AppError::internal(format!("không ghi được tệp: {e}")))?;
    Ok(Some(path.display().to_string()))
}
