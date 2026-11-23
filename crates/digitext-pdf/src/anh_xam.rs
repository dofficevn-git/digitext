// Copyright (c) 2026 DigiText. All rights reserved.
//! Rút **ảnh xám** của một trang PDF để đo chất lượng bản quét (trang trắng, trang lệch).
//!
//! Khác `page_jpeg` — hàm đó trả nguyên khối JPEG để đưa sang OCR nên chỉ nhận `DCTDecode`.
//! Ở đây chỉ cần mức sáng từng điểm, nên giải nén được thêm:
//!
//! | Nén | Gặp ở đâu |
//! |---|---|
//! | `DCTDecode` | bản quét màu / xám của máy quét đời mới |
//! | `CCITTFaxDecode` (G3, G4) | **bản quét đen trắng** — phần lớn tài liệu lưu trữ cũ |
//! | `FlateDecode`, `LZWDecode`, `RunLengthDecode`, không nén | ảnh do phần mềm ráp lại (pypdf, ImageMagick…) |
//!
//! `JPXDecode` (JPEG 2000) và `JBIG2Decode` thì chịu — phải vẽ lại trang mới đọc được; hai thứ đó
//! trả lỗi nói rõ tên để người dùng biết vì sao không đo được, thay vì im lặng bỏ qua.
//!
//! Hai cái bẫy ngoài chuyện nén, đều từng làm cả tệp "không đo được" dù trang vẫn là ảnh quét:
//!
//! * `/Resources` **thừa kế từ nút `/Pages` cha** (ISO 32000-1 §7.7.3.4) — tệp do pypdf ghép hay
//!   để nguyên như vậy. Phải đi ngược `/Parent` chứ không đọc mỗi trang.
//! * Ảnh nằm trong **Form XObject** lồng một tầng, không nằm thẳng ở `/Resources/XObject` của trang.

use crate::{ParsedPdf, PdfError};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};

/// Ảnh xám 8 bit, kích thước gốc của ảnh nhúng (không phải khổ trang).
#[derive(Debug, Clone)]
pub struct AnhXam {
    /// Mức sáng từng điểm, 0 = đen, 255 = trắng; dài đúng `rong * cao`.
    pub diem: Vec<u8>,
    pub rong: u32,
    pub cao: u32,
}

/// Lồng sâu hơn chừng này thì thôi — PDF thật không ai lồng Form quá vài tầng, còn tệp hỏng thì
/// có thể lồng vòng tròn.
const SAU_TOI_DA: usize = 3;
/// Số điểm ảnh tối đa chịu giải nén. A4 quét 600 dpi là 35 triệu điểm, nên 80 triệu đã rộng rãi;
/// đặt trần để tệp khai `/Width` láo không ngốn hết RAM giữa lúc quét cả thư mục.
const DIEM_TOI_DA: u64 = 80_000_000;

/// Ảnh xám của trang `page_no` (đếm từ 1). Trang nhiều ảnh thì lấy ảnh **to nhất** — trang quét
/// kèm logo hay chữ ký ảnh thì bản quét luôn là cái to nhất.
pub fn page_xam(parsed: &ParsedPdf, page_no: i32) -> Result<AnhXam, PdfError> {
    let doc = &parsed.0;
    let so_trang = u32::try_from(page_no)
        .map_err(|_| PdfError::NoPageImage(format!("page_no {page_no} không hợp lệ")))?;
    let pages = doc.get_pages();
    let id = *pages.get(&so_trang).ok_or_else(|| {
        PdfError::NoPageImage(format!("không có trang {page_no} (PDF {} trang)", pages.len()))
    })?;
    let can_ve = |ly_do: String| PdfError::NeedsRender { page: so_trang, reason: ly_do };

    let res = tai_nguyen(doc, id).ok_or_else(|| can_ve("trang không khai tài nguyên".into()))?;
    let mut anh = Vec::new();
    gom_anh(doc, &res, 0, &mut anh);
    if anh.is_empty() {
        return Err(can_ve("trang không có ảnh nhúng".into()));
    }
    anh.sort_by_key(|st| std::cmp::Reverse(dien_tich(doc, st)));

    let mut ly_do_dau = None;
    for st in anh {
        match giai_anh(doc, st) {
            Ok(a) => return Ok(a),
            Err(e) => ly_do_dau.get_or_insert(e),
        };
    }
    Err(can_ve(ly_do_dau.unwrap_or_else(|| "ảnh không giải nén được".into())))
}

