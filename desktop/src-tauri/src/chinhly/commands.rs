// Copyright (c) 2026 DigiText. All rights reserved.
//! Lệnh Tauri của chỉnh lý: đọc/ghi mirror (offline được), đồng bộ, xung đột, và proxy online `cl_remote`.

use super::store::{self, HoSoFilter, ListFilter, Row};
use super::sync::{self, SyncState};
use crate::auth::admin::f_2r8;
use crate::auth::{client, AuthState};
use crate::db::Db;
use crate::error::AppError;
use crate::live::{self, Topic};
use digitext_core::chinh_ly::spec::{self, WriteOp};
use digitext_core::Permission;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, State};

type J = Value;

fn project_of(db: &Db, auth: &AuthState) -> Result<(String, String), AppError> {
    super::project::resolve(db, auth)
}

fn require_read(auth: &AuthState) -> Result<(), AppError> {
    if auth.require_perm(Permission::DossierRead).is_ok() || auth.require_perm(Permission::CatalogRead).is_ok() {
        Ok(())
    } else {
        Err(AppError::new("forbidden", "Tài khoản của bạn không đủ quyền xem dữ liệu chỉnh lý."))
    }
}

fn table_of(s: &str) -> Result<&'static spec::TableSpec, AppError> {
    let t = s.trim().to_lowercase().replace('-', "_");
    let t = if t.starts_with("cl_") { t } else { format!("cl_{t}") };
    spec::spec(&t).ok_or_else(|| AppError::validation(format!("Không có bảng {s}")))
}

#[tauri::command]
pub fn cl_catalog_list(db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<J, AppError> {
    require_read(&auth)?;
    let (project, _) = project_of(&db, &auth)?;
    let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let mut out = serde_json::Map::new();
    for t in ["cl_khoi_phong", "cl_phong", "cl_phong_ban", "cl_mang", "cl_thbq", "cl_ngon_ngu", "cl_loai_van_ban", "cl_kho", "cl_gia", "cl_hop", "cl_bo_tap"] {
        let (rows, _) = store::list(&g, t, &project, &ListFilter::default())?;
        out.insert(t.trim_start_matches("cl_").to_owned(), Value::Array(rows.into_iter().map(Value::Object).collect()));
    }
    Ok(Value::Object(out))
}

#[tauri::command]
pub fn cl_rows(db: State<'_, Db>, auth: State<'_, AuthState>, table: String, filter: Option<ListFilter>) -> Result<J, AppError> {
    require_read(&auth)?;
    let (project, _) = project_of(&db, &auth)?;
    let sp_name = if table == "lich_su" || table == "cl_lich_su" { store::LICH_SU.to_owned() } else { table_of(&table)?.name.to_owned() };
    let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let (rows, total) = store::list(&g, &sp_name, &project, &filter.unwrap_or_default())?;
    Ok(json!({"rows": rows, "total": total}))
}

#[tauri::command]
pub fn cl_row(db: State<'_, Db>, auth: State<'_, AuthState>, table: String, id: String) -> Result<Option<store::LocalRow>, AppError> {
    require_read(&auth)?;
    let sp = table_of(&table)?;
    let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::get(&g, sp.name, &id)?)
}

#[tauri::command]
pub fn cl_ho_so_list(db: State<'_, Db>, auth: State<'_, AuthState>, filter: Option<HoSoFilter>) -> Result<J, AppError> {
    require_read(&auth)?;
    let (project, _) = project_of(&db, &auth)?;
    let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let (rows, total) = store::ho_so_list(&g, &project, &filter.unwrap_or_default())?;
    Ok(json!({"rows": rows, "total": total}))
}

#[tauri::command]
pub fn cl_van_ban_list(db: State<'_, Db>, auth: State<'_, AuthState>, ho_so_id: String) -> Result<Vec<Row>, AppError> {
    require_read(&auth)?;
    let (project, _) = project_of(&db, &auth)?;
    let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::van_ban_list(&g, &project, &ho_so_id)?)
}

#[tauri::command]
pub fn cl_next_stt(db: State<'_, Db>, auth: State<'_, AuthState>, ho_so_id: String) -> Result<i64, AppError> {
    require_read(&auth)?;
    let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::next_stt(&g, &ho_so_id)?)
}

/// Kết quả ghi cục bộ: dòng mới hoặc danh sách lỗi kiểm tra.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    pub ok: bool,
    pub id: String,
    pub row: Option<Row>,
    pub errors: Vec<store::LocalErr>,
}

