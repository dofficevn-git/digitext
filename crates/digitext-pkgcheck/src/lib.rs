// Copyright (c) 2026 DigiText. All rights reserved.
//! Kiểm tra gói tin SIP/AIP/DIP của phần mềm khác theo TT 05/2025/TT-BNV + E-ARK CSIP. Cùng các bước với
//! `ml/tools/check_tt05_package.py` (bản kiểm gói TKT 19/9 và 9/10/2026), cộng xác minh chữ ký số.
//! XSD: không có bộ kiểm XSD trong Rust — đối chiếu RÚT GỌN với schema đi kèm gói (đủ trường bắt buộc,
//! không có trường lạ), chưa kiểm kiểu dữ liệu từng trường.
pub mod dot;
mod huong_dan;
mod load;
pub mod report;
mod xml;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use xml::El;

pub use huong_dan::{huong_dan, HuongDan};
pub use load::{discover, tu_ban_ke, Loaded};

/// Trần số mục liệt kê trong [`Finding::doi_tuong`]. Một gói sai quy ước đặt tên có thể cho
/// hàng nghìn tệp cùng một mã — gửi hết thì thân phản hồi phình ra mà không ai đọc tới mục thứ
/// hai trăm.
pub const DOI_TUONG_TOI_DA: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Ok,
    Warn,
    Err,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub level: Level,
    pub group: String,
    pub message: String,
    /// Mã máy đọc, để bên nhận khỏi dò theo câu chữ tiếng Việt. Mọi phát hiện mức `Warn`/`Err`
    /// đều có mã — ở cả phép kiểm từng gói lẫn phép kiểm cả đợt. Riêng mức `Ok` thì rỗng
    /// (bên nhận gộp chúng thành một dòng "các phép đã đạt", không cần phân loại).
    ///
    /// Tra [`huong_dan`] để lấy tiêu đề lời thường + cách sửa. **Đừng chép bảng đó sang bên
    /// mình** — ba tháng sau hai nơi lệch nhau.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub code: String,
    /// Tên các gói mà phát hiện này nói tới. Rỗng = nói về cả đợt, không chỉ riêng gói nào.
    ///
    /// Có trường này thì cổng gắn được nhãn lên đúng dòng gói trong bảng. Tên gói vẫn nằm
    /// trong `message` để người đọc thấy, nhưng **đừng tách từ đó ra** — câu chữ sẽ đổi.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub goi: Vec<String>,
    /// Tệp / văn bản / trường bị ảnh hưởng, cho những câu **gộp số** ("6 văn bản…", "3/40 tệp…").
    ///
    /// Tồn tại vì một con số thì người đi sửa không biết mở tệp nào ra. Cắt ở
    /// [`DOI_TUONG_TOI_DA`]; phần bị cắt đếm ở [`Finding::doi_tuong_bi_cat`]. Rỗng = phát hiện
    /// đã tự nói rõ chỗ (câu chữ có sẵn tên tệp), hoặc nói về cả gói.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub doi_tuong: Vec<String>,
    /// Số mục bị cắt khỏi [`Finding::doi_tuong`]. `0` = danh sách đủ.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub doi_tuong_bi_cat: usize,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

