// Copyright (c) 2026 DigiText. All rights reserved.
use crate::error::WorkerError;
use crate::infer::Tgi;
use crate::worker::Tgj;
use digitext_db::ocr_results;
use digitext_pdf::{f_vz, Tgh, Tny};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
pub const C_FX: &str = "ocr_raw";
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tlw {
    pub t: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
pub fn f_2gl(page: &Tgi) -> serde_json::Value {
    let mut r_2zy: Vec<Tlw> = Vec::new();
    for r_2zx in 0..page.texts.len() {
        let t = digitext_vi::normalize(page.texts[r_2zx].trim());
        if t.is_empty() {
            continue;
        }
        let Some(r_2zw) = page.boxes.get(r_2zx) else {
            continue;
        };
        r_2zy.push(Tlw {
            t,
            x: r_2zw[0],
            y: r_2zw[1],
            w: r_2zw[2] - r_2zw[0],
            h: r_2zw[3] - r_2zw[1],
        });
    }
    serde_json::to_value(&r_2zy).unwrap_or_else(|_| serde_json::Value::Array(Vec::new()))
}
#[derive(Debug, thiserror::Error)]
pub enum AssembleError {
    #[error("lỗi DB khi ghép PDF: {0}")]
    Db(#[from] sqlx::Error),
    #[error("nguồn ảnh trang lỗi: {0}")]
    Source(WorkerError),
    #[error("dữ liệu ghép PDF hỏng: {0}")]
    Data(String),
}
pub async fn f_va<S: Tgj>(
    r_304: &PgPool,
    r_308: &S,
    r_300: Uuid,
    r_301: u32,
) -> Result<Option<Vec<u8>>, AssembleError> {
    let r_306 = ocr_results::f_1nt(r_304, r_300, C_FX).await?;
    if r_306.is_empty() {
        return Ok(None);
    }
    let mut r_303: Vec<Tgh> = Vec::with_capacity(r_306.len());
    for r_305 in &r_306 {
        let r_309: Vec<Tlw> = match &r_305.boxes {
            Some(r_30b) => serde_json::from_value(r_30b.clone()).map_err(|e| {
                AssembleError::Data(format!("span PDF trang {}: {e}", r_305.page_no))
            })?,
            None => Vec::new(),
        };
        let r_302 = r_308
            .load_page_image(r_300, r_305.page_no)
            .await
            .map_err(AssembleError::Source)?;
        let r_30a: Vec<Tny> = r_309
            .into_iter()
            .map(|r_307| Tny {
                text: r_307.t,
                x: r_307.x,
                y: r_307.y,
                width: r_307.w,
                height: r_307.h,
            })
            .collect();
        r_303.push(Tgh {
            jpeg: r_302.jpeg,
            width_px: r_302.width_px,
            height_px: r_302.height_px,
            dpi: r_302.dpi.unwrap_or(r_301),
            spans: r_30a,
        });
    }
    let r_2zz = f_vz(&r_303).map_err(|e| AssembleError::Data(format!("dựng PDF: {e}")))?;
    Ok(Some(r_2zz))
}
