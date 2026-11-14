// Copyright (c) 2026 DigiText. All rights reserved.
//! Phân tích mục lục hồ sơ: lưới ô → dòng hồ sơ + nhóm phòng ban / mảng + lỗi và cảnh báo.
//!
//! Bản Rust của `phanTichMucLuc` trong `desktop/src/features/chinhly/mucLucImport.ts`, **bám
//! từng nhánh và từng câu thông báo** — bộ đối chứng 23 mục lục Bắc Giang so cả số lỗi, số
//! cảnh báo và chuỗi thông báo, nên lệch một chữ là lệch bản đối chứng.
//!
//! Các quyết định nghiệp vụ đã chốt, ghi lại để đừng ai "sửa cho hợp lý":
//! * Chỉ `I.` và `1.` là phòng ban / mảng; dòng nhóm khác (`Sổ chứng thực`) là mục nhỏ trong
//!   mảng → **bỏ qua, không báo lỗi** (5/10/2026).
//! * `1.1 …` là mục nhỏ, KHÔNG phải mảng mới (26/10/2026 — trước đây bắt thành "Mảng 1. lặp lại").
//! * Phòng ban có hồ sơ đứng ngay dưới mà chưa có dòng mảng → tự tạo mảng `Vấn đề chung` mã 00
//!   (12/10/2026); nếu trong tệp đã có dòng mảng tên "Vấn đề chung" (kể cả đứng SAU hồ sơ) thì
//!   gộp vào đó, không tạo thêm.
//! * Tệp không có dòng `I.` nào (22/23 mục lục Bắc Giang) → một phòng ban `Chung` mã 00.
//! * Số tờ không phải số / thiếu số tờ / hộp lẫn hai loại chỉ là **cảnh báo** (vẫn nhập) —
//!   mục lục cũ ghi "quyển" hàng trăm dòng, chặn thì không nhập được tệp nào (2/11/2026).
//! * Vẫn **chặn**: trùng hồ sơ số cùng loại, thiếu tiêu đề, thiếu THBQ, thời gian không suy được.

use crate::thoi_gian::{kiem_va_dien_thoi_gian, KetLuan, KetQuaThoiGian};
use crate::{bo_dau, cell_text, tim_tieu_de_cot, CotMap};
use calamine::Data;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

/// Mã mảng tự tạo. 00 để không đụng số thứ tự 01, 02… trong tệp.
pub const MANG_VAN_DE_CHUNG: &str = "Vấn đề chung";
pub const MA_MANG_VAN_DE_CHUNG: &str = "00";
pub const MA_PHONG_BAN_CHUNG: &str = "00";
pub const PHONG_BAN_CHUNG: &str = "Chung";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NhomMang {
    /// Khoá bền để dòng hồ sơ trỏ tới, không dùng chỉ số (bước gộp "Vấn đề chung" có xoá mảng).
    pub id: usize,
    pub dong: usize,
    pub stt: i64,
    pub ma: String,
    pub ten: String,
    pub so_ho_so: usize,
    pub tu_dong: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NhomPhongBan {
    pub dong: usize,
    pub stt: i64,
    pub ma: String,
    pub ten: String,
    pub mangs: Vec<NhomMang>,
    pub so_ho_so: usize,
    pub tu_dong: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Muc {
    Loi,
    CanhBao,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VanDe {
    pub dong: usize,
    pub muc: Muc,
    pub noi_dung: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DongHoSo {
    pub dong: usize,
    pub hop_so: String,
    pub ho_so_so: String,
    pub tieu_de: String,
    pub thoi_gian_goc: String,
    pub so_to: String,
    pub thbq_goc: String,
    /// Tên THBQ đúng như Danh mục; `None` = không khớp.
    pub thbq: Option<String>,
    /// `Some(true)` vĩnh viễn · `Some(false)` có thời hạn · `None` chưa biết (THBQ không khớp).
    pub vinh_vien: Option<bool>,
    pub ghi_chu: String,
    pub pb_idx: Option<usize>,
    pub mang_id: Option<usize>,
    pub tg: KetQuaThoiGian,
    /// Lỗi chặn nhập của riêng dòng này.
    pub loi: Vec<String>,
    /// Cảnh báo của dòng (vẫn nhập).
    pub canh_bao: Vec<String>,
}

/// Lỗi/cảnh báo của một dòng **thuộc ô nào** trong tệp gốc — để bên ghi tô đúng ô đó.
///
/// Trả về khoá cột theo cách gọi của [`CotMap::khoa`] (`soTo` · `thbq` · `hoSoSo` · `hopSo` ·
/// `tieuDe`). Không khớp cái nào thì rơi về `tieuDe`, vì tiêu đề luôn có và luôn nhìn thấy.
///
/// Sống ở đây chứ không ở bên ghi: nó khớp theo **câu chữ** của chính các thông báo do mô-đun
/// này sinh ra. Để bên ghi tự đoán thì mỗi lần sửa một câu thông báo là lặng lẽ tô sai ô, không
/// ai biết. Trước 14/11/2026 nó là một chuỗi regex trong `desktop/.../mucLucXuat.ts`; cổng nhà
/// cung cấp sắp cần đúng luật đó nên dọn về một chỗ.
pub fn cot_cua_loi(thong_bao: &str) -> &'static str {
    let t = thong_bao.to_lowercase();
    if t.starts_with("số tờ") || t.starts_with("thiếu số tờ") {
        "soTo"
    } else if t.contains("thbq") || t.contains("bảo quản") {
        "thbq"
    } else if t.contains("hồ sơ số") {
        "hoSoSo"
    } else if t.contains("hộp") {
        "hopSo"
    } else {
        "tieuDe"
    }
}

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhanTich {
    pub ten_tep: String,
    pub sheet: String,
    pub sheets: Vec<String>,
    /// Tệp không có dòng `I. …` → hồ sơ gom vào phòng ban `Chung` tự tạo.
    pub khong_chia_phong_ban: bool,
    pub thbq_mac_dinh: String,
    pub ten_phong_trong_tep: String,
    pub dong_tieu_de_cot: usize,
    pub cot: CotMap,
    pub phong_bans: Vec<NhomPhongBan>,
    pub rows: Vec<DongHoSo>,
    /// Lỗi + cảnh báo **cấu trúc**, không gồm ghi chú thời gian từng dòng.
    pub van_de: Vec<VanDe>,
    pub thbq_tep: String,
    pub so_hop: usize,
    pub so_loi: usize,
    pub so_canh_bao: usize,
    pub so_da_dien: usize,
}

/// Có lỗi chặn nhập hay không — khuôn `coLoiChan` bên TS.
pub fn co_loi_chan(pt: &PhanTich) -> bool {
    pt.van_de.iter().any(|v| v.muc == Muc::Loi) || pt.rows.iter().any(|r| !r.loi.is_empty())
}

pub struct TuyChon<'a> {
    pub ten_tep: &'a str,
    pub sheet: &'a str,
    pub sheets: &'a [String],
    pub thbq_danh_muc: &'a [String],
    /// Tên THBQ coi là vĩnh viễn; rỗng → chỉ `Vĩnh viễn`.
    pub thbq_vinh_vien: &'a [String],
    /// Tên THBQ dùng cho ô THBQ trống; rỗng → ô trống là lỗi dòng.
    pub thbq_mac_dinh: Option<&'a str>,
}

// ------------------------------------------------------------------ biểu thức và tiện ích

fn re(pat: &'static str, o: &'static OnceLock<regex::Regex>) -> &'static regex::Regex {
    o.get_or_init(|| regex::Regex::new(pat).unwrap())
}
macro_rules! bt {
    ($ten:ident, $pat:expr) => {
        fn $ten() -> &'static regex::Regex {
            static R: OnceLock<regex::Regex> = OnceLock::new();
            re($pat, &R)
        }
    };
}
bt!(re_phong_ban, r"^\s*([IVXLCDMivxlcdm]+)\s*[.\-)]\s*(.+)$");
bt!(re_mang, r"^\s*(\d+)\s*[.\-)]\s*(.+)$");
bt!(re_mang_con, r"^\s*\d+(?:\s*\.\s*\d+)+\s*[.\-)]?\s*\S");
bt!(re_dong_danh_so, r"^\(?\s*-?\d{1,2}\s*\)?$");
bt!(re_phong_header, r"(?i)ph[oô]ng\s*[:：]\s*(.+)");
bt!(re_hai_muoi_nam, r"(?i)(\d+)\s*năm");
bt!(re_so_to_don_vi, r"(?i)^(\d+)\s*(tờ|to)\.?$");
bt!(re_gach_nhom, r"\s+[-–]\s*|\s*[-–]\s+");

