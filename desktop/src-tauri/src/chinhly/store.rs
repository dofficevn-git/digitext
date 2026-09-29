// Copyright (c) 2026 DigiText. All rights reserved.
//! Bản sao SQLite của dữ liệu chỉnh lý (cl_*). Mỗi dòng lưu nguyên JSON của máy chủ (`row_json`,
//! snake_case đúng tên cột) + cột đồng bộ (`ver`, `dirty`, `base_json`, `deleted_at`); các cột hay
//! lọc là cột SINH (`json_extract`) có chỉ mục. Ghi cục bộ đi qua một cửa `write_local` → outbox.

use digitext_core::chinh_ly::spec::{self, TableSpec, SYSTEM_COLS};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

pub type Row = Map<String, Value>;

/// Bảng mirror = 15 bảng spec + lịch sử.
pub const LICH_SU: &str = "cl_lich_su";

/// Cột được sinh từ JSON để lọc / join / chỉ mục.
pub fn indexed_cols(table: &str) -> &'static [&'static str] {
    match table {
        "cl_khoi_phong" => &["ma", "so_tt"],
        "cl_phong" => &["ma_du_an", "khoi_id", "so_tt"],
        "cl_phong_ban" => &["phong_id", "ma_don_vi", "so_tt"],
        "cl_mang" => &["phong_ban_id", "ma_mang", "so_tt"],
        "cl_thbq" => &["ten", "trang_thai", "gia_tri", "so_tt"],
        "cl_ngon_ngu" | "cl_loai_van_ban" => &["ma", "so_tt"],
        "cl_hop" => &["phong_ban_id", "hop_so"],
        "cl_bo_tap" => &["phong_ban_id", "loai", "so"],
        "cl_kho" => &["ten", "so_tt"],
        "cl_gia" => &["kho_id", "phong_ban_id", "tu_hop", "den_hop", "so_tt"],
        "cl_ho_so" => &[
            "phong_id", "phong_ban_id", "mang_id", "thbq_id", "hop_id", "bo_id", "tap_id", "gia_id", "so_tam", "ho_so_so", "tieu_de", "ngay_ket_thuc",
            "trung_thua", "thu_tu_sap_xep",
        ],
        "cl_van_ban" => &["ho_so_id", "so_thu_tu", "trich_yeu"],
        "cl_van_ban_file" => &["van_ban_id", "document_id"],
        "cl_muon_tra" => &["ho_so_id", "trang_thai", "ngay_du_kien_tra"],
        LICH_SU => &["ho_so_id", "entity", "entity_id", "actor_id", "at", "action"],
        _ => &[],
    }
}

pub fn all_tables() -> Vec<&'static str> {
    let mut v: Vec<&str> = spec::TABLES.iter().map(|t| t.name).collect();
    v.push(LICH_SU);
    v
}

fn table_ok(t: &str) -> bool {
    all_tables().contains(&t)
}

/// SQL tạo toàn bộ mirror + bảng đồng bộ (idempotent).
pub fn schema_sql() -> String {
    let mut s = String::new();
    for t in all_tables() {
        s.push_str(&format!(
            "CREATE TABLE IF NOT EXISTS {t} (\n  id TEXT PRIMARY KEY,\n  project_id TEXT,\n  ver INTEGER NOT NULL DEFAULT 0,\n  dirty INTEGER NOT NULL DEFAULT 0,\n  deleted_at TEXT,\n  base_json TEXT,\n  row_json TEXT NOT NULL,\n  local_at TEXT"
        ));
        for c in indexed_cols(t) {
            s.push_str(&format!(",\n  {c} GENERATED ALWAYS AS (json_extract(row_json, '$.{c}')) VIRTUAL"));
        }
        s.push_str("\n);\n");
        s.push_str(&format!("CREATE INDEX IF NOT EXISTS ix_{t}_ver ON {t} (project_id, ver);\n"));
        s.push_str(&format!("CREATE INDEX IF NOT EXISTS ix_{t}_dirty ON {t} (dirty) WHERE dirty <> 0;\n"));
        for c in indexed_cols(t) {
            s.push_str(&format!("CREATE INDEX IF NOT EXISTS ix_{t}_{c} ON {t} ({c});\n"));
        }
    }
    s.push_str(
        "CREATE TABLE IF NOT EXISTS cl_sync_outbox (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  op_id TEXT NOT NULL UNIQUE,
  project_id TEXT NOT NULL,
  tbl TEXT NOT NULL,
  row_id TEXT NOT NULL,
  op TEXT NOT NULL,
  base_json TEXT,
  new_json TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  edited_at TEXT,
  UNIQUE (tbl, row_id)
);
CREATE INDEX IF NOT EXISTS ix_cl_outbox_project ON cl_sync_outbox (project_id, status);
CREATE TABLE IF NOT EXISTS cl_sync_state (
  project_id TEXT PRIMARY KEY,
  cursor INTEGER NOT NULL DEFAULT 0,
  last_pull_at TEXT,
  last_push_at TEXT,
  last_error TEXT
);
CREATE TABLE IF NOT EXISTS cl_sync_conflicts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id TEXT NOT NULL,
  tbl TEXT NOT NULL,
  row_id TEXT NOT NULL,
  base_json TEXT,
  mine_json TEXT NOT NULL,
  theirs_json TEXT,
  fields_json TEXT NOT NULL DEFAULT '[]',
  message TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  resolved_at TEXT,
  resolution TEXT
);
CREATE INDEX IF NOT EXISTS ix_cl_conflicts_open ON cl_sync_conflicts (project_id) WHERE resolved_at IS NULL;
",
    );
    s
}

// ------------------------------------------------------------------ đọc

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalRow {
    pub row: Row,
    pub ver: i64,
    pub dirty: i64,
    pub deleted_at: Option<String>,
    pub base: Option<Row>,
}

fn parse_row(s: &str) -> Row {
    serde_json::from_str::<Value>(s).ok().and_then(|v| v.as_object().cloned()).unwrap_or_default()
}

