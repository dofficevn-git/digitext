// Copyright (c) 2026 DigiText. All rights reserved.
//! Đọc mục lục hồ sơ (MLHS) từ tệp Excel — tầng ô và tầng cột.
//!
//! Bản Rust của `desktop/src/features/chinhly/mucLucImport.ts`. Tầng này là chỗ dễ lệch nhất
//! giữa hai bản: bên TS dùng SheetJS, bên này dùng calamine, và hai thư viện không trình bày ô
//! giống nhau. Ba chỗ đã phải xử riêng, ghi lại để đừng ai "dọn" mất:
//!
//! 1. **Lưới phải bắt đầu từ A1.** SheetJS được gọi với `range: { s: {r:0,c:0}, e: ref.e }` nên
//!    luôn đệm từ A1. `calamine::Range` thì trả đúng vùng đã dùng, có thể bắt đầu ở C5 ⇒ chỉ số
//!    cột lệch. Phải đệm lại theo `range.start()`.
//! 2. **Số nguyên dạng thực.** SheetJS `raw: true` đưa ra số JS, `String(5)` = `"5"`. calamine
//!    đưa `Float(5.0)`, `to_string()` = `"5"`, nhưng `Float(5.5)` phải ra `"5.5"` — nên phải tự
//!    định dạng, đừng tin mặc định.
//! 3. **Ô ngày.** TS đọc với `cellDates: true` nên ô định dạng ngày thành `Date` → `dd/mm/yyyy`.
//!    calamine cần feature `dates` và trả `Data::DateTime`.
//!
//! Đối chứng bắt buộc trước khi tin bản này: xem đầu `Cargo.toml` của crate.

pub mod mau;
pub mod phan_tich;
pub mod thoi_gian;

use calamine::{Data, Reader};
use chrono::Datelike;
use std::path::Path;

/// Bỏ dấu + hạ chữ thường + nén khoảng trắng. Khuôn `boDau` bên TS (`mucLucImport.ts:122`).
pub fn bo_dau(s: &str) -> String {
    let khong_dau = digitext_vi::f_2hu(&s.to_lowercase());
    let mut ra = String::with_capacity(khong_dau.len());
    let mut vua_cach = false;
    for c in khong_dau.chars() {
        if c.is_whitespace() {
            vua_cach = true;
        } else {
            if vua_cach && !ra.is_empty() {
                ra.push(' ');
            }
            vua_cach = false;
            ra.push(c);
        }
    }
    ra
}