impl Finding {
    pub fn new(level: Level, group: &str, message: String) -> Self {
        Finding {
            level,
            group: group.into(),
            message,
            code: String::new(),
            goi: Vec::new(),
            doi_tuong: Vec::new(),
            doi_tuong_bi_cat: 0,
        }
    }
    /// Gắn mã máy đọc + danh sách gói liên quan (dùng ở [`crate::dot`]).
    pub fn ve(mut self, code: &str, goi: Vec<String>) -> Self {
        self.code = code.to_owned();
        self.goi = goi;
        self
    }
    /// Hướng dẫn sửa cho phát hiện này, `None` khi mã chưa có trong bảng hoặc mức là `Ok`.
    pub fn huong_dan(&self) -> Option<HuongDan> {
        huong_dan(&self.code)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfRow {
    pub file: String,
    pub bytes: u64,
    pub pdf_version: String,
    pub pages: u32,
    pub text_pages: u32,
    /// Loại PDF/A tệp tự khai ("PDF/A-2B"…) — None = không khai.
    pub pdfa: Option<String>,
    pub output_intent: bool,
    pub min_dpi: Option<u32>,
    pub bit_depth: Option<u32>,
    pub producer: Option<String>,
    pub signed: bool,
    /// Số chữ ký trong tệp. Công văn phát hành theo quy trình Ban Cơ yếu có tới 4 (phê duyệt,
    /// số công văn, ngày, dấu tổ chức) — `sig_valid` là kết luận cho **tất cả**, không phải
    /// cho một cái.
    #[serde(default)]
    pub so_chu_ky: usize,
    /// Chữ ký toàn vẹn (băm khớp, phủ trọn tệp). None = không ký.
    pub sig_valid: Option<bool>,
    pub signer: Option<String>,
    pub issuer: Option<String>,
    pub cert_until: Option<String>,
    pub signing_time: Option<String>,
    pub tsa: bool,
    pub ltv: bool,
    pub problems: Vec<String>,
    /// Kết luận ngắn: "Đạt" / "Không đạt" / "Đạt, có cảnh báo".
    pub verdict: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageResult {
    pub source: String,
    pub name: String,
    pub root: String,
    pub sha256: Option<String>,
    pub size: u64,
    /// SIP / AIP / DIP / "?".
    pub kind: String,
    pub objid: String,
    pub profile: String,
    pub creator_org: String,
    pub creator_software: String,
    pub created: String,
    pub file_code: String,
    pub title: String,
    pub total_doc: Option<u32>,
    pub pages: Option<u32>,
    pub data_files: usize,
    pub findings: Vec<Finding>,
    pub pdfs: Vec<PdfRow>,
    pub errors: usize,
    pub warnings: usize,
    /// "ĐẠT" | "ĐẠT (có cảnh báo)" | "KHÔNG ĐẠT" | "CHƯA KIỂM ĐỦ" | "CHƯA KIỂM ĐỦ (có cảnh báo)"
    pub verdict: String,
    /// Số phép kiểm **không** chạy được vì thiếu byte thật. `0` = kiểm đủ.
    /// Khác 0 thì `verdict` **không bao giờ** là "ĐẠT" trơn — xem `finish`.
    /// Luôn bằng `bo_qua.tong()`; đọc `bo_qua` khi cần biết bỏ những nhóm nào.
    pub chua_kiem_noi_dung: usize,
    /// Cùng số đó nhưng tách theo nhóm, để màn hình liệt kê được "chưa kiểm những gì"
    /// mà không phải viết cứng danh sách ở phía giao diện.
    pub bo_qua: BoQua,
}

/// Phép kiểm bị bỏ vì bản kê không mang byte thật, đếm **theo nhóm**.
///
/// Tồn tại vì một con số tổng thì giao diện không nói được "chưa kiểm cái gì" — nó sẽ phải
/// tự viết cứng danh sách ba nhóm, rồi đứng im khi ở đây thêm nhóm thứ tư.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BoQua {
    /// Tệp PDF không kiểm được PDF/A (lớp chữ, dpi, chữ ký).
    pub pdfa: usize,
    /// Mã băm khai trong METS không đối chiếu được.
    pub bam_mets: usize,
    /// Mã băm (fixity) khai trong PREMIS không đối chiếu được.
    pub bam_premis: usize,
}

impl BoQua {
    pub fn tong(&self) -> usize {
        self.pdfa + self.bam_mets + self.bam_premis
    }

    /// `(mã, nhãn, số phép, kiểm ở đâu)` của **riêng** những nhóm có phép bị bỏ.
    ///
    /// Thêm nhóm mới thì thêm một dòng ở đây là xong — `match` không có nhánh `_` nên
    /// quên là lỗi biên dịch.
    pub fn danh_sach(&self) -> Vec<(&'static str, &'static str, usize, &'static str)> {
        [Nhom::Pdfa, Nhom::BamMets, Nhom::BamPremis]
            .into_iter()
            .filter_map(|n| {
                let so = match n {
                    Nhom::Pdfa => self.pdfa,
                    Nhom::BamMets => self.bam_mets,
                    Nhom::BamPremis => self.bam_premis,
                };
                if so == 0 {
                    return None;
                }
                Some(match n {
                    Nhom::Pdfa => ("pdfa", "PDF/A từng tệp (lớp chữ, dpi, chữ ký)", so, "Tự kiểm PDF/A cho tệp mẫu"),
                    Nhom::BamMets => ("bam_mets", "Mã băm SHA-256 khai trong METS", so, "Bản kiểm đầy đủ lúc nộp gói"),
                    Nhom::BamPremis => ("bam_premis", "Mã băm (fixity) khai trong PREMIS", so, "Bản kiểm đầy đủ lúc nộp gói"),
                })
            })
            .collect()
    }
}

#[derive(Clone, Copy)]
enum Nhom {
    Pdfa,
    BamMets,
    BamPremis,
}

/// Nội dung gói đã đọc — để NHẬP gói vào Hệ thống (bản Lưu trữ, 4/11/2026), không chỉ kiểm.
#[derive(Debug, Clone, Default)]
pub struct Extract {
    /// Đường dẫn từ gốc gói → nội dung (mọi tệp).
    pub files: BTreeMap<String, Vec<u8>>,
    /// Trường đơn của EAD hồ sơ (tên phần tử cục bộ → chữ), vd `arcFileCode`, `title`, `maintenance`.
    pub ho_so: BTreeMap<String, String>,
    pub docs: Vec<DocExtract>,
}

#[derive(Debug, Clone, Default)]
pub struct DocExtract {
    pub ead_path: String,
    /// Trường đơn của EAD văn bản, vd `arcDocCode`, `typeName`, `subject`, `issuedDate`.
    pub fields: BTreeMap<String, String>,
    /// Tệp dữ liệu của văn bản: theo structMap METS (DMDID → FILEID), không có thì theo tên tệp = mã văn bản.
    pub data_path: Option<String>,
}

fn fields_of(e: &El) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    for c in &e.children {
        if c.children.is_empty() {
            m.entry(c.name.clone()).or_insert_with(|| c.text.trim().to_string());
        }
    }
    m
}

/// (EAD văn bản, tệp dữ liệu) theo structMap của METS bản đại diện: div DMDID → mdRef của dmdSec, fptr FILEID → FLocat.
fn cap_ead_tep(m: &El, base: &str) -> Vec<(String, String)> {
    let dmd: HashMap<&str, String> = m
        .all("dmdSec")
        .into_iter()
        .filter_map(|d| Some((d.attr("ID")?, resolve(base, d.all("mdRef").first()?.attr("href")?))))
        .collect();
    let file: HashMap<&str, String> = m
        .all("file")
        .into_iter()
        .filter_map(|f| Some((f.attr("ID")?, resolve(base, f.all("FLocat").first()?.attr("href")?))))
        .collect();
    let mut out = Vec::new();
    for div in m.all("div") {
        let Some(ids) = div.attr("DMDID") else { continue };
        let Some(tep) = div.all("fptr").iter().filter_map(|p| p.attr("FILEID")).find_map(|f| file.get(f)) else { continue };
        for id in ids.split_whitespace() {
            if let Some(ead) = dmd.get(id) {
                out.push((ead.clone(), tep.clone()));
            }
        }
    }
    out
}

struct Ctx {
    f: Vec<Finding>,
}
impl Ctx {
    /// Phép kiểm đã đạt — không có mã, bên nhận gộp chúng thành một dòng.
    fn ok(&mut self, g: &str, m: impl Into<String>) {
        self.f.push(Finding::new(Level::Ok, g, m.into()));
    }
    /// `ma` là tham số **đầu**, bắt buộc: thêm phát hiện mới mà quên mã thì không biên dịch
    /// được, và test `moi_ma_deu_co_huong_dan` bắt tiếp nếu mã chưa có hướng dẫn.
    fn warn(&mut self, ma: &str, g: &str, m: impl Into<String>) {
        self.f.push(Finding::new(Level::Warn, g, m.into()).ve(ma, Vec::new()));
    }
    fn err(&mut self, ma: &str, g: &str, m: impl Into<String>) {
        self.f.push(Finding::new(Level::Err, g, m.into()).ve(ma, Vec::new()));
    }
    /// Như [`Ctx::warn`] nhưng kèm danh sách chỗ bị ảnh hưởng — dùng cho câu **gộp số**.
    fn warn_ds(&mut self, ma: &str, g: &str, m: impl Into<String>, ds: Vec<String>) {
        self.f.push(gan_ds(Finding::new(Level::Warn, g, m.into()).ve(ma, Vec::new()), ds));
    }
    fn err_ds(&mut self, ma: &str, g: &str, m: impl Into<String>, ds: Vec<String>) {
        self.f.push(gan_ds(Finding::new(Level::Err, g, m.into()).ve(ma, Vec::new()), ds));
    }
}

/// Gắn danh sách chỗ bị ảnh hưởng, cắt ở [`DOI_TUONG_TOI_DA`] và đếm phần bị cắt.
fn gan_ds(mut f: Finding, mut ds: Vec<String>) -> Finding {
    if ds.len() > DOI_TUONG_TOI_DA {
        f.doi_tuong_bi_cat = ds.len() - DOI_TUONG_TOI_DA;
        ds.truncate(DOI_TUONG_TOI_DA);
    }
    f.doi_tuong = ds;
    f
}

fn sha256(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}

fn pct_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Ghép href (tương đối so với thư mục chứa tệp METS) thành đường dẫn từ gốc gói.
fn resolve(base_dir: &str, href: &str) -> String {
    let href = pct_decode(href.trim());
    let href = href.strip_prefix("file://").unwrap_or(&href);
    let mut parts: Vec<&str> = if base_dir.is_empty() { Vec::new() } else { base_dir.split('/').collect() };
    for p in href.split(['/', '\\']) {
        match p {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    parts.join("/")
}

fn dir_of(path: &str) -> &str {
    path.rsplit_once('/').map(|x| x.0).unwrap_or("")
}

const CODES: [(&str, u32); 6] = [("maintenance", 7), ("mode", 3), ("language", 11), ("typeName", 32), ("confidenceLevel", 3), ("format", 3)];
const OPTIONAL_CODES: [&str; 2] = ["format", "riskRecoveryStatus"];

fn is_dd_mm_yyyy(s: &str) -> bool {
    let p: Vec<&str> = s.split('/').collect();
    p.len() == 3 && p[0].len() == 2 && p[1].len() == 2 && p[2].len() == 4 && p.iter().all(|x| x.bytes().all(|b| b.is_ascii_digit()))
}

/// Đối chiếu RÚT GỌN với XSD đi kèm: tên phần tử gốc, các trường con bắt buộc (minOccurs ≠ 0), trường lạ.
fn xsd_lite(xsd: &El, doc: &El) -> Vec<String> {
    let mut out = Vec::new();
    let tops: Vec<&El> = xsd.children.iter().filter(|c| c.name == "element").collect();
    let Some(root_decl) = tops.iter().find(|e| e.attr("name") == Some(doc.name.as_str())).copied().or_else(|| tops.first().copied()) else {
        return out;
    };
    if root_decl.attr("name") != Some(doc.name.as_str()) {
        out.push(format!("phần tử gốc <{}> khác schema (<{}>)", doc.name, root_decl.attr("name").unwrap_or("?")));
        return out;
    }
    // Trường con: phần tử khai trong complexType của gốc (kể cả kiểu tham chiếu tên).
    let mut decls: Vec<&El> = root_decl.all("element").into_iter().filter(|e| !std::ptr::eq(*e, root_decl)).collect();
    if decls.is_empty() {
        if let Some(t) = root_decl.attr("type") {
            let t = t.rsplit(':').next().unwrap_or(t);
            if let Some(ct) = xsd.children.iter().find(|c| c.name == "complexType" && c.attr("name") == Some(t)) {
                decls = ct.all("element");
            }
        }
    }
    if decls.is_empty() {
        return out;
    }
    let allowed: BTreeSet<&str> = decls.iter().filter_map(|e| e.attr("name").or(e.attr("ref"))).map(|n| n.rsplit(':').next().unwrap_or(n)).collect();
    let present: BTreeSet<&str> = doc.children.iter().map(|c| c.name.as_str()).collect();
    for e in &decls {
        let Some(n) = e.attr("name").or(e.attr("ref")).map(|n| n.rsplit(':').next().unwrap_or(n)) else { continue };
        if e.attr("minOccurs") != Some("0") && !present.contains(n) {
            out.push(format!("thiếu trường bắt buộc <{n}>"));
        }
    }
    for p in &present {
        if !allowed.contains(p) {
            out.push(format!("có trường <{p}> không có trong schema"));
        }
    }
    out
}

/// `date_bad` gom theo tên trường → **danh sách nhãn** các bản EAD sai, không chỉ con số: người
/// đi sửa cần biết mở tệp nào ra. Khoá `#riskRecovery` là chỗ riêng cho cặp trường không nhất quán.
fn check_codes(c: &mut Ctx, label: &str, e: &El, date_bad: &mut BTreeMap<String, Vec<String>>) {
    for (k, max) in CODES {
        let Some(v) = e.child_text(k) else { continue };
        if v.is_empty() {
            if !OPTIONAL_CODES.contains(&k) {
                c.err("EAD-TRUONG-RONG", "EAD", format!("{label}: <{k}> rỗng"));
            }
            continue;
        }
        let ok = v.len() == 2 && v.parse::<u32>().is_ok_and(|n| (1..=max).contains(&n));
        if !ok {
            c.err("EAD-MA-NGOAI-BANG", "EAD", format!("{label}: <{k}>={v} ngoài bảng mã 01..{max:02}"));
        }
    }
    for k in ["startDate", "endDate", "issuedDate"] {
        if let Some(v) = e.child_text(k).filter(|v| !v.is_empty()) {
            if !is_dd_mm_yyyy(&v) {
                date_bad.entry(k.to_string()).or_default().push(label.to_string());
            }
        }
    }
    if let Some(v) = e.child_text("riskRecovery").filter(|v| !v.is_empty()) {
        if v != "0" && v != "1" {
            c.err("EAD-RISKRECOVERY-SAI", "EAD", format!("{label}: <riskRecovery>={v} (chỉ 0/1)"));
        } else if v == "0" && e.child_text("riskRecoveryStatus").is_some_and(|s| !s.is_empty()) {
            date_bad.entry("#riskRecovery".into()).or_default().push(label.to_string());
        }
    }
}

fn fmt_unix(t: i64) -> String {
    chrono::DateTime::from_timestamp(t, 0)
        .map(|d| d.with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07")).format("%d/%m/%Y %H:%M").to_string())
        .unwrap_or_default()
}

fn sig_m_time(pdf: &[u8]) -> Option<String> {
    // /M (D:YYYYMMDDHHmmSS…) của từ điển chữ ký — thời điểm theo đồng hồ máy ký.
    let at = pdf.windows(4).rposition(|w| w == b"/M (")? + 4;
    let s: String = pdf[at..(at + 20).min(pdf.len())].iter().map(|b| *b as char).collect();
    let s = s.strip_prefix("D:")?;
    if s.len() < 12 {
        return None;
    }
    Some(format!("{}/{}/{} {}:{}", &s[6..8], &s[4..6], &s[0..4], &s[8..10], &s[10..12]))
}

fn check_pdf(name: &str, bytes: &[u8], aip: bool) -> PdfRow {
    let r = digitext_extimport::pdfa::inspect_bytes(name, name, bytes);
    let mut row = PdfRow {
        file: name.to_string(),
        bytes: bytes.len() as u64,
        pdf_version: r.pdf_version.clone(),
        pages: r.pages,
        text_pages: r.text_pages,
        pdfa: r.declared.clone(),
        output_intent: r.has_output_intent,
        min_dpi: r.min_dpi,
        bit_depth: r.min_bit_depth,
        producer: r.producer.clone(),
        signed: r.signed,
        ..Default::default()
    };
    let mut hard = Vec::new();
    let mut soft = Vec::new();
    if let Some(e) = &r.error {
        hard.push(format!("PDF hỏng / không mở được: {e}"));
    } else {
        if r.declared.is_none() {
            hard.push("không phải PDF/A (không khai pdfaid trong XMP)".into());
        }
        if !r.has_output_intent {
            hard.push("không nhúng hồ sơ màu (OutputIntent)".into());
        }
        if r.text_pages < r.pages {
            hard.push(format!("{}/{} trang không có lớp chữ", r.pages - r.text_pages, r.pages));
        }
        if let Some(d) = r.min_dpi.filter(|d| *d < if r.is_map { 300 } else { 200 }) {
            hard.push(format!("độ phân giải {d} dpi thấp hơn TT05 Điều 8.2.a"));
        }
        if let Some(b) = r.min_bit_depth.filter(|b| *b < 24) {
            soft.push(format!("ảnh {b} bit (TT05 Điều 8.2.a: màu 24 bit)"));
        }
    }
    if r.signed {
        // `kiem_tat_ca` chứ KHÔNG phải `verify`: `verify` chỉ xác minh một chữ ký và đòi nó phủ
        // trọn tệp, nên tệp ký nhiều lần (công văn phát hành: phê duyệt → số → ngày → dấu tổ
        // chức) bị kết luận "đã bị sửa sau khi ký" — vu oan cho văn bản hợp lệ. Đổi 19/11/2026;
        // ghi chú đầy đủ ở đầu `sign-verify/src/nhieu.rs`.
        let bay_gio = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let kq = digitext_sign_verify::kiem_tat_ca(bytes);
        let k = digitext_sign_verify::ket_luan_tep(&kq, bay_gio);
        row.so_chu_ky = kq.so_chu_ky();
        row.sig_valid = Some(k.loi.is_empty());
        row.ltv = kq.co_dss;
        // Thông tin người ký lấy từ chữ ký SAU CÙNG — con dấu cuối là thứ bên nhận đối chiếu.
        // Chi tiết từng chữ ký thì xem Tiện ích › Kiểm chữ ký số.
        if let Some(cuoi) = kq.chu_ky.last() {
            row.tsa = cuoi.co_tsa;
            row.signing_time = cuoi.thoi_diem_tsa_unix.map(fmt_unix).or_else(|| sig_m_time(bytes));
            if let Some(ng) = &cuoi.nguoi_ky {
                row.signer = Some(ng.signer.clone());
                row.issuer = Some(ng.issuer.clone());
                row.cert_until = Some(fmt_unix(ng.not_after_unix));
            }
        }
        hard.extend(k.loi);
        soft.extend(k.canh_bao);
        if aip && !kq.co_dss {
            soft.push("chữ ký chưa có thông tin xác minh lâu dài (LTV/DSS) — nên có với gói AIP".into());
        }
    } else {
        soft.push("chưa ký số (TT05 Điều 8.2.a)".into());
    }
    row.verdict = if !hard.is_empty() {
        "Không đạt".into()
    } else if !soft.is_empty() {
        "Đạt, có cảnh báo".into()
    } else {
        "Đạt".into()
    };
    row.problems = hard.into_iter().chain(soft).collect();
    row
}

fn mets_info(m: &El) -> (String, String, String, String, String) {
    let kind = m.all("metsHdr").first().and_then(|h| h.attr("OAISPACKAGETYPE")).unwrap_or("").to_string();
    let mut org = String::new();
    let mut sw = String::new();
    for a in m.all("agent") {
        let name = a.child_text("name").unwrap_or_default();
        match a.attr("TYPE") {
            Some("ORGANIZATION") if org.is_empty() => {
                let code = a.all("note").first().map(|n| n.text.trim().to_string()).unwrap_or_default();
                org = if code.is_empty() { name } else { format!("{name} ({code})") };
            }
            Some("OTHER") | Some("SOFTWARE") if sw.is_empty() => {
                let ver = a.all("note").first().map(|n| n.text.trim().to_string()).unwrap_or_default();
                sw = format!("{name} {ver}").trim().to_string();
            }
            _ => {}
        }
    }
    let created = m.all("metsHdr").first().and_then(|h| h.attr("CREATEDATE")).unwrap_or("").to_string();
    (kind, org, sw, created, m.attr("PROFILE").unwrap_or("").to_string())
}

/// Kiểm một gói (tệp .zip hoặc thư mục).
pub fn check_path(path: &Path) -> PackageResult {
    check_and_extract(path).0
}

/// Kiểm một gói và trả kèm nội dung đã đọc (None khi không đọc được gói / thiếu METS.xml gốc).
pub fn check_and_extract(path: &Path) -> (PackageResult, Option<Extract>) {
    let name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let mut res = PackageResult { source: path.display().to_string(), name: name.clone(), kind: "?".into(), ..Default::default() };
    let loaded = match load::load(path) {
        Ok(l) => l,
        Err(e) => {
            res.findings.push(Finding::new(Level::Err, "Cấu trúc", e));
            return (finish(res), None);
        }
    };
    check_loaded(res, loaded)
}

/// Kiểm một gói **đã đọc sẵn**. Dùng cho bản rút gọn của cổng nhà cung cấp: gói ở đó chỉ mang
/// siêu dữ liệu, các tệp nội dung chỉ có tên trong `loaded.thieu_noi_dung`.
///
/// Mọi phép kiểm cấu trúc chạy y như gói đầy đủ — chúng chỉ đọc XML. Ba phép kiểm cần byte thật
/// (mã băm METS, mã băm PREMIS, PDF/A từng tệp) thì **bỏ qua và đếm lại**, chứ không được băm
/// một mảng rỗng rồi báo "sai mã băm" cho cả nghìn tệp lành lặn.
pub fn check_loaded(mut res: PackageResult, loaded: Loaded) -> (PackageResult, Option<Extract>) {
    res.root = loaded.root.clone();
    res.sha256 = loaded.sha256.clone();
    res.size = loaded.size;
    let mut c = Ctx { f: loaded.notes };
    let files = &loaded.files;
    let thieu = &loaded.thieu_noi_dung;
    let co_tep = &loaded.co_tep;
    // Số phép kiểm cần nội dung mà phải bỏ qua. Mỗi chỗ bỏ qua cộng một, và `finish` dùng con số
    // này để KHÔNG kết luận "ĐẠT".
    let mut bo_qua = BoQua::default();

    // 1. XML đọc được.
    let mut parsed: HashMap<&str, El> = HashMap::new();
    let mut bad_xml = 0;
    for (p, b) in files {
        let low = p.to_lowercase();
        if low.ends_with(".xml") || low.ends_with(".xsd") {
            match xml::parse(b) {
                Ok(e) => {
                    parsed.insert(p.as_str(), e);
                }
                Err(e) => {
                    bad_xml += 1;
                    c.err("XML-HONG", "XML", format!("{p}: XML hỏng ({e})"));
                }
            }
        }
    }
    let n_xml = parsed.len() + bad_xml;
    if bad_xml == 0 {
        c.ok("XML", format!("{n_xml} tệp XML/XSD đều well-formed"));
    }

    // 2. Tệp bắt buộc.
    let Some(root_mets) = parsed.get("METS.xml") else {
        c.err("THIEU-METS-GOC", "Cấu trúc", "Thiếu METS.xml ở gốc gói");
        res.findings = c.f;
        return (finish(res), None);
    };
    let (kind, org, sw, created, profile) = mets_info(root_mets);
    res.kind = if kind.is_empty() { "?".into() } else { kind.to_uppercase() };
    res.creator_org = org;
    res.creator_software = sw;
    res.created = created;
    res.profile = profile.clone();
    res.objid = root_mets.attr("OBJID").unwrap_or("").to_string();
    let aip = res.kind == "AIP";
    let reps: BTreeSet<String> = files
        .keys()
        .filter_map(|p| p.strip_prefix("representations/").and_then(|r| r.split_once('/')).map(|(r, _)| r.to_string()))
        .collect();
    if reps.is_empty() {
        c.err("THIEU-REPRESENTATIONS", "Cấu trúc", "Thiếu thư mục representations/ (không có bản đại diện nào)");
    }
    for r in &reps {
        if !files.contains_key(&format!("representations/{r}/METS.xml")) {
            c.err("THIEU-METS-BAN-DAI-DIEN", "Cấu trúc", format!("representations/{r}: thiếu METS.xml"));
        }
    }
    if !files.keys().any(|p| p.starts_with("schemas/") && p.to_lowercase().ends_with(".xsd")) {
        c.err("THIEU-SCHEMAS", "Cấu trúc", "Thiếu thư mục schemas/ (các tệp XSD đi kèm)");
    }

    // 3. Loại gói, OBJID, PROFILE.
    if kind.is_empty() {
        c.err("METS-THIEU-LOAI-GOI", "METS", "METS.xml thiếu csip:OAISPACKAGETYPE (SIP/AIP/DIP)");
    } else {
        c.ok("METS", format!("Loại gói OAIS: {}", res.kind));
    }
    if res.objid.is_empty() {
        c.err("METS-THIEU-OBJID", "METS", "METS.xml thiếu OBJID");
    } else if !loaded.root.is_empty() {
        let want = res.objid.replace(':', "_");
        if want == loaded.root {
            c.ok("METS", format!("OBJID {} khớp tên thư mục gốc", res.objid));
        } else {
            c.err("METS-OBJID-KHAC-TEN-THU-MUC", "METS", format!("OBJID \"{}\" không khớp tên thư mục gốc \"{}\"", res.objid, loaded.root));
        }
    }
    let csip = |p: &str| p.to_lowercase().contains("e-ark-csip") || p.to_lowercase().contains("earkcsip");
    if profile.is_empty() {
        c.warn("METS-KHONG-KHAI-PROFILE", "METS", "METS.xml không khai PROFILE");
    } else if !csip(&profile) {
        c.warn("METS-PROFILE-KHONG-CSIP", "METS", format!("METS.xml dùng PROFILE \"{profile}\" — không phải hồ sơ E-ARK CSIP"));
    }
    if root_mets.all("structMap").iter().any(|s| s.attr("LABEL") == Some("CSIP")) {
        c.ok("METS", "structMap LABEL=\"CSIP\" đúng E-ARK");
    } else {
        c.err("METS-THIEU-STRUCTMAP-CSIP", "METS", "Không có structMap LABEL=\"CSIP\"");
    }
    if res.kind == "SIP" {
        if root_mets.all("altRecordID").iter().any(|a| a.attr("TYPE") == Some("SUBMISSIONAGREEMENT")) {
            c.ok("METS", "SIP có mã thoả thuận nộp lưu (SUBMISSIONAGREEMENT)");
        } else {
            c.warn("SIP-THIEU-THOA-THUAN-NOP-LUU", "METS", "SIP không có altRecordID SUBMISSIONAGREEMENT (thoả thuận nộp lưu)");
        }
    }

    // 4. Tham chiếu METS: tồn tại, SHA-256, kích thước; tệp mồ côi.
    let mut referenced: BTreeSet<String> = BTreeSet::new();
    referenced.insert("METS.xml".into());
    let mets_paths: Vec<String> = std::iter::once("METS.xml".to_string()).chain(reps.iter().map(|r| format!("representations/{r}/METS.xml"))).collect();
    for mp in &mets_paths {
        let Some(m) = parsed.get(mp.as_str()) else { continue };
        if mp != "METS.xml" {
            let (_, _, _, _, p) = mets_info(m);
            if !p.is_empty() && csip(&p) != csip(&profile) {
                c.warn("METS-PROFILE-LECH-NHAU", "METS", format!("{mp} dùng PROFILE \"{p}\" khác METS gốc"));
            }
        }
        let base = dir_of(mp);
        let mut n = 0;
        let mut bad = 0;
        let mut items: Vec<(&El, Option<&str>)> = Vec::new();
        for f in m.all("file") {
            let href = f.all("FLocat").first().and_then(|l| l.attr("href"));
            items.push((f, href));
        }
        for r in m.all("mdRef") {
            items.push((r, r.attr("href")));
        }
        for (el, href) in items {
            n += 1;
            let Some(href) = href else {
                bad += 1;
                c.err("METS-MUC-THIEU-HREF", "METS", format!("{mp}: mục {} thiếu xlink:href", el.attr("ID").unwrap_or("?")));
                continue;
            };
            let target = resolve(base, href);
            referenced.insert(target.clone());
            let Some(data) = files.get(&target) else {
                bad += 1;
                c.err("METS-TRO-TEP-KHONG-CO", "METS", format!("{mp}: tham chiếu tới tệp không có trong gói: {target}"));
                continue;
            };
            if let Some(t) = el.attr("CHECKSUMTYPE") {
                if !t.eq_ignore_ascii_case("SHA-256") {
                    // Nói luôn HẬU QUẢ, không chỉ "nên SHA-256": nhánh đối chiếu bên dưới chỉ
                    // chạy khi CHECKSUMTYPE là SHA-256, nên với loại khác mã băm KHÔNG được
                    // kiểm — tệp bị sửa sau khi đóng gói cũng không ai biết. Câu cũ đọc như
                    // chuyện hình thức.
                    c.warn(
                        "METS-BAM-KHONG-SHA256",
                        "METS",
                        format!("{mp}: {target} dùng {t}, không phải SHA-256 — mã băm vì vậy KHÔNG được đối chiếu"),
                    );
                }
            }
            match el.attr("CHECKSUM") {
                // Không có byte thật thì không có gì để băm — bỏ qua, đếm lại, tuyệt đối không
                // so với mã băm của mảng rỗng.
                Some(_) if thieu.contains(&target) => {
                    bo_qua.bam_mets += 1;
                }
                Some(sum) if el.attr("CHECKSUMTYPE").is_none_or(|t| t.eq_ignore_ascii_case("SHA-256")) => {
                    if !sum.eq_ignore_ascii_case(&sha256(data)) {
                        bad += 1;
                        c.err("METS-BAM-SAI", "METS", format!("{mp}: {target} sai mã băm SHA-256 (tệp đã bị thay đổi sau khi đóng gói)"));
                    }
                }
                Some(_) => {}
                None => {
                    bad += 1;
                    c.err("METS-THIEU-BAM", "METS", format!("{mp}: {target} không ghi CHECKSUM"));
                }
            }
            if let Some(sz) = el.attr("SIZE") {
                // Cỡ lấy từ `co_tep`, không phải `data.len()`: bản rút gọn không gửi byte nào
                // nên `data` rỗng, mà cỡ thì bản kê vẫn có ⇒ phép kiểm này vẫn chạy được.
                let co_that = co_tep.get(&target).copied().unwrap_or(data.len() as u64);
                if sz.parse::<u64>().ok() != Some(co_that) {
                    bad += 1;
                    c.err("METS-SIZE-SAI", "METS", format!("{mp}: {target} SIZE={sz} khác thực tế {co_that}"));
                }
            }
        }
        for p in m.all("mptr") {
            if let Some(h) = p.attr("href") {
                let t = resolve(base, h);
                referenced.insert(t.clone());
                if !files.contains_key(&t) {
                    c.err("METS-MPTR-KHONG-CO", "METS", format!("{mp}: mptr trỏ tới tệp không có: {t}"));
                }
            }
        }
        if bad == 0 && n > 0 {
            c.ok("METS", format!("{mp}: {n} tham chiếu — tồn tại, đúng kích thước và SHA-256"));
        }
    }
    let orphans: Vec<&String> = files.keys().filter(|p| !referenced.contains(*p)).collect();
    if orphans.is_empty() {
        c.ok("METS", format!("Không có tệp mồ côi — cả {} tệp đều được METS tham chiếu", files.len()));
    } else {
        c.err_ds(
            "METS-TEP-MO-COI",
            "METS",
            format!("{} tệp không được METS nào tham chiếu: {}", orphans.len(), orphans.iter().take(8).map(|s| s.as_str()).collect::<Vec<_>>().join(", ")),
            orphans.iter().map(|s| (*s).clone()).collect(),
        );
    }

    // 5. EAD hồ sơ.
    let mut date_bad: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let ead_path = ["metadata/descriptive/EAD.xml", "metadata/descriptive/EAD_TL.xml"].into_iter().find(|p| parsed.contains_key(p));
    let mut file_code = String::new();
    match ead_path.and_then(|p| parsed.get(p).map(|e| (p, e))) {
        None => c.err("EAD-THIEU-HO-SO", "EAD", "Thiếu metadata/descriptive/EAD.xml (mô tả hồ sơ)"),
        Some((p, e)) => {
            check_codes(&mut c, "EAD hồ sơ", e, &mut date_bad);
            if res.kind == "DIP" {
                // Gói DIP: EAD.xml mô tả YÊU CẦU khai thác (Điều 42), không mô tả hồ sơ (5/11/2026).
                for k in ["requestID", "requestDate", "purpose"] {
                    if e.child_text(k).is_none_or(|v| v.is_empty()) {
                        c.err("EAD-DIP-THIEU-TRUONG", "EAD", format!("EAD yêu cầu DIP: thiếu <{k}>"));
                    }
                }
                // file_code để trống: văn bản trong DIP gom từ nhiều hồ sơ, không so khuôn <mã hồ sơ>.<STT>.
                res.title = e.child_text("purposeContent").unwrap_or_default();
            } else {
                for k in ["title", "maintenance", "mode", "language", "totalDoc", "confidenceLevel"] {
                    if e.child_text(k).is_none_or(|v| v.is_empty()) {
                        c.err("EAD-HO-SO-THIEU-TRUONG", "EAD", format!("EAD hồ sơ: thiếu <{k}>"));
                    }
                }
                file_code = e.child_text("arcFileCode").filter(|v| !v.is_empty()).or_else(|| e.child_text("fileCode").filter(|v| !v.is_empty())).unwrap_or_default();
                if file_code.is_empty() {
                    c.err("EAD-HO-SO-THIEU-MA", "EAD", "EAD hồ sơ: thiếu mã hồ sơ (arcFileCode / fileCode)");
                }
                if aip && e.child_text("arcFileCode").is_none_or(|v| v.is_empty()) {
                    c.warn("EAD-AIP-THIEU-ARCFILECODE", "EAD", "Gói AIP nhưng EAD hồ sơ không có arcFileCode (mã hồ sơ lưu trữ)");
                }
                res.title = e.child_text("title").unwrap_or_default();
            }
            res.total_doc = e.child_text("totalDoc").and_then(|v| v.parse().ok());
            res.pages = e.child_text("numberOfPage").and_then(|v| v.parse().ok());
            let xsd_name = format!("schemas/{}", p.rsplit('/').next().unwrap_or("EAD.xml").replace(".xml", ".xsd"));
            match parsed.get(xsd_name.as_str()) {
                Some(x) => {
                    let probs = xsd_lite(x, e);
                    if probs.is_empty() {
                        c.ok("EAD", format!("EAD hồ sơ khớp {xsd_name} (đủ trường bắt buộc, không trường lạ)"));
                    }
                    for pr in probs {
                        c.err("EAD-HO-SO-LECH-XSD", "EAD", format!("EAD hồ sơ so với {xsd_name}: {pr}"));
                    }
                }
                None => c.warn("EAD-KHONG-CO-XSD", "EAD", format!("Không có {xsd_name} để đối chiếu EAD hồ sơ")),
            }
        }
    }
    res.file_code = file_code.clone();

    // 6. EAD văn bản + tệp dữ liệu.
    let data: Vec<(&String, &Vec<u8>)> = files.iter().filter(|(p, _)| p.starts_with("representations/") && p.split('/').nth(2) == Some("data")).collect();
    res.data_files = data.len();
    if data.is_empty() {
        c.err("KHONG-CO-TEP-TAI-LIEU", "Cấu trúc", "Không có tệp tài liệu nào trong representations/*/data/");
    }
    // Xếp theo đường dẫn: `parsed` là HashMap nên thứ tự lặp đổi mỗi lần chạy. Không xếp thì
    // thứ tự các phát hiện EAD văn bản — và danh sách «chỗ bị ảnh hưởng» dựng từ vòng này —
    // nhảy loạn giữa hai lần kiểm cùng một gói.
    let mut doc_eads: Vec<(&str, &El)> = parsed
        .iter()
        .filter(|(p, _)| p.starts_with("representations/") && p.contains("/metadata/descriptive/"))
        .map(|(p, e)| (*p, e))
        .collect();
    doc_eads.sort_by_key(|(p, _)| *p);
    let doc_xsd = parsed.get("schemas/EAD_doc.xsd");
    let data_stems: BTreeSet<String> = data.iter().map(|(p, _)| {
        let n = p.rsplit('/').next().unwrap_or(p);
        n.rsplit_once('.').map(|x| x.0).unwrap_or(n).to_string()
    }).collect();
    let mut codes_seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut xsd_bad = 0;
    let mut pages_sum = 0u32;
    // Giữ NHÃN chứ không giữ số đếm: mấy câu gộp số bên dưới đều phải trả kèm danh sách chỗ
    // bị ảnh hưởng, "6 văn bản trùng trích yếu" mà không nói văn bản nào thì người đọc bế tắc.
    let mut no_subject: Vec<String> = Vec::new();
    let mut subjects: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut type_other: Vec<String> = Vec::new();
    for (p, e) in &doc_eads {
        let label = p.rsplit('/').next().unwrap_or(p).to_string();
        check_codes(&mut c, &label, e, &mut date_bad);
        let code = e.child_text("arcDocCode").filter(|v| !v.is_empty()).or_else(|| e.child_text("docCode").filter(|v| !v.is_empty()));
        match &code {
            None => c.err("EAD-VAN-BAN-THIEU-MA", "EAD", format!("{label}: thiếu mã văn bản (arcDocCode / docCode)")),
            Some(code) => {
                *codes_seen.entry(code.clone()).or_default() += 1;
                if !file_code.is_empty() {
                    let ok = code.strip_prefix(&format!("{file_code}.")).is_some_and(|rest| {
                        let parts: Vec<&str> = rest.split('.').collect();
                        let last = parts.last().copied().unwrap_or("");
                        last.len() == 7 && last.bytes().all(|b| b.is_ascii_digit()) && parts.len() <= 2 && (parts.len() == 1 || (1..=3).contains(&parts[0].len()))
                    });
                    if !ok {
                        c.err("EAD-MA-VAN-BAN-SAI-KHUON", "EAD", format!("{label}: mã văn bản \"{code}\" không theo khuôn <mã hồ sơ>.<STT 7 số>"));
                    }
                }
                if !data_stems.contains(code) {
                    c.err("EAD-KHONG-CO-TEP-DU-LIEU", "EAD", format!("{label}: không có tệp dữ liệu tên \"{code}.*\""));
                }
            }
        }
        match e.child_text("subject") {
            Some(s) if !s.is_empty() => subjects.entry(s).or_default().push(label.clone()),
            _ => no_subject.push(label.clone()),
        }
        if e.child_text("typeName").as_deref() == Some("32") {
            type_other.push(label.clone());
        }
        pages_sum += e.child_text("numberOfPage").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
        if let Some(x) = doc_xsd {
            let probs = xsd_lite(x, e);
            if !probs.is_empty() {
                xsd_bad += 1;
                c.err("EAD-VAN-BAN-LECH-XSD", "EAD", format!("{label} so với schemas/EAD_doc.xsd: {}", probs.join("; ")));
            }
        }
    }
    let dups: Vec<&String> = codes_seen.iter().filter(|(_, n)| **n > 1).map(|(k, _)| k).collect();
    if !dups.is_empty() {
        c.err_ds(
            "EAD-MA-VAN-BAN-TRUNG",
            "EAD",
            format!("Mã văn bản trùng: {}", dups.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")),
            dups.iter().map(|s| (*s).clone()).collect(),
        );
    }
    if !doc_eads.is_empty() && xsd_bad == 0 && doc_xsd.is_some() {
        c.ok("EAD", format!("{} EAD văn bản khớp schemas/EAD_doc.xsd", doc_eads.len()));
    }
    if !no_subject.is_empty() {
        c.warn_ds(
            "EAD-THIEU-TRICH-YEU",
            "EAD",
            format!("{} văn bản không có trích yếu (subject)", no_subject.len()),
            no_subject.clone(),
        );
    }
    // Chỉ các nhãn thuộc trích yếu BỊ TRÙNG, không phải mọi nhãn: người sửa cần đúng mấy văn
    // bản phải mở ra.
    let dup_nhan: Vec<String> = subjects.values().filter(|v| v.len() > 1).flatten().cloned().collect();
    if !dup_nhan.is_empty() {
        c.warn_ds(
            "EAD-TRICH-YEU-TRUNG",
            "EAD",
            format!("{} văn bản có trích yếu trùng nhau — kiểm lại trích yếu từng văn bản", dup_nhan.len()),
            dup_nhan,
        );
    }
    if !type_other.is_empty() && type_other.len() == doc_eads.len() {
        c.warn_ds(
            "EAD-TOAN-LOAI-KHAC",
            "EAD",
            format!("Cả {} văn bản đều là loại 32 (Khác) — nên xác định đúng tên loại văn bản", type_other.len()),
            type_other.clone(),
        );
    }
    for (k, nhan) in &date_bad {
        if k == "#riskRecovery" {
            c.warn_ds(
                "EAD-RISKRECOVERY-DU-TRANG-THAI",
                "EAD",
                format!("{} bản EAD ghi riskRecovery=0 (không dự phòng) nhưng vẫn điền riskRecoveryStatus", nhan.len()),
                nhan.clone(),
            );
        } else {
            c.warn_ds(
                "EAD-NGAY-SAI-DINH-DANG",
                "EAD",
                format!("{} trường <{k}> không theo dd/mm/yyyy (chỉ ghi năm hoặc sai định dạng)", nhan.len()),
                nhan.clone(),
            );
        }
    }
    match res.total_doc {
        Some(t) if t as usize == data.len() && t as usize == doc_eads.len() => c.ok("EAD", format!("totalDoc = số tệp dữ liệu = số EAD văn bản = {t}")),
        Some(t) => c.err("EAD-TOTALDOC-LECH", "EAD", format!("totalDoc={t} nhưng có {} tệp dữ liệu, {} EAD văn bản", data.len(), doc_eads.len())),
        None => {}
    }
    if let Some(p) = res.pages {
        if !doc_eads.is_empty() {
            if p == pages_sum {
                c.ok("EAD", format!("numberOfPage hồ sơ = tổng số trang các văn bản = {p}"));
            } else {
                c.warn("EAD-SO-TRANG-LECH", "EAD", format!("numberOfPage hồ sơ = {p} nhưng tổng các văn bản = {pages_sum}"));
            }
        }
    }

    // 7. PREMIS.
    let premis_root = parsed.get("metadata/preservation/PREMIS.xml");
    match (premis_root, aip) {
        (None, true) => c.err("PREMIS-AIP-THIEU-GOC", "PREMIS", "Gói AIP thiếu metadata/preservation/PREMIS.xml"),
        (None, false) => c.ok("PREMIS", format!("{} không kèm PREMIS (không bắt buộc)", res.kind)),
        (Some(p), _) => {
            let ev: BTreeSet<String> = p.all("eventType").iter().map(|e| e.text.trim().to_string()).collect();
            c.ok("PREMIS", format!("PREMIS gốc có sự kiện: {}", ev.into_iter().collect::<Vec<_>>().join(", ")));
        }
    }
    for r in &reps {
        let pp = format!("representations/{r}/metadata/preservation/PREMIS_{r}.xml");
        let Some(p) = parsed.get(pp.as_str()) else {
            if aip {
                c.err("PREMIS-AIP-THIEU-BAN-DAI-DIEN", "PREMIS", format!("Gói AIP thiếu {pp}"));
            }
            continue;
        };
        let objs = p.all("object");
        let digests: BTreeSet<String> = p.all("messageDigest").iter().map(|d| d.text.trim().to_lowercase()).collect();
        let mut miss = 0;
        let mut no_orig: Vec<String> = Vec::new();
        let rep_data: Vec<&(&String, &Vec<u8>)> = data.iter().filter(|(path, _)| path.starts_with(&format!("representations/{r}/"))).collect();
        for (path, b) in &rep_data {
            let fname = path.rsplit('/').next().unwrap_or(path);
            if thieu.contains(*path) {
                bo_qua.bam_premis += 1;
            } else if !digests.contains(&sha256(b)) {
                miss += 1;
                c.err("PREMIS-THIEU-BAM-DUNG", "PREMIS", format!("{pp}: không có mã băm SHA-256 đúng cho {fname}"));
            }
            let has_orig = objs.iter().any(|o| o.all("originalName").iter().any(|n| n.text.trim() == fname));
            if !has_orig {
                no_orig.push(fname.to_string());
            }
        }
        if miss == 0 && !rep_data.is_empty() {
            c.ok("PREMIS", format!("{pp}: fixity SHA-256 khớp {}/{} tệp", rep_data.len(), rep_data.len()));
        }
        if !no_orig.is_empty() {
            c.warn_ds(
                "PREMIS-THIEU-ORIGINALNAME",
                "PREMIS",
                format!("{pp}: {} tệp không có originalName — khó đối chiếu tự động", no_orig.len()),
                no_orig.clone(),
            );
        }
        if p.all("formatName").iter().any(|f| f.text.trim().eq_ignore_ascii_case("Portable Document Format")) {
            c.warn("PREMIS-FORMATNAME-CHUNG", "PREMIS", format!("{pp}: formatName chỉ ghi \"Portable Document Format\" — nên ghi PDF/A-2b (PRONOM fmt/477)"));
        }
    }

    // 8. Từng tệp PDF.
    for (p, b) in &data {
        if p.to_lowercase().ends_with(".pdf") {
            if thieu.contains(*p) {
                bo_qua.pdfa += 1;
                continue;
            }
            res.pdfs.push(check_pdf(p.rsplit('/').next().unwrap_or(p), b, aip));
        }
    }
    if !res.pdfs.is_empty() {
        let n = res.pdfs.len();
        // Giữ TÊN TỆP, không giữ số đếm: "3/40 tệp thiếu lớp chữ" mà không nói tệp nào thì
        // người sửa phải mở cả 40. Số đếm lấy lại bằng `.len()`.
        let ten = |loc: &dyn Fn(&PdfRow) -> bool| -> Vec<String> {
            res.pdfs.iter().filter(|r| loc(r)).map(|r| r.file.clone()).collect()
        };
        let not_pdfa = ten(&|r: &PdfRow| r.pdfa.is_none() || !r.output_intent);
        let no_text = ten(&|r: &PdfRow| r.text_pages < r.pages);
        let low_dpi = ten(&|r: &PdfRow| r.problems.iter().any(|x| x.contains("dpi")));
        let unsigned = ten(&|r: &PdfRow| !r.signed);
        let sig_bad = ten(&|r: &PdfRow| r.sig_valid == Some(false));
        let no_tsa = ten(&|r: &PdfRow| r.signed && !r.tsa);
        let no_ltv = ten(&|r: &PdfRow| r.signed && !r.ltv);
        let no_bind = ten(&|r: &PdfRow| r.problems.iter().any(|x| x.contains("ESS signing-certificate")));
        if !not_pdfa.is_empty() {
            c.err_ds("PDF-KHONG-PDFA", "PDF", format!("{}/{n} tệp không phải PDF/A (TT05 Điều 8.2.a: PDF/A hai lớp)", not_pdfa.len()), not_pdfa);
        } else {
            c.ok("PDF", format!("{n}/{n} tệp khai PDF/A và có hồ sơ màu"));
        }
        if !no_text.is_empty() {
            c.err_ds("PDF-THIEU-LOP-CHU", "PDF", format!("{}/{n} tệp có trang thiếu lớp chữ", no_text.len()), no_text);
        } else {
            c.ok("PDF", format!("{n}/{n} tệp có lớp chữ tìm kiếm được"));
        }
        if !low_dpi.is_empty() {
            c.err_ds("PDF-DPI-THAP", "PDF", format!("{}/{n} tệp có độ phân giải thấp hơn TT05", low_dpi.len()), low_dpi);
        }
        let sig_bad_rong = sig_bad.is_empty();
        if !sig_bad.is_empty() {
            c.err_ds("KY-KHONG-TOAN-VEN", "Chữ ký số", format!("{}/{n} tệp có chữ ký KHÔNG toàn vẹn / không xác minh được", sig_bad.len()), sig_bad);
        }
        if !unsigned.is_empty() {
            c.warn_ds("KY-CHUA-KY", "Chữ ký số", format!("{}/{n} tệp chưa ký số", unsigned.len()), unsigned);
        } else if sig_bad_rong {
            c.ok("Chữ ký số", format!("{n}/{n} tệp có chữ ký số toàn vẹn"));
        }
        if !no_bind.is_empty() {
            c.warn_ds("KY-THIEU-CHUNG-THU-ESS", "Chữ ký số", format!("{} chữ ký dạng adbe.pkcs7.detached không gắn chứng thư người ký (chưa đạt PAdES baseline B-B)", no_bind.len()), no_bind);
        }
        if !no_tsa.is_empty() {
            c.warn_ds("KY-THIEU-TSA", "Chữ ký số", format!("{} chữ ký không có dấu thời gian (TSA)", no_tsa.len()), no_tsa);
        }
        if aip && !no_ltv.is_empty() {
            c.warn_ds("KY-THIEU-LTV", "Chữ ký số", format!("{} chữ ký chưa có LTV (OCSP/CRL nhúng) — gói AIP nên ký PAdES-LT/LTA", no_ltv.len()), no_ltv);
        }
    }
    // Nói thẳng phần CHƯA kiểm, trước khi gán findings — người đọc phải thấy nó nằm cùng chỗ
    // với các phát hiện khác, không phải một dòng chú thích ở đâu đó.
    res.chua_kiem_noi_dung = bo_qua.tong();
    res.bo_qua = bo_qua;
    if res.chua_kiem_noi_dung > 0 {
        // Câu này dựng TỪ `bo_qua`, không viết cứng tên nhóm: thêm nhóm thứ tư thì câu tự đổi
        // theo, không để lại một dòng chữ nói thiếu.
        let ten: Vec<&str> = res.bo_qua.danh_sach().iter().map(|(_, t, _, _)| *t).collect();
        // Mã để bên nhận GỘP dòng này vào khối "chuaKiem[]" thay vì đếm nó như một cảnh báo
        // thứ hai cùng nghĩa — trên màn hình chữ "Chưa kiểm" đang hiện hai lần.
        c.warn(
            "CHUA-KIEM-NOI-DUNG",
            "Chưa kiểm",
            format!(
                "{} phép kiểm cần nội dung tệp chưa chạy ({}) — bản kê chỉ mang siêu dữ liệu. Dùng «Tự kiểm PDF/A» cho tệp mẫu, và bản kiểm đầy đủ chạy khi nộp gói thật.",
                res.chua_kiem_noi_dung,
                ten.join("; ")
            ),
        );
    }
    res.findings = c.f;

    // Nội dung để nhập: EAD hồ sơ, từng EAD văn bản ghép tệp dữ liệu.
    let ho_so = ead_path.and_then(|p| parsed.get(p)).map(fields_of).unwrap_or_default();
    let mut cap: HashMap<String, String> = HashMap::new();
    for mp in mets_paths.iter().skip(1) {
        if let Some(m) = parsed.get(mp.as_str()) {
            cap.extend(cap_ead_tep(m, dir_of(mp)));
        }
    }
    let mut docs: Vec<DocExtract> = doc_eads
        .iter()
        .map(|(p, e)| {
            let fields = fields_of(e);
            let data_path = cap.get(*p).filter(|t| files.contains_key(*t)).cloned().or_else(|| {
                let code = fields.get("arcDocCode").or_else(|| fields.get("docCode")).filter(|c| !c.is_empty())?;
                data.iter().map(|(dp, _)| dp.as_str()).find(|dp| {
                    let n = dp.rsplit('/').next().unwrap_or(dp);
                    n.rsplit_once('.').map(|x| x.0).unwrap_or(n) == code
                }).map(str::to_string)
            });
            DocExtract { ead_path: (*p).to_string(), fields, data_path }
        })
        .collect();
    docs.sort_by(|a, b| a.ead_path.cmp(&b.ead_path));
    drop(cap);
    drop(doc_eads);
    drop(data);
    drop(parsed);
    let files = loaded.files;
    (finish(res), Some(Extract { files, ho_so, docs }))
}

fn finish(mut res: PackageResult) -> PackageResult {
    res.errors = res.findings.iter().filter(|f| f.level == Level::Err).count();
    res.warnings = res.findings.iter().filter(|f| f.level == Level::Warn).count();
    // Lỗi thì luôn là KHÔNG ĐẠT, dù kiểm đủ hay không. Còn khi không có lỗi mà vẫn còn phép
    // kiểm chưa chạy thì **không được** nói "ĐẠT": người đọc sẽ hiểu là gói nộp được, trong khi
    // nội dung từng tệp PDF và mã băm chưa ai xem.
    res.verdict = match (res.errors > 0, res.chua_kiem_noi_dung > 0, res.warnings > 0) {
        (true, _, _) => "KHÔNG ĐẠT".into(),
        (false, true, true) => "CHƯA KIỂM ĐỦ (có cảnh báo)".into(),
        (false, true, false) => "CHƯA KIỂM ĐỦ".into(),
        (false, false, true) => "ĐẠT (có cảnh báo)".into(),
        (false, false, false) => "ĐẠT".into(),
    };
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mọi mã phát hiện phải có hướng dẫn sửa — bên nhận dựng cột «Cách sửa» từ bảng đó, thiếu
    /// một mã là một ô trống trên màn hình nhà thầu.
    ///
    /// Bài này **đọc chính mã nguồn** `lib.rs` thay vì đọc một danh sách viết tay, nên nó bắt
    /// được cả mã mới thêm vào ngày mai. Danh sách viết tay thì chính nó cũng là luật hai nơi.
    /// Hình dáng một mã: chỉ HOA / số / gạch ngang. Tham số thứ hai là *nhóm* ("METS",
    /// "Chữ ký số") nên có chữ thường hoặc dấu cách — nhờ đó phân biệt được hai tham số mà
    /// không phải phân tích cú pháp Rust.
    fn la_ma(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '-')
    }

    /// Quét mọi mã mà `lib.rs` thật sự sinh ra. Chịu được cả lời gọi một dòng và lời gọi
    /// nhiều dòng (mã xuống hàng riêng) — đây đúng là chỗ phép quét đầu tiên đã hụt.
    fn ma_trong_nguon() -> std::collections::BTreeSet<&'static str> {
        let mut thay = std::collections::BTreeSet::new();
        let nguon = include_str!("lib.rs");
        for goi in ["c.warn(", "c.err(", "c.warn_ds(", "c.err_ds("] {
            for doan in nguon.split(goi).skip(1) {
                let doan = doan.trim_start();
                let Some(doan) = doan.strip_prefix('"') else { continue };
                let Some(ma) = doan.split('"').next() else { continue };
                if la_ma(ma) {
                    thay.insert(ma);
                }
            }
        }
        // Phép kiểm cả đợt dựng `Finding` bằng tay nên không đi qua `Ctx`. Chỉ lấy mã mức
        // `Warn`/`Err`: hai mã mức `Ok` ở đó không cần hướng dẫn sửa.
        for doan in include_str!("dot.rs").split("Finding::new(").skip(1) {
            let Some(muc) = doan.split("Level::").nth(1).and_then(|x| x.split([',', ')', ' ']).next()) else { continue };
            if muc != "Warn" && muc != "Err" {
                continue;
            }
            let Some(ma) = doan.split(".ve(\"").nth(1).and_then(|x| x.split('"').next()) else { continue };
            if la_ma(ma) {
                thay.insert(ma);
            }
        }
        thay
    }

    #[test]
    fn moi_ma_deu_co_huong_dan() {
        let thay = ma_trong_nguon();
        assert!(thay.len() >= 61, "quét mã nguồn chỉ thấy {} mã — phép quét hỏng, không phải mã nguồn hết mã", thay.len());
        let thieu: Vec<&&str> = thay.iter().filter(|ma| huong_dan(ma).is_none()).collect();
        assert!(thieu.is_empty(), "{} mã chưa có hướng dẫn trong huong_dan.rs: {thieu:?}", thieu.len());
        for ma in &thay {
            let h = huong_dan(ma).expect(ma);
            assert!(!h.tieu_de.is_empty() && !h.cach_sua.is_empty(), "{ma}: hướng dẫn rỗng");
            // Tiêu đề viết cho người đi sửa gói, không phải cho người đọc mã nguồn.
            assert!(h.cach_sua.len() > h.tieu_de.len(), "{ma}: cách sửa phải nói được việc cần làm, không chỉ nhắc lại tiêu đề");
        }
    }

    /// Chiều ngược: bảng hướng dẫn không được chứa mã đã bỏ khỏi mã nguồn. Mã chết trong bảng
    /// thì không ai biết để xoá, rồi tài liệu gửi nhà thầu liệt kê mã không bao giờ xuất hiện.
    #[test]
    fn bang_huong_dan_khong_co_ma_chet() {
        let sinh_ra = ma_trong_nguon();
        let bang = include_str!("huong_dan.rs");
        let mut co_bang = 0;
        let mut chet = Vec::new();
        for doan in bang.split("\n        \"").skip(1) {
            let Some(ma) = doan.split('"').next() else { continue };
            if !la_ma(ma) {
                continue;
            }
            co_bang += 1;
            if !sinh_ra.contains(ma) {
                chet.push(ma);
            }
        }
        assert!(co_bang >= 61, "quét bảng chỉ thấy {co_bang} mã — phép quét hỏng");
        assert!(chet.is_empty(), "mã có hướng dẫn nhưng không chỗ nào sinh ra: {chet:?}");
    }

    /// Cảnh báo "chưa kiểm" **phải** mang cờ `nop_that_thanh_loi`, các cảnh báo khác thì không.
    ///
    /// Cờ này là chỗ bên nhận quyết định hiện «không chặn» hay «sẽ bị trả lúc giao». Đặt sai
    /// thì nhà thầu giao gói rồi mới biết bị trả.
    #[test]
    fn chi_chua_kiem_moi_co_the_thanh_loi_luc_nop_that() {
        let h = huong_dan("CHUA-KIEM-NOI-DUNG").expect("phải có hướng dẫn");
        assert!(h.nop_that_thanh_loi, "mã này nói phép kiểm CHƯA CHẠY, không nói đã đạt");
        for ma in ["METS-PROFILE-KHONG-CSIP", "EAD-TRICH-YEU-TRUNG", "KY-THIEU-TSA", "PREMIS-FORMATNAME-CHUNG"] {
            let h = huong_dan(ma).expect(ma);
            assert!(!h.nop_that_thanh_loi, "{ma}: cảnh báo này không phụ thuộc nội dung tệp, đủ byte cũng không thành lỗi");
        }
    }

    /// Chạy THẬT một gói qua bộ kiểm: mọi phát hiện phải có mã kèm hướng dẫn, và mấy câu
    /// **gộp số** phải nói được chỗ bị ảnh hưởng.
    ///
    /// Hai test quét mã nguồn ở trên chỉ chứng minh bảng hướng dẫn kín. Bài này chứng minh
    /// thứ khác: lúc chạy thật, `doi_tuong` có dữ liệu — chứ không phải một `Vec` rỗng mà
    /// cổng vẽ ra cái thẻ «Chỗ bị ảnh hưởng» trắng trơn.
    #[test]
    fn phat_hien_gop_so_phai_noi_ro_cho_bi_anh_huong() {
        let goc = "urn_H49_uuid_ABC";
        let mets = br#"<mets OBJID="urn_H49_uuid_ABC" PROFILE="https://earkcsip.dilcis.eu/profile/E-ARK-CSIP.xml"><metsHdr OAISPACKAGETYPE="AIP" CREATEDATE="2026-11-17"/><structMap LABEL="CSIP"/></mets>"#;
        let vb = |subject: &str, ngay: &str| {
            let s = if subject.is_empty() { String::new() } else { format!("<subject>{subject}</subject>") };
            format!("<eadDoc><arcDocCode>H49.101.01.1993.001.0000001</arcDocCode>{s}<issuedDate>{ngay}</issuedDate></eadDoc>").into_bytes()
        };
        let p = |x: &str| format!("{goc}/{x}");
        let l = tu_ban_ke(
            vec![
                (p("METS.xml"), Some(mets.to_vec()), None),
                (p("representations/rep1/METS.xml"), Some(mets.to_vec()), None),
                (p("metadata/descriptive/EAD.xml"), Some(r#"<eadFile><arcFileCode>H49.101.01.1993.001</arcFileCode><title>Hồ sơ thử</title></eadFile>"#.as_bytes().to_vec()), None),
                // Hai văn bản trùng trích yếu, một văn bản bỏ trống, một ngày chỉ ghi năm.
                (p("representations/rep1/metadata/descriptive/d1.xml"), Some(vb("Trùng nhau", "2012")), None),
                (p("representations/rep1/metadata/descriptive/d2.xml"), Some(vb("Trùng nhau", "09/07/2012")), None),
                (p("representations/rep1/metadata/descriptive/d3.xml"), Some(vb("", "09/07/2012")), None),
                (p("representations/rep1/data/a.pdf"), None, Some(1_351_408)),
            ],
            9_000_000,
        );
        let (r, _) = check_loaded(PackageResult { kind: "?".into(), ..Default::default() }, l);

        // 1. Không một phát hiện nào được thiếu mã hoặc thiếu hướng dẫn — đây là điều kiện để
        //    cổng dựng nổi cột «Cách sửa».
        for f in r.findings.iter().filter(|f| f.level != Level::Ok) {
            assert!(!f.code.is_empty(), "thiếu mã: {} / {}", f.group, f.message);
            assert!(f.huong_dan().is_some(), "mã {} chưa có hướng dẫn", f.code);
        }

        let lay = |ma: &str| {
            r.findings.iter().find(|f| f.code == ma).unwrap_or_else(|| {
                panic!("không thấy mã {ma}; có: {:?}", r.findings.iter().map(|f| f.code.as_str()).collect::<Vec<_>>())
            })
        };
        // 2. Trùng trích yếu: đúng HAI văn bản trùng nhau, không phải cả ba.
        assert_eq!(lay("EAD-TRICH-YEU-TRUNG").doi_tuong, vec!["d1.xml", "d2.xml"]);
        // 3. Thiếu trích yếu: đúng văn bản bỏ trống.
        assert_eq!(lay("EAD-THIEU-TRICH-YEU").doi_tuong, vec!["d3.xml"]);
        // 4. Ngày sai định dạng: chỉ bản ghi "2012", không kéo theo hai bản ghi đúng.
        assert_eq!(lay("EAD-NGAY-SAI-DINH-DANG").doi_tuong, vec!["d1.xml"]);
        // 5. Tệp mồ côi: câu chữ cắt ở 8 tệp, nhưng `doi_tuong` phải đủ để sửa.
        let mo_coi = lay("METS-TEP-MO-COI");
        assert_eq!(mo_coi.doi_tuong.len(), 6, "6 tệp ngoài METS.xml gốc đều mồ côi: {:?}", mo_coi.doi_tuong);
        // 6. "Chưa kiểm" mang cờ để cổng nói «sẽ bị trả lúc giao», không nói «không chặn».
        let ck = lay("CHUA-KIEM-NOI-DUNG");
        assert_eq!(ck.level, Level::Warn);
        assert!(ck.huong_dan().expect("có hướng dẫn").nop_that_thanh_loi);
    }

    /// Danh sách chỗ bị ảnh hưởng phải **cắt** và nói rõ đã cắt bao nhiêu — một gói sai quy ước
    /// đặt tên cho ra hàng nghìn tệp cùng một mã, gửi hết thì thân phản hồi phình vô ích.
    #[test]
    fn danh_sach_cho_bi_anh_huong_bi_cat_va_noi_ro_da_cat()
    {
        let ds: Vec<String> = (0..DOI_TUONG_TOI_DA + 37).map(|i| format!("t{i}.pdf")).collect();
        let f = gan_ds(Finding::new(Level::Err, "PDF", "nhiều".into()), ds);
        assert_eq!(f.doi_tuong.len(), DOI_TUONG_TOI_DA);
        assert_eq!(f.doi_tuong_bi_cat, 37);
        // Đủ chỗ thì KHÔNG được hiện "đã cắt 0" — câu vô nghĩa trên màn hình.
        let f = gan_ds(Finding::new(Level::Err, "PDF", "ít".into()), vec!["a.pdf".into()]);
        assert_eq!(f.doi_tuong_bi_cat, 0);
    }

    /// Kết luận **không được** nói "ĐẠT" khi còn phép kiểm chưa chạy. Đây là chỗ một dòng mã
    /// sai biến "gói chưa ai xem nội dung" thành "gói nộp được".
    #[test]
    fn bo_qua_tach_nhom_khong_lech_tong_va_khong_bo_sot_nhom() {
        // Mẫu KHÔNG có `..`: thêm trường vào BoQua mà quên `danh_sach()` là lỗi biên dịch,
        // không phải một dòng thiếu trên màn hình nhà thầu.
        let BoQua { pdfa, bam_mets, bam_premis } = BoQua { pdfa: 7, bam_mets: 7, bam_premis: 7 };
        let b = BoQua { pdfa, bam_mets, bam_premis };
        assert_eq!(b.tong(), 21);
        let ds = b.danh_sach();
        assert_eq!(ds.len(), 3, "ba nhóm đều khác 0 thì phải ra ba dòng");
        assert_eq!(ds.iter().map(|(_, _, so, _)| so).sum::<usize>(), b.tong(), "tổng danh sách lệch tổng");
        for (ma, ten, _, o_dau) in &ds {
            assert!(!ma.is_empty() && !ten.is_empty() && !o_dau.is_empty(), "{ma}: thiếu nhãn");
        }
        // Nhóm bằng 0 thì không được hiện — "0 phép kiểm chưa chạy" là câu vô nghĩa.
        let ds = BoQua { pdfa: 7, ..Default::default() }.danh_sach();
        assert_eq!(ds.len(), 1);
        assert_eq!(ds[0].0, "pdfa");
        assert!(BoQua::default().danh_sach().is_empty());
    }

    #[test]
    fn ket_luan_khong_noi_dat_khi_con_phep_kiem_bi_bo() {
        let dung = |loi: usize, canh_bao: usize, bo_qua: usize| {
            let mut r = PackageResult { chua_kiem_noi_dung: bo_qua, ..Default::default() };
            for _ in 0..loi {
                r.findings.push(Finding::new(Level::Err, "t", "x".into()));
            }
            for _ in 0..canh_bao {
                r.findings.push(Finding::new(Level::Warn, "t", "x".into()));
            }
            finish(r).verdict
        };
        assert_eq!(dung(0, 0, 0), "ĐẠT");
        assert_eq!(dung(0, 1, 0), "ĐẠT (có cảnh báo)");
        assert_eq!(dung(1, 0, 0), "KHÔNG ĐẠT");
        assert_eq!(dung(0, 0, 5), "CHƯA KIỂM ĐỦ");
        assert_eq!(dung(0, 1, 5), "CHƯA KIỂM ĐỦ (có cảnh báo)");
        // Có lỗi thì vẫn là KHÔNG ĐẠT, kiểm thiếu không làm nhẹ đi.
        assert_eq!(dung(1, 0, 5), "KHÔNG ĐẠT");
        for v in [dung(0, 0, 5), dung(0, 1, 5)] {
            assert!(!v.starts_with("ĐẠT"), "{v} đọc ra là đạt");
        }
    }

    /// Nhà thầu chọn nhầm thư mục BỌC NGOÀI gói (`AIP/` chứ không phải `AIP/urn_…/`).
    ///
    /// `split_root` cắt đúng một lớp, nên không có bước gỡ lớp bọc thì `METS.xml` tụt xuống sâu
    /// một bậc và cả phép kiểm sụp — một gói hoàn toàn đúng bị báo hỏng. DO365 hỏi đúng chỗ này
    /// ngày 13/11/2026 trước khi code màn chọn thư mục.
    #[test]
    fn chon_nham_thu_muc_boc_ngoai_van_kiem_dung_goi() {
        let goc = "urn_H49_uuid_ABC";
        let dung = |tien_to: &str| {
            let p = |x: &str| if tien_to.is_empty() { format!("{goc}/{x}") } else { format!("{tien_to}/{goc}/{x}") };
            tu_ban_ke(
                vec![
                    (p("METS.xml"), Some(b"<x/>".to_vec()), None),
                    (p("representations/rep1/METS.xml"), Some(b"<x/>".to_vec()), None),
                    (p("representations/rep1/data/a.pdf"), None, Some(1_351_408)),
                ],
                131_723_288,
            )
        };
        let thang = dung("");
        for boc in ["AIP", "Desktop/AIP", "D/giao/dot3/AIP"] {
            let l = dung(boc);
            assert_eq!(l.root, goc, "bọc {boc:?}: phải nhận ra thư mục gốc thật");
            assert_eq!(
                l.files.keys().collect::<Vec<_>>(),
                thang.files.keys().collect::<Vec<_>>(),
                "bọc {boc:?}: phải ra đúng tập đường dẫn như chọn đúng thư mục"
            );
            // Tập "thiếu nội dung" và bảng cỡ cũng phải cắt theo, kẻo phép kiểm SIZE tra trượt.
            assert_eq!(l.thieu_noi_dung, thang.thieu_noi_dung, "bọc {boc:?}: tập thiếu lệch");
            assert_eq!(l.co_tep, thang.co_tep, "bọc {boc:?}: bảng cỡ lệch");
            assert!(
                l.notes.iter().any(|n| n.message.contains("lớp thư mục bọc ngoài")),
                "bọc {boc:?}: phải NÓI là đã bỏ lớp bọc, đừng âm thầm"
            );
        }
        // Chọn thư mục chứa HAI gói: gỡ lớp bọc rồi để `split_root` nói thẳng là có hai gốc —
        // đừng đoán lấy gói nào. Câu đó gọi đúng tên hai thư mục nên nhà thầu biết phải chọn lại.
        let hai = tu_ban_ke(
            vec![
                ("AIP/urn_A/METS.xml".into(), Some(b"<x/>".to_vec()), None),
                ("AIP/urn_B/METS.xml".into(), Some(b"<x/>".to_vec()), None),
            ],
            0,
        );
        assert_eq!(hai.root, "", "hai gói thì không có thư mục gốc nào là đúng");
        let noi = hai.notes.iter().map(|n| n.message.as_str()).collect::<Vec<_>>().join(" | ");
        assert!(noi.contains("2 thư mục gốc") && noi.contains("urn_A") && noi.contains("urn_B"), "{noi}");
    }

    /// `tu_ban_ke` phải cắt tên thư mục gốc khỏi tập "thiếu nội dung" y như `split_root` cắt khỏi
    /// đường dẫn tệp. Lệch một bên thì mọi phép tra đều trượt, và bộ kiểm quay lại băm mảng rỗng.
    #[test]
    fn ban_ke_cat_thu_muc_goc_khoi_tap_thieu_noi_dung() {
        let goc = "urn_H49_uuid_ABC";
        let l = tu_ban_ke(
            vec![
                (format!("{goc}/METS.xml"), Some(b"<x/>".to_vec()), None),
                (format!("{goc}/metadata/preservation/PREMIS.xml"), Some(b"<x/>".to_vec()), None),
                (format!("{goc}/representations/rep1/data/a.pdf"), None, Some(1_351_408)),
                (format!("{goc}/representations/rep1/data/b.pdf"), None, Some(2_000)),
            ],
            131_723_288,
        );
        assert_eq!(l.root, goc);
        assert!(l.files.contains_key("METS.xml"), "đường dẫn phải đã cắt gốc: {:?}", l.files.keys());
        assert_eq!(l.size, 131_723_288);
        assert_eq!(
            l.thieu_noi_dung.iter().cloned().collect::<Vec<_>>(),
            vec!["representations/rep1/data/a.pdf", "representations/rep1/data/b.pdf"]
        );
        // Tệp siêu dữ liệu KHÔNG được nằm trong tập thiếu — nếu không thì mã băm của chính
        // METS/PREMIS cũng bị bỏ qua oan.
        assert!(!l.thieu_noi_dung.contains("METS.xml"));
        // Cỡ của tệp KHÔNG gửi nội dung phải lấy từ bản kê, nếu không phép kiểm SIZE của METS
        // sẽ so với 0 và báo lỗi giả cho mọi tệp (đã sập một lần, 13/11/2026).
        assert_eq!(l.co_tep.get("representations/rep1/data/a.pdf"), Some(&1_351_408));
        assert_eq!(l.co_tep.get("METS.xml"), Some(&4), "tệp có nội dung thì lấy cỡ thật");
    }

    /// Đối chiếu **gói thật** với **bản rút gọn của chính nó**: mọi phát hiện về cấu trúc phải
    /// giống hệt, chỉ khác đúng phần cần nội dung tệp.
    ///   $env:DIGITEXT_PKG_SAMPLE="D:\\DigiText\\test\\AIP.zip"
    ///   cargo test -p digitext-pkgcheck rut_gon_khop_goi_that -- --ignored --nocapture
    #[test]
    #[ignore]
    fn rut_gon_khop_goi_that() {
        let p = std::env::var("DIGITEXT_PKG_SAMPLE").expect("đặt DIGITEXT_PKG_SAMPLE");
        let goi = std::path::PathBuf::from(&p);
        let day_du = check_path(&goi);

        // Dựng bản rút gọn ĐÚNG như cổng sẽ dựng: giữ nội dung tệp .xml/.xsd, còn lại chỉ tên.
        let l = load::load(&goi).expect("đọc được gói");
        let tep: Vec<(String, Option<Vec<u8>>, Option<u64>)> = l
            .files
            .iter()
            .map(|(k, v)| {
                let low = k.to_lowercase();
                let giu = low.ends_with(".xml") || low.ends_with(".xsd");
                let duong = if l.root.is_empty() { k.clone() } else { format!("{}/{}", l.root, k) };
                (duong, giu.then(|| v.clone()), Some(v.len() as u64))
            })
            .collect();
        let n_thieu = tep.iter().filter(|(_, b, _)| b.is_none()).count();
        let co_goi = l.size;
        let rut = check_loaded(
            PackageResult { source: "rut-gon".into(), name: "rut-gon".into(), kind: "?".into(), ..Default::default() },
            tu_ban_ke(tep, co_goi),
        )
        .0;

        println!("đầy đủ : {} · lỗi {} · cảnh báo {}", day_du.verdict, day_du.errors, day_du.warnings);
        println!("rút gọn: {} · lỗi {} · cảnh báo {} · bỏ qua {} {:?}", rut.verdict, rut.errors, rut.warnings, rut.chua_kiem_noi_dung, rut.bo_qua);
        assert_eq!(rut.bo_qua.tong(), rut.chua_kiem_noi_dung, "tổng theo nhóm lệch số tổng");
        assert!(!rut.bo_qua.danh_sach().is_empty(), "bỏ qua phép kiểm mà không nói nhóm nào");
        for f in rut.findings.iter().filter(|f| f.level != Level::Ok) {
            println!("  {:?} [{}] {}", f.level, f.group, f.message);
        }

        assert!(n_thieu > 0, "gói mẫu phải có tệp nội dung");
        assert_eq!(rut.kind, day_du.kind);
        assert_eq!(rut.objid, day_du.objid);
        assert_eq!(rut.data_files, day_du.data_files);
        // Lỗi CẤU TRÚC phải khớp từng câu. Lỗi về mã băm thì bản rút gọn không có.
        let loi_cau_truc = |r: &PackageResult| -> Vec<String> {
            r.findings
                .iter()
                .filter(|f| f.level == Level::Err && f.group != "PDF" && !f.message.contains("mã băm"))
                .map(|f| format!("[{}] {}", f.group, f.message))
                .collect()
        };
        assert_eq!(loi_cau_truc(&rut), loi_cau_truc(&day_du), "lỗi cấu trúc phải giống hệt gói đầy đủ");
        assert!(rut.chua_kiem_noi_dung > 0, "phải đếm được phép kiểm đã bỏ");
        assert!(rut.pdfs.is_empty(), "bản rút gọn không kiểm được PDF nào");
        assert!(!rut.verdict.starts_with("ĐẠT"), "kết luận {} không được đọc ra là đạt", rut.verdict);
    }

    #[test]
    fn ghep_href() {
        assert_eq!(resolve("representations/rep1", "data/a%20b.pdf"), "representations/rep1/data/a b.pdf");
        assert_eq!(resolve("representations/rep1", "../../schemas/x.xsd"), "schemas/x.xsd");
        assert_eq!(resolve("", "./METS.xml"), "METS.xml");
        assert!(is_dd_mm_yyyy("01/02/2013") && !is_dd_mm_yyyy("2013"));
    }

    /// `DIGITEXT_PKG_SAMPLE=<gói.zip hoặc thư mục> cargo test -p digitext-pkgcheck thu_goi_that -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn thu_goi_that() {
        let p = std::env::var("DIGITEXT_PKG_SAMPLE").unwrap();
        let mut all = Vec::new();
        for g in discover(&[p]) {
            let r = check_path(&g);
            all.push(r.clone());
            println!("== {} · {} · {} · lỗi {} cảnh báo {}", r.name, r.kind, r.verdict, r.errors, r.warnings);
            for f in &r.findings {
                println!("  {:?} [{}] {}", f.level, f.group, f.message);
            }
            for p in &r.pdfs {
                println!("  PDF {} · {} tr · {:?} · ký {:?} {:?} {:?} tsa={} ltv={} · {} · {:?}", p.file, p.pages, p.pdfa, p.sig_valid, p.signer, p.signing_time, p.tsa, p.ltv, p.verdict, p.problems);
            }
        }
        if let (Ok(out), Some(font)) = (std::env::var("DIGITEXT_PKG_REPORT"), digitext_pdf::sans_font().and_then(|f| std::fs::read(f).ok())) {
            let h = report::ReportHeader { org: "Công ty DigiText".into(), place: "Hà Nội".into(), inspector: "Nguyễn Văn A".into(), ..Default::default() };
            std::fs::write(&out, report::render(&h, &all, &font).unwrap()).unwrap();
            println!("báo cáo: {out}");
        }
    }

    /// `DIGITEXT_PKG_SAMPLE=<gói.zip> cargo test -p digitext-pkgcheck thu_doc_noi_dung -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn thu_doc_noi_dung() {
        let p = std::env::var("DIGITEXT_PKG_SAMPLE").unwrap();
        let (r, ex) = check_and_extract(std::path::Path::new(&p));
        let ex = ex.expect("đọc được gói");
        println!("{} · {} · {} · hồ sơ {:?}", r.kind, r.objid, r.verdict, ex.ho_so);
        for f in r.findings.iter().filter(|f| f.level == Level::Err) {
            println!("  LỖI [{}] {}", f.group, f.message);
        }
        let co_tep = ex.docs.iter().filter(|d| d.data_path.is_some()).count();
        println!("{} văn bản, {co_tep} có tệp", ex.docs.len());
        for d in ex.docs.iter().take(3) {
            println!("  {} → {:?} · {:?}", d.ead_path, d.data_path, d.fields);
        }
        assert_eq!(co_tep, ex.docs.len());
    }

    #[test]
    fn bao_cao_dung_duoc_khi_khong_co_goi() {
        let Some(font) = digitext_pdf::sans_font().and_then(|f| std::fs::read(f).ok()) else { return };
        let r = check_path(std::path::Path::new("khong-ton-tai.zip"));
        assert_eq!(r.verdict, "KHÔNG ĐẠT");
        let pdf = report::render(&report::ReportHeader::default(), &[r], &font).unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
    }
}