/// Gộp khoảng trắng về một dấu cách rồi trim — khuôn `.replace(/\s+/g, " ").trim()`.
fn gon_cach(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Tên phòng ban / mảng: gộp khoảng trắng; gạch ngang có khoảng trắng ở **ít nhất một bên** →
/// `" - "`. Gạch liền (`Đảng-Đoàn`, `2010-2012`) giữ nguyên để khớp mảng đã có.
pub fn chuan_ten_nhom(s: &str) -> String {
    re_gach_nhom().replace_all(&gon_cach(s), " - ").trim().to_owned()
}

/// THBQ từ 20 năm trở lên. Không có số năm (`Đánh giá lại`) → false; vĩnh viễn xét riêng.
pub fn tu_hai_muoi_nam(thbq: &str) -> bool {
    re_hai_muoi_nam()
        .captures(thbq)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<i64>().ok())
        .is_some_and(|n| n >= 20)
}

/// Tên mảng là `Vấn đề chung` (không phân biệt hoa thường, khoảng trắng, dấu chấm cuối).
pub fn la_van_de_chung(ten: &str) -> bool {
    let t = gon_cach(&ten.to_lowercase());
    let t = t.trim_end_matches(['.', ':', ';']).trim();
    t == MANG_VAN_DE_CHUNG.to_lowercase()
}

const ROMAN: &[(char, i64)] = &[('I', 1), ('V', 5), ('X', 10), ('L', 50), ('C', 100), ('D', 500), ('M', 1000)];

pub fn roman_to_int(s: &str) -> i64 {
    let u: Vec<char> = s.to_uppercase().chars().collect();
    let gia = |c: Option<&char>| c.and_then(|c| ROMAN.iter().find(|(k, _)| k == c).map(|(_, v)| *v)).unwrap_or(0);
    let mut n = 0i64;
    for i in 0..u.len() {
        let a = gia(u.get(i));
        let b = gia(u.get(i + 1));
        n += if a < b { -a } else { a };
    }
    n
}

/// Số chuẩn: `001` → `1` (số thuần); `12A` giữ nguyên.
pub fn so_chuan(s: &str) -> String {
    let t = s.trim();
    if !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()) {
        t.parse::<u128>().map(|n| n.to_string()).unwrap_or_else(|_| t.to_owned())
    } else {
        t.to_owned()
    }
}

/// Khớp tên THBQ trong tệp với Danh mục: bỏ dấu cách thừa, không phân biệt hoa thường;
/// `20` → `20 năm`; `vv` / `vv.` → `Vĩnh viễn`.
pub fn khop_thbq(ten: &str, danh_muc: &[String]) -> Option<String> {
    let t = ten.trim().trim_end_matches(['.', ':', ';']).trim();
    if t.is_empty() {
        return None;
    }
    let k = bo_dau(t);
    // `CHUAN_THBQ` bên TS có cả khoá "vĩnh viễn", nhưng khoá được tra bằng `boDau(t)` nên chỉ
    // "vv" và "vinh vien" chạm tới được. Giữ nguyên hành vi đó.
    let chuan = match k.as_str() {
        "vv" | "vinh vien" => "Vĩnh viễn",
        _ => "",
    };
    let la_so = !k.is_empty() && k.bytes().all(|b| b.is_ascii_digit());
    let bo_khong_dau: String = {
        let mut c = k.chars().peekable();
        let mut so_khong = 0usize;
        while let Some('0') = c.peek() {
            c.next();
            so_khong += 1;
        }
        // `/^0+(\d)/ → "$1"`: chỉ bỏ số 0 đầu khi ngay sau còn một chữ số.
        if so_khong > 0 && c.peek().is_some_and(|x| x.is_ascii_digit()) {
            c.collect()
        } else {
            k.clone()
        }
    };
    let ung: Vec<String> = [t.to_owned(), chuan.to_owned(), if la_so { format!("{k} năm") } else { String::new() }, bo_khong_dau]
        .into_iter()
        .filter(|x| !x.is_empty())
        .collect();
    for u in &ung {
        let bu = bo_dau(u);
        if let Some(hit) = danh_muc.iter().find(|d| bo_dau(d) == bu) {
            return Some(hit.clone());
        }
    }
    None
}

