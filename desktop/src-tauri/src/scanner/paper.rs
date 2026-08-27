// Copyright (c) 2026 DigiText. All rights reserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperSize {
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    Unknown,
}
impl PaperSize {
    pub fn label(self) -> &'static str {
        match self {
            PaperSize::A0 => "A0",
            PaperSize::A1 => "A1",
            PaperSize::A2 => "A2",
            PaperSize::A3 => "A3",
            PaperSize::A4 => "A4",
            PaperSize::A5 => "A5",
            PaperSize::A6 => "A6",
            PaperSize::Unknown => "Unknown",
        }
    }
    pub fn is_known(self) -> bool {
        self != PaperSize::Unknown
    }
}
const C_9M: [(PaperSize, f32, f32); 7] = [
    (PaperSize::A0, 841.0, 1189.0),
    (PaperSize::A1, 594.0, 841.0),
    (PaperSize::A2, 420.0, 594.0),
    (PaperSize::A3, 297.0, 420.0),
    (PaperSize::A4, 210.0, 297.0),
    (PaperSize::A5, 148.0, 210.0),
    (PaperSize::A6, 105.0, 148.0),
];
const C_MN: f32 = 12.0;
pub const C_KH: [u32; 2] = [300, 600];
pub fn f_250(r_2h5: u32, r_2h4: u32) -> f32 {
    if r_2h4 == 0 {
        return 0.0;
    }
    r_2h5 as f32 / r_2h4 as f32 * 25.4
}
pub fn f_150(r_2hg: u32, r_2ha: u32, r_2h9: u32) -> (PaperSize, f32, f32) {
    let w_mm = f_250(r_2hg, r_2h9);
    let h_mm = f_250(r_2ha, r_2h9);
    let r_2hc = w_mm.min(h_mm);
    let r_2hb = w_mm.max(h_mm);
    let mut r_2h6 = PaperSize::Unknown;
    let mut r_2h7 = f32::MAX;
    for (r_2hd, r_2hf, r_2he) in C_9M {
        let r_2h8 = (r_2hc - r_2hf).abs().max((r_2hb - r_2he).abs());
        if r_2h8 < r_2h7 {
            r_2h7 = r_2h8;
            r_2h6 = r_2hd;
        }
    }
    if r_2h7 <= C_MN {
        (r_2h6, w_mm, h_mm)
    } else {
        (PaperSize::Unknown, w_mm, h_mm)
    }
}
pub fn f_17i(r_2hh: u32) -> bool {
    C_KH.contains(&r_2hh)
}
