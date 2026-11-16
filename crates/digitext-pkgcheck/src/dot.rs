// Copyright (c) 2026 DigiText. All rights reserved.
//! Kiểm **cả đợt giao**: những lỗi chỉ thấy khi đặt nhiều gói cạnh nhau.
//!
//! Kiểm từng gói một thì mỗi gói đều sạch mà cả đợt vẫn hỏng: hai gói cùng `OBJID` (công cụ
//! đóng gói dùng lại UUID), cùng một tên tệp, hay một đợt lẫn hai phông. Nhận gói vào kho mới
//! phát hiện thì đã muộn — phải đóng lại từ đầu.
//!
//! Ở crate này chứ không ở cổng nhà cung cấp: "thế nào là một đợt giao hợp lệ" là luật đóng gói,
//! cùng họ với luật trong [`crate::check_loaded`]. Cổng tự viết lại thì bản desktop và bản web
//! sẽ có ngày nói khác nhau.

use crate::{Finding, Level};
use std::collections::BTreeMap;

/// Phần tóm tắt của một gói đã kiểm — đúng những trường cửa `tu-kiem/goi` trả về.
///
/// Cố ý **không** nhận cả `PackageResult`: bên gọi chỉ cần gửi lại mấy dòng tóm tắt nó đã có,
/// không phải gửi lại bản kê của từng gói (một đợt 50 gói là hơn 20 MB).
#[derive(Debug, Clone, Default)]
pub struct GoiTomTat {
    pub ten_goi: String,
    pub objid: String,
    pub ma_phong: String,
    pub so_tai_lieu: u64,
    pub so_trang: u64,
    pub so_tep_du_lieu: u64,
    /// Kết luận của chính gói đó, nguyên văn. Rỗng thì coi như chưa kiểm.
    pub ket_luan: String,
    pub so_loi: usize,
    pub so_canh_bao: usize,
}

#[derive(Debug, Clone, Default)]
pub struct DotKetQua {
    pub findings: Vec<Finding>,
    pub so_goi: usize,
    pub tong_tai_lieu: u64,
    pub tong_trang: u64,
    pub tong_tep_du_lieu: u64,
    /// Gói có lỗi cấu trúc — phải sửa trước khi giao.
    pub so_goi_khong_dat: usize,
    /// Mã phông xuất hiện trong đợt, kèm số gói mang mã đó.
    pub ma_phong: Vec<(String, usize)>,
    pub so_loi: usize,
    pub so_canh_bao: usize,
}

/// Gom các giá trị trùng nhau: trả `(giá trị, [tên gói])` cho mỗi giá trị xuất hiện ≥ 2 lần.
fn trung<'a>(goi: &'a [GoiTomTat], lay: fn(&'a GoiTomTat) -> &'a str) -> Vec<(&'a str, Vec<&'a str>)> {
    let mut m: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for g in goi {
        let v = lay(g).trim();
        if !v.is_empty() {
            m.entry(v).or_default().push(g.ten_goi.as_str());
        }
    }
    m.into_iter().filter(|(_, v)| v.len() > 1).collect()
}

