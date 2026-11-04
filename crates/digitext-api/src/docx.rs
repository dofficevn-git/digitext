// Copyright (c) 2026 DigiText. All rights reserved.
//! Biên bản dạng Word (.docx, 4/11/2026): cùng nguồn khối với PDF (`bienban::render`), mẫu generic giữ bảng thật.
//! Tự viết WordprocessingML tối thiểu (document.xml + styles.xml) rồi nén zip — không thêm crate.
//! Trình bày theo thể thức văn bản hành chính (Nghị định 30/2020/NĐ-CP): A4, lề trái 3 cm, phải 2 cm, trên/dưới 2 cm,
//! Times New Roman 13.
use crate::bienban::{GenericBlock, GenericForm};
use digitext_pdf::FormBlock;
use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum DocBlock {
    Heading(String),
    Paragraph(String),
    Field { label: String, value: String },
    Bullets(Vec<String>),
    Table { columns: Vec<String>, rows: Vec<Vec<String>> },
    Signatures { left: Vec<String>, right: Vec<String> },
}

pub fn from_form(blocks: &[FormBlock]) -> Vec<DocBlock> {
    blocks
        .iter()
        .map(|b| match b {
            FormBlock::SectionHeading(t) => DocBlock::Heading(t.clone()),
            FormBlock::Paragraph(t) => DocBlock::Paragraph(t.clone()),
            FormBlock::Field { label, value } => DocBlock::Field { label: label.clone(), value: value.clone() },
            FormBlock::Bullets(v) => DocBlock::Bullets(v.clone()),
            FormBlock::Signatures { left, right } => DocBlock::Signatures { left: left.clone(), right: right.clone() },
        })
        .collect()
}

pub fn from_generic(f: &GenericForm) -> Vec<DocBlock> {
    f.blocks
        .iter()
        .map(|b| match b {
            GenericBlock::Heading { text } => DocBlock::Heading(text.clone()),
            GenericBlock::Paragraph { text } => DocBlock::Paragraph(text.clone()),
            GenericBlock::Field { label, value } => DocBlock::Field { label: label.clone(), value: value.clone() },
            GenericBlock::Bullets { items } => DocBlock::Bullets(items.clone()),
            GenericBlock::Table { columns, rows } => DocBlock::Table { columns: columns.clone(), rows: rows.clone() },
            GenericBlock::Signatures { left, right } => DocBlock::Signatures { left: left.clone(), right: right.clone() },
        })
        .collect()
}

/// Thoát ký tự XML và bỏ ký tự điều khiển XML 1.0 không cho phép (trừ tab / xuống dòng, xử lý riêng ở `runs`).
fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\t' | '\n' => o.push(c),
            c if (c as u32) < 0x20 => {}
            '\u{FFFE}' | '\u{FFFF}' => {}
            c => o.push(c),
        }
    }
    o
}

#[derive(Clone, Copy, Default)]
struct Fmt {
    bold: bool,
    italic: bool,
    size: Option<u32>, // nửa point
}

/// Một hay nhiều `w:r`: xuống dòng trong giá trị → `w:br`, tab → `w:tab`.
fn runs(text: &str, f: Fmt) -> String {
    let mut rpr = String::new();
    if f.bold {
        rpr.push_str("<w:b/>");
    }
    if f.italic {
        rpr.push_str("<w:i/>");
    }
    if let Some(sz) = f.size {
        rpr.push_str(&format!("<w:sz w:val=\"{sz}\"/><w:szCs w:val=\"{sz}\"/>"));
    }
    let rpr = if rpr.is_empty() { String::new() } else { format!("<w:rPr>{rpr}</w:rPr>") };
    let mut o = String::new();
    for (i, line) in text.replace("\r\n", "\n").split('\n').enumerate() {
        o.push_str("<w:r>");
        o.push_str(&rpr);
        if i > 0 {
            o.push_str("<w:br/>");
        }
        for (j, part) in line.split('\t').enumerate() {
            if j > 0 {
                o.push_str("<w:tab/>");
            }
            if !part.is_empty() {
                o.push_str(&format!("<w:t xml:space=\"preserve\">{}</w:t>", esc(part)));
            }
        }
        o.push_str("</w:r>");
    }
    o
}

#[derive(Clone, Copy, Default)]
struct Para {
    center: bool,
    justify: bool,
    before: u32,
    after: u32,
    indent: u32,
    hanging: u32,
    keep_next: bool,
}