/// Tên cột để so: bỏ dấu rồi bỏ MỌI thứ không phải chữ/số ("TH\nBQ" → "thbq", "N.BĐ" → "nbd").
pub fn chuan_ten_cot(s: &str) -> String {
    bo_dau(s).chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

/// Ô Excel → chuỗi. `la_cot_ngay` chỉ ảnh hưởng số serial, đúng như TS.
pub fn cell_text(d: &Data, la_cot_ngay: bool) -> String {
    match d {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_owned(),
        Data::Bool(b) => b.to_string(),
        Data::Int(n) => n.to_string(),
        Data::Float(f) => so_nhu_js(*f, la_cot_ngay),
        Data::DateTime(dt) => match dt.as_datetime() {
            Some(t) => format!("{:02}/{:02}/{}", t.day(), t.month(), t.year()),
            None => so_nhu_js(dt.as_f64(), la_cot_ngay),
        },
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.trim().to_owned(),
        Data::Error(_) => String::new(),
    }
}

/// Số như `String(v)` của JS, cộng luật serial ngày của TS (`20000 < v < 80000`, nguyên).
fn so_nhu_js(f: f64, la_cot_ngay: bool) -> String {
    if !f.is_finite() {
        return String::new();
    }
    let nguyen = f.fract() == 0.0;
    if la_cot_ngay && nguyen && f > 20000.0 && f < 80000.0 {
        // serial Excel: 1899-12-30 + n (TS dùng mốc 25569 = 1970-01-01)
        let ngay = f as i64 - 25569;
        if let Some(t) = ngay_tu_epoch(ngay) {
            return format!("{:02}/{:02}/{}", t.2, t.1, t.0);
        }
    }
    if nguyen && f.abs() < 1e21 {
        format!("{}", f as i64)
    } else {
        let s = format!("{f}");
        s
    }
}

/// (năm, tháng, ngày) từ số ngày kể từ 1970-01-01. Thuật toán ngày Julian, không cần chrono.
fn ngay_tu_epoch(ngay: i64) -> Option<(i64, u32, u32)> {
    let z = ngay.checked_add(719_468)?;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as i64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    Some((if m <= 2 { y + 1 } else { y }, m, d))
}

/// Chỉ số cột (0-based) của từng trường trong dòng tiêu đề.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
/// Bỏ hẳn khoá khi cột không có, KHÔNG trả `null`: bên TS kiểm bằng `cot.hopSo !== undefined`,
/// mà `null !== undefined` là đúng ⇒ trả `null` thì bên kia tưởng có cột ở chỉ số `null`.
pub struct CotMap {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hop_so: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ho_so_so: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tieu_de: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ngay_bat_dau: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ngay_ket_thuc: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thoi_gian: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub so_to: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thbq: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ghi_chu: Option<usize>,
}

impl CotMap {
    /// Tên khoá đã nhận, theo đúng thứ tự và cách viết của TS — để đối chứng bằng mắt.
    pub fn khoa(&self) -> Vec<&'static str> {
        [
            (self.hop_so, "hopSo"),
            (self.ho_so_so, "hoSoSo"),
            (self.tieu_de, "tieuDe"),
            (self.ngay_bat_dau, "ngayBatDau"),
            (self.ngay_ket_thuc, "ngayKetThuc"),
            (self.thoi_gian, "thoiGian"),
            (self.so_to, "soTo"),
            (self.thbq, "thbq"),
            (self.ghi_chu, "ghiChu"),
        ]
        .into_iter()
        .filter_map(|(v, t)| v.map(|_| t))
        .collect()
    }
    /// Chỉ số cột đã nhận cho mục thứ `k` của `NHAN_COT` / `MO_TA_COT`.
    pub fn vi_tri(&self, k: usize) -> Option<usize> {
        match k {
            0 => self.hop_so,
            1 => self.ho_so_so,
            2 => self.tieu_de,
            3 => self.ngay_bat_dau,
            4 => self.ngay_ket_thuc,
            5 => self.thoi_gian,
            6 => self.so_to,
            7 => self.thbq,
            _ => self.ghi_chu,
        }
    }
    fn dat(&mut self, k: usize, j: usize) {
        let o = match k {
            0 => &mut self.hop_so,
            1 => &mut self.ho_so_so,
            2 => &mut self.tieu_de,
            3 => &mut self.ngay_bat_dau,
            4 => &mut self.ngay_ket_thuc,
            5 => &mut self.thoi_gian,
            6 => &mut self.so_to,
            7 => &mut self.thbq,
            _ => &mut self.ghi_chu,
        };
        *o = Some(j);
    }
}

/// Bí danh tên cột, đã nén như `chuan_ten_cot`. `bang` = phải đúng bằng (tên ngắn như "BĐ", "SỐ"
/// — so "chứa" thì "SỐ" bắt nhầm "Số hồ sơ"). **Thứ tự quan trọng**: Hộp / Hồ sơ số nhận trước để
/// "Số hộp", "Số hồ sơ" không rơi vào Số tờ. "STT" / "TT" không bao giờ gán.
const NHAN_COT: &[(&[&str], &[&str])] = &[
    (&["hopso", "sohop", "capso", "hopmoi"], &[]),
    (&["hososo", "sohoso", "hsso", "sohs", "dvbq", "sochinhthuc", "sotam", "mahoso", "hosomoi", "hsmoi"], &[]),
    (&["tieude", "tenhoso", "noidung", "trichyeu", "nhomhoso"], &[]),
    (&["ngaybatdau", "batdau", "ngaybd", "tungay", "nbd"], &["bd"]),
    (&["ngayketthuc", "ketthuc", "ngaykt", "denngay", "nkt"], &["kt"]),
    (&["thoigian", "ngaythang", "ngayhoso", "bdkt"], &["nam"]),
    (&["soto", "sotrang", "soluongto"], &["so"]),
    (&["thbq", "thoihan", "baoquan"], &[]),
    (&["ghichu"], &[]),
];

