// Copyright (c) 2026 DigiText. All rights reserved.
//! BƯỚC KIỂM: hồ sơ lưu trên đĩa đã khớp tệp Excel metadata chưa?
//!
//! Đơn vị hồ sơ = THƯ MỤC chứa tệp (…\HOPnn\<số hồ sơ>), không phải "Mã hồ sơ": dữ liệu
//! thật có mã hồ sơ bị trùng giữa các thư mục khác nhau — chính là thứ bước kiểm phải bắt.
//!
//! Mức độ:  LỖI  = KHÔNG KHỚP về định danh hoặc dữ liệu: thiếu/thừa tệp, trùng mã hồ sơ, mã
//!                 hồ sơ lệch mã phông, thư mục lệch mã hồ sơ, «Tệp tin hồ sơ» lệch đường dẫn
//!                 thật, «Mã lưu trữ tài liệu» lệch tên tệp, tên tệp lệch mã phông/năm/số hồ
//!                 sơ, số trang Excel lệch PDF thật, thiếu trường bắt buộc, PDF hỏng…
//!                 → hồ sơ KHÔNG được đóng gói; mặc định còn lỗi ở BẤT KỲ đâu thì cả đợt
//!                 đóng gói không bắt đầu (yêu cầu nghiệp vụ: khớp hết rồi mới đóng gói).
//!                 Khi kiểm sâu: tệp CHƯA ĐẠT Điều 8.2.a TT05 (không phải PDF/A, thiếu lớp chữ,
//!                 < 200 dpi, < 24 bit màu, chưa ký số) cũng là LỖI — gói TT05 phải chuẩn.
//!          CẢNH BÁO = không phải chuyện khớp/lệch: ngày sai dạng hoặc thiếu, thiếu trích yếu,
//!                 STT không liên tục…
//! Lỗi liệt kê TỪNG dòng; cảnh báo gộp theo (hồ sơ, mã cảnh báo) kèm số lần + ví dụ đầu tiên.
use crate::excel::{Row, Sheet};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}
impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "LỖI",
            Severity::Warning => "CẢNH BÁO",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub severity: Severity,
    pub code: &'static str,
    /// Thư mục hồ sơ (rỗng = vấn đề cấp toàn kho).
    pub dossier: String,
    pub hs_code: String,
    /// Dòng Excel của lần gặp đầu tiên.
    pub line: Option<u32>,
    pub file: Option<String>,
    pub message: String,
    /// Số lần gặp (cảnh báo được gộp).
    pub count: u32,
}

