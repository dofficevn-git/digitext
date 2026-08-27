// Copyright (c) 2026 DigiText. All rights reserved.
use digitext_pdf::{f_vz, Tgh};
use std::fs;
fn f_1jw(bytes: &[u8]) -> Option<(u32, u32)> {
    let mut r_46c = 2;
    while r_46c + 9 < bytes.len() {
        if bytes[r_46c] != 0xFF {
            r_46c += 1;
            continue;
        }
        let r_46f = bytes[r_46c + 1];
        let r_46d = matches!(
            r_46f, 0xC0..= 0xC3 | 0xC5..= 0xC7 | 0xC9..= 0xCB | 0xCD..= 0xCF
        );
        let r_46e = ((bytes[r_46c + 2] as usize) << 8) | bytes[r_46c + 3] as usize;
        if r_46d {
            let r_46b = ((bytes[r_46c + 5] as u32) << 8) | bytes[r_46c + 6] as u32;
            let r_46g = ((bytes[r_46c + 7] as u32) << 8) | bytes[r_46c + 8] as u32;
            return Some((r_46g, r_46b));
        }
        r_46c += 2 + r_46e;
    }
    None
}
fn main() {
    let r_46h: Vec<String> = std::env::args().collect();
    if r_46h.len() < 4 {
        eprintln!(
            "ZMO5bmc6IGJ1aWxkX2Zyb21fanBlZ3MgPG91dC5wZGY+IDxkcGk+IDxwMS5qcGc+IFtwMi5qcGcgLi4uXQ=="
        );
        std::process::exit(2);
    }
    let out = &r_46h[1];
    let dpi: u32 = r_46h[2].parse().expect("ZHBpIHPhu5E=");
    let r_46k: Vec<Tgh> = r_46h[3..]
        .iter()
        .map(|p| {
            let r_46j = fs::read(p).unwrap_or_else(|e| panic!("xJHhu41jIA=={p}: {e}"));
            let (r_46l, r_46i) = f_1jw(&r_46j).unwrap_or_else(|| {
                panic!("a2jDtG5nIMSR4buNYyDEkcaw4bujYyBrw61jaCB0aMaw4bubYyA={p}")
            });
            Tgh {
                jpeg: r_46j,
                width_px: r_46l,
                height_px: r_46i,
                dpi,
                spans: vec![],
            }
        })
        .collect();
    let pdf = f_vz(&r_46k).expect("ZOG7sW5nIFBERg==");
    fs::write(out, &pdf).expect("ghi PDF");
    println!("✅ {out}: {} trang, {} bytes", r_46k.len(), pdf.len());
}