pub fn ten_loai(vv: Option<bool>) -> &'static str {
    match vv {
        None => "chưa rõ loại",
        Some(true) => "vĩnh viễn",
        Some(false) => "có thời hạn",
    }
}

// ------------------------------------------------------------------ phân tích

pub fn phan_tich_muc_luc(rows: &[Vec<Data>], o: &TuyChon<'_>) -> PhanTich {
    let thbq_mac_dinh = o.thbq_mac_dinh.and_then(|t| khop_thbq(t, o.thbq_danh_muc));
    let mut out = PhanTich {
        ten_tep: o.ten_tep.to_owned(),
        sheet: o.sheet.to_owned(),
        sheets: if o.sheets.is_empty() { vec![o.sheet.to_owned()] } else { o.sheets.to_vec() },
        thbq_mac_dinh: thbq_mac_dinh.clone().unwrap_or_default(),
        ..Default::default()
    };

    let Some((dong_td, cot)) = tim_tieu_de_cot(rows) else {
        out.van_de.push(VanDe {
            dong: 0,
            muc: Muc::Loi,
            noi_dung: "Không tìm thấy dòng tiêu đề cột (cần có cột Hồ sơ số và Tiêu đề hồ sơ trong 15 dòng đầu của sheet — thử chọn sheet khác)".into(),
        });
        out.so_loi = 1;
        return out;
    };
    out.dong_tieu_de_cot = dong_td;
    out.cot = cot.clone();

    // Tên phông ghi ở phần đầu sheet ("Phông: …") — quét các dòng TRƯỚC dòng tiêu đề cột.
    for dong in rows.iter().take(dong_td.saturating_sub(1)) {
        for c in dong {
            if let Some(m) = re_phong_header().captures(&gon_cach(&cell_text(c, false))) {
                out.ten_phong_trong_tep = m.get(1).unwrap().as_str().trim().to_owned();
            }
        }
    }

    let lay = |r: &[Data], j: Option<usize>, ngay: bool| -> String {
        match j {
            None => String::new(),
            Some(j) => r.get(j).map(|c| cell_text(c, ngay)).unwrap_or_default(),
        }
    };

    let mut pb_idx: Option<usize> = None;
    let mut mang_id: Option<usize> = None;
    let mut hop_hien_tai = String::new();
    let mut mang_id_ke = 1usize;
    // `${loại}|${hồ sơ số thường}` → dòng
    let mut da_co_ho_so_so: BTreeMap<String, usize> = BTreeMap::new();
    let mut dem_thbq: BTreeMap<String, usize> = BTreeMap::new();
    let ten_vv: BTreeSet<String> = if o.thbq_vinh_vien.is_empty() {
        ["vĩnh viễn".to_owned()].into_iter().collect()
    } else {
        o.thbq_vinh_vien.iter().map(|t| t.to_lowercase()).collect()
    };

    // Tệp không có dòng phòng ban "I. …" nào → một phòng ban "Chung" (00) cho cả tệp.
    let co_dong_phong_ban = rows.iter().skip(dong_td).any(|r| {
        so_chuan(&lay(r, cot.ho_so_so, false)).is_empty()
            && re_phong_ban().is_match(&gon_cach(&lay(r, cot.tieu_de, false)))
    });
    if !co_dong_phong_ban {
        out.khong_chia_phong_ban = true;
        out.phong_bans.push(NhomPhongBan {
            dong: dong_td,
            stt: 0,
            ma: MA_PHONG_BAN_CHUNG.into(),
            ten: PHONG_BAN_CHUNG.into(),
            mangs: Vec::new(),
            so_ho_so: 0,
            tu_dong: true,
        });
        pb_idx = Some(0);
    }

    for (i, r) in rows.iter().enumerate().skip(dong_td) {
        let dong = i + 1;
        let non_empty: Vec<String> = r.iter().map(|c| cell_text(c, false)).filter(|s| !s.is_empty()).collect();
        if non_empty.is_empty() {
            continue;
        }
        // Dòng đánh số cột "(1) (2) (3)…" ngay dưới tiêu đề.
        if i < dong_td + 2 && non_empty.iter().all(|c| re_dong_danh_so().is_match(c)) {
            continue;
        }
        let ho_so_so = so_chuan(&lay(r, cot.ho_so_so, false));
        let tieu_de = gon_cach(&lay(r, cot.tieu_de, false));
        // Hộp số chỉ ghi ở dòng đầu hộp → mang xuống tới khi gặp số hộp mới.
        let hop_o = lay(r, cot.hop_so, false).trim().to_owned();
        if !hop_o.is_empty() {
            hop_hien_tai = hop_o;
        }

        if ho_so_so.is_empty() {
            if tieu_de.is_empty() {
                continue; // chỉ có hộp số / ghi chú lẻ
            }
            if let Some(m) = re_phong_ban().captures(&tieu_de) {
                let stt = roman_to_int(m.get(1).unwrap().as_str());
                let ma = format!("{stt:02}");
                if let Some(cu) = out.phong_bans.iter().find(|p| p.ma == ma) {
                    let d = cu.dong;
                    out.van_de.push(VanDe {
                        dong,
                        muc: Muc::Loi,
                        noi_dung: format!("Phòng ban {}. lặp lại (đã có ở dòng {})", m.get(1).unwrap().as_str(), d),
                    });
                }
                out.phong_bans.push(NhomPhongBan {
                    dong,
                    stt,
                    ma,
                    ten: chuan_ten_nhom(m.get(2).unwrap().as_str()),
                    mangs: Vec::new(),
                    so_ho_so: 0,
                    tu_dong: false,
                });
                pb_idx = Some(out.phong_bans.len() - 1);
                mang_id = None;
                continue;
            }
            // "1.1 …" là mục nhỏ trong mảng hiện tại → bỏ qua dòng.
            if re_mang_con().is_match(&tieu_de) {
                continue;
            }
            if let Some(m) = re_mang().captures(&tieu_de) {
                let Some(pi) = pb_idx else {
                    out.van_de.push(VanDe {
                        dong,
                        muc: Muc::Loi,
                        noi_dung: format!("Mảng \"{tieu_de}\" đứng trước dòng phòng ban (I. …) — thêm dòng phòng ban phía trên"),
                    });
                    continue;
                };
                let stt: i64 = m.get(1).unwrap().as_str().parse().unwrap_or(0);
                let ma = format!("{stt:02}");
                if out.phong_bans[pi].mangs.iter().any(|x| x.ma == ma) {
                    let ten_pb = out.phong_bans[pi].ten.clone();
                    out.van_de.push(VanDe {
                        dong,
                        muc: Muc::Loi,
                        noi_dung: format!("Mảng {}. lặp lại trong phòng ban {}", m.get(1).unwrap().as_str(), ten_pb),
                    });
                }
                let id = mang_id_ke;
                mang_id_ke += 1;
                out.phong_bans[pi].mangs.push(NhomMang {
                    id,
                    dong,
                    stt,
                    ma,
                    ten: chuan_ten_nhom(m.get(2).unwrap().as_str()),
                    so_ho_so: 0,
                    tu_dong: false,
                });
                mang_id = Some(id);
                continue;
            }
            // Dòng nhóm không có I./1. = mục nhỏ trong mảng → bỏ qua, không báo lỗi.
            continue;
        }

        let mut loi: Vec<String> = Vec::new();
        if tieu_de.is_empty() {
            loi.push("Thiếu tiêu đề hồ sơ".into());
        }
        match pb_idx {
            None => loi.push("Chưa có dòng phòng ban (I. …) phía trên".into()),
            Some(pi) => {
                if mang_id.is_none() {
                    // Hồ sơ ngay dưới phòng ban → mảng "Vấn đề chung" tự tạo, một mảng / phòng ban.
                    mang_id = match out.phong_bans[pi].mangs.iter().find(|x| x.tu_dong) {
                        Some(m) => Some(m.id),
                        None => {
                            let id = mang_id_ke;
                            mang_id_ke += 1;
                            out.phong_bans[pi].mangs.push(NhomMang {
                                id,
                                dong,
                                stt: 0,
                                ma: MA_MANG_VAN_DE_CHUNG.into(),
                                ten: MANG_VAN_DE_CHUNG.into(),
                                so_ho_so: 0,
                                tu_dong: true,
                            });
                            Some(id)
                        }
                    };
                }
            }
        }

        let mut canh_bao: Vec<String> = Vec::new();
        let thbq_goc = lay(r, cot.thbq, false);
        let mut thbq = khop_thbq(&thbq_goc, o.thbq_danh_muc);
        if thbq_goc.is_empty() {
            match &thbq_mac_dinh {
                Some(md) => {
                    thbq = Some(md.clone());
                    canh_bao.push(format!("THBQ trống — dùng mặc định \"{md}\""));
                }
                None => loi.push("Thiếu thời hạn bảo quản (chọn THBQ mặc định cho tệp hoặc điền vào ô)".into()),
            }
        } else if thbq.is_none() {
            loi.push(format!("Thời hạn bảo quản \"{thbq_goc}\" không có trong Danh mục"));
        }
        if let Some(t) = &thbq {
            *dem_thbq.entry(t.clone()).or_insert(0) += 1;
        }
        let vinh_vien = thbq.as_ref().map(|t| ten_vv.contains(&t.to_lowercase()));

        // Hồ sơ số (= số tạm) duy nhất trong từng loại: hai mục lục đều đánh từ 1.
        let khoa = format!("{}|{}", loai_khoa(vinh_vien), ho_so_so.to_lowercase());
        match da_co_ho_so_so.get(&khoa) {
            Some(truoc) => loi.push(format!("Hồ sơ số {ho_so_so} trùng với dòng {truoc} (cùng loại {})", ten_loai(vinh_vien))),
            None => {
                da_co_ho_so_so.insert(khoa, dong);
            }
        }
        if cot.hop_so.is_some() && hop_hien_tai.is_empty() {
            canh_bao.push("Chưa có Hộp số — nhập không xếp hộp (xếp sau ở Hệ thống hoá)".into());
        }

        let thoi_gian_goc = if cot.thoi_gian.is_some() {
            lay(r, cot.thoi_gian, true)
        } else {
            [lay(r, cot.ngay_bat_dau, true), lay(r, cot.ngay_ket_thuc, true)]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" - ")
        };
        let tg = kiem_va_dien_thoi_gian(&thoi_gian_goc, &tieu_de);
        if tg.bat_dau.is_empty() || tg.ket_thuc.is_empty() {
            let g = thoi_gian_goc.trim();
            loi.push(if g.is_empty() {
                "Thiếu thời gian bắt đầu - kết thúc (ô trống, tiêu đề không ghi năm)".into()
            } else {
                format!("Thời gian \"{g}\" không đọc được — ghi dd/mm/yyyy - dd/mm/yyyy")
            });
        }
        if let Some(pi) = pb_idx {
            out.phong_bans[pi].so_ho_so += 1;
            if let Some(id) = mang_id {
                if let Some(m) = out.phong_bans[pi].mangs.iter_mut().find(|m| m.id == id) {
                    m.so_ho_so += 1;
                }
            }
        }

        // Số tờ chỉ nhận số tự nhiên; "47 tờ" / "47tờ" / "47 TỜ" / "47 to" nhận là 47.
        let so_to_goc = lay(r, cot.so_to, false).trim().to_owned();
        let mut so_to = re_so_to_don_vi().replace(&so_to_goc, "$1").into_owned();
        let mut ghi_chu = lay(r, cot.ghi_chu, false);
        let phai_danh_so_to = vinh_vien == Some(true) || thbq.as_deref().is_some_and(tu_hai_muoi_nam);
        let la_so = !so_to.is_empty() && so_to.bytes().all(|b| b.is_ascii_digit());
        if !so_to.is_empty() && !la_so {
            canh_bao.push(format!(
                "Số tờ \"{so_to_goc}\" không phải số — để trống, ghi vào Ghi chú{}",
                if phai_danh_so_to { " (tài liệu bảo quản từ 20 năm phải đánh số tờ)" } else { "" }
            ));
            ghi_chu = [ghi_chu.clone(), format!("Số tờ gốc: {so_to_goc}")]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("; ");
            so_to = String::new();
        } else if la_so && so_to.parse::<u128>().unwrap_or(1) == 0 {
            canh_bao.push("Số tờ 0 — để trống".into());
            so_to = String::new();
        } else if so_to.is_empty() && phai_danh_so_to {
            canh_bao.push(format!("Thiếu số tờ — tài liệu bảo quản {} phải đánh số tờ", thbq.as_deref().unwrap_or("")));
        }

        out.rows.push(DongHoSo {
            dong,
            hop_so: hop_hien_tai.clone(),
            ho_so_so,
            tieu_de,
            thoi_gian_goc,
            so_to,
            thbq_goc,
            thbq,
            vinh_vien,
            ghi_chu,
            pb_idx,
            mang_id,
            tg,
            loi,
            canh_bao,
        });
    }

    // Hộp thuộc đúng một loại: hộp lẫn hai loại → cảnh báo (máy chủ tách thành hai hộp cùng số).
    let mut loai_hop: BTreeMap<String, BTreeSet<bool>> = BTreeMap::new();
    for r in &out.rows {
        if !r.hop_so.is_empty() {
            if let Some(vv) = r.vinh_vien {
                loai_hop.entry(r.hop_so.clone()).or_default().insert(vv);
            }
        }
    }
    for r in out.rows.iter_mut() {
        if !r.hop_so.is_empty() && loai_hop.get(&r.hop_so).map_or(0, BTreeSet::len) > 1 {
            r.canh_bao.push(format!(
                "Hộp {} có cả hồ sơ vĩnh viễn lẫn có thời hạn — máy chủ sẽ tách thành hai hộp cùng số",
                r.hop_so
            ));
        }
    }
    out.so_hop = out
        .rows
        .iter()
        .filter(|r| !r.hop_so.is_empty())
        .map(|r| format!("{}|{}", loai_khoa(r.vinh_vien), r.hop_so))
        .collect::<BTreeSet<_>>()
        .len();

    // Phòng ban đã có dòng mảng "Vấn đề chung" trong tệp (kể cả đứng SAU hồ sơ) → hồ sơ đứng ngay
    // dưới phòng ban vào luôn mảng đó, không giữ thêm mảng 00 tự tạo.
    for pi in 0..out.phong_bans.len() {
        let tu = out.phong_bans[pi].mangs.iter().find(|m| m.tu_dong).map(|m| (m.id, m.so_ho_so));
        let co = out.phong_bans[pi].mangs.iter().find(|m| !m.tu_dong && la_van_de_chung(&m.ten)).map(|m| m.id);
        let (Some((tu_id, tu_so)), Some(co_id)) = (tu, co) else { continue };
        for r in out.rows.iter_mut() {
            if r.mang_id == Some(tu_id) {
                r.mang_id = Some(co_id);
            }
        }
        if let Some(m) = out.phong_bans[pi].mangs.iter_mut().find(|m| m.id == co_id) {
            m.so_ho_so += tu_so;
        }
        out.phong_bans[pi].mangs.retain(|m| m.id != tu_id);
    }

    if out.rows.is_empty() && !out.van_de.iter().any(|v| v.muc == Muc::Loi) {
        out.van_de.push(VanDe { dong: 0, muc: Muc::Loi, noi_dung: "Tệp không có dòng hồ sơ nào (cột Hồ sơ số trống cả)".into() });
    }
    // THBQ phổ biến nhất. `>` chứ không `>=` để giữ đúng thứ tự "gặp trước thắng" như Map của JS.
    let mut max = 0usize;
    for (k, v) in &dem_thbq {
        if *v > max {
            max = *v;
            out.thbq_tep = k.clone();
        }
    }
    out.so_loi = out.van_de.iter().filter(|v| v.muc == Muc::Loi).count() + out.rows.iter().filter(|r| !r.loi.is_empty()).count();
    out.so_canh_bao = out
        .rows
        .iter()
        .filter(|r| matches!(r.tg.ket_luan, KetLuan::CanhBao | KetLuan::Loi) || !r.canh_bao.is_empty())
        .count();
    out.so_da_dien = out.rows.iter().filter(|r| r.tg.ket_luan == KetLuan::DaDien).count();
    out
}

