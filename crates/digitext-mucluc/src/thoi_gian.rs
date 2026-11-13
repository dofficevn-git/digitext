// Copyright (c) 2026 DigiText. All rights reserved.
//! Kiểm + tự điền cột "Thời gian BĐ - KT" của mục lục hồ sơ.
//!
//! Bản Rust của `desktop/src/features/chinhly/thoiGianHoSo.ts`, **bám từng dòng**, kể cả câu
//! ghi chú — vì bộ đối chứng 23 mục lục Bắc Giang so cả chuỗi thông báo. Đừng "viết lại cho
//! gọn": lệch một chữ là lệch bản đối chứng.
//!
//! Quy tắc điền: `2020` → 01/01/2020–31/12/2020 · `2015-2019` → 01/01/2015–31/12/2019 ·
//! `04/2020` → 01/04/2020–30/04/2020 · một ngày → lặp hai mốc · đầu > cuối → đảo ·
//! mốc sai nhưng còn mốc đúng → giữ mốc đúng, mốc thiếu suy từ năm trong ô · ô trống → năm
//! trong tiêu đề · vẫn không ra → không điền.
//!
//! **Hai chỗ JS làm được mà Rust không**: `regex` của Rust không có lookbehind/lookahead, nên
//! `(?<!\d)(\d{4})(?!\d)` phải tự quét (xem `nam_trong_chuoi`, `nam_bare`). Chỗ có tiền tố
//! "năm" thì không cần biên nên vẫn dùng `regex` được — và phải KHÔNG thêm biên, vì bản TS
//! cũng không có: `"năm 20201"` ở cả hai bên đều ra 2020.

use std::sync::OnceLock;

const MIN_YEAR: i32 = 1800;
const MAX_YEAR: i32 = 2100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KetLuan {
    Dung,
    DaDien,
    CanhBao,
    Loi,
}

