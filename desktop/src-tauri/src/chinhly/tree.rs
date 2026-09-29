// Copyright (c) 2026 DigiText. All rights reserved.
//! Cây thư mục quét lấy hồ sơ từ mục lục chỉnh lý (bảng `cl_ho_so` của dự án đang làm) → bảng `ho_so`
//! (nguồn của "Tạo cây thư mục" / "Phát cho dự án"). Giữ dòng đã có theo khoá (hộp, hồ sơ số).

use super::store::{self, HoSoFilter, ListFilter};
use crate::archive::types::{PhongIn, T97, T9p};
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use rusqlite::params;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedArgs {
    pub ma_du_an: String,
}

fn text(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) => Some(s.trim().to_owned()).filter(|s| !s.is_empty()),
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    }
}

#[tauri::command]
pub fn tree_seed_from_cl(auth: State<'_, AuthState>, db: State<'_, Db>, args: SeedArgs) -> Result<T9p, AppError> {
    let who = auth.require_perm(Permission::DossierCreate)?;
    let (project, _) = super::project::resolve(&db, &auth)?;
    let ma = args.ma_du_an.trim().to_owned();
    if ma.is_empty() {
        return Err(AppError::validation("Chưa chọn phông."));
    }
    let mut g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;

    // Phông trong mục lục chỉnh lý
    let lf = ListFilter { eq: vec![("ma_du_an".into(), json!(ma))], ..Default::default() };
    let (phongs, _) = store::list(&g, "cl_phong", &project, &lf)?;
    let Some(p) = phongs.into_iter().next() else {
        return Err(AppError::not_found("Mục lục chỉnh lý của dự án chưa có phông này. Tạo phông ở Chỉnh lý › Danh mục trước."));
    };
    let phong_id = text(p.get("id")).unwrap_or_default();
    let phong = PhongIn {
        ma_du_an: ma.clone(),
        ten_du_an: text(p.get("ten")).unwrap_or_else(|| ma.clone()),
        ma_khoi: None,
        phong_so: text(p.get("phong_so")),
        muc_luc_so: text(p.get("muc_luc_so")),
    };

    // Hồ sơ đã có trong cây (giữ nguyên)
    let existing: HashSet<(String, String)> = {
        let mut st = g.prepare("SELECT hop_so, ho_so_so FROM ho_so WHERE phong_id = ?1")?;
        let v = st
            .query_map(params![ma], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        v.into_iter().collect()
    };

    let (rows, _) = store::ho_so_list(&g, &project, &HoSoFilter { phong_id: Some(phong_id), ..Default::default() })?;
    let mut items: Vec<T97> = Vec::new();
    for r in &rows {
        let Some(ho_so_so) = text(r.get("ho_so_so")).or_else(|| text(r.get("so_tam"))) else { continue };
        let hop_so = text(r.get("hop_so")).unwrap_or_default();
        if existing.contains(&(hop_so.clone(), ho_so_so.clone())) {
            continue;
        }
        items.push(T97 {
            hop_so,
            ho_so_so,
            tieu_de: text(r.get("tieu_de")).unwrap_or_default(),
            thoi_gian: None,
            so_to: text(r.get("so_to")),
            thbq: text(r.get("ten_thbq")),
            ghi_chu: text(r.get("ghi_chu")),
            nhom: text(r.get("ten_phong_ban")),
            nam: text(r.get("nam_ket_thuc")),
            ngay_bat_dau: text(r.get("ngay_bat_dau")),
            ngay_ket_thuc: text(r.get("ngay_ket_thuc")),
        });
    }
    if items.is_empty() {
        return Ok(T9p { ho_so: 0, dossier: 0 });
    }
    let tx = g.transaction()?;
    let (ho_so, dossier) = crate::archive::commands::f_225(&tx, &phong, &items, None, &who.user_id)?;
    tx.commit()?;
    tracing::info!(phong = %ma, ho_so, "tree_seed_from_cl");
    Ok(T9p { ho_so, dossier })
}