/// Khoá loại trong chuỗi, giống `${vinhVien}` của JS: `true` / `false` / `null`.
fn loai_khoa(vv: Option<bool>) -> &'static str {
    match vv {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn o(s: &str) -> Data {
        Data::String(s.into())
    }
    fn thbq() -> Vec<String> {
        ["Vĩnh viễn", "5 năm", "10 năm", "20 năm", "50 năm", "70 năm"].iter().map(|s| s.to_string()).collect()
    }
    fn chay(rows: Vec<Vec<Data>>) -> PhanTich {
        let dm = thbq();
        let vv = vec!["Vĩnh viễn".to_owned()];
        phan_tich_muc_luc(
            &rows,
            &TuyChon { ten_tep: "t.xlsx", sheet: "S", sheets: &[], thbq_danh_muc: &dm, thbq_vinh_vien: &vv, thbq_mac_dinh: None },
        )
    }
    fn tieu_de_cot() -> Vec<Data> {
        vec![o("Hộp số"), o("Hồ sơ số"), o("Tiêu đề hồ sơ"), o("Thời gian"), o("Số tờ"), o("THBQ"), o("Ghi chú")]
    }
    fn hs(hop: &str, so: &str, td: &str, tg: &str, to: &str, bq: &str) -> Vec<Data> {
        vec![o(hop), o(so), o(td), o(tg), o(to), o(bq), o("")]
    }

    #[test]
    fn khong_co_dong_phong_ban_thi_tu_tao_chung() {
        let pt = chay(vec![tieu_de_cot(), hs("1", "1", "Hồ sơ năm 2020", "2020", "10", "Vĩnh viễn")]);
        assert!(pt.khong_chia_phong_ban);
        assert_eq!(pt.phong_bans.len(), 1);
        assert_eq!((pt.phong_bans[0].ma.as_str(), pt.phong_bans[0].ten.as_str()), ("00", "Chung"));
        assert_eq!(pt.rows.len(), 1);
        assert_eq!(pt.so_loi, 0);
        // không có dòng mảng → tự tạo "Vấn đề chung"
        assert_eq!(pt.phong_bans[0].mangs.len(), 1);
        assert_eq!(pt.phong_bans[0].mangs[0].ten, MANG_VAN_DE_CHUNG);
        assert!(pt.phong_bans[0].mangs[0].tu_dong);
    }

    #[test]
    fn phong_ban_va_mang_duoc_nhan() {
        let pt = chay(vec![
            tieu_de_cot(),
            vec![o(""), o(""), o("I. Phòng Tổ chức"), o(""), o(""), o(""), o("")],
            vec![o(""), o(""), o("1. Văn bản đi"), o(""), o(""), o(""), o("")],
            hs("1", "1", "Hồ sơ năm 2020", "2020", "10", "Vĩnh viễn"),
        ]);
        assert!(!pt.khong_chia_phong_ban);
        assert_eq!(pt.phong_bans.len(), 1);
        assert_eq!(pt.phong_bans[0].ten, "Phòng Tổ chức");
        assert_eq!(pt.phong_bans[0].mangs.len(), 1);
        assert_eq!(pt.phong_bans[0].mangs[0].ten, "Văn bản đi");
        assert!(!pt.phong_bans[0].mangs[0].tu_dong);
        assert_eq!(pt.phong_bans[0].so_ho_so, 1);
        assert_eq!(pt.so_loi, 0);
    }

    /// "1.1 …" là mục nhỏ, KHÔNG phải mảng mới — hồi quy của 26/10/2026.
    #[test]
    fn muc_nho_khong_thanh_mang_moi() {
        let pt = chay(vec![
            tieu_de_cot(),
            vec![o(""), o(""), o("I. Phòng A"), o(""), o(""), o(""), o("")],
            vec![o(""), o(""), o("1. Văn bản đi"), o(""), o(""), o(""), o("")],
            vec![o(""), o(""), o("1.1 Công văn"), o(""), o(""), o(""), o("")],
            hs("1", "1", "Hồ sơ năm 2020", "2020", "10", "Vĩnh viễn"),
        ]);
        assert_eq!(pt.phong_bans[0].mangs.len(), 1, "không được sinh mảng thứ hai");
        assert_eq!(pt.so_loi, 0, "không được báo 'Mảng 1. lặp lại'");
    }

    /// Dòng nhóm không có I./1. thì bỏ qua, hồ sơ phía dưới vẫn thuộc mảng hiện tại.
    #[test]
    fn dong_nhom_la_bi_bo_qua_khong_bao_loi() {
        let pt = chay(vec![
            tieu_de_cot(),
            vec![o(""), o(""), o("I. Phòng A"), o(""), o(""), o(""), o("")],
            vec![o(""), o(""), o("1. Văn bản đi"), o(""), o(""), o(""), o("")],
            vec![o(""), o(""), o("Sổ chứng thực"), o(""), o(""), o(""), o("")],
            hs("1", "1", "Hồ sơ năm 2020", "2020", "10", "Vĩnh viễn"),
        ]);
        assert_eq!(pt.so_loi, 0);
        assert_eq!(pt.rows.len(), 1);
        assert_eq!(pt.rows[0].mang_id, Some(pt.phong_bans[0].mangs[0].id));
    }

    #[test]
    fn hop_so_mang_xuong_dong_sau() {
        let pt = chay(vec![
            tieu_de_cot(),
            hs("5", "1", "Hồ sơ năm 2020", "2020", "10", "Vĩnh viễn"),
            hs("", "2", "Hồ sơ năm 2021", "2021", "10", "Vĩnh viễn"),
        ]);
        assert_eq!(pt.rows[0].hop_so, "5");
        assert_eq!(pt.rows[1].hop_so, "5", "hộp số phải mang xuống");
    }

    /// Hồ sơ số trùng CÙNG LOẠI mới là lỗi; khác loại thì không.
    #[test]
    fn trung_ho_so_so_chi_trong_cung_loai() {
        let pt = chay(vec![
            tieu_de_cot(),
            hs("1", "1", "A năm 2020", "2020", "10", "Vĩnh viễn"),
            hs("1", "1", "B năm 2020", "2020", "10", "5 năm"),
        ]);
        assert_eq!(pt.so_loi, 0, "khác loại thì không trùng");

        let pt2 = chay(vec![
            tieu_de_cot(),
            hs("1", "1", "A năm 2020", "2020", "10", "Vĩnh viễn"),
            hs("1", "1", "B năm 2020", "2020", "10", "Vĩnh viễn"),
        ]);
        assert_eq!(pt2.so_loi, 1);
        assert!(pt2.rows[1].loi[0].contains("trùng với dòng 2"));
    }

    #[test]
    fn so_to_co_don_vi_va_khong_phai_so() {
        let pt = chay(vec![
            tieu_de_cot(),
            hs("1", "1", "A năm 2020", "2020", "47 tờ", "Vĩnh viễn"),
            hs("1", "2", "B năm 2020", "2020", "18 quyển", "Vĩnh viễn"),
            hs("1", "3", "C năm 2020", "2020", "0", "Vĩnh viễn"),
        ]);
        assert_eq!(pt.rows[0].so_to, "47", "\"47 tờ\" phải nhận là 47");
        assert_eq!(pt.rows[1].so_to, "", "\"18 quyển\" để trống");
        assert!(pt.rows[1].ghi_chu.contains("Số tờ gốc: 18 quyển"));
        assert!(pt.rows[1].canh_bao.iter().any(|c| c.contains("không phải số")));
        assert_eq!(pt.rows[2].so_to, "");
        assert!(pt.rows[2].canh_bao.iter().any(|c| c == "Số tờ 0 — để trống"));
        assert_eq!(pt.so_loi, 0, "số tờ sai chỉ là cảnh báo, không chặn");
    }

    #[test]
    fn hop_lan_hai_loai_thi_canh_bao() {
        let pt = chay(vec![
            tieu_de_cot(),
            hs("7", "1", "A năm 2020", "2020", "10", "Vĩnh viễn"),
            hs("7", "1", "B năm 2020", "2020", "10", "5 năm"),
        ]);
        assert!(pt.rows.iter().all(|r| r.canh_bao.iter().any(|c| c.contains("tách thành hai hộp cùng số"))));
        assert_eq!(pt.so_hop, 2, "hộp 7 đếm thành hai (mỗi loại một)");
    }

    /// Mảng "Vấn đề chung" có thật trong tệp (đứng SAU hồ sơ) thì gộp, không giữ mảng 00 tự tạo.
    #[test]
    fn gop_van_de_chung_co_that() {
        let pt = chay(vec![
            tieu_de_cot(),
            vec![o(""), o(""), o("I. Phòng A"), o(""), o(""), o(""), o("")],
            hs("1", "1", "A năm 2020", "2020", "10", "Vĩnh viễn"),
            vec![o(""), o(""), o("1. Vấn đề chung"), o(""), o(""), o(""), o("")],
            hs("1", "2", "B năm 2020", "2020", "10", "Vĩnh viễn"),
        ]);
        assert_eq!(pt.phong_bans[0].mangs.len(), 1, "chỉ còn mảng thật");
        let m = &pt.phong_bans[0].mangs[0];
        assert!(!m.tu_dong);
        assert_eq!(m.so_ho_so, 2, "gộp cả hồ sơ của mảng tự tạo");
        assert!(pt.rows.iter().all(|r| r.mang_id == Some(m.id)));
    }

    #[test]
    fn thieu_thbq_la_loi_chan() {
        let pt = chay(vec![tieu_de_cot(), hs("1", "1", "A năm 2020", "2020", "10", "")]);
        assert!(co_loi_chan(&pt));
        assert!(pt.rows[0].loi.iter().any(|l| l.contains("Thiếu thời hạn bảo quản")));
    }

    #[test]
    fn khong_tim_thay_dong_tieu_de() {
        let pt = chay(vec![vec![o("Bìa"), o("")], vec![o("Không có gì"), o("")]]);
        assert_eq!(pt.so_loi, 1);
        assert!(pt.van_de[0].noi_dung.contains("Không tìm thấy dòng tiêu đề cột"));
        assert!(co_loi_chan(&pt));
    }

    #[test]
    fn khop_thbq_cac_cach_viet() {
        let dm = thbq();
        assert_eq!(khop_thbq("vv", &dm).as_deref(), Some("Vĩnh viễn"));
        assert_eq!(khop_thbq("VV.", &dm).as_deref(), Some("Vĩnh viễn"));
        assert_eq!(khop_thbq("vĩnh viễn", &dm).as_deref(), Some("Vĩnh viễn"));
        assert_eq!(khop_thbq("20", &dm).as_deref(), Some("20 năm"));
        assert_eq!(khop_thbq("05 năm", &dm).as_deref(), Some("5 năm"));
        assert_eq!(khop_thbq("", &dm), None);
        assert_eq!(khop_thbq("không có", &dm), None);
    }

    #[test]
    fn so_chuan_va_chuan_ten_nhom() {
        assert_eq!(so_chuan("001"), "1");
        assert_eq!(so_chuan("12A"), "12A");
        assert_eq!(so_chuan("  "), "");
        assert_eq!(chuan_ten_nhom("Văn phòng- Thống kê"), "Văn phòng - Thống kê");
        assert_eq!(chuan_ten_nhom("Văn  phòng -Thống kê"), "Văn phòng - Thống kê");
        assert_eq!(chuan_ten_nhom("Đảng-Đoàn"), "Đảng-Đoàn", "gạch liền giữ nguyên");
    }

    #[test]
    fn roman_va_hai_muoi_nam() {
        assert_eq!(roman_to_int("IV"), 4);
        assert_eq!(roman_to_int("xiii"), 13);
        assert!(tu_hai_muoi_nam("20 năm"));
        assert!(tu_hai_muoi_nam("70 Năm"));
        assert!(!tu_hai_muoi_nam("10 năm"));
        assert!(!tu_hai_muoi_nam("Theo tuổi thọ công trình"));
    }

    #[test]
    fn la_van_de_chung_bo_dau_cau() {
        assert!(la_van_de_chung("Vấn đề chung"));
        assert!(la_van_de_chung("  vấn  đề   CHUNG. "));
        assert!(!la_van_de_chung("Vấn đề chung khác"));
    }
}

