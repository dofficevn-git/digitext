// Copyright (c) 2026 DigiText. All rights reserved.
//! Kiểm tra gói tin SIP/AIP/DIP của phần mềm khác theo TT 05/2025/TT-BNV + E-ARK CSIP. Cùng các bước với
//! `ml/tools/check_tt05_package.py` (bản kiểm gói TKT 19/9 và 9/10/2026), cộng xác minh chữ ký số.
//! XSD: không có bộ kiểm XSD trong Rust — đối chiếu RÚT GỌN với schema đi kèm gói (đủ trường bắt buộc,
//! không có trường lạ), chưa kiểm kiểu dữ liệu từng trường.
mod load;
pub mod report;
mod xml;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use xml::El;

pub use load::discover;

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
}
impl Finding {
    pub fn new(level: Level, group: &str, message: String) -> Self {
        Finding { level, group: group.into(), message }
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
    /// "ĐẠT" | "ĐẠT (có cảnh báo)" | "KHÔNG ĐẠT"
    pub verdict: String,
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
    fn ok(&mut self, g: &str, m: impl Into<String>) {
        self.f.push(Finding::new(Level::Ok, g, m.into()));
    }
    fn warn(&mut self, g: &str, m: impl Into<String>) {
        self.f.push(Finding::new(Level::Warn, g, m.into()));
    }
    fn err(&mut self, g: &str, m: impl Into<String>) {
        self.f.push(Finding::new(Level::Err, g, m.into()));
    }
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

fn check_codes(c: &mut Ctx, label: &str, e: &El, date_bad: &mut BTreeMap<String, usize>) {
    for (k, max) in CODES {
        let Some(v) = e.child_text(k) else { continue };
        if v.is_empty() {
            if !OPTIONAL_CODES.contains(&k) {
                c.err("EAD", format!("{label}: <{k}> rỗng"));
            }
            continue;
        }
        let ok = v.len() == 2 && v.parse::<u32>().is_ok_and(|n| (1..=max).contains(&n));
        if !ok {
            c.err("EAD", format!("{label}: <{k}>={v} ngoài bảng mã 01..{max:02}"));
        }
    }
    for k in ["startDate", "endDate", "issuedDate"] {
        if let Some(v) = e.child_text(k).filter(|v| !v.is_empty()) {
            if !is_dd_mm_yyyy(&v) {
                *date_bad.entry(k.to_string()).or_default() += 1;
            }
        }
    }
    if let Some(v) = e.child_text("riskRecovery").filter(|v| !v.is_empty()) {
        if v != "0" && v != "1" {
            c.err("EAD", format!("{label}: <riskRecovery>={v} (chỉ 0/1)"));
        } else if v == "0" && e.child_text("riskRecoveryStatus").is_some_and(|s| !s.is_empty()) {
            *date_bad.entry("#riskRecovery".into()).or_default() += 1;
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
        match digitext_sign_verify::verify(bytes) {
            Ok(t) => {
                // Toàn vẹn = chữ ký đúng + băm nội dung khớp. Thiếu thuộc tính ESS signing-certificate
                // (cert_binding) chỉ là chưa đạt PAdES baseline — chữ ký adbe.pkcs7.detached của iTextSharp…
                let intact = t.signature_ok && t.digest_ok;
                row.sig_valid = Some(intact);
                row.tsa = t.timestamp_present;
                row.ltv = t.has_dss;
                if !intact {
                    hard.push("chữ ký số KHÔNG toàn vẹn (tệp bị sửa sau khi ký hoặc chữ ký hỏng)".into());
                } else if !t.cert_binding_ok {
                    soft.push("chữ ký không gắn chứng thư người ký (thiếu thuộc tính ESS signing-certificate, chưa đạt PAdES baseline)".into());
                }
                row.signing_time = t.timestamp_time_unix.map(fmt_unix).or_else(|| sig_m_time(bytes));
                if !t.timestamp_present {
                    soft.push("chữ ký không có dấu thời gian (TSA)".into());
                }
                if aip && !t.has_dss {
                    soft.push("chữ ký chưa có thông tin xác minh lâu dài (LTV/DSS) — nên có với gói AIP".into());
                }
            }
            Err(e) => {
                row.sig_valid = Some(false);
                hard.push(format!("không xác minh được chữ ký: {e}"));
            }
        }
        if let Ok(s) = digitext_sign_verify::signer_info(bytes) {
            row.signer = Some(s.signer);
            row.issuer = Some(s.issuer);
            row.cert_until = Some(fmt_unix(s.not_after_unix));
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
    res.root = loaded.root.clone();
    res.sha256 = loaded.sha256.clone();
    res.size = loaded.size;
    let mut c = Ctx { f: loaded.notes };
    let files = &loaded.files;

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
                    c.err("XML", format!("{p}: XML hỏng ({e})"));
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
        c.err("Cấu trúc", "Thiếu METS.xml ở gốc gói");
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
        c.err("Cấu trúc", "Thiếu thư mục representations/ (không có bản đại diện nào)");
    }
    for r in &reps {
        if !files.contains_key(&format!("representations/{r}/METS.xml")) {
            c.err("Cấu trúc", format!("representations/{r}: thiếu METS.xml"));
        }
    }
    if !files.keys().any(|p| p.starts_with("schemas/") && p.to_lowercase().ends_with(".xsd")) {
        c.err("Cấu trúc", "Thiếu thư mục schemas/ (các tệp XSD đi kèm)");
    }

    // 3. Loại gói, OBJID, PROFILE.
    if kind.is_empty() {
        c.err("METS", "METS.xml thiếu csip:OAISPACKAGETYPE (SIP/AIP/DIP)");
    } else {
        c.ok("METS", format!("Loại gói OAIS: {}", res.kind));
    }
    if res.objid.is_empty() {
        c.err("METS", "METS.xml thiếu OBJID");
    } else if !loaded.root.is_empty() {
        let want = res.objid.replace(':', "_");
        if want == loaded.root {
            c.ok("METS", format!("OBJID {} khớp tên thư mục gốc", res.objid));
        } else {
            c.err("METS", format!("OBJID \"{}\" không khớp tên thư mục gốc \"{}\"", res.objid, loaded.root));
        }
    }
    let csip = |p: &str| p.to_lowercase().contains("e-ark-csip") || p.to_lowercase().contains("earkcsip");
    if profile.is_empty() {
        c.warn("METS", "METS.xml không khai PROFILE");
    } else if !csip(&profile) {
        c.warn("METS", format!("METS.xml dùng PROFILE \"{profile}\" — không phải hồ sơ E-ARK CSIP"));
    }
    if root_mets.all("structMap").iter().any(|s| s.attr("LABEL") == Some("CSIP")) {
        c.ok("METS", "structMap LABEL=\"CSIP\" đúng E-ARK");
    } else {
        c.err("METS", "Không có structMap LABEL=\"CSIP\"");
    }
    if res.kind == "SIP" {
        if root_mets.all("altRecordID").iter().any(|a| a.attr("TYPE") == Some("SUBMISSIONAGREEMENT")) {
            c.ok("METS", "SIP có mã thoả thuận nộp lưu (SUBMISSIONAGREEMENT)");
        } else {
            c.warn("METS", "SIP không có altRecordID SUBMISSIONAGREEMENT (thoả thuận nộp lưu)");
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
                c.warn("METS", format!("{mp} dùng PROFILE \"{p}\" khác METS gốc"));
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
                c.err("METS", format!("{mp}: mục {} thiếu xlink:href", el.attr("ID").unwrap_or("?")));
                continue;
            };
            let target = resolve(base, href);
            referenced.insert(target.clone());
            let Some(data) = files.get(&target) else {
                bad += 1;
                c.err("METS", format!("{mp}: tham chiếu tới tệp không có trong gói: {target}"));
                continue;
            };
            if let Some(t) = el.attr("CHECKSUMTYPE") {
                if !t.eq_ignore_ascii_case("SHA-256") {
                    c.warn("METS", format!("{mp}: {target} dùng {t} (nên SHA-256)"));
                }
            }
            match el.attr("CHECKSUM") {
                Some(sum) if el.attr("CHECKSUMTYPE").is_none_or(|t| t.eq_ignore_ascii_case("SHA-256")) => {
                    if !sum.eq_ignore_ascii_case(&sha256(data)) {
                        bad += 1;
                        c.err("METS", format!("{mp}: {target} sai mã băm SHA-256 (tệp đã bị thay đổi sau khi đóng gói)"));
                    }
                }
                Some(_) => {}
                None => {
                    bad += 1;
                    c.err("METS", format!("{mp}: {target} không ghi CHECKSUM"));
                }
            }
            if let Some(sz) = el.attr("SIZE") {
                if sz.parse::<u64>().ok() != Some(data.len() as u64) {
                    bad += 1;
                    c.err("METS", format!("{mp}: {target} SIZE={sz} khác thực tế {}", data.len()));
                }
            }
        }
        for p in m.all("mptr") {
            if let Some(h) = p.attr("href") {
                let t = resolve(base, h);
                referenced.insert(t.clone());
                if !files.contains_key(&t) {
                    c.err("METS", format!("{mp}: mptr trỏ tới tệp không có: {t}"));
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
        c.err("METS", format!("{} tệp không được METS nào tham chiếu: {}", orphans.len(), orphans.iter().take(8).map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
    }

    // 5. EAD hồ sơ.
    let mut date_bad: BTreeMap<String, usize> = BTreeMap::new();
    let ead_path = ["metadata/descriptive/EAD.xml", "metadata/descriptive/EAD_TL.xml"].into_iter().find(|p| parsed.contains_key(p));
    let mut file_code = String::new();
    match ead_path.and_then(|p| parsed.get(p).map(|e| (p, e))) {
        None => c.err("EAD", "Thiếu metadata/descriptive/EAD.xml (mô tả hồ sơ)"),
        Some((p, e)) => {
            check_codes(&mut c, "EAD hồ sơ", e, &mut date_bad);
            for k in ["title", "maintenance", "mode", "language", "totalDoc", "confidenceLevel"] {
                if e.child_text(k).is_none_or(|v| v.is_empty()) {
                    c.err("EAD", format!("EAD hồ sơ: thiếu <{k}>"));
                }
            }
            file_code = e.child_text("arcFileCode").filter(|v| !v.is_empty()).or_else(|| e.child_text("fileCode").filter(|v| !v.is_empty())).unwrap_or_default();
            if file_code.is_empty() {
                c.err("EAD", "EAD hồ sơ: thiếu mã hồ sơ (arcFileCode / fileCode)");
            }
            if aip && e.child_text("arcFileCode").is_none_or(|v| v.is_empty()) {
                c.warn("EAD", "Gói AIP nhưng EAD hồ sơ không có arcFileCode (mã hồ sơ lưu trữ)");
            }
            res.title = e.child_text("title").unwrap_or_default();
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
                        c.err("EAD", format!("EAD hồ sơ so với {xsd_name}: {pr}"));
                    }
                }
                None => c.warn("EAD", format!("Không có {xsd_name} để đối chiếu EAD hồ sơ")),
            }
        }
    }
    res.file_code = file_code.clone();

    // 6. EAD văn bản + tệp dữ liệu.
    let data: Vec<(&String, &Vec<u8>)> = files.iter().filter(|(p, _)| p.starts_with("representations/") && p.split('/').nth(2) == Some("data")).collect();
    res.data_files = data.len();
    if data.is_empty() {
        c.err("Cấu trúc", "Không có tệp tài liệu nào trong representations/*/data/");
    }
    let doc_eads: Vec<(&str, &El)> = parsed
        .iter()
        .filter(|(p, _)| p.starts_with("representations/") && p.contains("/metadata/descriptive/"))
        .map(|(p, e)| (*p, e))
        .collect();
    let doc_xsd = parsed.get("schemas/EAD_doc.xsd");
    let data_stems: BTreeSet<String> = data.iter().map(|(p, _)| {
        let n = p.rsplit('/').next().unwrap_or(p);
        n.rsplit_once('.').map(|x| x.0).unwrap_or(n).to_string()
    }).collect();
    let mut codes_seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut xsd_bad = 0;
    let mut pages_sum = 0u32;
    let mut no_subject = 0;
    let mut subjects: BTreeMap<String, usize> = BTreeMap::new();
    let mut type_other = 0;
    for (p, e) in &doc_eads {
        let label = p.rsplit('/').next().unwrap_or(p).to_string();
        check_codes(&mut c, &label, e, &mut date_bad);
        let code = e.child_text("arcDocCode").filter(|v| !v.is_empty()).or_else(|| e.child_text("docCode").filter(|v| !v.is_empty()));
        match &code {
            None => c.err("EAD", format!("{label}: thiếu mã văn bản (arcDocCode / docCode)")),
            Some(code) => {
                *codes_seen.entry(code.clone()).or_default() += 1;
                if !file_code.is_empty() {
                    let ok = code.strip_prefix(&format!("{file_code}.")).is_some_and(|rest| {
                        let parts: Vec<&str> = rest.split('.').collect();
                        let last = parts.last().copied().unwrap_or("");
                        last.len() == 7 && last.bytes().all(|b| b.is_ascii_digit()) && parts.len() <= 2 && (parts.len() == 1 || (1..=3).contains(&parts[0].len()))
                    });
                    if !ok {
                        c.err("EAD", format!("{label}: mã văn bản \"{code}\" không theo khuôn <mã hồ sơ>.<STT 7 số>"));
                    }
                }
                if !data_stems.contains(code) {
                    c.err("EAD", format!("{label}: không có tệp dữ liệu tên \"{code}.*\""));
                }
            }
        }
        match e.child_text("subject") {
            Some(s) if !s.is_empty() => *subjects.entry(s).or_default() += 1,
            _ => no_subject += 1,
        }
        if e.child_text("typeName").as_deref() == Some("32") {
            type_other += 1;
        }
        pages_sum += e.child_text("numberOfPage").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
        if let Some(x) = doc_xsd {
            let probs = xsd_lite(x, e);
            if !probs.is_empty() {
                xsd_bad += 1;
                c.err("EAD", format!("{label} so với schemas/EAD_doc.xsd: {}", probs.join("; ")));
            }
        }
    }
    let dups: Vec<&String> = codes_seen.iter().filter(|(_, n)| **n > 1).map(|(k, _)| k).collect();
    if !dups.is_empty() {
        c.err("EAD", format!("Mã văn bản trùng: {}", dups.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
    }
    if !doc_eads.is_empty() && xsd_bad == 0 && doc_xsd.is_some() {
        c.ok("EAD", format!("{} EAD văn bản khớp schemas/EAD_doc.xsd", doc_eads.len()));
    }
    if no_subject > 0 {
        c.warn("EAD", format!("{no_subject} văn bản không có trích yếu (subject)"));
    }
    let dup_sub: usize = subjects.values().filter(|n| **n > 1).copied().sum();
    if dup_sub > 0 {
        c.warn("EAD", format!("{dup_sub} văn bản có trích yếu trùng nhau — kiểm lại trích yếu từng văn bản"));
    }
    if type_other > 0 && type_other == doc_eads.len() {
        c.warn("EAD", format!("Cả {type_other} văn bản đều là loại 32 (Khác) — nên xác định đúng tên loại văn bản"));
    }
    for (k, n) in &date_bad {
        if k == "#riskRecovery" {
            c.warn("EAD", format!("{n} bản EAD ghi riskRecovery=0 (không dự phòng) nhưng vẫn điền riskRecoveryStatus"));
        } else {
            c.warn("EAD", format!("{n} trường <{k}> không theo dd/mm/yyyy (chỉ ghi năm hoặc sai định dạng)"));
        }
    }
    match res.total_doc {
        Some(t) if t as usize == data.len() && t as usize == doc_eads.len() => c.ok("EAD", format!("totalDoc = số tệp dữ liệu = số EAD văn bản = {t}")),
        Some(t) => c.err("EAD", format!("totalDoc={t} nhưng có {} tệp dữ liệu, {} EAD văn bản", data.len(), doc_eads.len())),
        None => {}
    }
    if let Some(p) = res.pages {
        if !doc_eads.is_empty() {
            if p == pages_sum {
                c.ok("EAD", format!("numberOfPage hồ sơ = tổng số trang các văn bản = {p}"));
            } else {
                c.warn("EAD", format!("numberOfPage hồ sơ = {p} nhưng tổng các văn bản = {pages_sum}"));
            }
        }
    }

    // 7. PREMIS.
    let premis_root = parsed.get("metadata/preservation/PREMIS.xml");
    match (premis_root, aip) {
        (None, true) => c.err("PREMIS", "Gói AIP thiếu metadata/preservation/PREMIS.xml"),
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
                c.err("PREMIS", format!("Gói AIP thiếu {pp}"));
            }
            continue;
        };
        let objs = p.all("object");
        let digests: BTreeSet<String> = p.all("messageDigest").iter().map(|d| d.text.trim().to_lowercase()).collect();
        let mut miss = 0;
        let mut no_orig = 0;
        let rep_data: Vec<&(&String, &Vec<u8>)> = data.iter().filter(|(path, _)| path.starts_with(&format!("representations/{r}/"))).collect();
        for (path, b) in &rep_data {
            let fname = path.rsplit('/').next().unwrap_or(path);
            if !digests.contains(&sha256(b)) {
                miss += 1;
                c.err("PREMIS", format!("{pp}: không có mã băm SHA-256 đúng cho {fname}"));
            }
            let has_orig = objs.iter().any(|o| o.all("originalName").iter().any(|n| n.text.trim() == fname));
            if !has_orig {
                no_orig += 1;
            }
        }
        if miss == 0 && !rep_data.is_empty() {
            c.ok("PREMIS", format!("{pp}: fixity SHA-256 khớp {}/{} tệp", rep_data.len(), rep_data.len()));
        }
        if no_orig > 0 {
            c.warn("PREMIS", format!("{pp}: {no_orig} tệp không có originalName — khó đối chiếu tự động"));
        }
        if p.all("formatName").iter().any(|f| f.text.trim().eq_ignore_ascii_case("Portable Document Format")) {
            c.warn("PREMIS", format!("{pp}: formatName chỉ ghi \"Portable Document Format\" — nên ghi PDF/A-2b (PRONOM fmt/477)"));
        }
    }

    // 8. Từng tệp PDF.
    for (p, b) in &data {
        if p.to_lowercase().ends_with(".pdf") {
            res.pdfs.push(check_pdf(p.rsplit('/').next().unwrap_or(p), b, aip));
        }
    }
    if !res.pdfs.is_empty() {
        let n = res.pdfs.len();
        let not_pdfa = res.pdfs.iter().filter(|r| r.pdfa.is_none() || !r.output_intent).count();
        let no_text = res.pdfs.iter().filter(|r| r.text_pages < r.pages).count();
        let low_dpi = res.pdfs.iter().filter(|r| r.problems.iter().any(|x| x.contains("dpi"))).count();
        let unsigned = res.pdfs.iter().filter(|r| !r.signed).count();
        let sig_bad = res.pdfs.iter().filter(|r| r.sig_valid == Some(false)).count();
        let no_tsa = res.pdfs.iter().filter(|r| r.signed && !r.tsa).count();
        let no_ltv = res.pdfs.iter().filter(|r| r.signed && !r.ltv).count();
        if not_pdfa > 0 {
            c.err("PDF", format!("{not_pdfa}/{n} tệp không phải PDF/A (TT05 Điều 8.2.a: PDF/A hai lớp)"));
        } else {
            c.ok("PDF", format!("{n}/{n} tệp khai PDF/A và có hồ sơ màu"));
        }
        if no_text > 0 {
            c.err("PDF", format!("{no_text}/{n} tệp có trang thiếu lớp chữ"));
        } else {
            c.ok("PDF", format!("{n}/{n} tệp có lớp chữ tìm kiếm được"));
        }
        if low_dpi > 0 {
            c.err("PDF", format!("{low_dpi}/{n} tệp có độ phân giải thấp hơn TT05"));
        }
        if sig_bad > 0 {
            c.err("Chữ ký số", format!("{sig_bad}/{n} tệp có chữ ký KHÔNG toàn vẹn / không xác minh được"));
        }
        if unsigned > 0 {
            c.warn("Chữ ký số", format!("{unsigned}/{n} tệp chưa ký số"));
        } else if sig_bad == 0 {
            c.ok("Chữ ký số", format!("{n}/{n} tệp có chữ ký số toàn vẹn"));
        }
        let no_bind = res.pdfs.iter().filter(|r| r.problems.iter().any(|x| x.contains("ESS signing-certificate"))).count();
        if no_bind > 0 {
            c.warn("Chữ ký số", format!("{no_bind} chữ ký dạng adbe.pkcs7.detached không gắn chứng thư người ký (chưa đạt PAdES baseline B-B)"));
        }
        if no_tsa > 0 {
            c.warn("Chữ ký số", format!("{no_tsa} chữ ký không có dấu thời gian (TSA)"));
        }
        if aip && no_ltv > 0 {
            c.warn("Chữ ký số", format!("{no_ltv} chữ ký chưa có LTV (OCSP/CRL nhúng) — gói AIP nên ký PAdES-LT/LTA"));
        }
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
    res.verdict = if res.errors > 0 {
        "KHÔNG ĐẠT".into()
    } else if res.warnings > 0 {
        "ĐẠT (có cảnh báo)".into()
    } else {
        "ĐẠT".into()
    };
    res
}

#[cfg(test)]
mod tests {
    use super::*;
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
