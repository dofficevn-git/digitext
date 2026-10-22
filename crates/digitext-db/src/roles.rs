// Copyright (c) 2026 DigiText. All rights reserved.
//! Vai trò động (migration 0022): bộ quyền của 5 vai trò hệ thống là dữ liệu sửa được; vai trò tuỳ chỉnh = tên +
//! vai trò gốc + bộ quyền. `users.role` vẫn là vai trò gốc (tầng / phạm vi), `users.role_code` trỏ tới dòng ở đây.
use chrono::{DateTime, Utc};
use digitext_core::{default_set, Permission, Role};
use sqlx::PgPool;
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RoleRow {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[sqlx(try_from = "String")]
    pub base_role: Role,
    pub is_system: bool,
    pub sort_order: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RoleFull {
    pub row: RoleRow,
    pub permissions: BTreeSet<Permission>,
    pub user_count: i64,
}

const COLS: &str = "code, name, description, base_role, is_system, sort_order, updated_at";

fn parse_perms(rows: Vec<String>) -> BTreeSet<Permission> {
    rows.into_iter().filter_map(|p| p.parse().ok()).collect()
}

pub async fn list(pool: &PgPool) -> sqlx::Result<Vec<RoleFull>> {
    let rows = sqlx::query_as::<_, RoleRow>(&format!("SELECT {COLS} FROM roles ORDER BY is_system DESC, sort_order, name"))
        .fetch_all(pool)
        .await?;
    let perms: Vec<(String, String)> = sqlx::query_as("SELECT role_code, permission FROM role_permissions").fetch_all(pool).await?;
    let counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT COALESCE(role_code, role) AS code, count(*)::bigint FROM users WHERE is_active GROUP BY 1",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let permissions = parse_perms(perms.iter().filter(|(c, _)| *c == row.code).map(|(_, p)| p.clone()).collect());
            let user_count = counts.iter().find(|(c, _)| *c == row.code).map(|(_, n)| *n).unwrap_or(0);
            RoleFull { row, permissions, user_count }
        })
        .collect())
}

pub async fn get(pool: &PgPool, code: &str) -> sqlx::Result<Option<RoleRow>> {
    sqlx::query_as::<_, RoleRow>(&format!("SELECT {COLS} FROM roles WHERE code = $1"))
        .bind(code)
        .fetch_optional(pool)
        .await
}

/// Bộ quyền của một vai trò; None khi vai trò không có trong bảng (→ người gọi dùng mặc định theo vai trò gốc).
pub async fn permissions_of(pool: &PgPool, code: &str) -> sqlx::Result<Option<BTreeSet<Permission>>> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM roles WHERE code = $1)").bind(code).fetch_one(pool).await?;
    if !exists {
        return Ok(None);
    }
    let rows: Vec<String> = sqlx::query_scalar("SELECT permission FROM role_permissions WHERE role_code = $1").bind(code).fetch_all(pool).await?;
    Ok(Some(parse_perms(rows)))
}

pub async fn user_count(pool: &PgPool, code: &str) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>("SELECT count(*)::bigint FROM users WHERE role_code = $1 OR (role_code IS NULL AND role = $1)")
        .bind(code)
        .fetch_one(pool)
        .await
}

async fn set_permissions(tx: &mut sqlx::PgConnection, code: &str, perms: &BTreeSet<Permission>) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM role_permissions WHERE role_code = $1").bind(code).execute(&mut *tx).await?;
    let list: Vec<String> = perms.iter().map(|p| p.as_str().to_owned()).collect();
    sqlx::query("INSERT INTO role_permissions (role_code, permission) SELECT $1, unnest($2::text[])")
        .bind(code)
        .bind(&list)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE roles SET updated_at = now() WHERE code = $1").bind(code).execute(&mut *tx).await?;
    Ok(())
}

pub async fn create(pool: &PgPool, code: &str, name: &str, description: Option<&str>, base: Role, perms: &BTreeSet<Permission>, by: Option<Uuid>) -> sqlx::Result<RoleRow> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as::<_, RoleRow>(&format!(
        "INSERT INTO roles (code, name, description, base_role, is_system, sort_order, created_by) VALUES ($1, $2, $3, $4, false, 200, $5) RETURNING {COLS}"
    ))
    .bind(code)
    .bind(name)
    .bind(description)
    .bind(base.as_str())
    .bind(by)
    .fetch_one(&mut *tx)
    .await?;
    set_permissions(&mut tx, code, perms).await?;
    tx.commit().await?;
    Ok(row)
}

pub async fn update(pool: &PgPool, code: &str, name: &str, description: Option<&str>, perms: &BTreeSet<Permission>) -> sqlx::Result<bool> {
    let mut tx = pool.begin().await?;
    let n = sqlx::query("UPDATE roles SET name = $2, description = $3, updated_at = now() WHERE code = $1")
        .bind(code)
        .bind(name)
        .bind(description)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if n == 0 {
        return Ok(false);
    }
    set_permissions(&mut tx, code, perms).await?;
    tx.commit().await?;
    Ok(true)
}

/// Vai trò hệ thống về bộ quyền mặc định của vai trò gốc.
pub async fn reset_system(pool: &PgPool, code: &str) -> sqlx::Result<Option<BTreeSet<Permission>>> {
    let Some(row) = get(pool, code).await? else { return Ok(None) };
    let perms = default_set(row.base_role);
    let mut tx = pool.begin().await?;
    set_permissions(&mut tx, code, &perms).await?;
    tx.commit().await?;
    Ok(Some(perms))
}

/// Xoá vai trò tuỳ chỉnh (không xoá vai trò hệ thống). Trả false khi không có hoặc là hệ thống.
pub async fn delete(pool: &PgPool, code: &str) -> sqlx::Result<bool> {
    let n = sqlx::query("DELETE FROM roles WHERE code = $1 AND NOT is_system").bind(code).execute(pool).await?.rows_affected();
    Ok(n > 0)
}

/// Chèn 5 vai trò hệ thống còn thiếu (lúc API khởi động). Không ghi đè bộ quyền đã có người sửa.
pub async fn ensure_system(pool: &PgPool) -> sqlx::Result<u32> {
    let mut added = 0;
    for (i, role) in Role::ALL.iter().enumerate() {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM roles WHERE code = $1)").bind(role.as_str()).fetch_one(pool).await?;
        if exists {
            continue;
        }
        let mut tx = pool.begin().await?;
        sqlx::query("INSERT INTO roles (code, name, description, base_role, is_system, sort_order) VALUES ($1, $2, $3, $1, true, $4) ON CONFLICT (code) DO NOTHING")
            .bind(role.as_str())
            .bind(role.display_name())
            .bind(system_description(*role))
            .bind(i as i32 * 10)
            .execute(&mut *tx)
            .await?;
        set_permissions(&mut tx, role.as_str(), &default_set(*role)).await?;
        tx.commit().await?;
        added += 1;
    }
    Ok(added)
}

fn system_description(role: Role) -> &'static str {
    match role {
        Role::SuperAdmin => "Toàn quyền: mọi dự án, mọi tài khoản, kể cả xoá dữ liệu dự án.",
        Role::Manager => "Xem và điều hành mọi dự án, tạo quản trị dự án và nhân viên. Không được xoá.",
        Role::Admin => "Quản trị một dự án: phân việc, duyệt, ký số, đóng gói, tạo nhân viên của dự án đó.",
        Role::User => "Làm các việc trong một dự án: quét, nhập liệu, chỉnh lý, ký số khi được giao.",
        Role::Customer => "Tài khoản cũ chỉ xem; không tạo mới.",
    }
}