/// Trang có ảnh nhúng nào không — kể cả ảnh không giải nén được.
///
/// Dùng để phân biệt hai kiểu "không đo được": trang **chữ thật** (không ảnh, đọc lệnh vẽ thay)
/// với trang **có ảnh mà chịu** (JBIG2, JPEG 2000). Trang kiểu sau không được phép đọc lệnh vẽ
/// rồi kết luận "không sao" — ảnh nằm đó mà chưa ai nhìn vào thì vẫn là chưa kiểm.
pub fn trang_co_anh(parsed: &ParsedPdf, page_no: i32) -> bool {
    let doc = &parsed.0;
    let Ok(so_trang) = u32::try_from(page_no) else { return false };
    let Some(id) = doc.get_pages().get(&so_trang).copied() else { return false };
    let Some(res) = tai_nguyen(doc, id) else { return false };
    let mut anh = Vec::new();
    gom_anh(doc, &res, 0, &mut anh);
    !anh.is_empty()
}

// ── tìm ảnh ────────────────────────────────────────────────────────────────────────────────────

/// `/Resources` của trang, đi ngược `/Parent` khi trang không tự khai (thừa kế, §7.7.3.4).
fn tai_nguyen(doc: &Document, trang: ObjectId) -> Option<Dictionary> {
    let mut id = trang;
    for _ in 0..64 {
        let d = doc.get_dictionary(id).ok()?;
        if let Some(r) = d.get(b"Resources").ok().and_then(|o| tu_dien(doc, o)) {
            return Some(r);
        }
        id = d.get(b"Parent").ok()?.as_reference().ok()?;
    }
    None
}

/// Mọi ảnh thấy được từ bộ tài nguyên này, kể cả ảnh nằm trong Form XObject lồng bên trong.
fn gom_anh<'a>(doc: &'a Document, res: &Dictionary, sau: usize, ra: &mut Vec<&'a Stream>) {
    if sau > SAU_TOI_DA {
        return;
    }
    let Some(xo) = res.get(b"XObject").ok().and_then(|o| tu_dien(doc, o)) else {
        return;
    };
    for (_ten, o) in xo.iter() {
        let Some(st) = luong(doc, o) else { continue };
        match st.dict.get(b"Subtype").and_then(|o| o.as_name()) {
            Ok(n) if n == b"Image" => ra.push(st),
            Ok(n) if n == b"Form" => {
                if let Some(r) = st.dict.get(b"Resources").ok().and_then(|o| tu_dien(doc, o)) {
                    gom_anh(doc, &r, sau + 1, ra);
                }
            }
            _ => {}
        }
    }
}

fn dien_tich(doc: &Document, st: &Stream) -> u64 {
    let w = so_nguyen(doc, &st.dict, b"Width").unwrap_or(0).max(0) as u64;
    let h = so_nguyen(doc, &st.dict, b"Height").unwrap_or(0).max(0) as u64;
    w * h
}

// ── đọc đối tượng ──────────────────────────────────────────────────────────────────────────────

fn tra<'a>(doc: &'a Document, o: &'a Object) -> &'a Object {
    let mut cur = o;
    for _ in 0..16 {
        match cur {
            Object::Reference(id) => match doc.get_object(*id) {
                Ok(n) => cur = n,
                Err(_) => return cur,
            },
            _ => return cur,
        }
    }
    cur
}

fn tu_dien(doc: &Document, o: &Object) -> Option<Dictionary> {
    match tra(doc, o) {
        Object::Dictionary(d) => Some(d.clone()),
        Object::Stream(s) => Some(s.dict.clone()),
        _ => None,
    }
}

/// Chỉ nhận luồng qua **tham chiếu** — ảnh nhúng thẳng trong `/XObject` không tồn tại trong PDF thật.
fn luong<'a>(doc: &'a Document, o: &Object) -> Option<&'a Stream> {
    let Object::Reference(id) = o else { return None };
    doc.get_object(*id).ok()?.as_stream().ok()
}