fn para(p: Para, inner: &str) -> String {
    let mut ppr = String::new();
    if p.keep_next {
        ppr.push_str("<w:keepNext/>");
    }
    ppr.push_str(&format!("<w:spacing w:before=\"{}\" w:after=\"{}\"/>", p.before, p.after));
    if p.indent > 0 || p.hanging > 0 {
        ppr.push_str(&format!("<w:ind w:left=\"{}\" w:hanging=\"{}\"/>", p.indent, p.hanging));
    }
    if p.center {
        ppr.push_str("<w:jc w:val=\"center\"/>");
    } else if p.justify {
        ppr.push_str("<w:jc w:val=\"both\"/>");
    }
    format!("<w:p><w:pPr>{ppr}</w:pPr>{inner}</w:p>")
}

const BODY_W: u32 = 9070; // A4 11906 − lề 1701 − 1134 (twip)

fn cell(width: u32, inner: &str, shade: bool) -> String {
    let shd = if shade { "<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"EDEDED\"/>" } else { "" };
    format!("<w:tc><w:tcPr><w:tcW w:w=\"{width}\" w:type=\"dxa\"/>{shd}</w:tcPr>{inner}</w:tc>")
}

fn table(columns: &[String], rows: &[Vec<String>]) -> String {
    let ncol = columns.len().max(rows.iter().map(Vec::len).max().unwrap_or(0)).max(1);
    // Cột "STT" tự thêm ở đầu cho dễ đối chiếu.
    let stt_w = 600u32;
    let w = (BODY_W - stt_w) / ncol as u32;
    let small = Fmt { size: Some(22), ..Fmt::default() };
    let bold_small = Fmt { bold: true, size: Some(22), ..Fmt::default() };
    let pc = Para { center: true, ..Para::default() };
    let pl = Para::default();
    let mut o = String::from("<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"pct\"/><w:tblBorders>");
    for side in ["top", "left", "bottom", "right", "insideH", "insideV"] {
        o.push_str(&format!("<w:{side} w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"000000\"/>"));
    }
    o.push_str("</w:tblBorders><w:tblLayout w:type=\"fixed\"/><w:tblCellMar><w:left w:w=\"70\" w:type=\"dxa\"/><w:right w:w=\"70\" w:type=\"dxa\"/></w:tblCellMar></w:tblPr><w:tblGrid>");
    o.push_str(&format!("<w:gridCol w:w=\"{stt_w}\"/>"));
    for _ in 0..ncol {
        o.push_str(&format!("<w:gridCol w:w=\"{w}\"/>"));
    }
    o.push_str("</w:tblGrid>");
    if !columns.is_empty() {
        o.push_str("<w:tr><w:trPr><w:tblHeader/><w:cantSplit/></w:trPr>");
        o.push_str(&cell(stt_w, &para(pc, &runs("STT", bold_small)), true));
        for i in 0..ncol {
            let t = columns.get(i).map(String::as_str).unwrap_or("");
            o.push_str(&cell(w, &para(pc, &runs(t, bold_small)), true));
        }
        o.push_str("</w:tr>");
    }
    for (k, r) in rows.iter().enumerate() {
        o.push_str("<w:tr><w:trPr><w:cantSplit/></w:trPr>");
        o.push_str(&cell(stt_w, &para(pc, &runs(&(k + 1).to_string(), small)), false));
        for i in 0..ncol {
            let t = r.get(i).map(String::as_str).unwrap_or("");
            o.push_str(&cell(w, &para(pl, &runs(t, small)), false));
        }
        o.push_str("</w:tr>");
    }
    o.push_str("</w:tbl>");
    o
}

fn signatures(left: &[String], right: &[String]) -> String {
    let half = BODY_W / 2;
    let side = |lines: &[String]| -> String {
        let mut s = String::new();
        for (i, l) in lines.iter().enumerate() {
            let f = if i == 0 { Fmt { bold: true, ..Fmt::default() } } else { Fmt { italic: true, ..Fmt::default() } };
            s.push_str(&para(Para { center: true, keep_next: true, ..Para::default() }, &runs(l, f)));
        }
        // Chỗ ký.
        for _ in 0..4 {
            s.push_str(&para(Para { center: true, ..Para::default() }, ""));
        }
        if s.is_empty() {
            s = para(Para::default(), "");
        }
        s
    };
    let none = "<w:tblBorders><w:top w:val=\"nil\"/><w:left w:val=\"nil\"/><w:bottom w:val=\"nil\"/><w:right w:val=\"nil\"/><w:insideH w:val=\"nil\"/><w:insideV w:val=\"nil\"/></w:tblBorders>";
    format!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"pct\"/>{none}<w:tblLayout w:type=\"fixed\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"{half}\"/><w:gridCol w:w=\"{half}\"/></w:tblGrid><w:tr><w:trPr><w:cantSplit/></w:trPr>{}{}</w:tr></w:tbl>",
        cell(half, &side(left), false),
        cell(half, &side(right), false)
    )
}

