// Copyright (c) 2026 DigiText. All rights reserved.
use super::store;
use super::types::*;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use tauri::State;
#[tauri::command]
pub fn report_so_hoa(auth: State<'_, AuthState>, db: State<'_, Db>) -> Result<Vec<Tlt>, AppError> {
    auth.require_perm(Permission::StatsViewProject)?;
    let r_2aq = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_2g2(&r_2aq)?)
}
#[tauri::command]
pub fn report_chinh_ly(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T2q,
) -> Result<Vec<T2r>, AppError> {
    auth.require_perm(Permission::StatsViewProject)?;
    let r_2ar = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_yq(&r_2ar, args.ma_da.as_deref())?)
}
#[tauri::command]
pub fn report_phan_mem(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Th3,
) -> Result<Vec<Th4>, AppError> {
    auth.require_perm(Permission::StatsViewProject)?;
    let r_2as = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_22b(
        &r_2as,
        args.tu_ngay.as_deref(),
        args.den_ngay.as_deref(),
    )?)
}
#[tauri::command]
pub fn report_nhan_vien(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
) -> Result<Vec<Te5>, AppError> {
    auth.require_perm(Permission::StatsViewProject)?;
    let r_2at = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_1ul(&r_2at)?)
}
