// Copyright (c) 2026 DigiText. All rights reserved.
//! Đóng gói › Kiểm tra gói tin: kiểm các gói SIP/AIP/DIP do phần mềm KHÁC tạo (tệp .zip hoặc thư mục) theo
//! TT 05/2025/TT-BNV + E-ARK CSIP (`digitext_pkgcheck`), rồi xuất báo cáo PDF.
use crate::auth::AuthState;
use crate::error::AppError;
use digitext_core::Permission;
use digitext_pkgcheck::report::ReportHeader;
use digitext_pkgcheck::PackageResult;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, State};

static RUNNING: AtomicBool = AtomicBool::new(false);
static CANCEL: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub done: usize,
    pub total: usize,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunArgs {
    pub paths: Vec<String>,
}

/// Tìm gói trong các đường dẫn đã chọn rồi kiểm lần lượt (mỗi gói đọc trọn vào bộ nhớ).
#[tauri::command]
pub async fn pkgcheck_run(app: AppHandle, auth: State<'_, AuthState>, args: RunArgs) -> Result<Vec<PackageResult>, AppError> {
    auth.require_perm(Permission::PackageView)?;
    if RUNNING.swap(true, Ordering::SeqCst) {
        return Err(AppError::validation("đang có một đợt kiểm tra gói chạy"));
    }
    CANCEL.store(false, Ordering::SeqCst);
    let res = tauri::async_runtime::spawn_blocking(move || {
        let found = digitext_pkgcheck::discover(&args.paths);
        let total = found.len();
        let mut out = Vec::with_capacity(total);
        for (i, p) in found.iter().enumerate() {
            if CANCEL.load(Ordering::SeqCst) {
                break;
            }
            let name = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let _ = app.emit("pkgcheck://progress", Progress { done: i, total, name: name.clone() });
            let r = digitext_pkgcheck::check_path(p);
            tracing::info!(goi = %name, ket_luan = %r.verdict, loi = r.errors, canh_bao = r.warnings, "pkgcheck");
            out.push(r);
            let _ = app.emit("pkgcheck://progress", Progress { done: i + 1, total, name });
        }
        out
    })
    .await
    .map_err(|e| AppError::internal(format!("kiểm tra gói lỗi: {e}")));
    RUNNING.store(false, Ordering::SeqCst);
    let res = res?;
    if res.is_empty() {
        return Err(AppError::validation("không tìm thấy gói nào (tệp .zip hoặc thư mục có METS.xml)"));
    }
    Ok(res)
}

#[tauri::command]
pub async fn pkgcheck_cancel(auth: State<'_, AuthState>) -> Result<(), AppError> {
    auth.require_perm(Permission::PackageView)?;
    CANCEL.store(true, Ordering::SeqCst);
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportArgs {
    pub results: Vec<PackageResult>,
    pub header: ReportHeader,
}

/// Dựng báo cáo PDF, trả nội dung tệp — giao diện lưu qua `save_export` (hộp thoại lưu chung của ứng dụng).
#[tauri::command]
pub async fn pkgcheck_report(auth: State<'_, AuthState>, args: ReportArgs) -> Result<Vec<u8>, AppError> {
    auth.require_perm(Permission::PackageView)?;
    tauri::async_runtime::spawn_blocking(move || {
        let font = digitext_pdf::sans_font()
            .and_then(|p| std::fs::read(p).ok())
            .ok_or_else(|| AppError::internal("không tìm thấy phông chữ hệ thống (Arial / Segoe UI) để dựng PDF"))?;
        digitext_pkgcheck::report::render(&args.header, &args.results, &font).map_err(AppError::internal)
    })
    .await
    .map_err(|e| AppError::internal(format!("dựng báo cáo lỗi: {e}")))?
}
