// Copyright (c) 2026 DigiText. All rights reserved.
//! Webhook nhận dự án từ hệ thống ngoài (migration 0028): khoá gọi, nhật ký nhận,
//! và các truy vấn dự án có thêm `loai` / `nguon` / `ma_ngoai`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Một hệ thống ngoài được phép gọi webhook.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ApiClient {
    pub id: Uuid,
    pub nguon: String,
    pub ten: String,
    pub key_id: String,
    pub secret_hash: String,
    /// Bí mật ký HMAC; None = chỉ cần khoá đúng, không bắt ký.
    pub secret_hmac: Option<String>,
    pub default_admin_id: Option<Uuid>,
    pub ip_allow: Option<Vec<String>>,
    pub is_active: bool,
    pub ghi_chu: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

const CLIENT_COLS: &str = "id, nguon, ten, key_id, secret_hash, secret_hmac, default_admin_id, \
     ip_allow, is_active, ghi_chu, created_at, last_seen_at";

pub async fn tim_client(pool: &PgPool, key_id: &str) -> sqlx::Result<Option<ApiClient>> {
    sqlx::query_as::<_, ApiClient>(&format!(
        "SELECT {CLIENT_COLS} FROM api_clients WHERE key_id = $1"
    ))
    .bind(key_id)
    .fetch_optional(pool)
    .await
}

pub async fn danh_sach_client(pool: &PgPool) -> sqlx::Result<Vec<ApiClient>> {
    sqlx::query_as::<_, ApiClient>(&format!(
        "SELECT {CLIENT_COLS} FROM api_clients ORDER BY nguon"
    ))
    .fetch_all(pool)
    .await
}

pub struct ClientMoi<'a> {
    pub nguon: &'a str,
    pub ten: &'a str,
    pub key_id: &'a str,
    pub secret_hash: &'a str,
    pub secret_hmac: Option<&'a str>,
    pub default_admin_id: Option<Uuid>,
    pub ip_allow: Option<Vec<String>>,
}

