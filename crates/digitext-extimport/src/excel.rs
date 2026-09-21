// Copyright (c) 2026 DigiText. All rights reserved.
//! Đọc tệp Excel metadata PHẲNG: mỗi dòng một tài liệu, mang luôn thông tin phông và hồ sơ.
//!
//! Tiêu đề cột lặp lại giữa ba khối (phông / hồ sơ / tài liệu): "Ngôn ngữ", "Ghi chú",
//! "Thời hạn lưu trữ", "Số lượng trang"… nên KHÔNG thể tra theo tên cột đơn thuần. Cách
//! làm: tìm hai mốc "Mã hồ sơ" (đầu khối hồ sơ) và "Mã định danh/Mã lưu trữ tài liệu" (đầu
//! khối tài liệu), rồi dò tên cột TRONG từng khối. Tên cột so sau khi bỏ dấu, thường hoá.
use calamine::{open_workbook_auto, Data, Reader};
use std::path::Path;

/// Một dòng Excel = một tài liệu. Mọi trường là chuỗi đã trim; rỗng = `None`.
#[derive(Debug, Clone, Default)]
pub struct Row {
    /// Số dòng trong Excel (1 = dòng đầu tiên của sheet) — để báo lỗi chỉ đúng chỗ.
    pub line: u32,
    // --- phông
    pub fond_code: Option<String>,
    pub fond_name: Option<String>,
    // --- hồ sơ
    pub hs_code: Option<String>,
    pub hs_title: Option<String>,
    pub hs_thbq: Option<String>,
    pub hs_mode: Option<String>,
    pub hs_lang: Option<String>,
    pub hs_start: Option<String>,
    pub hs_end: Option<String>,
    pub hs_keyword: Option<String>,
    pub hs_total_doc: Option<String>,
    pub hs_papers: Option<String>,
    pub hs_pages: Option<String>,
    pub hs_physical: Option<String>,
    pub hs_infor_sign: Option<String>,
    pub hs_paper_code: Option<String>,
    pub hs_dir: Option<String>,
    pub hs_note: Option<String>,
    // --- tài liệu
    pub doc_id: Option<String>,
    pub doc_file_code: Option<String>,
    pub doc_thbq: Option<String>,
    pub doc_type: Option<String>,
    pub doc_number: Option<String>,
    pub doc_notation: Option<String>,
    pub doc_date: Option<String>,
    pub doc_organ: Option<String>,
    pub doc_subject: Option<String>,
    pub doc_lang: Option<String>,
    pub doc_pages: Option<String>,
    pub doc_infor_sign: Option<String>,
    pub doc_keyword: Option<String>,
    pub doc_mode: Option<String>,
    pub doc_autograph: Option<String>,
    pub doc_physical: Option<String>,
    pub doc_path: Option<String>,
    pub doc_note: Option<String>,
}

#[derive(Debug)]
pub struct Sheet {
    pub sheet_name: String,
    pub header_line: u32,
    pub rows: Vec<Row>,
    /// Cột bắt buộc không tìm thấy — khác rỗng thì không dùng được tệp này.
    pub missing_columns: Vec<String>,
    /// Tên trường → tiêu đề cột đã khớp (để người dùng soát việc nhận cột).
    pub mapping: Vec<(String, String)>,
}

