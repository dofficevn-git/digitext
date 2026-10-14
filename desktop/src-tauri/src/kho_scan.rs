// Copyright (c) 2026 DigiText. All rights reserved.
//! "Quét Kho" (14/10/2026): dò cả thư mục Kho máy này theo dự án → phông, đối chiếu sổ Kho
//! (`collected_docs`) để biết phông nào còn tệp CHƯA GHI SỔ (chép tay bằng Explorer) / chưa đẩy /
//! lỗi. Ghi sổ thì giao diện gọi lại `collect_add_files` với thư mục phông nằm trong Kho (ghi sổ
//! tại chỗ, không chép); đẩy theo phông thì lấy id ở `kho_pending_ids` rồi `data_forward { ids }`.
use crate::auth::AuthState;
use crate::collect::PROJECT_MARKER;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Perm;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, State};

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KhoScanArgs {
    #[serde(default)]
    pub project_id: Option<String>,
}

/// Một phông (thư mục cấp 1 dưới thư mục dự án) trong Kho. `phong` rỗng = tệp nằm ngay gốc dự án.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KhoPhong {
    pub project_id: Option<String>,
    pub project_dir: String,
    pub phong: String,
    pub path: String,
    pub total: u64,
    pub chua_ghi_so: u64,
    pub chua_ghi_so_bytes: u64,
    pub chua_day: u64,
    pub loi: u64,
    pub da_day: u64,
}

/// Trạng thái sổ Kho theo đường dẫn: (đã đẩy, lỗi không đẩy lại được, có lỗi lần đẩy gần nhất).
type SoKho = HashMap<String, (bool, bool, bool)>;

fn khoa(p: &str) -> String {
    p.replace('/', "\\").to_lowercase()
}

fn doc_so_kho(db: &State<'_, Db>) -> Result<SoKho, AppError> {
    let conn = db.0.lock().map_err(|_| AppError::internal("DB bận"))?;
    let mut st = conn
        .prepare(
            "SELECT storage_path, forwarded_at IS NOT NULL, forward_fatal <> 0, forward_error IS NOT NULL \
             FROM collected_docs",
        )
        .map_err(|e| AppError::internal(e.to_string()))?;
    let rows = st
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, (r.get::<_, bool>(1)?, r.get::<_, bool>(2)?, r.get::<_, bool>(3)?)))
        })
        .map_err(|e| AppError::internal(e.to_string()))?;
    let mut m = SoKho::new();
    for row in rows.flatten() {
        m.insert(khoa(&row.0), row.1);
    }
    Ok(m)
}

fn la_pdf(p: &Path) -> bool {
    p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
}

/// Cộng mọi tệp PDF dưới `dir` (đệ quy) vào `out`. Bỏ thư mục ẩn / tạm (bắt đầu bằng "." hoặc "_").
fn dem(dir: &Path, so: &SoKho, out: &mut KhoPhong) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let Ok(ft) = e.file_type() else { continue };
        let p = e.path();
        if ft.is_dir() {
            let ten = e.file_name().to_string_lossy().into_owned();
            if !(ten.starts_with('.') || ten.starts_with('_')) {
                dem(&p, so, out);
            }
        } else if ft.is_file() && la_pdf(&p) {
            them_tep(&p, e.metadata().map(|m| m.len()).unwrap_or(0), so, out);
        }
    }
}

fn them_tep(p: &Path, size: u64, so: &SoKho, out: &mut KhoPhong) {
    out.total += 1;
    match so.get(&khoa(&p.to_string_lossy())) {
        None => {
            out.chua_ghi_so += 1;
            out.chua_ghi_so_bytes += size;
        }
        Some((true, _, _)) => out.da_day += 1,
        Some((false, true, _)) | Some((false, false, true)) => out.loi += 1,
        Some((false, false, false)) => out.chua_day += 1,
    }
}

