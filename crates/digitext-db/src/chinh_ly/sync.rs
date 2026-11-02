// Copyright (c) 2026 DigiText. All rights reserved.
//! Đồng bộ chỉnh lý: một luồng ghi duy nhất `apply_op` cho cả route online lẫn push offline
//! (gộp 3 chiều theo `base`), pull theo `ver` toàn cục, dọn tombstone.

use super::lich_su::{self, diff_rows};
use super::merge::{self, Row};
use super::rows;
use super::spec::{self, TableSpec, WriteOp, SYSTEM_COLS};
use super::validate::{self, FieldErr};
use chrono::{DateTime, Utc};
use digitext_core::Permission;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgConnection, PgPool, Row as _};
use uuid::Uuid;

/// Ngữ cảnh người gọi (route điền từ `Ts`).
pub struct OpCtx<'a> {
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub user_name: &'a str,
    pub device_id: Option<Uuid>,
    /// `online` (route REST) | `sync` (push offline) | `import`.
    pub source: &'a str,
    pub has_perm: &'a (dyn Fn(Permission) -> bool + Sync),
    pub is_super: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpKind {
    Upsert,
    Delete,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncOp {
    #[serde(default)]
    pub op_id: Option<Uuid>,
    pub table: String,
    pub id: Uuid,
    pub op: OpKind,
    /// Dòng lúc client kéo về (null = tạo offline). Route online có thể bỏ trống và dùng `if_ver`.
    #[serde(default)]
    pub base: Option<Row>,
    /// Các cột client ghi (cột vắng = không đổi).
    #[serde(default)]
    pub new: Row,
    #[serde(default)]
    pub if_ver: Option<i64>,
    #[serde(default)]
    pub edited_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OpResult {
    Applied { row: Option<Row>, ver: Option<i64> },
    Merged { row: Row, ver: i64, merged_fields: Vec<String> },
    NoChange { row: Row, ver: i64 },
    Conflict { row: Option<Row>, conflict_fields: Vec<String> },
    Rejected { reason: String, message: String, #[serde(skip_serializing_if = "Vec::is_empty")] errors: Vec<FieldErr> },
    Duplicate { result: Value },
}

impl OpResult {
    fn rejected(reason: &str, message: impl Into<String>) -> Self {
        OpResult::Rejected { reason: reason.to_owned(), message: message.into(), errors: vec![] }
    }
    pub fn is_ok(&self) -> bool {
        matches!(self, OpResult::Applied { .. } | OpResult::Merged { .. } | OpResult::NoChange { .. } | OpResult::Duplicate { .. })
    }
}

fn ver_of(row: &Row) -> Option<i64> {
    rows::i64_of(row, "ver")
}

/// Thông điệp tiếng Việt theo tên ràng buộc duy nhất.
pub fn unique_message(constraint: &str) -> String {
    match constraint {
        "ux_cl_ho_so_so_tam" => "Số tạm đã có trong phông (cùng loại vĩnh viễn / có thời hạn)".into(),
        "ux_cl_van_ban_stt" => "Số thứ tự văn bản đã có trong hồ sơ".into(),
        "ux_cl_hop_so" => "Số hộp đã có trong phòng ban (cùng loại vĩnh viễn / có thời hạn)".into(),
        "ux_cl_bo_tap_so" => "Số bó / tập đã có trong phòng ban".into(),
        "ux_cl_phong_ma" => "Mã phông đã tồn tại trong dự án".into(),
        "ux_cl_phong_ban_ma" => "Mã đơn vị đã có trong phông".into(),
        "ux_cl_mang_ma" => "Mã mảng đã có trong phòng ban".into(),
        "ux_cl_thbq_ten" => "Tên thời hạn bảo quản đã tồn tại".into(),
        "ux_cl_ngon_ngu_ma" => "Mã ngôn ngữ đã tồn tại".into(),
        "ux_cl_loai_van_ban_ma" => "Mã loại văn bản đã tồn tại".into(),
        "ux_cl_khoi_phong_ma" => "Mã khối phông đã tồn tại".into(),
        "ux_cl_kho_ten" => "Tên kho đã tồn tại".into(),
        "ux_cl_muon_tra_mo" => "Hồ sơ đang có phiếu mượn chưa trả".into(),
        other => format!("Trùng khoá duy nhất ({other})"),
    }
}

/// Chuyển lỗi DB khoá duy nhất / khoá ngoại thành `Rejected`; lỗi khác trả về nguyên.
fn map_db_err(e: sqlx::Error) -> Result<OpResult, sqlx::Error> {
    if let sqlx::Error::Database(db) = &e {
        let c = db.constraint().unwrap_or("").to_owned();
        match db.code().as_deref() {
            Some("23505") => return Ok(OpResult::rejected(&format!("unique:{c}"), unique_message(&c))),
            Some("23503") => return Ok(OpResult::rejected(&format!("fk:{c}"), "Tham chiếu tới bản ghi không tồn tại")),
            // Trigger ck_cl_ho_so_hop_loai (0015) mang câu tiếng Việt riêng.
            Some("23514") if c == "ck_cl_ho_so_hop_loai" => return Ok(OpResult::rejected(&format!("check:{c}"), db.message().to_owned())),
            Some("23514") => return Ok(OpResult::rejected(&format!("check:{c}"), "Giá trị không hợp lệ")),
            _ => {}
        }
    }
    Err(e)
}

async fn lock_project(conn: &mut PgConnection, project: Option<Uuid>) -> sqlx::Result<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(cl_lock_key($1))").bind(project).execute(conn).await?;
    Ok(())
}

/// Áp một op trong giao dịch riêng. Không bao giờ trả lỗi cho dữ liệu sai — chỉ `Rejected`.
pub async fn apply_op(pool: &PgPool, ctx: &OpCtx<'_>, op: &SyncOp) -> sqlx::Result<OpResult> {
    let Some(sp) = spec::spec(&op.table) else {
        return Ok(OpResult::rejected("unknown_table", format!("Không có bảng {}", op.table)));
    };
    let offline = ctx.source == "sync";
    if offline && !sp.offline_write {
        return Ok(OpResult::rejected("online_only_table", format!("{} chỉ sửa được khi có kết nối máy chủ", sp.name)));
    }
    let mut tx = pool.begin().await?;
    if let Some(op_id) = op.op_id {
        let prev: Option<Value> = sqlx::query_scalar("SELECT result FROM cl_sync_ops WHERE op_id = $1").bind(op_id).fetch_optional(&mut *tx).await?;
        if let Some(result) = prev {
            return Ok(OpResult::Duplicate { result });
        }
    }
    lock_project(&mut tx, Some(ctx.project_id)).await?;
    if sp.shared {
        lock_project(&mut tx, None).await?;
    }
    let server = rows::get_for_update(&mut tx, sp.name, op.id).await?;
    let res = match apply_inner(&mut tx, ctx, sp, op, server, offline).await {
        Ok(r) => r,
        Err(e) => {
            // Giao dịch đã hỏng: bỏ và ghi kết quả trong giao dịch mới (chỉ khi là lỗi dữ liệu).
            drop(tx);
            let r = map_db_err(e)?;
            if let Some(op_id) = op.op_id {
                record_op(pool, ctx, op, op_id, &r).await?;
            }
            return Ok(r);
        }
    };
    if let Some(op_id) = op.op_id {
        let v = serde_json::to_value(&res).unwrap_or(Value::Null);
        sqlx::query("INSERT INTO cl_sync_ops (op_id, project_id, user_id, device_id, tbl, row_id, outcome, result) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT DO NOTHING")
            .bind(op_id)
            .bind(ctx.project_id)
            .bind(ctx.user_id)
            .bind(ctx.device_id)
            .bind(sp.name)
            .bind(op.id)
            .bind(v.get("status").and_then(Value::as_str).unwrap_or("?"))
            .bind(&v)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(res)
}

async fn record_op(pool: &PgPool, ctx: &OpCtx<'_>, op: &SyncOp, op_id: Uuid, r: &OpResult) -> sqlx::Result<()> {
    let v = serde_json::to_value(r).unwrap_or(Value::Null);
    sqlx::query("INSERT INTO cl_sync_ops (op_id, project_id, user_id, device_id, tbl, row_id, outcome, result) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT DO NOTHING")
        .bind(op_id)
        .bind(ctx.project_id)
        .bind(ctx.user_id)
        .bind(ctx.device_id)
        .bind(&op.table)
        .bind(op.id)
        .bind(v.get("status").and_then(Value::as_str).unwrap_or("?"))
        .bind(&v)
        .execute(pool)
        .await?;
    Ok(())
}

fn history_fields(sp: &TableSpec) -> Vec<&'static str> {
    sp.writable().collect()
}

async fn write_history(conn: &mut PgConnection, ctx: &OpCtx<'_>, sp: &TableSpec, action: &str, before: Option<&Row>, after: &Row, edited_at: Option<DateTime<Utc>>) -> sqlx::Result<()> {
    if !sp.history {
        return Ok(());
    }
    let fields = history_fields(sp);
    let (diffs, n_new, n_edited) = match action {
        "delete" => (vec![], 0, 0),
        _ => diff_rows(before, after, &fields),
    };
    if action == "edit" && diffs.is_empty() {
        return Ok(());
    }
    // 0023: văn bản máy tạo / cập nhật từ OCR ghi hành động 'ocr' — dashboard_daily 4.10 chỉ đếm add/import/edit
    // nên người bấm nút không được cộng hàng nghìn văn bản vào tổng kết nhân viên.
    let action = if ctx.source == "ocr" && matches!(action, "add" | "edit") { "ocr" } else { action };
    let entity = sp.name.trim_start_matches("cl_");
    let entity_id = rows::uuid_of(after, "id").unwrap_or_default();
    let (ho_so_id, phong_id, title) = if sp.name == "cl_van_ban" {
        let hs = rows::uuid_of(after, "ho_so_id").unwrap_or_default();
        let phong: Option<Uuid> = sqlx::query_scalar("SELECT phong_id FROM cl_ho_so WHERE id = $1").bind(hs).fetch_optional(&mut *conn).await?;
        (hs, phong, rows::str_of(after, "trich_yeu").map(str::to_owned))
    } else {
        (entity_id, rows::uuid_of(after, "phong_id"), rows::str_of(after, "tieu_de").map(str::to_owned))
    };
    lich_su::insert(
        conn,
        lich_su::NewEntry {
            project_id: ctx.project_id,
            entity,
            entity_id,
            ho_so_id,
            phong_id,
            action,
            actor_id: Some(ctx.user_id),
            actor_name: Some(ctx.user_name),
            at_client: edited_at,
            source: ctx.source,
            title: title.as_deref(),
            diffs,
            n_new,
            n_edited,
        },
    )
    .await
}

async fn apply_inner(tx: &mut PgConnection, ctx: &OpCtx<'_>, sp: &TableSpec, op: &SyncOp, server: Option<Row>, offline: bool) -> sqlx::Result<OpResult> {
    let is_seed = server.as_ref().map(|r| r.get("project_id").map_or(false, Value::is_null)).unwrap_or(false);
    if is_seed && !ctx.is_super {
        return Ok(OpResult::rejected("forbidden", "Dòng danh mục dùng chung chỉ super admin sửa được"));
    }
    if let Some(s) = &server {
        if rows::uuid_of(s, "project_id").map_or(false, |p| p != ctx.project_id) {
            return Ok(OpResult::rejected("forbidden", "Bản ghi thuộc dự án khác"));
        }
    }
    match op.op {
        OpKind::Delete => {
            let Some(server) = server else {
                return Ok(OpResult::Applied { row: None, ver: None });
            };
            if !server.get("deleted_at").map_or(true, Value::is_null) {
                return Ok(OpResult::Applied { row: Some(server.clone()), ver: ver_of(&server) });
            }
            if !(ctx.has_perm)(sp.perm_for(WriteOp::Delete)) {
                return Ok(OpResult::rejected("forbidden", "Không có quyền xoá"));
            }
            if let Some(base) = &op.base {
                if ver_of(base) != ver_of(&server) {
                    let fields = history_fields(sp);
                    let changed = merge::changed_fields(base, &server, &fields);
                    if !changed.is_empty() {
                        return Ok(OpResult::Conflict { row: Some(server), conflict_fields: changed.keys().cloned().collect() });
                    }
                }
            } else if let Some(v) = op.if_ver {
                if Some(v) != ver_of(&server) {
                    return Ok(OpResult::Conflict { row: Some(server), conflict_fields: vec![] });
                }
            }
            if sp.name == "cl_ho_so" {
                let open: i64 = sqlx::query_scalar("SELECT count(*) FROM cl_muon_tra WHERE ho_so_id = $1 AND trang_thai = 'dang_muon' AND deleted_at IS NULL")
                    .bind(op.id)
                    .fetch_one(&mut *tx)
                    .await?;
                if open > 0 {
                    return Ok(OpResult::rejected("loan_open", "Hồ sơ đang cho mượn, chưa xoá được"));
                }
            }
            if !matches!(sp.name, "cl_ho_so" | "cl_van_ban" | "cl_van_ban_file" | "cl_muon_tra") {
                let refs = rows::live_refs(tx, sp, op.id).await?;
                if refs > 0 {
                    return Ok(OpResult::rejected("in_use", format!("Còn {refs} bản ghi đang dùng, không xoá được")));
                }
            }
            rows::soft_delete(tx, sp, op.id, Some(ctx.user_id)).await?;
            let after = rows::get(tx, sp.name, op.id).await?.unwrap_or_default();
            write_history(tx, ctx, sp, "delete", Some(&server), &after, op.edited_at).await?;
            Ok(OpResult::Applied { ver: ver_of(&after), row: Some(after) })
        }
        OpKind::Upsert => {
            // Lọc cột: bỏ cột hệ thống; cột lạ → từ chối; cột chỉ online → từ chối khi offline và có đổi.
            let mut new = Row::new();
            for (k, v) in &op.new {
                if SYSTEM_COLS.contains(&k.as_str()) || spec::is_derived(sp.name, k) {
                    continue;
                }
                if !sp.is_writable(k) {
                    return Ok(OpResult::rejected("unknown_field", format!("Cột {k} không có trong {}", sp.name)));
                }
                if offline && sp.is_online_only(k) {
                    let reference = op.base.as_ref().or(server.as_ref()).and_then(|r| r.get(k));
                    if !merge::same(reference, Some(v)) {
                        return Ok(OpResult::rejected("online_only_field", format!("Cột {k} chỉ sửa được khi có kết nối máy chủ (hệ thống hoá)")));
                    }
                    continue;
                }
                new.insert(k.clone(), v.clone());
            }
            match server {
                None => {
                    if op.base.is_some() {
                        return Ok(OpResult::rejected("missing", "Bản ghi gốc không còn trên máy chủ"));
                    }
                    if !(ctx.has_perm)(sp.perm_for(WriteOp::Insert)) {
                        return Ok(OpResult::rejected("forbidden", "Không có quyền thêm"));
                    }
                    validate::fill_defaults(sp.name, &mut new);
                    let errs = validate::normalize(sp.name, &mut new, true);
                    if !errs.is_empty() {
                        return Ok(OpResult::Rejected { reason: "validation".into(), message: errs[0].message.clone(), errors: errs });
                    }
                    if let Some(r) = check_parent(tx, ctx, sp, &new).await? {
                        return Ok(r);
                    }
                    new.insert("id".into(), json!(op.id));
                    new.insert("project_id".into(), json!(ctx.project_id));
                    new.insert("created_by".into(), json!(ctx.user_id));
                    new.insert("updated_by".into(), json!(ctx.user_id));
                    new.insert("deleted_at".into(), Value::Null);
                    let after = rows::insert(tx, sp, &new).await?;
                    write_history(tx, ctx, sp, "add", None, &after, op.edited_at).await?;
                    Ok(OpResult::Applied { ver: ver_of(&after), row: Some(after) })
                }
                Some(server) => {
                    if !server.get("deleted_at").map_or(true, Value::is_null) {
                        return Ok(OpResult::rejected("deleted_on_server", "Bản ghi đã bị xoá trên máy chủ"));
                    }
                    if !(ctx.has_perm)(sp.perm_for(WriteOp::Update)) {
                        return Ok(OpResult::rejected("forbidden", "Không có quyền sửa"));
                    }
                    let fields = history_fields(sp);
                    let sver = ver_of(&server);
                    let (patch, merged_fields) = match &op.base {
                        Some(base) if ver_of(base) != sver => match merge::three_way(&server, base, &new, &fields) {
                            merge::Merge::Conflict(f) => return Ok(OpResult::Conflict { row: Some(server), conflict_fields: f }),
                            merge::Merge::NoChange => return Ok(OpResult::NoChange { ver: sver.unwrap_or(0), row: server }),
                            merge::Merge::Apply(p) => {
                                let keys: Vec<String> = p.keys().cloned().collect();
                                (p, Some(keys))
                            }
                        },
                        Some(_) => (merge::changed_fields(&server, &new, &fields), None),
                        None => {
                            if let Some(v) = op.if_ver {
                                if Some(v) != sver {
                                    return Ok(OpResult::Conflict { row: Some(server), conflict_fields: vec![] });
                                }
                            }
                            (merge::changed_fields(&server, &new, &fields), None)
                        }
                    };
                    if patch.is_empty() {
                        return Ok(OpResult::NoChange { ver: sver.unwrap_or(0), row: server });
                    }
                    // Chuẩn hoá trên bản gộp (kiểm chéo ngày BĐ ≤ KT…), rồi chỉ giữ các cột trong patch.
                    let mut merged = server.clone();
                    for (k, v) in &patch {
                        merged.insert(k.clone(), v.clone());
                    }
                    let errs = validate::normalize(sp.name, &mut merged, false);
                    if !errs.is_empty() {
                        return Ok(OpResult::Rejected { reason: "validation".into(), message: errs[0].message.clone(), errors: errs });
                    }
                    let mut patch2 = Row::new();
                    for k in patch.keys() {
                        patch2.insert(k.clone(), merged.get(k).cloned().unwrap_or(Value::Null));
                    }
                    if let Some((col, _)) = sp.parent {
                        if patch2.contains_key(col) {
                            if let Some(r) = check_parent(tx, ctx, sp, &merged).await? {
                                return Ok(r);
                            }
                        }
                    }
                    patch2.insert("updated_by".into(), json!(ctx.user_id));
                    let after = rows::update(tx, sp, op.id, &patch2).await?;
                    write_history(tx, ctx, sp, "edit", Some(&server), &after, op.edited_at).await?;
                    let ver = ver_of(&after).unwrap_or(0);
                    Ok(match merged_fields {
                        Some(f) => OpResult::Merged { row: after, ver, merged_fields: f },
                        None => OpResult::Applied { ver: Some(ver), row: Some(after) },
                    })
                }
            }
        }
    }
}

/// Cha phải tồn tại, chưa xoá và cùng dự án (hoặc là dòng dùng chung).
async fn check_parent(tx: &mut PgConnection, ctx: &OpCtx<'_>, sp: &TableSpec, row: &Row) -> sqlx::Result<Option<OpResult>> {
    let Some((col, parent)) = sp.parent else { return Ok(None) };
    let Some(pid) = rows::uuid_of(row, col) else {
        // Cha tuỳ chọn (khoi_id) được phép trống; cha bắt buộc đã bị validate chặn.
        return Ok(None);
    };
    match rows::parent_project(tx, parent, pid).await? {
        None => Ok(Some(OpResult::rejected(&format!("fk:{col}"), format!("Bản ghi cha ({parent}) không tồn tại hoặc đã xoá")))),
        Some(Some(p)) if p != ctx.project_id => Ok(Some(OpResult::rejected("forbidden", "Bản ghi cha thuộc dự án khác"))),
        _ => Ok(None),
    }
}

/// Sắp lại lô op: cha trước con, upsert trước delete, delete con trước delete cha; cùng (bảng, id) giữ thứ tự client.
pub fn order_ops(ops: &mut [SyncOp]) {
    let rank = |t: &str| spec::spec(t).map(|s| s.rank as i32).unwrap_or(i32::MAX);
    ops.sort_by_key(|o| match o.op {
        OpKind::Upsert => (0, rank(&o.table)),
        OpKind::Delete => (1, -rank(&o.table)),
    });
}

// ------------------------------------------------------------------ pull

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRow {
    pub table: String,
    pub ver: i64,
    pub row: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pull {
    pub since: i64,
    pub next: i64,
    pub has_more: bool,
    pub full: bool,
    pub server_time: DateTime<Utc>,
    pub rows: Vec<PullRow>,
}

pub const PULL_MAX: i64 = 10_000;

/// Dòng của mọi bảng sync có `ver > since`, theo `ver` tăng. `since = 0` bỏ tombstone.
pub async fn pull(pool: &PgPool, project_id: Uuid, since: i64, limit: i64) -> sqlx::Result<Pull> {
    let limit = limit.clamp(1, PULL_MAX);
    let mut tx = pool.begin().await?;
    let pruned: i64 = sqlx::query_scalar("SELECT COALESCE(max(pruned_ver), 0) FROM cl_sync_meta").fetch_one(&mut *tx).await?;
    if since > 0 && since < pruned {
        return Ok(Pull { since, next: 0, has_more: false, full: true, server_time: Utc::now(), rows: vec![] });
    }
    sqlx::query("SELECT pg_advisory_xact_lock_shared(cl_lock_key($1)), pg_advisory_xact_lock_shared(cl_lock_key(NULL))")
        .bind(project_id)
        .execute(&mut *tx)
        .await?;
    let parts: Vec<String> = spec::PULL_TABLES
        .iter()
        .map(|t| {
            let shared = spec::spec(t).map_or(false, |s| s.shared);
            let scope = if shared { "(x.project_id = $1 OR x.project_id IS NULL)" } else { "x.project_id = $1" };
            let tomb = if *t == "cl_lich_su" { "" } else { " AND ($2::bigint > 0 OR x.deleted_at IS NULL)" };
            format!("SELECT '{t}'::text AS t, x.ver, to_jsonb(x) AS row FROM {t} x WHERE {scope} AND x.ver > $2{tomb}")
        })
        .collect();
    let sql = format!("SELECT t, ver, row FROM ({}) u ORDER BY ver LIMIT $3", parts.join(" UNION ALL "));
    let recs = sqlx::query(&sql).bind(project_id).bind(since).bind(limit + 1).fetch_all(&mut *tx).await?;
    tx.commit().await?;
    let has_more = recs.len() as i64 > limit;
    let rows: Vec<PullRow> = recs
        .into_iter()
        .take(limit as usize)
        .map(|r| PullRow { table: r.try_get("t").unwrap_or_default(), ver: r.try_get("ver").unwrap_or(0), row: r.try_get("row").unwrap_or(Value::Null) })
        .collect();
    let next = rows.last().map(|r| r.ver).unwrap_or(since);
    Ok(Pull { since, next, has_more, full: false, server_time: Utc::now(), rows })
}

/// Xoá cứng tombstone cũ hơn `days` ngày (con trước cha) và ghi mốc `pruned_ver`.
pub async fn prune(pool: &PgPool, days: i64) -> sqlx::Result<u64> {
    let mut tx = pool.begin().await?;
    lock_project(&mut tx, None).await?;
    let mut total = 0u64;
    let mut max_ver: i64 = 0;
    let mut tables: Vec<&TableSpec> = spec::TABLES.iter().collect();
    tables.sort_by_key(|t| std::cmp::Reverse(t.rank));
    for t in tables {
        let vers: Vec<i64> = sqlx::query_scalar(&format!("DELETE FROM {} WHERE deleted_at < now() - make_interval(days => $1::int) RETURNING ver", t.name))
            .bind(days as i32)
            .fetch_all(&mut *tx)
            .await?;
        total += vers.len() as u64;
        max_ver = max_ver.max(vers.into_iter().max().unwrap_or(0));
    }
    if max_ver > 0 {
        sqlx::query(
            "INSERT INTO cl_sync_meta (project_key, pruned_ver, pruned_at) VALUES ('00000000-0000-0000-0000-000000000000', $1, now()) \
             ON CONFLICT (project_key) DO UPDATE SET pruned_ver = greatest(cl_sync_meta.pruned_ver, EXCLUDED.pruned_ver), pruned_at = now()",
        )
        .bind(max_ver)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM cl_sync_ops WHERE at < now() - make_interval(days => $1::int)").bind(days as i32).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(table: &str, kind: OpKind) -> SyncOp {
        SyncOp { op_id: None, table: table.into(), id: Uuid::now_v7(), op: kind, base: None, new: Row::new(), if_ver: None, edited_at: None }
    }

    #[test]
    fn sap_op_cha_truoc_con_xoa_sau() {
        let mut ops = vec![
            op("cl_van_ban", OpKind::Delete),
            op("cl_van_ban", OpKind::Upsert),
            op("cl_ho_so", OpKind::Delete),
            op("cl_phong", OpKind::Upsert),
            op("cl_ho_so", OpKind::Upsert),
        ];
        order_ops(&mut ops);
        let seq: Vec<(String, OpKind)> = ops.iter().map(|o| (o.table.clone(), o.op)).collect();
        assert_eq!(
            seq,
            vec![
                ("cl_phong".to_string(), OpKind::Upsert),
                ("cl_ho_so".to_string(), OpKind::Upsert),
                ("cl_van_ban".to_string(), OpKind::Upsert),
                ("cl_van_ban".to_string(), OpKind::Delete),
                ("cl_ho_so".to_string(), OpKind::Delete),
            ]
        );
    }

    #[test]
    fn thong_diep_khoa_duy_nhat() {
        assert_eq!(unique_message("ux_cl_ho_so_so_tam"), "Số tạm đã có trong phông (cùng loại vĩnh viễn / có thời hạn)");
        assert!(unique_message("abc").contains("abc"));
    }
}
