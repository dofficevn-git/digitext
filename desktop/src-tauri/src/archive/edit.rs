// Copyright (c) 2026 DigiText. All rights reserved.
use super::paths::f_113;
use super::types::{RebuildResult, T6s, T9w, Tfh, Tfj, Tgg, Tif};
use crate::config;
use crate::db::{self, Db};
use crate::error::AppError;
use crate::scanner::{self, Tkt};
use lopdf::{Document, Object, ObjectId};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, State};
fn f_1y9<G>(r_6l: usize, r_6g: &str, r_6i: G) -> Result<Vec<Tkt>, String>
where
    G: Fn(usize) -> Result<(image::DynamicImage, u32), String> + Sync,
{
    if r_6l == 0 {
        return Ok(Vec::new());
    }
    let r_6p: Mutex<Vec<Option<Tkt>>> = Mutex::new((0..r_6l).map(|_| None).collect());
    let r_6h: Mutex<Option<String>> = Mutex::new(None);
    let r_6m = AtomicUsize::new(0);
    let r_6s = std::thread::available_parallelism()
        .map(|r_6r| r_6r.get())
        .unwrap_or(4)
        .min(r_6l);
    std::thread::scope(|r_6o| {
        for _ in 0..r_6s {
            r_6o.spawn(|| loop {
                if r_6h.lock().unwrap().is_some() {
                    break;
                }
                let i = r_6m.fetch_add(1, Ordering::Relaxed);
                if i >= r_6l {
                    break;
                }
                let r_6k = r_6i(i).and_then(|(r_6j, dpi)| {
                    let idx = (i as u32) + 1;
                    let r_6q = scanner::f_2bx(&r_6j, r_6g, idx);
                    if r_6q.is_empty() {
                        Err(format!(
                            "a2jDtG5nIGzGsHUgxJHGsOG7o2Mg4bqjbmggdOG6oW0gY2hvIHRyYW5nIA=={idx}"
                        ))
                    } else {
                        Ok(scanner::f_1xn(&r_6j, idx, dpi, r_6q))
                    }
                });
                match r_6k {
                    Ok(r_6n) => r_6p.lock().unwrap()[i] = Some(r_6n),
                    Err(e) => *r_6h.lock().unwrap() = Some(e),
                }
            });
        }
    });
    if let Some(e) = r_6h.into_inner().unwrap() {
        return Err(e);
    }
    Ok(r_6p.into_inner().unwrap().into_iter().flatten().collect())
}
fn f_1y7<T, R, F>(r_6x: Vec<T>, r_6v: F) -> Result<Vec<R>, String>
where
    T: Send,
    R: Send,
    F: Fn(T) -> Result<R, String> + Sync,
{
    let r_6y = r_6x.len();
    if r_6y == 0 {
        return Ok(Vec::new());
    }
    let r_6t: Vec<Mutex<Option<T>>> = r_6x
        .into_iter()
        .map(|r_73| Mutex::new(Some(r_73)))
        .collect();
    let r_70: Mutex<Vec<Option<R>>> = Mutex::new((0..r_6y).map(|_| None).collect());
    let r_6u: Mutex<Option<String>> = Mutex::new(None);
    let r_6z = AtomicUsize::new(0);
    let r_75 = std::thread::available_parallelism()
        .map(|r_74| r_74.get())
        .unwrap_or(4)
        .min(r_6y);
    std::thread::scope(|r_72| {
        for _ in 0..r_75 {
            r_72.spawn(|| loop {
                if r_6u.lock().unwrap().is_some() {
                    break;
                }
                let i = r_6z.fetch_add(1, Ordering::Relaxed);
                if i >= r_6y {
                    break;
                }
                let Some(r_6w) = r_6t[i].lock().unwrap().take() else {
                    continue;
                };
                match r_6v(r_6w) {
                    Ok(r_71) => r_70.lock().unwrap()[i] = Some(r_71),
                    Err(e) => *r_6u.lock().unwrap() = Some(e),
                }
            });
        }
    });
    if let Some(e) = r_6u.into_inner().unwrap() {
        return Err(e);
    }
    Ok(r_70.into_inner().unwrap().into_iter().flatten().collect())
}
fn f_1ou(trang: &[db::Toe], r_76: &str) -> Result<Vec<Tkt>, String> {
    f_1y9(trang.len(), r_76, |i| {
        let r_79 = &trang[i];
        let r_78 = image::open(&r_79.image_path).map_err(|e| {
            format!(
                "beG7nyDhuqNuaCB0cmFuZyA={}IGzhu5dpICg={}): {e}",
                i + 1,
                r_79.image_path
            )
        })?;
        let dpi = r_79.dpi.filter(|r_77| *r_77 > 0).unwrap_or(config::C_3U);
        Ok((r_78, dpi))
    })
}
fn f_2h0(pdf: &Path, r_7a: &str) -> Result<Vec<Tkt>, String> {
    let r_7b: Vec<Mutex<Option<(image::DynamicImage, u32)>>> = f_1b7(pdf)?
        .into_iter()
        .map(|r_7c| Mutex::new(Some(r_7c)))
        .collect();
    f_1y9(r_7b.len(), r_7a, |i| {
        r_7b[i]
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| "4bqjbmggUERGIMSRw6MgxJHGsOG7o2MgbOG6pXk=".to_string())
    })
}
#[tauri::command]
pub async fn open_ho_so_for_edit(
    auth: State<'_, crate::auth::AuthState>,
    db: State<'_, Db>,
    args: Tfh,
) -> Result<T6s, AppError> {
    auth.require_perm(digitext_core::Permission::EditorEdit)?;
    let r_7g = args.ho_so_id;
    let (full, trang) = {
        let r_7f = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        let full = db::f_1em(&r_7f, r_7g)?
            .ok_or_else(|| AppError::not_found("S2jDtG5nIHTDrG0gdGjhuqV5IGjhu5Mgc8ahLg=="))?;
        let r_7j = args.pdf_path.clone().or_else(|| full.pdf_path.clone());
        let trang = match r_7j.as_deref() {
            Some(p) => db::f_1oj(&r_7f, r_7g, p)?,
            None => db::f_1oi(&r_7f, r_7g)?,
        };
        (full, trang)
    };
    let r_7i = args.pdf_path.clone().or_else(|| full.pdf_path.clone());
    let r_7d = scanner::f_1dq("edit");
    let r_7e = r_7d.clone();
    let r_7h = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<Tkt>, String> {
            if !trang.is_empty() {
                return f_1ou(&trang, &r_7e);
            }
            let Some(pdf) = r_7i else {
                return Err(
                    "SOG7kyBzxqEgY2jGsGEgY8OzIHRyYW5nIOG6o25oIG7DoG8gxJHhu4MgY2jhu4luaCBz4butYSAoY2jGsGEgxJHDs25nIGfDs2kgdsOgIGtow7RuZyBjw7MgUERGKS4="
                        .into(),
                );
            };
            let p = Path::new(&pdf);
            if !p.is_file() {
                return Err(
                    "S2jDtG5nIHTDrG0gdGjhuqV5IGZpbGUgUERGIGPhu6dhIGjhu5Mgc8ahIMSR4buDIG3hu58gY2jhu4luaCBz4butYS4="
                        .into(),
                );
            }
            f_2h0(p, &r_7e)
        })
        .await
        .map_err(|e| format!("bHXhu5NuZyB0w6FpIG7huqFwIGzhu5dpOiA={e}"))??;
    tracing::info!(r_7g, r_7h = r_7h.len(), r_7d = % r_7d, "open_ho_so_for_edit");
    Ok(T6s {
        batch_id: r_7d,
        ho_so: full,
        pages: r_7h,
    })
}
#[tauri::command]
pub async fn open_pdf_for_edit(
    app: AppHandle,
    auth: State<'_, crate::auth::AuthState>,
    db: State<'_, Db>,
    args: Tfj,
) -> Result<T6s, AppError> {
    auth.require_perm(digitext_core::Permission::EditorEdit)?;
    let pdf = f_113(&app, &db, &args.pdf_path)?;
    if !pdf.is_file() {
        return Err(AppError::not_found(
            "RmlsZSBQREYga2jDtG5nIHThu5NuIHThuqFpLg==",
        ));
    }
    let r_7q = pdf.to_string_lossy().into_owned();
    let r_7p = pdf
        .parent()
        .map(|r_7o| r_7o.to_string_lossy().into_owned())
        .unwrap_or_default();
    let r_7r = {
        let guard = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        match db::f_1cx(&guard, &r_7q)? {
            Some(r_7n) => db::f_1em(&guard, r_7n)?,
            None => None,
        }
    };
    let r_7k = scanner::f_1dq("edit");
    let r_7l = r_7k.clone();
    if let Some(r_7m) = r_7r {
        let trang = {
            let guard = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
            db::f_1oj(&guard, r_7m.id, &pdf.to_string_lossy())?
        };
        let pdf_for_thread = pdf.clone();
        let pages = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<Tkt>, String> {
            if trang.is_empty() {
                f_2h0(&pdf_for_thread, &r_7l)
            } else {
                f_1ou(&trang, &r_7l)
            }
        })
        .await
        .map_err(|e| format!("bHXhu5NuZyB0w6FpIG7huqFwIGzhu5dpOiA={e}"))??;
        tracing::info!(
            ho_so_id = r_7m.id,
            pages = pages.len(),
            "b3Blbl9wZGZfZm9yX2VkaXQgKMSRxINuZyBrw70p"
        );
        return Ok(T6s {
            batch_id: r_7k,
            ho_so: r_7m,
            pages,
        });
    }
    let pdf_for_thread = pdf.clone();
    let pages = tauri::async_runtime::spawn_blocking(move || f_2h0(&pdf_for_thread, &r_7l))
        .await
        .map_err(|e| format!("bHXhu5NuZyB0w6FpIG7huqFwIGzhu5dpOiA={e}"))??;
    let r_7t = pdf
        .file_stem()
        .map(|r_7s| r_7s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "PDF".into());
    tracing::info!(pages = pages.len(), pdf = % r_7q, "open_pdf_for_edit (ad-hoc)");
    Ok(T6s {
        batch_id: r_7k,
        ho_so: db::T94 {
            id: 0,
            ma_du_an: String::new(),
            ten_du_an: String::new(),
            ma_khoi: None,
            phong_so: None,
            muc_luc_so: None,
            hop_so: String::new(),
            ho_so_so: r_7t,
            tieu_de: String::new(),
            thbq: None,
            nhom: None,
            nam: None,
            folder_path: Some(r_7p),
            pdf_path: Some(r_7q),
            status: "adhoc".into(),
            thoi_gian: None,
            ngay_bat_dau: None,
            ngay_ket_thuc: None,
        },
        pages,
    })
}
#[tauri::command]
pub async fn rebuild_pdf(
    app: AppHandle,
    auth: State<'_, crate::auth::AuthState>,
    db: State<'_, Db>,
    args: Tif,
) -> Result<RebuildResult, AppError> {
    auth.require_perm(digitext_core::Permission::EditorEdit)?;
    if args.pages.is_empty() {
        return Err(AppError::validation(
            "S2jDtG5nIGPDsyB0cmFuZyBuw6BvIMSR4buDIMSRw7NuZyBnw7NpLg==",
        ));
    }
    let r_7w = f_113(&app, &db, &args.pdf_out)?;
    let r_80 = args.options.jpeg_quality;
    let r_7y: Vec<Tgg> = args.pages;
    let r_81 =
        tauri::async_runtime::spawn_blocking(move || -> Result<(String, i64, i64), String> {
            let mut r_7z = Vec::with_capacity(r_7y.len());
            for (i, r_7x) in r_7y.iter().enumerate() {
                let no = (i as u32) + 1;
                let r_82 = scanner::f_295(&r_7x.source_path)?;
                let r_7v = image::open(&r_82)
                    .map_err(|e| format!("beG7nyDhuqNuaCB0cmFuZyA={no}IGzhu5dpOiA={e}"))?;
                let (jpeg, w, h) = crate::storage::f_198(&r_7v, r_80)?;
                let dpi = if r_7x.dpi == 0 {
                    config::C_3U
                } else {
                    r_7x.dpi
                };
                r_7z.push(crate::storage::Tgx {
                    jpeg,
                    width: w,
                    height: h,
                    dpi,
                });
            }
            let r_7u = crate::storage::f_sf(&r_7z)?;
            crate::storage::f_2rn(&r_7w, &r_7u)?;
            Ok((
                r_7w.to_string_lossy().into_owned(),
                r_7u.len() as i64,
                r_7z.len() as i64,
            ))
        })
        .await
        .map_err(|e| format!("bHXhu5NuZyBk4buxbmcgUERGIGzhu5dpOiA={e}"))??;
    tracing::info!(pdf = % r_81.0, r_7y = r_81.2, "rebuild_pdf");
    Ok(RebuildResult {
        pdf_path: r_81.0,
        pdf_size: r_81.1,
        page_count: r_81.2,
    })
}
static C_9K: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(50_000);
fn f_2gz(r_83: &str, r_84: &image::DynamicImage, dpi: u32) -> Result<Tkt, String> {
    let idx = C_9K.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let r_85 = scanner::f_2bw(r_84, r_83, idx);
    if r_85.is_empty() {
        return Err("a2jDtG5nIGzGsHUgxJHGsOG7o2Mg4bqjbmggdHJhbmcgY2jDqG4=".into());
    }
    Ok(scanner::f_1xn(r_84, idx, dpi, r_85))
}
#[tauri::command]
pub async fn insert_pages(
    auth: State<'_, crate::auth::AuthState>,
    args: T9w,
) -> Result<Vec<Tkt>, AppError> {
    auth.require_perm(digitext_core::Permission::EditorEdit)?;
    let T9w {
        batch_id: r_86,
        paths: r_8b,
    } = args;
    if r_8b.is_empty() {
        return Err(AppError::validation(
            "Q2jGsGEgY2jhu41uIHThu4dwIMSR4buDIGNow6huLg==",
        ));
    }
    let r_8a = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<Tkt>, String> {
        let mut r_88 = Vec::new();
        for path in &r_8b {
            let r_89 = Path::new(path);
            let r_87 = r_89
                .extension()
                .and_then(|r_8c| r_8c.to_str())
                .map(|e| e.eq_ignore_ascii_case("pdf"))
                .unwrap_or(false);
            if r_87 {
                for (img, dpi) in f_1b7(r_89)? {
                    r_88.push(f_2gz(&r_86, &img, dpi)?);
                }
            } else {
                let img = image::open(r_89)
                    .map_err(|e| format!("beG7nyDhuqNuaCAn{path}JyBs4buXaTog{e}"))?;
                r_88.push(f_2gz(&r_86, &img, config::C_3U)?);
            }
        }
        if r_88.is_empty() {
            return Err(
                "S2jDtG5nIHRyw61jaCDEkcaw4bujYyB0cmFuZyBuw6BvIHThu6sgdOG7h3AgxJHDoyBjaOG7jW4u"
                    .into(),
            );
        }
        Ok(r_88)
    })
    .await
    .map_err(|e| format!("bHXhu5NuZyBjaMOobiB0cmFuZyBs4buXaTog{e}"))??;
    Ok(r_8a)
}
#[tauri::command]
pub async fn render_pdf_pages(
    app: AppHandle,
    auth: State<'_, crate::auth::AuthState>,
    db: State<'_, Db>,
    args: super::types::Tiq,
) -> Result<Vec<String>, AppError> {
    auth.require_perm(digitext_core::Permission::DossierRead)?;
    let pdf = f_113(&app, &db, &args.pdf_path)?;
    let (r_8e, r_8d) = (args.from, args.count);
    let r_8f = tauri::async_runtime::spawn_blocking(move || f_214(&pdf, r_8e, r_8d))
        .await
        .map_err(|e| format!("bHXhu5NuZyByZW5kZXIgbOG7l2k6IA=={e}"))??;
    Ok(r_8f)
}
fn f_1y0(
    r_8j: &std::collections::BTreeMap<u32, ObjectId>,
    from: Option<u32>,
    r_8g: Option<u32>,
) -> Vec<(u32, ObjectId)> {
    let from = from.unwrap_or(1).max(1);
    r_8j.range(from..)
        .take(r_8g.unwrap_or(u32::MAX) as usize)
        .map(|(r_8i, r_8h)| (*r_8i, *r_8h))
        .collect()
}
fn f_214(pdf: &Path, r_8l: Option<u32>, r_8k: Option<u32>) -> Result<Vec<String>, String> {
    if !pdf.is_file() {
        return Err("S2jDtG5nIHTDrG0gdGjhuqV5IGZpbGUgUERGIMSR4buDIGhp4buDbiB0aOG7iy4=".into());
    }
    let r_8o = f_1b8(pdf, r_8l, r_8k)?;
    f_1y7(r_8o, |r_8n| {
        let r_8m = f_13k(r_8n)?;
        let r_8p = r_8m.thumbnail(scanner::C_G1, scanner::C_G1).to_rgb8();
        Ok(scanner::f_199(&r_8p))
    })
}
enum RawPageImage {
    Encoded(Vec<u8>),
    Raw8 {
        data: Vec<u8>,
        w: u32,
        h: u32,
        gray: bool,
    },
}
fn f_ul(doc: &Document, page_no: u32, r_8y: ObjectId) -> Result<RawPageImage, String> {
    let r_8v = doc
        .get_page_images(r_8y)
        .map_err(|e| format!("xJHhu41jIOG6o25oIHRyYW5nIA=={page_no}IGzhu5dpOiA={e}"))?;
    let r_8q = r_8v
        .into_iter()
        .max_by_key(|i| (i.width.max(0) as i128) * (i.height.max(0) as i128));
    let Some(r_8u) = r_8q else {
        return Err(
            format!(
                "Trang {page_no}IGPhu6dhIFBERiBraMO0bmcgY2jhu6lhIOG6o25oIOKAlCBjaMawYSBo4buXIHRy4bujIGNow6huIFBERiB2xINuIGLhuqNuL3ZlY3Rvci4="
            ),
        );
    };
    let r_8s = r_8u.filters.clone().unwrap_or_default();
    let r_8t = |r_8w: &str| r_8s.iter().any(|r_8r| r_8r == r_8w);
    if r_8t("DCTDecode") || r_8t("JPXDecode") {
        return Ok(RawPageImage::Encoded(r_8u.content.to_vec()));
    }
    let r_8z = doc
        .get_object(r_8u.id)
        .and_then(|r_8x| r_8x.as_stream())
        .map_err(|e| format!("trang {page_no}OiDEkeG7jWMgc3RyZWFtIOG6o25oIGzhu5dpOiA={e}"))?
        .decompressed_content()
        .map_err(|e| format!("trang {page_no}OiBnaeG6o2kgbsOpbiDhuqNuaCBs4buXaTog{e}"))?;
    if r_8u.bits_per_component.unwrap_or(8) != 8 {
        return Err(
            format!(
                "trang {page_no}OiDhuqNuaCBQREYga2jDtG5nIHBo4bqjaSA4LWJpdCAodmQgxJFlbi10cuG6r25nIDEtYml0KSDigJQgY2jGsGEgaOG7lyB0cuG7oy4="
            ),
        );
    }
    let gray = match r_8u.color_space.as_deref().unwrap_or("DeviceRGB") {
        "DeviceRGB" | "CalRGB" | "RGB" => false,
        "DeviceGray" | "CalGray" | "G" => true,
        other => {
            return Err(
                format!(
                    "trang {page_no}OiBraMO0bmcgZ2lhbiBtw6B1ICc={other}JyBjaMawYSBo4buXIHRy4bujIGtoaSB0csOtY2ggdOG7qyBQREYu"
                ),
            );
        }
    };
    Ok(RawPageImage::Raw8 {
        data: r_8z,
        w: r_8u.width.max(0) as u32,
        h: r_8u.height.max(0) as u32,
        gray,
    })
}
fn f_13k(r_91: RawPageImage) -> Result<image::DynamicImage, String> {
    match r_91 {
        RawPageImage::Encoded(r_90) => image::load_from_memory(&r_90)
            .map_err(|e| format!("Z2nhuqNpIG3DoyDhuqNuaCBuw6luIHRyb25nIFBERiBs4buXaTog{e}")),
        RawPageImage::Raw8 { data, w, h, gray } => {
            if gray {
                image::GrayImage::from_raw(w, h, data)
                    .map(image::DynamicImage::ImageLuma8)
                    .ok_or_else(|| {
                        "ZOG7ryBsaeG7h3Ug4bqjbmggeMOhbSBraMO0bmcga2jhu5twIGvDrWNoIHRoxrDhu5tj"
                            .to_string()
                    })
            } else {
                image::RgbImage::from_raw(w, h, data)
                    .map(image::DynamicImage::ImageRgb8)
                    .ok_or_else(|| {
                        "ZOG7ryBsaeG7h3Ug4bqjbmggUkdCIGtow7RuZyBraOG7m3Aga8OtY2ggdGjGsOG7m2M="
                            .to_string()
                    })
            }
        }
    }
}
fn f_1b8(path: &Path, r_93: Option<u32>, r_92: Option<u32>) -> Result<Vec<RawPageImage>, String> {
    let doc = Document::load(path).map_err(|e| format!("xJHhu41jIFBERiBs4buXaTog{e}"))?;
    let r_96 = doc.get_pages();
    if r_96.is_empty() {
        return Err("UERGIGtow7RuZyBjw7MgdHJhbmcgbsOgby4=".into());
    }
    let lat = f_1y0(&r_96, r_93, r_92);
    let mut r_94 = Vec::with_capacity(lat.len());
    for (page_no, r_95) in lat {
        r_94.push(f_ul(&doc, page_no, r_95)?);
    }
    Ok(r_94)
}
pub fn f_1b7(path: &Path) -> Result<Vec<(image::DynamicImage, u32)>, String> {
    let doc = Document::load(path).map_err(|e| format!("xJHhu41jIFBERiBs4buXaTog{e}"))?;
    let r_9a = doc.get_pages();
    if r_9a.is_empty() {
        return Err("UERGIGtow7RuZyBjw7MgdHJhbmcgbsOgby4=".into());
    }
    let mut r_98 = Vec::with_capacity(r_9a.len());
    for (page_no, r_99) in r_9a {
        let r_9b = f_ul(&doc, page_no, r_99)?;
        let r_97 = f_13k(r_9b).map_err(|e| format!("trang {page_no}: {e}"))?;
        let dpi = f_1s3(&doc, r_99, r_97.width()).unwrap_or(config::C_3U);
        r_98.push((r_97, dpi));
    }
    Ok(r_98)
}
fn f_1s3(doc: &Document, r_9g: ObjectId, r_9h: u32) -> Option<u32> {
    let r_9f = doc.get_dictionary(r_9g).ok()?;
    let r_9d = match r_9f.get(b"MediaBox").ok()? {
        Object::Array(r_9c) => r_9c.clone(),
        Object::Reference(r_9e) => doc.get_object(*r_9e).ok()?.as_array().ok()?.clone(),
        _ => return None,
    };
    if r_9d.len() != 4 {
        return None;
    }
    let r_9i = (r_9d[2].as_float().ok()? - r_9d[0].as_float().ok()?).abs();
    if r_9i < 1.0 {
        return None;
    }
    let dpi = (r_9h as f32 * 72.0 / r_9i).round();
    (50.0..=1200.0).contains(&dpi).then_some(dpi as u32)
}