/// Số nguyên, chịu được giá trị ghi gián tiếp (máy quét khổ rộng ghi `/Height 135 0 R`).
fn so_nguyen(doc: &Document, d: &Dictionary, khoa: &[u8]) -> Option<i64> {
    tra(doc, d.get(khoa).ok()?).as_i64().ok()
}

fn co_phai(doc: &Document, d: &Dictionary, khoa: &[u8]) -> bool {
    d.get(khoa).ok().map(|o| tra(doc, o)).and_then(|o| o.as_bool().ok()).unwrap_or(false)
}

/// Danh sách bộ lọc theo đúng thứ tự giải nén.
fn danh_sach_loc(doc: &Document, d: &Dictionary) -> Vec<Vec<u8>> {
    let Some(o) = d.get(b"Filter").ok().map(|o| tra(doc, o)) else {
        return Vec::new();
    };
    match o {
        Object::Name(n) => vec![n.clone()],
        Object::Array(a) => a.iter().filter_map(|f| f.as_name().ok().map(<[u8]>::to_vec)).collect(),
        _ => Vec::new(),
    }
}

/// `/DecodeParms` của bộ lọc cuối (mảng thì lấy phần tử cuối không rỗng).
fn tham_so_loc(doc: &Document, d: &Dictionary) -> Dictionary {
    let Some(o) = d.get(b"DecodeParms").or_else(|_| d.get(b"DP")).ok().map(|o| tra(doc, o)) else {
        return Dictionary::new();
    };
    match o {
        Object::Dictionary(x) => x.clone(),
        Object::Array(a) => a
            .iter()
            .rev()
            .find_map(|x| tu_dien(doc, x))
            .unwrap_or_default(),
        _ => Dictionary::new(),
    }
}

/// `/Decode [1 0 …]` — ảnh ghi ngược, trắng thành đen.
fn decode_dao(doc: &Document, d: &Dictionary) -> bool {
    let Some(Object::Array(a)) = d.get(b"Decode").ok().map(|o| tra(doc, o)) else {
        return false;
    };
    let f = |i: usize| -> Option<f32> {
        let o = a.get(i)?;
        o.as_float().ok().or_else(|| o.as_i64().ok().map(|v| v as f32))
    };
    matches!((f(0), f(1)), (Some(a0), Some(a1)) if a0 > a1)
}

// ── giải nén ảnh ───────────────────────────────────────────────────────────────────────────────

fn giai_anh(doc: &Document, st: &Stream) -> Result<AnhXam, String> {
    let rong = so_nguyen(doc, &st.dict, b"Width").unwrap_or(0);
    let cao = so_nguyen(doc, &st.dict, b"Height").unwrap_or(0);
    if rong <= 0 || cao <= 0 {
        return Err("ảnh không khai kích thước".into());
    }
    if rong as u64 * cao as u64 > DIEM_TOI_DA {
        return Err("ảnh quá lớn để đo".into());
    }
    let (rong, cao) = (rong as u32, cao as u32);
    let loc = danh_sach_loc(doc, &st.dict);
    match loc.last().map(Vec::as_slice) {
        Some(b"DCTDecode") if loc.len() == 1 => tu_jpeg(&st.content),
        Some(b"DCTDecode") => Err("ảnh JPEG nén chồng nhiều tầng".into()),
        Some(b"JPXDecode") => Err("ảnh JPEG 2000".into()),
        Some(b"JBIG2Decode") => Err("ảnh JBIG2".into()),
        Some(b"CCITTFaxDecode") => tu_ccitt(doc, st, rong, cao),
        _ => tu_raster(doc, st, rong, cao, &loc),
    }
}

fn tu_jpeg(byte: &[u8]) -> Result<AnhXam, String> {
    let anh = image::load_from_memory_with_format(byte, image::ImageFormat::Jpeg)
        .map_err(|e| format!("ảnh JPEG hỏng: {e}"))?;
    let xam = anh.to_luma8();
    let (rong, cao) = (xam.width(), xam.height());
    if rong == 0 || cao == 0 {
        return Err("ảnh JPEG rỗng".into());
    }
    Ok(AnhXam { diem: xam.into_raw(), rong, cao })
}

