// Copyright (c) 2026 DigiText. All rights reserved.
//! Engine đồng bộ chỉnh lý: đẩy outbox lên `/cl/sync/push` (máy chủ gộp 3 chiều), kéo thay đổi từ
//! `/cl/sync?since=<ver>`, ghi xung đột cho người dùng chọn. Chạy nền: khi phiên mở, mỗi 60 s, và
//! ngay sau mỗi lần ghi cục bộ (debounce 1,5 s).

use super::store::{self, Row};
use crate::auth::admin::f_2r8;
use crate::auth::{client, AuthState};
use crate::db::Db;
use crate::error::AppError;
use crate::live::{self, Topic};
use digitext_core::chinh_ly::spec;
use digitext_core::Permission;
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use tauri::{AppHandle, Emitter, Manager};

pub const EVENT: &str = "chinhly://sync";
const PUSH_BATCH: i64 = 200;
const PULL_LIMIT: i64 = 2000;
const INTERVAL_SECS: u64 = 60;
const DEBOUNCE_MS: u64 = 1500;

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// idle | pushing | pulling | offline | locked | error | no_project
    pub phase: String,
    pub pending: i64,
    pub failed: i64,
    pub conflicts: i64,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
    pub pushed: i64,
    pub pulled: i64,
}

#[derive(Default)]
pub struct SyncState {
    notify: tokio::sync::Notify,
    running: tokio::sync::Mutex<()>,
    status: RwLock<Status>,
    requested: AtomicBool,
}

impl SyncState {
    pub fn status(&self) -> Status {
        self.status.read().map(|s| s.clone()).unwrap_or_default()
    }
    fn set(&self, app: &AppHandle, s: Status) {
        if let Ok(mut g) = self.status.write() {
            *g = s.clone();
        }
        let _ = app.emit(EVENT, &s);
    }
}

/// Yêu cầu đồng bộ sớm (sau ghi cục bộ).
pub fn request_soon(app: &AppHandle) {
    if let Some(st) = app.try_state::<SyncState>() {
        st.requested.store(true, Ordering::SeqCst);
        st.notify.notify_one();
    }
}

pub fn spawn_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let st = app.state::<SyncState>();
            if tokio::time::timeout(std::time::Duration::from_secs(INTERVAL_SECS), st.notify.notified()).await.is_ok() {
                tokio::time::sleep(std::time::Duration::from_millis(DEBOUNCE_MS)).await;
            }
            st.requested.store(false, Ordering::SeqCst);
            if let Err(e) = run_once(&app).await {
                tracing::warn!(error = %e, "đồng bộ chỉnh lý lỗi");
            }
        }
    });
}

fn counts(app: &AppHandle, project: &str) -> (i64, i64, i64) {
    let db = app.state::<Db>();
    let g = match db.0.lock() {
        Ok(g) => g,
        Err(_) => return (0, 0, 0),
    };
    store::pending_count(&g, project).unwrap_or((0, 0, 0))
}

fn base_status(app: &AppHandle, project: &str, phase: &str) -> Status {
    let (pending, failed, conflicts) = counts(app, project);
    let last = {
        let db = app.state::<Db>();
        db.0.lock().ok().and_then(|g| store::last_pull_at(&g, project).ok().flatten())
    };
    Status { phase: phase.into(), pending, failed, conflicts, last_sync_at: last, ..Default::default() }
}

