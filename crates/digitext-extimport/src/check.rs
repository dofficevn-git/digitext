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
    /// Không phải vấn đề — ghi để người dùng soát một quyết định tự động (vd xếp loại bản đồ).
    Info,
}
impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "LỖI",
            Severity::Warning => "CẢNH BÁO",
            Severity::Info => "THÔNG TIN",
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
    pub maps: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    pub excel: String,
    pub sheet: String,
    pub header_line: u32,
    pub root: String,
    pub deep: bool,
    /// Đã xác minh ISO bằng veraPDF.
    pub verapdf: bool,
    pub rows: usize,
    pub files_on_disk: usize,
    pub dossiers: Vec<DossierSummary>,
    pub issues: Vec<Issue>,
    pub column_mapping: Vec<(String, String)>,
    /// Chỉ có khi kiểm sâu: số tệp theo loại PDF/A tự khai.
    pub pdfa_flavours: BTreeMap<String, u32>,
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
    parse_doc_name(file_name).stt
}

/// Tên tệp / mã văn bản tách theo quy tắc
/// `<mã cơ quan>.<phông số>.<năm>.<số hồ sơ>[.<mục lục số>].<STT 7 số>`:
/// `H49.101.01.1993.153.01.0000002.pdf` → năm 1993, hồ sơ 153, mục lục "01", STT 2.
/// Mốc nhận diện là đoạn NĂM (4 chữ số) — mã cơ quan có thể 1 hay 2 đoạn (H49 / H49.101).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocName {
    pub year: Option<String>,
    pub hs_no: Option<u32>,
    pub muc_luc: Option<String>,
    pub stt: Option<u32>,
}
fn is_year(s: &str) -> bool {
    s.len() == 4 && s.bytes().all(|b| b.is_ascii_digit()) && (1800..=2100).contains(&s.parse::<u32>().unwrap_or(0))
}
pub fn parse_doc_name(file_name: &str) -> DocName {
    let stem = match file_name.rsplit_once('.') {
        Some((s, ext)) if ext.len() <= 4 && !ext.bytes().all(|b| b.is_ascii_digit()) => s,
        _ => file_name,
    };
    let mut parts: Vec<&str> = stem.split('.').collect();
    let mut out = DocName::default();
    if let Some(last) = parts.last() {
        if last.len() == 7 && last.bytes().all(|b| b.is_ascii_digit()) {
            out.stt = last.parse().ok();
            parts.pop();
        }
    }
    let Some(y) = parts.iter().position(|p| is_year(p)) else { return out };
    out.year = Some(parts[y].to_string());
    out.hs_no = parts.get(y + 1).and_then(|p| p.parse().ok());
    let rest = &parts[(y + 2).min(parts.len())..];
    if !rest.is_empty() && rest.iter().all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit())) {
        out.muc_luc = Some(rest.join("."));
    }
    out
}
/// Số hồ sơ trong mã hồ sơ: đoạn ngay sau năm (`H49.101.01.1993.153` → 153, kể cả khi mã còn
/// đuôi mục lục `…153.01`); mã không có năm thì lấy đoạn cuối.
pub fn hs_no_of_code(code: &str) -> Option<u32> {
    let d = parse_doc_name(code);
    if d.year.is_some() {
        return d.hs_no;
    }
    code.rsplit('.').next().and_then(|s| s.parse().ok())
}

