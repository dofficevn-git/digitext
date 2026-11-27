// Copyright (c) 2026 DigiText. All rights reserved.
//! CẤU HÌNH CƠ QUAN LƯU TRỮ (0039, 27/11/2026 — đợt 5 đối chiếu Digifile B01; Điều 4 và Phụ lục I TT 05/2025/TT-BNV). Một dòng cho một
//! dự án (bản Lưu trữ: một dự án = một cơ quan). Không có dòng = mặc định như trước 0039: tên lấy từ `projects.name`, hạn nộp lưu
//! 60 ngày, nhắc 15/7/1, quyền sử dụng 360 ngày. Câu SQL khác (nộp lưu, quyền sử dụng) đọc thẳng bảng này bằng
//! `COALESCE((SELECT … FROM co_quan_luu_tru …), hằng)` để không phải chuyền tham số.
use chrono::{DateTime, Utc};
use digitext_core::luu_tru;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoQuanRow {
    pub project_id: Uuid,
    pub ma_co_quan: Option<String>,
    pub ten: Option<String>,
    pub ten_cap_tren: Option<String>,
    pub dia_chi: Option<String>,
    pub dien_thoai: Option<String>,
    pub email: Option<String>,
    pub loai_hinh: String,
    pub nguoi_ky: Option<String>,
    pub chuc_vu_nguoi_ky: Option<String>,
    pub han_nop_ngay: i32,
    pub nhac_truoc_ngay: Vec<i32>,
    pub quyen_su_dung_ngay: i32,
    pub ghi_chu: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// Thân PUT (mọi trường tuỳ chọn; vắng = giữ / mặc định).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Moi {
    #[serde(default)]
    pub ma_co_quan: Option<String>,
    #[serde(default)]
    pub ten: Option<String>,
    #[serde(default)]
    pub ten_cap_tren: Option<String>,
    #[serde(default)]
    pub dia_chi: Option<String>,
    #[serde(default)]
    pub dien_thoai: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub loai_hinh: Option<String>,
    #[serde(default)]
    pub nguoi_ky: Option<String>,
    #[serde(default)]
    pub chuc_vu_nguoi_ky: Option<String>,
    #[serde(default)]
    pub han_nop_ngay: Option<i32>,
    #[serde(default)]
    pub nhac_truoc_ngay: Option<Vec<i32>>,
    #[serde(default)]
    pub quyen_su_dung_ngay: Option<i32>,
    #[serde(default)]
    pub ghi_chu: Option<String>,
}

/// Giá trị hiệu dụng cho chỗ in ấn / tính hạn (đã điền mặc định).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HieuDung {
    pub ten: String,
    pub ma_co_quan: Option<String>,
    pub ten_cap_tren: Option<String>,
    pub loai_hinh: String,
    pub nguoi_ky: Option<String>,
    pub chuc_vu_nguoi_ky: String,
    pub han_nop_ngay: i32,
    pub nhac_truoc_ngay: Vec<u64>,
    pub quyen_su_dung_ngay: i32,
    /// Đã có dòng cấu hình chưa (desktop báo "chưa khai").
    pub da_khai: bool,
}

const COLS: &str = "project_id, ma_co_quan, ten, ten_cap_tren, dia_chi, dien_thoai, email, loai_hinh, nguoi_ky, chuc_vu_nguoi_ky, han_nop_ngay, \
    nhac_truoc_ngay, quyen_su_dung_ngay, ghi_chu, updated_at, updated_by";

pub async fn get(pool: &PgPool, project: Uuid) -> sqlx::Result<Option<CoQuanRow>> {
    sqlx::query_as::<_, CoQuanRow>(&format!("SELECT {COLS} FROM co_quan_luu_tru WHERE project_id = $1")).bind(project).fetch_optional(pool).await
}

fn sach(s: &Option<String>) -> Option<String> {
    s.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned)
}

