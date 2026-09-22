// Copyright (c) 2026 DigiText. All rights reserved.
//! KIỂM TRA PDF/A cho một tệp bất kỳ: tệp tự khai loại nào (PDF/A-1B, 2B, 3B, 2U, 1A…), có đủ
//! các dấu hiệu bắt buộc không, và — khi máy có Docker — veraPDF (bộ kiểm chuẩn ISO 19005) kết
//! luận ĐẠT/KHÔNG ĐẠT kèm luật vi phạm.
//!
//! Vì sao cần: "Ctrl+A bôi được chữ" chỉ chứng minh tệp có LỚP CHỮ (hai lớp), không chứng minh
//! là PDF/A; dải "PDF/A" của trình xem chỉ hiện khi bật tuỳ chọn. Kho Điền Xá (21/9/2026):
//! 34.354/34.354 tệp có lớp chữ + chữ ký số nhưng KHÔNG tệp nào là PDF/A.
use crate::check::{inspect_pdf, PdfFacts};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeraResult {
    /// Loại đã đem ra kiểm: "1b", "2b", "3b"…
    pub flavour: String,
    pub compliant: bool,
    /// "mục 6.2.4.3 phép thử 2 — <mô tả>"
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfaReport {
    pub path: String,
    pub name: String,
    pub bytes: u64,
    pub pdf_version: String,
    /// Tệp TỰ KHAI trong XMP: "PDF/A-2B"… `None` = không khai.
    pub declared: Option<String>,
    pub has_xmp: bool,
    pub has_output_intent: bool,
    pub pages: u32,
    pub text_pages: u32,
    pub min_dpi: Option<u32>,
    pub min_bit_depth: Option<u32>,
    pub signed: bool,
    pub producer: Option<String>,
    /// Kết luận ngắn cho người dùng.
    pub verdict: String,
    /// true = có khai PDF/A và đủ dấu hiệu cấu trúc (chưa phải kết luận ISO nếu chưa chạy veraPDF).
    pub looks_pdfa: bool,
    pub problems: Vec<String>,
    pub verapdf: Option<VeraResult>,
    pub error: Option<String>,
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Đọc `pdfaid:part` / `pdfaid:conformance` từ một khối XMP (dạng phần tử hoặc thuộc tính).
fn pdfaid_from_xmp(xmp: &[u8]) -> Option<String> {
    let value_after = |key: &[u8]| -> Option<String> {
        let at = find(xmp, key)? + key.len();
        let rest = &xmp[at..xmp.len().min(at + 40)];
        let s: String = rest
            .iter()
            .map(|b| *b as char)
            .skip_while(|c| matches!(c, '>' | '=' | '"' | '\'' | ' ' | '\n' | '\r' | '\t'))
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();
        (!s.is_empty()).then_some(s)
    };
    let part = value_after(b"pdfaid:part")?;
    let conf = value_after(b"pdfaid:conformance").unwrap_or_default();
    Some(format!("PDF/A-{part}{}", conf.to_uppercase()))
}

/// Loại PDF/A tệp tự khai. Thử trên byte thô trước (XMP thường không nén); nếu tệp có /Metadata
/// mà không thấy → mở bằng lopdf, giải nén luồng Metadata của Catalog rồi đọc lại.
pub fn declared_flavour(bytes: &[u8]) -> Option<String> {
    if let Some(f) = pdfaid_from_xmp(bytes) {
        return Some(f);
    }
    if find(bytes, b"/Metadata").is_none() {
        return None;
    }
    let doc = lopdf::Document::load_mem(bytes).ok()?;
    let cat = doc.catalog().ok()?;
    let id = cat.get(b"Metadata").ok()?.as_reference().ok()?;
    let lopdf::Object::Stream(st) = doc.get_object(id).ok()? else { return None };
    let xmp = st.decompressed_content().unwrap_or_else(|_| st.content.clone());
    pdfaid_from_xmp(&xmp)
}

fn producer_of(bytes: &[u8]) -> Option<String> {
    let at = find(bytes, b"/Producer")? + 9;
    let rest = &bytes[at..bytes.len().min(at + 200)];
    let open = rest.iter().position(|b| *b == b'(')?;
    let mut out = String::new();
    for b in &rest[open + 1..] {
        match *b {
            b')' => break,
            b'\\' => continue,
            c if c.is_ascii() && !c.is_ascii_control() => out.push(c as char),
            _ => {}
        }
    }
    let out = out.trim().to_string();
    (!out.is_empty()).then_some(out)
}

/// Soi MỘT tệp (chưa chạy veraPDF).
pub fn inspect(path: &Path) -> PdfaReport {
    let name = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let mut r = PdfaReport {
        path: path.display().to_string(), name, bytes: 0, pdf_version: String::new(), declared: None,
        has_xmp: false, has_output_intent: false, pages: 0, text_pages: 0, min_dpi: None, min_bit_depth: None,
        signed: false, producer: None, verdict: String::new(), looks_pdfa: false, problems: Vec::new(),
        verapdf: None, error: None,
    };
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            r.error = Some(format!("không đọc được tệp: {e}"));
            r.verdict = "KHÔNG ĐỌC ĐƯỢC".into();
            return r;
        }
    };
    r.bytes = bytes.len() as u64;
    r.pdf_version = bytes.iter().take(8).map(|b| *b as char).filter(|c| c.is_ascii_graphic()).collect::<String>().replace("%PDF-", "");
    let f: PdfFacts = inspect_pdf(&bytes);
    if let Some(why) = &f.encrypted_or_broken {
        r.error = Some(why.clone());
        r.verdict = "PDF HỎNG / KHÔNG MỞ ĐƯỢC".into();
        return r;
    }
    r.pages = f.pages;
    r.text_pages = f.text_pages;
    r.min_dpi = f.min_dpi;
    r.min_bit_depth = f.min_bit_depth;
    r.signed = f.signed;
    r.declared = declared_flavour(&bytes);
    r.has_xmp = find(&bytes, b"<x:xmpmeta").is_some() || find(&bytes, b"/Metadata").is_some();
    r.has_output_intent = find(&bytes, b"/OutputIntents").is_some();
    r.producer = producer_of(&bytes);

    if r.declared.is_none() {
        r.problems.push(if r.has_xmp {
            "có khối metadata nhưng KHÔNG khai pdfaid (loại PDF/A)".into()
        } else {
            "không có khối metadata XMP — tệp không khai là PDF/A (ISO 19005 mục 6.6.2.1)".into()
        });
    }
    if !r.has_output_intent {
        r.problems.push("không nhúng hồ sơ màu (OutputIntent) — dùng màu RGB/xám phụ thuộc thiết bị (mục 6.2.4.3)".into());
    }
    if r.text_pages < r.pages {
        r.problems.push(format!("{}/{} trang không có lớp chữ (TT05 đòi PDF/A HAI LỚP)", r.pages - r.text_pages, r.pages));
    }
    if let Some(d) = r.min_dpi.filter(|d| *d < digitext_pdf::FM1_MIN_DPI) {
        r.problems.push(format!("độ phân giải {d} dpi < 200 dpi (TT05 Điều 8.2.a)"));
    }
    if let Some(b) = r.min_bit_depth.filter(|b| *b < digitext_pdf::FM1_MIN_BIT_DEPTH) {
        r.problems.push(format!("ảnh {b} bit < 24 bit màu (TT05 Điều 8.2.a)"));
    }
    r.looks_pdfa = r.declared.is_some() && r.has_output_intent;
    r.verdict = match (&r.declared, r.looks_pdfa) {
        (Some(d), true) => format!("{d} (tệp tự khai, đủ dấu hiệu)"),
        (Some(d), false) => format!("khai {d} nhưng THIẾU hồ sơ màu"),
        (None, _) => "KHÔNG PHẢI PDF/A".into(),
    };
    r
}