fn document_xml(title: &str, subtitle: Option<&str>, so: Option<&str>, blocks: &[DocBlock]) -> String {
    let mut b = String::new();
    let bold = Fmt { bold: true, ..Fmt::default() };
    let center = Para { center: true, ..Para::default() };
    b.push_str(&para(center, &runs("CỘNG HOÀ XÃ HỘI CHỦ NGHĨA VIỆT NAM", Fmt { bold: true, size: Some(26), ..Fmt::default() })));
    b.push_str(&para(Para { center: true, after: 0, ..Para::default() }, &runs("Độc lập - Tự do - Hạnh phúc", Fmt { bold: true, size: Some(28), ..Fmt::default() })));
    b.push_str(&para(center, &runs("―――――――――――――", Fmt::default())));
    b.push_str(&para(Para { center: true, before: 240, after: 60, ..Para::default() }, &runs(&title.to_uppercase(), Fmt { bold: true, size: Some(28), ..Fmt::default() })));
    if let Some(s) = so.map(str::trim).filter(|s| !s.is_empty()) {
        b.push_str(&para(Para { center: true, after: 60, ..Para::default() }, &runs(&format!("Số: {s}"), Fmt::default())));
    }
    if let Some(s) = subtitle.map(str::trim).filter(|s| !s.is_empty()) {
        b.push_str(&para(Para { center: true, after: 240, ..Para::default() }, &runs(s, Fmt { italic: true, size: Some(24), ..Fmt::default() })));
    }
    let body = Para { justify: true, after: 80, ..Para::default() };
    for blk in blocks {
        match blk {
            DocBlock::Heading(t) => b.push_str(&para(Para { before: 160, after: 80, keep_next: true, ..Para::default() }, &runs(t, bold))),
            // Mỗi dòng một đoạn: đoạn căn đều có `w:br` bị Word giãn chữ ở dòng trước chỗ ngắt.
            DocBlock::Paragraph(t) => {
                for line in t.replace("\r\n", "\n").split('\n') {
                    b.push_str(&para(body, &runs(line, Fmt::default())));
                }
            }
            DocBlock::Field { label, value } => {
                let v = value.replace("\r\n", "\n");
                let mut lines = v.split('\n');
                let first = lines.next().unwrap_or("");
                let inner = format!("{}{}", runs(&format!("{label}: "), bold), runs(first, Fmt::default()));
                b.push_str(&para(body, &inner));
                for line in lines {
                    b.push_str(&para(Para { indent: 425, ..body }, &runs(line, Fmt::default())));
                }
            }
            DocBlock::Bullets(items) => {
                for it in items {
                    b.push_str(&para(Para { indent: 425, hanging: 283, ..body }, &runs(&format!("-\t{it}"), Fmt::default())));
                }
            }
            DocBlock::Table { columns, rows } => {
                b.push_str(&table(columns, rows));
                b.push_str(&para(Para { after: 80, ..Para::default() }, ""));
            }
            DocBlock::Signatures { left, right } => {
                b.push_str(&para(Para { before: 240, ..Para::default() }, ""));
                b.push_str(&signatures(left, right));
            }
        }
    }
    format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
            "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" ",
            "xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><w:body>{}",
            "<w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/>",
            "<w:pgMar w:top=\"1134\" w:right=\"1134\" w:bottom=\"1134\" w:left=\"1701\" w:header=\"720\" w:footer=\"720\" w:gutter=\"0\"/>",
            "</w:sectPr></w:body></w:document>"
        ),
        b
    )
}

