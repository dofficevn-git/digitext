// Copyright (c) 2026 DigiText. All rights reserved.
//! GHÉP trang thể thức "TÀI LIỆU LƯU TRỮ CHUYỂN ĐỔI" (Phụ lục VI TT 05/2025/TT-BNV) với tài liệu số khi chuyển đổi ra giấy
//! (Điều 13–16; 27/11/2026, đợt 4 đối chiếu Digifile — nhóm 3B). Hai việc:
//! - [`noi_truoc`]: nối hai PDF (trang thể thức TRƯỚC, tài liệu SAU) thành một tệp mới — đánh số lại đối tượng, một cây Pages
//!   phẳng, kéo thuộc tính kế thừa (Resources / MediaBox / CropBox / Rotate) xuống từng trang trước khi bỏ các nút Pages trung
//!   gian. Bỏ AcroForm / chữ ký / Outlines: bản in ra giấy không giữ chữ ký số (bản gốc trong kho không đổi; kết luận chữ ký
//!   đã được ghi lên trang thể thức lúc chuyển đổi).
//! - [`chan_trang`]: in một dòng nhỏ ở chân mỗi trang ("Bản in từ tài liệu lưu trữ số … trang i/n") bằng font TTF nhúng con —
//!   cùng bộ hàm với watermark bản đọc (`f_2h7`).
use crate::{f_18n, f_1gw, f_23r, f_2bf, f_2eo, f_qn, f_qv, f_zc, PdfError, C_0, C_1};
use lopdf::{Dictionary, Document, Object, ObjectId};
use std::collections::BTreeMap;

const KE_THUA: [&[u8]; 4] = [b"Resources", b"MediaBox", b"CropBox", b"Rotate"];

/// Chép thuộc tính kế thừa từ chuỗi Parent xuống trang (khi trang chưa có).
fn keo_ke_thua(doc: &Document, page: &mut Dictionary) {
    let mut parent = page.get(b"Parent").ok().and_then(|p| p.as_reference().ok());
    let mut n = 0;
    while let Some(pid) = parent {
        n += 1;
        if n > 32 {
            break;
        }
        let Ok(d) = doc.get_dictionary(pid) else { break };
        for k in KE_THUA {
            if !page.has(k) {
                if let Ok(v) = d.get(k) {
                    page.set(k, v.clone());
                }
            }
        }
        parent = d.get(b"Parent").ok().and_then(|p| p.as_reference().ok());
    }
}

fn loi(e: lopdf::Error) -> PdfError {
    PdfError::Encode(e.to_string())
}

/// Nối `truoc` rồi `sau` thành một PDF mới. Lỗi khi một trong hai không đọc được hoặc không có trang.
pub fn noi_truoc(truoc: &[u8], sau: &[u8]) -> Result<Vec<u8>, PdfError> {
    let a = Document::load_mem(truoc).map_err(|e| PdfError::NoPageImage(format!("đọc trang thể thức: {e}")))?;
    let b = Document::load_mem(sau).map_err(|e| PdfError::NoPageImage(format!("đọc tài liệu: {e}")))?;
    let mut max_id = 1u32;
    let mut trang: Vec<(ObjectId, Dictionary)> = Vec::new();
    let mut objects: BTreeMap<ObjectId, Object> = BTreeMap::new();
    for mut doc in [a, b] {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;
        let mut pages: Vec<(u32, ObjectId)> = doc.get_pages().into_iter().collect();
        pages.sort_by_key(|(n, _)| *n);
        for (_, pid) in pages {
            let mut d = doc.get_dictionary(pid).map_err(loi)?.clone();
            keo_ke_thua(&doc, &mut d);
            trang.push((pid, d));
        }
        objects.extend(doc.objects);
    }
    if trang.is_empty() {
        return Err(PdfError::NoPageImage("không có trang nào để ghép".into()));
    }
    let mut out = Document::with_version("1.7");
    let mut catalog: Option<(ObjectId, Dictionary)> = None;
    let mut pages_root: Option<ObjectId> = None;
    for (id, obj) in objects {
        let kieu = obj.type_name().unwrap_or(b"");
        match kieu {
            b"Catalog" => {
                if catalog.is_none() {
                    catalog = obj.as_dict().ok().map(|d| (id, d.clone()));
                }
            }
            b"Pages" => {
                if pages_root.is_none() {
                    pages_root = Some(id);
                }
            }
            // Trang chèn lại bên dưới với Parent mới; outline / form bỏ (không còn nghĩa sau khi ghép).
            b"Page" | b"Outlines" | b"Outline" => {}
            _ => {
                out.objects.insert(id, obj);
            }
        }
    }
    let pages_id = pages_root.unwrap_or_else(|| {
        max_id += 1;
        (max_id, 0)
    });
    let (cat_id, mut cat) = catalog.unwrap_or_else(|| {
        max_id += 1;
        ((max_id, 0), Dictionary::new())
    });
    let mut kids: Vec<Object> = Vec::with_capacity(trang.len());
    for (pid, mut d) in trang {
        d.set("Parent", Object::Reference(pages_id));
        kids.push(Object::Reference(pid));
        out.objects.insert(pid, Object::Dictionary(d));
    }
    let mut pages = Dictionary::new();
    pages.set("Type", Object::Name(b"Pages".to_vec()));
    pages.set("Count", Object::Integer(kids.len() as i64));
    pages.set("Kids", Object::Array(kids));
    out.objects.insert(pages_id, Object::Dictionary(pages));
    cat.set("Type", Object::Name(b"Catalog".to_vec()));
    cat.set("Pages", Object::Reference(pages_id));
    for k in [b"Outlines".as_slice(), b"AcroForm", b"Perms", b"OpenAction", b"PageMode", b"PageLabels", b"Names", b"Dests", b"OCProperties"] {
        cat.remove(k);
    }
    out.objects.insert(cat_id, Object::Dictionary(cat));
    out.trailer.set("Root", Object::Reference(cat_id));
    out.max_id = max_id.max(out.objects.keys().map(|k| k.0).max().unwrap_or(0));
    out.renumber_objects();
    let mut buf = Vec::new();
    out.save_to(&mut buf).map_err(|e| PdfError::Save(e.to_string()))?;
    Ok(buf)
}

