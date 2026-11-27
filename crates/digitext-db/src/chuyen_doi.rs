// Copyright (c) 2026 DigiText. All rights reserved.
//! SỔ CHUYỂN ĐỔI tài liệu lưu trữ số ra giấy (0037, 27/11/2026 — Điều 13–16, Phụ lục VI TT 05/2025/TT-BNV; đợt 4 nhóm 3B).
//! Mỗi lần in một tài liệu số ra giấy (có trang thể thức "TÀI LIỆU LƯU TRỮ CHUYỂN ĐỔI") ghi một dòng; số chuyển đổi
//! `CD-{năm}-{stt:06}` theo dự án / năm (khoá tư vấn khi cấp số). PDF đã ghép lưu ở kho object theo `pdf_key`.
use chrono::{DateTime, Datelike, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row as _};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChuyenDoiRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub document_id: Option<Uuid>,
    pub ho_so_id: Option<Uuid>,
    pub van_ban_id: Option<Uuid>,
    pub phong_id: Option<Uuid>,
    pub nam: i32,
    pub so_chuyen_doi: String,
    pub ma_tai_lieu: Option<String>,
    pub ten_tai_lieu: Option<String>,
    pub ma_ho_so: Option<String>,
    pub so_trang: Option<i32>,
    pub muc_dich: Option<String>,
    pub nguoi_nhan: Option<String>,
    pub can_cu: Option<String>,
    pub ky_so_ket_luan: Option<String>,
    pub ky_so_chi_tiet: Option<String>,
    pub nguoi_thuc_hien: Option<Uuid>,
    pub chuc_vu: Option<String>,
    pub pdf_key: String,
    pub dung_luong_bytes: Option<i64>,
    pub created_at: DateTime<Utc>,
}

const COLS: &str = "id, project_id, document_id, ho_so_id, van_ban_id, phong_id, nam, so_chuyen_doi, ma_tai_lieu, ten_tai_lieu, ma_ho_so, so_trang, \
    muc_dich, nguoi_nhan, can_cu, ky_so_ket_luan, ky_so_chi_tiet, nguoi_thuc_hien, chuc_vu, pdf_key, dung_luong_bytes, created_at";

pub struct Moi<'a> {
    pub id: Uuid,
    pub project_id: Uuid,
    pub document_id: Option<Uuid>,
    pub ho_so_id: Option<Uuid>,
    pub van_ban_id: Option<Uuid>,
    pub phong_id: Option<Uuid>,
    pub ma_tai_lieu: Option<&'a str>,
    pub ten_tai_lieu: Option<&'a str>,
    pub ma_ho_so: Option<&'a str>,
    pub so_trang: Option<i32>,
    pub muc_dich: Option<&'a str>,
    pub nguoi_nhan: Option<&'a str>,
    pub can_cu: Option<&'a str>,
    pub ky_so_ket_luan: Option<&'a str>,
    pub ky_so_chi_tiet: Option<&'a str>,
    pub nguoi_thuc_hien: Uuid,
    pub chuc_vu: Option<&'a str>,
    pub pdf_key: &'a str,
    pub dung_luong_bytes: i64,
}

/// Cấp số chuyển đổi theo (dự án, năm) — gọi TRONG giao dịch đã khoá tư vấn. Trả (năm, số).
pub async fn cap_so(tx: &mut sqlx::PgConnection, project: Uuid) -> sqlx::Result<(i32, String)> {
    let nam = (Utc::now() + chrono::Duration::hours(7)).year();
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('chuyen_doi:' || $1::text || ':' || $2::text, 0))")
        .bind(project.to_string())
        .bind(nam.to_string())
        .execute(&mut *tx)
        .await?;
    let stt: i64 = sqlx::query_scalar("SELECT count(*) + 1 FROM chuyen_doi_giay WHERE project_id = $1 AND nam = $2").bind(project).bind(nam).fetch_one(&mut *tx).await?;
    Ok((nam, format!("CD-{nam}-{stt:06}")))
}

/// Ghi dòng sổ (số đã cấp bằng `cap_so` trong cùng giao dịch).
pub async fn ghi(tx: &mut sqlx::PgConnection, m: &Moi<'_>, nam: i32, so: &str) -> sqlx::Result<ChuyenDoiRow> {
    sqlx::query_as::<_, ChuyenDoiRow>(&format!(
        "INSERT INTO chuyen_doi_giay ({COLS}) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, now()) RETURNING {COLS}"
    ))
    .bind(m.id)
    .bind(m.project_id)
    .bind(m.document_id)
    .bind(m.ho_so_id)
    .bind(m.van_ban_id)
    .bind(m.phong_id)
    .bind(nam)
    .bind(so)
    .bind(m.ma_tai_lieu)
    .bind(m.ten_tai_lieu)
    .bind(m.ma_ho_so)
    .bind(m.so_trang)
    .bind(m.muc_dich)
    .bind(m.nguoi_nhan)
    .bind(m.can_cu)
    .bind(m.ky_so_ket_luan)
    .bind(m.ky_so_chi_tiet)
    .bind(m.nguoi_thuc_hien)
    .bind(m.chuc_vu)
    .bind(m.pdf_key)
    .bind(m.dung_luong_bytes)
    .fetch_one(&mut *tx)
    .await
}

pub async fn get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<ChuyenDoiRow>> {
    sqlx::query_as::<_, ChuyenDoiRow>(&format!("SELECT {COLS} FROM chuyen_doi_giay WHERE id = $1")).bind(id).fetch_optional(pool).await
}

/// Sổ: dòng JSON (kèm tên người thực hiện, tên phông), lọc theo thời gian và phạm vi phông.
pub async fn so(pool: &PgPool, project: Uuid, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>, phong_ids: Option<&[Uuid]>, limit: i64, offset: i64) -> sqlx::Result<(Vec<Value>, i64)> {
    let where_sql = "c.project_id = $1 AND ($2::timestamptz IS NULL OR c.created_at >= $2) AND ($3::timestamptz IS NULL OR c.created_at < $3) \
        AND ($4::uuid[] IS NULL OR c.phong_id = ANY($4))";
    let rows = sqlx::query(&format!(
        "SELECT to_jsonb(c) || jsonb_build_object('nguoi_thuc_hien_ten', u.display_name, 'ten_phong', p.ma_du_an || ' — ' || p.ten) AS row \
         FROM chuyen_doi_giay c LEFT JOIN users u ON u.id = c.nguoi_thuc_hien LEFT JOIN cl_phong p ON p.id = c.phong_id \
         WHERE {where_sql} ORDER BY c.created_at DESC LIMIT $5 OFFSET $6"
    ))
    .bind(project)
    .bind(from)
    .bind(to)
    .bind(phong_ids)
    .bind(limit.clamp(1, 20_000))
    .bind(offset.max(0))
    .fetch_all(pool)
    .await?;
    let total: i64 = sqlx::query(&format!("SELECT count(*) FROM chuyen_doi_giay c WHERE {where_sql}"))
        .bind(project)
        .bind(from)
        .bind(to)
        .bind(phong_ids)
        .fetch_one(pool)
        .await?
        .try_get(0)?;
    Ok((rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect(), total))
}

/// Số lần một tài liệu đã được chuyển đổi (hiện trên màn Tra cứu).
pub async fn so_lan(pool: &PgPool, document_id: Uuid) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT count(*) FROM chuyen_doi_giay WHERE document_id = $1").bind(document_id).fetch_one(pool).await
}
