// Copyright (c) 2026 DigiText. All rights reserved.
//! Đọc / ghi một dòng cl_* dưới dạng JSON (snake_case đúng tên cột). Ghi dùng
//! `jsonb_populate_record` để Postgres tự ép kiểu theo cột; cột ghi lấy từ `TableSpec` (không bao
//! giờ ghi cột generated / ver / *_at).

use super::merge::Row;
use super::spec::TableSpec;
use serde_json::Value;
use sqlx::{PgConnection, Row as _};
use uuid::Uuid;

fn ident_ok(t: &str) -> bool {
    t.starts_with("cl_") && t.chars().all(|c| c.is_ascii_lowercase() || c == '_')
}

pub async fn get(conn: &mut PgConnection, table: &str, id: Uuid) -> sqlx::Result<Option<Row>> {
    debug_assert!(ident_ok(table));
    let r = sqlx::query(&format!("SELECT to_jsonb(t) AS row FROM {table} t WHERE id = $1"))
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(r.and_then(|r| r.try_get::<Value, _>("row").ok()).and_then(|v| v.as_object().cloned()))
}

pub async fn get_for_update(conn: &mut PgConnection, table: &str, id: Uuid) -> sqlx::Result<Option<Row>> {
    debug_assert!(ident_ok(table));
    let r = sqlx::query(&format!("SELECT to_jsonb(t) AS row FROM {table} t WHERE id = $1 FOR UPDATE"))
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(r.and_then(|r| r.try_get::<Value, _>("row").ok()).and_then(|v| v.as_object().cloned()))
}

/// INSERT đủ cột (hệ thống + nghiệp vụ); cột vắng trong JSON → NULL / DEFAULT không áp dụng
/// (jsonb_populate_record cho NULL) — nên caller phải điền `trung_thua`, `vinh_vien`… nếu cần.
pub async fn insert(conn: &mut PgConnection, spec: &TableSpec, row: &Row) -> sqlx::Result<Row> {
    let cols = spec.insert_cols();
    let list = cols.join(", ");
    let sql = format!(
        "INSERT INTO {t} AS t ({list}) SELECT {list} FROM jsonb_populate_record(NULL::{t}, $1) RETURNING to_jsonb(t) AS row",
        t = spec.name
    );
    let r = sqlx::query(&sql).bind(Value::Object(row.clone())).fetch_one(&mut *conn).await?;
    Ok(r.try_get::<Value, _>("row")?.as_object().cloned().unwrap_or_default())
}

/// UPDATE các cột có trong `patch` (cột hệ thống chỉ `updated_by`, `deleted_at`). Trả dòng mới.
pub async fn update(conn: &mut PgConnection, spec: &TableSpec, id: Uuid, patch: &Row) -> sqlx::Result<Row> {
    let cols: Vec<&str> = spec.update_cols().into_iter().filter(|c| patch.contains_key(*c)).collect();
    if cols.is_empty() {
        return get(conn, spec.name, id).await.map(|r| r.unwrap_or_default());
    }
    let list = cols.join(", ");
    let sql = format!(
        "UPDATE {t} AS t SET ({list}) = (SELECT {list} FROM jsonb_populate_record(NULL::{t}, $2)) WHERE id = $1 RETURNING to_jsonb(t) AS row",
        t = spec.name
    );
    let r = sqlx::query(&sql).bind(id).bind(Value::Object(patch.clone())).fetch_one(&mut *conn).await?;
    Ok(r.try_get::<Value, _>("row")?.as_object().cloned().unwrap_or_default())
}

/// Xoá mềm một dòng và mọi con (đệ quy theo `spec.children()`); trả số dòng đã đánh dấu.
pub async fn soft_delete(conn: &mut PgConnection, spec: &TableSpec, id: Uuid, by: Option<Uuid>) -> sqlx::Result<u64> {
    let mut n = sqlx::query(&format!("UPDATE {} SET deleted_at = now(), updated_by = $2 WHERE id = $1 AND deleted_at IS NULL", spec.name))
        .bind(id)
        .bind(by)
        .execute(&mut *conn)
        .await?
        .rows_affected();
    for child in spec.children() {
        let Some((col, _)) = child.parent else { continue };
        let ids: Vec<Uuid> = sqlx::query_scalar(&format!("SELECT id FROM {} WHERE {col} = $1 AND deleted_at IS NULL", child.name))
            .bind(id)
            .fetch_all(&mut *conn)
            .await?;
        for cid in ids {
            n += Box::pin(soft_delete(conn, child, cid, by)).await?;
        }
    }
    Ok(n)
}