/// Một hồ sơ = một thư mục + các dòng Excel trỏ vào nó (đã xếp theo số thứ tự tệp).
#[derive(Debug, Clone)]
pub struct Dossier {
    pub dir: String,
    pub rows: Vec<Row>,
}
impl Dossier {
    pub fn first(&self) -> &Row {
        &self.rows[0]
    }
    pub fn hs_code(&self) -> String {
        self.first().hs_code.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DossierSummary {
    pub dir: String,
    pub hs_code: String,
    pub title: String,
    pub fond_code: String,
    pub docs: usize,
    pub bytes: u64,
    pub errors: u32,
    pub warnings: u32,
    /// Chỉ có khi kiểm sâu.
    pub pdfa: Option<u32>,
    pub signed: Option<u32>,
    pub pages: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    pub excel: String,
    pub sheet: String,
    pub header_line: u32,
    pub root: String,
    pub deep: bool,
    pub rows: usize,
    pub files_on_disk: usize,
    pub dossiers: Vec<DossierSummary>,
    pub issues: Vec<Issue>,
    pub column_mapping: Vec<(String, String)>,
}
impl CheckReport {
    pub fn errors(&self) -> u32 {
        self.issues.iter().filter(|i| i.severity == Severity::Error).map(|i| i.count).sum()
    }
    pub fn warnings(&self) -> u32 {
        self.issues.iter().filter(|i| i.severity == Severity::Warning).map(|i| i.count).sum()
    }
    pub fn dossiers_ok(&self) -> usize {
        self.dossiers.iter().filter(|d| d.errors == 0).count()
    }
    /// Thư mục các hồ sơ KHÔNG có lỗi — đầu vào của bước đóng gói.
    pub fn packable(&self) -> HashSet<String> {
        self.dossiers.iter().filter(|d| d.errors == 0).map(|d| d.dir.clone()).collect()
    }
}

pub struct Plan {
    pub root: PathBuf,
    pub dossiers: Vec<Dossier>,
}

pub fn rel_to_path(root: &Path, rel: &str) -> PathBuf {
    let mut p = root.to_path_buf();
    for seg in rel.split(['\\', '/']).filter(|s| !s.is_empty()) {
        p.push(seg);
    }
    p
}
fn norm_rel(rel: &str) -> String {
    rel.split(['\\', '/']).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\\")
}
fn dir_of(rel: &str) -> String {
    match rel.rfind('\\') {
        Some(i) => rel[..i].to_string(),
        None => String::new(),
    }
}
fn base_of(rel: &str) -> &str {
    rel.rsplit('\\').next().unwrap_or(rel)
}
/// Đọc số nguyên chịu được rác nhập liệu: «36 '» → 36, «12 tờ» → 12. Trả (số, có rác?).
pub fn lenient_u32(s: &str) -> Option<(u32, bool)> {
    let t = s.trim();
    let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
    let n = digits.parse().ok()?;
    Some((n, digits.len() != t.len()))
}
/// Số thứ tự 7 chữ số ở đuôi tên tệp: `H49.132.02.2009.001.0000012.pdf` → 12.
pub fn seq_of(file_name: &str) -> Option<u32> {
    let stem = file_name.rsplit_once('.').map(|(s, _)| s).unwrap_or(file_name);
    let last = stem.rsplit('.').next()?;
    if last.len() == 7 && last.bytes().all(|b| b.is_ascii_digit()) {
        last.parse().ok()
    } else {
        None
    }
}

/// Gom dòng Excel thành hồ sơ theo thư mục. `root` được tự chỉnh nếu người dùng trỏ vào
/// thư mục con trùng với đoạn đầu của đường dẫn trong Excel.
pub fn plan(sheet: &Sheet, root: &Path) -> Plan {
    let mut root = root.to_path_buf();
    if let Some(first) = sheet.rows.iter().find_map(|r| r.doc_path.as_deref()) {
        let rel = norm_rel(first);
        if !rel_to_path(&root, &rel).exists() {
            if let Some(parent) = root.parent() {
                if rel_to_path(parent, &rel).exists() {
                    root = parent.to_path_buf();
                }
            }
        }
    }
    let mut map: BTreeMap<String, Vec<Row>> = BTreeMap::new();
    for r in &sheet.rows {
        let rel = r.doc_path.as_deref().map(norm_rel).unwrap_or_default();
        let dir = if rel.is_empty() {
            r.hs_dir.as_deref().map(norm_rel).unwrap_or_else(|| format!("?dòng {}", r.line))
        } else {
            dir_of(&rel)
        };
        map.entry(dir).or_default().push(r.clone());
    }
    let dossiers = map
        .into_iter()
        .map(|(dir, mut rows)| {
            rows.sort_by_key(|r| {
                let name = r.doc_path.as_deref().map(|p| base_of(&norm_rel(p)).to_string()).unwrap_or_default();
                (seq_of(&name).unwrap_or(u32::MAX), r.line)
            });
            Dossier { dir, rows }
        })
        .collect();
    Plan { root, dossiers }
}

fn walk_pdfs(dir: &Path, root: &Path, out: &mut HashMap<String, (String, u64)>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            walk_pdfs(&p, root, out);
        } else if p.extension().map_or(false, |x| x.eq_ignore_ascii_case("pdf")) {
            if let Ok(rel) = p.strip_prefix(root) {
                let rel = rel.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect::<Vec<_>>().join("\\");
                let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                out.insert(rel.to_lowercase(), (rel, size));
            }
        }
    }
}