/// Ghi / sửa một dòng tại máy (bảng sửa offline được), rồi xin đồng bộ.
#[tauri::command]
pub fn cl_save(app: AppHandle, db: State<'_, Db>, auth: State<'_, AuthState>, table: String, id: Option<String>, patch: Row) -> Result<SaveResult, AppError> {
    let sp = table_of(&table)?;
    if !sp.offline_write {
        return Err(AppError::new("online_only", "Bảng này chỉ sửa được khi có kết nối máy chủ."));
    }
    let (project, user) = project_of(&db, &auth)?;
    let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let id = id.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty()).unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
    let exists = store::get(&g, sp.name, &id)?.is_some();
    auth.require_perm(sp.perm_for(if exists { WriteOp::Update } else { WriteOp::Insert }))?;
    let mut patch = patch;
    if !exists {
        // bổ sung mặc định như máy chủ
        match sp.name {
            "cl_ho_so" => {
                patch.entry("tieu_de").or_insert(json!(""));
                patch.entry("trung_thua").or_insert(json!(false));
            }
            "cl_mang" => {
                patch.entry("ma_mang").or_insert(json!("00"));
            }
            _ => {}
        }
    }
    let mut check = if exists {
        let mut merged = store::get(&g, sp.name, &id)?.map(|e| e.row).unwrap_or_default();
        for (k, v) in &patch {
            merged.insert(k.clone(), v.clone());
        }
        merged
    } else {
        patch.clone()
    };
    let errors = store::validate_local(&g, sp.name, &id, &mut check);
    if !errors.is_empty() {
        return Ok(SaveResult { ok: false, id, row: None, errors });
    }
    // giữ giá trị đã chuẩn hoá (ngày) cho các cột client gửi
    for k in patch.keys().cloned().collect::<Vec<_>>() {
        if let Some(v) = check.get(&k) {
            patch.insert(k, v.clone());
        }
    }
    let row = store::write_local(&g, sp, &id, &patch, &store::Who { project_id: &project, user_id: &user }, None)?;
    super::legacy::project_row(&g, sp.name, &row)?;
    drop(g);
    live::touch(&app, &[Topic::ChinhLy, Topic::ClSync]);
    sync::request_soon(&app);
    Ok(SaveResult { ok: true, id, row: Some(row), errors: vec![] })
}

#[tauri::command]
pub fn cl_delete(app: AppHandle, db: State<'_, Db>, auth: State<'_, AuthState>, table: String, id: String) -> Result<u64, AppError> {
    let sp = table_of(&table)?;
    if !sp.offline_write {
        return Err(AppError::new("online_only", "Bảng này chỉ sửa được khi có kết nối máy chủ."));
    }
    auth.require_perm(sp.perm_for(WriteOp::Delete))?;
    let (project, user) = project_of(&db, &auth)?;
    let n = {
        let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        let n = store::delete_local(&g, sp, &id, &store::Who { project_id: &project, user_id: &user })?;
        super::legacy::prune_legacy(&g)?;
        n
    };
    live::touch(&app, &[Topic::ChinhLy, Topic::ClSync]);
    sync::request_soon(&app);
    Ok(n)
}

// ------------------------------------------------------------------ đồng bộ

#[tauri::command]
pub async fn cl_sync_now(app: AppHandle) -> Result<sync::Status, AppError> {
    sync::run_once(&app).await
}

