// Copyright (c) 2026 DigiText. All rights reserved.
//! Đặt tên thư mục / tệp cho bước "Tách TIFF" (tài liệu Đảng). Mỗi đơn vị một mẫu; hai mẫu dựng sẵn chép theo
//! tool Python cũ (`Tach_tiff.zip`):
//! - v1 (`TIFFBD`): đổi thư mục chứa PDF thành mã hồ sơ suy từ tên PDF đầu tiên —
//!   `H49.44-ĐU-KHAC-0001-1976` → `H49.44-ĐU-KHAC-1976` (bỏ STT giữa, giữ năm);
//!   `H49.44-001-01-0025-001` / `H49.44-001-01-0230.001` → bỏ STT cuối.
//! - v2 (`TIFFTV`): tên có mã loại văn bản Đảng (`-ĐU-`, `-NQ-`…) → giữ tên thư mục gốc (`01`, `02`…);
//!   tên dạng mã cơ quan `H49.44-001-01-0042-001` → `H49.44-001-01-0042`.
//!
//! Tệp TIFF: `<tên PDF><phân cách><STT>.tif`, STT chạy liên tục qua các PDF (sắp theo tên) của cùng thư mục
//! nguồn, sang thư mục khác đánh lại từ số bắt đầu. Khác tool cũ: thứ tự là tất định (không phụ thuộc thứ tự
//! bấm, không "nhớ STT" giữa các lần chạy) nên chạy lại cho đúng cùng một kết quả.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TiffNamingRule {
    /// v1 — "Bỏ STT, giữ năm".
    V1,
    /// v2 — "Văn bản Đảng giữ thư mục gốc".
    V2,
}