/// `dd/mm/yyyy` chuẩn hoá (đệm 0, đổi yyyy-mm-dd). Trả (chuỗi dùng cho EAD, đúng dạng đầy đủ?).
pub fn norm_date(s: &str) -> (String, bool) {
    let t = s.trim();
    // Ô ngày bị Excel lưu thành SỐ THỨ TỰ NGÀY (39052 = 01/12/2006) — gặp thật trong dữ liệu.
    if (5..=6).contains(&t.len()) && t.bytes().all(|b| b.is_ascii_digit()) {
        if let Ok(n) = t.parse::<i64>() {
            if (20000..80000).contains(&n) {
                if let Some(d) = chrono::NaiveDate::from_ymd_opt(1899, 12, 30).and_then(|b| b.checked_add_signed(chrono::Duration::days(n))) {
                    return (d.format("%d/%m/%Y").to_string(), true);
                }
            }
        }
    }
    let parts: Vec<&str> = t.split(['/', '-']).map(str::trim).collect();
    let num = |x: &str| x.parse::<u32>().ok();
    if parts.len() == 3 && parts.iter().all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit())) {
        let (d, m, y) = if parts[0].len() == 4 {
            (num(parts[2]), num(parts[1]), num(parts[0]))
        } else {
            (num(parts[0]), num(parts[1]), num(parts[2]))
        };
        if let (Some(d), Some(m), Some(y)) = (d, m, y) {
            if (1..=31).contains(&d) && (1..=12).contains(&m) && (1800..=2100).contains(&y) {
                return (format!("{d:02}/{m:02}/{y:04}"), true);
            }
        }
    }
    (t.to_string(), false)
}
fn date_key(s: &str) -> Option<u32> {
    let (n, ok) = norm_date(s);
    if !ok {
        return None;
    }
    let p: Vec<u32> = n.split('/').filter_map(|x| x.parse().ok()).collect();
    Some(p[2] * 10000 + p[1] * 100 + p[0])
}

struct Sink {
    errors: Vec<Issue>,
    warns: BTreeMap<(String, &'static str), Issue>,
}
impl Sink {
    fn new() -> Self {
        Sink { errors: Vec::new(), warns: BTreeMap::new() }
    }
    fn error(&mut self, code: &'static str, d: &str, hs: &str, line: Option<u32>, file: Option<&str>, msg: String) {
        // Liệt kê tối đa 25 lỗi cùng loại mỗi hồ sơ; phần còn lại cộng dồn vào dòng thứ 25
        // (một hồ sơ 300 tệp lệch tên không nên sinh 300 dòng báo cáo giống nhau).
        let same = self.errors.iter().filter(|i| i.code == code && i.dossier == d).count();
        if same >= 25 {
            if let Some(last) = self.errors.iter_mut().rev().find(|i| i.code == code && i.dossier == d) {
                last.count += 1;
                if !last.message.ends_with("(và các tệp tiếp theo cùng lỗi)") {
                    last.message.push_str(" (và các tệp tiếp theo cùng lỗi)");
                }
            }
            return;
        }
        self.errors.push(Issue {
            severity: Severity::Error, code, dossier: d.into(), hs_code: hs.into(), line,
            file: file.map(Into::into), message: msg, count: 1,
        });
    }
    fn warn(&mut self, code: &'static str, d: &str, hs: &str, line: Option<u32>, file: Option<&str>, msg: String) {
        self.warns
            .entry((d.to_string(), code))
            .and_modify(|i| i.count += 1)
            .or_insert(Issue {
                severity: Severity::Warning, code, dossier: d.into(), hs_code: hs.into(), line,
                file: file.map(Into::into), message: msg, count: 1,
            });
    }
    fn merge(&mut self, other: Sink) {
        self.errors.extend(other.errors);
        for (k, v) in other.warns {
            self.warns.entry(k).and_modify(|i| i.count += v.count).or_insert(v);
        }
    }
}

/// Kết quả soi MỘT tệp PDF (dùng cả ở bước đóng gói).
#[derive(Debug, Clone, Default)]
pub struct PdfFacts {
    pub pages: u32,
    pub text_pages: u32,
    pub min_dpi: Option<u32>,
    pub min_bit_depth: Option<u32>,
    pub pdfa: bool,
    pub signed: bool,
    pub encrypted_or_broken: Option<String>,
}
fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}
impl PdfFacts {
    /// Các điểm CHƯA ĐẠT Điều 8 khoản 2 điểm a TT 05/2025 (PDF/A hai lớp, màu ≥ 24 bit,
    /// ≥ 200 dpi, có chữ ký số). Trả (mã vấn đề, nội dung). Rỗng = đạt.
    /// Lưu ý: "PDF/A" ở đây là tệp CÓ KHAI pdfaid trong XMP; hợp chuẩn ISO 19005 đầy đủ phải
    /// kiểm bằng veraPDF (dev/verapdf-gate.sh).
    pub fn tt05_failures(&self) -> Vec<(&'static str, String)> {
        let mut v = Vec::new();
        if !self.pdfa {
            v.push(("PDF-KHONG-PHAI-PDFA", "tệp KHÔNG phải PDF/A (thiếu pdfaid trong XMP) — TT05 Điều 8.2.a đòi PDF/A; chuyển đổi sau khi đã ký sẽ phá chữ ký, phải ký lại".to_string()));
        }
        if self.text_pages < self.pages {
            v.push(("PDF-THIEU-LOP-CHU", format!("{}/{} trang không có lớp chữ — TT05 Điều 8.2.a đòi PDF/A HAI LỚP", self.pages - self.text_pages, self.pages)));
        }
        if let Some(d) = self.min_dpi.filter(|d| *d < digitext_pdf::FM1_MIN_DPI) {
            v.push(("PDF-DPI-THAP", format!("độ phân giải {d} dpi < 200 dpi (TT05 Điều 8.2.a)")));
        }
        if let Some(b) = self.min_bit_depth.filter(|b| *b < digitext_pdf::FM1_MIN_BIT_DEPTH) {
            v.push(("PDF-KHONG-MAU-24BIT", format!("ảnh {b} bit < 24 bit màu (TT05 Điều 8.2.a)")));
        }
        if !self.signed {
            v.push(("PDF-CHUA-KY-SO", "tệp chưa có chữ ký số (TT05 Điều 8.2.a)".to_string()));
        }
        v
    }
}
pub fn inspect_pdf(bytes: &[u8]) -> PdfFacts {
    let mut f = PdfFacts::default();
    if !bytes.starts_with(b"%PDF") {
        f.encrypted_or_broken = Some("không phải tệp PDF (thiếu đầu %PDF)".into());
        return f;
    }
    f.pdfa = contains(bytes, b"pdfaid:part");
    f.signed = contains(bytes, b"/ByteRange");
    let r = digitext_pdf::f_2p9(bytes, None);
    f.pages = r.pages as u32;
    f.text_pages = r.pages_with_text as u32;
    f.min_dpi = r.min_dpi;
    f.min_bit_depth = r.min_bit_depth;
    if r.pages == 0 {
        f.encrypted_or_broken = Some(r.reason.unwrap_or_else(|| "không đọc được trang nào".into()));
    }
    f
}