/// Cấu hình hiệu dụng của một dự án: dòng cấu hình (nếu có) phủ lên mặc định; tên rơi về `projects.name`.
pub async fn hieu_dung(pool: &PgPool, project: Uuid) -> sqlx::Result<HieuDung> {
    let ten_du_an: Option<String> = sqlx::query_scalar("SELECT name FROM projects WHERE id = $1").bind(project).fetch_optional(pool).await?;
    let row = get(pool, project).await?;
    let chuc_vu_mac_dinh = "LÃNH ĐẠO CƠ QUAN".to_string();
    Ok(match row {
        Some(r) => HieuDung {
            ten: sach(&r.ten).or(ten_du_an).unwrap_or_default(),
            ma_co_quan: sach(&r.ma_co_quan),
            ten_cap_tren: sach(&r.ten_cap_tren),
            loai_hinh: r.loai_hinh,
            nguoi_ky: sach(&r.nguoi_ky),
            chuc_vu_nguoi_ky: sach(&r.chuc_vu_nguoi_ky).unwrap_or(chuc_vu_mac_dinh),
            han_nop_ngay: r.han_nop_ngay,
            nhac_truoc_ngay: moc_sach(&r.nhac_truoc_ngay),
            quyen_su_dung_ngay: r.quyen_su_dung_ngay,
            da_khai: true,
        },
        None => HieuDung {
            ten: ten_du_an.unwrap_or_default(),
            ma_co_quan: None,
            ten_cap_tren: None,
            loai_hinh: "luu_tru_co_quan".into(),
            nguoi_ky: None,
            chuc_vu_nguoi_ky: chuc_vu_mac_dinh,
            han_nop_ngay: luu_tru::HAN_NOP_NGAY as i32,
            nhac_truoc_ngay: luu_tru::NHAC_TRUOC_NGAY.to_vec(),
            quyen_su_dung_ngay: luu_tru::QUYEN_SU_DUNG_NGAY as i32,
            da_khai: false,
        },
    })
}

/// Mốc nhắc: bỏ số âm / trùng, xếp giảm dần (15, 7, 1); rỗng → mặc định.
pub fn moc_sach(v: &[i32]) -> Vec<u64> {
    let mut m: Vec<u64> = v.iter().filter(|x| **x >= 0 && **x <= 3650).map(|x| *x as u64).collect();
    m.sort_unstable_by(|a, b| b.cmp(a));
    m.dedup();
    if m.is_empty() {
        luu_tru::NHAC_TRUOC_NGAY.to_vec()
    } else {
        m
    }
}