/// Bản **mặt người** của một mục trong `NHAN_COT` — dùng để dựng tệp mẫu và bảng hướng dẫn.
///
/// `NHAN_COT` là các mảnh tên đã bỏ dấu, nén khoảng trắng (`hopso`, `capso`) — in ra cho nhà
/// thầu đọc thì vô nghĩa. Bảng này cho mỗi cột một nhãn và vài cách viết **đúng như trong tệp
/// thật**, để mẫu và hướng dẫn không ai phải gõ tay lần nữa.
#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoTaCot {
    /// Trùng khoá trong `CotMap::khoa()` — `hopSo`, `hoSoSo`, …
    pub khoa: &'static str,
    /// Tên cột tệp mẫu in ra.
    pub nhan: &'static str,
    /// Các cách viết khác cũng được nhận. `nhan` không lặp lại ở đây.
    pub cung_nhan: &'static [&'static str],
    /// Thiếu cột này thì không tìm được dòng tiêu đề ⇒ cả tệp bị từ chối.
    pub bat_buoc: bool,
    pub kieu: &'static str,
    pub giai_thich: &'static str,
}

/// Mô tả 9 cột, **cùng thứ tự `NHAN_COT`**.
///
/// Test `moi_cach_viet_deu_vao_dung_cot` bắt từng chuỗi trong `nhan`/`cung_nhan` phải được
/// `tim_tieu_de_cot` xếp đúng chỉ số của nó ⇒ sửa bí danh mà quên bảng này là test đỏ ngay,
/// không phải đợi nhà thầu điền sai mẫu rồi mới biết.
pub const MO_TA_COT: &[MoTaCot] = &[
    MoTaCot {
        khoa: "hopSo",
        nhan: "Hộp số",
        cung_nhan: &["Cặp số", "Số hộp", "Hộp mới"],
        bat_buoc: false,
        kieu: "số",
        giai_thich: "Số hộp/cặp chứa hồ sơ. Để trống thì hồ sơ tính vào hộp của dòng trên.",
    },
    MoTaCot {
        khoa: "hoSoSo",
        nhan: "Hồ sơ số",
        cung_nhan: &["Số HS", "Số tạm", "Hồ sơ mới", "Mã hồ sơ", "ĐVBQ"],
        bat_buoc: true,
        kieu: "chữ hoặc số",
        giai_thich: "Số tạm của hồ sơ. Không được trùng nhau trong cùng một loại bảo quản.",
    },
    MoTaCot {
        khoa: "tieuDe",
        nhan: "Tiêu đề hồ sơ",
        cung_nhan: &["Tiêu đề", "Nội dung", "Trích yếu", "Tên hồ sơ"],
        bat_buoc: true,
        kieu: "chữ",
        giai_thich: "Tiêu đề hồ sơ. Dòng mở đầu bằng \"I.\" là phòng ban, mở đầu bằng \"1.\" là mảng công việc.",
    },
    MoTaCot {
        khoa: "ngayBatDau",
        nhan: "Ngày bắt đầu",
        cung_nhan: &["Từ ngày", "BĐ"],
        bat_buoc: false,
        kieu: "ngày dd/mm/yyyy",
        giai_thich: "Dùng cặp Ngày bắt đầu - Ngày kết thúc, HOẶC một cột Thời gian. Có cả hai cũng được.",
    },
    MoTaCot {
        khoa: "ngayKetThuc",
        nhan: "Ngày kết thúc",
        cung_nhan: &["Đến ngày", "KT"],
        bat_buoc: false,
        kieu: "ngày dd/mm/yyyy",
        giai_thich: "Xem Ngày bắt đầu.",
    },
    MoTaCot {
        khoa: "thoiGian",
        nhan: "Thời gian",
        cung_nhan: &["Ngày tháng", "Năm", "Ngày hồ sơ (BĐ-KT)"],
        bat_buoc: false,
        kieu: "chữ",
        giai_thich: "Một ô gộp, ví dụ \"05/01/2020 - 28/12/2020\" hoặc \"2020\". Trống thì máy chủ thử đọc năm trong tiêu đề.",
    },
    MoTaCot {
        khoa: "soTo",
        nhan: "Số tờ",
        cung_nhan: &["Số lượng tờ", "SỐ"],
        bat_buoc: false,
        kieu: "số",
        giai_thich: "Số tự nhiên. \"47 tờ\" cũng nhận; đơn vị khác (\"18 quyển\") thì bỏ qua và ghi lại vào Ghi chú.",
    },
    MoTaCot {
        khoa: "thbq",
        nhan: "THBQ",
        cung_nhan: &["Thời hạn bảo quản", "Thời hạn", "Bảo quản"],
        bat_buoc: false,
        kieu: "chữ",
        giai_thich: "Phải khớp một dòng trong sheet \"Danh mục THBQ\". Trống thì là lỗi, trừ khi đã chọn THBQ mặc định cho cả tệp.",
    },
    MoTaCot {
        khoa: "ghiChu",
        nhan: "Ghi chú",
        cung_nhan: &[],
        bat_buoc: false,
        kieu: "chữ",
        giai_thich: "Tuỳ ý.",
    },
];