/// In chân trang nhỏ (7,5 pt, xám) giữa đáy mỗi trang; `noi_dung(trang, tổng)` sinh chữ cho từng trang.
pub fn chan_trang(pdf: &[u8], noi_dung: &dyn Fn(usize, usize) -> String, font_bytes: &[u8]) -> Result<Vec<u8>, PdfError> {
    let mut doc = Document::load_mem(pdf).map_err(|e| PdfError::NoPageImage(format!("load PDF: {e}")))?;
    let face = ttf_parser::Face::parse(font_bytes, 0).map_err(|e| PdfError::BadFont(e.to_string()))?;
    let mut pids: Vec<(u32, ObjectId)> = doc.get_pages().into_iter().collect();
    pids.sort_by_key(|(n, _)| *n);
    let n = pids.len();
    let texts: Vec<String> = (0..n).map(|i| noi_dung(i + 1, n)).collect();
    let shaped = texts.iter().map(|t| f_2eo(&face, t)).collect::<Result<Vec<_>, _>>()?;
    let mut glyphs: Vec<(char, u16, u16)> = Vec::new();
    for (t, s) in texts.iter().zip(&shaped) {
        for (c, (gid, adv)) in t.chars().zip(s.glyphs.iter()) {
            if !glyphs.iter().any(|(_, g, _)| g == gid) {
                glyphs.push((c, *gid, *adv));
            }
        }
    }
    let font_id = f_qn(&mut doc, font_bytes, &glyphs);
    let size = 7.5_f32;
    for (i, (_, pid)) in pids.iter().enumerate() {
        let [x0, y0, x1, _] = f_18n(&doc, *pid).unwrap_or([0.0, 0.0, C_1, C_0]);
        f_qv(&mut doc, *pid, font_id)?;
        f_23r(&mut doc, *pid, b"q\n".to_vec())?;
        let w = f_2bf(&shaped[i], size);
        let cx = x0 + ((x1 - x0) - w) / 2.0;
        let s = format!("Q\nq\n0.35 g\nBT /DGXWM {size} Tf {cx:.2} {y:.2} Td <{hex}> Tj ET\nQ\n", y = y0 + 14.0, hex = f_1gw(&f_zc(&shaped[i])));
        doc.add_page_contents(*pid, s.into_bytes()).map_err(loi)?;
    }
    let mut buf = Vec::new();
    doc.save_to(&mut buf).map_err(|e| PdfError::Save(e.to_string()))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manual::{render_manual, ManualBlock};

    fn font() -> Vec<u8> {
        crate::sans_font().and_then(|p| std::fs::read(p).ok()).expect("cần font hệ thống")
    }

    /// Ghép 1 trang thể thức + tài liệu 3 trang → 4 trang, trang 1 là thể thức; chân trang đánh số i/n; lopdf đọc lại được.
    #[test]
    fn ghep_va_chan_trang() {
        let f = font();
        let the_thuc = render_manual(&[ManualBlock::Title("TÀI LIỆU LƯU TRỮ CHUYỂN ĐỔI".into()), ManualBlock::Paragraph("Mã lưu trữ tài liệu gốc: 01.01.2020.001.0000003".into())], &f).unwrap();
        let tai_lieu = render_manual(
            &[ManualBlock::Title("Trang 1".into()), ManualBlock::PageBreak, ManualBlock::Title("Trang 2".into()), ManualBlock::PageBreak, ManualBlock::Title("Trang 3".into())],
            &f,
        )
        .unwrap();
        assert_eq!(Document::load_mem(&tai_lieu).unwrap().get_pages().len(), 3);
        let ghep = noi_truoc(&the_thuc, &tai_lieu).unwrap();
        let d = Document::load_mem(&ghep).unwrap();
        assert_eq!(d.get_pages().len(), 4);
        let co_chan = chan_trang(&ghep, &|i, n| format!("Bản in từ tài liệu lưu trữ số — trang {i}/{n}"), &f).unwrap();
        let d2 = Document::load_mem(&co_chan).unwrap();
        assert_eq!(d2.get_pages().len(), 4);
        let chu = d2.extract_text(&[4]).unwrap_or_default();
        assert!(chu.contains("Trang 3"), "trang cuối phải là trang 3 của tài liệu: {chu}");
    }
}