/// Bỏ dấu tiếng Việt + thường hoá + gộp khoảng trắng, để so tên cột.
pub fn fold(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        let c = match c {
            'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ầ' | 'ấ' | 'ẩ'
            | 'ẫ' | 'ậ' | 'À' | 'Á' | 'Ả' | 'Ã' | 'Ạ' | 'Ă' | 'Ằ' | 'Ắ' | 'Ẳ' | 'Ẵ' | 'Ặ' | 'Â' | 'Ầ'
            | 'Ấ' | 'Ẩ' | 'Ẫ' | 'Ậ' => 'a',
            'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' | 'È' | 'É' | 'Ẻ' | 'Ẽ'
            | 'Ẹ' | 'Ê' | 'Ề' | 'Ế' | 'Ể' | 'Ễ' | 'Ệ' => 'e',
            'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' | 'Ì' | 'Í' | 'Ỉ' | 'Ĩ' | 'Ị' => 'i',
            'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ờ' | 'ớ' | 'ở'
            | 'ỡ' | 'ợ' | 'Ò' | 'Ó' | 'Ỏ' | 'Õ' | 'Ọ' | 'Ô' | 'Ồ' | 'Ố' | 'Ổ' | 'Ỗ' | 'Ộ' | 'Ơ' | 'Ờ'
            | 'Ớ' | 'Ở' | 'Ỡ' | 'Ợ' => 'o',
            'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' | 'Ù' | 'Ú' | 'Ủ' | 'Ũ'
            | 'Ụ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ử' | 'Ữ' | 'Ự' => 'u',
            'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' | 'Ỳ' | 'Ý' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'y',
            'đ' | 'Đ' => 'd',
            // Dấu tổ hợp (Unicode NFD) — bỏ hẳn.
            '\u{0300}'..='\u{036f}' => continue,
            c => c.to_ascii_lowercase(),
        };
        out.push(c);
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cell_to_string(c: &Data) -> Option<String> {
    let s = match c {
        Data::Empty => return None,
        Data::String(s) => s.trim().to_string(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => match dt.as_datetime() {
            Some(d) => d.format("%d/%m/%Y").to_string(),
            None => dt.to_string(),
        },
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.trim().to_string(),
        Data::Error(_) => return None,
    };
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// (tên trường, [mẫu tên cột đã fold — khớp theo `starts_with`], bắt buộc?)
type Spec = (&'static str, &'static [&'static str], bool);
const FOND: &[Spec] = &[
    ("fond_code", &["ma phong"], true),
    ("fond_name", &["ten phong"], false),
];
const HOSO: &[Spec] = &[
    ("hs_paper_code", &["ma ho so goc", "ma ho so giay"], false),
    ("hs_code", &["ma ho so"], true),
    ("hs_title", &["tieu de ho so", "tieu de"], true),
    ("hs_thbq", &["thoi han luu tru", "thoi han bao quan"], true),
    ("hs_mode", &["muc do tiep can", "che do su dung"], false),
    ("hs_lang", &["ngon ngu"], false),
    ("hs_start", &["thoi gian bat dau"], false),
    ("hs_end", &["thoi gian ket thuc"], false),
    ("hs_keyword", &["tu khoa"], false),
    ("hs_total_doc", &["tong so tai lieu"], false),
    ("hs_papers", &["so luong to"], false),
    ("hs_pages", &["so luong trang"], false),
    ("hs_physical", &["tinh trang vat ly"], false),
    ("hs_infor_sign", &["ky hieu thong tin"], false),
    ("hs_dir", &["tep tin ho so"], false),
    ("hs_note", &["ghi chu"], false),
];
const DOC: &[Spec] = &[
    ("doc_id", &["ma dinh danh tai lieu"], false),
    ("doc_file_code", &["ma luu tru tai lieu"], false),
    ("doc_thbq", &["thoi han luu tru", "thoi han bao quan"], false),
    ("doc_type", &["ten loai"], false),
    ("doc_number", &["so cua tai lieu", "so van ban"], false),
    ("doc_notation", &["ky hieu cua tai lieu", "ky hieu van ban"], false),
    ("doc_date", &["ngay thang nam", "ngay ban hanh"], false),
    ("doc_organ", &["ten co quan"], false),
    ("doc_subject", &["trich yeu"], false),
    ("doc_lang", &["ngon ngu"], false),
    ("doc_pages", &["so luong trang", "so trang"], false),
    ("doc_infor_sign", &["ky hieu thong tin"], false),
    ("doc_keyword", &["tu khoa"], false),
    ("doc_mode", &["muc do tiep can", "che do su dung"], false),
    ("doc_autograph", &["but tich"], false),
    ("doc_physical", &["tinh trang vat ly"], false),
    ("doc_path", &["tep tin tai lieu"], true),
    ("doc_note", &["ghi chu"], false),
];

fn set_field(r: &mut Row, name: &str, v: Option<String>) {
    match name {
        "fond_code" => r.fond_code = v,
        "fond_name" => r.fond_name = v,
        "hs_code" => r.hs_code = v,
        "hs_title" => r.hs_title = v,
        "hs_thbq" => r.hs_thbq = v,
        "hs_mode" => r.hs_mode = v,
        "hs_lang" => r.hs_lang = v,
        "hs_start" => r.hs_start = v,
        "hs_end" => r.hs_end = v,
        "hs_keyword" => r.hs_keyword = v,
        "hs_total_doc" => r.hs_total_doc = v,
        "hs_papers" => r.hs_papers = v,
        "hs_pages" => r.hs_pages = v,
        "hs_physical" => r.hs_physical = v,
        "hs_infor_sign" => r.hs_infor_sign = v,
        "hs_paper_code" => r.hs_paper_code = v,
        "hs_dir" => r.hs_dir = v,
        "hs_note" => r.hs_note = v,
        "doc_id" => r.doc_id = v,
        "doc_file_code" => r.doc_file_code = v,
        "doc_thbq" => r.doc_thbq = v,
        "doc_type" => r.doc_type = v,
        "doc_number" => r.doc_number = v,
        "doc_notation" => r.doc_notation = v,
        "doc_date" => r.doc_date = v,
        "doc_organ" => r.doc_organ = v,
        "doc_subject" => r.doc_subject = v,
        "doc_lang" => r.doc_lang = v,
        "doc_pages" => r.doc_pages = v,
        "doc_infor_sign" => r.doc_infor_sign = v,
        "doc_keyword" => r.doc_keyword = v,
        "doc_mode" => r.doc_mode = v,
        "doc_autograph" => r.doc_autograph = v,
        "doc_physical" => r.doc_physical = v,
        "doc_path" => r.doc_path = v,
        "doc_note" => r.doc_note = v,
        _ => {}
    }
}

/// Dò cột của một khối trong đoạn `[lo, hi)` của dòng tiêu đề.
fn map_block(
    specs: &[Spec],
    headers: &[String],
    lo: usize,
    hi: usize,
    out: &mut Vec<(&'static str, usize)>,
    missing: &mut Vec<String>,
) {
    let mut used = vec![false; headers.len()];
    for (name, pats, required) in specs {
        let hit = (lo..hi.min(headers.len()))
            .find(|&i| !used[i] && pats.iter().any(|p| headers[i].starts_with(p)));
        match hit {
            Some(i) => {
                used[i] = true;
                out.push((name, i));
            }
            None if *required => missing.push(format!("{name} ({})", pats[0])),
            None => {}
        }
    }
}

/// Mở Excel, tự tìm sheet + dòng tiêu đề (trong 15 dòng đầu), trả mọi dòng dữ liệu.
pub fn read(path: &Path) -> Result<Sheet, String> {
    let mut wb = open_workbook_auto(path).map_err(|e| format!("không mở được Excel {}: {e}", path.display()))?;
    let names = wb.sheet_names().to_vec();
    let mut best: Option<(usize, String, u32, calamine::Range<Data>)> = None;
    for name in names {
        let Ok(range) = wb.worksheet_range(&name) else { continue };
        for (i, row) in range.rows().take(15).enumerate() {
            let folded: Vec<String> = row.iter().map(|c| fold(&cell_to_string(c).unwrap_or_default())).collect();
            let score = folded
                .iter()
                .filter(|h| {
                    h.starts_with("ma ho so") || h.starts_with("ma phong") || h.starts_with("tep tin tai lieu")
                        || h.starts_with("trich yeu") || h.starts_with("tieu de ho so")
                })
                .count();
            if score >= 3 && best.as_ref().map_or(true, |b| score > b.0) {
                best = Some((score, name.clone(), i as u32, range.clone()));
            }
        }
    }
    let Some((_, sheet_name, header_idx, range)) = best else {
        return Err("không tìm thấy dòng tiêu đề (cần các cột: Mã phông, Mã hồ sơ, Tiêu đề hồ sơ, Tệp tin tài liệu…)".into());
    };
    let first_row = range.start().map(|s| s.0).unwrap_or(0);
    let raw_headers: Vec<String> = range
        .rows()
        .nth(header_idx as usize)
        .map(|r| r.iter().map(|c| cell_to_string(c).unwrap_or_default()).collect())
        .unwrap_or_default();
    let headers: Vec<String> = raw_headers.iter().map(|h| fold(h)).collect();
    let hs_at = headers.iter().position(|h| h.starts_with("ma ho so") && !h.contains("goc") && !h.contains("giay"));
    let doc_at = headers
        .iter()
        .position(|h| h.starts_with("ma dinh danh tai lieu") || h.starts_with("ma luu tru tai lieu"));
    let n = headers.len();
    let hs_lo = hs_at.unwrap_or(0);
    let doc_lo = doc_at.unwrap_or(hs_lo);
    let mut cols: Vec<(&'static str, usize)> = Vec::new();
    let mut missing = Vec::new();
    map_block(FOND, &headers, 0, if hs_at.is_some() { hs_lo } else { n }, &mut cols, &mut missing);
    map_block(HOSO, &headers, hs_lo, if doc_at.is_some() { doc_lo } else { n }, &mut cols, &mut missing);
    map_block(DOC, &headers, doc_lo, n, &mut cols, &mut missing);

    let mut rows = Vec::new();
    for (i, row) in range.rows().enumerate().skip(header_idx as usize + 1) {
        if row.iter().all(|c| matches!(c, Data::Empty)) {
            continue;
        }
        let mut r = Row { line: first_row + i as u32 + 1, ..Row::default() };
        for (name, idx) in &cols {
            set_field(&mut r, name, row.get(*idx).and_then(cell_to_string));
        }
        rows.push(r);
    }
    Ok(Sheet {
        sheet_name,
        header_line: first_row + header_idx + 1,
        rows,
        missing_columns: missing,
        mapping: cols.iter().map(|(n, i)| (n.to_string(), raw_headers[*i].trim().to_string())).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fold_bo_dau_va_gop_khoang_trang() {
        assert_eq!(fold("  Mã Phông,  công trình "), "ma phong, cong trinh");
        assert_eq!(fold("Thời hạn lưu trữ "), "thoi han luu tru");
        assert_eq!(fold("ĐƯỜNG"), "duong");
    }
    #[test]
    fn cot_trung_ten_duoc_tach_theo_khoi() {
        let raw = [
            "Mã Phông", "Tên Phông", "Ngôn ngữ", "Ghi chú", "Mã hồ sơ", "Tiêu đề hồ sơ", "Thời hạn lưu trữ",
            "Ngôn ngữ", "Số lượng trang", "Mã hồ sơ gốc giấy", "Ghi chú", "Mã định danh tài liệu",
            "Mã lưu trữ tài liệu", "Thời hạn lưu trữ", "Ngôn ngữ", "Số lượng trang", "Tệp tin tài liệu", "Ghi chú",
        ];
        let headers: Vec<String> = raw.iter().map(|h| fold(h)).collect();
        let (mut cols, mut missing) = (Vec::new(), Vec::new());
        map_block(FOND, &headers, 0, 4, &mut cols, &mut missing);
        map_block(HOSO, &headers, 4, 11, &mut cols, &mut missing);
        map_block(DOC, &headers, 11, headers.len(), &mut cols, &mut missing);
        assert!(missing.is_empty(), "{missing:?}");
        let at = |n: &str| cols.iter().find(|c| c.0 == n).map(|c| c.1);
        assert_eq!(at("hs_code"), Some(4));
        assert_eq!(at("hs_paper_code"), Some(9));
        assert_eq!(at("hs_lang"), Some(7));
        assert_eq!(at("doc_lang"), Some(14));
        assert_eq!(at("hs_pages"), Some(8));
        assert_eq!(at("doc_pages"), Some(15));
        assert_eq!(at("hs_thbq"), Some(6));
        assert_eq!(at("doc_thbq"), Some(13));
        assert_eq!(at("doc_path"), Some(16));
    }
}
