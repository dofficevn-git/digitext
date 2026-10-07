// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
mod glyphless;
mod icc;
pub mod pdfa;
pub mod manual;
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Dictionary, Document, Object, Stream, StringFormat};
#[derive(Debug, Clone)]
pub struct Tny {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
#[derive(Debug, Clone)]
pub struct Tgh {
    pub jpeg: Vec<u8>,
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: u32,
    pub spans: Vec<Tny>,
}
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("không có trang nào để ghép")]
    Empty,
    #[error("trang {page}: kích thước/DPI không hợp lệ")]
    BadPageSize { page: usize },
    #[error("mã hoá content stream lỗi: {0}")]
    Encode(String),
    #[error("ghi PDF lỗi: {0}")]
    Save(String),
    #[error("tách ảnh trang lỗi: {0}")]
    NoPageImage(String),
    #[error(
        "trang {page} không có ảnh JPEG nhúng (DCTDecode) — cần raster hoá"
    )]
    PageNotJpeg { page: i32 },
    #[error("font TTF không đọc được: {0}")]
    BadFont(String),
    #[error("font thiếu glyph cho ký tự {0:?}")]
    MissingGlyph(char),
}
fn f_24n(dpi: u32) -> f32 {
    72.0 / dpi as f32
}
const C_MP: &str = "/CIDInit /ProcSet findresource begin\n\
12 dict begin\n\
begincmap\n\
/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
/CMapName /Adobe-Identity-UCS def\n\
/CMapType 2 def\n\
1 begincodespacerange\n\
<0000> <FFFF>\n\
endcodespacerange\n\
1 beginbfrange\n\
<0000> <FFFF> <0000>\n\
endbfrange\n\
endcmap\n\
CMapName currentdict /CMap defineresource pop\n\
end\n\
end";
fn f_196(text: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len() * 2);
    for u in text.encode_utf16() {
        out.push((u >> 8) as u8);
        out.push((u & 0xff) as u8);
    }
    out
}
fn f_qq(r_49a: &mut Document) -> lopdf::ObjectId {
    let r_49d = r_49a.add_object(Stream::new(dictionary! {}, C_MP.as_bytes().to_vec()));
    let ttf = glyphless::f_1f8();
    let mut r_49c = Stream::new(
        dictionary! {
            "Length1" => ttf.len() as i64,
        },
        ttf,
    );
    let _ = r_49c.compress();
    let r_49b = r_49a.add_object(r_49c);
    let r_499 = r_49a.add_object(dictionary! {
        "Type" => "FontDescriptor", "FontName" => glyphless::C_7B, "Flags" => 4,
        "FontBBox" => vec![0.into(), 0.into(), 1000.into(), 1000.into()],
        "ItalicAngle" => 0, "Ascent" => 1000, "Descent" => 0, "CapHeight" =>
        1000, "StemV" => 80, "FontFile2" => r_49b,
    });
    let r_498 = r_49a.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "CIDFontType2", "BaseFont" =>
        glyphless::C_7B, "CIDSystemInfo" => dictionary! { "Registry" =>
        Object::String(b"Adobe".to_vec(), StringFormat::Literal), "Ordering" =>
        Object::String(b"Identity".to_vec(), StringFormat::Literal), "Supplement"
        => 0, }, "FontDescriptor" => r_499, "CIDToGIDMap" => "Identity", "DW" =>
        1000,
    });
    r_49a.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type0", "BaseFont" => glyphless::C_7B,
        "Encoding" => "Identity-H", "DescendantFonts" =>
        vec![Object::Reference(r_498)], "ToUnicode" => r_49d,
    })
}
fn f_1xm(page: &Tgh, page_w_pt: f32, r_49i: f32) -> Content {
    let r_49j = f_24n(page.dpi);
    let mut r_49h = vec![
        Operation::new("q", vec![]),
        Operation::new(
            "cm",
            vec![
                page_w_pt.into(),
                0.into(),
                0.into(),
                r_49i.into(),
                0.into(),
                0.into(),
            ],
        ),
        Operation::new("Do", vec![Object::Name(b"Im0".to_vec())]),
        Operation::new("Q", vec![]),
        Operation::new("BT", vec![]),
        Operation::new("Tr", vec![3.into()]),
    ];
    for span in &page.spans {
        let r_49g = span.text.encode_utf16().count();
        if r_49g == 0 {
            continue;
        }
        let r_49f = (span.height * r_49j).max(1.0);
        let r_49k = span.x * r_49j;
        let r_49l = r_49i - (span.y + span.height) * r_49j;
        let r_49e = (span.width * r_49j).max(0.1);
        let r_49m = (r_49e / (r_49g as f32 * r_49f) * 100.0).clamp(1.0, 1000.0);
        r_49h.push(Operation::new(
            "Tf",
            vec![Object::Name(b"F0".to_vec()), r_49f.into()],
        ));
        r_49h.push(Operation::new("Tz", vec![r_49m.into()]));
        r_49h.push(Operation::new(
            "Tm",
            vec![
                1.into(),
                0.into(),
                0.into(),
                1.into(),
                r_49k.into(),
                r_49l.into(),
            ],
        ));
        r_49h.push(Operation::new(
            "Tj",
            vec![Object::String(f_196(&span.text), StringFormat::Hexadecimal)],
        ));
    }
    r_49h.push(Operation::new("ET", vec![]));
    Content { operations: r_49h }
}
pub fn f_1b3(pdf: &[u8], page_no: i32) -> Result<Vec<u8>, PdfError> {
    f_1b5(pdf, page_no).map(|(jpeg, _)| jpeg)
}
pub const FM1_MIN_DPI: u32 = 200;
pub const FM1_MIN_BIT_DEPTH: u32 = 24;
const C_4F: u32 = 72;
const C_4E: u32 = 1200;
fn f_17h(width_px: u32, page_w_pt: f32) -> Option<u32> {
    if width_px == 0 || !page_w_pt.is_finite() || page_w_pt <= 1.0 {
        return None;
    }
    let dpi = (width_px as f32 * 72.0 / page_w_pt).round();
    let dpi = if dpi.is_finite() && dpi >= 1.0 {
        dpi as u32
    } else {
        return None;
    };
    (C_4F..=C_4E).contains(&dpi).then_some(dpi)
}
pub fn f_1b5(pdf: &[u8], page_no: i32) -> Result<(Vec<u8>, Option<u32>), PdfError> {
    page_jpeg(&ParsedPdf::parse(pdf)?, page_no)
}
/// PDF nguồn đã đọc một lần — lấy JPEG nhiều trang mà không phải đọc lại cả tệp mỗi trang.
pub struct ParsedPdf(Document);
impl ParsedPdf {
    pub fn parse(pdf: &[u8]) -> Result<Self, PdfError> {
        Document::load_mem(pdf)
            .map(ParsedPdf)
            .map_err(|e| PdfError::NoPageImage(format!("load PDF: {e}")))
    }
}
/// Như `f_1b5` nhưng trên PDF đã đọc sẵn.
pub fn page_jpeg(parsed: &ParsedPdf, page_no: i32) -> Result<(Vec<u8>, Option<u32>), PdfError> {
    let r_49p = &parsed.0;
    let r_49t = u32::try_from(page_no)
        .map_err(|_| PdfError::NoPageImage(format!("page_no {page_no} không hợp lệ")))?;
    let pages = r_49p.get_pages();
    let r_49s = *pages.get(&r_49t).ok_or_else(|| {
        PdfError::NoPageImage(format!(
            "không có trang {page_no} (PDF {} trang)",
            pages.len()
        ))
    })?;
    let r_49o = |obj: &Object| -> Option<lopdf::Dictionary> {
        match obj {
            Object::Dictionary(r_49n) => Some(r_49n.clone()),
            Object::Reference(id) => r_49p.get_dictionary(*id).ok().cloned(),
            _ => None,
        }
    };
    let page = r_49p
        .get_dictionary(r_49s)
        .map_err(|e| PdfError::NoPageImage(e.to_string()))?;
    let r_49u = page
        .get(b"Resources")
        .ok()
        .and_then(r_49o)
        .ok_or(PdfError::PageNotJpeg { page: page_no })?;
    let r_49z = r_49u
        .get(b"XObject")
        .ok()
        .and_then(r_49o)
        .ok_or(PdfError::PageNotJpeg { page: page_no })?;
    for (_name, obj) in r_49z.iter() {
        let Object::Reference(id) = obj else { continue };
        let Ok(r_49v) = r_49p.get_object(*id).and_then(|o| o.as_stream()) else {
            continue;
        };
        let r_49q = matches!(
            r_49v.dict.get(b"Subtype").and_then(| o | o.as_name()), Ok(n) if n ==
            b"Image"
        );
        let r_49r = match r_49v.dict.get(b"Filter") {
            Ok(Object::Name(n)) => n == b"DCTDecode",
            Ok(Object::Array(a)) => a
                .iter()
                .any(|f| matches!(f.as_name(), Ok(n) if n == b"DCTDecode")),
            _ => false,
        };
        if r_49q && r_49r {
            // Width/Height có thể là tham chiếu gián tiếp ("/Height 135 0 R" — máy quét khổ rộng ghi
            // chiều cao SAU khi quét xong, 7/10/2026).
            let num = |key: &[u8]| -> Option<u32> {
                let o = r_49v.dict.get(key).ok()?;
                let o = match o {
                    Object::Reference(id) => r_49p.get_object(*id).ok()?,
                    o => o,
                };
                u32::try_from(o.as_i64().ok()?).ok()
            };
            let width_px = num(b"Width");
            let page_w_pt = page
                .get(b"MediaBox")
                .ok()
                .and_then(|o| o.as_array().ok())
                .and_then(|a| match (a.first(), a.get(2)) {
                    (Some(r_49x), Some(r_49y)) => {
                        let f = |o: &Object| {
                            o.as_float()
                                .ok()
                                .or(o.as_i64().ok().map(|r_49w| r_49w as f32))
                        };
                        Some(f(r_49y)? - f(r_49x)?)
                    }
                    _ => None,
                });
            let dpi = match (width_px, page_w_pt) {
                (Some(w), Some(pt)) => f_17h(w, pt),
                _ => None,
            };
            let height_px = num(b"Height");
            let mut jpeg = r_49v.content.clone();
            if let (Some(w), Some(h)) = (width_px, height_px) {
                fix_jpeg_sof_dims(&mut jpeg, w, h);
            }
            return Ok((jpeg, dpi));
        }
    }
    Err(PdfError::PageNotJpeg { page: page_no })
}
/// Máy quét khổ rộng ghi chiều cao TẠM vào khung SOF của JPEG (đo 7/10/2026: 65000, dữ liệu ảnh thật
/// chỉ 3504 dòng; từ điển ảnh của PDF ghi đúng 3504). Bộ giải mã đọc theo SOF → đòi ~1,7 GB, vượt trần
/// bộ nhớ → trang bị "page_failed", cả tài liệu không ghép được PDF/A. Khi SOF lớn hơn (hoặc bằng 0 —
/// chiều cao để ở dấu DNL) so với Width/Height của PDF thì ghi lại cho khớp; dữ liệu ảnh không đụng tới.
/// Trả `true` nếu đã sửa.
pub fn fix_jpeg_sof_dims(b: &mut [u8], width: u32, height: u32) -> bool {
    if b.len() < 4 || b[0] != 0xFF || b[1] != 0xD8 || width == 0 || height == 0 {
        return false;
    }
    let (Ok(w16), Ok(h16)) = (u16::try_from(width), u16::try_from(height)) else {
        return false;
    };
    let mut i = 2;
    while i + 4 <= b.len() {
        if b[i] != 0xFF {
            return false;
        }
        let m = b[i + 1];
        if m == 0xFF {
            i += 1;
            continue;
        }
        if m == 0x01 || (0xD0..=0xD7).contains(&m) {
            i += 2;
            continue;
        }
        let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
        let is_sof = (0xC0..=0xCF).contains(&m) && !matches!(m, 0xC4 | 0xC8 | 0xCC);
        if is_sof {
            if i + 9 > b.len() {
                return false;
            }
            let sof_h = u16::from_be_bytes([b[i + 5], b[i + 6]]);
            let sof_w = u16::from_be_bytes([b[i + 7], b[i + 8]]);
            let mut changed = false;
            if sof_h != h16 && (sof_h == 0 || sof_h > h16) {
                b[i + 5..i + 7].copy_from_slice(&h16.to_be_bytes());
                changed = true;
            }
            if sof_w != w16 && (sof_w == 0 || sof_w > w16) {
                b[i + 7..i + 9].copy_from_slice(&w16.to_be_bytes());
                changed = true;
            }
            return changed;
        }
        if m == 0xDA || m == 0xD9 {
            return false;
        }
        i += 2 + len;
    }
    false
}
#[cfg(test)]
mod sof_tests {
    use super::fix_jpeg_sof_dims;
    fn sof_dims(b: &[u8]) -> (u16, u16) {
        let i = b.windows(2).position(|w| w == [0xFF, 0xC0]).unwrap();
        (u16::from_be_bytes([b[i + 7], b[i + 8]]), u16::from_be_bytes([b[i + 5], b[i + 6]]))
    }
    fn jpeg(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(w, h, image::Rgb([120, 30, 30]));
        let mut out = Vec::new();
        image::codecs::jpeg::JpegEncoder::new(&mut out).encode_image(&img).unwrap();
        out
    }
    #[test]
    fn chieu_cao_tam_65000_duoc_sua_theo_pdf() {
        let mut j = jpeg(40, 24);
        let i = j.windows(2).position(|w| w == [0xFF, 0xC0]).unwrap();
        j[i + 5..i + 7].copy_from_slice(&65000u16.to_be_bytes());
        assert!(image::load_from_memory(&j).map(|d| d.height()).unwrap_or(0) != 24);
        assert!(fix_jpeg_sof_dims(&mut j, 40, 24));
        assert_eq!(sof_dims(&j), (40, 24));
        let d = image::load_from_memory(&j).expect("giải mã được sau khi sửa");
        assert_eq!((d.width(), d.height()), (40, 24));
    }
    #[test]
    fn jpeg_dung_kich_thuoc_khong_doi() {
        let mut j = jpeg(40, 24);
        let goc = j.clone();
        assert!(!fix_jpeg_sof_dims(&mut j, 40, 24));
        assert!(!fix_jpeg_sof_dims(&mut j, 80, 48)); // PDF ghi lớn hơn → không đoán, giữ nguyên
        assert_eq!(j, goc);
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tgz {
    pub ok: bool,
    pub pages: usize,
    pub has_text_layer: bool,
    pub pages_with_text: usize,
    pub pages_without_text: Vec<usize>,
    pub has_image: bool,
    pub min_dpi: Option<u32>,
    pub min_bit_depth: Option<u32>,
    pub reason: Option<String>,
}
pub fn f_2p9(pdf: &[u8], r_4a6: Option<i32>) -> Tgz {
    let r_4a5 = match Document::load_mem(pdf) {
        Ok(d) => d,
        Err(e) => {
            return Tgz {
                ok: false,
                pages: 0,
                has_text_layer: false,
                pages_with_text: 0,
                pages_without_text: Vec::new(),
                has_image: false,
                min_dpi: None,
                min_bit_depth: None,
                reason: Some(format!("không mở được PDF: {e}")),
            };
        }
    };
    if r_4a5.trailer.get(b"Encrypt").is_ok() {
        return Tgz {
            ok: false,
            pages: 0,
            has_text_layer: false,
            pages_with_text: 0,
            pages_without_text: Vec::new(),
            has_image: false,
            min_dpi: None,
            min_bit_depth: None,
            reason: Some("PDF được mã hoá (PDF/A không cho phép)".into()),
        };
    }
    let pages = r_4a5.get_pages();
    let n = pages.len();
    let r_4a4 = |obj: &Object| -> Option<lopdf::Dictionary> {
        match obj {
            Object::Dictionary(d) => Some(d.clone()),
            Object::Reference(id) => r_4a5.get_dictionary(*id).ok().cloned(),
            _ => None,
        }
    };
    let mut pages_with_text = 0usize;
    let mut pages_without_text: Vec<usize> = Vec::new();
    let mut has_image = false;
    let mut min_dpi: Option<u32> = None;
    let mut min_bit_depth: Option<u32> = None;
    for (page_no, r_4a9) in pages.iter() {
        let Ok(page) = r_4a5.get_dictionary(*r_4a9) else {
            continue;
        };
        if f_1xp(&r_4a5, *r_4a9) {
            pages_with_text += 1;
        } else {
            pages_without_text.push(*page_no as usize);
        }
        let Some(r_4aa) = page.get(b"Resources").ok().and_then(r_4a4) else {
            continue;
        };
        if let Some(r_4ae) = r_4aa.get(b"XObject").ok().and_then(r_4a4) {
            for (_name, obj) in r_4ae.iter() {
                if let Object::Reference(id) = obj {
                    if let Ok(r_4ab) = r_4a5.get_object(*id).and_then(|o| o.as_stream()) {
                        if matches!(
                            r_4ab.dict.get(b"Subtype").and_then(| o | o.as_name()), Ok(s)
                            if s == b"Image"
                        ) {
                            has_image = true;
                            let r_4a8 = |r_4a7: &[u8]| {
                                r_4ab.dict.get(r_4a7).ok().and_then(|o| o.as_i64().ok())
                            };
                            let r_4a2 =
                                match r_4ab.dict.get(b"ColorSpace").and_then(|o| o.as_name()) {
                                    Ok(cs) if cs == b"DeviceRGB" || cs == b"CalRGB" => Some(3),
                                    Ok(cs) if cs == b"DeviceCMYK" => Some(4),
                                    Ok(cs) if cs == b"DeviceGray" || cs == b"CalGray" => Some(1),
                                    _ => None,
                                };
                            if let (Some(r_4a1), Some(r_4a0)) = (r_4a2, r_4a8(b"BitsPerComponent"))
                            {
                                let r_4a3 = r_4a1 as u32 * r_4a0 as u32;
                                min_bit_depth =
                                    Some(min_bit_depth.map_or(r_4a3, |m: u32| m.min(r_4a3)));
                            }
                            if let (Some(r_4ad), Some(r_4ac)) =
                                (r_4a8(b"Width"), page_w_pt(page, &r_4a5))
                            {
                                if let Some(dpi) =
                                    u32::try_from(r_4ad).ok().and_then(|w| f_17h(w, r_4ac))
                                {
                                    min_dpi = Some(min_dpi.map_or(dpi, |m: u32| m.min(dpi)));
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
    let has_text_layer = pages_with_text > 0;
    let reason = if n == 0 {
        Some("PDF không có trang nào".into())
    } else if matches!(r_4a6, Some(exp) if exp > 0 && exp as usize != n) {
        Some(format!(
            "số trang lệch: PDF {n} trang, tài liệu ghi {} trang",
            r_4a6.unwrap()
        ))
    } else if !has_text_layer {
        Some(
            "lớp text rỗng toàn bộ (OCR không ra chữ nào — không phải PDF 2 lớp tra-cứu-được)"
                .into(),
        )
    } else if !has_image {
        Some("thiếu ảnh nền trang".into())
    } else if matches!(min_bit_depth, Some(d) if d < FM1_MIN_BIT_DEPTH) {
        Some(
            format!(
                "ảnh nền {} bit — FM-1 (Điều 8.2.a) đòi ảnh màu tối thiểu {FM1_MIN_BIT_DEPTH} bit",
                min_bit_depth.unwrap()
            ),
        )
    } else if matches!(min_dpi, Some(d) if d < FM1_MIN_DPI) {
        Some(
            format!(
                "độ phân giải {} dpi — FM-1 (Điều 8.2.a) đòi tối thiểu {FM1_MIN_DPI} dpi với tài liệu hành chính (300 dpi với bản đồ, bản vẽ)",
                min_dpi.unwrap()
            ),
        )
    } else {
        None
    };
    Tgz {
        ok: reason.is_none(),
        pages: n,
        has_text_layer,
        pages_with_text,
        pages_without_text,
        has_image,
        min_dpi,
        min_bit_depth,
        reason,
    }
}
fn page_w_pt(page: &lopdf::Dictionary, r_4ah: &Document) -> Option<f32> {
    let r_4al = |r_4am: &Object| {
        r_4am
            .as_float()
            .ok()
            .or(r_4am.as_i64().ok().map(|r_4an| r_4an as f32))
    };
    let r_4af = match page.get(b"MediaBox") {
        Ok(Object::Array(a)) => Some(a.clone()),
        Ok(Object::Reference(r_4aj)) => r_4ah.get_object(*r_4aj).ok()?.as_array().ok().cloned(),
        _ => {
            let mut r_4ak = page.get(b"Parent").ok()?.as_reference().ok()?;
            let mut r_4ai = None;
            for _ in 0..8 {
                let r_4ag = r_4ah.get_dictionary(r_4ak).ok()?;
                if let Ok(Object::Array(a)) = r_4ag.get(b"MediaBox") {
                    r_4ai = Some(a.clone());
                    break;
                }
                r_4ak = r_4ag.get(b"Parent").ok()?.as_reference().ok()?;
            }
            r_4ai
        }
    }?;
    if r_4af.len() != 4 {
        return None;
    }
    Some(r_4al(&r_4af[2])? - r_4al(&r_4af[0])?)
}
/// Dò thô lớp chữ khi bộ phân tích toán tử không đọc nổi luồng nội dung. Gặp thật (21/9/2026,
/// 28.793 tệp của một kho quét bằng máy Canon): trang có NHIỀU luồng nội dung, luồng đầu chỉ là
/// dòng chú thích `% CANON_PFINF_TYPE0_TEXTON` — lopdf ghép các luồng rồi phân tích hỏng, khiến
/// trang có lớp chữ ẩn (`3 Tr … <6E3F> Tj`) bị coi là KHÔNG có chữ. Ở đây chỉ cần biết có toán tử
/// vẽ chữ mang chuỗi khác rỗng nằm sau `BT` hay không.
fn raw_has_text(doc: &Document, page: lopdf::ObjectId) -> bool {
    for id in doc.get_page_contents(page) {
        let Ok(Object::Stream(st)) = doc.get_object(id) else { continue };
        let bytes = st.decompressed_content().unwrap_or_else(|_| st.content.clone());
        let Some(bt) = bytes.windows(2).position(|w| w == b"BT") else { continue };
        let body = &bytes[bt..];
        let shows = |op: &[u8]| {
            body.windows(op.len() + 1).enumerate().any(|(i, w)| {
                &w[1..] == op
                    && matches!(w[0], b' ' | b'\n' | b'\r' | b'>' | b')' | b']')
                    && body.get(i + 1 + op.len()).map_or(true, |c| c.is_ascii_whitespace())
            })
        };
        let has_string = body.windows(2).any(|w| (w[0] == b'<' && w[1].is_ascii_hexdigit()) || (w[0] == b'(' && w[1] != b')'));
        if has_string && (shows(b"Tj") || shows(b"TJ")) {
            return true;
        }
    }
    false
}
fn f_1xp(r_4ao: &Document, r_4as: lopdf::ObjectId) -> bool {
    let Ok(content) = r_4ao.get_and_decode_page_content(r_4as) else {
        return raw_has_text(r_4ao, r_4as);
    };
    f_1xp_ops(&content) || raw_has_text(r_4ao, r_4as)
}
fn f_1xp_ops(content: &lopdf::content::Content) -> bool {
    content
        .operations
        .iter()
        .any(|r_4ar| match r_4ar.operator.as_str() {
            "Tj" => {
                matches!(
                    r_4ar.operands.first(), Some(Object::String(s, _)) if ! s.is_empty()
                )
            }
            "TJ" => match r_4ar.operands.first() {
                Some(Object::Array(r_4ap)) => r_4ap
                    .iter()
                    .any(|r_4aq| matches!(r_4aq, Object::String(s, _) if ! s.is_empty())),
                _ => false,
            },
            _ => false,
        })
}
fn f_11f(pages: &[Tgh]) -> [u8; 32] {
    let mut r_4at = blake3::Hasher::new();
    for page in pages {
        r_4at.update(&page.jpeg);
        r_4at.update(&page.width_px.to_le_bytes());
        r_4at.update(&page.height_px.to_le_bytes());
        r_4at.update(&page.dpi.to_le_bytes());
        for s in &page.spans {
            r_4at.update(s.text.as_bytes());
            for r_4au in [s.x, s.y, s.width, s.height] {
                r_4at.update(&r_4au.to_le_bytes());
            }
        }
    }
    *r_4at.finalize().as_bytes()
}
pub fn f_vz(pages: &[Tgh]) -> Result<Vec<u8>, PdfError> {
    if pages.is_empty() {
        return Err(PdfError::Empty);
    }
    let mut r_4b1 = Document::with_version("1.7");
    let r_4b9 = r_4b1.new_object_id();
    let r_4b2 = f_qq(&mut r_4b1);
    let mut r_4b7: Vec<Object> = Vec::with_capacity(pages.len());
    for (i, page) in pages.iter().enumerate() {
        if page.width_px == 0 || page.height_px == 0 || page.dpi == 0 {
            return Err(PdfError::BadPageSize { page: i });
        }
        let r_4bc = f_24n(page.dpi);
        let page_w_pt = page.width_px as f32 * r_4bc;
        let r_4b6 = page.height_px as f32 * r_4bc;
        let r_4b3 = Stream::new(
            dictionary! {
                "Type" => "XObject", "Subtype" => "Image", "Width" => page.width_px as
                i64, "Height" => page.height_px as i64, "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => 8, "Filter" => "DCTDecode",
            },
            page.jpeg.clone(),
        );
        let r_4b4 = r_4b1.add_object(r_4b3);
        let content = f_1xm(page, page_w_pt, r_4b6);
        let r_4ax = content
            .encode()
            .map_err(|e| PdfError::Encode(e.to_string()))?;
        let mut r_4az = Stream::new(dictionary! {}, r_4ax);
        let _ = r_4az.compress();
        let r_4ay = r_4b1.add_object(r_4az);
        let r_4bb = dictionary! {
            "XObject" => dictionary! { "Im0" => r_4b4 }, "Font" => dictionary! { "F0" =>
            r_4b2 },
        };
        let r_4b5 = dictionary! {
            "Type" => "Page", "Parent" => r_4b9, "MediaBox" => vec![0.into(), 0.into(),
            page_w_pt.into(), r_4b6.into()], "Contents" => r_4ay, "Resources" => r_4bb,
        };
        r_4b7.push(r_4b1.add_object(r_4b5).into());
    }
    let r_4b0 = r_4b7.len() as i64;
    let r_4b8 = dictionary! {
        "Type" => "Pages", "Kids" => r_4b7, "Count" => r_4b0,
    };
    r_4b1.objects.insert(r_4b9, Object::Dictionary(r_4b8));
    let r_4ba = pdfa::f_qs(&mut r_4b1, None);
    let r_4aw = r_4b1.add_object(dictionary! {
        "Type" => "Catalog", "Pages" => r_4b9, "Metadata" => r_4ba.metadata,
        "OutputIntents" => r_4ba.output_intents,
    });
    r_4b1.trailer.set("Root", r_4aw);
    pdfa::f_2e3(&mut r_4b1, &f_11f(pages));
    let mut r_4av = Vec::new();
    r_4b1
        .save_to(&mut r_4av)
        .map_err(|e| PdfError::Save(e.to_string()))?;
    Ok(r_4av)
}
#[derive(Debug, Clone)]
pub struct T2m {
    pub label: String,
    pub value: String,
}
pub fn f_1cq() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("DIGITEXT_CERT_FONT") {
        let p = std::path::PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    C_1M.iter()
        .map(std::path::PathBuf::from)
        .find(|p| p.is_file())
}
pub fn f_1cr(text: &str) -> Option<std::path::PathBuf> {
    let r_4bd = |r_4bf: &std::path::Path| -> bool {
        let Ok(bytes) = std::fs::read(r_4bf) else {
            return false;
        };
        let Ok(r_4be) = ttf_parser::Face::parse(&bytes, 0) else {
            return false;
        };
        text.chars()
            .all(|c| c.is_whitespace() || matches!(r_4be.glyph_index(c), Some(g) if g.0 != 0))
    };
    if let Ok(p) = std::env::var("DIGITEXT_CERT_FONT") {
        let p = std::path::PathBuf::from(p);
        if p.is_file() && r_4bd(&p) {
            return Some(p);
        }
    }
    C_1M.iter()
        .map(std::path::PathBuf::from)
        .find(|p| p.is_file() && r_4bd(p))
}
/// Font không chân (sans) cho tài liệu kỹ thuật kèm gói (tờ hướng dẫn gói tin); không có thì
/// quay về `f_1cq`. Ưu tiên `DIGITEXT_CERT_FONT` nếu đặt.
pub fn sans_font() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("DIGITEXT_CERT_FONT") {
        let p = std::path::PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    const SANS: &[&str] = &[
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/Library/Fonts/Arial.ttf",
    ];
    SANS.iter().map(std::path::PathBuf::from).find(|p| p.is_file()).or_else(f_1cq)
}
const C_1M: &[&str] = &[
    "/System/Library/Fonts/Supplemental/Times New Roman.ttf",
    "/Library/Fonts/Times New Roman.ttf",
    "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "C:\\Windows\\Fonts\\times.ttf",
    "C:\\Windows\\Fonts\\arial.ttf",
];
struct T8r {
    glyphs: Vec<(u16, u16)>,
}
fn f_2eo(r_4bi: &ttf_parser::Face<'_>, text: &str) -> Result<T8r, PdfError> {
    let r_4bj = r_4bi.units_per_em() as f32;
    let mut glyphs = Vec::with_capacity(text.chars().count());
    for c in text.chars() {
        let gid = match r_4bi.glyph_index(c) {
            Some(g) if g.0 != 0 => g,
            _ if c == '\u{00A0}' => r_4bi
                .glyph_index(' ')
                .filter(|g| g.0 != 0)
                .ok_or(PdfError::MissingGlyph(c))?,
            _ => return Err(PdfError::MissingGlyph(c)),
        };
        let r_4bg = r_4bi.glyph_hor_advance(gid).unwrap_or(r_4bj as u16);
        let r_4bh = ((r_4bg as f32) * 1000.0 / r_4bj).round() as u16;
        glyphs.push((gid.0, r_4bh));
    }
    Ok(T8r { glyphs })
}
fn f_2bf(r_4bl: &T8r, r_4bk: f32) -> f32 {
    r_4bl.glyphs.iter().map(|(_, a)| *a as f32).sum::<f32>() / 1000.0 * r_4bk
}
fn f_2k2<'a>(r_4bn: impl IntoIterator<Item = &'a str>) -> [u8; 32] {
    let mut r_4bm = blake3::Hasher::new();
    for l in r_4bn {
        r_4bm.update(l.as_bytes());
        r_4bm.update(b"\n");
    }
    *r_4bm.finalize().as_bytes()
}
fn f_zc(r_4bo: &T8r) -> Vec<u8> {
    r_4bo
        .glyphs
        .iter()
        .flat_map(|(gid, _)| gid.to_be_bytes())
        .collect()
}
fn f_1gw(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02X}"));
    }
    s
}
fn f_qn(r_4bu: &mut Document, ttf: &[u8], r_4bz: &[(char, u16, u16)]) -> lopdf::ObjectId {
    let mut r_4bs = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n\
         /CMapName /DigiTextCert def\n/CMapType 2 def\n\
         1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    r_4bs.push_str(&format!("{} beginbfchar\n", r_4bz.len()));
    for (c, gid, _) in r_4bz {
        let mut r_4bq = [0u16; 2];
        let r_4by = c.encode_utf16(&mut r_4bq);
        let uni: String = r_4by.iter().map(|u| format!("{u:04X}")).collect();
        r_4bs.push_str(&format!("<{gid:04X}> <{uni}>\n"));
    }
    r_4bs.push_str("endbfchar\nendcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    let r_4bx = r_4bu.add_object(Stream::new(dictionary! {}, r_4bs.into_bytes()));
    let mut r_4bw = Stream::new(
        dictionary! {
            "Length1" => ttf.len() as i64
        },
        ttf.to_vec(),
    );
    let _ = r_4bw.compress();
    let r_4bv = r_4bu.add_object(r_4bw);
    let r_4bt = r_4bu.add_object(dictionary! {
        "Type" => "FontDescriptor", "FontName" => "DigiTextCert", "Flags" => 4,
        "FontBBox" => vec![(- 1000).into(), (- 500).into(), 2000.into(), 1500
        .into()], "ItalicAngle" => 0, "Ascent" => 900, "Descent" => (- 220),
        "CapHeight" => 700, "StemV" => 80, "FontFile2" => r_4bv,
    });
    let mut w: Vec<Object> = Vec::new();
    for (_, gid, r_4bp) in r_4bz {
        w.push((*gid as i64).into());
        w.push(Object::Array(vec![(*r_4bp as i64).into()]));
    }
    let r_4br = r_4bu.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "CIDFontType2", "BaseFont" =>
        "DigiTextCert", "CIDSystemInfo" => dictionary! { "Registry" =>
        Object::String(b"Adobe".to_vec(), StringFormat::Literal), "Ordering" =>
        Object::String(b"Identity".to_vec(), StringFormat::Literal), "Supplement"
        => 0, }, "FontDescriptor" => r_4bt, "CIDToGIDMap" => "Identity", "DW" =>
        1000, "W" => w,
    });
    r_4bu.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type0", "BaseFont" => "DigiTextCert",
        "Encoding" => "Identity-H", "DescendantFonts" =>
        vec![Object::Reference(r_4br)], "ToUnicode" => r_4bx,
    })
}
const C_1: f32 = 595.0;
const C_0: f32 = 842.0;
pub const C_K3: f32 = 10.0;
const C_K4: f32 = 12.0;
const C_K6: f32 = 4.0;
pub fn f_w0(
    r_4c1: &mut Document,
    r_4c6: &[String],
    r_4c4: &[u8],
) -> Result<(lopdf::ObjectId, f32, f32), PdfError> {
    if r_4c6.is_empty() {
        return Err(PdfError::Empty);
    }
    let r_4c2 = ttf_parser::Face::parse(r_4c4, 0).map_err(|e| PdfError::BadFont(e.to_string()))?;
    let mut r_4cb: Vec<(char, u16, u16)> = Vec::new();
    let mut r_4c9 = std::collections::BTreeSet::new();
    for line in r_4c6 {
        for c in line.chars() {
            if r_4c9.insert(c) {
                let run = f_2eo(&r_4c2, &c.to_string())?;
                let (gid, r_4c0) = run.glyphs[0];
                r_4cb.push((c, gid, r_4c0));
            }
        }
    }
    let mut r_4c7: f32 = 0.0;
    for line in r_4c6 {
        let run = f_2eo(&r_4c2, line)?;
        r_4c7 = r_4c7.max(f_2bf(&run, C_K3));
    }
    let w = r_4c7 + C_K6 * 2.0;
    let r_4c5 = r_4c6.len() as f32 * C_K4 + C_K6 * 2.0;
    let r_4c3 = f_qn(r_4c1, r_4c4, &r_4cb);
    let mut r_4c8 = vec![
        Operation::new("G", vec![0.into()]),
        Operation::new("w", vec![0.5.into()]),
        Operation::new(
            "re",
            vec![
                0.25.into(),
                0.25.into(),
                (w - 0.5).into(),
                (r_4c5 - 0.5).into(),
            ],
        ),
        Operation::new("S", vec![]),
        Operation::new("g", vec![0.into()]),
        Operation::new("BT", vec![]),
    ];
    let mut y = r_4c5 - C_K6 - C_K3;
    for line in r_4c6 {
        let run = f_2eo(&r_4c2, line)?;
        r_4c8.push(Operation::new(
            "Tf",
            vec![Object::Name(b"F0".to_vec()), C_K3.into()],
        ));
        r_4c8.push(Operation::new(
            "Tm",
            vec![
                1.into(),
                0.into(),
                0.into(),
                1.into(),
                C_K6.into(),
                y.into(),
            ],
        ));
        r_4c8.push(Operation::new(
            "Tj",
            vec![Object::String(f_zc(&run), StringFormat::Hexadecimal)],
        ));
        y -= C_K4;
    }
    r_4c8.push(Operation::new("ET", vec![]));
    let bytes = Content { operations: r_4c8 }
        .encode()
        .map_err(|e| PdfError::Encode(e.to_string()))?;
    let mut r_4ca = Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Form", "FormType" => 1, "BBox" => vec![0
            .into(), 0.into(), w.into(), r_4c5.into()], "Resources" => dictionary! {
            "Font" => dictionary! { "F0" => r_4c3 }, },
        },
        bytes,
    );
    let _ = r_4ca.compress();
    Ok((r_4c1.add_object(r_4ca), w, r_4c5))
}
const C_KF: f32 = 24.0;
const C_KG: f32 = 2.0;
const C_KC: f32 = 5.0;
const C_KD: f32 = 5.5;
#[derive(Debug, Clone)]
pub struct Tlz {
    pub org: String,
    pub center_lines: Vec<String>,
    pub side_lines: Vec<String>,
    pub color: (f32, f32, f32),
}
impl Tlz {
    pub const RED: (f32, f32, f32) = (0.78, 0.05, 0.09);
}
fn f_zf(cx: f32, r_4cc: f32, r: f32) -> Vec<Operation> {
    const K: f32 = 0.552_284_8;
    let (r_4cd, mut r_4ce) = (K * r, Vec::with_capacity(6));
    r_4ce.push(Operation::new("m", vec![(cx + r).into(), r_4cc.into()]));
    let r_4cf = |r_4cg: f32, r_4cj: f32, r_4ch: f32, r_4ck: f32, r_4ci: f32, r_4cl: f32| {
        Operation::new(
            "c",
            vec![
                r_4cg.into(),
                r_4cj.into(),
                r_4ch.into(),
                r_4ck.into(),
                r_4ci.into(),
                r_4cl.into(),
            ],
        )
    };
    r_4ce.push(r_4cf(
        cx + r,
        r_4cc + r_4cd,
        cx + r_4cd,
        r_4cc + r,
        cx,
        r_4cc + r,
    ));
    r_4ce.push(r_4cf(
        cx - r_4cd,
        r_4cc + r,
        cx - r,
        r_4cc + r_4cd,
        cx - r,
        r_4cc,
    ));
    r_4ce.push(r_4cf(
        cx - r,
        r_4cc - r_4cd,
        cx - r_4cd,
        r_4cc - r,
        cx,
        r_4cc - r,
    ));
    r_4ce.push(r_4cf(
        cx + r_4cd,
        r_4cc - r,
        cx + r,
        r_4cc - r_4cd,
        cx + r,
        r_4cc,
    ));
    r_4ce
}
const C_KE: f32 = 2.6;
fn f_s7(r_4cm: &ttf_parser::Face<'_>, text: &str, r: f32) -> Result<f32, PdfError> {
    let r_4cn = f_2eo(r_4cm, text)?;
    Ok(f_2bf(&r_4cn, C_KC) / r)
}
fn f_2gv(
    r_4cp: &ttf_parser::Face<'_>,
    org: &str,
    r: f32,
) -> Result<(String, Option<String>), PdfError> {
    if f_s7(r_4cp, org, r)? <= C_KE {
        return Ok((org.to_string(), None));
    }
    let r_4cq = org.chars().count() / 2;
    let r_4co = org
        .char_indices()
        .filter(|(_, c)| *c == ' ')
        .min_by_key(|(i, _)| org[..*i].chars().count().abs_diff(r_4cq))
        .map(|(i, _)| i);
    match r_4co {
        Some(i) => Ok((
            org[..i].trim().to_string(),
            Some(org[i..].trim().to_string()),
        )),
        None => Ok((org.to_string(), None)),
    }
}
fn f_s8(
    r_4ct: &ttf_parser::Face<'_>,
    text: &str,
    cx: f32,
    r_4cs: f32,
    r: f32,
    r_4d1: bool,
) -> Result<Vec<Operation>, PdfError> {
    use std::f32::consts::FRAC_PI_2;
    let glyphs = f_2eo(r_4ct, text)?.glyphs;
    let r_4d2 = f_s7(r_4ct, text, r)?;
    let mut r_4cw = if r_4d1 {
        FRAC_PI_2 + r_4d2 / 2.0
    } else {
        -FRAC_PI_2 - r_4d2 / 2.0
    };
    let mut r_4cv = Vec::with_capacity(glyphs.len() * 2);
    for (gid, r_4cr) in &glyphs {
        let w = *r_4cr as f32 / 1000.0 * C_KC;
        let r_4d0 = w / r;
        let r_4cu = if r_4d1 {
            r_4cw - r_4d0 / 2.0
        } else {
            r_4cw + r_4d0 / 2.0
        };
        let (r_4cx, r_4cy) = (cx + r * r_4cu.cos(), r_4cs + r * r_4cu.sin());
        let r_4cz = if r_4d1 {
            r_4cu - FRAC_PI_2
        } else {
            r_4cu + FRAC_PI_2
        };
        let (c, s) = (r_4cz.cos(), r_4cz.sin());
        r_4cv.push(Operation::new(
            "Tm",
            vec![
                c.into(),
                s.into(),
                (-s).into(),
                c.into(),
                (r_4cx - c * w / 2.0).into(),
                (r_4cy - s * w / 2.0).into(),
            ],
        ));
        r_4cv.push(Operation::new(
            "Tj",
            vec![Object::String(
                gid.to_be_bytes().to_vec(),
                StringFormat::Hexadecimal,
            )],
        ));
        r_4cw = if r_4d1 { r_4cw - r_4d0 } else { r_4cw + r_4d0 };
    }
    Ok(r_4cv)
}
pub fn f_w1(
    r_4da: &mut Document,
    r_4dm: &Tlz,
    r_4dd: &[u8],
) -> Result<(lopdf::ObjectId, f32, f32), PdfError> {
    let r_4db = ttf_parser::Face::parse(r_4dd, 0).map_err(|e| PdfError::BadFont(e.to_string()))?;
    let mut r_4do: Vec<(char, u16, u16)> = Vec::new();
    let mut r_4dk = std::collections::BTreeSet::new();
    let r_4d4 = r_4dm
        .org
        .chars()
        .chain(r_4dm.center_lines.iter().flat_map(|l| l.chars()))
        .chain(r_4dm.side_lines.iter().flat_map(|l| l.chars()));
    for c in r_4d4 {
        if r_4dk.insert(c) {
            let run = f_2eo(&r_4db, &c.to_string())?;
            let (gid, r_4d3) = run.glyphs[0];
            r_4do.push((c, gid, r_4d3));
        }
    }
    let r_4dc = f_qn(r_4da, r_4dd, &r_4do);
    let mut r_4dl: f32 = 0.0;
    for line in &r_4dm.side_lines {
        let run = f_2eo(&r_4db, line)?;
        r_4dl = r_4dl.max(f_2bf(&run, C_K3));
    }
    let r_4de = if r_4dm.side_lines.is_empty() {
        0.0
    } else {
        6.0
    };
    let w = C_KF * 2.0 + r_4de + r_4dl + C_K6 * 2.0;
    let r_4df = (C_KF * 2.0).max(r_4dm.side_lines.len() as f32 * C_K4) + C_K6 * 2.0;
    let (cx, r_4d9) = (C_K6 + C_KF, r_4df / 2.0);
    let (r, g, b) = r_4dm.color;
    let mut r_4dj = vec![
        Operation::new("RG", vec![r.into(), g.into(), b.into()]),
        Operation::new("rg", vec![r.into(), g.into(), b.into()]),
        Operation::new("w", vec![1.2.into()]),
    ];
    r_4dj.extend(f_zf(cx, r_4d9, C_KF));
    r_4dj.push(Operation::new("s", vec![]));
    r_4dj.push(Operation::new("w", vec![0.6.into()]));
    r_4dj.extend(f_zf(cx, r_4d9, C_KF - C_KG));
    r_4dj.push(Operation::new("s", vec![]));
    let r_4d6 = C_KF - C_KG - C_KC * 0.85;
    r_4dj.push(Operation::new("BT", vec![]));
    r_4dj.push(Operation::new(
        "Tf",
        vec![Object::Name(b"F0".to_vec()), C_KC.into()],
    ));
    let (r_4d7, r_4d5) = f_2gv(&r_4db, &r_4dm.org, r_4d6)?;
    r_4dj.extend(f_s8(&r_4db, &r_4d7, cx, r_4d9, r_4d6, true)?);
    if let Some(r_4d8) = r_4d5 {
        r_4dj.extend(f_s8(&r_4db, &r_4d8, cx, r_4d9, r_4d6, false)?);
    }
    let r_4dg = 2.0 * (r_4d6 - C_KC * 1.1);
    let n = r_4dm.center_lines.len() as f32;
    let mut y = r_4d9 + (n - 1.0) * C_KD * 0.6;
    for line in &r_4dm.center_lines {
        let run = f_2eo(&r_4db, line)?;
        let r_4di = f_2bf(&run, C_KD);
        let size = if r_4di > r_4dg && r_4di > 0.0 {
            C_KD * r_4dg / r_4di
        } else {
            C_KD
        };
        let r_4dh = f_2bf(&run, size);
        r_4dj.push(Operation::new(
            "Tf",
            vec![Object::Name(b"F0".to_vec()), size.into()],
        ));
        r_4dj.push(Operation::new(
            "Tm",
            vec![
                1.into(),
                0.into(),
                0.into(),
                1.into(),
                (cx - r_4dh / 2.0).into(),
                (y - size / 2.0).into(),
            ],
        ));
        r_4dj.push(Operation::new(
            "Tj",
            vec![Object::String(f_zc(&run), StringFormat::Hexadecimal)],
        ));
        y -= C_KD * 1.25;
    }
    r_4dj.push(Operation::new("g", vec![0.into()]));
    let r_4dp = C_K6 + C_KF * 2.0 + r_4de;
    let mut y = r_4df / 2.0 + (r_4dm.side_lines.len() as f32 * C_K4) / 2.0 - C_K3;
    for line in &r_4dm.side_lines {
        let run = f_2eo(&r_4db, line)?;
        r_4dj.push(Operation::new(
            "Tf",
            vec![Object::Name(b"F0".to_vec()), C_K3.into()],
        ));
        r_4dj.push(Operation::new(
            "Tm",
            vec![
                1.into(),
                0.into(),
                0.into(),
                1.into(),
                r_4dp.into(),
                y.into(),
            ],
        ));
        r_4dj.push(Operation::new(
            "Tj",
            vec![Object::String(f_zc(&run), StringFormat::Hexadecimal)],
        ));
        y -= C_K4;
    }
    r_4dj.push(Operation::new("ET", vec![]));
    let bytes = Content { operations: r_4dj }
        .encode()
        .map_err(|e| PdfError::Encode(e.to_string()))?;
    let mut r_4dn = Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Form", "FormType" => 1, "BBox" => vec![0
            .into(), 0.into(), w.into(), r_4df.into()], "Resources" => dictionary! {
            "Font" => dictionary! { "F0" => r_4dc }, },
        },
        bytes,
    );
    let _ = r_4dn.compress();
    Ok((r_4da.add_object(r_4dn), w, r_4df))
}
pub fn f_v8(r_4ef: &str, r_4e4: &str, r_4e0: &[T2m], r_4e2: &[u8]) -> Result<Vec<u8>, PdfError> {
    let r_4dz = ttf_parser::Face::parse(r_4e2, 0).map_err(|e| PdfError::BadFont(e.to_string()))?;
    let r_4e6 = 56.0_f32;
    let r_4e7 = C_1 - r_4e6 * 2.0;
    let r_4eg = 14.0_f32;
    let r_4dr = 11.5_f32;
    let leading = 18.0_f32;
    let mut r_4e5: Vec<(String, f32)> = vec![
        (r_4e4.to_string(), r_4dr),
        (String::new(), r_4dr),
        (r_4ef.to_string(), r_4eg),
        (String::new(), r_4dr),
    ];
    for r_4dy in r_4e0 {
        let r_4e3 = format!("{}: {}", r_4dy.label, r_4dy.value);
        let mut current = String::new();
        for word in r_4e3.split(' ') {
            let r_4dt = if current.is_empty() {
                word.to_string()
            } else {
                format!("{current} {word}")
            };
            let run = f_2eo(&r_4dz, &r_4dt)?;
            if f_2bf(&run, r_4dr) > r_4e7 && !current.is_empty() {
                r_4e5.push((current, r_4dr));
                current = format!("    {word}");
            } else {
                current = r_4dt;
            }
        }
        r_4e5.push((current, r_4dr));
    }
    let mut r_4eh: Vec<(char, u16, u16)> = Vec::new();
    let mut r_4ed = std::collections::BTreeSet::new();
    for (text, _) in &r_4e5 {
        for c in text.chars() {
            if r_4ed.insert(c) {
                let run = f_2eo(&r_4dz, &c.to_string())?;
                let (gid, r_4dq) = run.glyphs[0];
                r_4eh.push((c, gid, r_4dq));
            }
        }
    }
    let mut r_4dx = Document::with_version("1.7");
    let r_4eb = r_4dx.new_object_id();
    let r_4e1 = f_qn(&mut r_4dx, r_4e2, &r_4eh);
    let mut r_4e8 = vec![Operation::new("BT", vec![])];
    let mut y = C_0 - r_4e6;
    for (text, size) in &r_4e5 {
        y -= leading;
        if text.is_empty() {
            continue;
        }
        let run = f_2eo(&r_4dz, text)?;
        let x = if (*size - r_4eg).abs() < f32::EPSILON {
            ((C_1 - f_2bf(&run, *size)) / 2.0).max(r_4e6)
        } else {
            r_4e6
        };
        r_4e8.push(Operation::new(
            "Tf",
            vec![Object::Name(b"F0".to_vec()), (*size).into()],
        ));
        r_4e8.push(Operation::new(
            "Tm",
            vec![1.into(), 0.into(), 0.into(), 1.into(), x.into(), y.into()],
        ));
        r_4e8.push(Operation::new(
            "Tj",
            vec![Object::String(f_zc(&run), StringFormat::Hexadecimal)],
        ));
    }
    r_4e8.push(Operation::new("ET", vec![]));
    let content = Content { operations: r_4e8 };
    let r_4dv = content
        .encode()
        .map_err(|e| PdfError::Encode(e.to_string()))?;
    let r_4dw = r_4dx.add_object(Stream::new(dictionary! {}, r_4dv));
    let r_4e9 = r_4dx.add_object(dictionary! {
        "Type" => "Page", "Parent" => r_4eb, "MediaBox" => vec![0.into(), 0
        .into(), C_1.into(), C_0.into()], "Contents" => r_4dw, "Resources" =>
        dictionary! { "Font" => dictionary! { "F0" => r_4e1 } },
    });
    let r_4ea = dictionary! {
        "Type" => "Pages", "Kids" => vec![Object::Reference(r_4e9)], "Count" => 1,
    };
    r_4dx.objects.insert(r_4eb, Object::Dictionary(r_4ea));
    let r_4ec = pdfa::f_qs(&mut r_4dx, Some(r_4ef));
    let r_4du = r_4dx.add_object(dictionary! {
        "Type" => "Catalog", "Pages" => r_4eb, "Metadata" => r_4ec.metadata,
        "OutputIntents" => r_4ec.output_intents,
    });
    r_4dx.trailer.set("Root", r_4du);
    pdfa::f_2e3(
        &mut r_4dx,
        &f_2k2(r_4e5.iter().map(|(r_4ee, _)| r_4ee.as_str())),
    );
    let mut r_4ds = Vec::new();
    r_4dx
        .save_to(&mut r_4ds)
        .map_err(|e| PdfError::Save(e.to_string()))?;
    Ok(r_4ds)
}
#[derive(Debug, Clone)]
pub enum FormBlock {
    SectionHeading(String),
    Paragraph(String),
    Field {
        label: String,
        value: String,
    },
    Bullets(Vec<String>),
    Signatures {
        left: Vec<String>,
        right: Vec<String>,
    },
}
struct Tan {
    text: String,
    size: f32,
    center: bool,
    indent: f32,
    leading: f32,
}
const C_7E: f32 = 56.0;
const C_7C: f32 = 11.5;
const C_7D: f32 = 17.0;
fn f_2rm(
    r_4ej: &ttf_parser::Face<'_>,
    text: &str,
    size: f32,
    r_4ek: f32,
    indent_cont: &str,
) -> Result<Vec<String>, PdfError> {
    let mut out = Vec::new();
    let mut current = String::new();
    for word in text.split(' ') {
        let r_4ei = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        let r_4el = f_2eo(r_4ej, &r_4ei)?;
        if f_2bf(&r_4el, size) > r_4ek && !current.is_empty() {
            out.push(current);
            current = format!("{indent_cont}{word}");
        } else {
            current = r_4ei;
        }
    }
    out.push(current);
    Ok(out)
}
pub fn f_ve(
    r_4fc: &str,
    r_4fa: Option<&str>,
    r_4eo: &[FormBlock],
    r_4ex: &[u8],
) -> Result<Vec<u8>, PdfError> {
    render_form(r_4fc, r_4fa, r_4eo, r_4ex, true)
}
/// Như `f_ve` nhưng không in quốc hiệu (tài liệu kỹ thuật kèm gói tin, không phải biên bản).
pub fn render_form_plain(
    title: &str,
    subtitle: Option<&str>,
    blocks: &[FormBlock],
    font: &[u8],
) -> Result<Vec<u8>, PdfError> {
    render_form(title, subtitle, blocks, font, false)
}
fn render_form(
    r_4fc: &str,
    r_4fa: Option<&str>,
    r_4eo: &[FormBlock],
    r_4ex: &[u8],
    quoc_hieu: bool,
) -> Result<Vec<u8>, PdfError> {
    let r_4eu = ttf_parser::Face::parse(r_4ex, 0).map_err(|e| PdfError::BadFont(e.to_string()))?;
    let r_4f1 = C_1 - C_7E * 2.0;
    let mut r_4f0: Vec<Tan> = Vec::new();
    let mut r_4f6 = |text: String, size: f32, center: bool, indent: f32, leading: f32| {
        r_4f0.push(Tan {
            text,
            size,
            center,
            indent,
            leading,
        });
    };
    if quoc_hieu {
        r_4f6(
            "CỘNG HOÀ XÃ HỘI CHỦ NGHĨA VIỆT NAM".into(),
            12.0,
            true,
            0.0,
            C_7D,
        );
        r_4f6(
            "Độc lập - Tự do - Hạnh phúc".into(),
            11.5,
            true,
            0.0,
            C_7D,
        );
        r_4f6(String::new(), C_7C, false, 0.0, C_7D);
    }
    r_4f6(r_4fc.to_uppercase(), 14.0, true, 0.0, C_7D + 3.0);
    if let Some(r_4f9) = r_4fa {
        r_4f6(r_4f9.to_string(), 10.5, true, 0.0, C_7D);
    }
    r_4f6(String::new(), C_7C, false, 0.0, C_7D);
    for r_4en in r_4eo {
        match r_4en {
            FormBlock::SectionHeading(r_4ey) => {
                r_4f6(String::new(), C_7C, false, 0.0, 6.0);
                for l in f_2rm(&r_4eu, r_4ey, 12.5, r_4f1, "")? {
                    r_4f6(l, 12.5, false, 0.0, C_7D + 2.0);
                }
            }
            FormBlock::Paragraph(r_4fb) => {
                for l in f_2rm(&r_4eu, r_4fb, C_7C, r_4f1, "")? {
                    r_4f6(l, C_7C, false, 0.0, C_7D);
                }
            }
            FormBlock::Field { label, value } => {
                let full = format!("{label}: {value}");
                for l in f_2rm(&r_4eu, &full, C_7C, r_4f1 - 12.0, "    ")? {
                    r_4f6(l, C_7C, false, 12.0, C_7D);
                }
            }
            FormBlock::Bullets(r_4ez) => {
                for item in r_4ez {
                    let full = format!("- {item}");
                    for l in f_2rm(&r_4eu, &full, C_7C, r_4f1 - 24.0, "  ")? {
                        r_4f6(l, C_7C, false, 24.0, C_7D);
                    }
                }
            }
            FormBlock::Signatures { left, right } => {
                r_4f6(String::new(), C_7C, false, 0.0, C_7D);
                let r_4f7 = left.len().max(right.len());
                for i in 0..r_4f7 {
                    let l = left.get(i).cloned().unwrap_or_default();
                    let r = right.get(i).cloned().unwrap_or_default();
                    r_4f6(format!("{l}\u{0009}{r}"), C_7C, false, -1.0, C_7D);
                }
                r_4f6(String::new(), C_7C, false, 0.0, 60.0);
            }
        }
    }
    let mut doc = Document::with_version("1.7");
    let r_4f4 = doc.new_object_id();
    let mut r_4fd: Vec<(char, u16, u16)> = Vec::new();
    let mut r_4f8 = std::collections::BTreeSet::new();
    for line in &r_4f0 {
        for c in line.text.chars().filter(|c| *c != '\u{0009}') {
            if r_4f8.insert(c) {
                let run = f_2eo(&r_4eu, &c.to_string())?;
                let (gid, r_4em) = run.glyphs[0];
                r_4fd.push((c, gid, r_4em));
            }
        }
    }
    let r_4ew = f_qn(&mut doc, r_4ex, &r_4fd);
    let mut page_ids: Vec<Object> = Vec::new();
    let mut ops: Vec<Operation> = vec![Operation::new("BT", vec![])];
    let mut y = C_0 - C_7E;
    let r_4ev = |doc: &mut Document, ops: &mut Vec<Operation>, page_ids: &mut Vec<Object>| {
        ops.push(Operation::new("ET", vec![]));
        let content = Content {
            operations: std::mem::take(ops),
        };
        let bytes = content.encode().expect("encode content");
        let r_4er = doc.add_object(Stream::new(dictionary! {}, bytes));
        let r_4f2 = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => r_4f4, "MediaBox" => vec![0.into(), 0
            .into(), C_1.into(), C_0.into()], "Contents" => r_4er, "Resources" =>
            dictionary! { "Font" => dictionary! { "F0" => r_4ew } },
        });
        page_ids.push(Object::Reference(r_4f2));
        *ops = vec![Operation::new("BT", vec![])];
    };
    let r_4et = |ops: &mut Vec<Operation>, text: &str, size: f32, x: f32, y: f32| {
        if text.is_empty() {
            return Ok::<(), PdfError>(());
        }
        let run = f_2eo(&r_4eu, text)?;
        ops.push(Operation::new(
            "Tf",
            vec![Object::Name(b"F0".to_vec()), size.into()],
        ));
        ops.push(Operation::new(
            "Tm",
            vec![1.into(), 0.into(), 0.into(), 1.into(), x.into(), y.into()],
        ));
        ops.push(Operation::new(
            "Tj",
            vec![Object::String(f_zc(&run), StringFormat::Hexadecimal)],
        ));
        Ok(())
    };
    for line in &r_4f0 {
        y -= line.leading;
        if y < C_7E {
            r_4ev(&mut doc, &mut ops, &mut page_ids);
            y = C_0 - C_7E - line.leading;
        }
        if line.text.is_empty() {
            continue;
        }
        if line.indent < 0.0 {
            let (l, r) = line
                .text
                .split_once('\u{0009}')
                .unwrap_or((line.text.as_str(), ""));
            if !l.is_empty() {
                let run = f_2eo(&r_4eu, l)?;
                let cx = (C_1 / 4.0 - f_2bf(&run, line.size) / 2.0).max(C_7E);
                r_4et(&mut ops, l, line.size, cx, y)?;
            }
            if !r.is_empty() {
                let run = f_2eo(&r_4eu, r)?;
                let cx = (C_1 * 3.0 / 4.0 - f_2bf(&run, line.size) / 2.0).max(C_1 / 2.0 + 10.0);
                r_4et(&mut ops, r, line.size, cx, y)?;
            }
            continue;
        }
        let x = if line.center {
            let run = f_2eo(&r_4eu, &line.text)?;
            ((C_1 - f_2bf(&run, line.size)) / 2.0).max(C_7E)
        } else {
            C_7E + line.indent
        };
        r_4et(&mut ops, &line.text, line.size, x, y)?;
    }
    r_4ev(&mut doc, &mut ops, &mut page_ids);
    let r_4es = page_ids.len() as i64;
    let r_4f3 = dictionary! {
        "Type" => "Pages", "Kids" => page_ids, "Count" => r_4es,
    };
    doc.objects.insert(r_4f4, Object::Dictionary(r_4f3));
    let r_4f5 = pdfa::f_qs(&mut doc, Some(r_4fc));
    let r_4eq = doc.add_object(dictionary! {
        "Type" => "Catalog", "Pages" => r_4f4, "Metadata" => r_4f5.metadata,
        "OutputIntents" => r_4f5.output_intents,
    });
    doc.trailer.set("Root", r_4eq);
    pdfa::f_2e3(&mut doc, &f_2k2(r_4f0.iter().map(|l| l.text.as_str())));
    let mut r_4ep = Vec::new();
    doc.save_to(&mut r_4ep)
        .map_err(|e| PdfError::Save(e.to_string()))?;
    Ok(r_4ep)
}
fn f_18n(r_4ff: &Document, r_4fj: lopdf::ObjectId) -> Option<[f32; 4]> {
    let mut dict = r_4ff.get_dictionary(r_4fj).ok()?;
    for _ in 0..16 {
        if let Ok(r_4fi) = dict.get(b"MediaBox") {
            let r_4fe = match r_4fi {
                Object::Array(a) => a.clone(),
                Object::Reference(r_4fh) => r_4ff.get_object(*r_4fh).ok()?.as_array().ok()?.clone(),
                _ => return None,
            };
            let mut out = [0f32; 4];
            for (i, r_4fl) in r_4fe.iter().take(4).enumerate() {
                out[i] = match r_4fl {
                    Object::Integer(n) => *n as f32,
                    Object::Real(r_4fg) => *r_4fg,
                    _ => return None,
                };
            }
            return Some(out);
        }
        let r_4fk = dict.get(b"Parent").and_then(Object::as_reference).ok()?;
        dict = r_4ff.get_dictionary(r_4fk).ok()?;
    }
    None
}
fn f_23r(r_4fn: &mut Document, r_4fr: lopdf::ObjectId, content: Vec<u8>) -> Result<(), PdfError> {
    let page = r_4fn
        .get_dictionary(r_4fr)
        .map_err(|e| PdfError::Encode(e.to_string()))?;
    let mut r_4fp: Vec<Object> = match page.get(b"Contents") {
        Ok(Object::Reference(r_4fo)) => vec![Object::Reference(*r_4fo)],
        Ok(Object::Array(r_4fm)) => r_4fm.clone(),
        _ => vec![],
    };
    let r_4fq = r_4fn.add_object(Object::Stream(Stream::new(dictionary! {}, content)));
    r_4fp.insert(0, Object::Reference(r_4fq));
    let r_4fs = r_4fn
        .get_object_mut(r_4fr)
        .and_then(Object::as_dict_mut)
        .map_err(|e| PdfError::Encode(e.to_string()))?;
    r_4fs.set("Contents", r_4fp);
    Ok(())
}
fn f_qv(
    r_4ft: &mut Document,
    r_4fz: lopdf::ObjectId,
    r_4fw: lopdf::ObjectId,
) -> Result<(), PdfError> {
    let r_4fu = |e: lopdf::Error| PdfError::Encode(e.to_string());
    let page = r_4ft.get_dictionary(r_4fz).map_err(r_4fu)?;
    let r_4fy = page.get(b"Resources").ok().cloned();
    let r_4g1: Dictionary = match r_4fy {
        Some(Object::Dictionary(d)) => d,
        Some(Object::Reference(id)) => r_4ft.get_dictionary(id).map_err(r_4fu)?.clone(),
        _ => {
            let (dict, r_4fx) = r_4ft
                .get_page_resources(r_4fz)
                .map_err(|e| PdfError::Encode(e.to_string()))?;
            if let Some(d) = dict {
                d.clone()
            } else if let Some(id) = r_4fx.first() {
                r_4ft.get_dictionary(*id).map_err(r_4fu)?.clone()
            } else {
                Dictionary::new()
            }
        }
    };
    let mut r_4g2 = r_4g1;
    let r_4fv = match r_4g2.get(b"Font") {
        Ok(Object::Dictionary(d)) => Some(d.clone()),
        Ok(Object::Reference(id)) => Some(r_4ft.get_dictionary(*id).map_err(r_4fu)?.clone()),
        _ => None,
    };
    let mut fonts: Dictionary = r_4fv.unwrap_or_default();
    fonts.set("DGXWM", Object::Reference(r_4fw));
    r_4g2.set("Font", Object::Dictionary(fonts));
    let r_4g0 = r_4ft
        .get_object_mut(r_4fz)
        .and_then(Object::as_dict_mut)
        .map_err(r_4fu)?;
    r_4g0.set("Resources", Object::Dictionary(r_4g2));
    Ok(())
}
pub fn f_2h7(pdf: &[u8], r_4gh: &str, r_4g4: bool, r_4gc: &[u8]) -> Result<Vec<u8>, PdfError> {
    let mut r_4g9 =
        Document::load_mem(pdf).map_err(|e| PdfError::NoPageImage(format!("load PDF: {e}")))?;
    let r_4ga = ttf_parser::Face::parse(r_4gc, 0).map_err(|e| PdfError::BadFont(e.to_string()))?;
    let r_4gm = r_4g9
        .trailer
        .get(b"Root")
        .and_then(Object::as_reference)
        .map_err(|e| PdfError::Encode(e.to_string()))?;
    if let Ok(r_4g8) = r_4g9.get_object_mut(r_4gm).and_then(Object::as_dict_mut) {
        r_4g8.remove(b"AcroForm");
        r_4g8.remove(b"Perms");
    }
    let r_4gk: Vec<lopdf::ObjectId> = r_4g9.get_pages().into_values().collect();
    for &pid in &r_4gk {
        let annots: Option<Vec<Object>> = r_4g9
            .get_dictionary(pid)
            .ok()
            .and_then(|r_4gj| r_4gj.get(b"Annots").ok())
            .and_then(|a| match a {
                Object::Array(arr) => Some(arr.clone()),
                Object::Reference(id) => r_4g9
                    .get_object(*id)
                    .ok()
                    .and_then(|r_4gf| r_4gf.as_array().ok())
                    .cloned(),
                _ => None,
            });
        if let Some(arr) = annots {
            let r_4ge: Vec<Object> = arr
                .into_iter()
                .filter(|a| {
                    let dict = match a {
                        Object::Reference(id) => r_4g9.get_dictionary(*id).ok(),
                        Object::Dictionary(d) => Some(d),
                        _ => None,
                    };
                    !dict.is_some_and(|d| {
                        d.get(b"Subtype").and_then(Object::as_name).ok() == Some(b"Widget")
                    })
                })
                .collect();
            if let Ok(r_4gl) = r_4g9.get_object_mut(pid).and_then(Object::as_dict_mut) {
                r_4gl.set("Annots", r_4ge);
            }
        }
    }
    let r_4gi = f_2eo(&r_4ga, r_4gh)?;
    let r_4g6 = "BẢN SAO";
    let r_4g5 = f_2eo(&r_4ga, r_4g6)?;
    let mut r_4go: Vec<(char, u16, u16)> = Vec::new();
    for (text, r_4gn) in [(r_4gh, &r_4gi), (r_4g6, &r_4g5)] {
        for (c, (gid, r_4g3)) in text.chars().zip(r_4gn.glyphs.iter()) {
            if !r_4go.iter().any(|(_, g, _)| g == gid) {
                r_4go.push((c, *gid, *r_4g3));
            }
        }
    }
    let r_4gb = f_qn(&mut r_4g9, r_4gc, &r_4go);
    let org_size = 8.0_f32;
    let ban_sao_size = 10.0_f32;
    for (r_4gd, &pid) in r_4gk.iter().enumerate() {
        let [r_4gp, r_4gr, r_4gq, r_4gs] = f_18n(&r_4g9, pid).unwrap_or([0.0, 0.0, C_1, C_0]);
        f_qv(&mut r_4g9, pid, r_4gb)?;
        f_23r(&mut r_4g9, pid, b"q\n".to_vec())?;
        let w = f_2bf(&r_4gi, org_size);
        let cx = r_4gp + ((r_4gq - r_4gp) - w) / 2.0;
        let mut r_4gg = format!(
            "Q\nq\n0 g\nBT /DGXWM {org_size} Tf {cx:.2} {y:.2} Td <{hex}> Tj ET\n",
            y = r_4gr + 20.0,
            hex = f_1gw(&f_zc(&r_4gi)),
        );
        if r_4g4 && r_4gd == 0 {
            let w = f_2bf(&r_4g5, ban_sao_size);
            r_4gg.push_str(&format!(
                "BT /DGXWM {ban_sao_size} Tf {x:.2} {y:.2} Td <{hex}> Tj ET\n",
                x = r_4gq - 40.0 - w,
                y = r_4gs - 30.0,
                hex = f_1gw(&f_zc(&r_4g5)),
            ));
        }
        r_4gg.push_str("Q\n");
        r_4g9
            .add_page_contents(pid, r_4gg.into_bytes())
            .map_err(|e| PdfError::Encode(e.to_string()))?;
    }
    let mut r_4g7 = Vec::new();
    r_4g9
        .save_to(&mut r_4g7)
        .map_err(|e| PdfError::Save(e.to_string()))?;
    Ok(r_4g7)
}