/// Một vòng: đẩy rồi kéo. Không chạy chồng.
pub async fn run_once(app: &AppHandle) -> Result<Status, AppError> {
    let st = app.state::<SyncState>();
    let _guard = match st.running.try_lock() {
        Ok(g) => g,
        Err(_) => return Ok(st.status()),
    };
    let auth = app.state::<AuthState>();
    let db = app.state::<Db>();
    let Some(session) = auth.snapshot() else {
        let s = Status { phase: "locked".into(), ..Default::default() };
        st.set(app, s.clone());
        return Ok(s);
    };
    let Some(project) = session.project_id.clone().filter(|p| !p.trim().is_empty()) else {
        let s = Status { phase: "no_project".into(), ..Default::default() };
        st.set(app, s.clone());
        return Ok(s);
    };
    if session.locked {
        let s = base_status(app, &project, "locked");
        st.set(app, s.clone());
        return Ok(s);
    }
    let url = match crate::auth::commands::server_url(&db) {
        Ok(u) => u,
        Err(e) => {
            let mut s = base_status(app, &project, "error");
            s.last_error = Some(e.to_string());
            st.set(app, s.clone());
            return Ok(s);
        }
    };
    if !client::f_22y(&url).await {
        let s = base_status(app, &project, "offline");
        st.set(app, s.clone());
        return Ok(s);
    }
    let mut pushed = 0i64;
    let mut pulled = 0i64;
    // ---- push
    st.set(app, base_status(app, &project, "pushing"));
    loop {
        let ops = {
            let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
            store::outbox_pending(&g, &project, PUSH_BATCH)?
        };
        if ops.is_empty() {
            break;
        }
        let body = json!({
            "projectId": project,
            "ops": ops.iter().map(|o| json!({
                "opId": o.op_id, "table": o.tbl, "id": o.row_id, "op": o.op,
                "base": o.base, "new": o.new, "editedAt": o.edited_at,
            })).collect::<Vec<_>>(),
        });
        let resp: Value = match f_2r8(&db, &auth, Permission::DossierRead, move |base, token| {
            let body = body.clone();
            async move { client::f_23d::<Value>(&base, &token, "/cl/sync/push", &body).await }
        })
        .await
        {
            Ok(v) => v,
            Err(e) => {
                let mut s = base_status(app, &project, "error");
                s.last_error = Some(e.to_string());
                let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
                let _ = store::state_error(&g, &project, Some(&e.to_string()));
                drop(g);
                st.set(app, s.clone());
                return Ok(s);
            }
        };
        let results = resp.get("results").and_then(Value::as_array).cloned().unwrap_or_default();
        let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        let by_id: std::collections::HashMap<String, &store::OutboxRow> = ops.iter().map(|o| (o.op_id.clone(), o)).collect();
        let mut touched: Vec<(String, Row)> = Vec::new();
        for r in results {
            let Some(op_id) = r.get("opId").and_then(Value::as_str) else { continue };
            let Some(o) = by_id.get(op_id) else { continue };
            apply_push_result(&g, &project, o, &r)?;
            if let Some(e) = store::get(&g, &o.tbl, &o.row_id)? {
                touched.push((o.tbl.clone(), e.row));
            }
            pushed += 1;
        }
        super::legacy::project_rows(&g, &touched)?;
        super::legacy::prune_legacy(&g)?;
        drop(g);
        if ops.len() < PUSH_BATCH as usize {
            break;
        }
    }
    // ---- pull
    st.set(app, base_status(app, &project, "pulling"));
    let mut cursor = {
        let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        store::cursor_get(&g, &project)?
    };
    let mut wiped = false;
    loop {
        let path = format!("/cl/sync?projectId={project}&since={cursor}&limit={PULL_LIMIT}");
        let page: Value = match f_2r8(&db, &auth, Permission::DossierRead, move |base, token| {
            let path = path.clone();
            async move { client::f_1eo::<Value>(&base, &token, &path).await }
        })
        .await
        {
            Ok(v) => v,
            Err(e) => {
                let mut s = base_status(app, &project, "error");
                s.last_error = Some(e.to_string());
                st.set(app, s.clone());
                return Ok(s);
            }
        };
        if page.get("full").and_then(Value::as_bool).unwrap_or(false) && !wiped {
            let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
            store::wipe_project(&g, &project)?;
            wiped = true;
            cursor = 0;
            continue;
        }
        let rows = page.get("rows").and_then(Value::as_array).cloned().unwrap_or_default();
        let next = page.get("next").and_then(Value::as_i64).unwrap_or(cursor);
        let has_more = page.get("hasMore").and_then(Value::as_bool).unwrap_or(false);
        {
            let g = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
            let tx = g.unchecked_transaction()?;
            let mut touched: Vec<(String, Row)> = Vec::new();
            for r in &rows {
                let (Some(table), Some(row)) = (r.get("table").and_then(Value::as_str), r.get("row").and_then(Value::as_object)) else { continue };
                if store::upsert_from_server(&tx, table, row, false)? {
                    // chiếu bản đang có trên máy (có thể là bản dirty cục bộ)
                    if let Some(e) = store::get(&tx, table, row.get("id").and_then(Value::as_str).unwrap_or(""))? {
                        touched.push((table.to_owned(), e.row));
                    }
                }
                pulled += 1;
            }
            super::legacy::project_rows(&tx, &touched)?;
            super::legacy::prune_legacy(&tx)?;
            store::cursor_set(&tx, &project, next)?;
            store::state_error(&tx, &project, None)?;
            tx.commit()?;
        }
        cursor = next;
        if !has_more {
            break;
        }
    }
    let mut s = base_status(app, &project, "idle");
    s.pushed = pushed;
    s.pulled = pulled;
    st.set(app, s.clone());
    if pushed > 0 || pulled > 0 {
        live::touch(app, &[Topic::ChinhLy, Topic::ClSync, Topic::Catalog, Topic::Khogia]);
    } else {
        live::touch(app, &[Topic::ClSync]);
    }
    Ok(s)
}

