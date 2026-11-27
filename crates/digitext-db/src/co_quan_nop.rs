// Copyright (c) 2026 DigiText. All rights reserved.
// Cơ quan nộp lưu vào lưu trữ lịch sử (0040, Digifile D11 / D12): danh mục cơ quan, tài khoản gắn cơ quan (users.co_quan_nop_id,
// chỉ thấy đợt nộp của mình) và khoá API (SHA-256 hex) để hệ thống của cơ quan nộp gọi /submissions bằng `X-Api-Key`.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoQuanNopRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub ma: String,
    pub ten: String,
    pub dia_chi: Option<String>,
    pub nguoi_lien_he: Option<String>,
    pub dien_thoai: Option<String>,
    pub email: Option<String>,
    pub ghi_chu: Option<String>,
    pub co_api_key: bool,
    pub api_key_tao_at: Option<DateTime<Utc>>,
    pub api_key_goi_at: Option<DateTime<Utc>>,
    pub so_tai_khoan: i64,
    pub so_dot_nop: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

const COLS: &str = "c.id, c.project_id, c.ma, c.ten, c.dia_chi, c.nguoi_lien_he, c.dien_thoai, c.email, c.ghi_chu, \
     (c.api_key_hash IS NOT NULL) AS co_api_key, c.api_key_tao_at, c.api_key_goi_at, \
     (SELECT count(*) FROM users u WHERE u.co_quan_nop_id = c.id AND u.is_active) AS so_tai_khoan, \
     (SELECT count(*) FROM submissions s WHERE s.co_quan_nop_id = c.id) AS so_dot_nop, \
     c.created_at, c.updated_at";

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Moi {
    pub ma: String,
    pub ten: String,
    #[serde(default)]
    pub dia_chi: Option<String>,
    #[serde(default)]
    pub nguoi_lien_he: Option<String>,
    #[serde(default)]
    pub dien_thoai: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub ghi_chu: Option<String>,
}

fn sach(v: &Option<String>) -> Option<&str> {
    v.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

pub async fn list(pool: &PgPool, project_id: Uuid) -> sqlx::Result<Vec<CoQuanNopRow>> {
    sqlx::query_as::<_, CoQuanNopRow>(&format!(
        "SELECT {COLS} FROM co_quan_nop c WHERE c.project_id = $1 AND c.deleted_at IS NULL ORDER BY c.ma"
    ))
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<CoQuanNopRow>> {
    sqlx::query_as::<_, CoQuanNopRow>(&format!("SELECT {COLS} FROM co_quan_nop c WHERE c.id = $1 AND c.deleted_at IS NULL"))
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn tao(pool: &PgPool, project_id: Uuid, m: &Moi, boi: Uuid) -> sqlx::Result<CoQuanNopRow> {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO co_quan_nop (id, project_id, ma, ten, dia_chi, nguoi_lien_he, dien_thoai, email, ghi_chu, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(id)
    .bind(project_id)
    .bind(m.ma.trim().to_uppercase())
    .bind(m.ten.trim())
    .bind(sach(&m.dia_chi))
    .bind(sach(&m.nguoi_lien_he))
    .bind(sach(&m.dien_thoai))
    .bind(sach(&m.email))
    .bind(sach(&m.ghi_chu))
    .bind(boi)
    .execute(pool)
    .await?;
    Ok(get(pool, id).await?.expect("vừa chèn"))
}

pub async fn sua(pool: &PgPool, id: Uuid, m: &Moi) -> sqlx::Result<Option<CoQuanNopRow>> {
    let n = sqlx::query(
        "UPDATE co_quan_nop SET ma = $2, ten = $3, dia_chi = $4, nguoi_lien_he = $5, dien_thoai = $6, email = $7, ghi_chu = $8, \
            updated_at = now() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(m.ma.trim().to_uppercase())
    .bind(m.ten.trim())
    .bind(sach(&m.dia_chi))
    .bind(sach(&m.nguoi_lien_he))
    .bind(sach(&m.dien_thoai))
    .bind(sach(&m.email))
    .bind(sach(&m.ghi_chu))
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 {
        return Ok(None);
    }
    get(pool, id).await
}

/// Xoá mềm; tài khoản gắn cơ quan được gỡ liên kết, đợt nộp cũ giữ nguyên tham chiếu (ON DELETE SET NULL chỉ khi xoá cứng).
pub async fn xoa(pool: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE users SET co_quan_nop_id = NULL WHERE co_quan_nop_id = $1").bind(id).execute(&mut *tx).await?;
    let n = sqlx::query("UPDATE co_quan_nop SET deleted_at = now(), api_key_hash = NULL, updated_at = now() WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    tx.commit().await?;
    Ok(n > 0)
}

/// Ghi băm khoá API mới (khoá thật chỉ trả về một lần ở tầng API). Trả false nếu cơ quan không tồn tại.
pub async fn dat_api_key(pool: &PgPool, id: Uuid, hash_hex: Option<&str>) -> sqlx::Result<bool> {
    let n = sqlx::query(
        "UPDATE co_quan_nop SET api_key_hash = $2, api_key_tao_at = CASE WHEN $2 IS NULL THEN NULL ELSE now() END, updated_at = now() \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(hash_hex)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(n > 0)
}

/// Tra cơ quan theo băm khoá API, kèm tài khoản đại diện (người dùng đang hoạt động gắn cơ quan, ưu tiên tạo sớm nhất).
pub async fn theo_api_key(pool: &PgPool, hash_hex: &str) -> sqlx::Result<Option<(Uuid, Uuid, Option<Uuid>)>> {
    let r: Option<(Uuid, Uuid, Option<Uuid>)> = sqlx::query_as(
        "SELECT c.id, c.project_id, \
                (SELECT u.id FROM users u WHERE u.co_quan_nop_id = c.id AND u.is_active ORDER BY u.created_at LIMIT 1) \
         FROM co_quan_nop c WHERE c.api_key_hash = $1 AND c.deleted_at IS NULL",
    )
    .bind(hash_hex)
    .fetch_optional(pool)
    .await?;
    if r.is_some() {
        sqlx::query("UPDATE co_quan_nop SET api_key_goi_at = now() WHERE api_key_hash = $1")
            .bind(hash_hex)
            .execute(pool)
            .await?;
    }
    Ok(r)
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaiKhoan {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub is_active: bool,
}

pub async fn tai_khoan(pool: &PgPool, id: Uuid) -> sqlx::Result<Vec<TaiKhoan>> {
    sqlx::query_as::<_, TaiKhoan>(
        "SELECT id, username, display_name, is_active FROM users WHERE co_quan_nop_id = $1 ORDER BY username",
    )
    .bind(id)
    .fetch_all(pool)
    .await
}