#[derive(Default)]
struct Deep {
    pdfa: u32,
    signed: u32,
    pages: u32,
}

fn check_dossier(d: &Dossier, root: &Path, disk: &HashMap<String, (String, u64)>, deep: bool, tt05_strict: bool, sink: &mut Sink) -> (u64, Option<Deep>) {
    let first = d.first();
    let hs = d.hs_code();
    let dir = d.dir.as_str();
    // ---- trường bắt buộc cấp hồ sơ
    for (name, v) in [("Mã phông", &first.fond_code), ("Mã hồ sơ", &first.hs_code), ("Tiêu đề hồ sơ", &first.hs_title), ("Thời hạn lưu trữ", &first.hs_thbq)] {
        if v.is_none() {
            sink.error("HS-THIEU-TRUONG", dir, &hs, Some(first.line), None, format!("hồ sơ thiếu trường bắt buộc «{name}»"));
        }
    }
    match first.hs_papers.as_deref().and_then(lenient_u32) {
        None => sink.error("HS-THIEU-SO-TO", dir, &hs, Some(first.line), None,
            format!("hồ sơ số hoá bắt buộc «Số lượng tờ» là số — đang là «{}»", first.hs_papers.as_deref().unwrap_or(""))),
        Some((n, true)) => sink.warn("HS-SO-TO-CO-RAC", dir, &hs, Some(first.line), None,
            format!("«Số lượng tờ» = «{}» lẫn ký tự thừa — hiểu là {n}", first.hs_papers.as_deref().unwrap_or(""))),
        Some(_) => {}
    }
    // ---- các dòng của một hồ sơ phải thống nhất thông tin cấp hồ sơ
    let pick: [(&str, fn(&Row) -> &Option<String>); 8] = [
        ("Mã hồ sơ", |r| &r.hs_code), ("Mã phông", |r| &r.fond_code), ("Tiêu đề hồ sơ", |r| &r.hs_title),
        ("Thời hạn lưu trữ", |r| &r.hs_thbq), ("Thời gian bắt đầu", |r| &r.hs_start),
        ("Thời gian kết thúc", |r| &r.hs_end), ("Số lượng tờ", |r| &r.hs_papers), ("Mã hồ sơ gốc giấy", |r| &r.hs_paper_code),
    ];
    for (name, get) in pick {
        let vals: BTreeSet<&str> = d.rows.iter().map(|r| get(r).as_deref().unwrap_or("")).collect();
        if vals.len() > 1 {
            let list = vals.iter().take(3).map(|v| format!("«{v}»")).collect::<Vec<_>>().join(", ");
            let line = d.rows.iter().find(|r| get(r) != get(first)).map(|r| r.line);
            if name == "Mã hồ sơ" || name == "Mã phông" {
                sink.error("HS-NHIEU-MA", dir, &hs, line, None, format!("một thư mục hồ sơ mà Excel ghi {} {name} khác nhau: {list}", vals.len()));
            } else {
                sink.warn("HS-KHONG-THONG-NHAT", dir, &hs, line, None, format!("các dòng cùng hồ sơ ghi «{name}» khác nhau: {list}"));
            }
        }
    }
    if let (Some(fc), Some(code)) = (&first.fond_code, &first.hs_code) {
        if !code.starts_with(&format!("{fc}.")) {
            sink.error("HS-MA-LECH-PHONG", dir, &hs, Some(first.line), None, format!("mã hồ sơ «{code}» không bắt đầu bằng mã phông «{fc}»"));
        }
    }
    for (name, v) in [("Thời gian bắt đầu", &first.hs_start), ("Thời gian kết thúc", &first.hs_end)] {
        if let Some(s) = v {
            if !norm_date(s).1 {
                sink.warn("HS-NGAY-SAI-DANG", dir, &hs, Some(first.line), None, format!("«{name}» = «{s}» không đúng dạng dd/mm/yyyy"));
            }
        }
    }
    let (ds, de) = (first.hs_start.as_deref().and_then(date_key), first.hs_end.as_deref().and_then(date_key));
    if let (Some(a), Some(b)) = (ds, de) {
        if a > b {
            sink.warn("HS-NGAY-NGUOC", dir, &hs, Some(first.line), None, "thời gian bắt đầu sau thời gian kết thúc".into());
        }
    }
    // ---- thư mục ↔ mã hồ sơ: …\HOPnn\<số hồ sơ>
    let code_parts: Vec<&str> = hs.split('.').collect();
    let segs: Vec<&str> = dir.split('\\').collect();
    let code_hs_no = code_parts.last().and_then(|s| s.parse::<u32>().ok());
    if let (Some(last), Some(no)) = (segs.last(), code_hs_no) {
        if let Ok(dno) = last.parse::<u32>() {
            if dno != no {
                sink.error("TM-LECH-SO-HO-SO", dir, &hs, Some(first.line), None, format!("thư mục hồ sơ số {dno} nhưng mã hồ sơ «{hs}» kết thúc bằng {no}"));
            }
        }
    }
    // ---- từng tài liệu
    let mut bytes_total = 0u64;
    let mut seqs: BTreeMap<u32, u32> = BTreeMap::new();
    let mut deep_acc = Deep::default();
    let mut sum_pages_xl = 0u32;
    for r in &d.rows {
        let Some(path) = r.doc_path.as_deref() else {
            sink.error("TL-THIEU-DUONG-DAN", dir, &hs, Some(r.line), None, "dòng không có «Tệp tin tài liệu»".into());
            continue;
        };
        let rel = norm_rel(path);
        let name = base_of(&rel).to_string();
        if let (Some(hd), true) = (r.hs_dir.as_deref(), true) {
            if norm_rel(hd).to_lowercase() != dir.to_lowercase() {
                sink.error("TL-LECH-THU-MUC", dir, &hs, Some(r.line), Some(&name), format!("«Tệp tin hồ sơ» = «{hd}» khác thư mục thật của tệp"));
            }
        }
        if let Some(fc) = r.doc_file_code.as_deref() {
            if !fc.eq_ignore_ascii_case(&name) {
                sink.error("TL-MA-LECH-TEN-TEP", dir, &hs, Some(r.line), Some(&name), format!("«Mã lưu trữ tài liệu» = «{fc}» khác tên tệp thật «{name}»"));
            }
        }
        match seq_of(&name) {
            Some(s) => *seqs.entry(s).or_default() += 1,
            None => sink.warn("TL-TEN-KHONG-CO-STT", dir, &hs, Some(r.line), Some(&name), format!("tên tệp «{name}» không kết thúc bằng số thứ tự 7 chữ số — sẽ đánh số theo thứ tự dòng")),
        }
        // năm + số hồ sơ trong tên tệp so với mã hồ sơ
        let stem_parts: Vec<&str> = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(&name).split('.').collect();
        if stem_parts.len() >= 3 && code_parts.len() >= 2 {
            let f_hs = stem_parts[stem_parts.len() - 2].parse::<u32>().ok();
            if let (Some(a), Some(b)) = (f_hs, code_hs_no) {
                if a != b {
                    sink.error("TL-TEN-LECH-SO-HO-SO", dir, &hs, Some(r.line), Some(&name), format!("tên tệp mang số hồ sơ {a}, mã hồ sơ là {b}"));
                }
            }
            // So với phần mã phông NẰM TRONG mã hồ sơ (3 đoạn đầu) — cột «Mã phông» lệch mã hồ
            // sơ đã có lỗi riêng HS-MA-LECH-PHONG, không báo lặp ở từng tệp.
            let hs_fond = code_parts.iter().take(3).copied().collect::<Vec<_>>().join(".");
            if code_parts.len() >= 4 {
                let fc = hs_fond.as_str();
                if !name.to_lowercase().starts_with(&format!("{}.", fc.to_lowercase())) {
                    sink.error("TL-TEN-LECH-PHONG", dir, &hs, Some(r.line), Some(&name), format!("tên tệp «{name}» không bắt đầu bằng mã phông «{fc}» của mã hồ sơ"));
                }
            }
            let f_year = stem_parts[stem_parts.len() - 3];
            if f_year.len() == 4 && f_year.bytes().all(|b| b.is_ascii_digit()) && !code_parts.contains(&f_year) {
                sink.error("TL-TEN-LECH-NAM", dir, &hs, Some(r.line), Some(&name), format!("tên tệp mang năm {f_year}, không có trong mã hồ sơ «{hs}»"));
            }
        }
        if r.doc_subject.is_none() {
            sink.warn("TL-THIEU-TRICH-YEU", dir, &hs, Some(r.line), Some(&name), "tài liệu không có trích yếu".into());
        }
        if r.doc_type.is_none() {
            sink.warn("TL-THIEU-TEN-LOAI", dir, &hs, Some(r.line), Some(&name), "tài liệu không có tên loại — sẽ ghi «Khác»".into());
        }
        if let (Some(a), Some(b)) = (r.doc_thbq.as_deref(), first.hs_thbq.as_deref()) {
            if crate::excel::fold(a) != crate::excel::fold(b) {
                sink.warn("TL-THBQ-KHAC-HO-SO", dir, &hs, Some(r.line), Some(&name), format!("thời hạn lưu trữ của tài liệu «{a}» khác của hồ sơ «{b}» — gói tin ghi theo hồ sơ"));
            }
        }
        match r.doc_date.as_deref() {
            None => sink.warn("TL-THIEU-NGAY", dir, &hs, Some(r.line), Some(&name), "tài liệu không có ngày tháng ban hành".into()),
            Some(s) => match date_key(s) {
                None => sink.warn("TL-NGAY-SAI-DANG", dir, &hs, Some(r.line), Some(&name), format!("ngày ban hành «{s}» không đúng dạng dd/mm/yyyy")),
                Some(k) => {
                    if let (Some(a), Some(b)) = (ds, de) {
                        if k < a || k > b {
                            sink.warn("TL-NGAY-NGOAI-HO-SO", dir, &hs, Some(r.line), Some(&name), format!("ngày ban hành «{s}» nằm ngoài khoảng thời gian của hồ sơ"));
                        }
                    }
                }
            },
        }
        let pages_xl = r.doc_pages.as_deref().and_then(|s| s.parse::<u32>().ok());
        if pages_xl.is_none() {
            sink.warn("TL-THIEU-SO-TRANG", dir, &hs, Some(r.line), Some(&name), "«Số lượng trang» của tài liệu trống hoặc không phải số".into());
        }
        sum_pages_xl += pages_xl.unwrap_or(0);
        // ---- tệp trên đĩa
        let Some((real_rel, size)) = disk.get(&rel.to_lowercase()) else {
            sink.error("TEP-KHONG-CO-TREN-DIA", dir, &hs, Some(r.line), Some(&name), format!("Excel có nhưng không thấy tệp: {rel}"));
            continue;
        };
        bytes_total += size;
        if *size == 0 {
            sink.error("TEP-RONG", dir, &hs, Some(r.line), Some(&name), "tệp 0 byte".into());
            continue;
        }
        if deep {
            match std::fs::read(rel_to_path(root, real_rel)) {
                Err(e) => sink.error("TEP-KHONG-DOC-DUOC", dir, &hs, Some(r.line), Some(&name), format!("không đọc được tệp: {e}")),
                Ok(bytes) => {
                    let f = inspect_pdf(&bytes);
                    if let Some(why) = &f.encrypted_or_broken {
                        sink.error("PDF-HONG", dir, &hs, Some(r.line), Some(&name), format!("PDF không dùng được: {why}"));
                        continue;
                    }
                    deep_acc.pages += f.pages;
                    deep_acc.pdfa += f.pdfa as u32;
                    deep_acc.signed += f.signed as u32;
                    if let Some(p) = pages_xl {
                        if p != f.pages {
                            sink.error("PDF-LECH-SO-TRANG", dir, &hs, Some(r.line), Some(&name), format!("Excel ghi {p} trang, PDF thật có {} trang", f.pages));
                        }
                    }
                    for (code, msg) in f.tt05_failures() {
                        if tt05_strict {
                            sink.error(code, dir, &hs, Some(r.line), Some(&name), msg);
                        } else {
                            sink.warn(code, dir, &hs, Some(r.line), Some(&name), msg);
                        }
                    }
                }
            }
        }
    }
    for (s, n) in &seqs {
        if *n > 1 {
            sink.error("TL-TRUNG-STT", dir, &hs, None, None, format!("{n} tệp trong hồ sơ cùng mang số thứ tự {s:07}"));
        }
    }
    if let (Some(min), Some(max)) = (seqs.keys().next(), seqs.keys().last()) {
        let missing = (max - min + 1) as usize - seqs.len();
        if *min != 1 || missing > 0 {
            sink.warn("TL-STT-KHONG-LIEN-TUC", dir, &hs, None, None, format!("số thứ tự tệp chạy {min:07}…{max:07}, thiếu {missing} số (hoặc không bắt đầu từ 1)"));
        }
    }
    if let Some(t) = first.hs_total_doc.as_deref().and_then(|s| s.parse::<usize>().ok()) {
        if t != d.rows.len() {
            sink.error("HS-LECH-TONG-TAI-LIEU", dir, &hs, Some(first.line), None, format!("«Tổng số tài liệu trong hồ sơ» = {t}, Excel liệt kê {} tài liệu", d.rows.len()));
        }
    }
    if let Some(p) = first.hs_pages.as_deref().and_then(|s| s.parse::<u32>().ok()) {
        if p != sum_pages_xl {
            sink.warn("HS-LECH-TONG-TRANG", dir, &hs, Some(first.line), None, format!("«Số lượng trang» hồ sơ = {p}, tổng trang các tài liệu = {sum_pages_xl}"));
        }
    }
    (bytes_total, deep.then_some(deep_acc))
}