const CONTENT_TYPES: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
    "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">",
    "<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>",
    "<Default Extension=\"xml\" ContentType=\"application/xml\"/>",
    "<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>",
    "<Override PartName=\"/word/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/>",
    "<Override PartName=\"/docProps/core.xml\" ContentType=\"application/vnd.openxmlformats-package.core-properties+xml\"/>",
    "</Types>"
);
const RELS: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
    "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>",
    "<Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties\" Target=\"docProps/core.xml\"/>",
    "</Relationships>"
);
const DOC_RELS: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
    "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>",
    "</Relationships>"
);
const STYLES: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
    "<w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
    "<w:docDefaults><w:rPrDefault><w:rPr>",
    "<w:rFonts w:ascii=\"Times New Roman\" w:hAnsi=\"Times New Roman\" w:eastAsia=\"Times New Roman\" w:cs=\"Times New Roman\"/>",
    "<w:sz w:val=\"26\"/><w:szCs w:val=\"26\"/><w:lang w:val=\"vi-VN\"/>",
    "</w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after=\"0\" w:line=\"276\" w:lineRule=\"auto\"/></w:pPr></w:pPrDefault></w:docDefaults>",
    "<w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\"><w:name w:val=\"Normal\"/><w:qFormat/></w:style>",
    "<w:style w:type=\"table\" w:default=\"1\" w:styleId=\"TableNormal\"><w:name w:val=\"Normal Table\"/>",
    "<w:tblPr><w:tblInd w:w=\"0\" w:type=\"dxa\"/><w:tblCellMar><w:top w:w=\"0\" w:type=\"dxa\"/><w:left w:w=\"108\" w:type=\"dxa\"/>",
    "<w:bottom w:w=\"0\" w:type=\"dxa\"/><w:right w:w=\"108\" w:type=\"dxa\"/></w:tblCellMar></w:tblPr></w:style>",
    "</w:styles>"
);

fn core_xml(title: &str) -> String {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
            "<cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" ",
            "xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" ",
            "xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">",
            "<dc:title>{}</dc:title><dc:creator>DigiText</dc:creator>",
            "<dcterms:created xsi:type=\"dcterms:W3CDTF\">{}</dcterms:created></cp:coreProperties>"
        ),
        esc(title),
        now
    )
}

/// Dựng tệp .docx: quốc hiệu, tiêu đề, số biên bản, căn cứ (subtitle), các khối.
pub fn render(title: &str, subtitle: Option<&str>, so: Option<&str>, blocks: &[DocBlock]) -> Result<Vec<u8>, String> {
    let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opt = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let parts: [(&str, String); 6] = [
        ("[Content_Types].xml", CONTENT_TYPES.to_string()),
        ("_rels/.rels", RELS.to_string()),
        ("word/_rels/document.xml.rels", DOC_RELS.to_string()),
        ("word/styles.xml", STYLES.to_string()),
        ("docProps/core.xml", core_xml(title)),
        ("word/document.xml", document_xml(title, subtitle, so, blocks)),
    ];
    for (name, body) in parts {
        w.start_file(name, opt).map_err(|e| format!("docx: {e}"))?;
        w.write_all(body.as_bytes()).map_err(|e| format!("docx: {e}"))?;
    }
    let c = w.finish().map_err(|e| format!("docx: {e}"))?;
    Ok(c.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn docx_co_du_phan_va_thoat_ky_tu() {
        let blocks = vec![
            DocBlock::Paragraph("Hôm nay <ngày> & \"giờ\"\nDòng 2".into()),
            DocBlock::Heading("I. NỘI DUNG".into()),
            DocBlock::Field { label: "Phông".into(), value: "UBND xã A\u{1}".into() },
            DocBlock::Bullets(vec!["một".into(), "hai".into()]),
            DocBlock::Table { columns: vec!["Hộp".into(), "Số HS".into()], rows: vec![vec!["01".into(), "12".into()], vec!["02".into()]] },
            DocBlock::Signatures { left: vec!["BÊN GIAO".into(), "(Ký)".into()], right: vec!["BÊN NHẬN".into()] },
        ];
        let bytes = render("Biên bản giao nhận", Some("(Theo khoản 1 Điều 9)"), Some("01/BB-GNG.2026"), &blocks).expect("render");
        if let Ok(p) = std::env::var("DOCX_MAU") {
            std::fs::write(p, &bytes).expect("ghi mẫu");
        }
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip");
        for p in ["[Content_Types].xml", "_rels/.rels", "word/_rels/document.xml.rels", "word/styles.xml", "docProps/core.xml"] {
            assert!(z.by_name(p).is_ok(), "thiếu {p}");
        }
        let mut doc = String::new();
        z.by_name("word/document.xml").unwrap().read_to_string(&mut doc).unwrap();
        assert!(doc.contains("BIÊN BẢN GIAO NHẬN"));
        assert!(doc.contains("Số: 01/BB-GNG.2026"));
        assert!(doc.contains("&lt;ngày&gt; &amp; &quot;giờ&quot;"));
        assert!(doc.contains("Dòng 2"));
        assert!(!doc.contains('\u{1}'));
        assert_eq!(doc.matches("<w:tbl>").count(), 2);
        assert!(doc.contains("<w:tblHeader/>"));
    }
}