/// Liệt kê PDF trong các đường dẫn (tệp hoặc thư mục, đệ quy), tối đa `limit`.
pub fn collect(paths: &[String], limit: usize) -> Vec<PathBuf> {
    fn walk(p: &Path, out: &mut Vec<PathBuf>, limit: usize) {
        if out.len() >= limit {
            return;
        }
        if p.is_dir() {
            let mut kids: Vec<PathBuf> = std::fs::read_dir(p).map(|rd| rd.flatten().map(|e| e.path()).collect()).unwrap_or_default();
            kids.sort();
            for k in kids {
                walk(&k, out, limit);
            }
        } else if p.extension().map_or(false, |x| x.eq_ignore_ascii_case("pdf")) {
            out.push(p.to_path_buf());
        }
    }
    let mut out = Vec::new();
    for p in paths {
        walk(Path::new(p), &mut out, limit);
    }
    out
}

fn docker() -> Command {
    let mut c = Command::new("docker");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW — không nháy cửa sổ đen khi gọi từ app
    }
    c
}

pub const VERAPDF_IMAGE: &str = "verapdf/cli:v1.30.2";

/// Máy này chạy được veraPDF qua Docker không (Docker đang bật + đã có image).
pub fn verapdf_available() -> bool {
    docker()
        .args(["image", "inspect", VERAPDF_IMAGE])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn attr<'a>(s: &'a str, key: &str) -> Option<&'a str> {
    let at = s.find(&format!("{key}=\""))? + key.len() + 2;
    let end = s[at..].find('"')? + at;
    Some(&s[at..end])
}

