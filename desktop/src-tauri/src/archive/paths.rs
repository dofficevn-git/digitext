// Copyright (c) 2026 DigiText. All rights reserved.
use crate::config;
use crate::db::{self, Db};
use crate::error::AppError;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};
pub fn f_22v(r_ar: Option<&str>, default_dir: PathBuf) -> PathBuf {
    match r_ar {
        Some(r_as) if !r_as.trim().is_empty() => PathBuf::from(r_as.trim()),
        _ => default_dir,
    }
}
pub fn f_2hn(r_at: &AppHandle, r_au: Option<&str>) -> Result<PathBuf, AppError> {
    let default_dir = r_at
        .path()
        .document_dir()
        .map_err(|e| {
            format!("không xác định được thư mục Documents: {e}")
        })?
        .join(config::STORAGE_DIR_NAME);
    Ok(f_22v(r_au, default_dir))
}
pub fn f_29d(r_av: &AppHandle, r_aw: Option<&str>) -> Result<PathBuf, AppError> {
    let r_ax = f_2hn(r_av, r_aw)?;
    std::fs::create_dir_all(&r_ax)
        .map_err(|e| format!("tạo thư mục lưu trữ lỗi: {e}"))?;
    Ok(r_ax)
}
pub fn f_112(r_ay: &State<'_, Db>) -> Option<String> {
    let r_az = r_ay.0.lock().ok()?;
    db::f_1ew(&r_az, config::SETTING_STORAGE_ROOT)
        .ok()
        .flatten()
        .filter(|r_b0| !r_b0.trim().is_empty())
}
pub fn f_18r(r_b1: &AppHandle, r_b2: &State<'_, Db>) -> Result<PathBuf, AppError> {
    f_2hn(r_b1, f_112(r_b2).as_deref())
}
pub fn f_113(r_b3: &AppHandle, r_b6: &State<'_, Db>, r_b7: &str) -> Result<PathBuf, AppError> {
    let r_b5 = f_18r(r_b3, r_b6)?
        .canonicalize()
        .map_err(|e| format!("ổ lưu trữ lỗi: {e}"))?;
    let r_b4 = Path::new(r_b7)
        .canonicalize()
        .map_err(|_| AppError::not_found("đường dẫn không tồn tại"))?;
    if !r_b4.starts_with(&r_b5) {
        return Err(AppError::validation(
            "Đường dẫn ngoài ổ lưu trữ — từ chối.",
        ));
    }
    if r_b4 == r_b5 {
        return Err(AppError::validation(
            "Không thể thao tác trên thư mục gốc.",
        ));
    }
    Ok(r_b4)
}
#[cfg(target_os = "macos")]
pub const OPEN_DENIED_HINT: &str = "macOS chưa cho DigiText đọc thư mục chứa file này. Mở Cài đặt Hệ thống → Quyền riêng tư & Bảo mật → Tệp và Thư mục và bật quyền cho DigiText (hoặc thêm DigiText vào Toàn quyền truy cập ổ đĩa), rồi thoát và mở lại DigiText. Lưu ý: sau mỗi lần cập nhật DigiText phải cấp lại quyền này.";
#[cfg(not(target_os = "macos"))]
pub const OPEN_DENIED_HINT: &str = "Không đủ quyền đọc file này.";
pub fn f_1w7(e: &std::io::Error) -> AppError {
    if e.kind() == std::io::ErrorKind::PermissionDenied {
        AppError::validation(OPEN_DENIED_HINT)
    } else {
        AppError::internal(format!("không mở được file: {e}"))
    }
}
pub fn f_1w9(r_b8: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    std::fs::File::open(r_b8)?;
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(r_b8).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg(r_b8)
            .spawn()?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(r_b8).spawn()?;
    }
    Ok(())
}
