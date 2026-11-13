// Copyright (c) 2026 DigiText. All rights reserved.
//! Mở PDF có bảng tham chiếu (xref) hỏng (13/11/2026). lopdf chỉ nạp đối tượng theo xref: đối tượng nằm sai vị trí
//! hoặc đọc lỗi thì bị BỎ QUA im lặng (chỉ ghi log), trang của đối tượng đó biến khỏi cây trang — tệp 70 trang có thể bị
//! đếm thành 1 trang mà không báo lỗi gì (Kiểm PDF/A, tệp 0000012.pdf, Ghostscript 10.02.1). Foxit / qpdf / pdf.js khi
//! gặp tệp như vậy dựng lại xref bằng cách quét các dòng `N G obj`; ở đây làm đúng cách đó:
//!
//! 1. Mở bình thường; so số trang đọc được với số trang tệp tự khai (`/Count` ở gốc cây trang).
//! 2. Thiếu trang (hoặc không mở được) → quét toàn tệp tìm `N G obj` (bỏ qua nội dung `stream … endstream`, đối tượng
//!    xuất hiện sau thắng — đúng thứ tự cập nhật nối đuôi), ghi một bảng xref cổ điển mới + trailer vào cuối bản sao rồi
//!    mở lại. Đối tượng nằm trong object stream (`/ObjStm`) được lopdf bung ra khi nạp chính stream đó.
//! 3. Dùng bản dựng lại khi nó đọc được NHIỀU trang hơn. Còn thiếu thì người gọi báo "đọc được X/Y trang".
use lopdf::{Document, Object};

/// Kết quả mở PDF.
pub struct MoPdf {
    pub doc: Document,
    /// Số trang tệp tự khai (`/Count` ở gốc cây trang), nếu đọc được.
    pub trang_khai: Option<usize>,
    /// Đã phải dựng lại bảng tham chiếu mới đọc được.
    pub dung_lai_xref: bool,
}

/// `/Count` ở gốc cây trang (Catalog → /Pages).
pub fn trang_khai(doc: &Document) -> Option<usize> {
    let pages = doc.catalog().ok()?.get(b"Pages").ok()?;
    let id = pages.as_reference().ok()?;
    let d = doc.get_dictionary(id).ok()?;
    let n = d.get(b"Count").ok().and_then(|o| match o {
        Object::Reference(r) => doc.get_object(*r).ok().and_then(|o| o.as_i64().ok()),
        o => o.as_i64().ok(),
    })?;
    usize::try_from(n).ok()
}

fn tim(hay: &[u8], needle: &[u8], tu: usize) -> Option<usize> {
    if tu >= hay.len() {
        return None;
    }
    hay[tu..].windows(needle.len()).position(|w| w == needle).map(|p| p + tu)
}

fn la_khoang_trang(c: u8) -> bool {
    matches!(c, b' ' | b'\n' | b'\r' | b'\t' | b'\x0c' | b'\0')
}

fn la_phan_cach(c: u8) -> bool {
    la_khoang_trang(c) || matches!(c, b'<' | b'>' | b'[' | b']' | b'(' | b')' | b'/' | b'%' | b'{' | b'}')
}

/// Đọc số nguyên không dấu từ `i`; trả (giá trị, vị trí sau số).
fn doc_so(b: &[u8], mut i: usize) -> Option<(u64, usize)> {
    let dau = i;
    let mut v: u64 = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        v = v.checked_mul(10)?.checked_add(u64::from(b[i] - b'0'))?;
        i += 1;
    }
    (i > dau && i - dau <= 10).then_some((v, i))
}

fn bo_khoang_trang(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && la_khoang_trang(b[i]) {
        i += 1;
    }
    i
}