#[tauri::command]
pub fn cl_sync_status(app: AppHandle, db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<sync::Status, AppError> {
    let st = app.state::<SyncState>();
    let mut s = st.status();
    if let Ok((project, _)) = project_of(&db, &auth) {
        if let Ok(g) = db.0.lock() {
            let (p, f, c) = store::pending_count(&g, &project).unwrap_or((0, 0, 0));
            s.pending = p;
            s.failed = f;
            s.conflicts = c;
            if s.last_sync_at.is_none() {
                s.last_sync_at = store::last_pull_at(&g, &project).ok().flatten();
            }
        }
    }
    Ok(s)
}

#[tauri::command]
pub fn cl_conflicts(db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<Vec<store::Conflict>, AppError> {
    require_read(&auth)?;
    let (project, _) = project_of(&db, &auth)?;
    let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::conflicts_open(&g, &project)?)
}

#[tauri::command]
pub fn cl_conflict_resolve(app: AppHandle, db: State<'_, Db>, auth: State<'_, AuthState>, id: i64, choice: String, merged: Option<Row>) -> Result<(), AppError> {
    auth.require_perm(Permission::DossierUpdate)?;
    let (_, user) = project_of(&db, &auth)?;
    {
        let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        sync::resolve_conflict(&g, id, &choice, merged, &user)?;
    }
    live::touch(&app, &[Topic::ChinhLy, Topic::ClSync]);
    sync::request_soon(&app);
    Ok(())
}

#[tauri::command]
pub fn cl_outbox(db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<Vec<store::OutboxRow>, AppError> {
    require_read(&auth)?;
    let (project, _) = project_of(&db, &auth)?;
    let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::outbox_all(&g, &project)?)
}

#[tauri::command]
pub fn cl_outbox_retry(app: AppHandle, db: State<'_, Db>, auth: State<'_, AuthState>, id: i64) -> Result<(), AppError> {
    auth.require_perm(Permission::DossierUpdate)?;
    {
        let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        store::outbox_retry(&g, id)?;
    }
    sync::request_soon(&app);
    Ok(())
}

#[tauri::command]
pub fn cl_outbox_discard(app: AppHandle, db: State<'_, Db>, auth: State<'_, AuthState>, id: i64) -> Result<(), AppError> {
    auth.require_perm(Permission::DossierUpdate)?;
    {
        let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        store::outbox_discard(&g, id)?;
    }
    live::touch(&app, &[Topic::ChinhLy, Topic::ClSync]);
    Ok(())
}

// ------------------------------------------------------------------ chuyển dữ liệu cũ trên máy

/// Số dòng bảng cũ (dossier, van_ban, danh mục…) chưa gắn với cl_* — xem trước wizard.
#[tauri::command]
pub fn cl_migrate_preview(db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<super::legacy::MigrateReport, AppError> {
    require_read(&auth)?;
    let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(super::legacy::count_unmigrated(&g)?)
}

/// Đẩy mọi dòng cũ chưa gắn cl_id sang bản sao cl_* (rồi outbox → máy chủ); chạy lại an toàn.
#[tauri::command]
pub fn cl_migrate_run(app: AppHandle, db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<super::legacy::MigrateReport, AppError> {
    auth.require_perm(Permission::DossierCreate)?;
    let (project, user) = project_of(&db, &auth)?;
    let rep = {
        let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        let rep = super::legacy::push_unmigrated(&g, &store::Who { project_id: &project, user_id: &user })?;
        super::legacy::project_all(&g, &project)?;
        rep
    };
    live::touch(&app, &[Topic::ChinhLy, Topic::ClSync, Topic::Dossier, Topic::Catalog]);
    sync::request_soon(&app);
    Ok(rep)
}

// ------------------------------------------------------------------ proxy online

/// Gọi thẳng một route `/cl/*` trên máy chủ (thao tác chỉ online: hệ thống hoá, kho/giá, mượn trả,
/// thống kê, nhập theo dòng). Máy chủ kiểm quyền cụ thể; ở đây chỉ cần quyền xem.
#[tauri::command]
pub async fn cl_remote(app: AppHandle, db: State<'_, Db>, auth: State<'_, AuthState>, method: String, path: String, body: Option<J>) -> Result<J, AppError> {
    require_read(&auth)?;
    if !path.starts_with("/cl/") || path.contains("..") {
        return Err(AppError::validation("đường dẫn không hợp lệ"));
    }
    let m = method.to_uppercase();
    let body = body.unwrap_or(json!({}));
    let out = f_2r8(&db, &auth, Permission::DossierRead, move |base, token| {
        let (m, path, body) = (m.clone(), path.clone(), body.clone());
        async move {
            match m.as_str() {
                "GET" => client::f_1eo::<J>(&base, &token, &path).await,
                "PUT" => client::f_24x::<J>(&base, &token, &path, &body).await,
                "PATCH" => client::f_20e::<J>(&base, &token, &path, &body).await,
                "DELETE" => client::f_13z::<J>(&base, &token, &path).await,
                _ => client::f_23d::<J>(&base, &token, &path, &body).await,
            }
        }
    })
    .await?;
    if method.to_uppercase() != "GET" {
        sync::request_soon(&app);
    }
    Ok(out)
}

// ---- dự án đang làm (tài khoản quản trị không gắn dự án)

#[tauri::command]
pub async fn cl_project_get(db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<super::project::ProjectInfo, AppError> {
    super::project::info(&db, &auth).await
}

#[tauri::command]
pub fn cl_project_set(app: AppHandle, db: State<'_, Db>, auth: State<'_, AuthState>, project_id: String) -> Result<(), AppError> {
    super::project::set(&db, &auth, &project_id)?;
    live::touch(&app, &[Topic::ChinhLy, Topic::ClSync, Topic::Dossier, Topic::Catalog]);
    sync::request_soon(&app);
    Ok(())
}