/// Bản quét đen trắng (fax). `/Columns` là bề ngang **lúc nén**, có thể lớn hơn `/Width` vì nén
/// theo bội số byte — đo xong cắt về đúng `/Width`.
fn tu_ccitt(doc: &Document, st: &Stream, rong: u32, cao: u32) -> Result<AnhXam, String> {
    use fax::{decoder, Color};

    let p = tham_so_loc(doc, &st.dict);
    let k = so_nguyen(doc, &p, b"K").unwrap_or(0);
    let cot = so_nguyen(doc, &p, b"Columns").unwrap_or(1728).clamp(1, 1 << 20) as u32;
    // `/Columns` không bị `/Width` chặn nên phải tự kiểm trần một lần nữa trước khi cấp bộ nhớ.
    if cot as u64 * cao as u64 > DIEM_TOI_DA {
        return Err("ảnh quá lớn để đo".into());
    }
    // `BlackIs1` chỉ đổi cách GHI bit ra ngoài; bộ giải trả thẳng màu nên chỉ ảnh hưởng khi đi
    // kèm `/Decode [1 0]` — hai cái cùng bật thì triệt tiêu nhau.
    let den_la_1 = co_phai(doc, &p, b"BlackIs1");
    let muc_den: u8 = if den_la_1 == decode_dao(doc, &st.dict) { 0 } else { 255 };
    let muc_nen = 255 - muc_den;

    let mut diem = vec![muc_nen; (cot as usize) * (cao as usize)];
    let mut y = 0usize;
    let mut ghi = |tr: &[u16]| {
        if y < cao as usize {
            let hang = y * cot as usize;
            for (x, c) in decoder::pels(tr, cot as u16).enumerate().take(cot as usize) {
                if c == Color::Black {
                    diem[hang + x] = muc_den;
                }
            }
        }
        y += 1;
    };
    let xong = if k < 0 {
        decoder::decode_g4(st.content.iter().copied(), cot as u16, Some(cao as u16), &mut ghi)
    } else {
        decoder::decode_g3(st.content.iter().copied(), &mut ghi)
    };
    if xong.is_none() && y == 0 {
        return Err(format!("ảnh CCITT (K={k}) không giải nén được"));
    }
    Ok(cat_ngang(diem, cot, cao, rong))
}

/// Cắt về đúng bề ngang khai báo khi `/Columns` rộng hơn.
fn cat_ngang(diem: Vec<u8>, cot: u32, cao: u32, rong: u32) -> AnhXam {
    if rong >= cot {
        return AnhXam { diem, rong: cot, cao };
    }
    let mut ra = Vec::with_capacity((rong as usize) * (cao as usize));
    for y in 0..cao as usize {
        let hang = y * cot as usize;
        ra.extend_from_slice(&diem[hang..hang + rong as usize]);
    }
    AnhXam { diem: ra, rong, cao }
}