/// Gom dòng Excel thành hồ sơ theo thư mục. `root` được tự chỉnh nếu người dùng trỏ vào
/// thư mục con trùng với đoạn đầu của đường dẫn trong Excel.
pub fn plan(sheet: &Sheet, root: &Path) -> Plan {
    let mut root = root.to_path_buf();
    if let Some(first) = sheet.rows.iter().find_map(|r| r.doc_path.as_deref()) {
        let rel = norm_rel(first);
        if !rel_to_path(&root, &rel).exists() {
            // Người dùng hay trỏ vào thư mục CON (…\UBND\DCXD) trong khi Excel ghi đường dẫn từ
            // «UBND\DCXD\…»: lùi dần lên các cấp cha tới khi tìm thấy tệp đầu tiên.
            let mut cur = root.clone();
            while let Some(parent) = cur.parent() {
                if rel_to_path(parent, &rel).exists() {
                    root = parent.to_path_buf();
                    break;
                }
                cur = parent.to_path_buf();
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
        self.note(Severity::Warning, code, d, hs, line, file, msg)
    }
    fn info(&mut self, code: &'static str, d: &str, hs: &str, line: Option<u32>, file: Option<&str>, msg: String) {
        self.note(Severity::Info, code, d, hs, line, file, msg)
    }
    fn note(&mut self, severity: Severity, code: &'static str, d: &str, hs: &str, line: Option<u32>, file: Option<&str>, msg: String) {
        self.warns
            .entry((d.to_string(), code))
            .and_modify(|i| i.count += 1)
            .or_insert(Issue {
                severity, code, dossier: d.into(), hs_code: hs.into(), line,
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
    /// Tệp tự khai một loại PDF/A hợp lệ (bất kỳ phiên bản/mức nào) VÀ có nhúng hồ sơ màu.
    pub pdfa: bool,
    /// Loại PDF/A tệp tự khai ("PDF/A-2B"…), `None` = không khai.
    pub declared: Option<String>,
    pub has_output_intent: bool,
    pub signed: bool,
    /// Cạnh dài nhất của trang (cm). Trang > A3 (cạnh dài ≥ 45 cm) coi là bản đồ/bản vẽ.
    pub max_page_cm: Option<f32>,
    pub encrypted_or_broken: Option<String>,
}

/// Ngưỡng dpi TT05 Điều 8.2.a: 300 với bản đồ/bản vẽ, 200 với tài liệu hành chính.
pub const MAP_MIN_DPI: u32 = 300;
/// Trang có cạnh dài từ đây trở lên (cm) là khổ lớn hơn A3 → coi là bản đồ/bản vẽ.
pub const MAP_PAGE_CM: f32 = 45.0;

/// Kết quả xếp loại bản đồ/bản vẽ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapClass {
    /// Văn bản hành chính → ngưỡng 200 dpi.
    No,
    /// CHẮC CHẮN bản đồ/bản vẽ → ngưỡng 300 dpi. Chuỗi = lý do (để người dùng soát).
    Sure(String),
    /// Chỉ có dấu hiệu yếu (từ khoá trong trích yếu, trang ≤ A3) → VẪN 200 dpi, chỉ nhắc.
    Suspect(String),
}
impl MapClass {
    pub fn is_map(&self) -> bool {
        matches!(self, MapClass::Sure(_))
    }
}

/// Xếp loại bản đồ/bản vẽ — cố ý BẢO THỦ: văn bản hành chính bị áp nhầm 300 dpi còn tệ hơn
/// bản đồ bị áp 200 dpi.
///   CHẮC: cột «Tên loại tài liệu» ghi bản đồ/bản vẽ/sơ đồ/bản trích đo…, HOẶC khổ trang có
///         cạnh dài ≥ 45 cm (lớn hơn A3 — văn bản hành chính không có khổ này).
///   NGHI: chỉ trích yếu có từ khoá («sơ đồ», «tỉ lệ 1/…») mà tên loại không nói và trang ≤ A3.
pub fn classify_map(doc_type: Option<&str>, subject: Option<&str>, max_page_cm: Option<f32>) -> MapClass {
    const TYPE_KW: &[&str] = &["ban do", "ban ve", "so do", "ban trich do", "trich luc ban do", "binh do", "mat cat", "mat bang"];
    const SUBJ_KW: &[&str] = &["ban do", "ban ve", "so do", "mat bang", "ti le 1/", "ty le 1/", "trac dia", "trich do"];
    if let Some(c) = max_page_cm.filter(|c| *c >= MAP_PAGE_CM) {
        return MapClass::Sure(format!("khổ trang {c:.0} cm, lớn hơn A3"));
    }
    let t = crate::excel::fold(doc_type.unwrap_or(""));
    if let Some(k) = TYPE_KW.iter().find(|k| t.contains(*k)) {
        return MapClass::Sure(format!("tên loại tài liệu «{}» (khớp «{k}»)", doc_type.unwrap_or("").trim()));
    }
    let sj = crate::excel::fold(subject.unwrap_or(""));
    if let Some(k) = SUBJ_KW.iter().find(|k| sj.contains(*k)) {
        return MapClass::Suspect(format!("trích yếu có «{k}» nhưng tên loại là «{}» và khổ trang ≤ A3", doc_type.unwrap_or("").trim()));
    }
    MapClass::No
}
fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}
impl PdfFacts {
    /// Các điểm CHƯA ĐẠT Điều 8 khoản 2 điểm a TT 05/2025 (PDF/A hai lớp, màu ≥ 24 bit,
    /// ≥ 200 dpi, có chữ ký số). Trả (mã vấn đề, nội dung). Rỗng = đạt.
    /// TT05 không chỉ định phiên bản PDF/A → nhận MỌI loại ISO 19005 (1a/1b/2a/2b/2u/3a/3b/3u/4…).
    /// Ở đây kiểm dấu hiệu cấu trúc (khai pdfaid + hồ sơ màu); hợp chuẩn ISO đầy đủ do veraPDF
    /// kiểm theo đúng loại tệp tự khai (bật `verapdf` ở bước kiểm/đóng gói).
    pub fn tt05_failures(&self, is_map: bool) -> Vec<(&'static str, String)> {
        let mut v = Vec::new();
        match (&self.declared, crate::pdfa::is_known_flavour(&self.declared), self.has_output_intent) {
            (None, _, _) => v.push(("PDF-KHONG-PHAI-PDFA", "tệp KHÔNG phải PDF/A (không khai pdfaid trong XMP) — TT05 Điều 8.2.a đòi PDF/A (bất kỳ phiên bản 1/2/3/4, mức a/b/u); chuyển đổi sau khi đã ký sẽ phá chữ ký, phải ký lại".to_string())),
            (Some(d), false, _) => v.push(("PDF-LOAI-PDFA-LA", format!("tệp khai «{d}» không phải một loại PDF/A mà ISO 19005 định nghĩa"))),
            (Some(d), true, false) => v.push(("PDF-THIEU-HO-SO-MAU", format!("tệp khai {d} nhưng không nhúng hồ sơ màu (OutputIntent) — vi phạm ISO 19005 mục 6.2.4.3"))),
            _ => {}
        }
        if self.text_pages < self.pages {
            v.push(("PDF-THIEU-LOP-CHU", format!("{}/{} trang không có lớp chữ — TT05 Điều 8.2.a đòi PDF/A HAI LỚP", self.pages - self.text_pages, self.pages)));
        }
        let (need, loai) = if is_map { (MAP_MIN_DPI, "bản đồ/bản vẽ") } else { (digitext_pdf::FM1_MIN_DPI, "tài liệu hành chính") };
        if let Some(d) = self.min_dpi.filter(|d| *d < need) {
            let kho = self.max_page_cm.map(|c| format!(", khổ trang {c:.0} cm")).unwrap_or_default();
            v.push(("PDF-DPI-THAP", format!("độ phân giải {d} dpi < {need} dpi cho {loai} (TT05 Điều 8.2.a{kho})")));
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
/// Cạnh dài nhất (cm) trong các trang, đọc MediaBox (kế thừa từ nút cha nếu trang không khai).
fn max_page_cm(bytes: &[u8]) -> Option<f32> {
    let doc = lopdf::Document::load_mem(bytes).ok()?;
    let mut best: f32 = 0.0;
    for (_, id) in doc.get_pages() {
        let Ok(page) = doc.get_dictionary(id) else { continue };
        let mut cur = Some(page);
        let mut depth = 0;
        while let Some(d) = cur {
            if let Ok(mb) = d.get(b"MediaBox").and_then(|o| doc.dereference(o).map(|(_, o)| o)) {
                if let Ok(arr) = mb.as_array() {
                    let v: Vec<f32> = arr.iter().filter_map(|o| o.as_float().ok().or_else(|| o.as_i64().ok().map(|i| i as f32))).collect();
                    if v.len() == 4 {
                        let w = (v[2] - v[0]).abs() / 72.0 * 2.54;
                        let h = (v[3] - v[1]).abs() / 72.0 * 2.54;
                        best = best.max(w).max(h);
                    }
                }
                break;
            }
            depth += 1;
            cur = if depth > 32 { None } else { d.get(b"Parent").ok().and_then(|p| p.as_reference().ok()).and_then(|r| doc.get_dictionary(r).ok()) };
        }
    }
    (best > 0.0).then_some(best)
}

pub fn inspect_pdf(bytes: &[u8]) -> PdfFacts {
    let mut f = PdfFacts::default();
    if !bytes.starts_with(b"%PDF") {
        f.encrypted_or_broken = Some("không phải tệp PDF (thiếu đầu %PDF)".into());
        return f;
    }
    f.declared = crate::pdfa::declared_flavour(bytes);
    f.has_output_intent = contains(bytes, b"/OutputIntents");
    f.pdfa = crate::pdfa::is_known_flavour(&f.declared) && f.has_output_intent;
    f.signed = contains(bytes, b"/ByteRange");
    let r = digitext_pdf::f_2p9(bytes, None);
    f.pages = r.pages as u32;
    f.text_pages = r.pages_with_text as u32;
    f.min_dpi = r.min_dpi;
    f.max_page_cm = max_page_cm(bytes);
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
    /// Số tệp được coi là bản đồ/bản vẽ (ngưỡng 300 dpi).
    maps: u32,
    /// Đếm theo loại tệp tự khai; khoá "không khai PDF/A" cho tệp không có pdfaid.
    flavours: BTreeMap<String, u32>,
}

fn check_dossier(d: &Dossier, root: &Path, disk: &HashMap<String, (String, u64)>, deep: bool, tt05_strict: bool, verapdf: bool, sink: &mut Sink) -> (u64, Option<Deep>) {
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
    let code_hs_no = hs_no_of_code(&hs);
    if let (Some(last), Some(no)) = (segs.last(), code_hs_no) {
        if let Ok(dno) = last.parse::<u32>() {
            if dno != no {
                sink.error("TM-LECH-SO-HO-SO", dir, &hs, Some(first.line), None, format!("thư mục hồ sơ số {dno} nhưng mã hồ sơ «{hs}» mang số hồ sơ {no}"));
            }
        }
    }
    // ---- từng tài liệu
    let mut bytes_total = 0u64;
    let mut seqs: BTreeMap<u32, u32> = BTreeMap::new();
    let mut deep_acc = Deep::default();
    let mut sum_pages_xl = 0u32;
    // Tệp qua kiểm cấu trúc → đem xác minh ISO bằng veraPDF theo đúng loại tự khai (nếu bật).
    let mut to_verify: Vec<(PathBuf, Option<String>, u32, String)> = Vec::new();
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
            // Excel có nơi ghi mã KHÔNG đuôi (.pdf) — so cả tên tệp lẫn phần trước đuôi.
            let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(&name);
            let fc = fc.trim();
            if !fc.eq_ignore_ascii_case(&name) && !fc.eq_ignore_ascii_case(stem) {
                sink.error("TL-MA-LECH-TEN-TEP", dir, &hs, Some(r.line), Some(&name), format!("«Mã lưu trữ tài liệu» = «{fc}» khác tên tệp thật «{name}»"));
            }
        }
        match seq_of(&name) {
            Some(s) => *seqs.entry(s).or_default() += 1,
            None => sink.warn("TL-TEN-KHONG-CO-STT", dir, &hs, Some(r.line), Some(&name), format!("tên tệp «{name}» không kết thúc bằng số thứ tự 7 chữ số — sẽ đánh số theo thứ tự dòng")),
        }
        // năm + số hồ sơ trong tên tệp so với mã hồ sơ — tách theo quy tắc
        // <mã cơ quan>.<phông>.<năm>.<số hồ sơ>[.<mục lục>].<STT>, mốc là đoạn năm.
        let stem_parts: Vec<&str> = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(&name).split('.').collect();
        let dn = parse_doc_name(&name);
        if stem_parts.len() >= 3 && code_parts.len() >= 2 {
            if let (Some(a), Some(b)) = (dn.hs_no, code_hs_no) {
                if a != b {
                    sink.error("TL-TEN-LECH-SO-HO-SO", dir, &hs, Some(r.line), Some(&name), format!("tên tệp mang số hồ sơ {a}, mã hồ sơ là {b}"));
                }
            }
            // Mã hồ sơ đã có đuôi mục lục thì tên tệp phải cùng mục lục.
            if let (Some(ml_code), Some(ml_name)) = (parse_doc_name(&hs).muc_luc.as_deref(), dn.muc_luc.as_deref()) {
                if ml_code != ml_name {
                    sink.error("TL-TEN-LECH-MUC-LUC", dir, &hs, Some(r.line), Some(&name), format!("tên tệp mang mục lục số {ml_name}, mã hồ sơ ghi mục lục {ml_code}"));
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
            if let Some(f_year) = dn.year.as_deref() {
                if !code_parts.contains(&f_year) {
                    sink.error("TL-TEN-LECH-NAM", dir, &hs, Some(r.line), Some(&name), format!("tên tệp mang năm {f_year}, không có trong mã hồ sơ «{hs}»"));
                }
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
                    *deep_acc.flavours.entry(f.declared.clone().unwrap_or_else(|| "không khai PDF/A".into())).or_default() += 1;
                    deep_acc.signed += f.signed as u32;
                    if let Some(p) = pages_xl {
                        if p != f.pages {
                            sink.error("PDF-LECH-SO-TRANG", dir, &hs, Some(r.line), Some(&name), format!("Excel ghi {p} trang, PDF thật có {} trang", f.pages));
                        }
                    }
                    let mc = classify_map(r.doc_type.as_deref(), r.doc_subject.as_deref(), f.max_page_cm);
                    match &mc {
                        MapClass::Sure(why) => {
                            deep_acc.maps += 1;
                            sink.info("TL-XEP-BAN-DO", dir, &hs, Some(r.line), Some(&name), format!("tệp được xếp là BẢN ĐỒ/BẢN VẼ → áp 300 dpi. Lý do: {why}. Nếu không phải, sửa «Tên loại tài liệu» trong Excel"));
                        }
                        MapClass::Suspect(why) => sink.warn("TL-NGHI-BAN-DO", dir, &hs, Some(r.line), Some(&name), format!("có thể là bản đồ/bản vẽ ({why}) — đang áp 200 dpi như văn bản hành chính; nếu đúng là bản đồ, ghi «Tên loại tài liệu» = «Bản đồ» / «Bản vẽ» để áp 300 dpi")),
                        MapClass::No => {}
                    }
                    let fails = f.tt05_failures(mc.is_map());
                    if verapdf && f.pdfa {
                        to_verify.push((rel_to_path(root, real_rel), f.declared.clone(), r.line, name.clone()));
                    }
                    for (code, msg) in fails {
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
    if !to_verify.is_empty() {
        let res = crate::pdfa::verify_files(&to_verify.iter().map(|(p, d, _, _)| (p.clone(), d.clone())).collect::<Vec<_>>());
        for (p, declared, line, name) in &to_verify {
            match res.get(p) {
                Some(v) if v.compliant => {}
                Some(v) => {
                    let msg = format!("veraPDF: KHÔNG ĐẠT {} — {}", declared.clone().unwrap_or_default(), v.rules.iter().take(3).cloned().collect::<Vec<_>>().join(" | "));
                    if tt05_strict { sink.error("PDF-KHONG-DAT-ISO", dir, &hs, Some(*line), Some(name), msg) } else { sink.warn("PDF-KHONG-DAT-ISO", dir, &hs, Some(*line), Some(name), msg) }
                }
                None => sink.warn("PDF-VERAPDF-KHONG-CHAY", dir, &hs, Some(*line), Some(name), "veraPDF không trả kết quả cho tệp này (Docker?) — chỉ mới kiểm dấu hiệu cấu trúc".into()),
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
    verapdf: bool,
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
            // Gợi ý thư mục đúng: cấp cha nào của thư mục đã chọn có tên trùng «t» thì gốc là cha của nó.
            let hint = plan
                .root
                .ancestors()
                .find(|a| a.file_name().map_or(false, |n| n.to_string_lossy().eq_ignore_ascii_case(t)))
                .and_then(|a| a.parent().map(|g| format!(" — Excel ghi đường dẫn từ «{t}\\…», hãy chọn thư mục chứa «{t}»: {}", g.display())))
                .unwrap_or_default();
            sink.error("THU-MUC-GOC-KHONG-CO", "", "", None, None, format!("không thấy thư mục «{}» dưới {}{hint}", t, plan.root.display()));
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
                let (bytes, dp) = check_dossier(&plan.dossiers[i], &plan.root, &disk, deep, tt05_strict, verapdf && deep, &mut local);
                results.lock().unwrap().push((i, bytes, dp, local));
                progress(done.fetch_add(1, Ordering::Relaxed) + 1, total);
            });
        }
    });
    let mut results = results.into_inner().unwrap();
    results.sort_by_key(|r| r.0);
    let mut per: Vec<(u64, Option<Deep>)> = Vec::with_capacity(total);
    let mut pdfa_flavours: BTreeMap<String, u32> = BTreeMap::new();
    for (_, bytes, dp, local) in results {
        if let Some(d) = &dp {
            for (k, v) in &d.flavours {
                *pdfa_flavours.entry(k.clone()).or_default() += v;
            }
        }
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
            Severity::Info => {}
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
                maps: dp.as_ref().map(|x| x.maps),
            }
        })
        .collect();
    CheckReport {
        excel: excel_path.display().to_string(),
        sheet: sheet.sheet_name.clone(),
        header_line: sheet.header_line,
        root: plan.root.display().to_string(),
        deep,
        verapdf: verapdf && deep,
        rows: sheet.rows.len(),
        files_on_disk: disk.len(),
        dossiers,
        issues,
        column_mapping: sheet.mapping.clone(),
        pdfa_flavours,
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
    fn tach_ten_tep_theo_quy_tac_co_muc_luc() {
        // Có mục lục số: …<năm>.<số hồ sơ>.<mục lục>.<STT>
        let d = parse_doc_name("H49.101.01.1993.153.01.0000002.pdf");
        assert_eq!(d, DocName { year: Some("1993".into()), hs_no: Some(153), muc_luc: Some("01".into()), stt: Some(2) });
        // Không mục lục
        let d = parse_doc_name("H49.139.01.2011.599.0000001.pdf");
        assert_eq!(d, DocName { year: Some("2011".into()), hs_no: Some(599), muc_luc: None, stt: Some(1) });
        // Mã cơ quan một đoạn
        assert_eq!(parse_doc_name("H49.02.2009.001.0000012.pdf").hs_no, Some(1));
        // Số hồ sơ trong mã hồ sơ (có / không đuôi mục lục)
        assert_eq!(hs_no_of_code("H49.101.01.1993.153"), Some(153));
        assert_eq!(hs_no_of_code("H49.101.01.1993.153.01"), Some(153));
        assert_eq!(hs_no_of_code("ABC.153"), Some(153));
        assert_eq!(seq_of("MLVB.pdf"), None);
    }
    #[test]
    fn ma_luu_tru_khong_duoi_van_khop_ten_tep() {
        let name = "H49.139.01.2011.599.0000001.pdf";
        let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap();
        for fc in ["H49.139.01.2011.599.0000001", "H49.139.01.2011.599.0000001.pdf", "h49.139.01.2011.599.0000001.PDF "] {
            let fc = fc.trim();
            assert!(fc.eq_ignore_ascii_case(name) || fc.eq_ignore_ascii_case(stem), "{fc}");
        }
        assert!(!"H49.139.01.2011.600.0000001".eq_ignore_ascii_case(stem));
    }
    #[test]
    fn nhan_dien_ban_do() {
        // Chỉ trích yếu gợi ý, trang A4 → NGHI, vẫn 200 dpi.
        assert!(matches!(classify_map(Some("Khác"), Some("Quy hoạch chi tiết xây dựng tỉ lệ 1/500 - Khu 2"), Some(29.7)), MapClass::Suspect(_)));
        // Quyết định phê duyệt quy hoạch, A4 → văn bản hành chính.
        assert_eq!(classify_map(Some("Quyết định"), Some("Phê duyệt quy hoạch chi tiết khu dân cư"), Some(29.7)), MapClass::No);
        // Tên loại nói rõ → CHẮC.
        assert!(classify_map(Some("Bản đồ"), None, None).is_map());
        assert!(classify_map(Some("Bản vẽ kỹ thuật"), Some("Hồ sơ nhà văn hoá"), Some(29.7)).is_map());
        // Khổ lớn → CHẮC, bất kể Excel ghi gì.
        assert!(classify_map(Some("Khác"), None, Some(161.0)).is_map());
        // A3 (42 cm) chưa phải khổ lớn.
        assert_eq!(classify_map(Some("Quyết định"), Some("Về việc cấp đất"), Some(42.0)), MapClass::No);
        // Từ khoá gợi ý bên trong từ khác: «sơ đồ tổ chức» trong quyết định → chỉ NGHI.
        assert!(matches!(classify_map(Some("Quyết định"), Some("Ban hành sơ đồ tổ chức bộ máy"), Some(29.7)), MapClass::Suspect(_)));
        let f = PdfFacts { min_dpi: Some(250), pages: 1, text_pages: 1, pdfa: true, declared: Some("PDF/A-2B".into()),
            has_output_intent: true, signed: true, ..PdfFacts::default() };
        assert!(f.tt05_failures(false).is_empty());
        assert_eq!(f.tt05_failures(true).len(), 1);
    }
    #[test]
    fn so_thu_tu_tu_ten_tep() {
        assert_eq!(seq_of("H49.132.02.2009.001.0000012.pdf"), Some(12));
        assert_eq!(seq_of("bia.pdf"), None);
        assert_eq!(seq_of("H49.132.01.2019.39.0000BIA.pdf"), None);
    }
}