pub async fn tao_client(pool: &PgPool, c: ClientMoi<'_>) -> sqlx::Result<ApiClient> {
    sqlx::query_as::<_, ApiClient>(&format!(
        "INSERT INTO api_clients (id, nguon, ten, key_id, secret_hash, secret_hmac, default_admin_id, ip_allow) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING {CLIENT_COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(c.nguon)
    .bind(c.ten)
    .bind(c.key_id)
    .bind(c.secret_hash)
    .bind(c.secret_hmac)
    .bind(c.default_admin_id)
    .bind(c.ip_allow)
    .fetch_one(pool)
    .await
}

/// Đổi khoá của một nguồn đã có (cấp lại khi bên kia làm lộ khoá).
pub async fn doi_khoa_client(
    pool: &PgPool,
    nguon: &str,
    key_id: &str,
    secret_hash: &str,
    secret_hmac: Option<&str>,
) -> sqlx::Result<Option<ApiClient>> {
    sqlx::query_as::<_, ApiClient>(&format!(
        "UPDATE api_clients SET key_id = $2, secret_hash = $3, secret_hmac = $4, is_active = true \
         WHERE nguon = $1 RETURNING {CLIENT_COLS}"
    ))
    .bind(nguon)
    .bind(key_id)
    .bind(secret_hash)
    .bind(secret_hmac)
    .fetch_optional(pool)
    .await
}

pub async fn thu_hoi_client(pool: &PgPool, nguon: &str) -> sqlx::Result<bool> {
    let r = sqlx::query("UPDATE api_clients SET is_active = false WHERE nguon = $1")
        .bind(nguon)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

/// Ghi dấu "nguồn này vừa gọi" — chỉ để người quản trị biết kênh còn sống.
pub async fn ghi_da_goi(pool: &PgPool, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE api_clients SET last_seen_at = now() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Dự án kèm các cột của 0028.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DuAn {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub loai: Option<String>,
    pub nguon: Option<String>,
    pub ma_ngoai: Option<String>,
    pub ma_tu_sinh: bool,
    /// Tên cơ quan chủ đầu tư (0029, 11/11/2026). Tách khỏi `name` vì gần như mọi dự án lưu trữ
    /// mang cùng một nội dung công việc, danh sách hoá ra toàn dòng giống nhau.
    pub chu_dau_tu: Option<String>,
}

const DU_AN_COLS: &str =
    "id, code, name, is_active, created_at, loai, nguon, ma_ngoai, ma_tu_sinh, chu_dau_tu";

pub async fn tim_theo_ma_ngoai(
    pool: &PgPool,
    nguon: &str,
    ma_ngoai: &str,
) -> sqlx::Result<Option<DuAn>> {
    sqlx::query_as::<_, DuAn>(&format!(
        "SELECT {DU_AN_COLS} FROM projects WHERE nguon = $1 AND ma_ngoai = $2"
    ))
    .bind(nguon)
    .bind(ma_ngoai)
    .fetch_optional(pool)
    .await
}

pub async fn tim_theo_ma(pool: &PgPool, code: &str) -> sqlx::Result<Option<DuAn>> {
    sqlx::query_as::<_, DuAn>(&format!(
        "SELECT {DU_AN_COLS} FROM projects WHERE upper(code) = upper($1)"
    ))
    .bind(code)
    .fetch_optional(pool)
    .await
}

pub struct DuAnMoi<'a> {
    pub code: &'a str,
    pub name: &'a str,
    pub loai: Option<&'a str>,
    pub nguon: &'a str,
    pub ma_ngoai: &'a str,
    pub ma_tu_sinh: bool,
    pub chu_dau_tu: Option<&'a str>,
}

pub async fn tao_du_an(pool: &PgPool, d: DuAnMoi<'_>) -> sqlx::Result<DuAn> {
    sqlx::query_as::<_, DuAn>(&format!(
        "INSERT INTO projects (id, code, name, loai, nguon, ma_ngoai, ma_tu_sinh, chu_dau_tu) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING {DU_AN_COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(d.code)
    .bind(d.name)
    .bind(d.loai)
    .bind(d.nguon)
    .bind(d.ma_ngoai)
    .bind(d.ma_tu_sinh)
    .bind(d.chu_dau_tu)
    .fetch_one(pool)
    .await
}

/// Cập nhật dự án do webhook gửi lại. Trường nào gửi None thì giữ nguyên.
/// `nguon`/`ma_ngoai` chỉ ghi khi dòng cũ còn trống — nhận dự án người ta đã tạo tay.
pub async fn cap_nhat_du_an(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    loai: Option<&str>,
    is_active: Option<bool>,
    nguon: &str,
    ma_ngoai: &str,
    chu_dau_tu: Option<&str>,
) -> sqlx::Result<Option<DuAn>> {
    sqlx::query_as::<_, DuAn>(&format!(
        "UPDATE projects SET \
           name = COALESCE($2, name), \
           loai = COALESCE($3, loai), \
           is_active = COALESCE($4, is_active), \
           nguon = COALESCE(nguon, $5), \
           ma_ngoai = COALESCE(ma_ngoai, $6), \
           chu_dau_tu = COALESCE($7, chu_dau_tu) \
         WHERE id = $1 RETURNING {DU_AN_COLS}"
    ))
    .bind(id)
    .bind(name)
    .bind(loai)
    .bind(is_active)
    .bind(nguon)
    .bind(ma_ngoai)
    .bind(chu_dau_tu)
    .fetch_optional(pool)
    .await
}

/// Mã tự sinh kế tiếp theo tiền tố (ví dụ "DA26" → "DA26001"). Đọc mã đã có rồi
/// tính trong Rust, không nhồi regex vào SQL.
pub async fn ma_tu_sinh_ke_tiep(pool: &PgPool, tien_to: &str) -> sqlx::Result<String> {
    let da_co: Vec<String> = sqlx::query_scalar("SELECT code FROM projects WHERE code LIKE $1")
        .bind(format!("{tien_to}%"))
        .fetch_all(pool)
        .await?;
    let lon_nhat = da_co
        .iter()
        .filter_map(|c| c.strip_prefix(tien_to))
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
        .filter_map(|s| s.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    Ok(format!("{tien_to}{:03}", lon_nhat + 1))
}

/// Gán quản trị cho dự án; bỏ qua người đã là quản trị. Trả về số dòng thêm mới.
pub async fn gan_quan_tri(
    pool: &PgPool,
    project_id: Uuid,
    admin_ids: &[Uuid],
) -> sqlx::Result<u64> {
    if admin_ids.is_empty() {
        return Ok(0);
    }
    let r = sqlx::query(
        "INSERT INTO project_admins (project_id, admin_id) \
         SELECT $1, u FROM UNNEST($2::uuid[]) AS u \
         ON CONFLICT (project_id, admin_id) DO NOTHING",
    )
    .bind(project_id)
    .bind(admin_ids)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

/// Tìm người dùng theo tên đăng nhập HOẶC email, chỉ lấy tài khoản còn hoạt động.
pub async fn tim_nguoi_de_gan(pool: &PgPool, khoa: &[String]) -> sqlx::Result<Vec<(Uuid, String)>> {
    sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, username FROM users \
         WHERE is_active AND (lower(username) = ANY($1) OR lower(email) = ANY($1))",
    )
    .bind(
        khoa.iter()
            .map(|k| k.trim().to_lowercase())
            .collect::<Vec<_>>(),
    )
    .fetch_all(pool)
    .await
}

/// Hợp đồng hiện tại của dự án — webhook gộp giá trị mới lên giá trị cũ, không xoá
/// trường người ta đã nhập tay trong app.
pub async fn lay_hop_dong(
    pool: &PgPool,
    project_id: Uuid,
) -> sqlx::Result<Option<crate::progress::Contract>> {
    sqlx::query_as::<_, crate::progress::Contract>(&format!(
        "SELECT {} FROM project_contracts WHERE project_id = $1",
        crate::progress::CONTRACT_COLS
    ))
    .bind(project_id)
    .fetch_optional(pool)
    .await
}

/// Tên đăng nhập theo id — để phản hồi webhook nói rõ đã gán ai làm quản trị.
pub async fn ten_nguoi(pool: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<(Uuid, String)>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as::<_, (Uuid, String)>("SELECT id, username FROM users WHERE id = ANY($1)")
        .bind(ids)
        .fetch_all(pool)
        .await
}

pub struct Delivery<'a> {
    pub client_id: Option<Uuid>,
    pub nguon: Option<&'a str>,
    pub event: Option<&'a str>,
    pub ma_ngoai: Option<&'a str>,
    pub payload: Option<serde_json::Value>,
    pub status: i32,
    pub ket_qua: Option<serde_json::Value>,
    pub loi: Option<&'a str>,
    pub project_id: Option<Uuid>,
    pub ip: Option<&'a str>,
}

pub async fn ghi_delivery(pool: &PgPool, d: Delivery<'_>) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO webhook_deliveries \
           (client_id, nguon, event, ma_ngoai, payload, status, ket_qua, loi, project_id, ip) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(d.client_id)
    .bind(d.nguon)
    .bind(d.event)
    .bind(d.ma_ngoai)
    .bind(d.payload)
    .bind(d.status)
    .bind(d.ket_qua)
    .bind(d.loi)
    .bind(d.project_id)
    .bind(d.ip)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DeliveryRow {
    pub id: i64,
    pub nguon: Option<String>,
    pub event: Option<String>,
    pub ma_ngoai: Option<String>,
    pub status: i32,
    pub loi: Option<String>,
    pub project_id: Option<Uuid>,
    pub ip: Option<String>,
    pub nhan_luc: DateTime<Utc>,
    pub payload: Option<serde_json::Value>,
    pub ket_qua: Option<serde_json::Value>,
}

pub async fn danh_sach_delivery(
    pool: &PgPool,
    nguon: Option<&str>,
    limit: i64,
) -> sqlx::Result<Vec<DeliveryRow>> {
    sqlx::query_as::<_, DeliveryRow>(
        "SELECT id, nguon, event, ma_ngoai, status, loi, project_id, ip, nhan_luc, payload, ket_qua \
         FROM webhook_deliveries \
         WHERE ($1::text IS NULL OR nguon = $1) \
         ORDER BY nhan_luc DESC LIMIT $2",
    )
    .bind(nguon)
    .bind(limit.clamp(1, 500))
    .fetch_all(pool)
    .await
}