pub const DEFAULT_DOC_TYPE_CODES: [&str; 10] = ["ĐU", "NQ", "QĐ", "SNQ", "KHAC", "TL", "BC", "CT", "TB", "HD"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TiffNaming {
    pub rule: TiffNamingRule,
    /// Dấu phân cách giữa tên PDF và STT: "_", "-", " " hoặc "".
    pub separator: String,
    /// Số chữ số của STT (3–6).
    pub pad: u8,
    /// STT bắt đầu (mặc định 1).
    pub start: u32,
    /// Giữ chuỗi thư mục cha, chỉ đổi thư mục cấp cuối. Tắt: mọi hồ sơ nằm thẳng trong thư mục xuất.
    pub mirror: bool,
    /// Mã loại văn bản Đảng (mẫu v2; v1 dùng để nhận dạng dạng "có năm").
    pub doc_type_codes: Vec<String>,
}

impl TiffNaming {
    /// Cấu hình mặc định giống `tool_config.json` / `tool_clone_config.json` của tool cũ:
    /// phân cách "-", STT 3 chữ số, bắt đầu 1, giữ thư mục cha.
    pub fn preset(rule: TiffNamingRule) -> Self {
        TiffNaming {
            rule,
            separator: "-".into(),
            pad: 3,
            start: 1,
            mirror: true,
            doc_type_codes: DEFAULT_DOC_TYPE_CODES.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if !(3..=6).contains(&self.pad) {
            return Err("độ dài STT phải từ 3 đến 6 chữ số".into());
        }
        if !matches!(self.separator.as_str(), "_" | "-" | " " | "") {
            return Err("dấu phân cách chỉ được là _, -, khoảng trắng hoặc để trống".into());
        }
        if self.start == 0 {
            return Err("STT bắt đầu phải từ 1".into());
        }
        Ok(())
    }

    fn has_doc_type(&self, stem: &str) -> bool {
        let up = stem.to_uppercase();
        let segs: Vec<&str> = up.split('-').collect();
        // Python cũ: `-(ĐU|NQ|…)-` — mã nằm giữa hai dấu gạch.
        segs.len() >= 3
            && segs[1..segs.len() - 1]
                .iter()
                .any(|s| self.doc_type_codes.iter().any(|c| c.to_uppercase() == *s))
    }

    /// Tên mới cho thư mục chứa PDF, suy từ tên (bỏ đuôi) của PDF đầu tiên theo thứ tự tên.
    pub fn folder_name(&self, first_stem: &str, original_folder: &str) -> String {
        match self.rule {
            TiffNamingRule::V1 => {
                if let Some(s) = drop_middle_seq_keep_year(first_stem).filter(|_| self.has_doc_type(first_stem)) {
                    return s;
                }
                drop_last_seq(first_stem).unwrap_or_else(|| first_stem.to_string())
            }
            TiffNamingRule::V2 => {
                if self.has_doc_type(first_stem) {
                    return original_folder.to_string();
                }
                if let Some(s) = agency_code_prefix(first_stem) {
                    return s;
                }
                if !first_stem.chars().any(char::is_alphabetic) {
                    if let Some(s) = drop_last_seq(first_stem) {
                        return s;
                    }
                }
                original_folder.to_string()
            }
        }
    }

    /// Chuỗi thư mục xuất (tương đối so với thư mục xuất) cho một thư mục nguồn.
    /// `source_dirs`: chuỗi thư mục nguồn tính từ thư mục được chọn, GỒM cả tên thư mục được chọn
    /// (vd. `["BINHDUONG","01","01"]`); phần tử cuối là thư mục chứa PDF.
    pub fn output_dirs(&self, source_dirs: &[String], first_stem: &str) -> Vec<String> {
        let last = source_dirs.last().map(String::as_str).unwrap_or("");
        let name = sanitize_segment(&self.folder_name(first_stem, last));
        if !self.mirror || source_dirs.is_empty() {
            return vec![name];
        }
        if source_dirs.len() == 1 {
            // PDF nằm ngay trong thư mục được chọn: v1 lồng dưới tên thư mục đó, v2 đặt thẳng (như tool cũ).
            return match self.rule {
                TiffNamingRule::V1 => vec![source_dirs[0].clone(), name],
                TiffNamingRule::V2 => vec![name],
            };
        }
        let mut out = source_dirs[..source_dirs.len() - 1].to_vec();
        out.push(name);
        out
    }

    pub fn file_name(&self, pdf_stem: &str, seq: u32) -> String {
        format!("{pdf_stem}{}{:0width$}.tif", self.separator, seq, width = self.pad as usize)
    }
}

fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// v1 dạng có năm: `<tiền tố>-<STT>-<năm 4 số>` → `<tiền tố>-<năm>`; năm phải trong 1900–2100.
fn drop_middle_seq_keep_year(stem: &str) -> Option<String> {
    let (rest, year) = stem.rsplit_once('-')?;
    let (prefix, seq) = rest.rsplit_once('-')?;
    let y: u32 = year.parse().ok().filter(|_| year.len() == 4 && all_digits(year))?;
    (all_digits(seq) && !prefix.is_empty() && (1900..=2100).contains(&y)).then(|| format!("{prefix}-{year}"))
}

/// Bỏ STT cuối: `<tiền tố>-<số>` hoặc `<tiền tố>.<số>` (dấu chấm chỉ nhận khi tiền tố có dấu gạch,
/// để không cắt nhầm `H49.44`).
fn drop_last_seq(stem: &str) -> Option<String> {
    let i = stem.rfind(['-', '.'])?;
    let (prefix, seq) = (&stem[..i], &stem[i + 1..]);
    if !all_digits(seq) || prefix.is_empty() {
        return None;
    }
    if stem.as_bytes()[i] == b'.' && !prefix.contains('-') {
        return None;
    }
    Some(prefix.to_string())
}

/// v2 mã cơ quan: `H?<số>[.số]*-<số>-<số>-<số>` + `-`/`.` + STT → phần trước STT.
fn agency_code_prefix(stem: &str) -> Option<String> {
    let prefix = drop_last_seq(stem)?;
    let parts: Vec<&str> = prefix.split('-').collect();
    if parts.len() != 4 || !parts[1..].iter().all(|p| all_digits(p)) {
        return None;
    }
    let p0 = parts[0].strip_prefix(['H', 'h']).unwrap_or(parts[0]);
    let ok = p0.starts_with(|c: char| c.is_ascii_digit()) && p0.chars().all(|c| c.is_ascii_digit() || c == '.');
    ok.then_some(prefix)
}

/// Ký tự không hợp lệ trong tên thư mục Windows → "_"; bỏ dấu chấm/khoảng trắng cuối.
pub fn sanitize_segment(s: &str) -> String {
    let t: String = s
        .chars()
        .map(|c| if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || c.is_control() { '_' } else { c })
        .collect();
    let t = t.trim_end_matches(['.', ' ']).trim_start();
    if t.is_empty() { "_".into() } else { t.to_string() }
}

/// Một PDF nguồn trong kế hoạch tách.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanInput {
    /// Chuỗi thư mục nguồn (gồm tên thư mục được chọn), phần tử cuối = thư mục chứa PDF.
    pub source_dirs: Vec<String>,
    /// Tên tệp PDF (có đuôi).
    pub file_name: String,
    pub pages: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanOutput {
    /// Chỉ số trong danh sách đầu vào.
    pub input: usize,
    pub output_dirs: Vec<String>,
    /// Tên TIFF cho từng trang theo thứ tự.
    pub tiff_names: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub files: Vec<PlanOutput>,
    /// Hai thư mục nguồn khác nhau ra cùng một thư mục xuất mà trùng tên TIFF — phải xử lý trước khi chạy.
    pub collisions: Vec<String>,
}

fn stem(file_name: &str) -> &str {
    match file_name.rsplit_once('.') {
        Some((s, ext)) if ext.eq_ignore_ascii_case("pdf") => s,
        _ => file_name,
    }
}

/// Một thư mục nguồn trong đợt tách: các PDF (chỉ số đầu vào, đã sắp theo tên) và thư mục xuất.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderPlan {
    pub source_dirs: Vec<String>,
    pub output_dirs: Vec<String>,
    pub files: Vec<usize>,
}

/// Gom PDF theo thư mục nguồn (không cần số trang). `inputs`: (chuỗi thư mục nguồn, tên tệp PDF).
/// Trả thêm danh sách xung đột: hai thư mục nguồn khác nhau ra cùng thư mục xuất và có PDF trùng tên
/// (→ chắc chắn trùng tên TIFF). Trong cùng thư mục nguồn tên PDF đã khác nhau nên không trùng.
pub fn group(naming: &TiffNaming, inputs: &[(Vec<String>, String)]) -> (Vec<FolderPlan>, Vec<String>) {
    use std::collections::{BTreeMap, HashMap};
    let mut groups: BTreeMap<&[String], Vec<usize>> = BTreeMap::new();
    for (i, (dirs, _)) in inputs.iter().enumerate() {
        groups.entry(dirs.as_slice()).or_default().push(i);
    }
    let mut out = Vec::with_capacity(groups.len());
    let mut owner: HashMap<String, &[String]> = HashMap::new();
    let mut collisions = Vec::new();
    for (dirs, mut idx) in groups {
        idx.sort_by(|a, b| natural_cmp(&inputs[*a].1, &inputs[*b].1));
        let output_dirs = naming.output_dirs(dirs, stem(&inputs[idx[0]].1));
        for &i in &idx {
            let key = format!("{}/{}", output_dirs.join("/"), stem(&inputs[i].1)).to_lowercase();
            match owner.get(key.as_str()) {
                Some(o) if *o != dirs => collisions.push(format!(
                    "{}: thư mục \"{}\" và \"{}\" cùng ra \"{}\"",
                    inputs[i].1,
                    o.join("/"),
                    dirs.join("/"),
                    output_dirs.join("/")
                )),
                _ => {
                    owner.insert(key, dirs);
                }
            }
        }
        out.push(FolderPlan { source_dirs: dirs.to_vec(), output_dirs, files: idx });
    }
    (out, collisions)
}

/// Lập kế hoạch tên cho cả đợt: gom theo thư mục nguồn, sắp PDF theo tên, đánh STT liên tục trong thư mục.
pub fn plan(naming: &TiffNaming, inputs: &[PlanInput]) -> Plan {
    use std::collections::{BTreeMap, HashMap};
    let mut groups: BTreeMap<&[String], Vec<usize>> = BTreeMap::new();
    for (i, f) in inputs.iter().enumerate() {
        groups.entry(f.source_dirs.as_slice()).or_default().push(i);
    }
    let mut files = Vec::with_capacity(inputs.len());
    let mut owner: HashMap<String, &[String]> = HashMap::new();
    let mut collisions = Vec::new();
    for (dirs, mut idx) in groups {
        idx.sort_by(|a, b| natural_cmp(&inputs[*a].file_name, &inputs[*b].file_name));
        let first = stem(&inputs[idx[0]].file_name);
        let out = naming.output_dirs(dirs, first);
        let mut seq = naming.start;
        for i in idx {
            let s = stem(&inputs[i].file_name);
            let names: Vec<String> = (0..inputs[i].pages)
                .map(|_| {
                    let n = naming.file_name(s, seq);
                    seq += 1;
                    n
                })
                .collect();
            for n in &names {
                let key = format!("{}/{}", out.join("/"), n).to_lowercase();
                match owner.get(key.as_str()) {
                    Some(o) if *o != dirs => collisions.push(format!(
                        "{} — trùng giữa thư mục \"{}\" và \"{}\"",
                        key,
                        o.join("/"),
                        dirs.join("/")
                    )),
                    Some(_) => collisions.push(format!("{key} — trùng trong cùng thư mục \"{}\"", dirs.join("/"))),
                    None => {
                        owner.insert(key, dirs);
                    }
                }
            }
            files.push(PlanOutput { input: i, output_dirs: out.clone(), tiff_names: names });
        }
    }
    files.sort_by_key(|f| f.input);
    Plan { files, collisions }
}

/// So tên kiểu Explorer: đoạn số so theo giá trị (`HO SO 9` < `HO SO 10`). Tên PDF mã hồ sơ đã đệm 0
/// nên kết quả giống `sorted()` của tool cũ.
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return a.cmp(b),
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                let mut n1 = String::new();
                while let Some(c) = x.peek().copied().filter(char::is_ascii_digit) {
                    n1.push(c);
                    x.next();
                }
                let mut n2 = String::new();
                while let Some(d) = y.peek().copied().filter(char::is_ascii_digit) {
                    n2.push(d);
                    y.next();
                }
                let (t1, t2) = (n1.trim_start_matches('0'), n2.trim_start_matches('0'));
                let o = t1.len().cmp(&t2.len()).then_with(|| t1.cmp(t2)).then_with(|| n1.len().cmp(&n2.len()));
                if o != Ordering::Equal {
                    return o;
                }
            }
            (Some(c), Some(d)) => {
                let o = c.to_lowercase().cmp(d.to_lowercase());
                if o != Ordering::Equal {
                    return o;
                }
                x.next();
                y.next();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn v(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn v1_vi_du_huong_dan() {
        let n = TiffNaming::preset(TiffNamingRule::V1);
        assert_eq!(n.folder_name("H49.44-ĐU-KHAC-0001-1976", "07"), "H49.44-ĐU-KHAC-1976");
        assert_eq!(n.folder_name("H49.44-001-01-0230.001", "07"), "H49.44-001-01-0230");
        assert_eq!(n.folder_name("H49.44-001-01-0025-001", "07"), "H49.44-001-01-0025");
        // Mã cơ quan đuôi 4 số không phải năm Đảng: không bị hiểu nhầm thành "bỏ STT giữa".
        assert_eq!(n.folder_name("H49.44-001-01-0025-2002", "07"), "H49.44-001-01-0025");
        assert_eq!(n.folder_name("H49.44", "07"), "H49.44");
        assert_eq!(
            n.output_dirs(&v(&["BINHDUONG", "01", "01"]), "H49.44-ĐU-KHAC-0001-1976"),
            v(&["BINHDUONG", "01", "H49.44-ĐU-KHAC-1976"])
        );
        assert_eq!(n.output_dirs(&v(&["BINHDUONG"]), "H49.44-001-01-0025-001"), v(&["BINHDUONG", "H49.44-001-01-0025"]));
    }

    #[test]
    fn v2_vi_du_huong_dan() {
        let n = TiffNaming::preset(TiffNamingRule::V2);
        assert_eq!(n.folder_name("H49.44-ĐU-NQ-0001-2002", "01"), "01");
        assert_eq!(n.folder_name("h49.44-đu-nq-0001-2002", "01"), "01");
        assert_eq!(n.folder_name("H49.44-001-01-0122-001", "42"), "H49.44-001-01-0122");
        assert_eq!(n.folder_name("H49.44-001-01-0042.001", "42"), "H49.44-001-01-0042");
        assert_eq!(n.folder_name("Bien ban hop", "42"), "42");
        assert_eq!(
            n.output_dirs(&v(&["TANVIET", "TIFF", "06", "42"]), "H49.44-001-01-0042-001"),
            v(&["TANVIET", "TIFF", "06", "H49.44-001-01-0042"])
        );
        assert_eq!(n.output_dirs(&v(&["TANVIET"]), "H49.44-001-01-0042-001"), v(&["H49.44-001-01-0042"]));
        let mut off = n.clone();
        off.mirror = false;
        assert_eq!(off.output_dirs(&v(&["A", "B", "42"]), "H49.44-001-01-0042-001"), v(&["H49.44-001-01-0042"]));
    }

    #[test]
    fn stt_lien_tuc_trong_thu_muc_va_dat_lai_khi_sang_thu_muc_khac() {
        let n = TiffNaming::preset(TiffNamingRule::V2);
        let inp = vec![
            // Cố tình xáo thứ tự — kết quả vẫn theo tên.
            PlanInput { source_dirs: v(&["R", "06", "42"]), file_name: "H49.44-001-01-0042-002.pdf".into(), pages: 2 },
            PlanInput { source_dirs: v(&["R", "06", "43"]), file_name: "H49.44-001-01-0043-001.pdf".into(), pages: 1 },
            PlanInput { source_dirs: v(&["R", "06", "42"]), file_name: "H49.44-001-01-0042-001.pdf".into(), pages: 3 },
        ];
        let p = plan(&n, &inp);
        assert!(p.collisions.is_empty(), "{:?}", p.collisions);
        assert_eq!(p.files[2].tiff_names, v(&["H49.44-001-01-0042-001-001.tif", "H49.44-001-01-0042-001-002.tif", "H49.44-001-01-0042-001-003.tif"]));
        assert_eq!(p.files[0].tiff_names, v(&["H49.44-001-01-0042-002-004.tif", "H49.44-001-01-0042-002-005.tif"]));
        assert_eq!(p.files[1].tiff_names, v(&["H49.44-001-01-0043-001-001.tif"]));
        assert_eq!(p.files[1].output_dirs, v(&["R", "06", "H49.44-001-01-0043"]));
    }

    #[test]
    fn bao_trung_khi_hai_thu_muc_ra_cung_mot_cho() {
        // v1, tắt giữ thư mục cha: hai thư mục khác nhau cùng mã hồ sơ → trùng tên TIFF.
        let mut n = TiffNaming::preset(TiffNamingRule::V1);
        n.mirror = false;
        let inp = vec![
            PlanInput { source_dirs: v(&["R", "a"]), file_name: "H49.44-ĐU-NQ-0001-2002.pdf".into(), pages: 1 },
            PlanInput { source_dirs: v(&["R", "b"]), file_name: "H49.44-ĐU-NQ-0001-2002.pdf".into(), pages: 1 },
        ];
        assert_eq!(plan(&n, &inp).collisions.len(), 1);
        let g: Vec<_> = inp.iter().map(|p| (p.source_dirs.clone(), p.file_name.clone())).collect();
        let (folders, coll) = group(&n, &g);
        assert_eq!((folders.len(), coll.len()), (2, 1));
        n.mirror = true;
        assert!(group(&n, &g).1.is_empty());
    }

    #[test]
    fn ten_tep_va_kiem_cau_hinh() {
        let mut n = TiffNaming::preset(TiffNamingRule::V1);
        n.separator = "_".into();
        n.pad = 4;
        n.start = 7;
        assert_eq!(n.file_name("X", 7), "X_0007.tif");
        assert!(n.validate().is_ok());
        n.pad = 2;
        assert!(n.validate().is_err());
        assert_eq!(sanitize_segment("a:b?. "), "a_b_");
        assert!(natural_cmp("HO SO 9", "HO SO 10").is_lt());
    }
}
