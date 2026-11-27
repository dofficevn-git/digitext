// Copyright (c) 2026 DigiText. All rights reserved.
//! In biểu mẫu ra PDF NGAY TẠI MÁY TRẠM (27/11/2026, đợt 4 nhóm 3C): giao diện gửi mảng `ManualBlock` dạng JSON
//! (`{"t": "title", "v": "…"}` — xem `digitext_pdf::manual`), Rust dựng PDF bằng chính bộ dựng dùng cho báo cáo / phiếu trên máy chủ,
//! rồi mở hộp thoại Lưu. Chạy được ngoại tuyến (dữ liệu từ bản sao SQLite); một bộ dựng cho cả hai bản. Font: sans hệ thống, không có
//! thì font chứng thư (`f_1cq`).
use crate::auth::AuthState;
use crate::error::AppError;
use digitext_core::Perm;
use digitext_pdf::manual::{render_manual, render_manual_ngang, ManualBlock};
use serde_json::Value;
use tauri::{AppHandle, State};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InPdfArgs {
    pub blocks: Value,
    #[serde(default)]
    pub ngang: bool,
    pub default_name: String,
    /// Có → ghi thẳng vào thư mục này (bộ hồ sơ nghiệm thu), không hỏi.
    #[serde(default)]
    pub dir: Option<String>,
}

/// Dựng PDF từ khối JSON rồi lưu; trả đường dẫn (None = người dùng huỷ hộp thoại).
#[tauri::command]
pub async fn pdf_manual(app: AppHandle, auth: State<'_, AuthState>, args: InPdfArgs) -> Result<Option<String>, AppError> {
    auth.require(Perm::Read)?;
    let blocks: Vec<ManualBlock> = serde_json::from_value(args.blocks).map_err(|e| AppError::validation(format!("khối in không hợp lệ: {e}")))?;
    if blocks.is_empty() {
        return Err(AppError::validation("Không có gì để in."));
    }
    let font_path = digitext_pdf::sans_font().ok_or_else(|| AppError::validation("Không tìm thấy font TTF trên máy để dựng PDF."))?;
    let font = std::fs::read(&font_path).map_err(|e| AppError::internal(format!("đọc font {}: {e}", font_path.display())))?;
    let ngang = args.ngang;
    let bytes = tauri::async_runtime::spawn_blocking(move || if ngang { render_manual_ngang(&blocks, &font) } else { render_manual(&blocks, &font) })
        .await
        .map_err(|e| AppError::internal(format!("luồng dựng PDF lỗi: {e}")))?
        .map_err(|e| AppError::validation(format!("dựng PDF lỗi: {e}")))?;
    let ten = std::path::Path::new(&args.default_name).file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "bieu-mau.pdf".into());
    let ten = if ten.to_lowercase().ends_with(".pdf") { ten } else { format!("{ten}.pdf") };
    let path = match args.dir.as_deref().map(str::trim).filter(|d| !d.is_empty()) {
        Some(d) => {
            let dir = std::path::PathBuf::from(d);
            std::fs::create_dir_all(&dir).map_err(|e| AppError::validation(format!("không tạo được thư mục {}: {e}", dir.display())))?;
            dir.join(ten)
        }
        None => {
            let chon = tauri::async_runtime::spawn_blocking(move || {
                use tauri_plugin_dialog::DialogExt;
                app.dialog().file().set_file_name(&ten).add_filter("PDF", &["pdf"]).blocking_save_file()
            })
            .await
            .map_err(|e| AppError::internal(format!("hộp thoại Lưu lỗi: {e}")))?;
            let Some(fp) = chon else { return Ok(None) };
            fp.into_path().map_err(|e| AppError::internal(format!("đường dẫn không hợp lệ: {e}")))?
        }
    };
    let p2 = path.clone();
    tauri::async_runtime::spawn_blocking(move || std::fs::write(&p2, &bytes))
        .await
        .map_err(|e| AppError::internal(format!("luồng ghi lỗi: {e}")))?
        .map_err(|e| AppError::internal(format!("không ghi được tệp: {e}")))?;
    Ok(Some(path.display().to_string()))
}
