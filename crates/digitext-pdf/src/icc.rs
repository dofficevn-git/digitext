// Copyright (c) 2026 DigiText. All rights reserved.
const C_MQ: usize = 1024;
fn f_2bi(r_482: f64) -> [u8; 4] {
    ((r_482 * 65536.0).round() as i32).to_be_bytes()
}
fn f_2gy(r_483: f64) -> f64 {
    if r_483 <= 0.040_45 {
        r_483 / 12.92
    } else {
        ((r_483 + 0.055) / 1.055).powf(2.4)
    }
}
fn f_2j1(r_485: f64, r_486: f64, r_487: f64) -> Vec<u8> {
    let mut r_484 = Vec::with_capacity(20);
    r_484.extend_from_slice(b"XYZ ");
    r_484.extend_from_slice(&[0; 4]);
    r_484.extend_from_slice(&f_2bi(r_485));
    r_484.extend_from_slice(&f_2bi(r_486));
    r_484.extend_from_slice(&f_2bi(r_487));
    r_484
}
fn f_2iz() -> Vec<u8> {
    let mut r_489 = Vec::with_capacity(12 + C_MQ * 2);
    r_489.extend_from_slice(b"curv");
    r_489.extend_from_slice(&[0; 4]);
    r_489.extend_from_slice(&(C_MQ as u32).to_be_bytes());
    for i in 0..C_MQ {
        let r_48a = i as f64 / (C_MQ - 1) as f64;
        let r_488 = f_2gy(r_48a) * 65535.0;
        r_489.extend_from_slice(&(r_488.round() as u16).to_be_bytes());
    }
    r_489
}
fn f_2ix(r_48b: &str) -> Vec<u8> {
    let r_48c = r_48b.len() + 1;
    let mut r_48d = Vec::with_capacity(90 + r_48c);
    r_48d.extend_from_slice(b"desc");
    r_48d.extend_from_slice(&[0; 4]);
    r_48d.extend_from_slice(&(r_48c as u32).to_be_bytes());
    r_48d.extend_from_slice(r_48b.as_bytes());
    r_48d.push(0);
    r_48d.extend_from_slice(&[0; 4]);
    r_48d.extend_from_slice(&[0; 4]);
    r_48d.extend_from_slice(&[0; 2]);
    r_48d.push(0);
    r_48d.extend_from_slice(&[0; 67]);
    r_48d
}
fn f_2iy(r_48e: &str) -> Vec<u8> {
    let mut r_48f = Vec::with_capacity(8 + r_48e.len() + 1);
    r_48f.extend_from_slice(b"text");
    r_48f.extend_from_slice(&[0; 4]);
    r_48f.extend_from_slice(r_48e.as_bytes());
    r_48f.push(0);
    r_48f
}
pub const C_KB: &str = "sRGB IEC61966-2.1";
pub fn f_2gx() -> Vec<u8> {
    let r_48p = f_2iz();
    let mut r_48n: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"desc", f_2ix("sRGB IEC61966-2.1 (DigiText)")),
        (b"cprt", f_2iy("Public Domain")),
        (b"wtpt", f_2j1(0.964_20, 1.0, 0.824_90)),
        (b"rXYZ", f_2j1(0.436_066, 0.222_488, 0.013_916)),
        (b"gXYZ", f_2j1(0.385_147, 0.716_873, 0.097_076)),
        (b"bXYZ", f_2j1(0.143_066, 0.060_608, 0.714_096)),
        (b"rTRC", r_48p.clone()),
        (b"gTRC", r_48p.clone()),
        (b"bTRC", r_48p),
    ];
    r_48n.sort_by(|r_48g, b| r_48g.0.cmp(b.0));
    let r_48i = 128;
    let r_48m = 4 + r_48n.len() * 12;
    let mut r_48h = Vec::new();
    let mut r_48l = Vec::with_capacity(r_48m);
    r_48l.extend_from_slice(&(r_48n.len() as u32).to_be_bytes());
    let mut r_48r: Vec<(&[u8], u32, u32)> = Vec::new();
    for (r_48k, r_48j) in &r_48n {
        let (offset, size) = match r_48r.iter().find(|(p, _, _)| *p == r_48j.as_slice()) {
            Some((_, off, size)) => (*off, *size),
            None => {
                let off = (r_48i + r_48m + r_48h.len()) as u32;
                r_48h.extend_from_slice(r_48j);
                while r_48h.len() % 4 != 0 {
                    r_48h.push(0);
                }
                r_48r.push((r_48j.as_slice(), off, r_48j.len() as u32));
                (off, r_48j.len() as u32)
            }
        };
        r_48l.extend_from_slice(r_48k.as_slice());
        r_48l.extend_from_slice(&offset.to_be_bytes());
        r_48l.extend_from_slice(&size.to_be_bytes());
    }
    let r_48o = r_48i + r_48m + r_48h.len();
    let mut p = Vec::with_capacity(r_48o);
    p.extend_from_slice(&(r_48o as u32).to_be_bytes());
    p.extend_from_slice(&[0; 4]);
    p.extend_from_slice(&[0x02, 0x10, 0x00, 0x00]);
    p.extend_from_slice(b"mntr");
    p.extend_from_slice(b"RGB ");
    p.extend_from_slice(b"XYZ ");
    for r_48q in [2026u16, 1, 1, 0, 0, 0] {
        p.extend_from_slice(&r_48q.to_be_bytes());
    }
    p.extend_from_slice(b"acsp");
    p.extend_from_slice(&[0; 4]);
    p.extend_from_slice(&[0; 4]);
    p.extend_from_slice(&[0; 4]);
    p.extend_from_slice(&[0; 4]);
    p.extend_from_slice(&[0; 8]);
    p.extend_from_slice(&[0; 4]);
    p.extend_from_slice(&f_2bi(0.964_20));
    p.extend_from_slice(&f_2bi(1.0));
    p.extend_from_slice(&f_2bi(0.824_90));
    p.extend_from_slice(&[0; 4]);
    p.extend_from_slice(&[0; 16]);
    p.extend_from_slice(&[0; 28]);
    debug_assert_eq!(p.len(), r_48i);
    p.extend_from_slice(&r_48l);
    p.extend_from_slice(&r_48h);
    p
}