fn quet(root: &Path, chi_du_an: Option<&str>, so: &SoKho) -> Vec<KhoPhong> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else { return out };
    let mut du_an: Vec<PathBuf> = rd
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
        .filter(|p| {
            let ten = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            !(ten.starts_with('.') || ten.starts_with('_'))
        })
        .collect();
    du_an.sort();
    for dir in du_an {
        let project_id = std::fs::read_to_string(dir.join(PROJECT_MARKER))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        if let Some(chi) = chi_du_an {
            if project_id.as_deref() != Some(chi) {
                continue;
            }
        }
        let project_dir = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let moi = |phong: String, path: &Path| KhoPhong {
            project_id: project_id.clone(),
            project_dir: project_dir.clone(),
            phong,
            path: path.to_string_lossy().into_owned(),
            ..Default::default()
        };
        let mut goc = moi(String::new(), &dir);
        let mut phongs: Vec<KhoPhong> = Vec::new();
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        let mut con: Vec<_> = rd.flatten().collect();
        con.sort_by_key(|e| e.file_name());
        for e in con {
            let Ok(ft) = e.file_type() else { continue };
            let p = e.path();
            let ten = e.file_name().to_string_lossy().into_owned();
            if ft.is_dir() && !(ten.starts_with('.') || ten.starts_with('_')) {
                let mut ph = moi(ten, &p);
                dem(&p, so, &mut ph);
                if ph.total > 0 {
                    phongs.push(ph);
                }
            } else if ft.is_file() && la_pdf(&p) {
                them_tep(&p, e.metadata().map(|m| m.len()).unwrap_or(0), so, &mut goc);
            }
        }
        if goc.total > 0 {
            out.push(goc);
        }
        out.extend(phongs);
    }
    out
}

/// Dò Kho máy này theo dự án → phông, đếm tệp chưa ghi sổ / chưa đẩy / lỗi / đã đẩy.
#[tauri::command]
pub async fn kho_scan(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Option<KhoScanArgs>,
) -> Result<Vec<KhoPhong>, AppError> {
    auth.require(Perm::Admin)?;
    let root = crate::archive::paths::f_18r(&app, &db)?;
    let so = doc_so_kho(&db)?;
    let chi = args.and_then(|a| a.project_id).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    tauri::async_runtime::spawn_blocking(move || quet(&root, chi.as_deref(), &so))
        .await
        .map_err(|e| AppError::internal(format!("luồng quét Kho lỗi: {e}")))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KhoPendingArgs {
    /// Thư mục phông (hoặc dự án) trong Kho, đúng `path` của `kho_scan`.
    pub path: String,
}

/// Id các tệp đã ghi sổ mà chưa lên máy chủ (bỏ tệp máy chủ đã từ chối hẳn) nằm dưới `path`.
#[tauri::command]
pub fn kho_pending_ids(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: KhoPendingArgs,
) -> Result<Vec<i64>, AppError> {
    auth.require(Perm::Admin)?;
    let mut tien_to = args.path.trim().trim_end_matches(['\\', '/']).replace('/', "\\");
    if tien_to.is_empty() {
        return Err(AppError::validation("thiếu thư mục phông"));
    }
    tien_to.push('\\');
    let conn = db.0.lock().map_err(|_| AppError::internal("DB bận"))?;
    let mut st = conn
        .prepare(
            "SELECT id, storage_path FROM collected_docs \
             WHERE forwarded_at IS NULL AND forward_fatal = 0",
        )
        .map_err(|e| AppError::internal(e.to_string()))?;
    let tien_to = tien_to.to_lowercase();
    let ids = st
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| AppError::internal(e.to_string()))?
        .flatten()
        .filter(|(_, p)| khoa(p).starts_with(&tien_to))
        .map(|(id, _)| id)
        .collect();
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dem_theo_phong_va_so_kho() {
        let tmp = std::env::temp_dir().join(format!("kho-scan-{}", std::process::id()));
        let du_an = tmp.join("DA · Thử");
        let ho_so = du_an.join("PHONG A").join("HOP 01").join("HO SO 01");
        std::fs::create_dir_all(&ho_so).unwrap();
        std::fs::create_dir_all(du_an.join("_xem_tam")).unwrap();
        std::fs::write(du_an.join(PROJECT_MARKER), "p1").unwrap();
        for t in ["a.pdf", "b.PDF", "c.pdf", "ghi-chu.txt"] {
            std::fs::write(ho_so.join(t), b"x").unwrap();
        }
        std::fs::write(du_an.join("_xem_tam").join("bo-qua.pdf"), b"x").unwrap();
        std::fs::write(du_an.join("le.pdf"), b"x").unwrap();
        let mut so = SoKho::new();
        so.insert(khoa(&ho_so.join("a.pdf").to_string_lossy()), (true, false, false));
        so.insert(khoa(&ho_so.join("b.PDF").to_string_lossy()), (false, false, false));
        let r = quet(&tmp, None, &so);
        let _ = std::fs::remove_dir_all(&tmp);
        assert_eq!(r.len(), 2);
        let goc = &r[0];
        assert_eq!((goc.phong.as_str(), goc.total, goc.chua_ghi_so), ("", 1, 1));
        let a = &r[1];
        assert_eq!(a.project_id.as_deref(), Some("p1"));
        assert_eq!(a.phong, "PHONG A");
        assert_eq!((a.total, a.da_day, a.chua_day, a.chua_ghi_so), (3, 1, 1, 1));
        assert!(quet(&tmp, Some("khac"), &so).is_empty());
    }
}
