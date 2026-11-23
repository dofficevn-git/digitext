// Copyright (c) 2026 DigiText. All rights reserved.
//! Dựng PDF **bản quét giả** để thử màn Kiểm PDF/A — mỗi trang một kiểu lỗi.
//!
//!     cargo run -p digitext-pdf --example tao-mau-quet -- <thư mục ra>
//!
//! Ra hai tệp, cùng nội dung nhưng khác cách nén, vì đó chính là chỗ bộ đo hay vấp:
//!
//! * `PDF_mau_quet_CCITT.pdf` — nén CCITT G4, kiểu bản quét đen trắng của tài liệu lưu trữ cũ.
//! * `PDF_mau_quet_xam.pdf` — ảnh xám 8 bit nén Flate, kiểu phần mềm ráp lại (pypdf, ImageMagick).
//!
//! Sáu trang: bình thường · trắng · nghiêng 6° · quét ngang 90° · gần như trắng · bình thường.
//! Tệp chữ thật (không ảnh) thì đã có `PDF_mau_trang_trang_lech.pdf` của pypdf.

use fax::{encoder::Encoder, Color, VecWriter};
use lopdf::{dictionary, Document, Object, Stream};

/// A4 ở 150 dpi.
const RONG: u32 = 1240;
const CAO: u32 = 1754;

fn main() {
    let ra = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    std::fs::create_dir_all(&ra).expect("tạo thư mục");

    let trang: Vec<(&str, Vec<u8>)> = vec![
        ("bình thường", to_trang(18, 0.0, false)),
        ("TRẮNG", vec![255u8; (RONG * CAO) as usize]),
        ("nghiêng 6°", to_trang(18, 6.0, false)),
        ("quét ngang 90°", to_trang(18, 0.0, true)),
        ("gần như trắng", to_trang(1, 0.0, false)),
        ("bình thường", to_trang(18, 0.0, false)),
    ];

    for (ten, g4) in [("PDF_mau_quet_CCITT.pdf", true), ("PDF_mau_quet_xam.pdf", false)] {
        let p = std::path::Path::new(&ra).join(ten);
        std::fs::write(&p, dung_pdf(&trang, g4)).expect("ghi PDF");
        let co = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        println!("{} — {} trang, {} KB", p.display(), trang.len(), co / 1024);
    }
    for (i, (ten, _)) in trang.iter().enumerate() {
        println!("  trang {}: {ten}", i + 1);
    }
}

/// Ảnh xám một trang: nền trắng, vẽ `so_dong` vạch đen giả làm dòng chữ, nghiêng `goc` độ.
/// `dung` = vạch dựng đứng, tức trang bị quét ngang 90°.
fn to_trang(so_dong: u32, goc: f32, dung: bool) -> Vec<u8> {
    let mut v = vec![255u8; (RONG * CAO) as usize];
    let (w, h) = (RONG as f32, CAO as f32);
    let t = goc.to_radians().tan();
    for i in 0..so_dong {
        // Vạch thứ i nằm ở đâu theo chiều vuông góc với nó.
        let giua = if dung { w } else { h } * (i as f32 + 1.0) / (so_dong as f32 + 1.0);
        let (d0, d1) = if dung { (CAO / 8, CAO * 7 / 8) } else { (RONG / 8, RONG * 7 / 8) };
        for d in d0..d1 {
            let lech = t * (d as f32 - if dung { h } else { w } / 2.0);
            for day in 0..6 {
                let (x, y) = if dung {
                    ((giua + lech) as i32 + day, d as i32)
                } else {
                    (d as i32, (giua + lech) as i32 + day)
                };
                if x >= 0 && y >= 0 && (x as u32) < RONG && (y as u32) < CAO {
                    v[(y as u32 * RONG + x as u32) as usize] = 0;
                }
            }
        }
    }
    v
}

fn dung_pdf(trang: &[(&str, Vec<u8>)], g4: bool) -> Vec<u8> {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let mut kids: Vec<Object> = Vec::new();
    for (_, xam) in trang {
        let anh = doc.add_object(if g4 { anh_ccitt(xam) } else { anh_xam_flate(xam) });
        let ct = doc.add_object(Stream::new(
            dictionary! {},
            b"q 595.276 0 0 841.89 0 0 cm /Im0 Do Q".to_vec(),
        ));
        kids.push(
            doc.add_object(dictionary! {
                "Type" => "Page", "Parent" => pages_id, "Contents" => ct,
                "MediaBox" => vec![0.into(), 0.into(), 595.276.into(), 841.89.into()],
                "Resources" => dictionary! { "XObject" => dictionary! { "Im0" => anh } },
            })
            .into(),
        );
    }
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! { "Type" => "Pages", "Count" => kids.len() as i64, "Kids" => kids }),
    );
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog);
    let mut ra = Vec::new();
    doc.save_to(&mut ra).expect("ghi PDF");
    ra
}

fn anh_ccitt(xam: &[u8]) -> Stream {
    let mut enc = Encoder::new(VecWriter::new());
    for y in 0..CAO as usize {
        let hang = &xam[y * RONG as usize..(y + 1) * RONG as usize];
        let pels = hang.iter().map(|p| if *p < 128 { Color::Black } else { Color::White });
        enc.encode_line(pels, RONG as u16).expect("nén G4");
    }
    Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Image",
            "Width" => RONG as i64, "Height" => CAO as i64,
            "ColorSpace" => "DeviceGray", "BitsPerComponent" => 1,
            "Filter" => "CCITTFaxDecode",
            "DecodeParms" => dictionary! { "K" => -1, "Columns" => RONG as i64, "Rows" => CAO as i64 },
        },
        enc.finish().expect("kết thúc G4").finish(),
    )
}

fn anh_xam_flate(xam: &[u8]) -> Stream {
    let mut st = Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Image",
            "Width" => RONG as i64, "Height" => CAO as i64,
            "ColorSpace" => "DeviceGray", "BitsPerComponent" => 8,
        },
        xam.to_vec(),
    );
    st.compress().expect("nén Flate");
    st
}