/// Ghi (UPSERT) cấu hình. Trả lỗi nghiệp vụ dạng chuỗi khi giá trị không hợp lệ.
pub async fn dat(pool: &PgPool, project: Uuid, m: &Moi, by: Uuid) -> sqlx::Result<Result<CoQuanRow, String>> {
    let loai_hinh = m.loai_hinh.clone().unwrap_or_else(|| "luu_tru_co_quan".into());
    if !matches!(loai_hinh.as_str(), "luu_tru_co_quan" | "luu_tru_lich_su") {
        return Ok(Err("loaiHinh nhận 'luu_tru_co_quan' | 'luu_tru_lich_su'".into()));
    }
    let han = m.han_nop_ngay.unwrap_or(luu_tru::HAN_NOP_NGAY as i32);
    if !(1..=3650).contains(&han) {
        return Ok(Err("hanNopNgay phải từ 1 đến 3650".into()));
    }
    let qsd = m.quyen_su_dung_ngay.unwrap_or(luu_tru::QUYEN_SU_DUNG_NGAY as i32);
    if !(0..=3650).contains(&qsd) {
        return Ok(Err("quyenSuDungNgay phải từ 0 đến 3650".into()));
    }
    let moc: Vec<i32> = match &m.nhac_truoc_ngay {
        Some(v) => moc_sach(v).into_iter().map(|x| x as i32).collect(),
        None => luu_tru::NHAC_TRUOC_NGAY.iter().map(|x| *x as i32).collect(),
    };
    if let Some(e) = sach(&m.email) {
        if !e.contains('@') {
            return Ok(Err("email không hợp lệ".into()));
        }
    }
    let row = sqlx::query_as::<_, CoQuanRow>(&format!(
        "INSERT INTO co_quan_luu_tru (project_id, ma_co_quan, ten, ten_cap_tren, dia_chi, dien_thoai, email, loai_hinh, nguoi_ky, chuc_vu_nguoi_ky, \
            han_nop_ngay, nhac_truoc_ngay, quyen_su_dung_ngay, ghi_chu, updated_at, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, now(), $15) \
         ON CONFLICT (project_id) DO UPDATE SET ma_co_quan = EXCLUDED.ma_co_quan, ten = EXCLUDED.ten, ten_cap_tren = EXCLUDED.ten_cap_tren, \
            dia_chi = EXCLUDED.dia_chi, dien_thoai = EXCLUDED.dien_thoai, email = EXCLUDED.email, loai_hinh = EXCLUDED.loai_hinh, \
            nguoi_ky = EXCLUDED.nguoi_ky, chuc_vu_nguoi_ky = EXCLUDED.chuc_vu_nguoi_ky, han_nop_ngay = EXCLUDED.han_nop_ngay, \
            nhac_truoc_ngay = EXCLUDED.nhac_truoc_ngay, quyen_su_dung_ngay = EXCLUDED.quyen_su_dung_ngay, ghi_chu = EXCLUDED.ghi_chu, \
            updated_at = now(), updated_by = EXCLUDED.updated_by \
         RETURNING {COLS}"
    ))
    .bind(project)
    .bind(sach(&m.ma_co_quan).map(|s| s.to_uppercase()))
    .bind(sach(&m.ten))
    .bind(sach(&m.ten_cap_tren))
    .bind(sach(&m.dia_chi))
    .bind(sach(&m.dien_thoai))
    .bind(sach(&m.email))
    .bind(&loai_hinh)
    .bind(sach(&m.nguoi_ky))
    .bind(sach(&m.chuc_vu_nguoi_ky))
    .bind(han)
    .bind(&moc)
    .bind(qsd)
    .bind(sach(&m.ghi_chu))
    .bind(by)
    .fetch_one(pool)
    .await?;
    Ok(Ok(row))
}

/// Mốc nhắc theo dự án cho worker (dự án không khai → mặc định).
pub async fn moc_nhac_theo_du_an(pool: &PgPool) -> sqlx::Result<std::collections::HashMap<Uuid, Vec<u64>>> {
    let rows: Vec<(Uuid, Vec<i32>)> = sqlx::query_as("SELECT project_id, nhac_truoc_ngay FROM co_quan_luu_tru").fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(p, m)| (p, moc_sach(&m))).collect())
}

/// Biểu thức SQL: hạn nộp (ngày) của dự án `cot_project` — dùng trong câu có bí danh bảng hồ sơ.
pub fn sql_han_nop(cot_project: &str) -> String {
    format!("COALESCE((SELECT c.han_nop_ngay FROM co_quan_luu_tru c WHERE c.project_id = {cot_project}), {})", luu_tru::HAN_NOP_NGAY)
}

/// Biểu thức SQL: số ngày quyền sử dụng sau khoá của dự án `cot_project`.
pub fn sql_quyen_su_dung(cot_project: &str) -> String {
    format!("COALESCE((SELECT c.quyen_su_dung_ngay FROM co_quan_luu_tru c WHERE c.project_id = {cot_project}), {})", luu_tru::QUYEN_SU_DUNG_NGAY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moc_sach_bo_trung_am_va_xep_giam() {
        assert_eq!(moc_sach(&[1, 7, 15, 7, -3]), vec![15, 7, 1]);
        assert_eq!(moc_sach(&[]), vec![15, 7, 1]);
        assert_eq!(moc_sach(&[30]), vec![30]);
    }

    #[test]
    fn sql_han_nop_co_mac_dinh() {
        assert!(sql_han_nop("v.project_id").ends_with(", 60)"));
        assert!(sql_quyen_su_dung("h.project_id").contains("quyen_su_dung_ngay"));
    }
}
