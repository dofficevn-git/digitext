// Copyright (c) 2026 DigiText. All rights reserved.
pub const C_OY: u16 = 1000;
pub const C_DO: u16 = u16::MAX;
pub const ADVANCE: u16 = 1000;
pub const C_7B: &str = "DigiTextInvisible";
fn checksum(r_46s: &[u8]) -> u32 {
    let mut r_46t = 0u32;
    let mut r_46r = [0u8; 4];
    for r_46q in r_46s.chunks(4) {
        r_46r = [0; 4];
        r_46r[..r_46q.len()].copy_from_slice(r_46q);
        r_46t = r_46t.wrapping_add(u32::from_be_bytes(r_46r));
    }
    let _ = r_46r;
    r_46t
}
fn f_2io() -> Vec<u8> {
    let mut r_46u = Vec::with_capacity(54);
    r_46u.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    r_46u.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    r_46u.extend_from_slice(&0u32.to_be_bytes());
    r_46u.extend_from_slice(&0x5F0F_3CF5u32.to_be_bytes());
    r_46u.extend_from_slice(&0u16.to_be_bytes());
    r_46u.extend_from_slice(&C_OY.to_be_bytes());
    r_46u.extend_from_slice(&0i64.to_be_bytes());
    r_46u.extend_from_slice(&0i64.to_be_bytes());
    for _ in 0..4 {
        r_46u.extend_from_slice(&0i16.to_be_bytes());
    }
    r_46u.extend_from_slice(&0u16.to_be_bytes());
    r_46u.extend_from_slice(&8u16.to_be_bytes());
    r_46u.extend_from_slice(&2i16.to_be_bytes());
    r_46u.extend_from_slice(&0i16.to_be_bytes());
    r_46u.extend_from_slice(&0i16.to_be_bytes());
    r_46u
}
fn f_2ip() -> Vec<u8> {
    let mut r_46v = Vec::with_capacity(36);
    r_46v.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    r_46v.extend_from_slice(&(C_OY as i16).to_be_bytes());
    r_46v.extend_from_slice(&0i16.to_be_bytes());
    r_46v.extend_from_slice(&0i16.to_be_bytes());
    r_46v.extend_from_slice(&ADVANCE.to_be_bytes());
    r_46v.extend_from_slice(&0i16.to_be_bytes());
    r_46v.extend_from_slice(&0i16.to_be_bytes());
    r_46v.extend_from_slice(&0i16.to_be_bytes());
    r_46v.extend_from_slice(&1i16.to_be_bytes());
    r_46v.extend_from_slice(&0i16.to_be_bytes());
    r_46v.extend_from_slice(&0i16.to_be_bytes());
    r_46v.extend_from_slice(&[0; 8]);
    r_46v.extend_from_slice(&0i16.to_be_bytes());
    r_46v.extend_from_slice(&1u16.to_be_bytes());
    r_46v
}
fn f_2is() -> Vec<u8> {
    let mut r_46w = Vec::with_capacity(32);
    r_46w.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    r_46w.extend_from_slice(&C_DO.to_be_bytes());
    r_46w.extend_from_slice(&[0; 26]);
    r_46w
}
fn f_2iq() -> Vec<u8> {
    let mut r_46x = vec![0u8; 4 + (C_DO as usize - 1) * 2];
    r_46x[0..2].copy_from_slice(&ADVANCE.to_be_bytes());
    r_46x
}
fn f_2ir() -> Vec<u8> {
    vec![0u8; (C_DO as usize + 1) * 2]
}
fn f_2in() -> Vec<u8> {
    let mut r_46y = Vec::with_capacity(24);
    r_46y.extend_from_slice(&4u16.to_be_bytes());
    r_46y.extend_from_slice(&24u16.to_be_bytes());
    r_46y.extend_from_slice(&0u16.to_be_bytes());
    r_46y.extend_from_slice(&2u16.to_be_bytes());
    r_46y.extend_from_slice(&2u16.to_be_bytes());
    r_46y.extend_from_slice(&0u16.to_be_bytes());
    r_46y.extend_from_slice(&0u16.to_be_bytes());
    r_46y.extend_from_slice(&0xFFFFu16.to_be_bytes());
    r_46y.extend_from_slice(&0u16.to_be_bytes());
    r_46y.extend_from_slice(&0xFFFFu16.to_be_bytes());
    r_46y.extend_from_slice(&1u16.to_be_bytes());
    r_46y.extend_from_slice(&0u16.to_be_bytes());
    let mut r_46z = Vec::with_capacity(12 + r_46y.len());
    r_46z.extend_from_slice(&0u16.to_be_bytes());
    r_46z.extend_from_slice(&1u16.to_be_bytes());
    r_46z.extend_from_slice(&3u16.to_be_bytes());
    r_46z.extend_from_slice(&1u16.to_be_bytes());
    r_46z.extend_from_slice(&12u32.to_be_bytes());
    r_46z.extend_from_slice(&r_46y);
    r_46z
}
fn f_2it() -> Vec<u8> {
    let r_471: [(u16, &str); 3] = [(1, C_7B), (2, "Regular"), (6, C_7B)];
    let mut r_474 = Vec::new();
    let mut r_473 = Vec::new();
    for (r_472, r_478) in r_471 {
        let r_477: Vec<u8> = r_478
            .encode_utf16()
            .flat_map(|r_476| r_476.to_be_bytes())
            .collect::<Vec<u8>>();
        r_473.extend_from_slice(&3u16.to_be_bytes());
        r_473.extend_from_slice(&1u16.to_be_bytes());
        r_473.extend_from_slice(&0x0409u16.to_be_bytes());
        r_473.extend_from_slice(&r_472.to_be_bytes());
        r_473.extend_from_slice(&(r_477.len() as u16).to_be_bytes());
        r_473.extend_from_slice(&(r_474.len() as u16).to_be_bytes());
        r_474.extend_from_slice(&r_477);
    }
    let r_470 = r_471.len() as u16;
    let mut r_475 = Vec::with_capacity(6 + r_473.len() + r_474.len());
    r_475.extend_from_slice(&0u16.to_be_bytes());
    r_475.extend_from_slice(&r_470.to_be_bytes());
    r_475.extend_from_slice(&(6 + r_470 * 12).to_be_bytes());
    r_475.extend_from_slice(&r_473);
    r_475.extend_from_slice(&r_474);
    r_475
}
fn f_2iu() -> Vec<u8> {
    let mut r_479 = Vec::with_capacity(32);
    r_479.extend_from_slice(&0x0003_0000u32.to_be_bytes());
    r_479.extend_from_slice(&0i32.to_be_bytes());
    r_479.extend_from_slice(&0i16.to_be_bytes());
    r_479.extend_from_slice(&0i16.to_be_bytes());
    r_479.extend_from_slice(&0u32.to_be_bytes());
    r_479.extend_from_slice(&[0; 16]);
    r_479
}
pub fn f_1f8() -> Vec<u8> {
    let r_47j: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"cmap", f_2in()),
        (b"glyf", vec![0; 4]),
        (b"head", f_2io()),
        (b"hhea", f_2ip()),
        (b"hmtx", f_2iq()),
        (b"loca", f_2ir()),
        (b"maxp", f_2is()),
        (b"name", f_2it()),
        (b"post", f_2iu()),
    ];
    debug_assert!(
        r_47j.windows(2).all(|w| w[0].0 < w[1].0),
        "YuG6o25nIHBo4bqjaSBz4bqvcCB0aGVvIHRhZw=="
    );
    let r_47g = r_47j.len() as u16;
    let r_47e = (15 - r_47g.leading_zeros()) as u16;
    let r_47i = (1u16 << r_47e) * 16;
    let mut font = Vec::new();
    font.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    font.extend_from_slice(&r_47g.to_be_bytes());
    font.extend_from_slice(&r_47i.to_be_bytes());
    font.extend_from_slice(&r_47e.to_be_bytes());
    font.extend_from_slice(&(r_47g * 16 - r_47i).to_be_bytes());
    let mut r_47h = 12 + r_47j.len() * 16;
    let mut r_47f = 0usize;
    let mut r_47d = Vec::with_capacity(r_47j.len() * 16);
    let mut r_47b = Vec::new();
    for (tag, r_47c) in &r_47j {
        if tag.as_slice() == b"head" {
            r_47f = r_47h;
        }
        r_47d.extend_from_slice(tag.as_slice());
        r_47d.extend_from_slice(&checksum(r_47c).to_be_bytes());
        r_47d.extend_from_slice(&(r_47h as u32).to_be_bytes());
        r_47d.extend_from_slice(&(r_47c.len() as u32).to_be_bytes());
        r_47b.extend_from_slice(r_47c);
        while r_47b.len() % 4 != 0 {
            r_47b.push(0);
        }
        r_47h = 12 + r_47j.len() * 16 + r_47b.len();
    }
    font.extend_from_slice(&r_47d);
    font.extend_from_slice(&r_47b);
    let r_47a = 0xB1B0_AFBAu32.wrapping_sub(checksum(&font));
    font[r_47f + 8..r_47f + 12].copy_from_slice(&r_47a.to_be_bytes());
    font
}