/// Chạy veraPDF cho các tệp CÙNG một thư mục, cùng một loại (flavour). Trả tên tệp → kết quả.
pub fn verapdf_dir(dir: &Path, names: &[String], flavour: &str) -> BTreeMap<String, VeraResult> {
    let mut out = BTreeMap::new();
    let mut cmd = docker();
    cmd.args(["run", "--rm", "-v"]).arg(format!("{}:/data:ro", dir.display())).arg(VERAPDF_IMAGE).args(["-f", flavour, "--format", "xml"]);
    for n in names {
        cmd.arg(format!("/data/{n}"));
    }
    let Ok(o) = cmd.output() else { return out };
    let xml = String::from_utf8_lossy(&o.stdout);
    for job in xml.split("<job>").skip(1) {
        let Some(a) = job.find("<name>") else { continue };
        let Some(b) = job[a..].find("</name>") else { continue };
        let full = &job[a + 6..a + b];
        let name = full.rsplit('/').next().unwrap_or(full).to_string();
        let Some(compliant) = attr(job, "isCompliant") else { continue };
        let mut rules = Vec::new();
        for rule in job.split("<rule ").skip(1) {
            let (Some(clause), Some(test)) = (attr(rule, "clause"), attr(rule, "testNumber")) else { continue };
            let desc = rule.find("<description>").and_then(|s| rule[s + 13..].find("</description>").map(|e| &rule[s + 13..s + 13 + e])).unwrap_or("");
            let line = format!("mục {clause} phép thử {test} — {desc}");
            if !rules.contains(&line) {
                rules.push(line);
            }
        }
        out.insert(name, VeraResult { flavour: flavour.to_string(), compliant: compliant == "true", rules });
    }
    out
}

/// Mọi phiên bản/mức PDF/A mà ISO 19005 định nghĩa — TT05 chỉ đòi "PDF/A hai lớp", không chỉ
/// định phiên bản, nên loại nào trong danh sách này cũng được nhận.
pub const KNOWN_FLAVOURS: &[&str] = &["1a", "1b", "2a", "2b", "2u", "3a", "3b", "3u", "4", "4e", "4f"];