/// Tìm dòng tiêu đề cột trong 15 dòng đầu: phải có Hồ sơ số + Tiêu đề. Thời gian không bắt buộc
/// (suy được từ tiêu đề). Trả `(số dòng 1-based, bản đồ cột)`.
pub fn tim_tieu_de_cot(rows: &[Vec<Data>]) -> Option<(usize, CotMap)> {
    for (i, dong) in rows.iter().take(15).enumerate() {
        let o: Vec<String> = dong.iter().map(|c| chuan_ten_cot(&cell_text(c, false))).collect();
        let mut cot = CotMap::default();
        let mut dung: Vec<usize> = Vec::new();
        for (k, (chua, bang)) in NHAN_COT.iter().enumerate() {
            let j = o.iter().position(|c| {
                !c.is_empty() && (chua.iter().any(|a| c.contains(a)) || bang.contains(&c.as_str()))
            });
            // `findIndex` của TS đã loại chỉ số đã dùng ngay trong vị từ — phải tìm lại cho đúng.
            let j = match j {
                Some(_) => o.iter().enumerate().position(|(idx, c)| {
                    !dung.contains(&idx)
                        && !c.is_empty()
                        && (chua.iter().any(|a| c.contains(a)) || bang.contains(&c.as_str()))
                }),
                None => None,
            };
            if let Some(j) = j {
                cot.dat(k, j);
                dung.push(j);
            }
        }
        if cot.ho_so_so.is_some() && cot.tieu_de.is_some() {
            return Some((i + 1, cot));
        }
    }
    None
}

/// Một tệp đã mở: tên các sheet, sheet được chọn, và lưới của sheet đó (đệm từ A1).
pub struct SoMucLuc {
    pub sheets: Vec<String>,
    pub sheet: String,
    pub rows: Vec<Vec<Data>>,
    pub dong_tieu_de: Option<usize>,
    pub cot: CotMap,
}

/// Mở tệp và chọn sheet như TS: sheet ĐẦU TIÊN tìm được dòng tiêu đề; không có thì sheet thứ nhất.
pub fn doc_so(duong: &Path, sheet_chi_dinh: Option<&str>) -> Result<SoMucLuc, String> {
    let wb = calamine::open_workbook_auto(duong).map_err(|e| format!("không mở được tệp: {e}"))?;
    doc_so_tu_wb(wb, sheet_chi_dinh)
}