// ------------------------------------------------------------------ dòng gửi lệnh nhập

/// Một dòng hồ sơ → bản ghi gửi `POST /cl/ho-so/import`.
///
/// Bản Rust của `dongMau` bên TS, **giữ nguyên quy ước bỏ khoá rỗng**: khoá có giá trị `""` bị
/// xoá hẳn khỏi bản ghi, vì `validate::normalize` phía máy chủ coi khoá-có-mặt-nhưng-rỗng khác
/// với khoá-không-có (ghi đè bằng rỗng vs. giữ nguyên).
///
/// Ba giá trị `che_do_su_dung` / `ngon_ngu` / `tinh_trang_vat_ly` là mặc định TT 05 của lệnh
/// nhập mục lục, không đọc từ tệp.
pub fn dong_nhap(pt: &PhanTich, r: &DongHoSo) -> serde_json::Map<String, serde_json::Value> {
    use serde_json::Value;
    let pb = r.pb_idx.and_then(|i| pt.phong_bans.get(i));
    let mang = r
        .mang_id
        .and_then(|id| pb.and_then(|p| p.mangs.iter().find(|m| m.id == id)));
    let mut o = serde_json::Map::new();
    let mut dat = |k: &str, v: String| {
        // Khoá rỗng bị bỏ — xem phần chú thích ở trên.
        if !v.is_empty() {
            o.insert(k.to_owned(), Value::String(v));
        }
    };
    dat("ma_don_vi", pb.map(|p| p.ma.clone()).unwrap_or_default());
    dat("ma_mang", mang.map(|m| m.ma.clone()).unwrap_or_default());
    dat("ten_phong_ban", pb.map(|p| p.ten.clone()).unwrap_or_default());
    dat("ten_mang", mang.map(|m| m.ten.clone()).unwrap_or_default());
    // Số tạm = Hồ sơ số trong tệp (khuôn `soTamCua`): mục lục đã có số thì dùng luôn.
    dat("so_tam", r.ho_so_so.clone());
    dat("ho_so_so", r.ho_so_so.clone());
    dat("hop_so", r.hop_so.clone());
    dat("tieu_de", r.tieu_de.clone());
    dat("ngay_bat_dau", r.tg.bat_dau.clone());
    dat("ngay_ket_thuc", r.tg.ket_thuc.clone());
    dat("thbq", r.thbq.clone().unwrap_or_else(|| r.thbq_goc.clone()));
    dat("so_to", r.so_to.clone());
    dat("ghi_chu", r.ghi_chu.clone());
    dat("che_do_su_dung", "01".into());
    dat("ngon_ngu", "01".into());
    dat("tinh_trang_vat_ly", "02".into());
    o
}