/// Dấu hiệu PDF/A tối thiểu của một tệp: loại tự khai trong XMP (`pdfaid:part`/`conformance`),
/// có OutputIntent (hồ sơ màu) hay không, và phiên bản PDF ở dòng đầu. Không thay cho veraPDF.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PdfaMarkers {
    pub declared: Option<String>,
    pub has_output_intent: bool,
    pub version: Option<String>,
}
fn pdfaid_from_xmp(xmp: &[u8]) -> Option<String> {
    let find = |hay: &[u8], needle: &[u8]| hay.windows(needle.len()).position(|w| w == needle);
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
pub fn pdfa_markers(bytes: &[u8]) -> PdfaMarkers {
    let find = |hay: &[u8], needle: &[u8]| hay.windows(needle.len()).any(|w| w == needle);
    let version = bytes
        .get(..8)
        .and_then(|h| std::str::from_utf8(h).ok())
        .and_then(|h| h.strip_prefix("%PDF-"))
        .map(|v| v.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect::<String>())
        .filter(|v| !v.is_empty());
    let mut m = PdfaMarkers { declared: pdfaid_from_xmp(bytes), has_output_intent: find(bytes, b"/OutputIntents"), version };
    if m.declared.is_none() && find(bytes, b"/Metadata") {
        if let Ok(doc) = Document::load_mem(bytes) {
            if let Ok(cat) = doc.catalog() {
                if let Some(id) = cat.get(b"Metadata").ok().and_then(|o| o.as_reference().ok()) {
                    if let Ok(Object::Stream(st)) = doc.get_object(id) {
                        let xmp = st.decompressed_content().unwrap_or_else(|_| st.content.clone());
                        m.declared = pdfaid_from_xmp(&xmp);
                    }
                }
                m.has_output_intent = m.has_output_intent || cat.get(b"OutputIntents").is_ok();
            }
        }
    }
    m
}