pub fn get(conn: &Connection, table: &str, id: &str) -> rusqlite::Result<Option<LocalRow>> {
    if !table_ok(table) {
        return Ok(None);
    }
    conn.query_row(&format!("SELECT row_json, ver, dirty, deleted_at, base_json FROM {table} WHERE id = ?1"), params![id], |r| {
        Ok(LocalRow {
            row: parse_row(&r.get::<_, String>(0)?),
            ver: r.get(1)?,
            dirty: r.get(2)?,
            deleted_at: r.get(3)?,
            base: r.get::<_, Option<String>>(4)?.map(|s| parse_row(&s)),
        })
    })
    .optional()
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFilter {
    /// Bằng nhau trên cột sinh: [["phong_id", "..."], ...]
    #[serde(default)]
    pub eq: Vec<(String, Value)>,
    /// Tìm chữ (LIKE) trên các cột sinh.
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub q_cols: Vec<String>,
    #[serde(default)]
    pub include_deleted: bool,
    #[serde(default)]
    pub order: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

fn col_ok(table: &str, c: &str) -> bool {
    indexed_cols(table).contains(&c) || matches!(c, "id" | "project_id" | "ver" | "dirty" | "deleted_at")
}

/// Danh sách dòng (JSON + cột đồng bộ gắn vào `_sync`) của một bảng trong dự án (kể cả dòng dùng chung).
pub fn list(conn: &Connection, table: &str, project_id: &str, f: &ListFilter) -> rusqlite::Result<(Vec<Row>, i64)> {
    if !table_ok(table) {
        return Ok((vec![], 0));
    }
    let mut wh = vec!["(project_id = ?1 OR project_id IS NULL)".to_string()];
    let mut binds: Vec<Value> = vec![json!(project_id)];
    if !f.include_deleted {
        wh.push("deleted_at IS NULL".into());
    }
    for (c, v) in &f.eq {
        if col_ok(table, c) {
            binds.push(v.clone());
            wh.push(format!("{c} IS ?{}", binds.len()));
        }
    }
    if let Some(q) = f.q.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let cols: Vec<&String> = f.q_cols.iter().filter(|c| col_ok(table, c)).collect();
        if !cols.is_empty() {
            binds.push(json!(format!("%{q}%")));
            let n = binds.len();
            wh.push(format!("({})", cols.iter().map(|c| format!("{c} LIKE ?{n}")).collect::<Vec<_>>().join(" OR ")));
        }
    }
    let order = match f.order.as_deref() {
        Some(o) if o.split_whitespace().next().map_or(false, |c| col_ok(table, c)) => o.to_string(),
        _ => {
            let ic = indexed_cols(table);
            if ic.contains(&"so_tt") {
                "so_tt, id".into()
            } else if ic.contains(&"so_thu_tu") {
                "so_thu_tu, id".into()
            } else {
                "id".into()
            }
        }
    };
    let where_sql = wh.join(" AND ");
    let params_vec: Vec<rusqlite::types::Value> = binds.into_iter().map(json_to_sql).collect();
    let total: i64 = conn.query_row(&format!("SELECT count(*) FROM {table} WHERE {where_sql}"), rusqlite::params_from_iter(params_vec.iter()), |r| r.get(0))?;
    let limit = f.limit.unwrap_or(100_000).clamp(1, 200_000);
    let offset = f.offset.unwrap_or(0).max(0);
    let mut st = conn.prepare(&format!("SELECT row_json, ver, dirty, deleted_at FROM {table} WHERE {where_sql} ORDER BY {order} LIMIT {limit} OFFSET {offset}"))?;
    let rows = st
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |r| {
            let mut row = parse_row(&r.get::<_, String>(0)?);
            row.insert("_sync".into(), json!({"ver": r.get::<_, i64>(1)?, "dirty": r.get::<_, i64>(2)?, "deletedAt": r.get::<_, Option<String>>(3)?}));
            Ok(row)
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok((rows, total))
}

fn json_to_sql(v: Value) -> rusqlite::types::Value {
    use rusqlite::types::Value as S;
    match v {
        Value::Null => S::Null,
        Value::Bool(b) => S::Integer(b as i64),
        Value::Number(n) => n.as_i64().map(S::Integer).or_else(|| n.as_f64().map(S::Real)).unwrap_or(S::Null),
        Value::String(s) => S::Text(s),
        other => S::Text(other.to_string()),
    }
}

// ------------------------------------------------------------------ ghi từ máy chủ (pull / kết quả push)

/// Ghi dòng máy chủ. `force` = ghi đè cả khi đang dirty (kết quả push); mặc định bỏ qua dòng dirty.
pub fn upsert_from_server(conn: &Connection, table: &str, row: &Row, force: bool) -> rusqlite::Result<bool> {
    if !table_ok(table) {
        return Ok(false);
    }
    let id = row.get("id").and_then(Value::as_str).unwrap_or("").to_owned();
    if id.is_empty() {
        return Ok(false);
    }
    let ver = row.get("ver").and_then(Value::as_i64).unwrap_or(0);
    let project_id = row.get("project_id").and_then(Value::as_str).map(str::to_owned);
    let deleted_at = row.get("deleted_at").and_then(Value::as_str).map(str::to_owned);
    let json = Value::Object(row.clone()).to_string();
    let guard = if force { "" } else { " WHERE " };
    let cond = if force { "" } else { "dirty = 0" };
    let n = conn.execute(
        &format!(
            "INSERT INTO {table} (id, project_id, ver, dirty, deleted_at, base_json, row_json, local_at) VALUES (?1, ?2, ?3, 0, ?4, NULL, ?5, NULL)
             ON CONFLICT(id) DO UPDATE SET project_id = excluded.project_id, ver = excluded.ver, dirty = 0, deleted_at = excluded.deleted_at,
                base_json = NULL, row_json = excluded.row_json, local_at = NULL{guard}{cond}"
        ),
        params![id, project_id, ver, deleted_at, json],
    )?;
    if table == "cl_phong" && deleted_at.is_none() {
        // Cầu nối sang bảng `phong` cũ (cây thư mục, quét, LAN) — giữ mã phông tương ứng.
        if let (Some(ma), Some(ten)) = (row.get("ma_du_an").and_then(Value::as_str), row.get("ten").and_then(Value::as_str)) {
            let _ = crate::db::f_2o6(conn, ma, ten, None, row.get("phong_so").and_then(Value::as_str), row.get("muc_luc_so").and_then(Value::as_str));
        }
    }
    Ok(n > 0)
}

/// Xoá toàn bộ dữ liệu của dự án (kéo lại từ đầu). Giữ dòng dùng chung (project_id NULL).
pub fn wipe_project(conn: &Connection, project_id: &str) -> rusqlite::Result<()> {
    for t in all_tables() {
        conn.execute(&format!("DELETE FROM {t} WHERE project_id = ?1 AND dirty = 0"), params![project_id])?;
    }
    conn.execute("UPDATE cl_sync_state SET cursor = 0 WHERE project_id = ?1", params![project_id])?;
    Ok(())
}

// ------------------------------------------------------------------ ghi cục bộ (offline)

pub struct Who<'a> {
    pub project_id: &'a str,
    pub user_id: &'a str,
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Lỗi nghiệp vụ cục bộ (trước khi lên máy chủ).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalErr {
    pub field: String,
    pub message: String,
}

fn str_of(row: &Row, f: &str) -> Option<String> {
    match row.get(f) {
        Some(Value::String(s)) => Some(s.trim().to_owned()).filter(|s| !s.is_empty()),
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    }
}

/// Chuẩn hoá nhẹ tại máy: trim, ngày về dạng chuẩn, số tạm / tiêu đề / STT bắt buộc, số tạm không trùng trong phông.
pub fn validate_local(conn: &Connection, table: &str, id: &str, row: &mut Row) -> Vec<LocalErr> {
    let mut errs = vec![];
    for (_, v) in row.iter_mut() {
        if let Value::String(s) = v {
            let t = s.trim();
            if t.is_empty() {
                *v = Value::Null;
            } else if t.len() != s.len() {
                *v = Value::String(t.to_owned());
            }
        }
    }
    let date_cols: &[&str] = match table {
        "cl_ho_so" => &["ngay_bat_dau", "ngay_ket_thuc"],
        "cl_van_ban" => &["ngay_thang"],
        _ => &[],
    };
    for f in date_cols {
        if let Some(s) = str_of(row, f) {
            match digitext_core::chinh_ly::ClDate::parse(&s) {
                Ok(Some(d)) => {
                    row.insert((*f).to_owned(), Value::String(d.to_text()));
                }
                Ok(None) => {
                    row.insert((*f).to_owned(), Value::Null);
                }
                Err(e) => errs.push(LocalErr { field: (*f).to_owned(), message: e.to_string() }),
            }
        }
    }
    match table {
        "cl_ho_so" => {
            if str_of(row, "so_tam").is_none() {
                errs.push(LocalErr { field: "so_tam".into(), message: "Thiếu số tạm".into() });
            }
            if str_of(row, "tieu_de").is_none() {
                errs.push(LocalErr { field: "tieu_de".into(), message: "Thiếu tiêu đề hồ sơ".into() });
            }
            if str_of(row, "phong_id").is_none() {
                errs.push(LocalErr { field: "phong_id".into(), message: "Thiếu phông".into() });
            }
            if let (Some(p), Some(st)) = (str_of(row, "phong_id"), str_of(row, "so_tam")) {
                let dup: i64 = conn
                    .query_row(
                        "SELECT count(*) FROM cl_ho_so WHERE phong_id = ?1 AND lower(trim(so_tam)) = lower(?2) AND id <> ?3 AND deleted_at IS NULL",
                        params![p, st, id],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if dup > 0 {
                    errs.push(LocalErr { field: "so_tam".into(), message: format!("Số tạm {st:?} đã có trong phông") });
                }
            }
        }
        "cl_van_ban" => {
            if str_of(row, "ho_so_id").is_none() {
                errs.push(LocalErr { field: "ho_so_id".into(), message: "Thiếu hồ sơ".into() });
            }
            match row.get("so_thu_tu").and_then(Value::as_i64).or_else(|| str_of(row, "so_thu_tu").and_then(|s| s.parse().ok())) {
                Some(n) if n >= 1 => {
                    row.insert("so_thu_tu".into(), json!(n));
                    if let Some(hs) = str_of(row, "ho_so_id") {
                        let dup: i64 = conn
                            .query_row("SELECT count(*) FROM cl_van_ban WHERE ho_so_id = ?1 AND so_thu_tu = ?2 AND id <> ?3 AND deleted_at IS NULL", params![hs, n, id], |r| r.get(0))
                            .unwrap_or(0);
                        if dup > 0 {
                            errs.push(LocalErr { field: "so_thu_tu".into(), message: format!("Số thứ tự {n} đã có trong hồ sơ") });
                        }
                    }
                }
                _ => errs.push(LocalErr { field: "so_thu_tu".into(), message: "Số thứ tự văn bản phải ≥ 1".into() }),
            }
        }
        _ => {}
    }
    errs
}

/// Ghi / sửa một dòng tại máy: gộp `patch` vào bản hiện có, đánh dấu dirty, xếp outbox (giữ base của lần đầu).
pub fn write_local(conn: &Connection, sp: &TableSpec, id: &str, patch: &Row, who: &Who, edited_at: Option<&str>) -> rusqlite::Result<Row> {
    let existing = get(conn, sp.name, id)?;
    let now = now_iso();
    let mut row = existing.as_ref().map(|e| e.row.clone()).unwrap_or_default();
    for (k, v) in patch {
        if SYSTEM_COLS.contains(&k.as_str()) || !sp.is_writable(k) {
            continue;
        }
        row.insert(k.clone(), v.clone());
    }
    // Không đổi gì so với bản hiện có → không đánh dấu dirty, không tạo op.
    if let Some(e) = &existing {
        if writable_only(sp, &e.row) == writable_only(sp, &row) && e.deleted_at.is_none() {
            return Ok(e.row.clone());
        }
    }
    // Mở giao dịch riêng chỉ khi chưa ở trong giao dịch (caller như push_unmigrated đã mở).
    let own_tx = conn.is_autocommit();
    if own_tx {
        conn.execute_batch("BEGIN")?;
    }
    let tx = conn;
    let done = (|| -> rusqlite::Result<()> {
    match existing {
        None => {
            row.insert("id".into(), json!(id));
            row.insert("project_id".into(), json!(who.project_id));
            row.insert("created_by".into(), json!(who.user_id));
            row.insert("updated_by".into(), json!(who.user_id));
            row.insert("created_at".into(), json!(now));
            row.insert("updated_at".into(), json!(now));
            row.insert("deleted_at".into(), Value::Null);
            row.insert("ver".into(), json!(0));
            tx.execute(
                &format!("INSERT INTO {} (id, project_id, ver, dirty, deleted_at, base_json, row_json, local_at) VALUES (?1, ?2, 0, 2, NULL, NULL, ?3, ?4)", sp.name),
                params![id, who.project_id, Value::Object(row.clone()).to_string(), now],
            )?;
            outbox_put(tx, who.project_id, sp.name, id, "upsert", None, &writable_only(sp, &row), edited_at)?;
        }
        Some(e) => {
            row.insert("updated_by".into(), json!(who.user_id));
            row.insert("updated_at".into(), json!(now));
            let dirty = if e.dirty == 2 { 2 } else { 1 };
            let base = if e.dirty == 0 { Some(Value::Object(e.row.clone()).to_string()) } else { e.base.as_ref().map(|b| Value::Object(b.clone()).to_string()) };
            tx.execute(
                &format!("UPDATE {} SET row_json = ?2, dirty = ?3, base_json = COALESCE(base_json, ?4), local_at = ?5 WHERE id = ?1", sp.name),
                params![id, Value::Object(row.clone()).to_string(), dirty, base, now],
            )?;
            let base_row = if e.dirty == 0 { Some(e.row.clone()) } else { e.base.clone() };
            outbox_put(tx, who.project_id, sp.name, id, "upsert", base_row.as_ref(), &writable_only(sp, &row), edited_at)?;
        }
    }
    Ok(())
    })();
    if own_tx {
        match done {
            Ok(()) => conn.execute_batch("COMMIT")?,
            Err(_) => {
                let _ = conn.execute_batch("ROLLBACK");
            }
        }
    }
    done?;
    Ok(row)
}

fn writable_only(sp: &TableSpec, row: &Row) -> Row {
    let mut out = Row::new();
    for c in sp.writable() {
        if let Some(v) = row.get(c) {
            out.insert(c.to_owned(), v.clone());
        }
    }
    out
}

/// Xoá tại máy: dòng chưa từng lên máy chủ thì xoá hẳn; còn lại đánh dấu deleted_at + op delete. Lan xuống con.
pub fn delete_local(conn: &Connection, sp: &TableSpec, id: &str, who: &Who) -> rusqlite::Result<u64> {
    if !conn.is_autocommit() {
        return delete_inner(conn, sp, id, who);
    }
    let tx = conn.unchecked_transaction()?;
    let n = delete_inner(&tx, sp, id, who)?;
    tx.commit()?;
    Ok(n)
}

fn delete_inner(conn: &Connection, sp: &TableSpec, id: &str, who: &Who) -> rusqlite::Result<u64> {
    let Some(e) = get(conn, sp.name, id)? else { return Ok(0) };
    let mut n = 0u64;
    if e.dirty == 2 {
        conn.execute(&format!("DELETE FROM {} WHERE id = ?1", sp.name), params![id])?;
        conn.execute("DELETE FROM cl_sync_outbox WHERE tbl = ?1 AND row_id = ?2", params![sp.name, id])?;
        n += 1;
    } else if e.deleted_at.is_none() {
        let now = now_iso();
        let base = if e.dirty == 0 { Some(e.row.clone()) } else { e.base.clone() };
        conn.execute(&format!("UPDATE {} SET deleted_at = ?2, dirty = 1, base_json = COALESCE(base_json, ?3), local_at = ?2 WHERE id = ?1", sp.name), params![id, now, base.as_ref().map(|b| Value::Object(b.clone()).to_string())])?;
        outbox_put(conn, who.project_id, sp.name, id, "delete", base.as_ref(), &Row::new(), None)?;
        n += 1;
    }
    for child in sp.children() {
        let Some((col, _)) = child.parent else { continue };
        if !indexed_cols(child.name).contains(&col) {
            continue;
        }
        let ids: Vec<String> = {
            let mut st = conn.prepare(&format!("SELECT id FROM {} WHERE {col} = ?1 AND deleted_at IS NULL", child.name))?;
            let v = st.query_map(params![id], |r| r.get(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
            v
        };
        for cid in ids {
            n += delete_inner(conn, child, &cid, who)?;
        }
    }
    Ok(n)
}

fn outbox_put(conn: &Connection, project_id: &str, tbl: &str, row_id: &str, op: &str, base: Option<&Row>, new: &Row, edited_at: Option<&str>) -> rusqlite::Result<()> {
    let op_id = uuid::Uuid::now_v7().to_string();
    let base_json = base.map(|b| Value::Object(b.clone()).to_string());
    let edited = edited_at.map(str::to_owned).unwrap_or_else(now_iso);
    conn.execute(
        "INSERT INTO cl_sync_outbox (op_id, project_id, tbl, row_id, op, base_json, new_json, status, attempts, last_error, edited_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', 0, NULL, ?8)
         ON CONFLICT(tbl, row_id) DO UPDATE SET op = excluded.op, new_json = excluded.new_json, status = 'pending', attempts = 0, last_error = NULL,
            edited_at = excluded.edited_at, op_id = CASE WHEN cl_sync_outbox.attempts > 0 THEN excluded.op_id ELSE cl_sync_outbox.op_id END",
        params![op_id, project_id, tbl, row_id, op, base_json, Value::Object(new.clone()).to_string(), edited],
    )?;
    Ok(())
}

// ------------------------------------------------------------------ outbox / trạng thái / xung đột

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboxRow {
    pub id: i64,
    pub op_id: String,
    pub tbl: String,
    pub row_id: String,
    pub op: String,
    pub base: Option<Row>,
    pub new: Row,
    pub status: String,
    pub attempts: i64,
    pub last_error: Option<String>,
    pub created_at: String,
    pub edited_at: Option<String>,
}

fn outbox_row(r: &rusqlite::Row) -> rusqlite::Result<OutboxRow> {
    Ok(OutboxRow {
        id: r.get(0)?,
        op_id: r.get(1)?,
        tbl: r.get(2)?,
        row_id: r.get(3)?,
        op: r.get(4)?,
        base: r.get::<_, Option<String>>(5)?.map(|s| parse_row(&s)),
        new: parse_row(&r.get::<_, String>(6)?),
        status: r.get(7)?,
        attempts: r.get(8)?,
        last_error: r.get(9)?,
        created_at: r.get(10)?,
        edited_at: r.get(11)?,
    })
}

const OUTBOX_COLS: &str = "id, op_id, tbl, row_id, op, base_json, new_json, status, attempts, last_error, created_at, edited_at";

pub fn outbox_pending(conn: &Connection, project_id: &str, limit: i64) -> rusqlite::Result<Vec<OutboxRow>> {
    let mut st = conn.prepare(&format!("SELECT {OUTBOX_COLS} FROM cl_sync_outbox WHERE project_id = ?1 AND status = 'pending' AND attempts < 5 ORDER BY id LIMIT ?2"))?;
    let v = st.query_map(params![project_id, limit], outbox_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(v)
}

pub fn outbox_all(conn: &Connection, project_id: &str) -> rusqlite::Result<Vec<OutboxRow>> {
    let mut st = conn.prepare(&format!("SELECT {OUTBOX_COLS} FROM cl_sync_outbox WHERE project_id = ?1 ORDER BY id"))?;
    let v = st.query_map(params![project_id], outbox_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(v)
}

pub fn outbox_done(conn: &Connection, op_id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM cl_sync_outbox WHERE op_id = ?1", params![op_id])?;
    Ok(())
}

pub fn outbox_fail(conn: &Connection, op_id: &str, status: &str, err: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE cl_sync_outbox SET attempts = attempts + 1, last_error = ?2, status = ?3 WHERE op_id = ?1", params![op_id, err, status])?;
    Ok(())
}

pub fn outbox_retry(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("UPDATE cl_sync_outbox SET status = 'pending', attempts = 0, last_error = NULL, op_id = ?2 WHERE id = ?1", params![id, uuid::Uuid::now_v7().to_string()])?;
    Ok(())
}

/// Bỏ thay đổi cục bộ: khôi phục bản máy chủ (base) hoặc xoá dòng chưa từng lên máy chủ.
pub fn outbox_discard(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    let Some((tbl, row_id)): Option<(String, String)> = conn.query_row("SELECT tbl, row_id FROM cl_sync_outbox WHERE id = ?1", params![id], |r| Ok((r.get(0)?, r.get(1)?))).optional()? else {
        return Ok(());
    };
    let tx = conn.unchecked_transaction()?;
    if let Some(e) = get(&tx, &tbl, &row_id)? {
        match e.base {
            Some(b) => {
                let deleted = b.get("deleted_at").and_then(Value::as_str).map(str::to_owned);
                tx.execute(&format!("UPDATE {tbl} SET row_json = ?2, dirty = 0, base_json = NULL, deleted_at = ?3, local_at = NULL WHERE id = ?1"), params![row_id, Value::Object(b).to_string(), deleted])?;
            }
            None => {
                tx.execute(&format!("DELETE FROM {tbl} WHERE id = ?1"), params![row_id])?;
            }
        }
    }
    tx.execute("DELETE FROM cl_sync_outbox WHERE id = ?1", params![id])?;
    tx.commit()
}

pub fn pending_count(conn: &Connection, project_id: &str) -> rusqlite::Result<(i64, i64, i64)> {
    let pending: i64 = conn.query_row("SELECT count(*) FROM cl_sync_outbox WHERE project_id = ?1 AND status = 'pending'", params![project_id], |r| r.get(0))?;
    let failed: i64 = conn.query_row("SELECT count(*) FROM cl_sync_outbox WHERE project_id = ?1 AND status <> 'pending'", params![project_id], |r| r.get(0))?;
    let conflicts: i64 = conn.query_row("SELECT count(*) FROM cl_sync_conflicts WHERE project_id = ?1 AND resolved_at IS NULL", params![project_id], |r| r.get(0))?;
    Ok((pending, failed, conflicts))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    pub id: i64,
    pub tbl: String,
    pub row_id: String,
    pub base: Option<Row>,
    pub mine: Row,
    pub theirs: Option<Row>,
    pub fields: Vec<String>,
    pub message: Option<String>,
    pub created_at: String,
}

pub fn conflict_put(conn: &Connection, project_id: &str, tbl: &str, row_id: &str, base: Option<&Row>, mine: &Row, theirs: Option<&Row>, fields: &[String], message: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO cl_sync_conflicts (project_id, tbl, row_id, base_json, mine_json, theirs_json, fields_json, message) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            project_id,
            tbl,
            row_id,
            base.map(|b| Value::Object(b.clone()).to_string()),
            Value::Object(mine.clone()).to_string(),
            theirs.map(|t| Value::Object(t.clone()).to_string()),
            json!(fields).to_string(),
            message
        ],
    )?;
    Ok(())
}

pub fn conflicts_open(conn: &Connection, project_id: &str) -> rusqlite::Result<Vec<Conflict>> {
    let mut st = conn.prepare("SELECT id, tbl, row_id, base_json, mine_json, theirs_json, fields_json, message, created_at FROM cl_sync_conflicts WHERE project_id = ?1 AND resolved_at IS NULL ORDER BY id")?;
    let v = st
        .query_map(params![project_id], |r| {
            Ok(Conflict {
                id: r.get(0)?,
                tbl: r.get(1)?,
                row_id: r.get(2)?,
                base: r.get::<_, Option<String>>(3)?.map(|s| parse_row(&s)),
                mine: parse_row(&r.get::<_, String>(4)?),
                theirs: r.get::<_, Option<String>>(5)?.map(|s| parse_row(&s)),
                fields: serde_json::from_str(&r.get::<_, String>(6)?).unwrap_or_default(),
                message: r.get(7)?,
                created_at: r.get(8)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(v)
}

pub fn conflict_get(conn: &Connection, id: i64) -> rusqlite::Result<Option<(String, Conflict)>> {
    conn.query_row("SELECT project_id, id, tbl, row_id, base_json, mine_json, theirs_json, fields_json, message, created_at FROM cl_sync_conflicts WHERE id = ?1", params![id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            Conflict {
                id: r.get(1)?,
                tbl: r.get(2)?,
                row_id: r.get(3)?,
                base: r.get::<_, Option<String>>(4)?.map(|s| parse_row(&s)),
                mine: parse_row(&r.get::<_, String>(5)?),
                theirs: r.get::<_, Option<String>>(6)?.map(|s| parse_row(&s)),
                fields: serde_json::from_str(&r.get::<_, String>(7)?).unwrap_or_default(),
                message: r.get(8)?,
                created_at: r.get(9)?,
            },
        ))
    })
    .optional()
}

pub fn conflict_close(conn: &Connection, id: i64, resolution: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE cl_sync_conflicts SET resolved_at = datetime('now'), resolution = ?2 WHERE id = ?1", params![id, resolution])?;
    Ok(())
}

pub fn cursor_get(conn: &Connection, project_id: &str) -> rusqlite::Result<i64> {
    Ok(conn.query_row("SELECT cursor FROM cl_sync_state WHERE project_id = ?1", params![project_id], |r| r.get(0)).optional()?.unwrap_or(0))
}

pub fn cursor_set(conn: &Connection, project_id: &str, cursor: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO cl_sync_state (project_id, cursor, last_pull_at) VALUES (?1, ?2, datetime('now')) ON CONFLICT(project_id) DO UPDATE SET cursor = excluded.cursor, last_pull_at = datetime('now')",
        params![project_id, cursor],
    )?;
    Ok(())
}

pub fn state_error(conn: &Connection, project_id: &str, err: Option<&str>) -> rusqlite::Result<()> {
    conn.execute("INSERT INTO cl_sync_state (project_id, last_error) VALUES (?1, ?2) ON CONFLICT(project_id) DO UPDATE SET last_error = excluded.last_error", params![project_id, err])?;
    Ok(())
}

pub fn last_pull_at(conn: &Connection, project_id: &str) -> rusqlite::Result<Option<String>> {
    Ok(conn.query_row("SELECT last_pull_at FROM cl_sync_state WHERE project_id = ?1", params![project_id], |r| r.get(0)).optional()?.flatten())
}

// ------------------------------------------------------------------ hồ sơ: danh sách làm giàu

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoSoFilter {
    #[serde(default)]
    pub phong_id: Option<String>,
    #[serde(default)]
    pub phong_ban_id: Option<String>,
    #[serde(default)]
    pub mang_id: Option<String>,
    /// Danh sách id THBQ; rỗng = tất cả.
    #[serde(default)]
    pub thbq_ids: Vec<String>,
    /// Chỉ THBQ có thời hạn (trang_thai = 2).
    #[serde(default)]
    pub thbq_co_thoi_han: bool,
    /// VV | GL | HET
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub trung_thua: Option<bool>,
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub kho_id: Option<String>,
    #[serde(default)]
    pub gia_id: Option<String>,
    #[serde(default)]
    pub has_hop: Option<bool>,
    #[serde(default)]
    pub has_ho_so_so: Option<bool>,
    #[serde(default)]
    pub hop_id: Option<String>,
    #[serde(default)]
    pub only_dirty: bool,
    #[serde(default)]
    pub include_deleted: bool,
    /// so_tam | ho_so_so | ngay_kt | mang | thbq | updated
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub desc: bool,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

fn s(row: &Row, k: &str) -> Option<String> {
    row.get(k).and_then(Value::as_str).map(str::to_owned)
}
fn i(row: &Row, k: &str) -> Option<i64> {
    match row.get(k) {
        Some(Value::Number(n)) => n.as_i64(),
        Some(Value::String(t)) => t.trim().parse().ok(),
        _ => None,
    }
}

/// Bảng tra danh mục (id → dòng) đọc một lần cho làm giàu.
pub struct Lookups {
    pub phong: std::collections::HashMap<String, Row>,
    pub phong_ban: std::collections::HashMap<String, Row>,
    pub mang: std::collections::HashMap<String, Row>,
    pub thbq: std::collections::HashMap<String, Row>,
    pub hop: std::collections::HashMap<String, Row>,
    pub bo_tap: std::collections::HashMap<String, Row>,
    pub gia: Vec<Row>,
    pub kho: std::collections::HashMap<String, Row>,
}

pub fn lookups(conn: &Connection, project_id: &str) -> rusqlite::Result<Lookups> {
    let map = |t: &str| -> rusqlite::Result<std::collections::HashMap<String, Row>> {
        let (rows, _) = list(conn, t, project_id, &ListFilter { include_deleted: true, ..Default::default() })?;
        Ok(rows.into_iter().filter_map(|r| Some((s(&r, "id")?, r))).collect())
    };
    let (gia, _) = list(conn, "cl_gia", project_id, &ListFilter::default())?;
    Ok(Lookups { phong: map("cl_phong")?, phong_ban: map("cl_phong_ban")?, mang: map("cl_mang")?, thbq: map("cl_thbq")?, hop: map("cl_hop")?, bo_tap: map("cl_bo_tap")?, gia, kho: map("cl_kho")? })
}

/// Làm giàu một dòng hồ sơ như view `cl_ho_so_v` trên máy chủ (thêm `thbq_status`, `ma_ho_so`, tên danh mục, kho/giá).
pub fn enrich_ho_so(row: &mut Row, lk: &Lookups, nam_now: i32, so_van_ban: i64, conflict: bool) {
    let get = |m: &std::collections::HashMap<String, Row>, k: &str| s(row, k).and_then(|id| m.get(&id).cloned());
    let phong = get(&lk.phong, "phong_id");
    let pb = get(&lk.phong_ban, "phong_ban_id");
    let mang = get(&lk.mang, "mang_id");
    let thbq = get(&lk.thbq, "thbq_id");
    let hop = get(&lk.hop, "hop_id");
    let bo = get(&lk.bo_tap, "bo_id");
    let tap = get(&lk.bo_tap, "tap_id");
    let nam_kt = s(row, "ngay_ket_thuc").and_then(|t| digitext_core::chinh_ly::ClDate::parse(&t).ok().flatten()).map(|d| d.year);
    row.insert("nam_ket_thuc".into(), json!(nam_kt));
    row.insert("ten_phong".into(), json!(phong.as_ref().and_then(|p| s(p, "ten"))));
    row.insert("ma_du_an".into(), json!(phong.as_ref().and_then(|p| s(p, "ma_du_an"))));
    row.insert("phong_so".into(), json!(phong.as_ref().and_then(|p| s(p, "phong_so"))));
    row.insert("muc_luc_so".into(), json!(phong.as_ref().and_then(|p| s(p, "muc_luc_so"))));
    row.insert("ten_phong_ban".into(), json!(pb.as_ref().and_then(|p| s(p, "ten"))));
    row.insert("ma_don_vi".into(), json!(pb.as_ref().and_then(|p| s(p, "ma_don_vi"))));
    row.insert("ten_mang".into(), json!(mang.as_ref().and_then(|p| s(p, "ten"))));
    row.insert("ma_mang".into(), json!(mang.as_ref().and_then(|p| s(p, "ma_mang"))));
    row.insert("mang_so_tt".into(), json!(mang.as_ref().and_then(|p| i(p, "so_tt"))));
    row.insert("ten_thbq".into(), json!(thbq.as_ref().and_then(|p| s(p, "ten"))));
    let tt = thbq.as_ref().and_then(|p| i(p, "trang_thai"));
    let gt = thbq.as_ref().and_then(|p| i(p, "gia_tri"));
    row.insert("thbq_trang_thai".into(), json!(tt));
    row.insert("thbq_gia_tri".into(), json!(gt));
    row.insert("thbq_so_tt".into(), json!(thbq.as_ref().and_then(|p| i(p, "so_tt"))));
    row.insert("thbq_ma_tt05".into(), json!(thbq.as_ref().and_then(|p| s(p, "ma_tt05"))));
    row.insert("thbq_status".into(), json!(digitext_core::chinh_ly::thbq_status(tt.map(|v| v as i16), gt.map(|v| v as i32), nam_kt, nam_now)));
    let hop_so = hop.as_ref().and_then(|h| s(h, "hop_so"));
    let hop_int: Option<i64> = hop_so.as_deref().and_then(|h| h.trim().parse().ok());
    row.insert("hop_so".into(), json!(hop_so));
    row.insert("hop_so_int".into(), json!(hop_int));
    row.insert("bo_so".into(), json!(bo.as_ref().and_then(|b| i(b, "so"))));
    row.insert("tap_so".into(), json!(tap.as_ref().and_then(|b| i(b, "so"))));
    // giá: gán tay hoặc suy ra từ dải hộp của phòng ban
    let pb_id = s(row, "phong_ban_id");
    let gia = s(row, "gia_id").and_then(|gid| lk.gia.iter().find(|g| s(g, "id").as_deref() == Some(&gid)).cloned()).or_else(|| {
        let h = hop_int?;
        let pbid = pb_id.clone()?;
        lk.gia.iter().find(|g| s(g, "phong_ban_id").as_deref() == Some(&pbid) && i(g, "tu_hop").map_or(false, |a| a <= h) && i(g, "den_hop").map_or(false, |b| h <= b)).cloned()
    });
    let kho = gia.as_ref().and_then(|g| s(g, "kho_id")).and_then(|k| lk.kho.get(&k).cloned());
    row.insert("gia_eff_id".into(), json!(gia.as_ref().and_then(|g| s(g, "id"))));
    row.insert("ten_gia".into(), json!(gia.as_ref().and_then(|g| s(g, "ten"))));
    row.insert("kho_eff_id".into(), json!(kho.as_ref().and_then(|k| s(k, "id"))));
    row.insert("ten_kho".into(), json!(kho.as_ref().and_then(|k| s(k, "ten"))));
    row.insert(
        "ma_ho_so".into(),
        json!(digitext_core::chinh_ly::ma_ho_so(
            phong.as_ref().and_then(|p| s(p, "ma_du_an")).as_deref(),
            phong.as_ref().and_then(|p| s(p, "phong_so")).as_deref(),
            nam_kt,
            s(row, "ho_so_so").as_deref(),
            phong.as_ref().and_then(|p| s(p, "muc_luc_so")).as_deref(),
        )),
    );
    row.insert("so_van_ban".into(), json!(so_van_ban));
    let dirty = row.get("_sync").and_then(|v| v.get("dirty")).and_then(Value::as_i64).unwrap_or(0);
    row.insert("sync".into(), json!(if conflict { "conflict" } else if dirty != 0 { "pending" } else { "clean" }));
}

pub fn ho_so_list(conn: &Connection, project_id: &str, f: &HoSoFilter) -> rusqlite::Result<(Vec<Row>, i64)> {
    let mut eq = vec![];
    for (c, v) in [("phong_id", &f.phong_id), ("phong_ban_id", &f.phong_ban_id), ("mang_id", &f.mang_id), ("hop_id", &f.hop_id)] {
        if let Some(v) = v.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            eq.push((c.to_string(), json!(v)));
        }
    }
    if let Some(t) = f.trung_thua {
        eq.push(("trung_thua".into(), json!(t as i64)));
    }
    let lf = ListFilter { eq, q: f.q.clone(), q_cols: vec!["tieu_de".into(), "so_tam".into(), "ho_so_so".into()], include_deleted: f.include_deleted, order: Some("so_tam, id".into()), limit: Some(200_000), offset: None };
    let (mut rows, _) = list(conn, "cl_ho_so", project_id, &lf)?;
    let lk = lookups(conn, project_id)?;
    let nam_now = chrono::Utc::now().format("%Y").to_string().parse().unwrap_or(2026);
    let conflicts: std::collections::HashSet<String> = {
        let mut st = conn.prepare("SELECT row_id FROM cl_sync_conflicts WHERE project_id = ?1 AND tbl = 'cl_ho_so' AND resolved_at IS NULL")?;
        let v = st.query_map(params![project_id], |r| r.get::<_, String>(0))?.collect::<rusqlite::Result<_>>()?;
        v
    };
    let counts: std::collections::HashMap<String, i64> = {
        let mut st = conn.prepare("SELECT ho_so_id, count(*) FROM cl_van_ban WHERE deleted_at IS NULL GROUP BY ho_so_id")?;
        let v = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?.collect::<rusqlite::Result<_>>()?;
        v
    };
    for row in rows.iter_mut() {
        let id = s(row, "id").unwrap_or_default();
        let dirty = row.get("_sync").and_then(|v| v.get("dirty")).and_then(Value::as_i64).unwrap_or(0);
        let _ = dirty;
        enrich_ho_so(row, &lk, nam_now, counts.get(&id).copied().unwrap_or(0), conflicts.contains(&id));
    }
    let thbq_set: std::collections::HashSet<&str> = f.thbq_ids.iter().map(String::as_str).collect();
    rows.retain(|r| {
        if !thbq_set.is_empty() && !s(r, "thbq_id").map_or(false, |t| thbq_set.contains(t.as_str())) {
            return false;
        }
        if f.thbq_co_thoi_han && i(r, "thbq_trang_thai") != Some(2) {
            return false;
        }
        if let Some(st) = f.status.as_deref() {
            if s(r, "thbq_status").as_deref() != Some(st) {
                return false;
            }
        }
        if let Some(k) = f.kho_id.as_deref() {
            if s(r, "kho_eff_id").as_deref() != Some(k) {
                return false;
            }
        }
        if let Some(g) = f.gia_id.as_deref() {
            if s(r, "gia_eff_id").as_deref() != Some(g) {
                return false;
            }
        }
        if let Some(h) = f.has_hop {
            if s(r, "hop_id").is_some() != h {
                return false;
            }
        }
        if let Some(h) = f.has_ho_so_so {
            if s(r, "ho_so_so").map_or(false, |v| !v.trim().is_empty()) != h {
                return false;
            }
        }
        if f.only_dirty && s(r, "sync").as_deref() == Some("clean") {
            return false;
        }
        true
    });
    let key_date = |r: &Row| s(r, "ngay_ket_thuc").and_then(|t| digitext_core::chinh_ly::ClDate::parse(&t).ok().flatten()).map(|d| d.sort_key());
    let cmp_opt = |a: Option<(i32, u32, u32)>, b: Option<(i32, u32, u32)>| match (a, b) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        _ => std::cmp::Ordering::Equal,
    };
    let nat = |a: &Row, b: &Row, k: &str| digitext_core::chinh_ly::natural_cmp(&s(a, k).unwrap_or_default(), &s(b, k).unwrap_or_default());
    rows.sort_by(|a, b| {
        let o = match f.sort.as_deref().unwrap_or("so_tam") {
            "ho_so_so" => i(a, "hop_so_int").cmp(&i(b, "hop_so_int")).then_with(|| nat(a, b, "hop_so")).then_with(|| nat(a, b, "ho_so_so")).then_with(|| nat(a, b, "so_tam")),
            "ngay_kt" => cmp_opt(key_date(a), key_date(b)).then_with(|| nat(a, b, "so_tam")),
            "mang" => i(a, "mang_so_tt").cmp(&i(b, "mang_so_tt")).then_with(|| s(a, "ten_mang").cmp(&s(b, "ten_mang"))).then_with(|| cmp_opt(key_date(a), key_date(b))),
            "thbq" => i(a, "thbq_so_tt").cmp(&i(b, "thbq_so_tt")).then_with(|| i(a, "mang_so_tt").cmp(&i(b, "mang_so_tt"))).then_with(|| cmp_opt(key_date(a), key_date(b))),
            "updated" => s(a, "updated_at").cmp(&s(b, "updated_at")),
            _ => nat(a, b, "so_tam"),
        };
        if f.desc {
            o.reverse()
        } else {
            o
        }
    });
    let total = rows.len() as i64;
    let offset = f.offset.unwrap_or(0).max(0) as usize;
    let limit = f.limit.unwrap_or(500).clamp(1, 200_000) as usize;
    let page: Vec<Row> = rows.into_iter().skip(offset).take(limit).collect();
    Ok((page, total))
}

/// Văn bản của hồ sơ (kèm `files`, `ma_van_ban`, tên loại VB), theo số thứ tự.
pub fn van_ban_list(conn: &Connection, project_id: &str, ho_so_id: &str) -> rusqlite::Result<Vec<Row>> {
    let (mut rows, _) = list(conn, "cl_van_ban", project_id, &ListFilter { eq: vec![("ho_so_id".into(), json!(ho_so_id))], order: Some("so_thu_tu, id".into()), ..Default::default() })?;
    let (files, _) = list(conn, "cl_van_ban_file", project_id, &ListFilter { limit: Some(200_000), ..Default::default() })?;
    let (loai, _) = list(conn, "cl_loai_van_ban", project_id, &ListFilter::default())?;
    let ma_hs = {
        let (hs, _) = ho_so_list(conn, project_id, &HoSoFilter { include_deleted: true, limit: Some(1), ..Default::default() }).map(|(mut v, t)| {
            v.retain(|r| s(r, "id").as_deref() == Some(ho_so_id));
            (v, t)
        })?;
        hs.first().and_then(|h| s(h, "ma_ho_so"))
    };
    for r in rows.iter_mut() {
        let id = s(r, "id").unwrap_or_default();
        let fs: Vec<Row> = files.iter().filter(|f| s(f, "van_ban_id").as_deref() == Some(&id)).cloned().collect();
        r.insert("files".into(), json!(fs));
        let stt = i(r, "so_thu_tu").unwrap_or(0);
        r.insert("ma_van_ban".into(), json!(ma_hs.as_deref().map(|m| digitext_core::chinh_ly::ma_van_ban_tt05(m, stt))));
        let ma = s(r, "loai_vb");
        r.insert("ten_loai_vb".into(), json!(ma.and_then(|m| loai.iter().find(|l| s(l, "ma").as_deref() == Some(&m)).and_then(|l| s(l, "ten")))));
    }
    Ok(rows)
}

/// Số thứ tự văn bản tiếp theo của hồ sơ.
pub fn next_stt(conn: &Connection, ho_so_id: &str) -> rusqlite::Result<i64> {
    let n: Option<i64> = conn.query_row("SELECT max(so_thu_tu) FROM cl_van_ban WHERE ho_so_id = ?1 AND deleted_at IS NULL", params![ho_so_id], |r| r.get(0))?;
    Ok(n.unwrap_or(0) + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(&schema_sql()).unwrap();
        c
    }
    fn who() -> Who<'static> {
        Who { project_id: "p1", user_id: "u1" }
    }
    fn hs() -> &'static TableSpec {
        spec::spec("cl_ho_so").unwrap()
    }

    #[test]
    fn ghi_cuc_bo_tao_outbox_va_gop_lan_sua() {
        let c = conn();
        let phong = json!({"id": "ph1", "project_id": "p1", "ma_du_an": "UBND", "ten": "Phông", "ver": 5}).as_object().unwrap().clone();
        assert!(upsert_from_server(&c, "cl_phong", &phong, false).unwrap());
        let mut patch: Row = json!({"phong_id": "ph1", "so_tam": " 12 ", "tieu_de": "A", "ngay_ket_thuc": "3/2021"}).as_object().unwrap().clone();
        assert!(validate_local(&c, "cl_ho_so", "h1", &mut patch).is_empty());
        assert_eq!(patch["ngay_ket_thuc"], json!("03/2021"));
        write_local(&c, hs(), "h1", &patch, &who(), None).unwrap();
        let e = get(&c, "cl_ho_so", "h1").unwrap().unwrap();
        assert_eq!(e.dirty, 2);
        let ob = outbox_pending(&c, "p1", 10).unwrap();
        assert_eq!(ob.len(), 1);
        assert_eq!(ob[0].op, "upsert");
        assert!(ob[0].base.is_none());
        // giả lập máy chủ chấp nhận
        let mut srv = e.row.clone();
        srv.insert("ver".into(), json!(10));
        upsert_from_server(&c, "cl_ho_so", &srv, true).unwrap();
        outbox_done(&c, &ob[0].op_id).unwrap();
        // sửa hai lần → 1 op, base = bản máy chủ ver 10
        write_local(&c, hs(), "h1", &json!({"tieu_de": "B"}).as_object().unwrap().clone(), &who(), None).unwrap();
        write_local(&c, hs(), "h1", &json!({"so_to": "9"}).as_object().unwrap().clone(), &who(), None).unwrap();
        let ob = outbox_pending(&c, "p1", 10).unwrap();
        assert_eq!(ob.len(), 1);
        assert_eq!(ob[0].base.as_ref().unwrap()["ver"], json!(10));
        assert_eq!(ob[0].new["tieu_de"], json!("B"));
        assert_eq!(ob[0].new["so_to"], json!("9"));
        // pull không ghi đè dòng dirty
        let mut srv2 = srv.clone();
        srv2.insert("tieu_de".into(), json!("Z"));
        srv2.insert("ver".into(), json!(11));
        assert!(!upsert_from_server(&c, "cl_ho_so", &srv2, false).unwrap());
        assert_eq!(get(&c, "cl_ho_so", "h1").unwrap().unwrap().row["tieu_de"], json!("B"));
        // trùng số tạm cục bộ
        let mut dup: Row = json!({"phong_id": "ph1", "so_tam": "12", "tieu_de": "C"}).as_object().unwrap().clone();
        assert_eq!(validate_local(&c, "cl_ho_so", "h2", &mut dup).len(), 1);
        // danh sách làm giàu
        let (rows, total) = ho_so_list(&c, "p1", &HoSoFilter::default()).unwrap();
        assert_eq!(total, 1);
        assert_eq!(rows[0]["ten_phong"], json!("Phông"));
        assert_eq!(rows[0]["sync"], json!("pending"));
        // bỏ thay đổi → về bản máy chủ
        outbox_discard(&c, ob[0].id).unwrap();
        let e = get(&c, "cl_ho_so", "h1").unwrap().unwrap();
        assert_eq!(e.dirty, 0);
        assert_eq!(e.row["tieu_de"], json!("A"));
    }

    #[test]
    fn xoa_cuc_bo_lan_xuong_van_ban() {
        let c = conn();
        write_local(&c, hs(), "h1", &json!({"phong_id": "ph1", "so_tam": "1", "tieu_de": "A"}).as_object().unwrap().clone(), &who(), None).unwrap();
        let mut srv = get(&c, "cl_ho_so", "h1").unwrap().unwrap().row;
        srv.insert("ver".into(), json!(3));
        upsert_from_server(&c, "cl_ho_so", &srv, true).unwrap();
        c.execute("DELETE FROM cl_sync_outbox", []).unwrap();
        let vb = spec::spec("cl_van_ban").unwrap();
        write_local(&c, vb, "v1", &json!({"ho_so_id": "h1", "so_thu_tu": 1, "trich_yeu": "x"}).as_object().unwrap().clone(), &who(), None).unwrap();
        let n = delete_local(&c, hs(), "h1", &who()).unwrap();
        assert_eq!(n, 2);
        assert!(get(&c, "cl_van_ban", "v1").unwrap().is_none(), "văn bản chưa lên máy chủ thì xoá hẳn");
        let e = get(&c, "cl_ho_so", "h1").unwrap().unwrap();
        assert!(e.deleted_at.is_some());
        let ob = outbox_pending(&c, "p1", 10).unwrap();
        assert_eq!(ob.len(), 1);
        assert_eq!(ob[0].op, "delete");
        assert_eq!(pending_count(&c, "p1").unwrap(), (1, 0, 0));
    }
}