/// Áp kết quả một op: applied/merged/no_change → ghi dòng máy chủ; conflict/missing → ghi xung đột;
/// rejected → giữ lại kèm lỗi.
fn apply_push_result(conn: &rusqlite::Connection, project: &str, o: &store::OutboxRow, r: &Value) -> Result<(), AppError> {
    let status = r.get("status").and_then(Value::as_str).unwrap_or("");
    let row_of = |v: &Value| v.get("row").and_then(Value::as_object).cloned();
    match status {
        "applied" | "merged" | "no_change" => {
            match row_of(r) {
                Some(row) => {
                    store::upsert_from_server(conn, &o.tbl, &row, true)?;
                }
                None => {
                    // xoá dòng không còn trên máy chủ
                    conn.execute(&format!("DELETE FROM {} WHERE id = ?1", o.tbl), rusqlite::params![o.row_id])?;
                }
            }
            store::outbox_done(conn, &o.op_id)?;
        }
        "duplicate" => {
            let inner = r.get("result").cloned().unwrap_or(Value::Null);
            if let Some(row) = row_of(&inner) {
                store::upsert_from_server(conn, &o.tbl, &row, true)?;
            } else {
                conn.execute(&format!("UPDATE {} SET dirty = 0, base_json = NULL WHERE id = ?1", o.tbl), rusqlite::params![o.row_id])?;
            }
            store::outbox_done(conn, &o.op_id)?;
        }
        "conflict" => {
            let theirs = row_of(r);
            let fields: Vec<String> = r.get("conflictFields").and_then(Value::as_array).map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect()).unwrap_or_default();
            store::conflict_put(conn, project, &o.tbl, &o.row_id, o.base.as_ref(), &o.new, theirs.as_ref(), &fields, Some("Người khác đã sửa cùng trường"))?;
            if let Some(t) = theirs {
                store::upsert_from_server(conn, &o.tbl, &t, true)?;
            }
            store::outbox_done(conn, &o.op_id)?;
        }
        "rejected" => {
            let reason = r.get("reason").and_then(Value::as_str).unwrap_or("");
            let msg = r.get("message").and_then(Value::as_str).unwrap_or("Máy chủ từ chối").to_owned();
            if reason == "missing" || reason == "deleted_on_server" {
                store::conflict_put(conn, project, &o.tbl, &o.row_id, o.base.as_ref(), &o.new, None, &[], Some(&msg))?;
                conn.execute(&format!("UPDATE {} SET dirty = 0, base_json = NULL, deleted_at = COALESCE(deleted_at, datetime('now')) WHERE id = ?1", o.tbl), rusqlite::params![o.row_id])?;
                store::outbox_done(conn, &o.op_id)?;
            } else {
                store::outbox_fail(conn, &o.op_id, "rejected", &msg)?;
            }
        }
        _ => {
            store::outbox_fail(conn, &o.op_id, "rejected", &format!("Kết quả lạ: {status}"))?;
        }
    }
    Ok(())
}

/// Ghi lại quyết định xung đột: `mine` → ghi lại bản của tôi (base = bản máy chủ) ; `theirs` → chỉ đóng;
/// `merged` → ghi bản người dùng ghép.
pub fn resolve_conflict(conn: &rusqlite::Connection, id: i64, choice: &str, merged: Option<Row>, user_id: &str) -> Result<(), AppError> {
    let Some((project, c)) = store::conflict_get(conn, id)? else {
        return Err(AppError::not_found("Không thấy xung đột"));
    };
    let sp = spec::spec(&c.tbl).ok_or_else(|| AppError::validation("bảng lạ"))?;
    let who = store::Who { project_id: &project, user_id };
    match choice {
        "theirs" => {}
        "mine" | "merged" => {
            let patch = if choice == "mine" { c.mine.clone() } else { merged.unwrap_or_else(|| c.mine.clone()) };
            if c.theirs.as_ref().map_or(true, |t| !t.get("deleted_at").map_or(true, Value::is_null)) {
                // máy chủ đã xoá: ghi lại = tạo lại với id mới
                let new_id = uuid::Uuid::now_v7().to_string();
                let mut p = patch.clone();
                p.remove("id");
                conn.execute(&format!("DELETE FROM {} WHERE id = ?1", c.tbl), rusqlite::params![c.row_id])?;
                store::write_local(conn, sp, &new_id, &p, &who, None)?;
            } else {
                store::write_local(conn, sp, &c.row_id, &patch, &who, None)?;
            }
        }
        _ => return Err(AppError::validation("lựa chọn phải là mine | theirs | merged")),
    }
    store::conflict_close(conn, id, choice)?;
    Ok(())
}
