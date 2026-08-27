// Copyright (c) 2026 DigiText. All rights reserved.
use super::store;
use super::types::*;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::{Perm, Permission};
use tauri::State;
#[tauri::command]
pub fn khai_thac_search(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tky,
) -> Result<Ta4, AppError> {
    auth.require(Perm::Read)?;
    let r_1x6 = args.filter.unwrap_or_default();
    let r_1x7 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_1x9 = store::count(&r_1x7, &r_1x6)?;
    let r_1x8 = store::search(&r_1x7, &r_1x6)?;
    Ok(Ta4 {
        total: r_1x9,
        rows: r_1x8,
    })
}
#[tauri::command]
pub fn khai_thac_rows(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tky,
) -> Result<Vec<Ta5>, AppError> {
    auth.require(Perm::Read)?;
    let r_1xa = args.filter.unwrap_or_default();
    let r_1xb = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_2cr(&r_1xb, &r_1xa)?)
}
#[tauri::command]
pub fn khai_thac_van_ban(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tky,
) -> Result<Vec<Ta6>, AppError> {
    auth.require(Perm::Read)?;
    let r_1xc = args.filter.unwrap_or_default();
    let r_1xd = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_2pe(&r_1xd, &r_1xc)?)
}
#[tauri::command]
pub fn kho_list(auth: State<'_, AuthState>, db: State<'_, Db>) -> Result<Vec<Ta9>, AppError> {
    auth.require(Perm::Read)?;
    let r_1xe = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_1nq(&r_1xe)?)
}
#[tauri::command]
pub fn gia_list(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T8n,
) -> Result<Vec<T8o>, AppError> {
    auth.require(Perm::Read)?;
    let r_1xf = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_1nl(&r_1xf, args.ma_kho)?)
}
#[tauri::command]
pub fn kho_list_detail(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
) -> Result<Vec<Ta7>, AppError> {
    auth.require_perm(Permission::CatalogRead)?;
    let r_1xg = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_1nr(&r_1xg)?)
}
#[tauri::command]
pub fn kho_upsert(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Taa,
) -> Result<i64, AppError> {
    auth.require_perm(Permission::CatalogUpdate)?;
    if args.kho.ten_kho.trim().is_empty() {
        return Err(AppError::validation(
            "Thiếu tên kho lưu trữ.",
        ));
    }
    let r_1xh = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_2o2(&r_1xh, &args.kho)?)
}
#[tauri::command]
pub fn kho_delete(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tcr,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogDelete)?;
    let mut r_1xi = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_1xj = r_1xi.transaction()?;
    store::f_140(&r_1xj, args.ma_kho)?;
    r_1xj.commit()?;
    Ok(())
}
#[tauri::command]
pub fn gia_list_detail(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T8n,
) -> Result<Vec<T8l>, AppError> {
    auth.require_perm(Permission::CatalogRead)?;
    let r_1xk = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_1nm(&r_1xk, args.ma_kho)?)
}
#[tauri::command]
pub fn gia_upsert(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T8p,
) -> Result<i64, AppError> {
    auth.require_perm(Permission::CatalogUpdate)?;
    if args.gia.ten_gia.trim().is_empty() {
        return Err(AppError::validation(
            "Thiếu tên giá lưu trữ.",
        ));
    }
    if let (Some(r_1xn), Some(r_1xl)) = (args.gia.tu_hop, args.gia.den_hop) {
        if r_1xl < r_1xn {
            return Err(AppError::validation(
                "Đến hộp phải ≥ Từ hộp.",
            ));
        }
    }
    let r_1xm = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_2ny(&r_1xm, &args.gia)?)
}
#[tauri::command]
pub fn gia_delete(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tcq,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogDelete)?;
    let mut r_1xo = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_1xp = r_1xo.transaction()?;
    store::f_13w(&r_1xp, args.ma_gia)?;
    r_1xp.commit()?;
    Ok(())
}
#[tauri::command]
pub fn dossier_assign_gia(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tn,
) -> Result<usize, AppError> {
    auth.require_perm(Permission::DossierUpdate)?;
    if args.dossier_ids.is_empty() {
        return Ok(0);
    }
    let mut r_1xq = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_1xs = r_1xq.transaction()?;
    let r_1xr = store::f_sh(&r_1xs, args.ma_gia, &args.dossier_ids)?;
    r_1xs.commit()?;
    Ok(r_1xr)
}
#[tauri::command]
pub fn dossier_assign_gia_by_range(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tcq,
) -> Result<usize, AppError> {
    auth.require_perm(Permission::DossierUpdate)?;
    let r_1xt = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_si(&r_1xt, args.ma_gia)?)
}