/// Những gì chỉ thấy khi đặt cả đợt cạnh nhau.
///
/// Không nhận xét gì về việc mã phông của gói có thuộc dự án không: mã trong METS dài hơn mã
/// phông của dự án (đo trên gói thật — `H49.155.3.01.2013.09.01` so với `H49.155.03`), nên so
/// khớp là báo oan. Cửa trả danh sách phông của dự án để người đọc tự đối chiếu.
pub fn kiem_dot(goi: &[GoiTomTat]) -> DotKetQua {
    let mut r = DotKetQua { so_goi: goi.len(), ..Default::default() };
    let mut f: Vec<Finding> = Vec::new();

    if goi.is_empty() {
        f.push(Finding::new(Level::Err, "Đợt giao", "Đợt rỗng — chưa có gói nào".into()));
        r.findings = f;
        return r;
    }

    for g in goi {
        r.tong_tai_lieu += g.so_tai_lieu;
        r.tong_trang += g.so_trang;
        r.tong_tep_du_lieu += g.so_tep_du_lieu;
        r.so_loi += g.so_loi;
        r.so_canh_bao += g.so_canh_bao;
        if g.so_loi > 0 || g.ket_luan.starts_with("KHÔNG ĐẠT") {
            r.so_goi_khong_dat += 1;
        }
    }

    // 1. Trùng OBJID — nặng nhất. Hai gói cùng định danh thì kho chỉ giữ được một.
    for (v, ten) in trung(goi, |g| &g.objid) {
        f.push(Finding::new(
            Level::Err,
            "Đợt giao",
            format!("{} gói cùng OBJID {v} ({}) — mỗi gói phải có định danh riêng, đóng gói lại", ten.len(), ten.join(", ")),
        ));
    }

    // 2. Trùng tên tệp gói — không giao cùng lúc được, và dễ ghi đè nhau lúc chép.
    for (v, ten) in trung(goi, |g| &g.ten_goi) {
        f.push(Finding::new(
            Level::Err,
            "Đợt giao",
            format!("{} gói cùng tên tệp {v:?} — đổi tên để không ghi đè nhau khi giao", ten.len()),
        ));
    }

    // 3. Gói không đọc được OBJID: không đối chiếu trùng lặp được, nói thẳng là đã bỏ qua.
    let thieu: Vec<&str> = goi.iter().filter(|g| g.objid.trim().is_empty()).map(|g| g.ten_goi.as_str()).collect();
    if !thieu.is_empty() {
        f.push(Finding::new(
            Level::Warn,
            "Đợt giao",
            format!("{} gói không đọc được OBJID ({}) — chưa đối chiếu trùng lặp được cho những gói này", thieu.len(), thieu.join(", ")),
        ));
    }

    // 4. Một đợt lẫn nhiều phông: gần như luôn là chọn nhầm thư mục, nhưng KHÔNG phải lỗi —
    //    có đơn vị giao gộp thật. Cảnh báo, kèm số gói từng mã để người ta thấy ngay cái lạc đàn.
    let mut dem: BTreeMap<&str, usize> = BTreeMap::new();
    for g in goi {
        let v = g.ma_phong.trim();
        if !v.is_empty() {
            *dem.entry(v).or_default() += 1;
        }
    }
    r.ma_phong = dem.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect();
    if dem.len() > 1 {
        let ke: Vec<String> = dem.iter().map(|(k, v)| format!("{k} ({v} gói)")).collect();
        f.push(Finding::new(
            Level::Warn,
            "Đợt giao",
            format!("Đợt có {} mã phông khác nhau: {} — kiểm lại xem có gói lạc sang đợt khác không", dem.len(), ke.join(" · ")),
        ));
    }

    // 5. Tổng kết, để người ta không phải tự cộng.
    if r.so_goi_khong_dat > 0 {
        f.push(Finding::new(
            Level::Err,
            "Đợt giao",
            format!("{}/{} gói còn lỗi cấu trúc — sửa rồi kiểm lại trước khi giao", r.so_goi_khong_dat, r.so_goi),
        ));
    } else {
        f.push(Finding::new(
            Level::Ok,
            "Đợt giao",
            format!("{} gói, không gói nào có lỗi cấu trúc", r.so_goi),
        ));
    }
    f.push(Finding::new(
        Level::Ok,
        "Đợt giao",
        format!("Cộng cả đợt: {} tài liệu · {} trang · {} tệp dữ liệu", r.tong_tai_lieu, r.tong_trang, r.tong_tep_du_lieu),
    ));

    r.findings = f;
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(ten: &str, objid: &str, phong: &str) -> GoiTomTat {
        GoiTomTat {
            ten_goi: ten.into(),
            objid: objid.into(),
            ma_phong: phong.into(),
            so_tai_lieu: 7,
            so_trang: 123,
            so_tep_du_lieu: 7,
            ket_luan: "CHƯA KIỂM ĐỦ (có cảnh báo)".into(),
            so_loi: 0,
            so_canh_bao: 11,
            ..Default::default()
        }
    }
    fn chu(r: &DotKetQua, muc: Level) -> String {
        r.findings.iter().filter(|f| f.level == muc).map(|f| f.message.as_str()).collect::<Vec<_>>().join(" | ")
    }

    /// Đợt sạch: cộng đúng, không bịa ra lỗi nào.
    #[test]
    fn dot_sach_thi_chi_cong_so() {
        let r = kiem_dot(&[g("a.zip", "urn:A", "H49"), g("b.zip", "urn:B", "H49")]);
        assert_eq!(chu(&r, Level::Err), "", "đợt sạch mà vẫn báo lỗi");
        assert_eq!(chu(&r, Level::Warn), "", "đợt sạch mà vẫn cảnh báo");
        assert_eq!((r.so_goi, r.tong_tai_lieu, r.tong_trang, r.tong_tep_du_lieu), (2, 14, 246, 14));
        assert_eq!(r.so_goi_khong_dat, 0);
        assert_eq!(r.ma_phong, vec![("H49".to_string(), 2)]);
    }

    /// Lỗi CHỈ thấy khi đặt cạnh nhau — từng gói một thì cả hai đều sạch.
    #[test]
    fn trung_objid_la_loi_va_goi_dung_ten_hai_goi() {
        let r = kiem_dot(&[g("a.zip", "urn:TRUNG", "H49"), g("b.zip", "urn:TRUNG", "H49")]);
        let e = chu(&r, Level::Err);
        assert!(e.contains("urn:TRUNG") && e.contains("a.zip") && e.contains("b.zip"), "{e}");
        // Mỗi gói riêng lẻ vẫn 0 lỗi — đúng cái mà kiểm từng gói không bắt được.
        assert_eq!(r.so_loi, 0, "số lỗi cộng từ các gói không đổi, lỗi này là của ĐỢT");
    }

    #[test]
    fn trung_ten_tep_cung_la_loi() {
        let r = kiem_dot(&[g("AIP.zip", "urn:A", "H49"), g("AIP.zip", "urn:B", "H49")]);
        assert!(chu(&r, Level::Err).contains("cùng tên tệp"), "{}", chu(&r, Level::Err));
    }

    /// Nhiều phông là CẢNH BÁO, không phải lỗi: có đơn vị giao gộp thật.
    #[test]
    fn nhieu_phong_chi_canh_bao_va_chi_ro_goi_lac_dan() {
        let r = kiem_dot(&[g("a.zip", "urn:A", "H49"), g("b.zip", "urn:B", "H49"), g("c.zip", "urn:C", "H50")]);
        assert_eq!(chu(&r, Level::Err), "", "nhiều phông không được là lỗi");
        let w = chu(&r, Level::Warn);
        assert!(w.contains("H49 (2 gói)") && w.contains("H50 (1 gói)"), "{w}");
        assert_eq!(r.ma_phong, vec![("H49".to_string(), 2), ("H50".to_string(), 1)]);
    }

    /// Gói thiếu OBJID không được âm thầm bỏ qua — nói rõ là CHƯA đối chiếu được.
    #[test]
    fn thieu_objid_phai_noi_la_chua_doi_chieu_duoc() {
        let r = kiem_dot(&[g("a.zip", "", "H49"), g("b.zip", "", "H49")]);
        let w = chu(&r, Level::Warn);
        assert!(w.contains("không đọc được OBJID") && w.contains("a.zip"), "{w}");
        // Hai ô trống KHÔNG được coi là trùng nhau.
        assert_eq!(chu(&r, Level::Err), "", "hai gói cùng thiếu OBJID không phải là trùng OBJID");
    }

    #[test]
    fn goi_khong_dat_duoc_dem_va_noi_ro() {
        let mut x = g("a.zip", "urn:A", "H49");
        x.so_loi = 3;
        x.ket_luan = "KHÔNG ĐẠT".into();
        let r = kiem_dot(&[x, g("b.zip", "urn:B", "H49")]);
        assert_eq!(r.so_goi_khong_dat, 1);
        assert!(chu(&r, Level::Err).contains("1/2 gói còn lỗi cấu trúc"), "{}", chu(&r, Level::Err));
    }

    #[test]
    fn dot_rong_noi_thang_chu_khong_bao_dat() {
        let r = kiem_dot(&[]);
        assert!(chu(&r, Level::Err).contains("Đợt rỗng"));
        assert_eq!(r.so_goi, 0);
    }
}
