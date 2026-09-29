// Copyright (c) 2026 DigiText. All rights reserved.
//! Logic thuần của nghiệp vụ CHỈNH LÝ tài liệu giấy (kế hoạch 29/9/2026, từ Digitext.2025):
//! đọc ngày dạng chữ, mã hồ sơ, trạng thái thời hạn bảo quản, 6 thứ tự hệ thống hoá,
//! bảng mã TT 05/2025/TT-BNV, đọc số thành chữ. Dùng chung máy chủ và desktop.

use std::cmp::Ordering;

// ------------------------------------------------------------------ ngày tháng

/// Ngày của hồ sơ / văn bản có thể thiếu ngày hoặc tháng ("3/2021", "2021").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClDate {
    pub year: i32,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DateErr {
    #[error("không đọc được ngày: {0:?}")]
    Format(String),
    #[error("ngày không tồn tại: {0:?}")]
    Invalid(String),
    #[error("năm {0} ngoài khoảng cho phép")]
    Year(i32),
}

pub const MIN_YEAR: i32 = 1800;
pub const MAX_YEAR: i32 = 2100;

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Số ngày kể từ 1970-01-01 → (năm, tháng, ngày). Thuật toán Howard Hinnant.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((if m <= 2 { y + 1 } else { y }) as i32, m, d)
}

impl ClDate {
    /// Đọc chuỗi ngày người dùng nhập / Excel: `d/M/yyyy`, `dd-MM-yyyy`, `d.M.yyyy`, `M/yyyy`,
    /// `yyyy`, `yyyy-MM-dd` (ISO), số serial Excel (5 chữ số). Chuỗi rỗng → `Ok(None)`.
    pub fn parse(s: &str) -> Result<Option<ClDate>, DateErr> {
        let s = s.trim();
        if s.is_empty() {
            return Ok(None);
        }
        let s = s.split(|c: char| c == 'T' || c == ' ').next().unwrap_or(s);
        let sep = s.chars().find(|c| matches!(c, '/' | '-' | '.'));
        let parts: Vec<&str> = match sep {
            Some(c) => s.split(c).map(str::trim).collect(),
            None => vec![s],
        };
        let num = |p: &str| -> Result<i64, DateErr> {
            if p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()) {
                return Err(DateErr::Format(s.to_owned()));
            }
            p.parse::<i64>().map_err(|_| DateErr::Format(s.to_owned()))
        };
        let (y, m, d): (i64, Option<i64>, Option<i64>) = match parts.as_slice() {
            [only] => {
                let n = num(only)?;
                if only.len() >= 5 {
                    // Serial Excel (1899-12-30 + n).
                    let (yy, mm, dd) = civil_from_days(n - 25_569);
                    (yy as i64, Some(mm as i64), Some(dd as i64))
                } else {
                    (n, None, None)
                }
            }
            [a, b] => (num(b)?, Some(num(a)?), None),
            [a, b, c] => {
                if a.len() == 4 {
                    (num(a)?, Some(num(b)?), Some(num(c)?))
                } else {
                    (num(c)?, Some(num(b)?), Some(num(a)?))
                }
            }
            _ => return Err(DateErr::Format(s.to_owned())),
        };
        if !(MIN_YEAR as i64..=MAX_YEAR as i64).contains(&y) {
            return Err(DateErr::Year(y as i32));
        }
        let year = y as i32;
        let month = match m {
            Some(m) if (1..=12).contains(&m) => Some(m as u32),
            Some(_) => return Err(DateErr::Invalid(s.to_owned())),
            None => None,
        };
        let day = match (d, month) {
            (Some(d), Some(mo)) if d >= 1 && d <= days_in_month(year, mo) as i64 => Some(d as u32),
            (Some(_), _) => return Err(DateErr::Invalid(s.to_owned())),
            (None, _) => None,
        };
        Ok(Some(ClDate { year, month, day }))
    }

    /// Dạng chuẩn lưu trữ: `dd/MM/yyyy`, `MM/yyyy` hoặc `yyyy`.
    pub fn to_text(&self) -> String {
        match (self.month, self.day) {
            (Some(m), Some(d)) => format!("{d:02}/{m:02}/{}", self.year),
            (Some(m), None) => format!("{m:02}/{}", self.year),
            _ => format!("{}", self.year),
        }
    }

    /// Khoá sắp xếp (thiếu tháng/ngày coi như 1).
    pub fn sort_key(&self) -> (i32, u32, u32) {
        (self.year, self.month.unwrap_or(1), self.day.unwrap_or(1))
    }

    /// Dạng ISO `yyyy-MM-dd` (thiếu → ngày 1).
    pub fn to_iso(&self) -> String {
        let (y, m, d) = self.sort_key();
        format!("{y:04}-{m:02}-{d:02}")
    }
}

