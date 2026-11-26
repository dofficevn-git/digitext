// Copyright (c) 2026 DigiText. All rights reserved.
//! Bảo quản tài liệu lưu trữ số — Điều 29–31 TT 05/2025/TT-BNV (26/11/2026). Quy tắc thuần dùng chung cho API (tình trạng
//! lịch sao lưu, bản cũ cần xoá, phương tiện sắp hết hạn) và worker (nhắc). Không truy CSDL, có test.
use chrono::{DateTime, Duration, NaiveDate, Utc};

/// Nhắc chuyển đổi phương tiện lưu trữ trước khi hết hạn độ bền chừng này ngày (Điều 30.4: "trước 1 năm").
pub const NHAC_PHUONG_TIEN_TRUOC_NGAY: i64 = 365;
/// Kiểm tra tài liệu: mỗi tệp ít nhất một lần trong chừng này năm (Điều 29.2).
pub const CHU_KY_KIEM_NAM: i64 = 3;

/// Một lịch sao lưu đứng ở đâu so với lần sao lưu đạt gần nhất.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TinhTrangLich {
    /// Chưa có lần đạt nào.
    ChuaCo,
    /// Đúng hạn; `han_ke_tiep` là lúc phải có bản tiếp theo.
    DungHan { han_ke_tiep: DateTime<Utc> },
    /// Trễ `gio` giờ so với hạn kế tiếp (đã trừ độ trễ cho phép).
    Tre { han_ke_tiep: DateTime<Utc>, gio: i64 },
}

/// Hạn của lần sao lưu kế tiếp = lần đạt gần nhất + chu kỳ.
pub fn han_ke_tiep(lan_dat_gan_nhat: DateTime<Utc>, chu_ky_gio: i64) -> DateTime<Utc> {
    lan_dat_gan_nhat + Duration::hours(chu_ky_gio.max(1))
}

pub fn tinh_trang_lich(lan_dat_gan_nhat: Option<DateTime<Utc>>, chu_ky_gio: i64, do_tre_cho_phep_gio: i64, bay_gio: DateTime<Utc>) -> TinhTrangLich {
    let Some(t) = lan_dat_gan_nhat else { return TinhTrangLich::ChuaCo };
    let han = han_ke_tiep(t, chu_ky_gio);
    let tre = bay_gio - han - Duration::hours(do_tre_cho_phep_gio.max(0));
    if tre > Duration::zero() {
        TinhTrangLich::Tre { han_ke_tiep: han, gio: tre.num_hours().max(1) }
    } else {
        TinhTrangLich::DungHan { han_ke_tiep: han }
    }
}

/// Số bản sao lưu ĐẠT còn giữ vượt số bản phải giữ → cần xoá bấy nhiêu bản cũ nhất (Điều 30: giữ đúng số bản theo lịch).
pub fn so_ban_can_xoa(so_ban_dang_giu: i64, giu_so_ban: i64) -> i64 {
    (so_ban_dang_giu - giu_so_ban.max(1)).max(0)
}

/// Phương tiện còn bao nhiêu ngày tới hạn độ bền; `Some(âm)` = đã quá hạn. None khi không khai hạn.
pub fn ngay_toi_han_do_ben(han_do_ben: Option<NaiveDate>, hom_nay: NaiveDate) -> Option<i64> {
    han_do_ben.map(|h| (h - hom_nay).num_days())
}

/// Đến lúc nhắc chuyển đổi phương tiện chưa (≤ 365 ngày tới hạn, kể cả đã quá hạn).
pub fn can_nhac_phuong_tien(han_do_ben: Option<NaiveDate>, hom_nay: NaiveDate) -> bool {
    matches!(ngay_toi_han_do_ben(han_do_ben, hom_nay), Some(n) if n <= NHAC_PHUONG_TIEN_TRUOC_NGAY)
}

/// Khoá chống nhắc lặp theo tháng cho phương tiện: "<mã>:<yyyy-mm>" — mỗi tháng nhắc một lần tới khi thay.
pub fn khoa_nhac_thang(ma: &str, hom_nay: NaiveDate) -> String {
    format!("{ma}:{}", hom_nay.format("%Y-%m"))
}