/// Phần chung của `doc_so` và `doc_so_tu_bytes` — chọn sheet rồi dựng lưới.
fn doc_so_tu_wb<R: std::io::Read + std::io::Seek>(
    mut wb: calamine::Sheets<R>,
    sheet_chi_dinh: Option<&str>,
) -> Result<SoMucLuc, String> {
    let sheets: Vec<String> = wb.sheet_names().to_vec();
    if sheets.is_empty() {
        return Err("tệp không có sheet nào".into());
    }
    let luoi = |wb: &mut calamine::Sheets<_>, ten: &str| -> Vec<Vec<Data>> {
        match wb.worksheet_range(ten) {
            Ok(r) => day_ve_a1(&r),
            Err(_) => Vec::new(),
        }
    };

    if let Some(s) = sheet_chi_dinh.filter(|s| sheets.iter().any(|x| x == s)) {
        let rows = luoi(&mut wb, s);
        let (dong, cot) = tim_tieu_de_cot(&rows).map_or((None, CotMap::default()), |(d, c)| (Some(d), c));
        return Ok(SoMucLuc { sheets, sheet: s.to_owned(), rows, dong_tieu_de: dong, cot });
    }
    for s in sheets.clone() {
        let rows = luoi(&mut wb, &s);
        if let Some((dong, cot)) = tim_tieu_de_cot(&rows) {
            return Ok(SoMucLuc { sheets, sheet: s, rows, dong_tieu_de: Some(dong), cot });
        }
    }
    let dau = sheets[0].clone();
    let rows = luoi(&mut wb, &dau);
    Ok(SoMucLuc { sheets, sheet: dau, rows, dong_tieu_de: None, cot: CotMap::default() })
}

/// Như `doc_so` nhưng đọc từ bộ nhớ — dùng cho tệp tải lên qua HTTP, không phải ghi ra đĩa.
pub fn doc_so_tu_bytes(bytes: &[u8], sheet_chi_dinh: Option<&str>) -> Result<SoMucLuc, String> {
    let cur = std::io::Cursor::new(bytes.to_vec());
    let wb = calamine::open_workbook_auto_from_rs(cur).map_err(|e| format!("khong doc duoc tep Excel: {e}"))?;
    doc_so_tu_wb(wb, sheet_chi_dinh)
}