#[cfg(test)]
mod tests_dong_nhap {
    use super::*;

    #[test]
    fn bo_khoa_rong_va_mang_ten_nhom() {
        let o = |s: &str| Data::String(s.into());
        let dm: Vec<String> = ["Vĩnh viễn"].iter().map(|s| s.to_string()).collect();
        let vv = vec!["Vĩnh viễn".to_owned()];
        let rows = vec![
            vec![o("Hộp số"), o("Hồ sơ số"), o("Tiêu đề hồ sơ"), o("Thời gian"), o("Số tờ"), o("THBQ"), o("Ghi chú")],
            vec![o(""), o(""), o("I. Phòng Tổ chức"), o(""), o(""), o(""), o("")],
            vec![o(""), o(""), o("1. Văn bản đi"), o(""), o(""), o(""), o("")],
            // số tờ để TRỐNG để chắc khoá rỗng bị bỏ
            vec![o("3"), o("7"), o("Hồ sơ năm 2020"), o("2020"), o(""), o("Vĩnh viễn"), o("")],
        ];
        let pt = phan_tich_muc_luc(
            &rows,
            &TuyChon { ten_tep: "t.xlsx", sheet: "S", sheets: &[], thbq_danh_muc: &dm, thbq_vinh_vien: &vv, thbq_mac_dinh: None },
        );
        assert_eq!(pt.rows.len(), 1);
        let d = dong_nhap(&pt, &pt.rows[0]);
        assert_eq!(d.get("ma_don_vi").unwrap(), "01");
        assert_eq!(d.get("ten_phong_ban").unwrap(), "Phòng Tổ chức");
        assert_eq!(d.get("ma_mang").unwrap(), "01");
        assert_eq!(d.get("ten_mang").unwrap(), "Văn bản đi");
        assert_eq!(d.get("so_tam").unwrap(), "7");
        assert_eq!(d.get("hop_so").unwrap(), "3");
        assert_eq!(d.get("ngay_bat_dau").unwrap(), "01/01/2020");
        assert_eq!(d.get("ngay_ket_thuc").unwrap(), "31/12/2020");
        assert_eq!(d.get("thbq").unwrap(), "Vĩnh viễn");
        assert_eq!(d.get("che_do_su_dung").unwrap(), "01");
        // khoá rỗng phải BỊ BỎ, không phải trả ""
        assert!(!d.contains_key("so_to"), "số tờ rỗng phải bỏ khoá, không gửi \"\"");
        assert!(!d.contains_key("ghi_chu"));
    }

