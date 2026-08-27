// Copyright (c) 2026 DigiText. All rights reserved.
use image::codecs::jpeg::JpegEncoder;
use image::DynamicImage;
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use unicode_normalization::UnicodeNormalization;
const FORBIDDEN: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
fn f_274(r_2ri: &str) -> String {
    r_2ri
        .nfd()
        .filter(|c| !('\u{0300}'..='\u{036f}').contains(c))
        .map(|c| match c {
            'đ' => 'd',
            'Đ' => 'D',
            r_2rh => r_2rh,
        })
        .collect()
}
pub fn f_1j2(name: &str) -> bool {
    name.starts_with('.') || name.starts_with('_')
}
pub fn sanitize(name: &str, r_2rk: usize, r_2rm: bool) -> String {
    let r_2rj = if r_2rm { f_274(name) } else { name.to_string() };
    let mut r_2rl: String = r_2rj
        .chars()
        .map(|c| {
            if FORBIDDEN.contains(&c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    r_2rl = r_2rl.trim().trim_matches('.').trim().to_string();
    if r_2rl.chars().count() > r_2rk {
        r_2rl = r_2rl
            .chars()
            .take(r_2rk)
            .collect::<String>()
            .trim_end()
            .to_string();
    }
    if r_2rl.is_empty() {
        r_2rl.push('_');
    }
    r_2rl
}
static C_MM: AtomicU64 = AtomicU64::new(0);
pub fn f_2rn(r_2rr: &Path, r_2rn: &[u8]) -> Result<(), String> {
    let r_2rq = r_2rr
        .parent()
        .ok_or_else(|| "đường dẫn không có thư mục cha".to_string())?;
    let fname = r_2rr
        .file_name()
        .and_then(|r_2rs| r_2rs.to_str())
        .ok_or_else(|| "tên file không hợp lệ".to_string())?;
    let uniq = format!(
        "{}.{}",
        std::process::id(),
        C_MM.fetch_add(1, Ordering::Relaxed)
    );
    let tmp = r_2rq.join(format!(".{fname}.{uniq}.tmp"));
    {
        let mut r_2rp =
            fs::File::create(&tmp).map_err(|e| format!("tạo file tạm lỗi: {e}"))?;
        r_2rp.write_all(r_2rn).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            format!("ghi file tạm lỗi: {e}")
        })?;
        r_2rp.sync_all().map_err(|e| {
            let _ = fs::remove_file(&tmp);
            format!("fsync file tạm lỗi: {e}")
        })?;
    }
    fs::rename(&tmp, r_2rr).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("đổi tên file lỗi: {e}")
    })?;
    if let Ok(r_2ro) = fs::File::open(r_2rq) {
        let _ = r_2ro.sync_all();
    }
    Ok(())
}
pub fn f_198(r_2rv: &DynamicImage, r_2rw: u8) -> Result<(Vec<u8>, u32, u32), String> {
    let r_2rx = r_2rv.to_rgb8();
    let (r_2ry, r_2ru) = (r_2rx.width(), r_2rx.height());
    let mut r_2rt = Vec::new();
    JpegEncoder::new_with_quality(&mut Cursor::new(&mut r_2rt), r_2rw.clamp(40, 100))
        .encode_image(&r_2rx)
        .map_err(|e| format!("mã hoá JPEG lỗi: {e}"))?;
    Ok((r_2rt, r_2ry, r_2ru))
}
pub fn f_12p(r_2s5: &DynamicImage, x: f32, r_2s9: f32, r_2s8: f32, r_2s3: f32) -> DynamicImage {
    let (r_2s6, r_2s4) = (r_2s5.width(), r_2s5.height());
    let r_2s7 = r_2s8.is_finite() && r_2s3.is_finite() && r_2s8 > 0.0 && r_2s3 > 0.0;
    if r_2s6 == 0 || r_2s4 == 0 || !r_2s7 {
        return r_2s5.clone();
    }
    let r_2s1 = (x.clamp(0.0, 1.0) * r_2s6 as f32).round() as u32;
    let r_2s2 = (r_2s9.clamp(0.0, 1.0) * r_2s4 as f32).round() as u32;
    let r_2s0 = ((r_2s8.max(0.0) * r_2s6 as f32).round() as u32)
        .min(r_2s6.saturating_sub(r_2s1))
        .max(1);
    let r_2rz = ((r_2s3.max(0.0) * r_2s4 as f32).round() as u32)
        .min(r_2s4.saturating_sub(r_2s2))
        .max(1);
    r_2s5.crop_imm(r_2s1, r_2s2, r_2s0, r_2rz)
}
fn f_2bk(r_2sl: &image::RgbaImage, x: f32, r_2sr: f32) -> image::Rgba<u8> {
    let (r_2sn, r_2si) = (r_2sl.width() as i32, r_2sl.height() as i32);
    let r_2so = x.floor() as i32;
    let r_2ss = r_2sr.floor() as i32;
    let (r_2sp, r_2st) = (r_2so + 1, r_2ss + 1);
    let r_2sg = x - r_2so as f32;
    let r_2sh = r_2sr - r_2ss as f32;
    let r_2sa = |r_2sq: i32, r_2su: i32| -> [f32; 4] {
        if r_2sq < 0 || r_2su < 0 || r_2sq >= r_2sn || r_2su >= r_2si {
            [255.0, 255.0, 255.0, 255.0]
        } else {
            let p = r_2sl.get_pixel(r_2sq as u32, r_2su as u32).0;
            [p[0] as f32, p[1] as f32, p[2] as f32, p[3] as f32]
        }
    };
    let (r_2sc, r_2se) = (r_2sa(r_2so, r_2ss), r_2sa(r_2sp, r_2ss));
    let (r_2sd, r_2sf) = (r_2sa(r_2so, r_2st), r_2sa(r_2sp, r_2st));
    let mut r_2sk = [0u8; 4];
    for r_2sj in 0..4 {
        let r_2sm = r_2sc[r_2sj] * (1.0 - r_2sg) + r_2se[r_2sj] * r_2sg;
        let r_2sb = r_2sd[r_2sj] * (1.0 - r_2sg) + r_2sf[r_2sj] * r_2sg;
        r_2sk[r_2sj] = (r_2sm * (1.0 - r_2sh) + r_2sb * r_2sh)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    image::Rgba(r_2sk)
}
pub fn f_2ad(r_2t6: &DynamicImage, r_2sy: f32) -> DynamicImage {
    if r_2sy.abs() < 0.01 {
        return r_2t6.clone();
    }
    let r_2td = r_2t6.to_rgba8();
    let (r_2te, r_2tb) = (r_2td.width() as f32, r_2td.height() as f32);
    let r_2th = r_2sy.to_radians();
    let (r_2tc, r_2sv) = r_2th.sin_cos();
    let r_2t2 = (r_2te * r_2sv.abs() + r_2tb * r_2tc.abs()).round().max(1.0);
    let r_2sz = (r_2te * r_2tc.abs() + r_2tb * r_2sv.abs()).round().max(1.0);
    let (r_2t3, r_2t0) = (r_2t2 as u32, r_2sz as u32);
    let mut r_2t1 = image::RgbaImage::from_pixel(r_2t3, r_2t0, image::Rgba([255, 255, 255, 255]));
    let (r_2t9, r_2ta) = (r_2te / 2.0, r_2tb / 2.0);
    let (r_2sw, r_2sx) = (r_2t2 / 2.0, r_2sz / 2.0);
    for r_2t5 in 0..r_2t0 {
        for r_2t4 in 0..r_2t3 {
            let r_2t7 = r_2t4 as f32 + 0.5 - r_2sw;
            let r_2t8 = r_2t5 as f32 + 0.5 - r_2sx;
            let r_2tf = r_2sv * r_2t7 + r_2tc * r_2t8 + r_2t9 - 0.5;
            let r_2tg = -r_2tc * r_2t7 + r_2sv * r_2t8 + r_2ta - 0.5;
            r_2t1.put_pixel(r_2t4, r_2t5, f_2bk(&r_2td, r_2tf, r_2tg));
        }
    }
    DynamicImage::ImageRgba8(r_2t1)
}
const C_2B: i32 = 40;
fn f_te(r_2tj: &image::GrayImage) -> i32 {
    let (r_2tl, r_2tk) = (r_2tj.width(), r_2tj.height());
    let mut r_2ti: Vec<u8> = Vec::with_capacity(2 * (r_2tl + r_2tk) as usize);
    for x in 0..r_2tl {
        r_2ti.push(r_2tj.get_pixel(x, 0).0[0]);
        r_2ti.push(r_2tj.get_pixel(x, r_2tk - 1).0[0]);
    }
    for r_2tm in 0..r_2tk {
        r_2ti.push(r_2tj.get_pixel(0, r_2tm).0[0]);
        r_2ti.push(r_2tj.get_pixel(r_2tl - 1, r_2tm).0[0]);
    }
    r_2ti.sort_unstable();
    r_2ti[r_2ti.len() / 2] as i32
}
fn f_11e(r_2ts: &DynamicImage) -> Option<(usize, usize, usize, usize)> {
    let r_2tq = r_2ts.to_luma8();
    let (r_2tx, r_2tr) = (r_2tq.width() as usize, r_2tq.height() as usize);
    if r_2tx < 8 || r_2tr < 8 {
        return None;
    }
    let r_2tn = f_te(&r_2tq);
    let mut r_2tw = vec![0u32; r_2tr];
    let mut r_2tp = vec![0u32; r_2tx];
    for (r_2ty, r_2tv) in r_2tw.iter_mut().enumerate() {
        let r_2tz = r_2ty as u32;
        for (x, r_2to) in r_2tp.iter_mut().enumerate() {
            if (r_2tq.get_pixel(x as u32, r_2tz).0[0] as i32 - r_2tn).abs() > C_2B {
                *r_2tv += 1;
                *r_2to += 1;
            }
        }
    }
    let r_2tu = ((r_2tx as f32) * 0.005).ceil() as u32;
    let r_2tt = ((r_2tr as f32) * 0.005).ceil() as u32;
    Some((
        r_2tp.iter().position(|&c| c >= r_2tt)?,
        r_2tw.iter().position(|&c| c >= r_2tu)?,
        r_2tp.iter().rposition(|&c| c >= r_2tt)?,
        r_2tw.iter().rposition(|&c| c >= r_2tu)?,
    ))
}
pub fn auto_crop(r_2u2: &DynamicImage) -> DynamicImage {
    let (r_2u7, r_2u1) = (r_2u2.width() as usize, r_2u2.height() as usize);
    let Some((r_2u3, r_2u6, r_2u5, r_2u0)) = f_11e(r_2u2) else {
        return r_2u2.clone();
    };
    let r_2u4 = ((r_2u7.min(r_2u1) as f32) * 0.01).round() as usize;
    let r_2u8 = r_2u3.saturating_sub(r_2u4);
    let r_2ua = r_2u6.saturating_sub(r_2u4);
    let r_2u9 = (r_2u5 + r_2u4).min(r_2u7 - 1);
    let r_2ub = (r_2u0 + r_2u4).min(r_2u1 - 1);
    if r_2u8 == 0 && r_2ua == 0 && r_2u9 == r_2u7 - 1 && r_2ub == r_2u1 - 1 {
        return r_2u2.clone();
    }
    r_2u2.crop_imm(
        r_2u8 as u32,
        r_2ua as u32,
        (r_2u9 - r_2u8 + 1) as u32,
        (r_2ub - r_2ua + 1) as u32,
    )
}
#[derive(Clone, Copy)]
pub enum Placement {
    Center,
    TopLeft,
}
pub fn f_231(r_2uh: &DynamicImage, r_2uk: Placement) -> DynamicImage {
    let (r_2up, r_2ug) = (r_2uh.width(), r_2uh.height());
    let Some((r_2ui, r_2uo, r_2un, r_2uc)) = f_11e(r_2uh) else {
        return r_2uh.clone();
    };
    let r_2ue = r_2uh
        .crop_imm(
            r_2ui as u32,
            r_2uo as u32,
            (r_2un - r_2ui + 1) as u32,
            (r_2uc - r_2uo + 1) as u32,
        )
        .to_rgba8();
    let (r_2uf, r_2ud) = (r_2ue.width(), r_2ue.height());
    let mut canvas = image::RgbaImage::from_pixel(r_2up, r_2ug, image::Rgba([255, 255, 255, 255]));
    let (r_2ul, r_2um) = match r_2uk {
        Placement::Center => (
            ((r_2up.saturating_sub(r_2uf)) / 2) as i64,
            ((r_2ug.saturating_sub(r_2ud)) / 2) as i64,
        ),
        Placement::TopLeft => {
            let r_2uj = ((r_2up.min(r_2ug) as f32) * 0.03).round() as i64;
            (r_2uj, r_2uj)
        }
    };
    image::imageops::overlay(&mut canvas, &r_2ue, r_2ul, r_2um);
    DynamicImage::ImageRgba8(canvas)
}
fn f_11g(r_2uu: &DynamicImage) -> Option<DynamicImage> {
    let r_2us = r_2uu.to_luma8();
    let (r_2uw, r_2ut) = (r_2us.width(), r_2us.height());
    if r_2uw < 8 || r_2ut < 8 {
        return None;
    }
    let r_2ur = f_te(&r_2us);
    let mut r_2uv = image::GrayImage::from_pixel(r_2uw, r_2ut, image::Luma([255]));
    let mut r_2uq = false;
    for r_2ux in 0..r_2ut {
        for x in 0..r_2uw {
            if (r_2us.get_pixel(x, r_2ux).0[0] as i32 - r_2ur).abs() > C_2B {
                r_2uv.put_pixel(x, r_2ux, image::Luma([0]));
                r_2uq = true;
            }
        }
    }
    r_2uq.then_some(DynamicImage::ImageLuma8(r_2uv))
}
fn f_2aj(r_2uz: &image::GrayImage) -> f64 {
    let (r_2v3, r_2v0) = (r_2uz.width(), r_2uz.height());
    if r_2v0 == 0 {
        return 0.0;
    }
    let mut r_2uy = vec![0u32; r_2v0 as usize];
    for r_2v4 in 0..r_2v0 {
        let mut c = 0u32;
        for x in 0..r_2v3 {
            if r_2uz.get_pixel(x, r_2v4).0[0] < 128 {
                c += 1;
            }
        }
        r_2uy[r_2v4 as usize] = c;
    }
    let r_2v2 = r_2uy.len() as f64;
    let r_2v1 = r_2uy.iter().map(|&c| c as f64).sum::<f64>() / r_2v2;
    r_2uy
        .iter()
        .map(|&c| (c as f64 - r_2v1).powi(2))
        .sum::<f64>()
        / r_2v2
}
fn f_2ct(r_2v8: &DynamicImage, lo: f32, hi: f32, r_2vb: f32, r_2v7: f32) -> f32 {
    let mut r_2v5 = r_2v7;
    let mut r_2v6 = -1.0f64;
    let mut a = lo;
    while a <= hi + 1e-6 {
        let r_2v9 = if a.abs() < 1e-6 {
            r_2v8.to_luma8()
        } else {
            f_2ad(r_2v8, a).to_luma8()
        };
        let r_2va = f_2aj(&r_2v9);
        if r_2va > r_2v6 {
            r_2v6 = r_2va;
            r_2v5 = a;
        }
        a += r_2vb;
    }
    r_2v5
}
fn f_151(r_2vd: &DynamicImage) -> f32 {
    let Some(r_2ve) = f_11g(&r_2vd.thumbnail(900, 900)) else {
        return 0.0;
    };
    let r_2vc = f_2ct(&r_2ve, -8.0, 8.0, 1.0, 0.0);
    f_2ct(&r_2ve, r_2vc - 1.0, r_2vc + 1.0, 0.2, r_2vc)
}
pub fn auto_deskew(r_2vg: &DynamicImage) -> DynamicImage {
    let r_2vf = f_151(r_2vg);
    if r_2vf.abs() < 0.1 {
        return r_2vg.clone();
    }
    f_2ad(r_2vg, r_2vf)
}
pub fn despeckle(r_2vm: &DynamicImage) -> DynamicImage {
    let r_2vn = r_2vm.to_rgb8();
    let (r_2vp, r_2vk) = (r_2vn.width(), r_2vn.height());
    if r_2vp < 3 || r_2vk < 3 {
        return r_2vm.clone();
    }
    let mut r_2vh = r_2vn.clone();
    for r_2vq in 1..r_2vk - 1 {
        for x in 1..r_2vp - 1 {
            for c in 0..3 {
                let mut r_2vo = [0u8; 9];
                let mut r_2vl = 0;
                for r_2vj in -1i32..=1 {
                    for r_2vi in -1i32..=1 {
                        r_2vo[r_2vl] = r_2vn
                            .get_pixel((x as i32 + r_2vi) as u32, (r_2vq as i32 + r_2vj) as u32)
                            .0[c];
                        r_2vl += 1;
                    }
                }
                r_2vo.sort_unstable();
                r_2vh.get_pixel_mut(x, r_2vq).0[c] = r_2vo[4];
            }
        }
    }
    DynamicImage::ImageRgb8(r_2vh)
}
pub fn denoise(r_2vr: &DynamicImage) -> DynamicImage {
    r_2vr.blur(0.7)
}
pub fn sharpen(r_2vs: &DynamicImage) -> DynamicImage {
    r_2vs.unsharpen(1.0, 2)
}
const C_G7: f32 = 0.012;
const C_G5: f32 = 0.06;
const C_G4: f32 = 0.18;
const C_G6: f32 = 0.6;
pub fn f_277(r_2wf: &DynamicImage) -> DynamicImage {
    let r_2wa = r_2wf.to_luma8();
    let (r_2x1, r_2wb) = (r_2wa.width(), r_2wa.height());
    if r_2x1 < 16 || r_2wb < 16 {
        return r_2wf.clone();
    }
    let r_2vt = f_te(&r_2wa);
    let r_2wv = r_2x1.min(r_2wb) as f32;
    let r_2wk = (r_2wv * C_G7).max(3.0);
    let r_2wj = r_2wv * C_G5;
    let r_2wi = ((r_2wj * r_2wj) as usize).saturating_add(1);
    let r_2wl = (r_2x1 as f32 * C_G4) as u32;
    let r_2wm = (r_2wb as f32 * C_G4) as u32;
    let r_2wh = |r_2x0: u8| (r_2x0 as i32 - r_2vt).abs() > C_2B;
    let mut r_2wu = vec![false; (r_2x1 as usize) * (r_2wb as usize)];
    let mut r_2wd: Vec<Vec<(u32, u32)>> = Vec::new();
    for r_2x2 in 0..r_2wb {
        for x in 0..r_2x1 {
            if !(x < r_2wl || x >= r_2x1 - r_2wl || r_2x2 < r_2wm || r_2x2 >= r_2wb - r_2wm) {
                continue;
            }
            let r_2we = (r_2x2 as usize) * (r_2x1 as usize) + x as usize;
            if r_2wu[r_2we] || !r_2wh(r_2wa.get_pixel(x, r_2x2).0[0]) {
                continue;
            }
            r_2wu[r_2we] = true;
            let mut stack = vec![(x, r_2x2)];
            let mut px: Vec<(u32, u32)> = Vec::new();
            let (mut x0, mut y0, mut x1, mut y1) = (x, r_2x2, x, r_2x2);
            let mut r_2vz = 0usize;
            let mut r_2wz = false;
            while let Some((r_2w0, r_2w1)) = stack.pop() {
                r_2vz += 1;
                if r_2vz > r_2wi {
                    r_2wz = true;
                    px.clear();
                } else if !r_2wz {
                    px.push((r_2w0, r_2w1));
                }
                x0 = x0.min(r_2w0);
                y0 = y0.min(r_2w1);
                x1 = x1.max(r_2w0);
                y1 = y1.max(r_2w1);
                let mut r_2wq = |nx: u32, ny: u32, stack: &mut Vec<(u32, u32)>| {
                    let r_2wo = (ny as usize) * (r_2x1 as usize) + nx as usize;
                    if !r_2wu[r_2wo] && r_2wh(r_2wa.get_pixel(nx, ny).0[0]) {
                        r_2wu[r_2wo] = true;
                        stack.push((nx, ny));
                    }
                };
                if r_2w0 > 0 {
                    r_2wq(r_2w0 - 1, r_2w1, &mut stack);
                }
                if r_2w0 + 1 < r_2x1 {
                    r_2wq(r_2w0 + 1, r_2w1, &mut stack);
                }
                if r_2w1 > 0 {
                    r_2wq(r_2w0, r_2w1 - 1, &mut stack);
                }
                if r_2w1 + 1 < r_2wb {
                    r_2wq(r_2w0, r_2w1 + 1, &mut stack);
                }
            }
            if r_2wz {
                continue;
            }
            let (r_2vv, r_2vu) = ((x1 - x0 + 1) as f32, (y1 - y0 + 1) as f32);
            let r_2w2 = r_2vv.max(r_2vu);
            let r_2ws = r_2vv / r_2vu;
            let r_2w7 = r_2vz as f32 / (r_2vv * r_2vu);
            let (r_2vw, r_2vx) = ((x0 + x1) / 2, (y0 + y1) / 2);
            let r_2vy = r_2vw < r_2wl
                || r_2vw >= r_2x1.saturating_sub(r_2wl)
                || r_2vx < r_2wm
                || r_2vx >= r_2wb.saturating_sub(r_2wm);
            if r_2w2 >= r_2wk
                && r_2w2 <= r_2wj
                && (0.65..=1.55).contains(&r_2ws)
                && r_2w7 >= C_G6
                && r_2vy
            {
                r_2wd.push(px);
            }
        }
    }
    if r_2wd.is_empty() {
        return r_2wf.clone();
    }
    let mut r_2wp = r_2wf.to_rgba8();
    let r_2wg: std::collections::HashSet<(u32, u32)> = r_2wd.iter().flatten().copied().collect();
    for r_2wc in &r_2wd {
        let (mut x0, mut y0, mut x1, mut y1) = (r_2x1, r_2wb, 0u32, 0u32);
        for &(px, py) in r_2wc {
            x0 = x0.min(px);
            y0 = y0.min(py);
            x1 = x1.max(px);
            y1 = y1.max(py);
        }
        let (r_2wx, r_2wy) = (x0.saturating_sub(3), y0.saturating_sub(3));
        let (r_2w5, r_2w6) = ((x1 + 3).min(r_2x1 - 1), (y1 + 3).min(r_2wb - 1));
        let (mut r_2ww, mut r_2wn) = ([0u64; 3], 0u64);
        for py in r_2wy..=r_2w6 {
            for px in r_2wx..=r_2w5 {
                if r_2wg.contains(&(px, py)) {
                    continue;
                }
                let p = r_2wp.get_pixel(px, py).0;
                r_2ww[0] += p[0] as u64;
                r_2ww[1] += p[1] as u64;
                r_2ww[2] += p[2] as u64;
                r_2wn += 1;
            }
        }
        let r_2w8 = match r_2ww.map(|r_2wt| r_2wt.checked_div(r_2wn)) {
            [Some(r_2wr), Some(r_2w9), Some(b)] => [r_2wr as u8, r_2w9 as u8, b as u8, 255],
            _ => [255, 255, 255, 255],
        };
        for &(px, py) in r_2wc {
            for r_2w4 in -1i32..=1 {
                for r_2w3 in -1i32..=1 {
                    let (nx, ny) = (px as i32 + r_2w3, py as i32 + r_2w4);
                    if nx >= 0 && ny >= 0 && (nx as u32) < r_2x1 && (ny as u32) < r_2wb {
                        r_2wp.put_pixel(nx as u32, ny as u32, image::Rgba(r_2w8));
                    }
                }
            }
        }
    }
    DynamicImage::ImageRgba8(r_2wp)
}
pub fn auto_tone(r_2x8: &DynamicImage) -> DynamicImage {
    let r_2x5 = r_2x8.to_luma8();
    let r_2xe = (r_2x5.width() as u64) * (r_2x5.height() as u64);
    if r_2xe == 0 {
        return r_2x8.clone();
    }
    let mut r_2x6 = [0u64; 256];
    for p in r_2x5.pixels() {
        r_2x6[p.0[0] as usize] += 1;
    }
    let r_2x4 = ((r_2xe as f64) * 0.005) as u64;
    let mut r_2x3 = 0u64;
    let mut lo = 0usize;
    for (v, &c) in r_2x6.iter().enumerate() {
        r_2x3 += c;
        if r_2x3 > r_2x4 {
            lo = v;
            break;
        }
    }
    r_2x3 = 0;
    let mut hi = 255usize;
    for (v, &c) in r_2x6.iter().enumerate().rev() {
        r_2x3 += c;
        if r_2x3 > r_2x4 {
            hi = v;
            break;
        }
    }
    if hi <= lo + 8 {
        return r_2x8.clone();
    }
    let (r_2x9, r_2xd) = (lo as f32, (hi - lo) as f32);
    let r_2xa: [u8; 256] = std::array::from_fn(|r_2x7| {
        (((r_2x7 as f32 - r_2x9) / r_2xd) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8
    });
    let mut r_2xc = r_2x8.to_rgba8();
    for r_2xb in r_2xc.pixels_mut() {
        r_2xb[0] = r_2xa[r_2xb[0] as usize];
        r_2xb[1] = r_2xa[r_2xb[1] as usize];
        r_2xb[2] = r_2xa[r_2xb[2] as usize];
    }
    DynamicImage::ImageRgba8(r_2xc)
}
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnhanceFlags {
    #[serde(default)]
    pub deskew: bool,
    #[serde(default)]
    pub punch_holes: bool,
    #[serde(default)]
    pub despeckle: bool,
    #[serde(default)]
    pub denoise: bool,
    #[serde(default)]
    pub sharpen: bool,
}
impl EnhanceFlags {
    pub const ALL: Self = Self {
        deskew: true,
        punch_holes: true,
        despeckle: true,
        denoise: true,
        sharpen: true,
    };
    pub fn any(&self) -> bool {
        self.deskew || self.punch_holes || self.despeckle || self.denoise || self.sharpen
    }
}
impl Default for EnhanceFlags {
    fn default() -> Self {
        Self::ALL
    }
}
pub fn f_t1(r_2xg: &DynamicImage, r_2xf: EnhanceFlags) -> DynamicImage {
    let mut r_2xh = r_2xg.clone();
    if r_2xf.deskew {
        r_2xh = auto_deskew(&r_2xh);
    }
    if r_2xf.punch_holes {
        r_2xh = f_277(&r_2xh);
    }
    if r_2xf.despeckle {
        r_2xh = despeckle(&r_2xh);
    }
    if r_2xf.denoise {
        r_2xh = denoise(&r_2xh);
    }
    if r_2xf.sharpen {
        r_2xh = sharpen(&r_2xh);
    }
    r_2xh
}
pub fn f_28l(r_2xk: &DynamicImage, r_2xi: u32, r_2xm: u32) -> DynamicImage {
    if r_2xi == 0 || r_2xm == 0 || r_2xi == r_2xm {
        return r_2xk.clone();
    }
    let r_2xl = r_2xm as f32 / r_2xi as f32;
    let r_2xn = ((r_2xk.width() as f32 * r_2xl).round() as u32).max(1);
    let r_2xj = ((r_2xk.height() as f32 * r_2xl).round() as u32).max(1);
    if r_2xn == r_2xk.width() && r_2xj == r_2xk.height() {
        return r_2xk.clone();
    }
    r_2xk.resize_exact(r_2xn, r_2xj, image::imageops::FilterType::Lanczos3)
}
pub struct Tgx {
    pub jpeg: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
}
fn f_213(r_2xo: &str) -> Object {
    Object::Name(r_2xo.as_bytes().to_vec())
}
pub fn f_sf(r_2y0: &[Tgx]) -> Result<Vec<u8>, String> {
    if r_2y0.is_empty() {
        return Err("không có trang để ghép PDF".into());
    }
    let mut r_2xu = Document::with_version("1.5");
    let r_2y1 = r_2xu.new_object_id();
    let mut r_2xx: Vec<Object> = Vec::with_capacity(r_2y0.len());
    for r_2y2 in r_2y0 {
        let dpi = if r_2y2.dpi == 0 { 300 } else { r_2y2.dpi };
        let r_2y4 = r_2y2.width as f32 * 72.0 / dpi as f32;
        let r_2y3 = r_2y2.height as f32 * 72.0 / dpi as f32;
        let r_2xv = dictionary! {
            "Type" => f_213("XObject"), "Subtype" => f_213("Image"), "Width" => r_2y2
            .width as i64, "Height" => r_2y2.height as i64, "ColorSpace" =>
            f_213("DeviceRGB"), "BitsPerComponent" => 8_i64, "Filter" =>
            f_213("DCTDecode"),
        };
        let r_2xw =
            r_2xu.add_object(Stream::new(r_2xv, r_2y2.jpeg.clone()).with_compression(false));
        let r_2xy = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new(
                    "cm",
                    vec![
                        Object::Real(r_2y4),
                        Object::Real(0.0),
                        Object::Real(0.0),
                        Object::Real(r_2y3),
                        Object::Real(0.0),
                        Object::Real(0.0),
                    ],
                ),
                Operation::new("Do", vec![f_213("Im0")]),
                Operation::new("Q", vec![]),
            ],
        };
        let r_2xr = r_2xy
            .encode()
            .map_err(|e| format!("dựng nội dung PDF lỗi: {e}"))?;
        let r_2xs = r_2xu.add_object(Stream::new(dictionary! {}, r_2xr));
        let r_2xz = dictionary! {
            "Type" => f_213("Page"), "Parent" => r_2y1, "MediaBox" =>
            Object::Array(vec![Object::Real(0.0), Object::Real(0.0), Object::Real(r_2y4),
            Object::Real(r_2y3),]), "Resources" => dictionary! { "XObject" => dictionary!
            { "Im0" => r_2xw }, }, "Contents" => r_2xs,
        };
        r_2xx.push(Object::Reference(r_2xu.add_object(r_2xz)));
    }
    let r_2xt = r_2xx.len() as i64;
    r_2xu.objects.insert(
        r_2y1,
        Object::Dictionary(dictionary! {
            "Type" => f_213("Pages"), "Kids" => Object::Array(r_2xx), "Count" =>
            r_2xt,
        }),
    );
    let r_2xq = r_2xu.add_object(dictionary! {
        "Type" => f_213("Catalog"), "Pages" => r_2y1,
    });
    r_2xu.trailer.set("Root", r_2xq);
    r_2xu.compress();
    let mut r_2xp = Vec::new();
    r_2xu
        .save_to(&mut r_2xp)
        .map_err(|e| format!("lưu PDF lỗi: {e}"))?;
    Ok(r_2xp)
}
pub fn f_1jt(r_2y9: &Path, r_2y8: &[&str], r_2ya: bool) -> PathBuf {
    let mut p = r_2y9.to_path_buf();
    for (r_2y5, r_2y7) in r_2y8.iter().enumerate() {
        let r_2y6 = if r_2y5 == 0 { 120 } else { 60 };
        p.push(sanitize(r_2y7, r_2y6, r_2ya));
    }
    p
}
pub fn f_1bd(
    r_2yg: Option<&str>,
    r_2yf: Option<&str>,
    r_2yk: Option<&str>,
) -> Result<String, String> {
    let r_2yh = |s: &str| -> Option<String> {
        if let Some(r_2yc) = s.trim().split('/').nth(2) {
            let y = r_2yc.trim();
            if y.len() == 4 && y.chars().all(|c| c.is_ascii_digit()) {
                return Some(y.to_string());
            }
        }
        None
    };
    let r_2yi = |s: &str| -> Option<String> {
        let s = s.trim();
        if s.len() >= 4 && s.as_bytes()[..4].iter().all(|c| c.is_ascii_digit()) {
            let y = &s[..4];
            if y.chars().all(|c| c.is_ascii_digit()) {
                return Some(y.to_string());
            }
        }
        None
    };
    let r_2yj = |s: &str| -> Option<String> {
        let r_2yb = s.as_bytes();
        let r_2ye = r_2yb.len();
        for r_2yd in 0..r_2ye.saturating_sub(3) {
            if r_2yb[r_2yd].is_ascii_digit()
                && r_2yb[r_2yd + 1].is_ascii_digit()
                && r_2yb[r_2yd + 2].is_ascii_digit()
                && r_2yb[r_2yd + 3].is_ascii_digit()
            {
                return Some(s[r_2yd..r_2yd + 4].to_string());
            }
        }
        None
    };
    if let Some(v) = r_2yg.and_then(|s| {
        let s = s.trim();
        r_2yi(s).or_else(|| r_2yh(s))
    }) {
        return Ok(v.to_string());
    }
    if let Some(v) = r_2yf.and_then(r_2yj) {
        return Ok(v);
    }
    if let Some(v) = r_2yk.and_then(r_2yj) {
        return Ok(v);
    }
    Err(
        "Không xác định được năm hình thành hồ sơ (thiếu ngày bắt đầu, năm, hoặc thời gian)."
            .into(),
    )
}
pub fn f_1uc(r_2yl: &Path, prefix: &str) -> u32 {
    let r_2yq = format!("{prefix}.");
    let r_2yn = ".pdf";
    let mut r_2yo: u32 = 0;
    if let Ok(r_2ys) = std::fs::read_dir(r_2yl) {
        for r_2ym in r_2ys.flatten() {
            let name = r_2ym.file_name().to_string_lossy().into_owned();
            if let Some(r_2yt) = name
                .strip_prefix(&r_2yq)
                .and_then(|r_2yr| r_2yr.strip_suffix(r_2yn))
            {
                if let Ok(r_2yp) = r_2yt.parse::<u32>() {
                    if r_2yp > r_2yo {
                        r_2yo = r_2yp;
                    }
                }
            }
        }
    }
    r_2yo + 1
}
pub fn f_1qm(r_2yv: &str, r_2yy: &str, r_2yx: &str, r_2yu: &str, r_2yw: &str) -> String {
    format!(
        "{}.{}.{}.{}.{}",
        sanitize(r_2yv, 60, false),
        sanitize(r_2yy, 20, false),
        sanitize(r_2yx, 4, false),
        sanitize(r_2yu, 60, false),
        sanitize(r_2yw, 20, false),
    )
}