/// Phạm vi kế hoạch kiểm tra (Điều 29.2) — JSON `pham_vi` của bk_ke_hoach.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PhamViKiemTra {
    /// Chỉ tài liệu thuộc các phông này (qua mục lục); trống = cả dự án.
    pub phong_ids: Vec<uuid::Uuid>,
    /// Lấy mẫu xác suất 1–100 (%); None hoặc ≥ 100 = toàn bộ.
    pub xac_suat: Option<u8>,
    /// Chỉ tệp chưa kiểm trong 3 năm.
    pub lau_chua_kiem: bool,
    /// Kiểm cả zip gói AIP trong kho.
    pub goi: bool,
}

impl PhamViKiemTra {
    /// Tỷ lệ lấy mẫu 0 < p ≤ 1 (1 = toàn bộ).
    pub fn ty_le(&self) -> f64 {
        match self.xac_suat {
            Some(p) if (1..100).contains(&p) => f64::from(p) / 100.0,
            _ => 1.0,
        }
    }
    /// Mô tả ngắn in vào biên bản Mẫu 01.
    pub fn mo_ta(&self, ten_phong: &[String]) -> String {
        let mut p = Vec::new();
        p.push(if ten_phong.is_empty() { "toàn bộ tài liệu của kho".to_string() } else { format!("phông: {}", ten_phong.join(", ")) });
        if let Some(x) = self.xac_suat.filter(|x| (1..100).contains(x)) {
            p.push(format!("lấy mẫu xác suất {x}%"));
        }
        if self.lau_chua_kiem {
            p.push(format!("chỉ tệp chưa kiểm trong {CHU_KY_KIEM_NAM} năm"));
        }
        if self.goi {
            p.push("kèm gói AIP (sha256 zip)".into());
        }
        p.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn t(s: &str) -> DateTime<Utc> {
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap().and_utc()
    }

    #[test]
    fn lich_hang_ngay() {
        let gan_nhat = t("2026-11-25 01:10");
        assert_eq!(tinh_trang_lich(None, 24, 2, t("2026-11-26 10:00")), TinhTrangLich::ChuaCo);
        // 24h + 2h trễ cho phép: 03:10 ngày 26 vẫn đúng hạn
        assert!(matches!(tinh_trang_lich(Some(gan_nhat), 24, 2, t("2026-11-26 03:00")), TinhTrangLich::DungHan { .. }));
        match tinh_trang_lich(Some(gan_nhat), 24, 2, t("2026-11-26 10:00")) {
            TinhTrangLich::Tre { gio, han_ke_tiep } => {
                assert_eq!(han_ke_tiep, t("2026-11-26 01:10"));
                assert_eq!(gio, 6);
            }
            x => panic!("{x:?}"),
        }
    }

    #[test]
    fn so_ban_xoa() {
        assert_eq!(so_ban_can_xoa(14, 14), 0);
        assert_eq!(so_ban_can_xoa(17, 14), 3);
        assert_eq!(so_ban_can_xoa(2, 0), 1);
    }

    #[test]
    fn phuong_tien_nhac_truoc_mot_nam() {
        let hom_nay = NaiveDate::from_ymd_opt(2026, 11, 26).unwrap();
        assert!(!can_nhac_phuong_tien(None, hom_nay));
        assert!(!can_nhac_phuong_tien(NaiveDate::from_ymd_opt(2028, 1, 1), hom_nay));
        assert!(can_nhac_phuong_tien(NaiveDate::from_ymd_opt(2027, 11, 20), hom_nay));
        assert!(can_nhac_phuong_tien(NaiveDate::from_ymd_opt(2026, 1, 1), hom_nay));
        assert_eq!(ngay_toi_han_do_ben(NaiveDate::from_ymd_opt(2026, 11, 20), hom_nay), Some(-6));
        assert_eq!(khoa_nhac_thang("HDD-01", hom_nay), "HDD-01:2026-11");
    }

    #[test]
    fn pham_vi_json_va_mo_ta() {
        let pv: PhamViKiemTra = serde_json::from_str(r#"{"xacSuat": 10, "goi": true}"#).unwrap();
        assert_eq!(pv.ty_le(), 0.1);
        assert!(pv.phong_ids.is_empty());
        assert_eq!(pv.mo_ta(&[]), "toàn bộ tài liệu của kho; lấy mẫu xác suất 10%; kèm gói AIP (sha256 zip)");
        let full: PhamViKiemTra = serde_json::from_str("{}").unwrap();
        assert_eq!(full.ty_le(), 1.0);
        assert_eq!(PhamViKiemTra { xac_suat: Some(100), ..Default::default() }.ty_le(), 1.0);
    }
}