/// Tệp tự khai một loại PDF/A hợp lệ ("PDF/A-3U"…) không?
pub fn is_known_flavour(declared: &Option<String>) -> bool {
    declared
        .as_deref()
        .and_then(|d| d.strip_prefix("PDF/A-"))
        .map(|s| KNOWN_FLAVOURS.contains(&s.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Xác minh ISO 19005 bằng veraPDF cho một danh sách tệp, MỖI TỆP THEO ĐÚNG LOẠI NÓ TỰ KHAI
/// (tệp không khai → kiểm theo 2b, chắc chắn rớt). Gom theo thư mục + loại, 150 tệp/lượt.
/// Trả đường dẫn → kết quả; tệp không có trong kết quả = veraPDF không chạy được.
pub fn verify_files(files: &[(PathBuf, Option<String>)]) -> BTreeMap<PathBuf, VeraResult> {
    let mut groups: BTreeMap<(PathBuf, String), Vec<&PathBuf>> = BTreeMap::new();
    for (p, declared) in files {
        let dir = p.parent().map(Path::to_path_buf).unwrap_or_default();
        groups.entry((dir, flavour_for(declared))).or_default().push(p);
    }
    let mut out = BTreeMap::new();
    for ((dir, flavour), paths) in groups {
        for chunk in paths.chunks(150) {
            let names: Vec<String> = chunk.iter().map(|p| p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()).collect();
            let res = verapdf_dir(&dir, &names, &flavour);
            for (p, n) in chunk.iter().zip(&names) {
                if let Some(v) = res.get(n) {
                    out.insert((*p).clone(), v.clone());
                }
            }
        }
    }
    out
}

/// "PDF/A-2B" → "2b". Tệp không khai → kiểm theo "2b" (loại DigiText và TT05 đang dùng).
pub fn flavour_for(declared: &Option<String>) -> String {
    declared
        .as_deref()
        .and_then(|d| d.strip_prefix("PDF/A-"))
        .map(|s| s.to_lowercase())
        .filter(|s| KNOWN_FLAVOURS.contains(&s.as_str()))
        .unwrap_or_else(|| "2b".into())
}

/// Soi danh sách tệp; `use_verapdf` thì chạy thêm veraPDF (gom theo thư mục + loại, 150 tệp/lượt).
pub fn run(files: &[PathBuf], use_verapdf: bool, progress: &(dyn Fn(usize, usize) + Sync)) -> Vec<PdfaReport> {
    let total = files.len() * if use_verapdf { 2 } else { 1 };
    let mut reports: Vec<PdfaReport> = Vec::with_capacity(files.len());
    for (i, f) in files.iter().enumerate() {
        reports.push(inspect(f));
        progress(i + 1, total);
    }
    if use_verapdf {
        let mut groups: BTreeMap<(PathBuf, String), Vec<usize>> = BTreeMap::new();
        for (i, r) in reports.iter().enumerate() {
            if r.error.is_none() {
                let dir = files[i].parent().map(Path::to_path_buf).unwrap_or_default();
                groups.entry((dir, flavour_for(&r.declared))).or_default().push(i);
            }
        }
        let mut done = files.len();
        for ((dir, flavour), idx) in groups {
            for chunk in idx.chunks(150) {
                let names: Vec<String> = chunk.iter().map(|i| reports[*i].name.clone()).collect();
                let res = verapdf_dir(&dir, &names, &flavour);
                for i in chunk {
                    if let Some(v) = res.get(&reports[*i].name) {
                        let r = &mut reports[*i];
                        r.verdict = match (&r.declared, v.compliant) {
                            (Some(d), true) => format!("{d} — veraPDF: ĐẠT"),
                            (Some(d), false) => format!("khai {d} nhưng veraPDF: KHÔNG ĐẠT"),
                            (None, _) => "KHÔNG PHẢI PDF/A — veraPDF: KHÔNG ĐẠT".into(),
                        };
                        r.looks_pdfa = v.compliant;
                        r.verapdf = Some(v.clone());
                    }
                }
                done += chunk.len();
                progress(done, total);
            }
        }
    }
    reports
}

/// Ghi bảng kết quả ra CSV (UTF-8 có BOM — mở thẳng bằng Excel). Trả đường dẫn đã ghi.
pub fn write_csv(reports: &[PdfaReport], path: &Path) -> Result<String, String> {
    let esc = |s: &str| if s.contains([',', '"', '\n', ';']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() };
    let mut out = String::from("\u{feff}Kết luận,Loại tự khai,veraPDF,Tệp,Thư mục,PDF,Trang,Trang có lớp chữ,dpi thấp nhất,Bit màu,Hồ sơ màu,Chữ ký số,Phần mềm tạo,Vấn đề,Luật veraPDF vi phạm\r\n");
    for r in reports {
        let dir = Path::new(&r.path).parent().map(|p| p.display().to_string()).unwrap_or_default();
        let vera = match &r.verapdf {
            Some(v) => format!("{} ({})", if v.compliant { "ĐẠT" } else { "KHÔNG ĐẠT" }, v.flavour),
            None => "chưa chạy".into(),
        };
        let cols = [
            r.verdict.clone(), r.declared.clone().unwrap_or_else(|| "không khai".into()), vera, r.name.clone(), dir,
            r.pdf_version.clone(), r.pages.to_string(), r.text_pages.to_string(),
            r.min_dpi.map(|d| d.to_string()).unwrap_or_default(), r.min_bit_depth.map(|d| d.to_string()).unwrap_or_default(),
            if r.has_output_intent { "có" } else { "KHÔNG" }.to_string(), if r.signed { "có" } else { "không" }.to_string(),
            r.producer.clone().unwrap_or_default(),
            r.error.clone().map(|e| vec![e]).unwrap_or_else(|| r.problems.clone()).join(" | "),
            r.verapdf.as_ref().map(|v| v.rules.join(" | ")).unwrap_or_default(),
        ];
        out.push_str(&cols.iter().map(|c| esc(c)).collect::<Vec<_>>().join(","));
        out.push_str("\r\n");
    }
    std::fs::write(path, out).map_err(|e| format!("không ghi được {}: {e}", path.display()))?;
    Ok(path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn doc_pdfaid_dang_phan_tu_va_thuoc_tinh() {
        assert_eq!(pdfaid_from_xmp(b"<pdfaid:part>2</pdfaid:part><pdfaid:conformance>B</pdfaid:conformance>").as_deref(), Some("PDF/A-2B"));
        assert_eq!(pdfaid_from_xmp(b"<rdf:Description pdfaid:part=\"1\" pdfaid:conformance=\"b\"/>").as_deref(), Some("PDF/A-1B"));
        assert_eq!(pdfaid_from_xmp(b"<pdfaid:part>4</pdfaid:part>").as_deref(), Some("PDF/A-4"));
        assert_eq!(pdfaid_from_xmp(b"%PDF-1.3 khong co gi"), None);
    }
    #[test]
    fn loai_dem_kiem_verapdf() {
        assert_eq!(flavour_for(&Some("PDF/A-1B".into())), "1b");
        assert_eq!(flavour_for(&Some("PDF/A-3U".into())), "3u");
        assert_eq!(flavour_for(&None), "2b");
        assert_eq!(flavour_for(&Some("PDF/A-9Z".into())), "2b");
    }
}