/// Chuẩn hoá chuỗi ngày về dạng lưu trữ; rỗng → `Ok(None)`.
pub fn normalize_date(s: &str) -> Result<Option<String>, DateErr> {
    Ok(ClDate::parse(s)?.map(|d| d.to_text()))
}

// ------------------------------------------------------------------ mã hồ sơ

/// Mã hồ sơ = `{MaDuAn|00}.{PhongSo|00}.{nămKT}.{ho_so_so}[.{MucLucSo}]`.
/// Chưa có số hồ sơ chính thức hoặc chưa có năm kết thúc → `None`.
pub fn ma_ho_so(
    ma_du_an: Option<&str>,
    phong_so: Option<&str>,
    nam_ket_thuc: Option<i32>,
    ho_so_so: Option<&str>,
    muc_luc_so: Option<&str>,
) -> Option<String> {
    let hs = ho_so_so.map(str::trim).filter(|s| !s.is_empty())?;
    let nam = nam_ket_thuc?;
    let or00 = |v: Option<&str>| v.map(str::trim).filter(|s| !s.is_empty()).unwrap_or("00").to_owned();
    let mut out = format!("{}.{}.{}.{}", or00(ma_du_an), or00(phong_so), nam, hs);
    if let Some(ml) = muc_luc_so.map(str::trim).filter(|s| !s.is_empty()) {
        out.push('.');
        out.push_str(ml);
    }
    Some(out)
}

/// Mã lưu trữ tài liệu (TT05) = mã hồ sơ + "." + STT đệm 7 chữ số.
pub fn ma_van_ban_tt05(ma_ho_so: &str, so_thu_tu: i64) -> String {
    format!("{ma_ho_so}.{so_thu_tu:07}")
}

// ------------------------------------------------------------------ thời hạn bảo quản

/// Trạng thái hồ sơ theo THBQ, tính động (bản cũ lưu cứng nên không bao giờ cập nhật).
/// `trang_thai`: 1 vĩnh viễn · 2 có thời hạn (`gia_tri` năm) · 3 đánh giá lại · 4 theo tuổi thọ công trình.
pub fn thbq_status(trang_thai: Option<i16>, gia_tri: Option<i32>, nam_ket_thuc: Option<i32>, nam_now: i32) -> Option<&'static str> {
    match trang_thai? {
        1 => Some("VV"),
        2 => match nam_ket_thuc {
            Some(kt) if (nam_now - kt + 1) > gia_tri.unwrap_or(0) => Some("HET"),
            _ => Some("GL"),
        },
        _ => Some("GL"),
    }
}

// ------------------------------------------------------------------ so sánh số tự nhiên

