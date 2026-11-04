// Copyright (c) 2026 DigiText. All rights reserved.
//! Nén một thư mục thành `<thư mục>.zip` cạnh nó (bộ hồ sơ nghiệm thu, 4/11/2026). Tệp đã nén sẵn (zip, pdf, xlsx,
//! docx, ảnh) lưu nguyên (Stored), còn lại Deflate; đọc trực tiếp, không nạp cả tệp vào bộ nhớ.
use crate::auth::AuthState;
use crate::error::AppError;
use digitext_core::Permission;
use serde::Deserialize;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZipArgs {
    pub dir: String,
}

const DA_NEN: [&str; 10] = ["zip", "pdf", "xlsx", "docx", "jpg", "jpeg", "png", "tif", "tiff", "7z"];

fn liet_ke(goc: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut items: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    items.sort_by_key(|e| e.file_name());
    for e in items {
        let p = e.path();
        let t = e.file_type()?;
        if t.is_dir() {
            liet_ke(goc, &p, out)?;
        } else if t.is_file() {
            out.push(p);
        }
    }
    Ok(())
}

pub fn nen_thu_muc(dir: &Path) -> Result<PathBuf, String> {
    if !dir.is_dir() {
        return Err(format!("không có thư mục {}", dir.display()));
    }
    let ten = dir.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "bo-ho-so".into());
    let dich = dir.with_file_name(format!("{ten}.zip"));
    let mut files = Vec::new();
    liet_ke(dir, dir, &mut files).map_err(|e| format!("đọc thư mục lỗi: {e}"))?;
    let tam = dich.with_extension("zip.part");
    let f = std::fs::File::create(&tam).map_err(|e| format!("không tạo được {}: {e}", tam.display()))?;
    let mut w = zip::ZipWriter::new(std::io::BufWriter::new(f));
    let ket_qua = (|| -> Result<(), String> {
        for p in &files {
            let rel = p.strip_prefix(dir).map_err(|e| e.to_string())?;
            let name = format!("{ten}/{}", rel.to_string_lossy().replace('\\', "/"));
            let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
            let method = if DA_NEN.contains(&ext.as_str()) { zip::CompressionMethod::Stored } else { zip::CompressionMethod::Deflated };
            let opt = zip::write::SimpleFileOptions::default().compression_method(method).large_file(true);
            w.start_file(name, opt).map_err(|e| format!("zip: {e}"))?;
            let mut src = std::fs::File::open(p).map_err(|e| format!("mở {} lỗi: {e}", p.display()))?;
            std::io::copy(&mut src, &mut w).map_err(|e| format!("nén {} lỗi: {e}", p.display()))?;
        }
        let mut inner = w.finish().map_err(|e| format!("zip: {e}"))?;
        inner.flush().map_err(|e| format!("ghi zip lỗi: {e}"))?;
        Ok(())
    })();
    if let Err(e) = ket_qua {
        let _ = std::fs::remove_file(&tam);
        return Err(e);
    }
    if dich.exists() {
        std::fs::remove_file(&dich).map_err(|e| format!("không ghi đè được {}: {e}", dich.display()))?;
    }
    std::fs::rename(&tam, &dich).map_err(|e| format!("đổi tên zip lỗi: {e}"))?;
    Ok(dich)
}

/// Nén thư mục, trả đường dẫn tệp zip.
#[tauri::command]
pub async fn zip_dir(auth: State<'_, AuthState>, args: ZipArgs) -> Result<String, AppError> {
    auth.require_perm(Permission::RecordView)?;
    let dir = PathBuf::from(args.dir.trim());
    let r = tauri::async_runtime::spawn_blocking(move || nen_thu_muc(&dir))
        .await
        .map_err(|e| AppError::internal(format!("luồng nén lỗi: {e}")))?
        .map_err(AppError::validation)?;
    tracing::info!(zip = %r.display(), "zip_dir: đã nén bộ hồ sơ");
    Ok(r.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nen_thu_muc_giu_cau_truc() {
        let goc = std::env::temp_dir().join(format!("digitext-zipdir-{}", std::process::id()));
        let d = goc.join("Ho-so");
        std::fs::create_dir_all(d.join("02-Bien-ban")).unwrap();
        std::fs::write(d.join("01.xlsx"), b"x").unwrap();
        std::fs::write(d.join("02-Bien-ban").join("a.txt"), b"hello hello hello").unwrap();
        let z = nen_thu_muc(&d).unwrap();
        let mut a = zip::ZipArchive::new(std::fs::File::open(&z).unwrap()).unwrap();
        let mut ten: Vec<String> = (0..a.len()).map(|i| a.by_index(i).unwrap().name().to_string()).collect();
        ten.sort();
        assert_eq!(ten, vec!["Ho-so/01.xlsx", "Ho-so/02-Bien-ban/a.txt"]);
        std::fs::remove_dir_all(&goc).unwrap();
    }
}
