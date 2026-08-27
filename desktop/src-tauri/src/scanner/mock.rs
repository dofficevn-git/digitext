// Copyright (c) 2026 DigiText. All rights reserved.
use super::paper::f_250;
use super::{
    f_199, f_19b, f_2bw, f_vo, BatchState, CancelToken, ColorMode, PaperSource, Tko, Tkr, Tkv, Tkw,
};
use image::{Rgb, RgbImage};
use std::thread;
use std::time::Duration;
const C_3Z: u32 = 24;
const C_T: u32 = 4;
const C_78: Duration = Duration::from_millis(140);
pub struct Tdi;
impl Tdi {
    pub fn new() -> Self {
        Tdi
    }
}
impl Tkv for Tdi {
    fn list_devices(&self) -> Result<Vec<Tkw>, String> {
        Ok(vec![Tkw {
            id: "mock-canon-drg2140".into(),
            name: "Canon DR-G2140 (Demo)".into(),
            model: "DR-G2140".into(),
            connected: true,
        }])
    }
    fn scan_batch(&self, r_2c4: &str, r_2cc: &Tko, r_2cg: &dyn Tkr, r_2c5: &CancelToken) {
        r_2cg.state(r_2c4, BatchState::Starting);
        let r_2c6 = r_2cc
            .device_id
            .clone()
            .unwrap_or_else(|| "Canon DR-G2140 (Demo)".into());
        r_2cg.state(r_2c4, BatchState::Capturing { device: r_2c6 });
        let r_2ca = match (r_2cc.source, r_2cc.max_pages) {
            (PaperSource::Glass, _) => r_2cc.start_index,
            (_, Some(r_2cb)) => r_2cc.start_index + r_2cb - 1,
            (_, None) => C_3Z,
        };
        let mut r_2cf = 0u32;
        for r_2c9 in r_2cc.start_index..=r_2ca {
            if r_2c5.is_cancelled() {
                r_2cg.state(r_2c4, BatchState::Cancelled { pages: r_2cf });
                return;
            }
            thread::sleep(C_78);
            let r_2c8 = f_1xo(r_2c9, r_2cc.dpi);
            let r_2c7 = f_2ih(&r_2c8, r_2cc.color);
            let r_2ch = f_2ii(&r_2c8, r_2cc.color);
            let r_2ce = f_199(&r_2c7);
            let mut r_2cd = f_vo(
                r_2c9,
                r_2c8.width_px,
                r_2c8.height_px,
                r_2c8.dpi,
                r_2ch,
                r_2ce,
            );
            r_2cd.source_path = f_2bw(&image::DynamicImage::ImageRgb8(r_2c7), r_2c4, r_2c9);
            r_2cg.page(r_2c4, r_2cd);
            r_2cf += 1;
        }
        r_2cg.state(r_2c4, BatchState::Completed { pages: r_2ca });
    }
}
fn f_1xo(r_2ci: u32, dpi: u32) -> Tku {
    if r_2ci == C_T {
        return Tku {
            width_px: f_1tb(297.0, 290),
            height_px: f_1tb(420.0, 290),
            dpi: 290,
        };
    }
    Tku {
        width_px: f_1tb(210.0, dpi),
        height_px: f_1tb(297.0, dpi),
        dpi,
    }
}
fn f_1tb(r_2cj: f32, dpi: u32) -> u32 {
    (r_2cj / 25.4 * dpi as f32).round() as u32
}
fn f_2ii(r_2co: &Tku, r_2cm: ColorMode) -> String {
    let r_2ck =
        f_250(r_2co.width_px, r_2co.dpi).max(1.0) / f_250(r_2co.height_px, r_2co.dpi).max(1.0);
    let r_2cp: u32 = 118;
    let r_2cs = ((r_2cp as f32) * r_2ck).round().clamp(40.0, 160.0) as u32;
    let r_2cn = !super::paper::f_17i(r_2co.dpi);
    let r_2cr = match r_2cm {
        ColorMode::Gray => Rgb([236, 236, 236]),
        ColorMode::Rgb => Rgb([250, 250, 248]),
    };
    let mut r_2cq = RgbImage::from_pixel(r_2cs, r_2cp, r_2cr);
    let r_2cl = Rgb(if r_2cn { [196, 132, 28] } else { [70, 80, 100] });
    for r_2cu in 0..r_2cp.min(16) {
        for r_2ct in 0..r_2cs {
            r_2cq.put_pixel(r_2ct, r_2cu, r_2cl);
        }
    }
    f_19b(&r_2cq)
}
fn f_2ih(r_2cy: &Tku, r_2cw: ColorMode) -> RgbImage {
    const C_1N: u32 = 1240;
    let (r_2d0, r_2cz) = (r_2cy.width_px.max(1), r_2cy.height_px.max(1));
    let r_2d6 = (C_1N as f32 / r_2d0.max(r_2cz) as f32).min(1.0);
    let r_2d8 = ((r_2d0 as f32 * r_2d6).round() as u32).max(1);
    let r_2d1 = ((r_2cz as f32 * r_2d6).round() as u32).max(1);
    let r_2cx = !super::paper::f_17i(r_2cy.dpi);
    let r_2d5 = match r_2cw {
        ColorMode::Gray => Rgb([240, 240, 240]),
        ColorMode::Rgb => Rgb([252, 252, 250]),
    };
    let mut r_2d2 = RgbImage::from_pixel(r_2d8, r_2d1, r_2d5);
    let r_2cv = Rgb(if r_2cx { [196, 132, 28] } else { [70, 80, 100] });
    let r_2d7 = (r_2d1 / 12).max(4);
    for y in 0..r_2d7.min(r_2d1) {
        for x in 0..r_2d8 {
            r_2d2.put_pixel(x, y, r_2cv);
        }
    }
    let r_2d3 = Rgb([200, 205, 214]);
    let r_2d4 = r_2d8 / 12;
    let mut y = r_2d7 * 2;
    while y + 3 < r_2d1 {
        for x in r_2d4..r_2d8.saturating_sub(r_2d4) {
            for r_2d9 in y..(y + 3).min(r_2d1) {
                r_2d2.put_pixel(x, r_2d9, r_2d3);
            }
        }
        y += r_2d7;
    }
    r_2d2
}
pub(super) struct Tku {
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: u32,
}