/// Quét `N G obj` → (số, thế hệ, vị trí tính từ `goc`). Đối tượng trùng số: lần xuất hiện sau thắng.
fn quet_doi_tuong(b: &[u8], goc: usize) -> std::collections::BTreeMap<u32, (u16, usize)> {
    let mut out = std::collections::BTreeMap::new();
    let mut i = goc;
    while i < b.len() {
        let c = b[i];
        // Bỏ qua dữ liệu stream: nhị phân nén có thể chứa chuỗi giống "1 0 obj".
        if c == b's' && b[i..].starts_with(b"stream") && (i == 0 || la_khoang_trang(b[i - 1]) || b[i - 1] == b'>') {
            let sau = i + 6;
            if sau < b.len() && (b[sau] == b'\n' || b[sau] == b'\r') {
                match tim(b, b"endstream", sau) {
                    Some(e) => {
                        i = e + 9;
                        continue;
                    }
                    None => break,
                }
            }
        }
        if c.is_ascii_digit() && (i == goc || la_phan_cach(b[i - 1])) {
            if let Some((so, j)) = doc_so(b, i) {
                let j2 = bo_khoang_trang(b, j);
                if j2 > j {
                    if let Some((the_he, k)) = doc_so(b, j2) {
                        let k2 = bo_khoang_trang(b, k);
                        if k2 > k && b[k2..].starts_with(b"obj") && b.get(k2 + 3).is_none_or(|&x| la_phan_cach(x)) {
                            if let (Ok(so), Ok(the_he)) = (u32::try_from(so), u16::try_from(the_he)) {
                                if so > 0 {
                                    out.insert(so, (the_he, i - goc));
                                }
                            }
                            i = k2 + 3;
                            continue;
                        }
                    }
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// `/Root N G R` xuất hiện SAU CÙNG trong tệp (trailer cổ điển hoặc từ điển XRef stream của lần cập nhật cuối).
fn tim_root(b: &[u8]) -> Option<(u32, u16)> {
    let mut i = 0;
    let mut kq = None;
    while let Some(p) = tim(b, b"/Root", i) {
        let j = bo_khoang_trang(b, p + 5);
        if let Some((so, k)) = doc_so(b, j) {
            let k2 = bo_khoang_trang(b, k);
            if let Some((the_he, m)) = doc_so(b, k2) {
                let m2 = bo_khoang_trang(b, m);
                if b.get(m2) == Some(&b'R') {
                    if let (Ok(so), Ok(the_he)) = (u32::try_from(so), u16::try_from(the_he)) {
                        kq = Some((so, the_he));
                    }
                }
            }
        }
        i = p + 5;
    }
    kq
}

/// Bản sao của tệp với một bảng xref cổ điển mới (dựng từ quét `N G obj`) nối ở cuối. `None` = không dựng được
/// (không thấy đối tượng nào, hoặc tệp mã hoá — dựng lại sẽ mất từ điển /Encrypt).
pub fn dung_lai(bytes: &[u8]) -> Option<Vec<u8>> {
    let goc = tim(bytes, b"%PDF-", 0)?;
    if tim(bytes, b"/Encrypt", goc).is_some() {
        return None;
    }
    let dt = quet_doi_tuong(bytes, goc);
    let max = *dt.keys().next_back()?;
    let mut out = bytes.to_vec();
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    let xref_tai = out.len() - goc;
    let mut bang = format!("xref\n0 {}\n0000000000 65535 f \n", max + 1);
    for so in 1..=max {
        match dt.get(&so) {
            Some((g, off)) => bang.push_str(&format!("{off:010} {g:05} n \n")),
            None => bang.push_str("0000000000 00000 f \n"),
        }
    }
    let root = tim_root(bytes).filter(|(so, _)| dt.contains_key(so));
    bang.push_str(&format!("trailer\n<< /Size {}", max + 1));
    if let Some((so, g)) = root {
        bang.push_str(&format!(" /Root {so} {g} R"));
    }
    bang.push_str(&format!(" >>\nstartxref\n{xref_tai}\n%%EOF\n"));
    out.extend_from_slice(bang.as_bytes());
    Some(out)
}

/// Thiếu /Root (trailer hỏng) → tìm đối tượng /Type /Catalog có /Pages.
fn va_root(doc: &mut Document) {
    if doc.catalog().is_ok() {
        return;
    }
    let cat = doc.objects.iter().find_map(|(id, o)| {
        let d = o.as_dict().ok()?;
        (d.has_type(b"Catalog") && d.get(b"Pages").is_ok()).then_some(*id)
    });
    if let Some(id) = cat {
        doc.trailer.set("Root", Object::Reference(id));
    }
}

fn so_trang(doc: &Document) -> usize {
    doc.get_pages().len()
}

/// Mở PDF; thiếu trang so với `/Count` hoặc không mở được thì dựng lại xref và dùng bản đọc được nhiều trang hơn.
pub fn mo(bytes: &[u8]) -> Result<MoPdf, lopdf::Error> {
    let thuong = Document::load_mem(bytes);
    let du = |d: &Document| -> bool {
        let n = so_trang(d);
        n > 0 && trang_khai(d).is_none_or(|k| n >= k)
    };
    if let Ok(d) = &thuong {
        if du(d) {
            let k = trang_khai(d);
            return Ok(MoPdf { doc: thuong.unwrap(), trang_khai: k, dung_lai_xref: false });
        }
    }
    let lai = dung_lai(bytes).and_then(|b| Document::load_mem(&b).ok()).map(|mut d| {
        va_root(&mut d);
        d
    });
    match (thuong, lai) {
        (Ok(d), Some(d2)) if so_trang(&d2) > so_trang(&d) => {
            let k = trang_khai(&d2).max(trang_khai(&d));
            Ok(MoPdf { doc: d2, trang_khai: k, dung_lai_xref: true })
        }
        (Ok(d), _) => {
            let k = trang_khai(&d);
            Ok(MoPdf { doc: d, trang_khai: k, dung_lai_xref: false })
        }
        (Err(_), Some(d2)) if so_trang(&d2) > 0 => {
            let k = trang_khai(&d2);
            Ok(MoPdf { doc: d2, trang_khai: k, dung_lai_xref: true })
        }
        (Err(e), _) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Stream};

    /// PDF `n` trang, mỗi trang một dòng chữ.
    fn pdf_mau(n: usize) -> Vec<u8> {
        let mut doc = Document::with_version("1.7");
        // Bảng xref chữ cổ điển (như Ghostscript ghi) để làm hỏng từng dòng; lopdf 0.43 mặc định ghi xref stream.
        doc.reference_table.cross_reference_type = lopdf::xref::XrefType::CrossReferenceTable;
        let pages_id = doc.new_object_id();
        let font = doc.add_object(dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica" });
        let mut kids = Vec::new();
        for i in 0..n {
            let noi_dung = format!("BT /F1 12 Tf 72 720 Td (Trang {}) Tj ET", i + 1);
            let c = doc.add_object(Stream::new(dictionary! {}, noi_dung.into_bytes()));
            let p = doc.add_object(dictionary! {
                "Type" => "Page", "Parent" => pages_id, "Contents" => c,
                "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
                "Resources" => dictionary! { "Font" => dictionary! { "F1" => font } },
            });
            kids.push(Object::Reference(p));
        }
        doc.objects.insert(pages_id, Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => kids, "Count" => n as i64 }));
        let cat = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", cat);
        let mut out = Vec::new();
        doc.save_to(&mut out).unwrap();
        out
    }

    /// Làm sai vị trí của các đối tượng trang (trừ trang đầu) trong bảng xref — lopdf bỏ qua chúng.
    fn lam_hong_xref(pdf: &[u8]) -> Vec<u8> {
        let doc = Document::load_mem(pdf).unwrap();
        let xr = pdf.windows(5).rposition(|w| w == b"\nxref").unwrap();
        let mut out = pdf.to_vec();
        for id in doc.get_pages().values().skip(1) {
            let Some(lopdf::xref::XrefEntry::Normal { offset, generation }) = doc.reference_table.get(id.0) else { panic!() };
            let muc = format!("{offset:010} {generation:05} n");
            let p = out[xr..].windows(muc.len()).position(|w| w == muc.as_bytes()).unwrap() + xr;
            out[p..p + 10].copy_from_slice(b"0000000009");
        }
        out
    }

    #[test]
    fn tep_lanh_khong_dung_lai() {
        let pdf = pdf_mau(5);
        let m = mo(&pdf).unwrap();
        assert_eq!(m.doc.get_pages().len(), 5);
        assert_eq!(m.trang_khai, Some(5));
        assert!(!m.dung_lai_xref);
    }

    #[test]
    fn xref_hong_doc_thieu_trang_dung_lai_doc_du() {
        let hong = lam_hong_xref(&pdf_mau(70));
        let thuong = Document::load_mem(&hong).unwrap();
        assert!(thuong.get_pages().len() < 70, "tệp làm hỏng phải bị lopdf đếm thiếu");
        let m = mo(&hong).unwrap();
        assert_eq!(m.doc.get_pages().len(), 70);
        assert_eq!(m.trang_khai, Some(70));
        assert!(m.dung_lai_xref);
    }

    /// Đường thật của Kiểm PDF/A / worker: `f_2p9` đếm đủ trang và báo đã dựng lại xref.
    #[test]
    fn f_2p9_dem_du_trang_tep_xref_hong() {
        let hong = lam_hong_xref(&pdf_mau(70));
        let r = crate::f_2p9(&hong, None);
        assert_eq!(r.pages, 70);
        assert_eq!(r.pages_with_text, 70);
        assert_eq!(r.pages_declared, Some(70));
        assert!(r.xref_rebuilt);
        let lanh = crate::f_2p9(&pdf_mau(3), None);
        assert_eq!((lanh.pages, lanh.xref_rebuilt), (3, false));
    }

    #[test]
    fn mat_han_bang_xref_van_mo_duoc() {
        let mut pdf = pdf_mau(3);
        let xr = pdf.windows(5).rposition(|w| w == b"\nxref").unwrap();
        pdf.truncate(xr + 1);
        assert!(Document::load_mem(&pdf).is_err());
        let m = mo(&pdf).unwrap();
        assert_eq!(m.doc.get_pages().len(), 3);
        assert!(m.dung_lai_xref);
    }

    #[test]
    fn quet_bo_qua_noi_dung_stream() {
        let b = b"%PDF-1.7\n1 0 obj\n<< /Length 9 >>\nstream\n7 0 obj\nendstream\nendobj\n2 0 obj\n<< >>\nendobj\n";
        let dt = quet_doi_tuong(b, 0);
        assert_eq!(dt.keys().copied().collect::<Vec<_>>(), vec![1, 2]);
    }
}