/// Khôi phục dòng đã xoá mềm và con bị xoá cùng lúc (deleted_at trong ±2 giây).
pub async fn restore(conn: &mut PgConnection, spec: &TableSpec, id: Uuid, by: Option<Uuid>) -> sqlx::Result<u64> {
    let at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar(&format!("SELECT deleted_at FROM {} WHERE id = $1", spec.name)).bind(id).fetch_optional(&mut *conn).await?.flatten();
    let Some(at) = at else { return Ok(0) };
    let mut n = sqlx::query(&format!("UPDATE {} SET deleted_at = NULL, updated_by = $2 WHERE id = $1", spec.name))
        .bind(id)
        .bind(by)
        .execute(&mut *conn)
        .await?
        .rows_affected();
    for child in spec.children() {
        let Some((col, _)) = child.parent else { continue };
        let ids: Vec<Uuid> = sqlx::query_scalar(&format!(
            "SELECT id FROM {} WHERE {col} = $1 AND deleted_at IS NOT NULL AND abs(extract(epoch FROM (deleted_at - $2::timestamptz))) < 2",
            child.name
        ))
        .bind(id)
        .bind(at)
        .fetch_all(&mut *conn)
        .await?;
        for cid in ids {
            n += Box::pin(restore(conn, child, cid, by)).await?;
        }
    }
    Ok(n)
}

/// Còn bản ghi sống nào tham chiếu tới dòng này không (chặn xoá danh mục đang dùng).
pub async fn live_refs(conn: &mut PgConnection, spec: &TableSpec, id: Uuid) -> sqlx::Result<u64> {
    let mut n = 0u64;
    for child in spec.children() {
        let Some((col, _)) = child.parent else { continue };
        let c: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {} WHERE {col} = $1 AND deleted_at IS NULL", child.name))
            .bind(id)
            .fetch_one(&mut *conn)
            .await?;
        n += c as u64;
    }
    // Tham chiếu không phải "cha" (thbq / mang / phong_ban / hop / gia trên hồ sơ).
    let extra: &[(&str, &str)] = match spec.name {
        "cl_thbq" => &[("cl_ho_so", "thbq_id")],
        "cl_mang" => &[("cl_ho_so", "mang_id")],
        "cl_phong_ban" => &[("cl_ho_so", "phong_ban_id"), ("cl_gia", "phong_ban_id")],
        "cl_hop" => &[("cl_ho_so", "hop_id")],
        "cl_bo_tap" => &[("cl_ho_so", "bo_id"), ("cl_ho_so", "tap_id")],
        "cl_gia" => &[("cl_ho_so", "gia_id")],
        "cl_khoi_phong" => &[("cl_phong", "khoi_id")],
        _ => &[],
    };
    for (t, col) in extra {
        let c: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {t} WHERE {col} = $1 AND deleted_at IS NULL"))
            .bind(id)
            .fetch_one(&mut *conn)
            .await?;
        n += c as u64;
    }
    Ok(n)
}

/// `project_id` của dòng cha (kiểm tra con cùng dự án). `None` nếu cha không tồn tại / đã xoá.
pub async fn parent_project(conn: &mut PgConnection, parent_table: &str, parent_id: Uuid) -> sqlx::Result<Option<Option<Uuid>>> {
    debug_assert!(ident_ok(parent_table));
    let r: Option<Option<Uuid>> = sqlx::query_scalar(&format!("SELECT project_id FROM {parent_table} WHERE id = $1 AND deleted_at IS NULL"))
        .bind(parent_id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(r)
}

pub fn uuid_of(row: &Row, f: &str) -> Option<Uuid> {
    row.get(f).and_then(Value::as_str).and_then(|s| Uuid::parse_str(s).ok())
}

pub fn i64_of(row: &Row, f: &str) -> Option<i64> {
    match row.get(f) {
        Some(Value::Number(n)) => n.as_i64(),
        Some(Value::String(s)) => s.trim().parse().ok(),
        _ => None,
    }
}

pub fn str_of<'a>(row: &'a Row, f: &str) -> Option<&'a str> {
    row.get(f).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty())
}