/// So sánh "9" < "10" < "10A" < "572A": chạy số so theo giá trị, phần chữ so theo chữ (bỏ hoa/thường).
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let mut ia = a.trim().chars().peekable();
    let mut ib = b.trim().chars().peekable();
    loop {
        match (ia.peek().copied(), ib.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(ca), Some(cb)) => {
                if ca.is_ascii_digit() && cb.is_ascii_digit() {
                    let mut na = String::new();
                    while let Some(c) = ia.peek().copied().filter(char::is_ascii_digit) {
                        na.push(c);
                        ia.next();
                    }
                    let mut nb = String::new();
                    while let Some(c) = ib.peek().copied().filter(char::is_ascii_digit) {
                        nb.push(c);
                        ib.next();
                    }
                    let ta = na.trim_start_matches('0');
                    let tb = nb.trim_start_matches('0');
                    let o = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
                    if o != Ordering::Equal {
                        return o;
                    }
                } else {
                    let la = ca.to_lowercase().next().unwrap_or(ca);
                    let lb = cb.to_lowercase().next().unwrap_or(cb);
                    let o = la.cmp(&lb);
                    if o != Ordering::Equal {
                        return o;
                    }
                    ia.next();
                    ib.next();
                }
            }
        }
    }
}

// ------------------------------------------------------------------ hệ thống hoá: 6 thứ tự

/// Thứ tự xuất mục lục tạm / đánh số hồ sơ (giữ đúng 6 kiểu của bản cũ).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// 1: mảng công việc → ngày kết thúc
    MangThoiGian = 1,
    /// 2: ngày kết thúc → mảng
    ThoiGianMang = 2,
    /// 3: THBQ tăng → mảng → ngày KT
    ThbqAscMangThoiGian = 3,
    /// 4: THBQ giảm → mảng → ngày KT
    ThbqDescMangThoiGian = 4,
    /// 5: THBQ tăng → ngày KT → mảng
    ThbqAscThoiGianMang = 5,
    /// 6: THBQ giảm → ngày KT → mảng
    ThbqDescThoiGianMang = 6,
}

impl SortOrder {
    pub fn from_i32(n: i32) -> Option<SortOrder> {
        Some(match n {
            1 => SortOrder::MangThoiGian,
            2 => SortOrder::ThoiGianMang,
            3 => SortOrder::ThbqAscMangThoiGian,
            4 => SortOrder::ThbqDescMangThoiGian,
            5 => SortOrder::ThbqAscThoiGianMang,
            6 => SortOrder::ThbqDescThoiGianMang,
            _ => return None,
        })
    }
    pub fn label(self) -> &'static str {
        match self {
            SortOrder::MangThoiGian => "Mặt hoạt động - Thời gian",
            SortOrder::ThoiGianMang => "Thời gian - Mặt hoạt động",
            SortOrder::ThbqAscMangThoiGian => "THBQ tăng dần - Mặt hoạt động - Thời gian",
            SortOrder::ThbqDescMangThoiGian => "THBQ giảm dần - Mặt hoạt động - Thời gian",
            SortOrder::ThbqAscThoiGianMang => "THBQ tăng dần - Thời gian - Mặt hoạt động",
            SortOrder::ThbqDescThoiGianMang => "THBQ giảm dần - Thời gian - Mặt hoạt động",
        }
    }
}

/// Dữ liệu tối thiểu của một hồ sơ để sắp xếp.
#[derive(Debug, Clone)]
pub struct SortItem<K> {
    pub key: K,
    /// `cl_thbq.so_tt` (vĩnh viễn nhỏ nhất).
    pub thbq_sort: i32,
    /// Thứ tự mảng công việc (so_tt), rồi tên mảng để ổn định.
    pub mang_sort: i32,
    pub mang_ten: String,
    /// Ngày kết thúc; thiếu → xếp cuối.
    pub ngay_kt: Option<(i32, u32, u32)>,
    pub so_tam: String,
}