/// Đệm lưới về gốc A1 — xem mục 1 ở đầu tệp. Không đệm thì chỉ số cột lệch với bản TS.
fn day_ve_a1(r: &calamine::Range<Data>) -> Vec<Vec<Data>> {
    let (r0, c0) = r.start().unwrap_or((0, 0));
    let cao = r0 as usize + r.height();
    let rong = c0 as usize + r.width();
    let mut ra: Vec<Vec<Data>> = vec![vec![Data::Empty; rong]; cao];
    for (i, dong) in r.rows().enumerate() {
        for (j, o) in dong.iter().enumerate() {
            ra[r0 as usize + i][c0 as usize + j] = o.clone();
        }
    }
    ra
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bo_dau_va_chuan_ten_cot() {
        assert_eq!(bo_dau("Hồ sơ  số"), "ho so so");
        assert_eq!(bo_dau("ĐƠN VỊ"), "don vi");
        assert_eq!(chuan_ten_cot("TH\nBQ"), "thbq");
        assert_eq!(chuan_ten_cot("N.BĐ"), "nbd");
        assert_eq!(chuan_ten_cot("Số\n hộp"), "sohop");
        assert_eq!(chuan_ten_cot("  "), "");
    }

    /// Số phải in như `String(v)` của JS, kẻo "5" thành "5.0" rồi lệch cả bản đối chứng.
    #[test]
    fn so_in_nhu_js() {
        assert_eq!(so_nhu_js(5.0, false), "5");
        assert_eq!(so_nhu_js(5.5, false), "5.5");
        assert_eq!(so_nhu_js(0.0, false), "0");
        assert_eq!(so_nhu_js(-12.0, false), "-12");
        // ngoài cột ngày thì serial vẫn là số
        assert_eq!(so_nhu_js(45000.0, false), "45000");
    }

    /// Bốn giá trị dưới đây đã đối chiếu với chính biểu thức của TS
    /// (`new Date((v - 25569) * 86400000)`, chạy bằng node 10/11/2026) — đừng sửa bằng cảm giác.
    #[test]
    fn serial_ngay_dung_moc_cua_ts() {
        assert_eq!(so_nhu_js(45000.0, true), "15/03/2023");
        assert_eq!(so_nhu_js(25569.0, true), "01/01/1970");
        assert_eq!(so_nhu_js(20001.0, true), "04/10/1954");
        assert_eq!(so_nhu_js(79999.0, true), "10/01/2119");
        // Ngoài khoảng 20000..80000 thì KHÔNG coi là serial, dù ở cột ngày.
        assert_eq!(so_nhu_js(20000.0, true), "20000");
        assert_eq!(so_nhu_js(80000.0, true), "80000");
    }

    #[test]
    fn hop_so_nhan_truoc_so_to() {
        // "Số hộp" phải vào hopSo, không rơi vào soTo (bí danh "so")
        let dong = vec![
            Data::String("Số hộp".into()),
            Data::String("Hồ sơ số".into()),
            Data::String("Tiêu đề".into()),
            Data::String("Số tờ".into()),
        ];
        let (d, cot) = tim_tieu_de_cot(&[dong]).expect("phải tìm được dòng tiêu đề");
        assert_eq!(d, 1);
        assert_eq!(cot.hop_so, Some(0));
        assert_eq!(cot.ho_so_so, Some(1));
        assert_eq!(cot.tieu_de, Some(2));
        assert_eq!(cot.so_to, Some(3));
    }

    /// Thiếu Tiêu đề thì KHÔNG nhận dòng đó là tiêu đề cột.
    #[test]
    fn thieu_tieu_de_thi_khong_nhan() {
        let dong = vec![Data::String("Hồ sơ số".into()), Data::String("Số tờ".into())];
        assert!(tim_tieu_de_cot(&[dong]).is_none());
    }

    #[test]
    fn stt_khong_bao_gio_duoc_gan() {
        let dong = vec![
            Data::String("STT".into()),
            Data::String("Hồ sơ số".into()),
            Data::String("Trích yếu".into()),
        ];
        let (_, cot) = tim_tieu_de_cot(&[dong]).unwrap();
        assert_eq!(cot.ho_so_so, Some(1));
        assert_eq!(cot.tieu_de, Some(2));
        assert_eq!(cot.hop_so, None);
    }

    /// `MO_TA_COT` là thứ in ra tệp mẫu và bảng hướng dẫn cho nhà thầu. Nếu một cách viết trong
    /// đó không được `tim_tieu_de_cot` xếp đúng cột của nó thì mẫu đang dạy người ta điền sai.
    /// Mỗi vòng lấy một cách viết của từng cột, dựng một dòng tiêu đề đủ 9 cột.
    #[test]
    fn moi_cach_viet_deu_vao_dung_cot() {
        assert_eq!(MO_TA_COT.len(), NHAN_COT.len(), "hai bảng phải cùng số mục");
        let vong = 1 + MO_TA_COT.iter().map(|m| m.cung_nhan.len()).max().unwrap();
        for r in 0..vong {
            let ten: Vec<&str> = MO_TA_COT
                .iter()
                .map(|m| if r == 0 { m.nhan } else { *m.cung_nhan.get(r - 1).unwrap_or(&m.nhan) })
                .collect();
            let dong: Vec<Data> = ten.iter().map(|t| Data::String((*t).to_string())).collect();
            let (_, cot) = tim_tieu_de_cot(&[dong]).unwrap_or_else(|| panic!("vòng {r}: {ten:?} không nhận ra dòng tiêu đề"));
            for (k, m) in MO_TA_COT.iter().enumerate() {
                assert_eq!(cot.vi_tri(k), Some(k), "vòng {r}: \"{}\" ({}) rơi sai cột trong {ten:?}", ten[k], m.khoa);
            }
        }
    }

    /// Cột `bat_buoc` phải đúng là hai cột `tim_tieu_de_cot` đòi có, không nhiều không ít —
    /// hướng dẫn nói sai chỗ này thì nhà thầu bỏ cột và cả tệp bị từ chối mà không hiểu vì sao.
    #[test]
    fn cot_bat_buoc_dung_bang_dieu_kien_nhan_dong_tieu_de() {
        let bb: Vec<&str> = MO_TA_COT.iter().filter(|m| m.bat_buoc).map(|m| m.khoa).collect();
        assert_eq!(bb, vec!["hoSoSo", "tieuDe"]);
        for bo in &bb {
            let dong: Vec<Data> = MO_TA_COT
                .iter()
                .filter(|m| m.khoa != *bo)
                .map(|m| Data::String(m.nhan.to_string()))
                .collect();
            assert!(tim_tieu_de_cot(&[dong]).is_none(), "bỏ {bo} mà vẫn nhận được dòng tiêu đề");
        }
    }
}