/// Chạy toàn bộ bước kiểm. `progress(đã xong, tổng)` được gọi sau mỗi hồ sơ.
pub fn run(
    sheet: &Sheet,
    excel_path: &Path,
    plan: &Plan,
    deep: bool,
    tt05_strict: bool,
    threads: usize,
    progress: &(dyn Fn(usize, usize) + Sync),
) -> CheckReport {
    let mut sink = Sink::new();
    // ---- liệt kê đĩa: chỉ quét các thư mục cấp đầu mà Excel nhắc tới
    let mut tops: BTreeSet<String> = BTreeSet::new();
    for d in &plan.dossiers {
        if let Some(t) = d.dir.split('\\').next() {
            if !t.is_empty() && !t.starts_with('?') {
                tops.insert(t.to_string());
            }
        }
    }
    let mut disk: HashMap<String, (String, u64)> = HashMap::new();
    for t in &tops {
        let p = plan.root.join(t);
        if !p.is_dir() {
            sink.error("THU-MUC-GOC-KHONG-CO", "", "", None, None, format!("không thấy thư mục «{}» dưới {}", t, plan.root.display()));
        }
        walk_pdfs(&p, &plan.root, &mut disk);
    }
    // ---- đường dẫn trùng trong Excel + tệp trên đĩa không có trong Excel
    let mut seen: HashMap<String, u32> = HashMap::new();
    for d in &plan.dossiers {
        for r in &d.rows {
            if let Some(p) = r.doc_path.as_deref() {
                let k = norm_rel(p).to_lowercase();
                if let Some(prev) = seen.insert(k, r.line) {
                    sink.error("EXCEL-TRUNG-TEP", &d.dir, &d.hs_code(), Some(r.line), Some(base_of(&norm_rel(p))), format!("cùng một tệp được khai ở hai dòng Excel ({prev} và {})", r.line));
                }
            }
        }
    }
    let dir_codes: HashMap<String, String> = plan.dossiers.iter().map(|d| (d.dir.to_lowercase(), d.hs_code())).collect();
    let mut extra: Vec<&(String, u64)> = disk.iter().filter(|(k, _)| !seen.contains_key(*k)).map(|(_, v)| v).collect();
    extra.sort();
    for (rel, _) in extra {
        let dir = dir_of(rel);
        let hs = dir_codes.get(&dir.to_lowercase()).cloned().unwrap_or_default();
        sink.error("TEP-KHONG-CO-TRONG-EXCEL", &dir, &hs, None, Some(base_of(rel)), format!("tệp có trên đĩa nhưng Excel không khai: {rel}"));
    }
    // ---- mã hồ sơ trùng giữa các thư mục
    let mut by_code: BTreeMap<String, Vec<&Dossier>> = BTreeMap::new();
    for d in &plan.dossiers {
        let c = d.hs_code();
        if !c.is_empty() {
            by_code.entry(c).or_default().push(d);
        }
    }
    for (code, ds) in &by_code {
        if ds.len() > 1 {
            let dirs = ds.iter().map(|d| d.dir.as_str()).collect::<Vec<_>>().join(" | ");
            for d in ds {
                sink.error("HS-TRUNG-MA", &d.dir, code, Some(d.first().line), None, format!("mã hồ sơ «{code}» dùng cho {} thư mục hồ sơ khác nhau: {dirs}", ds.len()));
            }
        }
    }
    // ---- từng hồ sơ (song song khi kiểm sâu: đọc tệp là phần chậm)
    let total = plan.dossiers.len();
    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, u64, Option<Deep>, Sink)>> = Mutex::new(Vec::with_capacity(total));
    let workers = if deep { threads.max(1) } else { 1 };
    std::thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= total {
                    break;
                }
                let mut local = Sink::new();
                let (bytes, dp) = check_dossier(&plan.dossiers[i], &plan.root, &disk, deep, tt05_strict, &mut local);
                results.lock().unwrap().push((i, bytes, dp, local));
                progress(done.fetch_add(1, Ordering::Relaxed) + 1, total);
            });
        }
    });
    let mut results = results.into_inner().unwrap();
    results.sort_by_key(|r| r.0);
    let mut per: Vec<(u64, Option<Deep>)> = Vec::with_capacity(total);
    for (_, bytes, dp, local) in results {
        sink.merge(local);
        per.push((bytes, dp));
    }
    let mut issues = sink.errors;
    issues.extend(sink.warns.into_values());
    issues.sort_by(|a, b| (a.severity, &a.dossier, a.code, a.line).cmp(&(b.severity, &b.dossier, b.code, b.line)));
    let mut count: HashMap<&str, (u32, u32)> = HashMap::new();
    for i in &issues {
        let e = count.entry(i.dossier.as_str()).or_default();
        match i.severity {
            Severity::Error => e.0 += i.count,
            Severity::Warning => e.1 += i.count,
        }
    }
    let dossiers = plan
        .dossiers
        .iter()
        .zip(per)
        .map(|(d, (bytes, dp))| {
            let (errors, warnings) = count.get(d.dir.as_str()).copied().unwrap_or((0, 0));
            DossierSummary {
                dir: d.dir.clone(),
                hs_code: d.hs_code(),
                title: d.first().hs_title.clone().unwrap_or_default(),
                fond_code: d.first().fond_code.clone().unwrap_or_default(),
                docs: d.rows.len(),
                bytes,
                errors,
                warnings,
                pdfa: dp.as_ref().map(|x| x.pdfa),
                signed: dp.as_ref().map(|x| x.signed),
                pages: dp.as_ref().map(|x| x.pages),
            }
        })
        .collect();
    CheckReport {
        excel: excel_path.display().to_string(),
        sheet: sheet.sheet_name.clone(),
        header_line: sheet.header_line,
        root: plan.root.display().to_string(),
        deep,
        rows: sheet.rows.len(),
        files_on_disk: disk.len(),
        dossiers,
        issues,
        column_mapping: sheet.mapping.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chuan_hoa_ngay() {
        assert_eq!(norm_date("9/7/2012"), ("09/07/2012".into(), true));
        assert_eq!(norm_date("2017-11-01"), ("01/11/2017".into(), true));
        assert_eq!(norm_date("12/14/20").1, false);
        assert_eq!(norm_date("2021").1, false);
        assert_eq!(norm_date("01/01/2017 - 31/12/2017").1, false);
    }
    #[test]
    fn so_thu_tu_tu_ten_tep() {
        assert_eq!(seq_of("H49.132.02.2009.001.0000012.pdf"), Some(12));
        assert_eq!(seq_of("bia.pdf"), None);
        assert_eq!(seq_of("H49.132.01.2019.39.0000BIA.pdf"), None);
    }
}
