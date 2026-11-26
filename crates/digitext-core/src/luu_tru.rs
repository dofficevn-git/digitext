// Copyright (c) 2026 DigiText. All rights reserved.
//! Lưu trữ hiện hành — Điều 19 TT 05/2025/TT-BNV (26/11/2026). Quy tắc thuần, dùng chung cho API (lọc hồ sơ đến hạn,
//! kiểm quyền sử dụng) và worker (nhắc hạn). Chốt với anh Hiếu 26/11/2026: hạn nộp tính theo **ngày lịch** từ **ngày kết
//! thúc hồ sơ**; ngày không đầy đủ lấy cuối kỳ (năm → 31/12, tháng → ngày cuối tháng) — khớp hàm SQL `cl_date_end_of` (0031).
use chrono::{Datelike, Days, NaiveDate};

/// Hồ sơ kết thúc rồi phải nộp vào lưu trữ cơ quan trong vòng chừng này ngày (Điều 19.1).
pub const HAN_NOP_NGAY: u64 = 60;
/// Sau tiếp nhận, người/đơn vị nộp còn được sử dụng hồ sơ đã nộp trong chừng này ngày (Điều 19.5).
pub const QUYEN_SU_DUNG_NGAY: u64 = 360;
/// Nhắc trước hạn nộp ở các mốc còn lại này (ngày).
pub const NHAC_TRUOC_NGAY: [u64; 3] = [15, 7, 1];

/// Ngày CUỐI kỳ của chuỗi ngày không đầy đủ mà mục lục hay dùng: `dd/MM/yyyy` · `MM/yyyy` · `yyyy`. Khác với đầu kỳ
/// (`cl_date_of`, `ngay_kt_sort`) vì hồ sơ "2025" kết thúc trong năm 2025 chứ không phải ngày 01/01.
pub fn ngay_cuoi_ky(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    let phan: Vec<&str> = s.split('/').collect();
    match phan.as_slice() {
        [nam] => NaiveDate::from_ymd_opt(nam.parse().ok()?, 12, 31),
        [thang, nam] => cuoi_thang(nam.parse().ok()?, thang.parse().ok()?),
        [ngay, thang, nam] => NaiveDate::from_ymd_opt(nam.parse().ok()?, thang.parse().ok()?, ngay.parse().ok()?),
        _ => None,
    }
}

fn cuoi_thang(nam: i32, thang: u32) -> Option<NaiveDate> {
    let dau = NaiveDate::from_ymd_opt(nam, thang, 1)?;
    let (n2, t2) = if thang == 12 { (nam + 1, 1) } else { (nam, thang + 1) };
    NaiveDate::from_ymd_opt(n2, t2, 1)?.pred_opt().filter(|d| d.month() == dau.month())
}

/// Hạn nộp lưu của hồ sơ có ngày kết thúc `ngay_ket_thuc`; None khi chưa có / không đọc được ngày.
pub fn han_nop(ngay_ket_thuc: Option<&str>) -> Option<NaiveDate> {
    ngay_cuoi_ky(ngay_ket_thuc?)?.checked_add_days(Days::new(HAN_NOP_NGAY))
}

/// Hồ sơ chưa nộp đứng ở đâu so với hạn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TinhTrangHan {
    /// Còn xa hạn (hơn mốc nhắc đầu tiên).
    ChuaDen,
    /// Đến hạn: còn `n` ngày (0 = hôm nay).
    DenHan(u64),
    /// Quá hạn `n` ngày.
    QuaHan(u64),
}

pub fn tinh_trang_han(han: NaiveDate, hom_nay: NaiveDate) -> TinhTrangHan {
    let lech = (han - hom_nay).num_days();
    if lech < 0 {
        TinhTrangHan::QuaHan(lech.unsigned_abs())
    } else if (lech as u64) <= NHAC_TRUOC_NGAY[0] {
        TinhTrangHan::DenHan(lech as u64)
    } else {
        TinhTrangHan::ChuaDen
    }
}

/// Hôm nay có đúng là một mốc nhắc trước hạn không (15 / 7 / 1 ngày) → trả mốc đó để làm khoá chống nhắc lặp.
pub fn moc_nhac(han: NaiveDate, hom_nay: NaiveDate) -> Option<u64> {
    let lech = (han - hom_nay).num_days();
    NHAC_TRUOC_NGAY.into_iter().find(|m| lech == *m as i64)
}

/// Người nộp còn quyền sử dụng hồ sơ đã nộp không (tính từ lúc khoá, tức lúc tiếp nhận).
pub fn con_quyen_su_dung(khoa_luc: NaiveDate, hom_nay: NaiveDate) -> bool {
    khoa_luc.checked_add_days(Days::new(QUYEN_SU_DUNG_NGAY)).is_some_and(|het| hom_nay <= het)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn d(y: i32, m: u32, dd: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, dd).unwrap()
    }

    #[test]
    fn ngay_cuoi_ky_lay_cuoi_nam_cuoi_thang() {
        assert_eq!(ngay_cuoi_ky("2025"), Some(d(2025, 12, 31)));
        assert_eq!(ngay_cuoi_ky("02/2024"), Some(d(2024, 2, 29)), "năm nhuận");
        assert_eq!(ngay_cuoi_ky("12/2025"), Some(d(2025, 12, 31)));
        assert_eq!(ngay_cuoi_ky(" 15/03/2025 "), Some(d(2025, 3, 15)));
        assert_eq!(ngay_cuoi_ky("13/2025"), None, "tháng 13 không có");
        assert_eq!(ngay_cuoi_ky(""), None);
        assert_eq!(ngay_cuoi_ky("abc"), None);
    }

    #[test]
    fn han_nop_60_ngay_lich_tu_cuoi_ky() {
        assert_eq!(han_nop(Some("2025")), Some(d(2026, 3, 1)), "31/12 + 60 = 01/03");
        assert_eq!(han_nop(Some("31/10/2025")), Some(d(2025, 12, 30)));
        assert_eq!(han_nop(None), None);
    }

    #[test]
    fn tinh_trang_va_moc_nhac() {
        let han = d(2026, 3, 1);
        assert_eq!(tinh_trang_han(han, d(2026, 1, 1)), TinhTrangHan::ChuaDen);
        assert_eq!(tinh_trang_han(han, d(2026, 2, 14)), TinhTrangHan::DenHan(15));
        assert_eq!(tinh_trang_han(han, d(2026, 3, 1)), TinhTrangHan::DenHan(0));
        assert_eq!(tinh_trang_han(han, d(2026, 3, 4)), TinhTrangHan::QuaHan(3));
        assert_eq!(moc_nhac(han, d(2026, 2, 14)), Some(15));
        assert_eq!(moc_nhac(han, d(2026, 2, 22)), Some(7));
        assert_eq!(moc_nhac(han, d(2026, 2, 28)), Some(1));
        assert_eq!(moc_nhac(han, d(2026, 2, 20)), None);
    }

    #[test]
    fn quyen_su_dung_360_ngay() {
        let khoa = d(2026, 1, 1);
        assert!(con_quyen_su_dung(khoa, d(2026, 12, 27)));
        assert!(!con_quyen_su_dung(khoa, d(2026, 12, 28)));
    }
}