fn cmp_ngay(a: &Option<(i32, u32, u32)>, b: &Option<(i32, u32, u32)>) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => x.cmp(y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Sắp xếp ổn định theo `order`; hoà thì theo số tạm (số tự nhiên).
pub fn sort_items<K>(items: &mut [SortItem<K>], order: SortOrder) {
    let mang = |a: &SortItem<K>, b: &SortItem<K>| a.mang_sort.cmp(&b.mang_sort).then_with(|| a.mang_ten.cmp(&b.mang_ten));
    let ngay = |a: &SortItem<K>, b: &SortItem<K>| cmp_ngay(&a.ngay_kt, &b.ngay_kt);
    let thbq_asc = |a: &SortItem<K>, b: &SortItem<K>| a.thbq_sort.cmp(&b.thbq_sort);
    let thbq_desc = |a: &SortItem<K>, b: &SortItem<K>| b.thbq_sort.cmp(&a.thbq_sort);
    items.sort_by(|a, b| {
        let o = match order {
            SortOrder::MangThoiGian => mang(a, b).then_with(|| ngay(a, b)),
            SortOrder::ThoiGianMang => ngay(a, b).then_with(|| mang(a, b)),
            SortOrder::ThbqAscMangThoiGian => thbq_asc(a, b).then_with(|| mang(a, b)).then_with(|| ngay(a, b)),
            SortOrder::ThbqDescMangThoiGian => thbq_desc(a, b).then_with(|| mang(a, b)).then_with(|| ngay(a, b)),
            SortOrder::ThbqAscThoiGianMang => thbq_asc(a, b).then_with(|| ngay(a, b)).then_with(|| mang(a, b)),
            SortOrder::ThbqDescThoiGianMang => thbq_desc(a, b).then_with(|| ngay(a, b)).then_with(|| mang(a, b)),
        };
        o.then_with(|| natural_cmp(&a.so_tam, &b.so_tam))
    });
}

// ------------------------------------------------------------------ bảng mã TT 05/2025/TT-BNV

pub mod codes {
    /// Thời hạn lưu trữ (cột 12 mẫu TT05).
    pub const THBQ: &[(&str, &str)] = &[
        ("01", "Vĩnh viễn"),
        ("02", "70 năm"),
        ("03", "50 năm"),
        ("04", "30 năm"),
        ("05", "20 năm"),
        ("06", "10 năm"),
        ("07", "Khác"),
    ];
    /// Mức độ tiếp cận / chế độ sử dụng.
    pub const TIEP_CAN: &[(&str, &str)] = &[("01", "Công khai"), ("02", "Sử dụng có điều kiện"), ("03", "Mật")];
    /// Tình trạng vật lý.
    pub const TINH_TRANG_VAT_LY: &[(&str, &str)] = &[("01", "Tốt"), ("02", "Bình thường"), ("03", "Hỏng")];
    /// Mức độ tin cậy.
    pub const TIN_CAY: &[(&str, &str)] = &[("01", "Gốc điện tử"), ("02", "Số hoá"), ("03", "Hỗn hợp")];
    pub const KHO_GIAY: &[&str] = &["A0", "A1", "A2", "A3", "A4", "A5"];
    /// Ngôn ngữ (cột 14 / 38).
    pub const NGON_NGU: &[(&str, &str)] = &[
        ("01", "Tiếng Việt"),
        ("02", "Tiếng Anh"),
        ("03", "Tiếng Pháp"),
        ("04", "Tiếng Nga"),
        ("05", "Tiếng Trung"),
        ("06", "Việt - Anh"),
        ("07", "Việt - Nga"),
        ("08", "Việt - Pháp"),
        ("09", "Hán Nôm"),
        ("10", "Việt - Trung"),
        ("11", "Khác"),
    ];
    /// Tên loại tài liệu (cột 32).
    pub const LOAI_VAN_BAN: &[(&str, &str)] = &[
        ("01", "Nghị quyết"),
        ("02", "Quyết định"),
        ("03", "Chỉ thị"),
        ("04", "Quy chế"),
        ("05", "Quy định"),
        ("06", "Thông cáo"),
        ("07", "Thông báo"),
        ("08", "Hướng dẫn"),
        ("09", "Chương trình"),
        ("10", "Kế hoạch"),
        ("11", "Phương án"),
        ("12", "Đề án"),
        ("13", "Dự án"),
        ("14", "Báo cáo"),
        ("15", "Tờ trình"),
        ("16", "Giấy ủy quyền"),
        ("17", "Phiếu gửi"),
        ("18", "Phiếu chuyển"),
        ("19", "Phiếu báo"),
        ("20", "Biên bản"),
        ("21", "Hợp đồng"),
        ("22", "Công văn"),
        ("23", "Công điện"),
        ("24", "Bản ghi nhớ"),
        ("25", "Bản thỏa thuận"),
        ("26", "Giấy mời"),
        ("27", "Giấy giới thiệu"),
        ("28", "Giấy nghỉ phép"),
        ("29", "Thư công"),
        ("30", "Bản đồ"),
        ("31", "Bản vẽ kỹ thuật"),
        ("32", "Khác"),
    ];
    /// 14 mức THBQ của bản cũ: (tên, trạng_thái, giá_trị năm, mã TT05).
    pub const THBQ_SEED: &[(&str, i16, i32, &str)] = &[
        ("Vĩnh viễn", 1, 0, "01"),
        ("5 năm", 2, 5, "07"),
        ("10 năm", 2, 10, "06"),
        ("15 năm", 2, 15, "07"),
        ("20 năm", 2, 20, "05"),
        ("50 năm", 2, 50, "03"),
        ("70 năm", 2, 70, "02"),
        ("05 năm đánh giá", 3, 0, "07"),
        ("10 năm đánh giá", 3, 0, "07"),
        ("15 năm đánh giá", 3, 0, "07"),
        ("20 năm đánh giá", 3, 0, "07"),
        ("50 năm đánh giá", 3, 0, "07"),
        ("70 năm đánh giá", 3, 0, "07"),
        ("Theo tuổi thọ công trình", 4, 0, "07"),
    ];
    /// Mã TT05 của một THBQ theo số năm (loại 2); loại 1 → 01; còn lại → 07.
    pub fn thbq_ma_tt05(trang_thai: i16, gia_tri: i32) -> &'static str {
        match (trang_thai, gia_tri) {
            (1, _) => "01",
            (2, 70) => "02",
            (2, 50) => "03",
            (2, 30) => "04",
            (2, 20) => "05",
            (2, 10) => "06",
            _ => "07",
        }
    }
    pub fn label(table: &[(&'static str, &'static str)], ma: &str) -> Option<&'static str> {
        table.iter().find(|(m, _)| *m == ma).map(|(_, l)| *l)
    }
}