/// Ảnh lưu thẳng từng điểm (Flate / LZW / RunLength / không nén).
fn tu_raster(
    doc: &Document,
    st: &Stream,
    rong: u32,
    cao: u32,
    loc: &[Vec<u8>],
) -> Result<AnhXam, String> {
    let data = giai_nen(st, loc)?;
    let mat_na = co_phai(doc, &st.dict, b"ImageMask");
    let bit = if mat_na { 1 } else { so_nguyen(doc, &st.dict, b"BitsPerComponent").unwrap_or(8) };
    if !matches!(bit, 1 | 2 | 4 | 8 | 16) {
        return Err(format!("ảnh {bit} bit mỗi thành phần"));
    }
    let bit = bit as u32;
    let (thanh_phan, bang) = if mat_na { (1, None) } else { khong_gian_mau(doc, &st.dict)? };
    let dao = decode_dao(doc, &st.dict);

    let moi_hang = ((rong as u64 * thanh_phan as u64 * bit as u64 + 7) / 8) as usize;
    if data.len() < moi_hang * cao as usize {
        return Err("ảnh thiếu dữ liệu so với kích thước khai báo".into());
    }
    let toi_da = (1u32 << bit) - 1;
    let mut diem = Vec::with_capacity((rong as usize) * (cao as usize));
    for y in 0..cao as usize {
        let hang = &data[y * moi_hang..(y + 1) * moi_hang];
        for x in 0..rong as usize {
            let mut tp = [0u8; 4];
            for c in 0..thanh_phan.min(4) {
                let v = mau_sample(hang, (x * thanh_phan + c) as u32, bit);
                tp[c] = if bit == 8 || bit == 16 {
                    v as u8
                } else {
                    // 1/2/4 bit: giãn về thang 0..255 (1 bit → 0 hoặc 255).
                    ((v * 255) / toi_da) as u8
                };
            }
            let g = match &bang {
                // Ảnh bảng màu: giá trị đọc được là CHỈ SỐ trong bảng, không phải mức sáng.
                Some(b) => {
                    let i = mau_sample(hang, (x * thanh_phan) as u32, bit) as usize;
                    *b.get(i).unwrap_or(&0)
                }
                None => xam_tu_thanh_phan(&tp[..thanh_phan.min(4)]),
            };
            diem.push(if dao { 255 - g } else { g });
        }
    }
    Ok(AnhXam { diem, rong, cao })
}

/// Lấy mẫu thứ `i` của một hàng, mỗi mẫu `bit` bit. Với 16 bit thì trả byte cao.
fn mau_sample(hang: &[u8], i: u32, bit: u32) -> u32 {
    match bit {
        8 => hang.get(i as usize).copied().unwrap_or(0) as u32,
        16 => hang.get(i as usize * 2).copied().unwrap_or(0) as u32,
        _ => {
            let vi_tri = i as usize * bit as usize;
            let byte = hang.get(vi_tri / 8).copied().unwrap_or(0) as u32;
            let dich = 8 - bit - (vi_tri % 8) as u32;
            (byte >> dich) & ((1 << bit) - 1)
        }
    }
}

fn xam_tu_thanh_phan(tp: &[u8]) -> u8 {
    match tp.len() {
        1 => tp[0],
        3 => luma(tp[0], tp[1], tp[2]),
        4 => {
            // CMYK ghi theo lượng mực: 0 = không mực. Quy về RGB rồi lấy độ sáng.
            let k = tp[3] as u32;
            let f = |c: u8| (255u32.saturating_sub(c as u32) * 255u32.saturating_sub(k) / 255) as u8;
            luma(f(tp[0]), f(tp[1]), f(tp[2]))
        }
        _ => tp.first().copied().unwrap_or(0),
    }
}

