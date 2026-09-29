// Copyright (c) 2026 DigiText. All rights reserved.
use super::history::{self, Who};
use super::store;
use super::types::*;
use crate::auth::AuthState;
use crate::db::{self, Db};
use crate::error::AppError;
use digitext_core::{Perm, Permission};
use tauri::State;
const C_EG: &str = "ocr_url";
const C_EF: &str = "http://113.190.252.175:5002/login";

/// Cầu nối sang bản sao cl_* (đồng bộ máy chủ): chỉ khi phiên gắn dự án.
fn cl_who(conn: &rusqlite::Connection, auth: &AuthState) -> Option<(String, String)> {
    crate::chinhly::project::effective_with(conn, auth)
}
fn cl_forward_dossier(conn: &rusqlite::Connection, auth: &AuthState, id: i64) {
    if let Some((p, u)) = cl_who(conn, auth) {
        if let Err(e) = crate::chinhly::legacy::forward_dossier(conn, id, &crate::chinhly::store::Who { project_id: &p, user_id: &u }) {
            tracing::warn!(error = %e, id, "không đẩy được hồ sơ cũ sang cl_ho_so");
        }
    }
}
fn cl_forward_van_ban(conn: &rusqlite::Connection, auth: &AuthState, id: i64) {
    if let Some((p, u)) = cl_who(conn, auth) {
        if let Err(e) = crate::chinhly::legacy::forward_van_ban(conn, id, &crate::chinhly::store::Who { project_id: &p, user_id: &u }) {
            tracing::warn!(error = %e, id, "không đẩy được văn bản cũ sang cl_van_ban");
        }
    }
}
fn cl_after_bulk(conn: &rusqlite::Connection, auth: &AuthState) {
    if let Some((p, u)) = cl_who(conn, auth) {
        let who = crate::chinhly::store::Who { project_id: &p, user_id: &u };
        if let Err(e) = crate::chinhly::legacy::push_unmigrated(conn, &who) {
            tracing::warn!(error = %e, "không đẩy được dữ liệu cũ sang cl_*");
        }
        if let Err(e) = crate::chinhly::legacy::reconcile_deleted(conn, &who) {
            tracing::warn!(error = %e, "không đồng bộ được xoá sang cl_*");
        }
    }
}
#[tauri::command]
pub fn dossier_list(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tay,
) -> Result<Vec<T55>, AppError> {
    auth.require_perm(Permission::DossierRead)?;
    let r_1mn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_1nc(&r_1mn, &args.filter.unwrap_or_default())?)
}
#[tauri::command]
pub fn dossier_get(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T53,
) -> Result<T55, AppError> {
    auth.require_perm(Permission::DossierRead)?;
    let r_1mo = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_1ek(&r_1mo, args.id)?
        .ok_or_else(|| AppError::not_found("Không tìm thấy hồ sơ."))
}
#[tauri::command]
pub fn dossier_create(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T38,
) -> Result<i64, AppError> {
    let r_1mr = Who::from(&auth.require_perm(Permission::DossierCreate)?);
    let r_1mp = args.dossier;
    if r_1mp.so_tam.trim().is_empty() {
        return Err(AppError::validation(
            "Thiếu số tạm của hồ sơ.",
        ));
    }
    let r_1mq = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    if store::f_2gf(&r_1mq, r_1mp.ma_da.as_deref(), r_1mp.so_tam.trim(), None)? {
        return Err(AppError::validation(format!(
            "Số tạm \"{}\" đã tồn tại trong phông này.",
            r_1mp.so_tam.trim()
        )));
    }
    let id = store::f_128(&r_1mq, &r_1mp)?;
    store::f_2h3(&r_1mq, id, Some(&r_1mr.user_id), None)?;
    history::record(
        &r_1mq,
        "hoso",
        id,
        Some(id),
        r_1mp.ma_da.as_deref(),
        "create",
        &format!(
            "Thêm hồ sơ số tạm {}",
            r_1mp.so_tam.trim()
        ),
        &r_1mp.tieu_de,
        &r_1mr,
    )?;
    tracing::info!(
        id, so_tam = % r_1mp.so_tam, nguoi = % r_1mr.username, "dossier_create"
    );
    cl_forward_dossier(&r_1mq, &auth, id);
    Ok(id)
}
#[tauri::command]
pub fn dossier_sync_from_kho(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tmd,
) -> Result<Tmf, AppError> {
    let r_1mv = Who::from(&auth.require_perm(Permission::DossierCreate)?);
    let mut r_1ms = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_1mu = r_1ms.transaction()?;
    let r_1mt = store::f_2if(&r_1mu, args.ma_da.as_deref(), &r_1mv.user_id)?;
    if r_1mt.created > 0 {
        history::record(
            &r_1mu,
            "hoso",
            0,
            None,
            args.ma_da.as_deref(),
            "create",
            &format!(
                "Đồng bộ {} hồ sơ từ Kho tài liệu sang mục lục",
                r_1mt.created
            ),
            "",
            &r_1mv,
        )?;
    }
    r_1mu.commit()?;
    tracing::info!(
        created = r_1mt.created, nguoi = % r_1mv.username, "dossier_sync_from_kho"
    );
    cl_after_bulk(&r_1ms, &auth);
    Ok(r_1mt)
}
#[tauri::command]
pub fn dossier_update(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tp5,
) -> Result<(), AppError> {
    let r_1my = Who::from(&auth.require_perm(Permission::DossierUpdate)?);
    let Tp5 { id, dossier: r_1mw } = args;
    if r_1mw.so_tam.trim().is_empty() {
        return Err(AppError::validation(
            "Thiếu số tạm của hồ sơ.",
        ));
    }
    let r_1mx = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    if store::f_2gf(
        &r_1mx,
        r_1mw.ma_da.as_deref(),
        r_1mw.so_tam.trim(),
        Some(id),
    )? {
        return Err(AppError::validation(format!(
            "Số tạm \"{}\" đã tồn tại trong phông này.",
            r_1mw.so_tam.trim()
        )));
    }
    if !store::f_2ne(&r_1mx, id, &r_1mw)? {
        return Err(AppError::not_found(
            "Không tìm thấy hồ sơ cần sửa.",
        ));
    }
    store::f_2h3(&r_1mx, id, None, Some(&r_1my.user_id))?;
    history::record(
        &r_1mx,
        "hoso",
        id,
        Some(id),
        r_1mw.ma_da.as_deref(),
        "update",
        &format!(
            "Sửa hồ sơ số tạm {}",
            r_1mw.so_tam.trim()
        ),
        &r_1mw.tieu_de,
        &r_1my,
    )?;
    cl_forward_dossier(&r_1mx, &auth, id);
    Ok(())
}
#[tauri::command]
pub fn dossier_delete(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T53,
) -> Result<(), AppError> {
    let r_1n2 = Who::from(&auth.require_perm(Permission::DossierDelete)?);
    let r_1mz = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_1n1 = store::f_1ek(&r_1mz, args.id)?;
    if !store::f_13v(&r_1mz, args.id)? {
        return Err(AppError::not_found(
            "Không tìm thấy hồ sơ cần xoá.",
        ));
    }
    if let Some(r_1n0) = r_1n1 {
        history::record(
            &r_1mz,
            "hoso",
            args.id,
            Some(args.id),
            r_1n0.ma_da.as_deref(),
            "delete",
            &format!("Xoá hồ sơ số tạm {}", r_1n0.so_tam),
            &r_1n0.tieu_de,
            &r_1n2,
        )?;
    }
    cl_after_bulk(&r_1mz, &auth);
    Ok(())
}
#[tauri::command]
pub fn van_ban_list(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tb2,
) -> Result<Vec<Tpq>, AppError> {
    auth.require_perm(Permission::DossierRead)?;
    let r_1n3 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_1op(&r_1n3, args.dossier_id)?)
}
#[tauri::command]
pub fn van_ban_create(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T3j,
) -> Result<i64, AppError> {
    let r_1n7 = Who::from(&auth.require_perm(Permission::DossierUpdate)?);
    let T3j {
        dossier_id,
        van_ban: r_1n6,
    } = args;
    let r_1n4 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    if !store::f_16x(&r_1n4, dossier_id)? {
        return Err(AppError::not_found(
            "Hồ sơ chứa văn bản không tồn tại.",
        ));
    }
    let id = store::f_12h(&r_1n4, dossier_id, &r_1n6)?;
    store::f_2h8(&r_1n4, id, &r_1n7.user_id)?;
    let r_1n5 = store::f_16y(&r_1n4, dossier_id)?;
    history::record(
        &r_1n4,
        "vanban",
        id,
        Some(dossier_id),
        r_1n5.as_deref(),
        "create",
        &format!("Thêm văn bản vào hồ sơ #{dossier_id}"),
        r_1n6.trich_yeu.as_deref().unwrap_or(""),
        &r_1n7,
    )?;
    cl_forward_van_ban(&r_1n4, &auth, id);
    Ok(id)
}
#[tauri::command]
pub fn van_ban_update(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tp7,
) -> Result<(), AppError> {
    let r_1nc = Who::from(&auth.require_perm(Permission::DossierUpdate)?);
    let Tp7 { id, van_ban: r_1nb } = args;
    let r_1n9 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let dossier_id = store::f_2ph(&r_1n9, id)?;
    if !store::f_2ni(&r_1n9, id, &r_1nb)? {
        return Err(AppError::not_found(
            "Không tìm thấy văn bản cần sửa.",
        ));
    }
    let r_1na = dossier_id.and_then(|r_1n8| store::f_16y(&r_1n9, r_1n8).ok().flatten());
    history::record(
        &r_1n9,
        "vanban",
        id,
        dossier_id,
        r_1na.as_deref(),
        "update",
        "Sửa văn bản",
        r_1nb.trich_yeu.as_deref().unwrap_or(""),
        &r_1nc,
    )?;
    cl_forward_van_ban(&r_1n9, &auth, id);
    Ok(())
}
#[tauri::command]
pub fn van_ban_delete(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tpo,
) -> Result<(), AppError> {
    let r_1ng = Who::from(&auth.require_perm(Permission::DossierUpdate)?);
    let r_1ne = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let dossier_id = store::f_2ph(&r_1ne, args.id)?;
    if !store::f_149(&r_1ne, args.id)? {
        return Err(AppError::not_found(
            "Không tìm thấy văn bản cần xoá.",
        ));
    }
    let r_1nf = dossier_id.and_then(|r_1nd| store::f_16y(&r_1ne, r_1nd).ok().flatten());
    history::record(
        &r_1ne,
        "vanban",
        args.id,
        dossier_id,
        r_1nf.as_deref(),
        "delete",
        "Xoá văn bản",
        "",
        &r_1ng,
    )?;
    cl_after_bulk(&r_1ne, &auth);
    Ok(())
}
#[tauri::command]
pub fn dossier_systematize(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tmg,
) -> Result<usize, AppError> {
    auth.require(Perm::Admin)?;
    let r_1ni = args.scheme.as_deref().unwrap_or("thbqAsc");
    let mut r_1nh = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_1nj = r_1nh.transaction()?;
    let n = store::f_2ij(&r_1nj, args.ma_da.as_deref(), r_1ni, args.per_box.max(0))?;
    r_1nj.commit()?;
    tracing::info!(
        r_1ni,
        per_box = args.per_box,
        count = n,
        "dossier_systematize"
    );
    Ok(n)
}
#[tauri::command]
pub fn dossier_set_boxing(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tlc,
) -> Result<(), AppError> {
    auth.require(Perm::Admin)?;
    let r_1nk = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    if !store::f_2e0(
        &r_1nk,
        args.id,
        args.hop_so.as_deref(),
        args.bo_so.as_deref(),
        args.tap_so.as_deref(),
    )? {
        return Err(AppError::not_found(
            "Không tìm thấy hồ sơ.",
        ));
    }
    Ok(())
}
#[tauri::command]
pub fn dossier_bulk_upsert(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T1h,
) -> Result<usize, AppError> {
    let r_1nn = Who::from(&auth.require_perm(Permission::DossierUpdate)?);
    let mut r_1nl = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_1nm = r_1nl.transaction()?;
    let n = store::f_wd(&r_1nm, args.ma_da.as_deref(), &args.rows)?;
    history::record(
        &r_1nm,
        "hoso",
        0,
        None,
        args.ma_da.as_deref(),
        "update",
        &format!("Cập nhật {n} hồ sơ từ Excel"),
        "",
        &r_1nn,
    )?;
    r_1nm.commit()?;
    tracing::info!(count = n, nguoi = % r_1nn.username, "dossier_bulk_upsert");
    // Excel có thể sửa dòng đã có cl_id → đẩy lại toàn bộ hồ sơ của phông (write_local bỏ qua dòng không đổi).
    if let Some((p, u)) = cl_who(&r_1nl, &auth) {
        let who = crate::chinhly::store::Who { project_id: &p, user_id: &u };
        let ids: Vec<i64> = {
            let mut st = r_1nl.prepare("SELECT id FROM dossier WHERE (?1 IS NULL OR ma_da = ?1)")?;
            let v = st.query_map(rusqlite::params![args.ma_da.as_deref()], |r| r.get(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
            v
        };
        for id in ids {
            let _ = crate::chinhly::legacy::forward_dossier(&r_1nl, id, &who);
        }
    }
    Ok(n)
}
#[tauri::command]
pub fn van_ban_bulk_insert(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T1i,
) -> Result<usize, AppError> {
    let r_1ns = Who::from(&auth.require_perm(Permission::DossierUpdate)?);
    let r_1nq: Vec<(i64, VanBanIn)> = args
        .items
        .into_iter()
        .map(|r_1np| (r_1np.dossier_id, r_1np.van_ban))
        .collect();
    let mut r_1no = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_1nr = r_1no.transaction()?;
    let n = store::f_wa(&r_1nr, &r_1nq)?;
    history::record(
        &r_1nr,
        "vanban",
        0,
        None,
        None,
        "create",
        &format!("Nhập {n} văn bản từ Excel"),
        "",
        &r_1ns,
    )?;
    r_1nr.commit()?;
    tracing::info!(count = n, nguoi = % r_1ns.username, "van_ban_bulk_insert");
    cl_after_bulk(&r_1no, &auth);
    Ok(n)
}
fn nonempty(r_1nt: &Option<String>) -> Option<&str> {
    r_1nt.as_deref().map(str::trim).filter(|s| !s.is_empty())
}
#[tauri::command]
pub fn dossier_history(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T91,
) -> Result<Vec<history::T92>, AppError> {
    auth.require_perm(Permission::DossierRead)?;
    let r_1nv = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_1nu = history::HistoryFilter {
        entity: nonempty(&args.entity),
        ma_da: nonempty(&args.ma_da),
        tu_ngay: nonempty(&args.tu_ngay),
        den_ngay: nonempty(&args.den_ngay),
        search: nonempty(&args.search),
    };
    Ok(history::list(&r_1nv, &r_1nu)?)
}
#[tauri::command]
pub fn dossier_incomplete_van_ban(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T9q,
) -> Result<Vec<T9r>, AppError> {
    auth.require_perm(Permission::DossierRead)?;
    let r_1nw = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_1no(&r_1nw, &args.filter.unwrap_or_default())?)
}
#[tauri::command]
pub fn dossier_delete_incomplete_van_ban(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T9q,
) -> Result<usize, AppError> {
    let r_1o0 = Who::from(&auth.require_perm(Permission::DossierUpdate)?);
    let r_1nx = args.filter.unwrap_or_default();
    let mut r_1ny = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_1nz = r_1ny.transaction()?;
    let n = store::f_13y(&r_1nz, &r_1nx)?;
    history::record(
        &r_1nz,
        "vanban",
        0,
        None,
        r_1nx.ma_da_opt(),
        "delete",
        &format!("Hủy {n} văn bản chưa hoàn thiện"),
        "",
        &r_1o0,
    )?;
    r_1nz.commit()?;
    tracing::info!(
        count = n, nguoi = % r_1o0.username, "dossier_delete_incomplete_van_ban"
    );
    cl_after_bulk(&r_1ny, &auth);
    Ok(n)
}
#[tauri::command]
pub fn dossier_scan_files(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tkm,
) -> Result<Vec<Tkl>, AppError> {
    auth.require_perm(Permission::DossierRead)?;
    let r_1o3 = args
        .min_dpi
        .filter(|r_1o1| *r_1o1 > 0)
        .unwrap_or(store::C_CC);
    let r_1o2 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_2c2(&r_1o2, &args.ids, r_1o3)?)
}
#[tauri::command]
pub fn dossier_scan_pdf_files(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: T53,
) -> Result<Vec<Tkp>, AppError> {
    auth.require_perm(Permission::DossierRead)?;
    let (dir, van_ban) = {
        let r_1o4 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        let dir = store::f_1rn(&r_1o4, args.id)?;
        let van_ban = store::f_1op(&r_1o4, args.id)?;
        (dir, van_ban)
    };
    let Some(dir) = dir else {
        return Ok(Vec::new());
    };
    Ok(store::f_1oa(std::path::Path::new(&dir), &van_ban))
}
#[tauri::command]
pub fn get_ocr_url(auth: State<'_, AuthState>, db: State<'_, Db>) -> Result<String, AppError> {
    auth.require(Perm::Read)?;
    let r_1o5 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(db::f_1ew(&r_1o5, C_EG)?
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| C_EF.to_string()))
}
#[tauri::command]
pub fn set_ocr_url(
    auth: State<'_, AuthState>,
    db: State<'_, Db>,
    args: Tle,
) -> Result<(), AppError> {
    auth.require(Perm::Admin)?;
    let r_1o7 = args.url.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let r_1o6 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    db::f_2ei(&r_1o6, C_EG, r_1o7)?;
    Ok(())
}
fn f_1ji(r_1o9: &str) -> bool {
    if !(r_1o9.starts_with("http://") || r_1o9.starts_with("https://")) {
        return false;
    }
    !r_1o9.chars().any(|r_1o8| {
        r_1o8.is_control()
            || r_1o8.is_whitespace()
            || matches!(r_1o8, '"' | '\'' | '`' | '<' | '>' | '^' | '|' | '\\')
    })
}
#[tauri::command]
pub fn open_url(auth: State<'_, AuthState>, args: Tfk) -> Result<(), AppError> {
    auth.require(Perm::Read)?;
    let r_1oa = args.url.trim();
    if !f_1ji(r_1oa) {
        return Err(
            AppError::validation(
                "Liên kết không hợp lệ (chỉ http(s), không ký tự đặc biệt).",
            ),
        );
    }
    f_1w8(r_1oa).map_err(|e| {
        AppError::internal(format!(
            "Không mở được liên kết: {e}"
        ))
    })
}
fn f_1w8(r_1ob: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(r_1ob).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg(r_1ob)
            .spawn()?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(r_1ob).spawn()?;
    }
    Ok(())
}
