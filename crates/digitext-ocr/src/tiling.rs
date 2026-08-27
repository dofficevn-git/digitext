// Copyright (c) 2026 DigiText. All rights reserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct To7 {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
pub fn f_2l0(r_3fh: u32, r_3fg: u32, tile: u32, overlap: u32) -> Vec<To7> {
    assert!(tile > overlap, "tile phải > overlap");
    assert!(
        r_3fh > 0 && r_3fg > 0,
        "kích thước trang phải > 0"
    );
    let r_3fj = tile - overlap;
    let mut r_3fi = Vec::new();
    let mut y = 0;
    loop {
        let h = tile.min(r_3fg - y);
        let mut x = 0;
        loop {
            let w = tile.min(r_3fh - x);
            r_3fi.push(To7 { x, y, w, h });
            if x + tile >= r_3fh {
                break;
            }
            x += r_3fj;
        }
        if y + tile >= r_3fg {
            break;
        }
        y += r_3fj;
    }
    r_3fi
}