fn luma(r: u8, g: u8, b: u8) -> u8 {
    ((r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000).min(255) as u8
}

/// Trả `(số thành phần, bảng màu quy ra mức xám nếu là /Indexed)`.
fn khong_gian_mau(doc: &Document, d: &Dictionary) -> Result<(usize, Option<Vec<u8>>), String> {
    let Some(o) = d.get(b"ColorSpace").or_else(|_| d.get(b"CS")).ok().map(|o| tra(doc, o)) else {
        return Ok((1, None));
    };
    match o {
        Object::Name(n) => Ok((theo_ten(n), None)),
        Object::Array(a) => {
            let Some(dau) = a.first().and_then(|x| x.as_name().ok()) else {
                return Ok((1, None));
            };
            match dau {
                b"ICCBased" => {
                    let n = a
                        .get(1)
                        .and_then(|x| tu_dien(doc, x))
                        .and_then(|s| s.get(b"N").ok().and_then(|o| o.as_i64().ok()))
                        .unwrap_or(1);
                    Ok((n.clamp(1, 4) as usize, None))
                }
                b"Indexed" | b"I" => Ok((1, Some(bang_mau(doc, a)?))),
                b"CalRGB" | b"Lab" => Ok((3, None)),
                b"CalGray" | b"Separation" => Ok((1, None)),
                b"DeviceN" => {
                    let n = a.get(1).map(|x| tra(doc, x)).and_then(|x| x.as_array().ok()).map_or(1, Vec::len);
                    Ok((n.clamp(1, 4), None))
                }
                n => Ok((theo_ten(n), None)),
            }
        }
        _ => Ok((1, None)),
    }
}

fn theo_ten(n: &[u8]) -> usize {
    match n {
        b"DeviceRGB" | b"RGB" | b"CalRGB" => 3,
        b"DeviceCMYK" | b"CMYK" => 4,
        _ => 1,
    }
}

/// `/Indexed base hival lookup` → mức xám của từng chỉ số.
fn bang_mau(doc: &Document, a: &[Object]) -> Result<Vec<u8>, String> {
    let goc = a.get(1).ok_or("bảng màu thiếu không gian gốc")?;
    let n = match tra(doc, goc) {
        Object::Name(x) => theo_ten(x),
        Object::Array(_) => {
            let d = Dictionary::from_iter([("ColorSpace", goc.clone())]);
            khong_gian_mau(doc, &d)?.0
        }
        _ => 1,
    };
    let byte = match a.get(3).map(|o| tra(doc, o)) {
        Some(Object::String(s, _)) => s.clone(),
        Some(Object::Stream(s)) => s.decompressed_content().map_err(|e| format!("bảng màu: {e}"))?,
        _ => return Err("bảng màu không đọc được".into()),
    };
    Ok(byte.chunks(n.max(1)).map(xam_tu_thanh_phan).collect())
}

fn giai_nen(st: &Stream, loc: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    if loc.is_empty() {
        return Ok(st.content.clone());
    }
    if loc.len() == 1 && loc[0] == b"RunLengthDecode" {
        return Ok(run_length(&st.content));
    }
    let biet = |f: &Vec<u8>| {
        matches!(f.as_slice(), b"FlateDecode" | b"Fl" | b"LZWDecode" | b"LZW" | b"ASCII85Decode" | b"A85")
    };
    if loc.iter().all(biet) {
        return st.decompressed_content().map_err(|e| format!("ảnh không giải nén được: {e}"));
    }
    let ten = loc.last().map(|f| String::from_utf8_lossy(f).into_owned()).unwrap_or_default();
    Err(format!("ảnh nén kiểu {ten}"))
}

/// RunLengthDecode (ISO 32000-1 §7.4.5): byte đầu mỗi gói cho biết chép thẳng hay lặp lại.
fn run_length(vao: &[u8]) -> Vec<u8> {
    let mut ra = Vec::with_capacity(vao.len() * 2);
    let mut i = 0usize;
    while i < vao.len() {
        let n = vao[i] as usize;
        i += 1;
        match n {
            128 => break,
            0..=127 => {
                let het = (i + n + 1).min(vao.len());
                ra.extend_from_slice(&vao[i..het]);
                i = het;
            }
            _ => {
                if let Some(b) = vao.get(i) {
                    ra.extend(std::iter::repeat_n(*b, 257 - n));
                }
                i += 1;
            }
        }
    }
    ra
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::dictionary;

    /// Dựng PDF một trang mang đúng một ảnh. `tren_cha` = để `/Resources` ở nút `/Pages` cha
    /// (kiểu thừa kế) thay vì trên trang.
    fn pdf_mot_anh(anh: Stream, tren_cha: bool) -> Vec<u8> {
        let mut doc = Document::with_version("1.7");
        let anh_id = doc.add_object(anh);
        let res = dictionary! { "XObject" => dictionary! { "Im0" => anh_id } };
        let pages_id = doc.new_object_id();
        let mut trang = dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        };
        let mut cha = dictionary! { "Type" => "Pages", "Count" => 1 };
        if tren_cha {
            cha.set("Resources", res);
        } else {
            trang.set("Resources", res);
        }
        let trang_id = doc.add_object(trang);
        cha.set("Kids", vec![trang_id.into()]);
        doc.objects.insert(pages_id, Object::Dictionary(cha));
        let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog);
        let mut ra = Vec::new();
        doc.save_to(&mut ra).unwrap();
        ra
    }

    /// Ảnh xám 8 bit không nén: nửa trên trắng, nửa dưới đen.
    fn anh_xam_doi(rong: u32, cao: u32) -> Stream {
        let mut d = Vec::new();
        for y in 0..cao {
            for _ in 0..rong {
                d.push(if y < cao / 2 { 255 } else { 0 });
            }
        }
        Stream::new(
            dictionary! {
                "Type" => "XObject", "Subtype" => "Image",
                "Width" => rong as i64, "Height" => cao as i64,
                "ColorSpace" => "DeviceGray", "BitsPerComponent" => 8,
            },
            d,
        )
    }

    fn doc_trang(pdf: &[u8]) -> AnhXam {
        page_xam(&ParsedPdf::parse(pdf).unwrap(), 1).unwrap()
    }

    #[test]
    fn anh_xam_khong_nen_doc_dung_tung_diem() {
        let a = doc_trang(&pdf_mot_anh(anh_xam_doi(8, 4), false));
        assert_eq!((a.rong, a.cao), (8, 4));
        assert_eq!(a.diem[0], 255);
        assert_eq!(a.diem[a.diem.len() - 1], 0);
    }

    /// `/Resources` thừa kế từ nút `/Pages` — kiểu pypdf hay để lại. Trước 19/11/2026 cả tệp
    /// như vậy báo "không đo được" dù trang vẫn là ảnh quét.
    #[test]
    fn resources_thua_ke_tu_pages_cha_van_tim_ra_anh() {
        let a = doc_trang(&pdf_mot_anh(anh_xam_doi(8, 4), true));
        assert_eq!((a.rong, a.cao), (8, 4));
    }

    /// Ảnh nằm trong Form XObject lồng một tầng.
    #[test]
    fn anh_trong_form_xobject_long_mot_tang() {
        let mut doc = Document::with_version("1.7");
        let anh_id = doc.add_object(anh_xam_doi(8, 4));
        let form = Stream::new(
            dictionary! {
                "Type" => "XObject", "Subtype" => "Form",
                "BBox" => vec![0.into(), 0.into(), 10.into(), 10.into()],
                "Resources" => dictionary! { "XObject" => dictionary! { "Im0" => anh_id } },
            },
            b"q Q".to_vec(),
        );
        let form_id = doc.add_object(form);
        let pages_id = doc.new_object_id();
        let trang_id = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            "Resources" => dictionary! { "XObject" => dictionary! { "Fm0" => form_id } },
        });
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! { "Type" => "Pages", "Count" => 1, "Kids" => vec![trang_id.into()] }),
        );
        let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog);
        let mut pdf = Vec::new();
        doc.save_to(&mut pdf).unwrap();
        assert_eq!(doc_trang(&pdf).rong, 8);
    }

    /// Trang có logo nhỏ + bản quét to: phải đo bản quét.
    #[test]
    fn trang_nhieu_anh_thi_lay_anh_to_nhat() {
        let mut doc = Document::with_version("1.7");
        let nho = doc.add_object(anh_xam_doi(4, 4));
        let to = doc.add_object(anh_xam_doi(64, 40));
        let pages_id = doc.new_object_id();
        let trang_id = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            "Resources" => dictionary! { "XObject" => dictionary! { "Logo" => nho, "Im0" => to } },
        });
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! { "Type" => "Pages", "Count" => 1, "Kids" => vec![trang_id.into()] }),
        );
        let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog);
        let mut pdf = Vec::new();
        doc.save_to(&mut pdf).unwrap();
        assert_eq!(doc_trang(&pdf).rong, 64, "phải lấy ảnh to, không lấy logo");
    }

    /// Bản quét đen trắng CCITT G4 — dạng phổ biến nhất của tài liệu lưu trữ cũ.
    #[test]
    fn ccitt_g4_doc_dung_cho_den_cho_trang() {
        use fax::{encoder::Encoder, Color, VecWriter};
        let (rong, cao) = (64u32, 16u32);
        let mut enc = Encoder::new(VecWriter::new());
        for y in 0..cao {
            let hang: Vec<Color> = (0..rong)
                .map(|_| if y < cao / 2 { Color::White } else { Color::Black })
                .collect();
            enc.encode_line(hang.into_iter(), rong as u16).unwrap();
        }
        let nen = enc.finish().unwrap().finish();
        let st = Stream::new(
            dictionary! {
                "Type" => "XObject", "Subtype" => "Image",
                "Width" => rong as i64, "Height" => cao as i64,
                "ColorSpace" => "DeviceGray", "BitsPerComponent" => 1,
                "Filter" => "CCITTFaxDecode",
                "DecodeParms" => dictionary! { "K" => -1, "Columns" => rong as i64, "Rows" => cao as i64 },
            },
            nen,
        );
        let a = doc_trang(&pdf_mot_anh(st, false));
        assert_eq!((a.rong, a.cao), (rong, cao));
        assert_eq!(a.diem[0], 255, "nửa trên phải trắng");
        assert_eq!(a.diem[a.diem.len() - 1], 0, "nửa dưới phải đen");
    }

    /// `/Decode [1 0]` đảo trắng đen.
    #[test]
    fn decode_dao_thi_doi_trang_den() {
        let mut st = anh_xam_doi(8, 4);
        st.dict.set("Decode", vec![1.into(), 0.into()]);
        let a = doc_trang(&pdf_mot_anh(st, false));
        assert_eq!(a.diem[0], 0, "điểm trắng bị /Decode đảo thành đen");
    }

    /// Ảnh bảng màu: giá trị đọc được là chỉ số, phải tra bảng mới ra mức sáng.
    #[test]
    fn anh_bang_mau_tra_dung_muc_sang() {
        // Bảng 2 ô: 0 = đỏ sẫm (tối), 1 = trắng. Dữ liệu 4 bit/điểm cho dễ xếp.
        let bang = Object::String(vec![0x40, 0x00, 0x00, 0xff, 0xff, 0xff], lopdf::StringFormat::Hexadecimal);
        let st = Stream::new(
            dictionary! {
                "Type" => "XObject", "Subtype" => "Image",
                "Width" => 2i64, "Height" => 1i64, "BitsPerComponent" => 8,
                "ColorSpace" => Object::Array(vec!["Indexed".into(), "DeviceRGB".into(), 1.into(), bang]),
            },
            vec![0u8, 1u8],
        );
        let a = doc_trang(&pdf_mot_anh(st, false));
        assert_eq!(a.diem.len(), 2);
        assert!(a.diem[0] < 80, "ô 0 phải tối: {}", a.diem[0]);
        assert_eq!(a.diem[1], 255, "ô 1 phải trắng");
    }

    #[test]
    fn run_length_giai_dung() {
        // 2 byte chép thẳng (n=1) rồi 3 byte lặp (n=254) rồi kết thúc (128).
        assert_eq!(run_length(&[1, b'a', b'b', 254, b'c', 128]), b"abccc");
    }

    /// JPEG 2000 và JBIG2 thì chịu — nhưng phải nói rõ vì sao, đừng im lặng.
    #[test]
    fn jpx_va_jbig2_bao_ro_ly_do() {
        for (ten, chu) in [("JPXDecode", "JPEG 2000"), ("JBIG2Decode", "JBIG2")] {
            let st = Stream::new(
                dictionary! {
                    "Type" => "XObject", "Subtype" => "Image",
                    "Width" => 8i64, "Height" => 4i64, "Filter" => ten,
                },
                vec![0u8; 16],
            );
            let e = page_xam(&ParsedPdf::parse(&pdf_mot_anh(st, false)).unwrap(), 1).unwrap_err();
            assert!(e.to_string().contains(chu), "{ten}: {e}");
        }
    }

    #[test]
    fn trang_chu_that_khong_co_anh_thi_noi_ro() {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let trang_id = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            "Resources" => dictionary! { "Font" => dictionary! {} },
        });
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! { "Type" => "Pages", "Count" => 1, "Kids" => vec![trang_id.into()] }),
        );
        let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog);
        let mut pdf = Vec::new();
        doc.save_to(&mut pdf).unwrap();
        let e = page_xam(&ParsedPdf::parse(&pdf).unwrap(), 1).unwrap_err();
        assert!(e.to_string().contains("không có ảnh"), "{e}");
    }
}