impl KetLuan {
    pub fn nhan(self) -> &'static str {
        match self {
            KetLuan::Dung => "Đúng",
            KetLuan::DaDien => "Đã điền",
            KetLuan::CanhBao => "Cảnh báo",
            KetLuan::Loi => "Lỗi",
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KetQuaThoiGian {
    /// `dd/mm/yyyy`, hoặc rỗng khi không điền được.
    pub bat_dau: String,
    pub ket_thuc: String,
    pub ket_luan: KetLuanOpt,
    pub ghi_chu: Vec<String>,
}

/// `KetLuan` có `Default` = `Dung`, tách ra để `KetQuaThoiGian` dẫn xuất được `Default`.
pub type KetLuanOpt = KetLuan;
impl Default for KetLuan {
    fn default() -> Self {
        KetLuan::Dung
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaiMoc {
    Ngay,
    Thang,
    Nam,
    Sai,
}

#[derive(Debug, Clone)]
pub struct Moc {
    pub loai: LoaiMoc,
    pub raw: String,
    pub y: i32,
    pub m: u32,
    pub d: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ymd {
    y: i32,
    m: u32,
    d: u32,
}

impl Ymd {
    fn fmt(self) -> String {
        format!("{:02}/{:02}/{}", self.d, self.m, self.y)
    }
    fn cmp_ymd(self, b: Ymd) -> i32 {
        if self.y != b.y {
            return self.y - b.y;
        }
        if self.m != b.m {
            return self.m as i32 - b.m as i32;
        }
        self.d as i32 - b.d as i32
    }
}

/// Bỏ dấu nháy đơn Excel ở MỌI vị trí (`'05/12/2025`, `2016-'2018`) rồi trim.
pub fn bo_nhay(s: &str) -> String {
    s.chars().filter(|c| !matches!(c, '\'' | '\u{2019}' | '\u{2018}' | '`')).collect::<String>().trim().to_owned()
}

pub fn so_ngay_trong_thang(y: i32, m: u32) -> u32 {
    if m == 2 {
        return if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 29 } else { 28 };
    }
    if matches!(m, 4 | 6 | 9 | 11) {
        30
    } else {
        31
    }
}

fn ngay_hop_le(y: i32, m: u32, d: u32) -> bool {
    (MIN_YEAR..=MAX_YEAR).contains(&y) && (1..=12).contains(&m) && d >= 1 && d <= so_ngay_trong_thang(y, m)
}

/// Từ nối bị bỏ khi tách mốc — so không phân biệt chữ hoa, đúng như `/i` của TS.
const TU_NOI: &[&str] = &["từ", "đến", "tới", "tu", "den", "toi", "to", "from"];

/// Tách ô thành các mốc theo dấu gạch / xuống dòng / khoảng trắng / `;` / `,`.
pub fn tach_moc(raw: &str) -> Vec<Moc> {
    let s = bo_nhay(raw);
    if s.is_empty() {
        return Vec::new();
    }
    s.split(|c: char| matches!(c, '-' | '\u{2013}' | '\u{2014}' | ';' | ',') || c.is_whitespace())
        .map(str::trim)
        .filter(|t| !t.is_empty() && !TU_NOI.iter().any(|n| n.eq_ignore_ascii_case(t) || *n == t.to_lowercase()))
        .map(doc_mot_moc)
        .collect()
}

fn doc_mot_moc(t: &str) -> Moc {
    let sai = |y: i32| Moc { loai: LoaiMoc::Sai, raw: t.to_owned(), y, m: 0, d: 0 };
    // "12.5.2020" → "12/5/2020" (chỉ khi khớp đúng khuôn, như TS)
    let tok = if la_ngay_dau_cham(t) { t.replace('.', "/") } else { t.to_owned() };

    if let Some((a, b, c)) = ba_so_cach_bang(&tok) {
        let (d, m, y) = (a, b, c);
        if d.1 <= 2 && m.1 <= 2 && y.1 == 4 {
            let (d, m, y) = (d.0 as u32, m.0 as u32, y.0);
            return if ngay_hop_le(y, m, d) {
                Moc { loai: LoaiMoc::Ngay, raw: t.to_owned(), y, m, d }
            } else {
                sai(if (MIN_YEAR..=MAX_YEAR).contains(&y) { y } else { 0 })
            };
        }
        return sai(0);
    }
    if let Some((a, b)) = hai_so_cach_bang(&tok) {
        if a.1 <= 2 && b.1 == 4 {
            let (m, y) = (a.0 as u32, b.0);
            return if (1..=12).contains(&m) && (MIN_YEAR..=MAX_YEAR).contains(&y) {
                Moc { loai: LoaiMoc::Thang, raw: t.to_owned(), y, m, d: 0 }
            } else {
                sai(0)
            };
        }
        return sai(0);
    }
    if tok.len() == 4 && tok.bytes().all(|b| b.is_ascii_digit()) {
        let y: i32 = tok.parse().unwrap_or(0);
        return if (MIN_YEAR..=MAX_YEAR).contains(&y) {
            Moc { loai: LoaiMoc::Nam, raw: t.to_owned(), y, m: 0, d: 0 }
        } else {
            sai(0)
        };
    }
    sai(0)
}

/// `^\d{1,2}\.\d{1,2}\.\d{4}$`
fn la_ngay_dau_cham(t: &str) -> bool {
    let p: Vec<&str> = t.split('.').collect();
    p.len() == 3
        && (1..=2).contains(&p[0].len())
        && (1..=2).contains(&p[1].len())
        && p[2].len() == 4
        && p.iter().all(|x| !x.is_empty() && x.bytes().all(|b| b.is_ascii_digit()))
}

/// `^(\d+)\s*/\s*(\d+)\s*/\s*(\d+)$` → `((giá trị, số chữ số), …)`. Khoảng trắng quanh `/`
/// được bỏ, đúng như `\s*` của TS.
fn ba_so_cach_bang(t: &str) -> Option<((i32, usize), (i32, usize), (i32, usize))> {
    let p: Vec<&str> = t.split('/').collect();
    if p.len() != 3 {
        return None;
    }
    let a = so_thuan(p[0])?;
    let b = so_thuan(p[1])?;
    let c = so_thuan(p[2])?;
    Some((a, b, c))
}

fn hai_so_cach_bang(t: &str) -> Option<((i32, usize), (i32, usize))> {
    let p: Vec<&str> = t.split('/').collect();
    if p.len() != 2 {
        return None;
    }
    Some((so_thuan(p[0])?, so_thuan(p[1])?))
}

/// Chuỗi sau khi trim phải là toàn chữ số; trả `(giá trị, số chữ số)`.
fn so_thuan(s: &str) -> Option<(i32, usize)> {
    let t = s.trim();
    if t.is_empty() || !t.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((t.parse().ok()?, t.len()))
}

/// Mốc → ngày bắt đầu: `06/2013` → 01/06/2013, `2013` → 01/01/2013.
fn moc_dau(m: &Moc) -> Ymd {
    match m.loai {
        LoaiMoc::Ngay => Ymd { y: m.y, m: m.m, d: m.d },
        LoaiMoc::Thang => Ymd { y: m.y, m: m.m, d: 1 },
        _ => Ymd { y: m.y, m: 1, d: 1 },
    }
}

/// Mốc → ngày kết thúc: `06/2013` → 30/06/2013, `2013` → 31/12/2013.
fn moc_cuoi(m: &Moc) -> Ymd {
    match m.loai {
        LoaiMoc::Ngay => Ymd { y: m.y, m: m.m, d: m.d },
        LoaiMoc::Thang => Ymd { y: m.y, m: m.m, d: so_ngay_trong_thang(m.y, m.m) },
        _ => Ymd { y: m.y, m: 12, d: 31 },
    }
}

/// Mọi cụm **đúng 4 chữ số** (không dính chữ số hai bên) **trong dải năm**, theo thứ tự.
/// Đây là chỗ thay cho `(?<!\d)(\d{4})(?!\d)` của JS.
pub fn nam_trong_chuoi(s: &str) -> Vec<i32> {
    quet_cum_4(s).into_iter().filter(|(y, _, _)| (MIN_YEAR..=MAX_YEAR).contains(y)).map(|(y, _, _)| y).collect()
}

/// `(giá trị, đầu, cuối)` theo byte của từng cụm **đúng 4 chữ số**, **không lọc dải năm**.
///
/// Không lọc ở đây là có chủ ý và là chỗ đã sập một lần (10/11/2026, dòng 295 của «PHÔNG SỞ
/// NÔNG NGHIỆP … BẮC GIANG»): biểu thức của JS **ăn trọn** `1428-1856` thành một lần khớp rồi
/// mới loại vì 1428 ngoài dải, nên `1856` không bao giờ được xét riêng. Lọc dải trước khi ghép
/// thì `1856` lọt vào và thành "năm tài liệu ở tiêu đề" → sai một cảnh báo. Việc loại theo dải
/// phải làm Ở THÂN VÒNG LẶP, đúng chỗ `continue` của JS.
fn quet_cum_4(s: &str) -> Vec<(i32, usize, usize)> {
    let b = s.as_bytes();
    let mut ra = Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        if !b[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let dau = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i - dau == 4 {
            if let Ok(y) = s[dau..i].parse::<i32>() {
                ra.push((y, dau, i));
            }
        }
    }
    ra
}

#[derive(Debug, Clone)]
pub struct NamTieuDe {
    pub y1: i32,
    pub y2: i32,
    pub raw: String,
}

fn re_nhiem_ky() -> &'static regex::Regex {
    static R: OnceLock<regex::Regex> = OnceLock::new();
    R.get_or_init(|| regex::Regex::new(r"(?i)nhi[ệe]m\s*k[ỳy]\s*\d{4}(?:\s*[-–—]\s*\d{4})?").unwrap())
}

fn re_nam() -> &'static regex::Regex {
    static R: OnceLock<regex::Regex> = OnceLock::new();
    R.get_or_init(|| regex::Regex::new(r"(?i)n[ăa]m\s*(\d{4})(?:\s*[-–—]\s*(\d{4}))?").unwrap())
}

/// Dải năm tài liệu ghi ở tiêu đề: cụm năm ngay sau chữ "năm", không có thì cụm năm **cuối
/// cùng**. Bỏ "nhiệm kỳ 2020 - 2025" (tên đại hội, không phải thời gian tài liệu).
pub fn nam_trong_tieu_de(tieu_de: &str) -> Option<NamTieuDe> {
    let s = re_nhiem_ky().replace_all(tieu_de, " ").into_owned();

    let mut cuoi_co_chu_nam: Option<NamTieuDe> = None;
    for c in re_nam().captures_iter(&s) {
        let y1: i32 = c.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
        if !(MIN_YEAR..=MAX_YEAR).contains(&y1) {
            continue;
        }
        let y2 = c.get(2).and_then(|m| m.as_str().parse::<i32>().ok()).unwrap_or(y1);
        let y2 = if y2 >= y1 && y2 <= MAX_YEAR { y2 } else { y1 };
        cuoi_co_chu_nam = Some(NamTieuDe { y1, y2, raw: c.get(0).unwrap().as_str().trim().to_owned() });
    }
    if cuoi_co_chu_nam.is_some() {
        return cuoi_co_chu_nam;
    }

    // Không có chữ "năm": lấy cụm năm cuối cùng, có thể kèm "- yyyy".
    let moc = quet_cum_4(&s);
    let mut cuoi: Option<NamTieuDe> = None;
    let mut i = 0usize;
    while i < moc.len() {
        let (y1, dau, het) = moc[i];
        // Sau cụm này có "[-–—] yyyy" liền kề (chỉ cách bằng khoảng trắng) thì ăn luôn — kể cả
        // khi y1 ngoài dải năm, vì bên JS lần khớp vẫn tiêu thụ cả hai cụm (xem `quet_cum_4`).
        let mut y2 = y1;
        let mut raw_het = het;
        let mut nhay = 1usize;
        if i + 1 < moc.len() {
            let (yn, dau_n, het_n) = moc[i + 1];
            let giua = &s[het..dau_n];
            if !giua.is_empty()
                && giua.chars().all(|c| c.is_whitespace() || matches!(c, '-' | '\u{2013}' | '\u{2014}'))
                && giua.chars().any(|c| matches!(c, '-' | '\u{2013}' | '\u{2014}'))
            {
                y2 = yn;
                raw_het = het_n;
                nhay = 2;
            }
        }
        i += nhay;
        // Đúng chỗ `continue` của JS: loại theo dải năm SAU khi đã tiêu thụ cụm ghép.
        if !(MIN_YEAR..=MAX_YEAR).contains(&y1) {
            continue;
        }
        let y2 = if y2 >= y1 && y2 <= MAX_YEAR { y2 } else { y1 };
        cuoi = Some(NamTieuDe { y1, y2, raw: s[dau..raw_het].trim().to_owned() });
    }
    cuoi
}

/// Kiểm và điền một ô thời gian. `tieu_de` dùng để suy năm khi ô thiếu và để đối chiếu.
pub fn kiem_va_dien_thoi_gian(raw: &str, tieu_de: &str) -> KetQuaThoiGian {
    let mut ghi_chu: Vec<String> = Vec::new();
    let mut ket_luan = KetLuan::Dung;
    macro_rules! nang {
        ($k:expr) => {
            if $k > ket_luan {
                ket_luan = $k;
            }
        };
    }

    let goc = bo_nhay(raw);
    let mocs = tach_moc(&goc);
    let sai: Vec<&Moc> = mocs.iter().filter(|m| m.loai == LoaiMoc::Sai).collect();
    let dung: Vec<&Moc> = mocs.iter().filter(|m| m.loai != LoaiMoc::Sai).collect();
    let td_nam = nam_trong_tieu_de(tieu_de);

    let mut bd: Option<Ymd> = None;
    let mut kt: Option<Ymd> = None;

    if goc.is_empty() {
        match &td_nam {
            Some(t) => {
                bd = Some(Ymd { y: t.y1, m: 1, d: 1 });
                kt = Some(Ymd { y: t.y2, m: 12, d: 31 });
                ghi_chu.push(format!("Ô trống — điền theo năm trong tiêu đề ({})", t.raw));
                nang!(KetLuan::DaDien);
            }
            None => {
                ghi_chu.push("Ô thời gian trống và tiêu đề không ghi năm — không điền được".into());
                nang!(KetLuan::Loi);
            }
        }
    } else if sai.is_empty() && !dung.is_empty() {
        bd = Some(moc_dau(dung[0]));
        kt = Some(moc_cuoi(dung[dung.len() - 1]));
        if dung.len() == 1 {
            ghi_chu.push(
                match dung[0].loai {
                    LoaiMoc::Ngay => "Chỉ ghi 1 ngày — lặp lại thành 2 mốc",
                    LoaiMoc::Thang => "Chỉ ghi tháng/năm — trải cả tháng",
                    _ => "Chỉ ghi năm — trải cả năm",
                }
                .into(),
            );
            nang!(KetLuan::DaDien);
        } else {
            let thieu: Vec<&&Moc> = dung.iter().filter(|m| m.loai != LoaiMoc::Ngay).collect();
            if !thieu.is_empty() {
                let la_nam = thieu[0].loai == LoaiMoc::Nam;
                ghi_chu.push(format!(
                    "Mốc {} chỉ ghi {} — trải cả {}",
                    thieu.iter().map(|m| format!("'{}'", m.raw)).collect::<Vec<_>>().join(", "),
                    if la_nam { "năm" } else { "tháng/năm" },
                    if la_nam { "năm" } else { "tháng" }
                ));
                nang!(KetLuan::DaDien);
            }
        }
        if let (Some(a), Some(b)) = (bd, kt) {
            if a.cmp_ymd(b) > 0 {
                bd = Some(b);
                kt = Some(a);
                ghi_chu.push("Ngày đầu sau ngày cuối — đã đảo lại thứ tự".into());
                nang!(KetLuan::CanhBao);
            }
        }
    } else {
        for m in &sai {
            ghi_chu.push(format!("Sai định dạng: '{}' không khớp dd/mm/yyyy", m.raw));
        }
        nang!(KetLuan::CanhBao);
        let nam = nam_trong_chuoi(&goc);
        let y_min = nam.iter().copied().min();
        let y_max = nam.iter().copied().max();
        let dau_dung = mocs.first().filter(|m| m.loai != LoaiMoc::Sai);
        let cuoi_dung = mocs.last().filter(|m| m.loai != LoaiMoc::Sai);
        match (y_min, y_max) {
            (Some(lo), Some(hi)) => {
                bd = Some(dau_dung.map_or(Ymd { y: lo, m: 1, d: 1 }, moc_dau));
                kt = Some(cuoi_dung.map_or(Ymd { y: hi, m: 12, d: 31 }, moc_cuoi));
                ghi_chu.push(
                    if dau_dung.is_some() || cuoi_dung.is_some() {
                        "Giữ mốc đúng, mốc thiếu suy từ năm trong ô"
                    } else {
                        "Giá trị sai — điền theo năm trong ô"
                    }
                    .into(),
                );
            }
            _ => match &td_nam {
                Some(t) => {
                    bd = Some(Ymd { y: t.y1, m: 1, d: 1 });
                    kt = Some(Ymd { y: t.y2, m: 12, d: 31 });
                    ghi_chu.push(format!("Không đọc được năm trong ô — điền theo năm trong tiêu đề ({})", t.raw));
                }
                None => {
                    ghi_chu.push("Không đọc được năm trong ô lẫn tiêu đề — không điền được, cần sửa tay".into());
                    nang!(KetLuan::Loi);
                }
            },
        }
        if let (Some(a), Some(b)) = (bd, kt) {
            if a.cmp_ymd(b) > 0 {
                bd = Some(b);
                kt = Some(a);
                ghi_chu.push("Ngày đầu sau ngày cuối — đã đảo lại thứ tự".into());
            }
        }
    }

    if let (Some(a), Some(b)) = (bd, kt) {
        match &td_nam {
            Some(t) => {
                let mut ngoai: Vec<i32> = [a.y, b.y].into_iter().filter(|y| *y < t.y1 || *y > t.y2).collect();
                if !ngoai.is_empty() {
                    ngoai.dedup();
                    ghi_chu.push(format!(
                        "Năm {} không nằm trong năm tài liệu ở tiêu đề ({})",
                        ngoai.iter().map(i32::to_string).collect::<Vec<_>>().join(", "),
                        t.raw
                    ));
                    nang!(KetLuan::CanhBao);
                }
            }
            None => {
                if !goc.is_empty() {
                    ghi_chu.push("Tiêu đề không ghi năm tài liệu để đối chiếu".into());
                    nang!(KetLuan::CanhBao);
                }
            }
        }
    }

    KetQuaThoiGian {
        bat_dau: bd.map(Ymd::fmt).unwrap_or_default(),
        ket_thuc: kt.map(Ymd::fmt).unwrap_or_default(),
        ket_luan,
        ghi_chu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bd_kt(raw: &str, td: &str) -> (String, String, KetLuan) {
        let r = kiem_va_dien_thoi_gian(raw, td);
        (r.bat_dau, r.ket_thuc, r.ket_luan)
    }

    #[test]
    fn bo_nhay_moi_vi_tri() {
        assert_eq!(bo_nhay("'05/12/2025"), "05/12/2025");
        assert_eq!(bo_nhay("2016-'2018"), "2016-2018");
        assert_eq!(bo_nhay("  '2020  "), "2020");
    }

    #[test]
    fn mot_nam_trai_ca_nam() {
        assert_eq!(bd_kt("2020", "Hồ sơ năm 2020"), ("01/01/2020".into(), "31/12/2020".into(), KetLuan::DaDien));
    }

    #[test]
    fn dai_nam() {
        assert_eq!(bd_kt("2015-2019", "Tập lưu năm 2015 - 2019"), ("01/01/2015".into(), "31/12/2019".into(), KetLuan::DaDien));
    }

    #[test]
    fn thang_nam_trai_ca_thang() {
        assert_eq!(bd_kt("04/2020", "Báo cáo năm 2020"), ("01/04/2020".into(), "30/04/2020".into(), KetLuan::DaDien));
        // tháng 2 năm nhuận
        assert_eq!(bd_kt("02/2020", "năm 2020").1, "29/02/2020");
        assert_eq!(bd_kt("02/2021", "năm 2021").1, "28/02/2021");
    }

    #[test]
    fn mot_ngay_lap_hai_moc() {
        assert_eq!(bd_kt("04/05/2020", "Công văn năm 2020"), ("04/05/2020".into(), "04/05/2020".into(), KetLuan::DaDien));
    }

    #[test]
    fn ngay_dau_cham_duoc_nhan() {
        assert_eq!(bd_kt("12.5.2020", "năm 2020").0, "12/05/2020");
    }

    #[test]
    fn dau_sau_cuoi_thi_dao() {
        let r = kiem_va_dien_thoi_gian("31/12/2020 - 01/01/2020", "năm 2020");
        assert_eq!((r.bat_dau.as_str(), r.ket_thuc.as_str()), ("01/01/2020", "31/12/2020"));
        assert_eq!(r.ket_luan, KetLuan::CanhBao);
        assert!(r.ghi_chu.iter().any(|g| g.contains("đã đảo lại thứ tự")));
    }

    #[test]
    fn o_trong_suy_tu_tieu_de() {
        let r = kiem_va_dien_thoi_gian("", "Sổ theo dõi năm 1997 - 2017");
        assert_eq!((r.bat_dau.as_str(), r.ket_thuc.as_str()), ("01/01/1997", "31/12/2017"));
        assert_eq!(r.ket_luan, KetLuan::DaDien);
    }

    #[test]
    fn o_trong_tieu_de_khong_co_nam_thi_loi() {
        let r = kiem_va_dien_thoi_gian("", "Hồ sơ không ghi năm");
        assert_eq!(r.ket_luan, KetLuan::Loi);
        assert_eq!(r.bat_dau, "");
    }

    /// "nhiệm kỳ 2020 - 2025" là tên đại hội, KHÔNG phải thời gian tài liệu.
    #[test]
    fn bo_nhiem_ky() {
        let t = nam_trong_tieu_de("Đại hội nhiệm kỳ 2020 - 2025, tài liệu năm 2019").unwrap();
        assert_eq!((t.y1, t.y2), (2019, 2019));
        // Chỉ có nhiệm kỳ thì không còn năm nào
        assert!(nam_trong_tieu_de("Đại hội nhiệm kỳ 2020-2025").is_none());
    }

    /// Không có chữ "năm" thì lấy cụm năm CUỐI CÙNG, và ăn cả "- yyyy" liền kề.
    #[test]
    fn khong_co_chu_nam_thi_lay_cum_cuoi() {
        let t = nam_trong_tieu_de("Hồ sơ 1995 về việc 2010 - 2012").unwrap();
        assert_eq!((t.y1, t.y2), (2010, 2012));
        let t2 = nam_trong_tieu_de("Tập 2008").unwrap();
        assert_eq!((t2.y1, t2.y2), (2008, 2008));
    }

    /// Có chữ "năm" thì cụm sau chữ "năm" thắng, dù đứng trước cụm khác.
    #[test]
    fn cum_sau_chu_nam_thang() {
        let t = nam_trong_tieu_de("Quyết định năm 2001 ban hành 2015").unwrap();
        assert_eq!((t.y1, t.y2), (2001, 2001));
    }

    #[test]
    fn nam_ngoai_dai_bi_bo() {
        assert!(nam_trong_tieu_de("Hồ sơ 1700").is_none());
        assert!(nam_trong_tieu_de("Hồ sơ 2200").is_none());
        assert_eq!(nam_trong_chuoi("12345 1999 123456"), vec![1999]);
    }

    /// Cụm 5 chữ số KHÔNG phải năm — đây là chỗ JS dùng lookbehind.
    #[test]
    fn cum_nam_chu_so_khong_phai_nam() {
        assert!(nam_trong_chuoi("20201").is_empty());
        assert_eq!(nam_trong_chuoi("a2020b"), vec![2020]);
    }

    /// Hồi quy thật: dòng 295 của «PHÔNG SỞ NÔNG NGHIỆP VÀ PTNT BẮC GIANG» (10/11/2026).
    /// `1428-1856` là **số văn bản**, không phải năm. Bên JS lần khớp ăn trọn cả cặp rồi mới
    /// loại vì 1428 ngoài dải, nên 1856 không bao giờ thành "năm tài liệu". Lọc dải trước khi
    /// ghép thì sinh thêm một cảnh báo sai.
    #[test]
    fn so_van_ban_khong_bi_hieu_thanh_nam() {
        let td = "Tập lưu Quyết định của Sở Nông nghiệp và PTNT tháng 10-11/2008 Từ số: 1428-1856";
        let t = nam_trong_tieu_de(td).expect("phải lấy được năm");
        assert_eq!((t.y1, t.y2), (2008, 2008), "1856 không được thành năm tài liệu");

        let r = kiem_va_dien_thoi_gian("01/10/2008\n28/11/2008", td);
        assert_eq!(r.ket_luan, KetLuan::Dung, "không được sinh cảnh báo nào");
        assert_eq!((r.bat_dau.as_str(), r.ket_thuc.as_str()), ("01/10/2008", "28/11/2008"));
    }

    /// Cặp "yyyy-yyyy" mà cụm đầu ngoài dải thì BỎ CẢ CẶP, không lấy cụm sau.
    #[test]
    fn cap_nam_dau_ngoai_dai_thi_bo_ca_cap() {
        assert!(nam_trong_tieu_de("Từ số: 1428-1856").is_none());
        // nhưng hai cụm KHÔNG ghép bằng gạch thì cụm sau vẫn được xét
        let t = nam_trong_tieu_de("Từ số 1428 đến 1856").unwrap();
        assert_eq!((t.y1, t.y2), (1856, 1856));
    }

    #[test]
    fn sai_dinh_dang_nhung_con_nam_trong_o() {
        let r = kiem_va_dien_thoi_gian("32/13/2020", "năm 2020");
        assert_eq!(r.ket_luan, KetLuan::CanhBao);
        assert_eq!((r.bat_dau.as_str(), r.ket_thuc.as_str()), ("01/01/2020", "31/12/2020"));
        assert!(r.ghi_chu.iter().any(|g| g.starts_with("Sai định dạng:")));
    }

    #[test]
    fn tu_noi_bi_bo() {
        let m = tach_moc("từ 2015 đến 2019");
        assert_eq!(m.len(), 2);
        assert_eq!((m[0].y, m[1].y), (2015, 2019));
    }

    #[test]
    fn nam_ngoai_dai_tieu_de_thi_canh_bao() {
        let r = kiem_va_dien_thoi_gian("01/01/1990", "Hồ sơ năm 2020");
        assert_eq!(r.ket_luan, KetLuan::CanhBao);
        assert!(r.ghi_chu.iter().any(|g| g.contains("không nằm trong năm tài liệu")));
    }

    #[test]
    fn tieu_de_khong_co_nam_de_doi_chieu() {
        let r = kiem_va_dien_thoi_gian("2020", "Hồ sơ chung");
        assert_eq!(r.ket_luan, KetLuan::CanhBao);
        assert!(r.ghi_chu.iter().any(|g| g.contains("để đối chiếu")));
    }
}
