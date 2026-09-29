// Copyright (c) 2026 DigiText. All rights reserved.
use super::build::{f_1sb, f_vb, T1d, Td9};
use super::paths::{f_112, f_113, f_1w9, f_29d};
use super::types::*;
use crate::auth::AuthState;
use crate::db::{self, Db, T98};
use crate::error::AppError;
use crate::storage;
use digitext_core::{Perm, Permission};
use std::path::Path;
use tauri::{AppHandle, State};
type T89 = (String, String, String);
pub(crate) fn f_225(
    r_2s: &rusqlite::Transaction<'_>,
    r_2q: &PhongIn,
    r_2n: &[T97],
    r_2l: Option<&std::collections::HashMap<(String, String), String>>,
    r_2k: &str,
) -> Result<(usize, usize), AppError> {
    db::f_2o6(
        r_2s,
        &r_2q.ma_du_an,
        &r_2q.ten_du_an,
        r_2q.ma_khoi.as_deref(),
        r_2q.phong_so.as_deref(),
        r_2q.muc_luc_so.as_deref(),
    )?;
    let mut n = 0usize;
    for r_2m in r_2n {
        if r_2m.ho_so_so.trim().is_empty() {
            continue;
        }
        let r_2o = db::f_2nz(r_2s, &r_2m.to_input(&r_2q.ma_du_an))?;
        db::f_2h4(r_2s, r_2o, r_2k)?;
        if let Some(r_2p) = r_2l {
            if let Some(path) = r_2p.get(&(r_2m.hop_so.clone(), r_2m.ho_so_so.clone())) {
                db::f_2e4(r_2s, r_2o, path)?;
            }
        }
        n += 1;
    }
    let r_2r = crate::dossier::store::f_2if(r_2s, Some(&r_2q.ma_du_an), r_2k)?;
    Ok((n, r_2r.created))
}
#[tauri::command]
pub fn import_ho_so(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T9o,
) -> Result<T9p, AppError> {
    let r_2w = auth.require_perm(Permission::DossierCreate)?;
    let T9o {
        phong: r_2v,
        ho_sos: r_2u,
    } = args;
    if r_2v.ma_du_an.trim().is_empty() {
        return Err(AppError::validation(
            "Thiếu mã phông (cơ quan).",
        ));
    }
    let mut r_2t = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_2x = r_2t.transaction()?;
    let (ho_so, dossier) = f_225(&r_2x, &r_2v, &r_2u, None, &r_2w.user_id)?;
    r_2x.commit()?;
    tracing::info!(
        r_2v = % r_2v.ma_du_an, count = ho_so, dossier,
        "import_ho_so (kèm đồng bộ mục lục chỉnh lý)"
    );
    Ok(T9p { ho_so, dossier })
}
#[tauri::command]
pub async fn create_folder_tree(
    app: AppHandle,
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T8a,
) -> Result<T8b, AppError> {
    let r_2y = auth.require_perm(Permission::TreeAssign)?;
    apply_folder_tree(
        &app,
        &db,
        args.phong,
        args.ho_sos,
        args.options.strip_accents,
        args.options.storage_root,
        &r_2y.user_id,
    )
    .await
}
pub(crate) async fn apply_folder_tree(
    r_2z: &AppHandle,
    r_33: &State<'_, Db>,
    r_3f: PhongIn,
    r_39: Vec<T97>,
    r_3m: bool,
    r_3l: Option<String>,
    r_32: &str,
) -> Result<T8b, AppError> {
    if r_3f.ma_du_an.trim().is_empty() || r_3f.ten_du_an.trim().is_empty() {
        return Err(AppError::validation(
            "Thiếu thông tin cơ quan (Phông).",
        ));
    }
    let r_3e = r_3l.or_else(|| f_112(r_33));
    let r_3j = f_29d(r_2z, r_3e.as_deref())?;
    let r_3k = r_3j.display().to_string();
    let r_3i = r_3f.ten_du_an.clone();
    let r_3a = r_39.clone();
    let (r_3h, r_36) =
        tauri::async_runtime::spawn_blocking(move || -> Result<(String, Vec<T89>), String> {
            let r_31 = r_3j
                .canonicalize()
                .map_err(|e| format!("thư mục gốc lỗi: {e}"))?;
            let r_3g = storage::f_1jt(&r_3j, &[&r_3i], r_3m);
            std::fs::create_dir_all(&r_3g)
                .map_err(|e| format!("tạo thư mục cơ quan lỗi: {e}"))?;
            let mut r_3d: Vec<T89> = Vec::with_capacity(r_3a.len());
            for r_38 in &r_3a {
                if r_38.ho_so_so.trim().is_empty() {
                    continue;
                }
                let r_34 = storage::f_1jt(&r_3j, &[&r_3i, &r_38.hop_so, &r_38.ho_so_so], r_3m);
                std::fs::create_dir_all(&r_34)
                    .map_err(|e| format!("tạo thư mục hồ sơ lỗi: {e}"))?;
                let r_30 = r_34.canonicalize().map_err(|e| e.to_string())?;
                if !r_30.starts_with(&r_31) {
                    return Err(
                        "đường dẫn thư mục không hợp lệ".into(),
                    );
                }
                r_3d.push((
                    r_38.hop_so.clone(),
                    r_38.ho_so_so.clone(),
                    r_30.to_string_lossy().into_owned(),
                ));
            }
            Ok((r_3g.to_string_lossy().into_owned(), r_3d))
        })
        .await
        .map_err(|e| format!("luồng tạo thư mục lỗi: {e}"))??;
    let r_3o;
    {
        let mut r_35: std::collections::HashMap<(String, String), String> =
            std::collections::HashMap::new();
        for (r_3b, r_3c, path) in &r_36 {
            r_35.insert((r_3b.clone(), r_3c.clone()), path.clone());
        }
        let mut r_37 = r_33.0.lock().map_err(|_| AppError::DbPoisoned)?;
        let r_3p = r_37.transaction()?;
        let (_, r_3n) = f_225(&r_3p, &r_3f, &r_39, Some(&r_35), r_32)?;
        r_3p.commit()?;
        r_3o = r_3n;
    }
    tracing::info!(
        r_3f = % r_3f.ma_du_an, created = r_36.len(), r_3o,
        "apply_folder_tree (kèm đồng bộ mục lục chỉnh lý)"
    );
    Ok(T8b {
        root_path: r_3k,
        phong_folder: r_3h,
        created: r_36.len(),
        synced_dossier: r_3o,
    })
}
#[tauri::command]
pub async fn finalize_ho_so(
    app: AppHandle,
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T85,
) -> Result<T87, AppError> {
    let r_3x = auth.require_perm(Permission::DossierUpdate)?;
    if args.pages.is_empty() {
        return Err(AppError::validation(
            "Không có trang nào để đóng gói.",
        ));
    }
    if args.ho_so_id.is_none() {
        if args.ho_so.ho_so_so.trim().is_empty() {
            return Err(AppError::validation("Thiếu Hồ sơ số."));
        }
        if args.phong.ma_du_an.trim().is_empty() || args.phong.ten_du_an.trim().is_empty() {
            return Err(AppError::validation(
                "Thiếu thông tin cơ quan (Phông).",
            ));
        }
    }
    let r_3u = args.options.storage_root.clone().or_else(|| f_112(&db));
    let r_3w = f_29d(&app, r_3u.as_deref())?;
    let r_3s = T1d {
        root: r_3w,
        phong_name: args.phong.ten_du_an.clone(),
        hop_so: args.ho_so.hop_so.clone(),
        ho_so_so: args.ho_so.ho_so_so.clone(),
        pages: args.pages.clone(),
        quality: args.options.jpeg_quality,
        strip: args.options.strip_accents,
        phong_ma_du_an: args.phong.ma_du_an.clone(),
        phong_phong_so: args.phong.phong_so.clone().unwrap_or_default(),
        phong_muc_luc_so: args.phong.muc_luc_so.clone().unwrap_or_default(),
        ngay_bat_dau: args.ho_so.ngay_bat_dau.clone(),
        nam: args.ho_so.nam.clone(),
        thoi_gian: args.ho_so.thoi_gian.clone(),
        pdf_out: args.pdf_out.as_deref().map(std::path::PathBuf::from),
    };
    let r_3t = tauri::async_runtime::spawn_blocking(move || f_vb(r_3s))
        .await
        .map_err(|e| format!("luồng đóng gói lỗi: {e}"))??;
    let r_3v = args.phong;
    let r_3r = args.ho_so.to_input(&r_3v.ma_du_an);
    let id = {
        let mut r_3q = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        let r_3y = r_3q.transaction()?;
        let id = match args.ho_so_id {
            Some(id) => id,
            None => {
                db::f_2o6(
                    &r_3y,
                    &r_3v.ma_du_an,
                    &r_3v.ten_du_an,
                    r_3v.ma_khoi.as_deref(),
                    r_3v.phong_so.as_deref(),
                    r_3v.muc_luc_so.as_deref(),
                )?;
                db::f_2nz(&r_3y, &r_3r)?
            }
        };
        db::f_27w(&r_3y, id, &r_3t.pdf_path, &r_3t.trang)?;
        db::f_2e5(
            &r_3y,
            id,
            &r_3t.folder_path,
            &r_3t.pdf_path,
            r_3t.pdf_size,
            r_3t.page_count,
        )?;
        db::f_2h4(&r_3y, id, &r_3x.user_id)?;
        db::f_2h5(&r_3y, id, &r_3x.user_id)?;
        db::f_2h6(&r_3y, id, &r_3x.user_id)?;
        r_3y.commit()?;
        id
    };
    tracing::info!(
        ho_so_id = id,
        pages = r_3t.page_count,
        bytes = r_3t.pdf_size,
        "finalize_ho_so"
    );
    Ok(T87 {
        ho_so_id: id,
        folder_path: r_3t.folder_path,
        pdf_path: r_3t.pdf_path,
        pdf_size: r_3t.pdf_size,
        page_count: r_3t.page_count,
    })
}
#[tauri::command]
pub async fn merge_scanned_pages_pdf(
    app: AppHandle,
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Td8,
) -> Result<Tdb, AppError> {
    auth.require_perm(Permission::ScanExecute)?;
    if args.pages.is_empty() {
        return Err(AppError::validation(
            "Chưa chọn ảnh nào để gộp.",
        ));
    }
    if args.ho_so.ho_so_so.trim().is_empty() {
        return Err(
            AppError::validation(
                "Thiếu Hồ sơ số (cần để đặt tên & chọn thư mục lưu).",
            ),
        );
    }
    if args.phong.ten_du_an.trim().is_empty() {
        return Err(AppError::validation(
            "Thiếu thông tin cơ quan (Phông).",
        ));
    }
    let r_41 = args.options.storage_root.clone().or_else(|| f_112(&db));
    let r_42 = f_29d(&app, r_41.as_deref())?;
    let r_3z = Td9 {
        root: r_42,
        phong_name: args.phong.ten_du_an.clone(),
        hop_so: args.ho_so.hop_so.clone(),
        ho_so_so: args.ho_so.ho_so_so.clone(),
        pages: args.pages.clone(),
        quality: args.options.jpeg_quality,
        strip: args.options.strip_accents,
        phong_ma_du_an: args.phong.ma_du_an.clone(),
        phong_phong_so: args.phong.phong_so.clone().unwrap_or_default(),
        phong_muc_luc_so: args.phong.muc_luc_so.clone().unwrap_or_default(),
        ngay_bat_dau: args.ho_so.ngay_bat_dau.clone(),
        nam: args.ho_so.nam.clone(),
        thoi_gian: args.ho_so.thoi_gian.clone(),
    };
    let r_40 = tauri::async_runtime::spawn_blocking(move || f_1sb(r_3z))
        .await
        .map_err(|e| format!("luồng gộp PDF lỗi: {e}"))??;
    tracing::info!(
        pages = r_40.page_count,
        bytes = r_40.pdf_size,
        "merge_scanned_pages_pdf"
    );
    Ok(Tdb {
        pdf_path: r_40.pdf_path,
        pdf_size: r_40.pdf_size,
        page_count: r_40.page_count,
    })
}
#[tauri::command]
pub fn list_ho_so(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tau,
) -> Result<Vec<T98>, AppError> {
    auth.require_perm(Permission::DossierRead)?;
    let r_43 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(db::list_ho_so(&r_43, args.phong_id.as_deref())?)
}
#[tauri::command]
pub fn ho_so_delete(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T95,
) -> Result<(), AppError> {
    auth.require_perm(Permission::DossierDelete)?;
    let r_44 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    db::f_13x(&r_44, args.id)?;
    Ok(())
}
#[tauri::command]
pub fn ho_so_update(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T99,
) -> Result<(), AppError> {
    auth.require_perm(Permission::DossierUpdate)?;
    let T99 { id: r_46, ho_so } = args;
    if ho_so.ho_so_so.trim().is_empty() {
        return Err(AppError::validation(
            "Thiếu hồ sơ số (khóa định danh hồ sơ).",
        ));
    }
    let r_45 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_47 = db::f_2nf(&r_45, r_46, &ho_so.to_input(""))?;
    if !r_47 {
        return Err(AppError::not_found(
            "Không tìm thấy hồ sơ để cập nhật.",
        ));
    }
    Ok(())
}
#[tauri::command]
pub fn list_phong(auth: State<'_, AuthState>, db: State<'_, Db>) -> Result<Vec<db::Th9>, AppError> {
    auth.require_perm(Permission::CatalogRead)?;
    let r_48 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(db::list_phong(&r_48)?)
}
fn f_1fx(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
}
#[tauri::command]
pub fn open_pdf(auth: State<'_, AuthState>, args: Tfi) -> Result<(), AppError> {
    auth.require(Perm::Read)?;
    if !f_1fx(&args.path) {
        return Err(AppError::validation(
            "Chỉ mở được file .pdf.",
        ));
    }
    let r_49 = Path::new(&args.path);
    if !r_49.is_file() {
        return Err(AppError::not_found("File không tồn tại."));
    }
    f_1w9(r_49).map_err(|e| super::paths::f_1w7(&e))
}
const C_6O: [&str; 4] = ["xlsx", "html", "pdf", "csv"];
fn f_1ao(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}
#[tauri::command]
pub async fn save_export(
    app: AppHandle,
    auth: State<'_, AuthState>,
    args: Tkk,
) -> Result<Option<String>, AppError> {
    auth.require(Perm::Read)?;
    let ext = args.ext.as_deref().unwrap_or("html").to_lowercase();
    if !C_6O.contains(&ext.as_str()) {
        return Err(AppError::validation(
            "Đuôi file không hợp lệ.",
        ));
    }
    let path = if args.open {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CTR: AtomicU64 = AtomicU64::new(0);
        let n = CTR.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("digitext-export-{}-{n}.{ext}", std::process::id()))
    } else {
        let r_4e = args
            .default_name
            .as_deref()
            .map(|n| f_2bp(n, &ext))
            .unwrap_or_else(|| format!("digitext-export.{ext}"));
        let r_4c = ext.clone();
        let r_4a = tauri::async_runtime::spawn_blocking(move || {
            use tauri_plugin_dialog::DialogExt;
            app.dialog()
                .file()
                .set_file_name(&r_4e)
                .add_filter(r_4c.to_uppercase(), &[r_4c.as_str()])
                .blocking_save_file()
        })
        .await
        .map_err(|e| AppError::internal(format!("hộp thoại Lưu lỗi: {e}")))?;
        let Some(r_4d) = r_4a else {
            return Ok(None);
        };
        r_4d.into_path().map_err(|e| {
            AppError::internal(format!(
                "đường dẫn không hợp lệ: {e}"
            ))
        })?
    };
    let r_4b = args.data;
    let r_4f = path.clone();
    tauri::async_runtime::spawn_blocking(move || crate::storage::f_2rn(&r_4f, &r_4b))
        .await
        .map_err(|e| AppError::internal(format!("luồng ghi lỗi: {e}")))?
        .map_err(|e| AppError::internal(format!("không ghi được file: {e}")))?;
    if args.open {
        f_1w9(&path).map_err(|e| super::paths::f_1w7(&e))?;
    }
    Ok(Some(path.display().to_string()))
}
fn f_2bp(r_4g: &str, ext: &str) -> String {
    let base = Path::new(r_4g)
        .file_name()
        .map(|r_4h| r_4h.to_string_lossy().into_owned())
        .unwrap_or_default();
    let base = base.trim();
    if base.is_empty() {
        return format!("digitext-export.{ext}");
    }
    if f_1ao(base) == ext {
        base.to_string()
    } else {
        format!("{base}.{ext}")
    }
}
#[tauri::command]
pub fn get_storage_root(
    app: AppHandle,
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
) -> Result<String, AppError> {
    auth.require(Perm::Read)?;
    let r_4i = f_29d(&app, f_112(&db).as_deref())?;
    Ok(r_4i.display().to_string())
}
#[tauri::command]
pub fn set_storage_root(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tlk,
) -> Result<(), AppError> {
    auth.require(Perm::Admin)?;
    let r_4l = args
        .path
        .as_deref()
        .map(str::trim)
        .filter(|r_4k| !r_4k.is_empty());
    let r_4j = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    db::f_2ei(&r_4j, crate::config::SETTING_STORAGE_ROOT, r_4l)?;
    Ok(())
}
#[tauri::command]
pub fn delete_path(
    app: AppHandle,
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tgu,
) -> Result<(), AppError> {
    auth.require(Perm::Admin)?;
    let r_4n = f_113(&app, &db, &args.path)?;
    let r_4o = r_4n.to_string_lossy().to_string();
    let r_4m = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    if r_4n.is_dir() {
        std::fs::remove_dir_all(&r_4n)
            .map_err(|e| format!("xoá thư mục lỗi: {e}"))?;
        db::f_148(&r_4m, &r_4o)?;
    } else {
        std::fs::remove_file(&r_4n).map_err(|e| format!("xoá file lỗi: {e}"))?;
        db::f_102(&r_4m, &r_4o)?;
    }
    tracing::info!(path = % r_4o, "delete_path");
    Ok(())
}
fn f_yd(r_4p: &str, nhan: &str) -> Result<(), AppError> {
    if r_4p.is_empty() {
        return Err(AppError::validation(format!(
            "{nhan} không hợp lệ."
        )));
    }
    if r_4p.starts_with('_') {
        return Err(
            AppError::validation(
                format!(
                    "{nhan} không được bắt đầu bằng dấu gạch dưới (_): tiền tố này dành cho thư mục nội bộ của ứng dụng và sẽ không hiện trong Kho."
                ),
            ),
        );
    }
    Ok(())
}
#[tauri::command]
pub fn rename_path(
    app: AppHandle,
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tip,
) -> Result<Tgv, AppError> {
    auth.require(Perm::Admin)?;
    let r_4w = f_113(&app, &db, &args.path)?;
    let r_4x = r_4w
        .parent()
        .ok_or_else(|| AppError::validation("Không có thư mục cha."))?;
    let r_4r = r_4w.is_file();
    let ext = if r_4r {
        r_4w.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    } else {
        None
    };
    let stem = storage::sanitize(&args.new_name, 90, false);
    f_yd(&stem, "Tên")?;
    let r_4s = match &ext {
        Some(e) => format!("{stem}.{e}"),
        None => stem,
    };
    let r_4t = r_4x.join(&r_4s);
    if r_4t.exists() {
        return Err(AppError::validation(
            "Tên đã tồn tại trong thư mục.",
        ));
    }
    std::fs::rename(&r_4w, &r_4t).map_err(|e| format!("đổi tên lỗi: {e}"))?;
    let r_4v = r_4w.to_string_lossy().to_string();
    let r_4u = r_4t.to_string_lossy().to_string();
    {
        let r_4q = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        db::f_27b(&r_4q, &r_4v, &r_4u)?;
    }
    tracing::info!(from = % r_4v, to = % r_4u, "rename_path");
    Ok(Tgv { path: r_4u })
}
#[tauri::command]
pub fn create_subfolder(
    app: AppHandle,
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T39,
) -> Result<Tgv, AppError> {
    auth.require(Perm::Operate)?;
    let r_50 = f_113(&app, &db, &args.parent_path)?;
    if !r_50.is_dir() {
        return Err(AppError::validation(
            "Thư mục cha không hợp lệ.",
        ));
    }
    let r_4z = storage::sanitize(&args.name, 90, false);
    f_yd(&r_4z, "Tên thư mục")?;
    let r_4y = r_50.join(&r_4z);
    if r_4y.exists() {
        return Err(AppError::validation(
            "Thư mục đã tồn tại.",
        ));
    }
    std::fs::create_dir_all(&r_4y).map_err(|e| format!("tạo thư mục lỗi: {e}"))?;
    Ok(Tgv {
        path: r_4y.to_string_lossy().to_string(),
    })
}
#[tauri::command]
pub fn move_path(
    app: AppHandle,
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tdk,
) -> Result<Tgv, AppError> {
    auth.require(Perm::Admin)?;
    let r_58 = f_113(&app, &db, &args.path)?;
    let r_51 = f_29d(&app, f_112(&db).as_deref())?
        .canonicalize()
        .map_err(|e| format!("ổ lưu trữ lỗi: {e}"))?;
    let r_52 = Path::new(args.dest_dir.trim())
        .canonicalize()
        .map_err(|_| AppError::not_found("Thư mục đích không tồn tại."))?;
    if !r_52.starts_with(&r_51) {
        return Err(AppError::validation(
            "Đích ngoài ổ lưu trữ.",
        ));
    }
    if !r_52.is_dir() {
        return Err(AppError::validation(
            "Đích không phải thư mục.",
        ));
    }
    if r_52.starts_with(&r_58) {
        return Err(AppError::validation(
            "Không thể di chuyển thư mục vào chính nó.",
        ));
    }
    let r_54 = r_58
        .file_name()
        .ok_or_else(|| AppError::validation("Tên không hợp lệ."))?;
    let r_55 = r_52.join(r_54);
    if r_55 == r_58 {
        return Ok(Tgv {
            path: r_58.to_string_lossy().to_string(),
        });
    }
    if r_55.exists() {
        return Err(AppError::validation(
            "Đích đã có mục cùng tên.",
        ));
    }
    std::fs::rename(&r_58, &r_55).map_err(|e| format!("di chuyển lỗi: {e}"))?;
    let r_57 = r_58.to_string_lossy().to_string();
    let r_56 = r_55.to_string_lossy().to_string();
    {
        let r_53 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        db::f_27b(&r_53, &r_57, &r_56)?;
    }
    tracing::info!(from = % r_57, to = % r_56, "move_path");
    Ok(Tgv { path: r_56 })
}
fn f_1kq(r_5a: &str, r_59: bool) -> bool {
    if storage::f_1j2(r_5a) {
        return false;
    }
    r_59 || r_5a.to_lowercase().ends_with(".pdf")
}
#[tauri::command]
pub fn read_dir(
    app: AppHandle,
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tie,
) -> Result<Vec<T4q>, AppError> {
    auth.require(Perm::Read)?;
    let r_5c = f_29d(&app, f_112(&db).as_deref())?
        .canonicalize()
        .map_err(|e| format!("ổ lưu trữ lỗi: {e}"))?;
    let r_5d = match args.path.as_deref() {
        Some(r_5e) if !r_5e.trim().is_empty() => {
            let r_5b = Path::new(r_5e).canonicalize().map_err(|_| {
                AppError::not_found("đường dẫn không tồn tại")
            })?;
            if !r_5b.starts_with(&r_5c) {
                return Err(AppError::validation(
                    "Đường dẫn ngoài ổ lưu trữ.",
                ));
            }
            r_5b
        }
        _ => r_5c,
    };
    if !r_5d.is_dir() {
        return Err(AppError::validation("Không phải thư mục."));
    }
    f_1ns(&r_5d)
}
fn f_1ns(r_5f: &Path) -> Result<Vec<T4q>, AppError> {
    let mut r_5l: Vec<T4q> = Vec::new();
    for r_5g in std::fs::read_dir(r_5f)
        .map_err(|e| format!("đọc thư mục lỗi: {e}"))?
        .flatten()
    {
        let r_5k = r_5g.file_name().to_string_lossy().into_owned();
        let r_5j = r_5g.metadata().ok();
        let r_5h = r_5j.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        if !f_1kq(&r_5k, r_5h) {
            continue;
        }
        let r_5i = !r_5h;
        r_5l.push(T4q {
            name: r_5k,
            path: r_5g.path().to_string_lossy().into_owned(),
            is_dir: r_5h,
            is_pdf: r_5i,
            size: r_5j.map(|m| m.len()).unwrap_or(0),
        });
    }
    r_5l.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(r_5l)
}
#[tauri::command]
pub fn ho_so_at(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T88,
) -> Result<Option<db::T93>, AppError> {
    auth.require(Perm::Read)?;
    let r_5m = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(db::f_1cw(&r_5m, &args.folder_path)?)
}
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tm2 {
    pub configured: bool,
    pub exists: bool,
    pub path: Option<String>,
}
#[tauri::command]
pub fn storage_root_status(app: AppHandle, db: State<'_, Db>) -> Result<Tm2, AppError> {
    let path = super::paths::f_18r(&app, &db)?;
    Ok(Tm2 {
        configured: f_112(&db).is_some(),
        exists: path.exists(),
        path: Some(path.to_string_lossy().into_owned()),
    })
}
