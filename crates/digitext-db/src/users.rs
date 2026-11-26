// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use digitext_core::Role;
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub display_name: String,
    #[sqlx(try_from = "String")]
    pub role: Role,
    /// Vai trò động (bảng roles, migration 0022); None = dùng đúng vai trò gốc `role`.
    pub role_code: Option<String>,
    pub project_id: Option<Uuid>,
    /// Phòng ban (đơn vị hình thành phông) của người dùng — lưu trữ hiện hành 0031; None = không giới hạn phạm vi.
    pub phong_ban_id: Option<Uuid>,
    pub is_active: bool,
    pub token_version: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
const USER_COLS: &str = "id, username, email, display_name, role, role_code, project_id, phong_ban_id, \
     is_active, token_version, created_by, created_at, updated_at";
#[derive(Debug)]
pub struct Te1<'a> {
    pub username: &'a str,
    pub email: Option<&'a str>,
    pub display_name: &'a str,
    pub pwd_hash: &'a str,
    pub role: Role,
    pub role_code: Option<&'a str>,
    pub project_id: Option<Uuid>,
    pub phong_ban_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
}
#[derive(Debug, Default)]
pub struct Tph<'a> {
    pub display_name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub is_active: Option<bool>,
    /// `Some(None)` = bỏ gắn phòng ban; `None` = không đổi.
    pub phong_ban_id: Option<Option<Uuid>>,
}
pub async fn create(r_2ex: &PgPool, r_2ew: Te1<'_>) -> sqlx::Result<User> {
    sqlx::query_as::<
        _,
        User,
    >(
            &format!(
                "INSERT INTO users (id, username, email, display_name, pwd_hash, role, role_code, project_id, created_by, phong_ban_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING {USER_COLS}"
            ),
        )
        .bind(Uuid::now_v7())
        .bind(r_2ew.username)
        .bind(r_2ew.email)
        .bind(r_2ew.display_name)
        .bind(r_2ew.pwd_hash)
        .bind(r_2ew.role.as_str())
        .bind(r_2ew.role_code)
        .bind(r_2ew.project_id)
        .bind(r_2ew.created_by)
        .bind(r_2ew.phong_ban_id)
        .fetch_one(r_2ex)
        .await
}
/// Đổi vai trò (gốc + mã vai trò động). Tăng token_version để mọi phiên cũ phải đăng nhập lại và nhận bộ quyền mới.
pub async fn set_role(pool: &PgPool, id: Uuid, role: Role, role_code: Option<&str>) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(&format!(
        "UPDATE users SET role = $2, role_code = $3, token_version = token_version + 1, updated_at = now() WHERE id = $1 RETURNING {USER_COLS}"
    ))
    .bind(id)
    .bind(role.as_str())
    .bind(role_code)
    .fetch_optional(pool)
    .await
}
pub async fn f_1eg(r_2ey: &PgPool, id: Uuid) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(&format!("SELECT {USER_COLS} FROM users WHERE id = $1"))
        .bind(id)
        .fetch_optional(r_2ey)
        .await
}
pub async fn f_1eh(r_2ez: &PgPool, username: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLS} FROM users WHERE username = $1"
    ))
    .bind(username)
    .fetch_optional(r_2ez)
    .await
}
#[derive(sqlx::FromRow)]
struct T3k {
    #[sqlx(flatten)]
    user: User,
    pwd_hash: String,
}
pub async fn f_1ej(r_2f0: &PgPool, username: &str) -> sqlx::Result<Option<(User, String)>> {
    let r_2f2 = sqlx::query_as::<_, T3k>(&format!(
        "SELECT {USER_COLS}, pwd_hash FROM users WHERE username = $1"
    ))
    .bind(username)
    .fetch_optional(r_2f0)
    .await?;
    Ok(r_2f2.map(|r_2f1| (r_2f1.user, r_2f1.pwd_hash)))
}
pub async fn list(
    r_2f3: &PgPool,
    project_id: Option<Uuid>,
    role: Option<Role>,
) -> sqlx::Result<Vec<User>> {
    sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLS} FROM users \
         WHERE ($1::uuid IS NULL OR project_id = $1) \
           AND ($2::text IS NULL OR role = $2) \
         ORDER BY username"
    ))
    .bind(project_id)
    .bind(role.map(Role::as_str))
    .fetch_all(r_2f3)
    .await
}
pub async fn update(r_2f5: &PgPool, id: Uuid, r_2f4: Tph<'_>) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(&format!(
        "UPDATE users SET \
            display_name = COALESCE($2, display_name), \
            email        = COALESCE($3, email), \
            is_active    = COALESCE($4, is_active), \
            phong_ban_id = CASE WHEN $5::bool THEN $6 ELSE phong_ban_id END, \
            updated_at   = now() \
         WHERE id = $1 RETURNING {USER_COLS}"
    ))
    .bind(id)
    .bind(r_2f4.display_name)
    .bind(r_2f4.email)
    .bind(r_2f4.is_active)
    .bind(r_2f4.phong_ban_id.is_some())
    .bind(r_2f4.phong_ban_id.flatten())
    .fetch_optional(r_2f5)
    .await
}
pub async fn f_2e8(r_2f6: &PgPool, id: Uuid, pwd_hash: &str) -> sqlx::Result<Option<i32>> {
    sqlx::query_scalar::<_, i32>(
        "UPDATE users SET pwd_hash = $2, token_version = token_version + 1, updated_at = now() \
         WHERE id = $1 RETURNING token_version",
    )
    .bind(id)
    .bind(pwd_hash)
    .fetch_optional(r_2f6)
    .await
}
pub async fn f_wi(r_2f7: &PgPool, id: Uuid) -> sqlx::Result<Option<i32>> {
    sqlx::query_scalar::<_, i32>(
        "UPDATE users SET token_version = token_version + 1, updated_at = now() \
         WHERE id = $1 RETURNING token_version",
    )
    .bind(id)
    .fetch_optional(r_2f7)
    .await
}
pub async fn list_in_projects(
    r_2f8: &PgPool,
    r_2f9: &[Uuid],
    role: Option<Role>,
) -> sqlx::Result<Vec<User>> {
    sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLS} FROM users \
         WHERE project_id = ANY($1) \
           AND ($2::text IS NULL OR role = $2) \
         ORDER BY username"
    ))
    .bind(r_2f9)
    .bind(role.map(Role::as_str))
    .fetch_all(r_2f8)
    .await
}
pub async fn f_2ec(
    r_2fa: &PgPool,
    id: Uuid,
    project_id: Option<Uuid>,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(&format!(
        "UPDATE users SET project_id = $2, updated_at = now() WHERE id = $1 RETURNING {USER_COLS}"
    ))
    .bind(id)
    .bind(project_id)
    .fetch_optional(r_2fa)
    .await
}
pub async fn count(r_2fb: &PgPool) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(r_2fb)
        .await
}

/// Người dùng đang hoạt động của một phòng ban trong dự án (nhận nhắc hạn nộp lưu — 0031).
pub async fn cua_phong_ban(pool: &PgPool, project_id: Uuid, phong_ban_id: Uuid) -> sqlx::Result<Vec<Uuid>> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE phong_ban_id = $2 AND is_active AND (project_id = $1 OR project_id IS NULL) ORDER BY username",
    )
    .bind(project_id)
    .bind(phong_ban_id)
    .fetch_all(pool)
    .await
}
