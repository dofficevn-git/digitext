// Copyright (c) 2026 DigiText. All rights reserved.
//! LỊCH SỬ ĐƠN VỊ HÌNH THÀNH PHÔNG / LỊCH SỬ PHÔNG có PHIÊN BẢN (0039, 27/11/2026 — đợt 5 đối chiếu Digifile B02; Điều 4 khoản 1a,
//! Điều 22 khoản 1a TT 05/2025/TT-BNV). Mỗi lần nộp bổ sung / chỉnh sửa tạo một phiên bản mới; bản cũ giữ nguyên để tra lại.
//! Chỉ ghi trực tuyến; không vào bộ máy `cl_*` nên không có trên bản sao offline.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LichSuRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub phong_id: Uuid,
    pub phien_ban: i32,
    pub lich_su_don_vi: Option<String>,
    pub lich_su_phong: Option<String>,
    pub thoi_gian_tai_lieu: Option<String>,
    pub tong_so_tai_lieu: Option<String>,
    pub phuong_an_phan_loai: Option<String>,
    pub ngon_ngu: Option<String>,
    pub trang_thai_phong: String,
    pub nguon: Option<String>,
    pub ghi_chu: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    /// Tên người lập (join users).
    pub nguoi_lap: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Moi {
    #[serde(default)]
    pub lich_su_don_vi: Option<String>,
    #[serde(default)]
    pub lich_su_phong: Option<String>,
    #[serde(default)]
    pub thoi_gian_tai_lieu: Option<String>,
    #[serde(default)]
    pub tong_so_tai_lieu: Option<String>,
    #[serde(default)]
    pub phuong_an_phan_loai: Option<String>,
    #[serde(default)]
    pub ngon_ngu: Option<String>,
    #[serde(default)]
    pub trang_thai_phong: Option<String>,
    #[serde(default)]
    pub nguon: Option<String>,
    #[serde(default)]
    pub ghi_chu: Option<String>,
}

const SEL: &str = "SELECT l.id, l.project_id, l.phong_id, l.phien_ban, l.lich_su_don_vi, l.lich_su_phong, l.thoi_gian_tai_lieu, l.tong_so_tai_lieu, \
    l.phuong_an_phan_loai, l.ngon_ngu, l.trang_thai_phong, l.nguon, l.ghi_chu, l.created_by, l.created_at, u.display_name AS nguoi_lap \
    FROM phong_lich_su l LEFT JOIN users u ON u.id = l.created_by";

/// Mọi phiên bản của một phông, mới nhất trước.
pub async fn list(pool: &PgPool, phong: Uuid) -> sqlx::Result<Vec<LichSuRow>> {
    sqlx::query_as::<_, LichSuRow>(&format!("{SEL} WHERE l.phong_id = $1 ORDER BY l.phien_ban DESC")).bind(phong).fetch_all(pool).await
}

/// Phiên bản mới nhất (nếu có).
pub async fn moi_nhat(pool: &PgPool, phong: Uuid) -> sqlx::Result<Option<LichSuRow>> {
    sqlx::query_as::<_, LichSuRow>(&format!("{SEL} WHERE l.phong_id = $1 ORDER BY l.phien_ban DESC LIMIT 1")).bind(phong).fetch_optional(pool).await
}

fn sach(s: &Option<String>) -> Option<String> {
    s.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned)
}

/// Thêm phiên bản mới = max + 1 (khoá tư vấn theo phông để hai người cùng lưu không trùng số). Phải có ít nhất một nội dung.
pub async fn them(pool: &PgPool, project: Uuid, phong: Uuid, m: &Moi, by: Uuid) -> sqlx::Result<Result<LichSuRow, String>> {
    let tt = m.trang_thai_phong.clone().unwrap_or_else(|| "mo".into());
    if !matches!(tt.as_str(), "mo" | "dong") {
        return Ok(Err("trangThaiPhong nhận 'mo' | 'dong'".into()));
    }
    if sach(&m.lich_su_don_vi).is_none() && sach(&m.lich_su_phong).is_none() {
        return Ok(Err("Phải có lịch sử đơn vị hình thành phông hoặc lịch sử phông".into()));
    }
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))").bind(format!("phong_lich_su:{phong}")).execute(&mut *tx).await?;
    let pb: i32 = sqlx::query_scalar("SELECT COALESCE(max(phien_ban), 0) + 1 FROM phong_lich_su WHERE phong_id = $1").bind(phong).fetch_one(&mut *tx).await?;
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO phong_lich_su (id, project_id, phong_id, phien_ban, lich_su_don_vi, lich_su_phong, thoi_gian_tai_lieu, tong_so_tai_lieu, \
            phuong_an_phan_loai, ngon_ngu, trang_thai_phong, nguon, ghi_chu, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
    )
    .bind(id)
    .bind(project)
    .bind(phong)
    .bind(pb)
    .bind(sach(&m.lich_su_don_vi))
    .bind(sach(&m.lich_su_phong))
    .bind(sach(&m.thoi_gian_tai_lieu))
    .bind(sach(&m.tong_so_tai_lieu))
    .bind(sach(&m.phuong_an_phan_loai))
    .bind(sach(&m.ngon_ngu))
    .bind(&tt)
    .bind(sach(&m.nguon))
    .bind(sach(&m.ghi_chu))
    .bind(by)
    .execute(&mut *tx)
    .await?;
    let row = sqlx::query_as::<_, LichSuRow>(&format!("{SEL} WHERE l.id = $1")).bind(id).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Ok(row))
}
