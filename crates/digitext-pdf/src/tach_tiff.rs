// Copyright (c) 2026 DigiText. All rights reserved.
//! Tách PDF (đã ký số) thành TIFF mỗi trang một tệp — bước "Tách TIFF" giữa Ký số và Đóng gói (tài liệu Đảng).
//! Lấy NGUYÊN ảnh quét nhúng trong trang (JPEG/DCTDecode hoặc Flate 8 bit), không vẽ lại trang: giữ đúng kích
//! thước điểm ảnh, dpi, màu của bản quét và không dính ô chữ ký số (ô chữ ký là chú thích, không nằm trong ảnh).
use std::io::Cursor;

use lopdf::{Document, Object};
use sha2::{Digest, Sha256};
use tiff::encoder::{colortype, Compression, Rational, TiffEncoder};
use tiff::tags::ResolutionUnit;

use crate::{fix_jpeg_sof_dims, ParsedPdf, PdfError};

/// Dpi ghi vào TIFF khi không suy được từ trang (giống tool cũ: 300).
pub const DEFAULT_TIFF_DPI: u32 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TiffCompression {
    #[default]
    Lzw,
    None,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TiffOpts {
    /// Ép mọi trang về RGB 24 bit (như tool cũ). Mặc định giữ màu gốc — bản quét xám ra TIFF xám 8 bit.
    pub force_rgb: bool,
    pub compression: TiffCompression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TiffColor {
    Rgb24,
    Gray8,
}

#[derive(Debug, Clone)]
pub struct PageTiff {
    /// Số trang, đánh từ 1.
    pub page: u32,
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// Dpi ghi trong tệp; `dpi_guessed` = không suy được từ PDF, đã ghi `DEFAULT_TIFF_DPI`.
    pub dpi: u32,
    pub dpi_guessed: bool,
    pub color: TiffColor,
    /// SHA-256 của `bytes`, chữ thường hex.
    pub sha256: String,
}

enum Raw {
    Jpeg(Vec<u8>),
    Pixels { w: u32, h: u32, color: TiffColor, data: Vec<u8> },
}

fn deref<'a>(doc: &'a Document, o: &'a Object) -> &'a Object {
    match o {
        Object::Reference(id) => doc.get_object(*id).unwrap_or(o),
        o => o,
    }
}

fn num(doc: &Document, d: &lopdf::Dictionary, key: &[u8]) -> Option<u32> {
    u32::try_from(deref(doc, d.get(key).ok()?).as_i64().ok()?).ok()
}

fn filters(doc: &Document, d: &lopdf::Dictionary) -> Vec<Vec<u8>> {
    match d.get(b"Filter").map(|o| deref(doc, o)) {
        Ok(Object::Name(n)) => vec![n.clone()],
        Ok(Object::Array(a)) => a.iter().filter_map(|f| f.as_name().ok().map(<[u8]>::to_vec)).collect(),
        _ => Vec::new(),
    }
}

/// Số kênh của không gian màu ảnh: DeviceGray/CalGray = 1, DeviceRGB/CalRGB = 3, ICCBased theo /N.
fn components(doc: &Document, d: &lopdf::Dictionary) -> Option<u32> {
    let cs = deref(doc, d.get(b"ColorSpace").ok()?);
    match cs {
        Object::Name(n) => match n.as_slice() {
            b"DeviceGray" | b"CalGray" | b"G" => Some(1),
            b"DeviceRGB" | b"CalRGB" | b"RGB" => Some(3),
            _ => None,
        },
        Object::Array(a) => match a.first().and_then(|o| o.as_name().ok()) {
            Some(b"ICCBased") => {
                let s = deref(doc, a.get(1)?).as_stream().ok()?;
                num(doc, &s.dict, b"N")
            }
            Some(b"CalGray") => Some(1),
            Some(b"CalRGB") => Some(3),
            _ => None,
        },
        _ => None,
    }
}

/// Ảnh quét lớn nhất của trang (theo số điểm ảnh) + bề ngang trang (pt) theo MediaBox để suy dpi.
fn page_scan(parsed: &ParsedPdf, page_no: u32) -> Result<(Raw, Option<f32>), PdfError> {
    let doc = &parsed.0;
    let pages = doc.get_pages();
    let pid = *pages.get(&page_no).ok_or_else(|| {
        PdfError::NoPageImage(format!("không có trang {page_no} (PDF {} trang)", pages.len()))
    })?;
    let page = doc.get_dictionary(pid).map_err(|e| PdfError::NoPageImage(e.to_string()))?;
    let dict_of = |o: &Object| -> Option<lopdf::Dictionary> {
        match deref(doc, o) {
            Object::Dictionary(d) => Some(d.clone()),
            _ => None,
        }
    };
    // Resources có thể kế thừa từ nút Pages cha.
    let mut res = page.get(b"Resources").ok().and_then(dict_of);
    let mut parent = page.get(b"Parent").ok().and_then(|o| o.as_reference().ok());
    while res.is_none() {
        let Some(p) = parent else { break };
        let Ok(pd) = doc.get_dictionary(p) else { break };
        res = pd.get(b"Resources").ok().and_then(dict_of);
        parent = pd.get(b"Parent").ok().and_then(|o| o.as_reference().ok());
    }
    let render = |reason: String| PdfError::NeedsRender { page: page_no, reason };

    // Mọi ảnh của trang, kể cả ảnh lồng trong Form XObject (PDF nén nhiều lớp MRC hay để ảnh trong form).
    let mut images: Vec<lopdf::ObjectId> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut stack: Vec<lopdf::Dictionary> = res.into_iter().collect();
    while let Some(r) = stack.pop() {
        let Some(xobj) = r.get(b"XObject").ok().and_then(dict_of) else { continue };
        for (_name, obj) in xobj.iter() {
            let Ok(id) = obj.as_reference() else { continue };
            if !seen.insert(id) || seen.len() > 500 {
                continue;
            }
            let Ok(s) = doc.get_object(id).and_then(|o| o.as_stream()) else { continue };
            match s.dict.get(b"Subtype").and_then(|o| o.as_name()) {
                Ok(b"Image") => images.push(id),
                Ok(b"Form") => {
                    if let Some(fr) = s.dict.get(b"Resources").ok().and_then(dict_of) {
                        stack.push(fr);
                    }
                }
                _ => {}
            }
        }
    }
    let page_box = page
        .get(b"MediaBox")
        .ok()
        .map(|o| deref(doc, o))
        .and_then(|o| o.as_array().ok())
        .and_then(|a| {
            let f = |o: &Object| o.as_float().ok().or(o.as_i64().ok().map(|v| v as f32));
            let g = |i: usize| a.get(i).map(|o| deref(doc, o)).and_then(f);
            Some((g(2)? - g(0)?, g(3)? - g(1)?))
        });
    // Chỉ "rút ảnh gốc" khi trang đúng là MỘT ảnh quét phủ cả trang; còn lại (nhiều lớp, JPEG 2000, ảnh có
    // mặt nạ, trang chữ không ảnh…) phải vẽ lại trang để không mất lớp nào.
    let id = match images.as_slice() {
        [] => return Err(render("trang không có ảnh quét".into())),
        [one] => *one,
        many => return Err(render(format!("trang ghép {} lớp ảnh", many.len()))),
    };
    let s = doc.get_object(id).and_then(|o| o.as_stream()).map_err(|e| PdfError::NoPageImage(e.to_string()))?;
    let d = &s.dict;
    if d.has(b"SMask") || d.has(b"Mask") {
        return Err(render("ảnh có lớp mặt nạ".into()));
    }
    let (Some(w), Some(h)) = (num(doc, d, b"Width"), num(doc, d, b"Height")) else {
        return Err(render("ảnh thiếu kích thước".into()));
    };
    if let Some((pw, ph)) = page_box {
        let (ra, rp) = (w as f32 / h as f32, pw / ph);
        // Ảnh không cùng tỉ lệ khổ trang (ảnh minh hoạ trong trang chữ, trang xoay…) → vẽ lại.
        if !(rp > 0.0 && ((ra / rp) - 1.0).abs() <= 0.03) || (w as f32) < pw {
            return Err(render("ảnh không phủ cả trang".into()));
        }
    }
    let f = filters(doc, d);
    let unpack = || if f.is_empty() || f.len() == 1 && f[0] == b"DCTDecode" { Ok(s.content.clone()) } else { s.decompressed_content() };
    let raw = if f.last().is_some_and(|x| x == b"DCTDecode") && f[..f.len() - 1].iter().all(|x| x == b"FlateDecode") {
        let mut jpeg = unpack().map_err(|e| render(format!("giải nén ảnh: {e}")))?;
        fix_jpeg_sof_dims(&mut jpeg, w, h);
        Raw::Jpeg(jpeg)
    } else if f.iter().all(|x| x == b"FlateDecode") && num(doc, d, b"BitsPerComponent") == Some(8) && !d.has(b"DecodeParms") {
        let color = match components(doc, d) {
            Some(1) => TiffColor::Gray8,
            Some(3) => TiffColor::Rgb24,
            _ => return Err(render("không gian màu ảnh không hỗ trợ".into())),
        };
        let data = unpack().map_err(|e| render(format!("giải nén ảnh: {e}")))?;
        let need = w as usize * h as usize * if color == TiffColor::Gray8 { 1 } else { 3 };
        if data.len() < need {
            return Err(render("dữ liệu ảnh thiếu".into()));
        }
        Raw::Pixels { w, h, color, data }
    } else {
        let names: Vec<String> = f.iter().map(|x| String::from_utf8_lossy(x).into_owned()).collect();
        return Err(render(format!("ảnh nén {}", if names.is_empty() { "dạng khác".into() } else { names.join("+") })));
    };
    Ok((raw, page_box.map(|b| b.0)))
}

fn encode(w: u32, h: u32, color: TiffColor, data: &[u8], dpi: u32, opts: TiffOpts) -> Result<Vec<u8>, PdfError> {
    let err = |e: tiff::TiffError| PdfError::Encode(format!("ghi TIFF: {e}"));
    let mut out = Cursor::new(Vec::new());
    let comp = match opts.compression {
        TiffCompression::Lzw => Compression::Lzw,
        TiffCompression::None => Compression::Uncompressed,
    };
    let mut enc = TiffEncoder::new(&mut out).map_err(err)?.with_compression(comp);
    let res = Rational { n: dpi, d: 1 };
    match color {
        TiffColor::Rgb24 => {
            let mut im = enc.new_image::<colortype::RGB8>(w, h).map_err(err)?;
            im.resolution(ResolutionUnit::Inch, res);
            im.write_data(data).map_err(err)?;
        }
        TiffColor::Gray8 => {
            let mut im = enc.new_image::<colortype::Gray8>(w, h).map_err(err)?;
            im.resolution(ResolutionUnit::Inch, res);
            im.write_data(data).map_err(err)?;
        }
    }
    Ok(out.into_inner())
}

fn gray_to_rgb(g: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(g.len() * 3);
    for &p in g {
        v.extend_from_slice(&[p, p, p]);
    }
    v
}

/// TIFF của một trang (đánh số từ 1).
pub fn page_tiff(parsed: &ParsedPdf, page_no: u32, opts: TiffOpts) -> Result<PageTiff, PdfError> {
    let (raw, page_w_pt) = page_scan(parsed, page_no)?;
    let (w, h, mut color, mut data) = match raw {
        Raw::Pixels { w, h, color, data } => (w, h, color, data),
        Raw::Jpeg(jpeg) => {
            let img = image::load_from_memory_with_format(&jpeg, image::ImageFormat::Jpeg)
                .map_err(|e| PdfError::NoPageImage(format!("trang {page_no}: giải mã JPEG lỗi: {e}")))?;
            let (w, h) = (img.width(), img.height());
            match img {
                image::DynamicImage::ImageLuma8(g) => (w, h, TiffColor::Gray8, g.into_raw()),
                other => (w, h, TiffColor::Rgb24, other.into_rgb8().into_raw()),
            }
        }
    };
    let dpi = page_w_pt.and_then(|pt| crate::f_17h(w, pt));
    if opts.force_rgb && color == TiffColor::Gray8 {
        data = gray_to_rgb(&data);
        color = TiffColor::Rgb24;
    }
    let bytes = encode(w, h, color, &data, dpi.unwrap_or(DEFAULT_TIFF_DPI), opts)?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    Ok(PageTiff {
        page: page_no,
        bytes,
        width: w,
        height: h,
        dpi: dpi.unwrap_or(DEFAULT_TIFF_DPI),
        dpi_guessed: dpi.is_none(),
        color,
        sha256,
    })
}

/// TIFF từ ảnh RGB 24 bit đã có sẵn (trang vẽ lại bằng bộ vẽ PDF ở bên gọi — xem `PdfError::NeedsRender`).
pub fn tiff_from_rgb(page_no: u32, w: u32, h: u32, rgb: &[u8], dpi: u32, opts: TiffOpts) -> Result<PageTiff, PdfError> {
    if rgb.len() < w as usize * h as usize * 3 {
        return Err(PdfError::Encode(format!("trang {page_no}: ảnh vẽ lại thiếu dữ liệu")));
    }
    let bytes = encode(w, h, TiffColor::Rgb24, rgb, dpi, opts)?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    Ok(PageTiff { page: page_no, bytes, width: w, height: h, dpi, dpi_guessed: false, color: TiffColor::Rgb24, sha256 })
}

/// Số trang của PDF.
pub fn page_count(parsed: &ParsedPdf) -> u32 {
    parsed.0.get_pages().len() as u32
}

/// Tách cả PDF; lỗi ở bất kỳ trang nào làm hỏng cả tệp (không xuất bộ TIFF thiếu trang).
pub fn split_pdf_to_tiffs(pdf: &[u8], opts: TiffOpts) -> Result<Vec<PageTiff>, PdfError> {
    let parsed = ParsedPdf::parse(pdf)?;
    let n = page_count(&parsed);
    if n == 0 {
        return Err(PdfError::Empty);
    }
    let out = (1..=n).map(|p| page_tiff(&parsed, p, opts)).collect::<Result<Vec<_>, _>>()?;
    debug_assert_eq!(out.len() as u32, n);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{f_vz, Tgh};

    fn jpeg_rgb(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbImage::from_fn(w, h, |x, y| image::Rgb([(x % 256) as u8, (y % 256) as u8, 90]));
        let mut out = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 95).encode_image(&img).unwrap();
        out
    }
    fn jpeg_gray(w: u32, h: u32) -> Vec<u8> {
        let img = image::GrayImage::from_fn(w, h, |x, _| image::Luma([(x % 256) as u8]));
        let mut out = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 95).encode_image(&img).unwrap();
        out
    }
    fn pdf(pages: Vec<(Vec<u8>, u32, u32, u32)>) -> Vec<u8> {
        let p: Vec<Tgh> = pages
            .into_iter()
            .map(|(jpeg, w, h, dpi)| Tgh { jpeg, width_px: w, height_px: h, dpi, spans: vec![] })
            .collect();
        f_vz(&p).unwrap()
    }
    fn read_tiff(b: &[u8]) -> (u32, u32, tiff::ColorType, Vec<u8>, f64) {
        let mut d = tiff::decoder::Decoder::new(Cursor::new(b)).unwrap();
        let (w, h) = d.dimensions().unwrap();
        let ct = d.colortype().unwrap();
        let xres = match d.get_tag(tiff::tags::Tag::XResolution).unwrap() {
            tiff::decoder::ifd::Value::Rational(n, den) => n as f64 / den as f64,
            v => panic!("XResolution {v:?}"),
        };
        let px = match d.read_image().unwrap() {
            tiff::decoder::DecodingResult::U8(v) => v,
            _ => panic!("không phải 8 bit"),
        };
        (w, h, ct, px, xres)
    }

    #[test]
    fn tach_rgb_giu_kich_thuoc_dpi_va_diem_anh() {
        let j = jpeg_rgb(240, 320);
        let out = split_pdf_to_tiffs(&pdf(vec![(j.clone(), 240, 320, 300), (j.clone(), 240, 320, 300)]), TiffOpts::default()).unwrap();
        assert_eq!(out.len(), 2);
        let t = &out[0];
        assert_eq!((t.page, t.width, t.height, t.dpi, t.dpi_guessed, t.color), (1, 240, 320, 300, false, TiffColor::Rgb24));
        assert_eq!(t.sha256.len(), 64);
        let (w, h, ct, px, xres) = read_tiff(&t.bytes);
        assert_eq!((w, h, ct, xres), (240, 320, tiff::ColorType::RGB(8), 300.0));
        // Điểm ảnh y hệt JPEG giải mã — không co giãn, không nén lại.
        let goc = image::load_from_memory(&j).unwrap().into_rgb8().into_raw();
        assert_eq!(px, goc);
    }

    #[test]
    fn anh_xam_giu_xam_tru_khi_ep_rgb() {
        let p = pdf(vec![(jpeg_gray(200, 100), 200, 100, 200)]);
        let t = &split_pdf_to_tiffs(&p, TiffOpts::default()).unwrap()[0];
        assert_eq!((t.color, t.dpi), (TiffColor::Gray8, 200));
        assert_eq!(read_tiff(&t.bytes).2, tiff::ColorType::Gray(8));
        let t = &split_pdf_to_tiffs(&p, TiffOpts { force_rgb: true, ..Default::default() }).unwrap()[0];
        assert_eq!(t.color, TiffColor::Rgb24);
        assert_eq!(read_tiff(&t.bytes).2, tiff::ColorType::RGB(8));
    }

    #[test]
    fn khong_nen_va_lzw_cung_diem_anh() {
        let p = pdf(vec![(jpeg_rgb(64, 64), 64, 64, 300)]);
        let a = &split_pdf_to_tiffs(&p, TiffOpts::default()).unwrap()[0];
        let b = &split_pdf_to_tiffs(&p, TiffOpts { compression: TiffCompression::None, ..Default::default() }).unwrap()[0];
        assert!(b.bytes.len() >= 64 * 64 * 3);
        assert_eq!(read_tiff(&a.bytes).3, read_tiff(&b.bytes).3);
    }

    #[test]
    fn trang_khong_co_anh_quet_bao_loi() {
        use lopdf::{dictionary, Stream};
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let content = doc.add_object(Stream::new(dictionary! {}, b"BT ET".to_vec()));
        let page = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id, "Contents" => content,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        doc.objects.insert(pages_id, Object::Dictionary(dictionary! {
            "Type" => "Pages", "Kids" => vec![page.into()], "Count" => 1,
        }));
        let cat = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", cat);
        let mut b = Vec::new();
        doc.save_to(&mut b).unwrap();
        let e = split_pdf_to_tiffs(&b, TiffOpts::default()).unwrap_err();
        assert!(e.to_string().contains("không có ảnh quét"), "{e}");
    }
}

#[cfg(test)]
mod thu_that {
    /// `DIGITEXT_TIFF_SAMPLE=<thư mục PDF> cargo test -p digitext-pdf thu_pdf_that -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn thu_pdf_that() {
        let dir = std::env::var("DIGITEXT_TIFF_SAMPLE").expect("DIGITEXT_TIFF_SAMPLE");
        let mut tong = 0;
        let t0 = std::time::Instant::now();
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.extension().and_then(|x| x.to_str()) != Some("pdf") {
                continue;
            }
            let b = std::fs::read(&p).unwrap();
            match super::split_pdf_to_tiffs(&b, Default::default()) {
                Ok(v) => {
                    tong += v.len();
                    let t = &v[0];
                    let mb: usize = v.iter().map(|t| t.bytes.len()).sum();
                    println!("{} · {} trang · {}x{} {}dpi{} {:?} · TIFF {} MB", p.file_name().unwrap().to_string_lossy(), v.len(), t.width, t.height, t.dpi, if t.dpi_guessed { "?" } else { "" }, t.color, mb / 1_000_000);
                }
                Err(e) => println!("{} LỖI {e}", p.display()),
            }
        }
        println!("{tong} trang trong {:.1}s", t0.elapsed().as_secs_f32());
    }
}