// ------------------------------------------------------------------ đọc số thành chữ

/// Đọc số nguyên không âm thành chữ tiếng Việt (giọng Bắc: "linh", "mươi", "mốt", "tư", "lăm").
pub fn so_thanh_chu(n: u64) -> String {
    const DIGITS: [&str; 10] = ["không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín"];
    const UNITS: [&str; 5] = ["", "nghìn", "triệu", "tỷ", "nghìn tỷ"];
    if n == 0 {
        return "không".to_owned();
    }
    fn group(v: u64, full: bool, out: &mut Vec<String>) {
        let h = (v / 100) as usize;
        let t = ((v / 10) % 10) as usize;
        let u = (v % 10) as usize;
        if h > 0 || full {
            out.push(format!("{} trăm", DIGITS[h]));
        }
        match t {
            0 => {
                if u > 0 {
                    if h > 0 || full {
                        out.push("linh".to_owned());
                    }
                    out.push(DIGITS[u].to_owned());
                }
            }
            1 => {
                out.push("mười".to_owned());
                match u {
                    0 => {}
                    5 => out.push("lăm".to_owned()),
                    _ => out.push(DIGITS[u].to_owned()),
                }
            }
            _ => {
                out.push(format!("{} mươi", DIGITS[t]));
                match u {
                    0 => {}
                    1 => out.push("mốt".to_owned()),
                    4 => out.push("tư".to_owned()),
                    5 => out.push("lăm".to_owned()),
                    _ => out.push(DIGITS[u].to_owned()),
                }
            }
        }
    }
    let mut groups = Vec::new();
    let mut v = n;
    while v > 0 {
        groups.push(v % 1000);
        v /= 1000;
    }
    let top = groups.len() - 1;
    let mut out: Vec<String> = Vec::new();
    for (i, g) in groups.iter().enumerate().rev() {
        if *g == 0 {
            continue;
        }
        group(*g, i != top, &mut out);
        if i > 0 {
            out.push(UNITS[i.min(UNITS.len() - 1)].to_owned());
        }
    }
    out.join(" ")
}

