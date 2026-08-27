// Copyright (c) 2026 DigiText. All rights reserved.
use super::store::{self, T2l};
use super::types::*;
use crate::auth::AuthState;
use crate::db::{self, Db};
use crate::error::AppError;
use digitext_core::Permission;
use tauri::State;
#[tauri::command]
pub fn catalog_list(auth: State<'_, AuthState>, db: State<'_, Db>) -> Result<T2l, AppError> {
    auth.require_perm(Permission::CatalogRead)?;
    let r_oj = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::snapshot(&r_oj)?)
}
#[tauri::command]
pub fn phong_ban_upsert(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Th6,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogUpdate)?;
    let r_ol = args.phong_ban;
    if r_ol.ma_dv.trim().is_empty() || r_ol.ma_phong.trim().is_empty() {
        return Err(AppError::validation(
            "VGhp4bq/dSBtw6MgcGjDsm5nIGJhbiBob+G6t2MgbcOjIHBow7RuZy4=",
        ));
    }
    if r_ol.ten_don_vi.trim().is_empty() {
        return Err(AppError::validation("VGhp4bq/dSB0w6puIHBow7JuZyBiYW4u"));
    }
    let r_ok = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_2o7(&r_ok, &r_ol)?;
    Ok(())
}
#[tauri::command]
pub fn phong_ban_delete(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tcp,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogDelete)?;
    let r_om = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_146(&r_om, &args.ma_dv)?;
    Ok(())
}
#[tauri::command]
pub fn mang_upsert(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tcx,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogUpdate)?;
    let r_oo = args.mang;
    if r_oo.ten_mang.trim().is_empty() || r_oo.ma_dv.trim().is_empty() {
        return Err(AppError::validation(
            "VGhp4bq/dSB0w6puIG3huqNuZyBob+G6t2MgcGjDsm5nIGJhbi4=",
        ));
    }
    let r_on = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_2o4(&r_on, &r_oo)?;
    Ok(())
}
#[tauri::command]
pub fn mang_delete(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tcy,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogDelete)?;
    let r_op = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_143(&r_op, args.mang_cvid)?;
    Ok(())
}
#[tauri::command]
pub fn thbq_upsert(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tnz,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogUpdate)?;
    let r_or = args.thbq;
    if r_or.ten_thbq.trim().is_empty() {
        return Err(AppError::validation(
            "VGhp4bq/dSB0w6puIHRo4budaSBo4bqhbiBi4bqjbyBxdeG6o24u",
        ));
    }
    let r_oq = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_2o8(&r_oq, &r_or)?;
    Ok(())
}
#[tauri::command]
pub fn thbq_delete(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: To0,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogDelete)?;
    let r_os = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_147(&r_os, args.thbqid)?;
    Ok(())
}
#[tauri::command]
pub fn phong_upsert(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Th5,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogUpdate)?;
    let r_ou = args.phong;
    if r_ou.ma_du_an.trim().is_empty() {
        return Err(AppError::validation(
            "VGhp4bq/dSBtw6MgcGjDtG5nIGzGsHUgdHLhu68u",
        ));
    }
    if r_ou.ten_du_an.trim().is_empty() {
        return Err(AppError::validation(
            "VGhp4bq/dSB0w6puIHBow7RuZyBsxrB1IHRy4buvLg==",
        ));
    }
    let r_ot = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    db::f_2o6(
        &r_ot,
        r_ou.ma_du_an.trim(),
        r_ou.ten_du_an.trim(),
        r_ou.ma_khoi.as_deref(),
        r_ou.phong_so.as_deref(),
        r_ou.muc_luc_so.as_deref(),
    )?;
    Ok(())
}
#[tauri::command]
pub fn phong_delete(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tco,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogDelete)?;
    let r_ov = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    db::f_145(&r_ov, &args.ma_du_an)?;
    Ok(())
}
#[tauri::command]
pub fn ngon_ngu_upsert(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Te2,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogUpdate)?;
    let r_ox = args.ngon_ngu;
    if r_ox.ma_ngon_ngu.trim().is_empty() || r_ox.ten_ngon_ngu.trim().is_empty() {
        return Err(AppError::validation(
            "VGhp4bq/dSBtw6MgaG/hurdjIHTDqm4gbmfDtG4gbmfhu68u",
        ));
    }
    let r_ow = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_2o5(&r_ow, &r_ox)?;
    Ok(())
}
#[tauri::command]
pub fn ngon_ngu_delete(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tct,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogDelete)?;
    let r_oy = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_144(&r_oy, &args.ma_ngon_ngu)?;
    Ok(())
}
#[tauri::command]
pub fn loai_van_ban_upsert(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tb5,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogUpdate)?;
    let r_p0 = args.loai_van_ban;
    if r_p0.ma_loai.trim().is_empty() || r_p0.ten_loai.trim().is_empty() {
        return Err(AppError::validation(
            "VGhp4bq/dSBtw6MgaG/hurdjIHTDqm4gbG/huqFpIHTDoGkgbGnhu4d1Lg==",
        ));
    }
    let r_oz = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_2o3(&r_oz, &r_p0)?;
    Ok(())
}
#[tauri::command]
pub fn loai_van_ban_delete(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tcs,
) -> Result<(), AppError> {
    auth.require_perm(Permission::CatalogDelete)?;
    let r_p1 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_142(&r_p1, &args.ma_loai)?;
    Ok(())
}