    /// THBQ không khớp Danh mục thì gửi nguyên văn trong tệp để máy chủ báo lỗi đúng chỗ.
    #[test]
    fn thbq_khong_khop_thi_gui_nguyen_van() {
        let o = |s: &str| Data::String(s.into());
        let dm: Vec<String> = ["Vĩnh viễn"].iter().map(|s| s.to_string()).collect();
        let vv = vec!["Vĩnh viễn".to_owned()];
        let rows = vec![
            vec![o("Hồ sơ số"), o("Tiêu đề hồ sơ"), o("Thời gian"), o("THBQ")],
            vec![o("1"), o("Hồ sơ năm 2020"), o("2020"), o("Ba chục năm")],
        ];
        let pt = phan_tich_muc_luc(
            &rows,
            &TuyChon { ten_tep: "t.xlsx", sheet: "S", sheets: &[], thbq_danh_muc: &dm, thbq_vinh_vien: &vv, thbq_mac_dinh: None },
        );
        let d = dong_nhap(&pt, &pt.rows[0]);
        assert_eq!(d.get("thbq").unwrap(), "Ba chục năm");
    }
    /// Luật "lỗi này thuộc ô nào" phải khớp **đúng câu chữ thật** của mô-đun này, không phải
    /// câu chữ tưởng tượng. Sửa một thông báo mà quên đây thì test này đổ.
    #[test]
    fn moi_thong_bao_that_deu_tro_dung_cot() {
        for (tb, cot) in [
            ("Thiếu tiêu đề hồ sơ", "tieuDe"),
            ("Chưa có dòng phòng ban (I. …) phía trên", "tieuDe"),
            ("THBQ trống — dùng mặc định \"Vĩnh viễn\"", "thbq"),
            ("Thiếu thời hạn bảo quản (chọn THBQ mặc định cho tệp hoặc điền vào ô)", "thbq"),
            ("Thời hạn bảo quản \"70 nam\" không có trong Danh mục", "thbq"),
            ("Hồ sơ số K39 trùng với dòng 12 (cùng loại Vĩnh viễn)", "hoSoSo"),
            ("Chưa có Hộp số — nhập không xếp hộp (xếp sau ở Hệ thống hoá)", "hopSo"),
            ("Số tờ 0 — để trống", "soTo"),
            ("Thiếu số tờ — tài liệu bảo quản Vĩnh viễn phải đánh số tờ", "soTo"),
        ] {
            assert_eq!(cot_cua_loi(tb), cot, "{tb:?}");
        }
        // Không khớp gì thì về tiêu đề, không được hoảng.
        assert_eq!(cot_cua_loi("câu gì đó chưa từng có"), "tieuDe");
        assert_eq!(cot_cua_loi(""), "tieuDe");
    }

}