/// Viết hoa chữ cái đầu ("một trăm" → "Một trăm").
pub fn hoa_dau_cau(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_ngay_cac_dinh_dang() {
        let d = |s: &str| ClDate::parse(s).unwrap().unwrap().to_text();
        assert_eq!(d("5/3/2021"), "05/03/2021");
        assert_eq!(d("05/03/2021"), "05/03/2021");
        assert_eq!(d("5-3-2021"), "05/03/2021");
        assert_eq!(d("5.3.2021"), "05/03/2021");
        assert_eq!(d("3/2021"), "03/2021");
        assert_eq!(d("2021"), "2021");
        assert_eq!(d("2021-03-05"), "05/03/2021");
        assert_eq!(d("2021-03-05T00:00:00"), "05/03/2021");
        assert_eq!(d("44260"), "05/03/2021"); // serial Excel
        assert_eq!(ClDate::parse("  ").unwrap(), None);
        assert!(matches!(ClDate::parse("31/02/2021"), Err(DateErr::Invalid(_))));
        assert!(matches!(ClDate::parse("13/2021"), Err(DateErr::Invalid(_))));
        assert!(matches!(ClDate::parse("1/1/1500"), Err(DateErr::Year(1500))));
        assert!(matches!(ClDate::parse("abc"), Err(DateErr::Format(_))));
        assert_eq!(ClDate::parse("2021").unwrap().unwrap().sort_key(), (2021, 1, 1));
        assert_eq!(ClDate::parse("29/02/2024").unwrap().unwrap().to_iso(), "2024-02-29");
    }

    #[test]
    fn ma_ho_so_dung_cong_thuc() {
        assert_eq!(ma_ho_so(Some("UBND"), Some("01"), Some(1999), Some("30"), Some("02")).as_deref(), Some("UBND.01.1999.30.02"));
        assert_eq!(ma_ho_so(None, None, Some(1999), Some("30"), None).as_deref(), Some("00.00.1999.30"));
        assert_eq!(ma_ho_so(Some("UBND"), Some("01"), Some(1999), Some("30"), Some("  ")).as_deref(), Some("UBND.01.1999.30"));
        assert_eq!(ma_ho_so(Some("UBND"), Some("01"), None, Some("30"), None), None);
        assert_eq!(ma_ho_so(Some("UBND"), Some("01"), Some(1999), None, None), None);
        assert_eq!(ma_van_ban_tt05("UBND.01.1999.30", 7), "UBND.01.1999.30.0000007");
    }

    #[test]
    fn trang_thai_thbq() {
        assert_eq!(thbq_status(Some(1), Some(0), None, 2026), Some("VV"));
        // 10 năm, kết thúc 2016: 2026-2016+1 = 11 > 10 → hết hạn; 2025 → 10, không > 10 → giữ lại
        assert_eq!(thbq_status(Some(2), Some(10), Some(2016), 2026), Some("HET"));
        assert_eq!(thbq_status(Some(2), Some(10), Some(2016), 2025), Some("GL"));
        assert_eq!(thbq_status(Some(2), Some(10), None, 2026), Some("GL"));
        assert_eq!(thbq_status(Some(3), Some(0), Some(1990), 2026), Some("GL"));
        assert_eq!(thbq_status(Some(4), Some(0), Some(1990), 2026), Some("GL"));
        assert_eq!(thbq_status(None, None, Some(1990), 2026), None);
    }

    #[test]
    fn so_sanh_tu_nhien() {
        assert_eq!(natural_cmp("2", "10"), Ordering::Less);
        assert_eq!(natural_cmp("10", "10A"), Ordering::Less);
        assert_eq!(natural_cmp("572A", "572a"), Ordering::Equal);
        assert_eq!(natural_cmp("010", "10"), Ordering::Equal);
        assert_eq!(natural_cmp("9", "12A"), Ordering::Less);
        assert_eq!(natural_cmp("b", "a"), Ordering::Greater);
    }

    fn it(key: &str, thbq: i32, mang: i32, kt: Option<(i32, u32, u32)>, so_tam: &str) -> SortItem<String> {
        SortItem { key: key.to_owned(), thbq_sort: thbq, mang_sort: mang, mang_ten: format!("M{mang}"), ngay_kt: kt, so_tam: so_tam.to_owned() }
    }
    fn keys(v: &[SortItem<String>]) -> String {
        v.iter().map(|i| i.key.as_str()).collect::<Vec<_>>().join(",")
    }

    #[test]
    fn sau_thu_tu_he_thong_hoa() {
        let base = vec![
            it("a", 2, 1, Some((2020, 1, 1)), "10"),
            it("b", 1, 2, Some((2019, 5, 1)), "2"),
            it("c", 3, 1, None, "3"),
            it("d", 2, 2, Some((2019, 5, 1)), "1"),
            it("e", 1, 1, Some((2021, 1, 1)), "5"),
        ];
        let run = |o: SortOrder| {
            let mut v = base.clone();
            sort_items(&mut v, o);
            keys(&v)
        };
        assert_eq!(run(SortOrder::MangThoiGian), "a,e,c,d,b");
        assert_eq!(run(SortOrder::ThoiGianMang), "d,b,a,e,c");
        assert_eq!(run(SortOrder::ThbqAscMangThoiGian), "e,b,a,d,c");
        assert_eq!(run(SortOrder::ThbqDescMangThoiGian), "c,a,d,e,b");
        assert_eq!(run(SortOrder::ThbqAscThoiGianMang), "b,e,d,a,c");
        assert_eq!(run(SortOrder::ThbqDescThoiGianMang), "c,d,a,b,e");
        // hoà mọi khoá → số tạm theo số tự nhiên
        let mut v = vec![it("x", 1, 1, None, "10"), it("y", 1, 1, None, "9")];
        sort_items(&mut v, SortOrder::MangThoiGian);
        assert_eq!(keys(&v), "y,x");
    }

    #[test]
    fn doc_so_thanh_chu() {
        assert_eq!(so_thanh_chu(0), "không");
        assert_eq!(so_thanh_chu(5), "năm");
        assert_eq!(so_thanh_chu(15), "mười lăm");
        assert_eq!(so_thanh_chu(21), "hai mươi mốt");
        assert_eq!(so_thanh_chu(24), "hai mươi tư");
        assert_eq!(so_thanh_chu(25), "hai mươi lăm");
        assert_eq!(so_thanh_chu(105), "một trăm linh năm");
        assert_eq!(so_thanh_chu(1005), "một nghìn không trăm linh năm");
        assert_eq!(so_thanh_chu(12345), "mười hai nghìn ba trăm bốn mươi lăm");
        assert_eq!(so_thanh_chu(100000), "một trăm nghìn");
        assert_eq!(so_thanh_chu(1000001), "một triệu không trăm linh một");
        assert_eq!(hoa_dau_cau("một trăm"), "Một trăm");
    }

    #[test]
    fn ma_tt05_thbq() {
        assert_eq!(codes::thbq_ma_tt05(1, 0), "01");
        assert_eq!(codes::thbq_ma_tt05(2, 70), "02");
        assert_eq!(codes::thbq_ma_tt05(2, 5), "07");
        assert_eq!(codes::label(codes::LOAI_VAN_BAN, "22"), Some("Công văn"));
        assert_eq!(codes::THBQ_SEED.len(), 14);
        assert_eq!(codes::LOAI_VAN_BAN.len(), 32);
        assert_eq!(codes::NGON_NGU.len(), 11);
    }
}
