// Copyright (c) 2026 DigiText. All rights reserved.
use super::types::Tgg;
use crate::config;
use crate::db::Tod;
use crate::error::AppError;
use crate::scanner;
use crate::storage::{self, Tgx};
use std::path::{Path, PathBuf};
pub struct T83 {
    pub folder_path: String,
    pub pdf_path: String,
    pub pdf_size: i64,
    pub page_count: i64,
    pub trang: Vec<Tod>,
}
pub struct T1d {
    pub root: PathBuf,
    pub phong_name: String,
    pub hop_so: String,
    pub ho_so_so: String,
    pub pages: Vec<Tgg>,
    pub quality: u8,
    pub strip: bool,
    pub phong_ma_du_an: String,
    pub phong_phong_so: String,
    pub phong_muc_luc_so: String,
    pub ngay_bat_dau: Option<String>,
    pub nam: Option<String>,
    pub thoi_gian: Option<String>,
    pub pdf_out: Option<PathBuf>,
}
pub struct Td9 {
    pub root: PathBuf,
    pub phong_name: String,
    pub hop_so: String,
    pub ho_so_so: String,
    pub pages: Vec<Tgg>,
    pub quality: u8,
    pub strip: bool,
    pub phong_ma_du_an: String,
    pub phong_phong_so: String,
    pub phong_muc_luc_so: String,
    pub ngay_bat_dau: Option<String>,
    pub nam: Option<String>,
    pub thoi_gian: Option<String>,
}
#[derive(Debug)]
pub struct Tda {
    pub pdf_path: String,
    pub pdf_size: i64,
    pub page_count: i64,
}
pub fn f_vb(mut r_z: T1d) -> Result<T83, AppError> {
    let r_t = storage::f_1jt(
        &r_z.root,
        &[&r_z.phong_name, &r_z.hop_so, &r_z.ho_so_so],
        r_z.strip,
    );
    std::fs::create_dir_all(&r_t).map_err(|e| format!("dOG6oW8gdGjGsCBt4bulYyBs4buXaTog{e}"))?;
    let r_r = r_z
        .root
        .canonicalize()
        .map_err(|e| format!("dGjGsCBt4bulYyBn4buRYyBs4buXaTog{e}"))?;
    let r_q = r_t
        .canonicalize()
        .map_err(|e| format!("dGjGsCBt4bulYyBo4buTIHPGoSBs4buXaTog{e}"))?;
    if !r_q.starts_with(&r_r) {
        return Err(AppError::validation(
            "xJHGsOG7nW5nIGThuqtuIHRoxrAgbeG7pWMga2jDtG5nIGjhu6NwIGzhu4c=",
        ));
    }
    let pdf_path = match r_z.pdf_out.take() {
        Some(r_12) => {
            let r_14 = r_12
                .parent()
                .ok_or_else(|| {
                    AppError::validation(
                        "xJHGsOG7nW5nIGThuqtuIFBERiBnaGkgxJHDqCBraMO0bmcgaOG7o3AgbOG7hw==",
                    )
                })?
                .canonicalize()
                .map_err(|e| format!("dGjGsCBt4bulYyBQREYgZ2hpIMSRw6ggbOG7l2k6IA=={e}"))?;
            if !r_14.starts_with(&r_r) {
                return Err(AppError::validation(
                    "xJHGsOG7nW5nIGThuqtuIFBERiBnaGkgxJHDqCBuZ2/DoGkg4buVIGzGsHUgdHLhu68=",
                ));
            }
            let r_11 = r_12.file_name().ok_or_else(|| {
                AppError::validation("dMOqbiBmaWxlIFBERiBnaGkgxJHDqCBraMO0bmcgaOG7o3AgbOG7hw==")
            })?;
            r_14.join(r_11)
        }
        None => {
            let nam = storage::f_1bd(
                r_z.ngay_bat_dau.as_deref(),
                r_z.nam.as_deref(),
                r_z.thoi_gian.as_deref(),
            )?;
            let stem = storage::f_1qm(
                &r_z.phong_ma_du_an,
                &r_z.phong_phong_so,
                &nam,
                &r_z.ho_so_so,
                &r_z.phong_muc_luc_so,
            );
            let seq = storage::f_1uc(&r_q, &stem);
            r_q.join(format!("{stem}.{seq:07}.pdf"))
        }
    };
    let file_stem = pdf_path
        .file_stem()
        .map(|r_17| r_17.to_string_lossy().into_owned())
        .ok_or_else(|| AppError::validation("dMOqbiBmaWxlIFBERiBraMO0bmcgaOG7o3AgbOG7hw=="))?;
    let mut r_16: Vec<Tgx> = Vec::with_capacity(r_z.pages.len());
    let mut trang: Vec<Tod> = Vec::with_capacity(r_z.pages.len());
    let mut r_y: Vec<PathBuf> = Vec::with_capacity(r_z.pages.len());
    let pages = std::mem::take(&mut r_z.pages);
    for (r_v, r_13) in pages.iter().enumerate() {
        let no = (r_v as u32) + 1;
        let r_18 = scanner::f_295(&r_13.source_path)?;
        let r_w = image::open(&r_18)
            .map_err(|e| format!("beG7nyDhuqNuaCB0cmFuZyA={no}IGzhu5dpOiA={e}"))?;
        let (r_10, r_19, r_u) = storage::f_198(&r_w, r_z.quality)?;
        let r_x = format!("{file_stem}_trang{no:03}.jpg");
        let img_path = r_q.join(&r_x);
        let r_s = if r_13.dpi == 0 {
            config::C_3U
        } else {
            r_13.dpi
        };
        trang.push(Tod {
            page_no: no,
            image_path: img_path.to_string_lossy().into_owned(),
            width_px: r_19,
            height_px: r_u,
            dpi: r_s,
            paper: r_13.paper.clone(),
        });
        r_y.push(img_path);
        r_16.push(Tgx {
            jpeg: r_10,
            width: r_19,
            height: r_u,
            dpi: r_s,
        });
    }
    let r_15 = storage::f_sf(&r_16)?;
    storage::f_2rn(&pdf_path, &r_15)?;
    for (img_path, page) in r_y.iter().zip(&r_16) {
        storage::f_2rn(img_path, &page.jpeg)?;
    }
    f_zw(&r_q, &file_stem, r_16.len() as u32);
    Ok(T83 {
        folder_path: r_q.to_string_lossy().into_owned(),
        pdf_path: pdf_path.to_string_lossy().into_owned(),
        pdf_size: r_15.len() as i64,
        page_count: r_16.len() as i64,
        trang,
    })
}
pub fn f_1sb(r_1h: Td9) -> Result<Tda, AppError> {
    if r_1h.pages.is_empty() {
        return Err(AppError::validation(
            "Q2jGsGEgY2jhu41uIOG6o25oIG7DoG8gxJHhu4MgZ+G7mXAu",
        ));
    }
    let r_1d = storage::f_1jt(
        &r_1h.root,
        &[&r_1h.phong_name, &r_1h.hop_so, &r_1h.ho_so_so],
        r_1h.strip,
    );
    std::fs::create_dir_all(&r_1d).map_err(|e| format!("dOG6oW8gdGjGsCBt4bulYyBs4buXaTog{e}"))?;
    let r_1b = r_1h
        .root
        .canonicalize()
        .map_err(|e| format!("dGjGsCBt4bulYyBn4buRYyBs4buXaTog{e}"))?;
    let r_1a = r_1d
        .canonicalize()
        .map_err(|e| format!("dGjGsCBt4bulYyBo4buTIHPGoSBs4buXaTog{e}"))?;
    if !r_1a.starts_with(&r_1b) {
        return Err(AppError::validation(
            "xJHGsOG7nW5nIGThuqtuIHRoxrAgbeG7pWMga2jDtG5nIGjhu6NwIGzhu4c=",
        ));
    }
    let mut r_1l: Vec<Tgx> = Vec::with_capacity(r_1h.pages.len());
    for (r_1f, r_1j) in r_1h.pages.iter().enumerate() {
        let no = r_1f + 1;
        let r_1m = scanner::f_295(&r_1j.source_path)?;
        let r_1g = image::open(&r_1m)
            .map_err(|e| format!("beG7nyDhuqNuaCB0cmFuZyA={no}IGzhu5dpOiA={e}"))?;
        let (r_1i, r_1n, r_1e) = storage::f_198(&r_1g, r_1h.quality)?;
        let r_1c = if r_1j.dpi == 0 {
            config::C_3U
        } else {
            r_1j.dpi
        };
        r_1l.push(Tgx {
            jpeg: r_1i,
            width: r_1n,
            height: r_1e,
            dpi: r_1c,
        });
    }
    let r_1k = storage::f_sf(&r_1l)?;
    let nam = storage::f_1bd(
        r_1h.ngay_bat_dau.as_deref(),
        r_1h.nam.as_deref(),
        r_1h.thoi_gian.as_deref(),
    )?;
    let stem = storage::f_1qm(
        &r_1h.phong_ma_du_an,
        &r_1h.phong_phong_so,
        &nam,
        &r_1h.ho_so_so,
        &r_1h.phong_muc_luc_so,
    );
    let seq = storage::f_1uc(&r_1a, &stem);
    let pdf_path = r_1a.join(format!("{stem}.{seq:07}.pdf"));
    storage::f_2rn(&pdf_path, &r_1k)?;
    Ok(Tda {
        pdf_path: pdf_path.to_string_lossy().into_owned(),
        pdf_size: r_1k.len() as i64,
        page_count: r_1l.len() as i64,
    })
}
fn f_zw(r_1p: &Path, stem: &str, r_1q: u32) {
    let r_1t = format!("{stem}_trang");
    let Ok(r_1u) = std::fs::read_dir(r_1p) else {
        return;
    };
    for r_1o in r_1u.flatten() {
        let r_1r = r_1o.file_name().to_string_lossy().into_owned();
        if let Some(r_1s) = r_1r
            .strip_prefix(&r_1t)
            .and_then(|r_1v| r_1v.strip_suffix(".jpg"))
        {
            if r_1s.parse::<u32>().map(|no| no > r_1q).unwrap_or(false) {
                let _ = std::fs::remove_file(r_1o.path());
            }
        }
    }
}
