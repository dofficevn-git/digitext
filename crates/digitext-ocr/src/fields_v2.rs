// Copyright (c) 2026 DigiText. All rights reserved.
use crate::error::WorkerError;
use crate::vlm_fields::VlmError;
use digitext_core::VanBanField;
use digitext_db::ocr_fields::{self, Tdw};
use digitext_db::review_queue;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;
pub type T81<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<(VanBanField, String)>, VlmError>> + Send + 'a>>;
pub trait T80: Send + Sync {
    fn extract_v2<'a>(&'a self, image_jpeg: &'a [u8]) -> T81<'a>;
}
#[derive(Debug)]
pub enum ExtractV2Error {
    Source(WorkerError),
    Vlm(VlmError),
}
impl ExtractV2Error {
    pub fn is_permanent(&self) -> bool {
        match self {
            ExtractV2Error::Source(e) => e.is_permanent(),
            ExtractV2Error::Vlm(_) => false,
        }
    }
}
impl std::fmt::Display for ExtractV2Error {
    fn fmt(&self, r_37a: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractV2Error::Source(e) => write!(r_37a, "dOG6o2kg4bqjbmggdHJhbmc6IA=={e}"),
            ExtractV2Error::Vlm(e) => write!(r_37a, "{e}"),
        }
    }
}
pub fn f_1sa(r_37g: &[(i32, Vec<(VanBanField, String)>)]) -> Vec<(i32, VanBanField, String)> {
    let mut r_37e = Vec::new();
    for r_37c in VanBanField::ALL {
        'pages: for (r_37f, r_37d) in r_37g {
            for (r_37b, r_37h) in r_37d {
                if *r_37b == r_37c && !r_37h.trim().is_empty() {
                    r_37e.push((*r_37f, r_37c, r_37h.clone()));
                    break 'pages;
                }
            }
        }
    }
    r_37e
}
pub fn f_14m(
    page_count: Option<i32>,
    r_37i: &[(i32, VanBanField, String)],
) -> Vec<(VanBanField, String, f32)> {
    let r_37k = |r_37j: VanBanField| r_37i.iter().any(|(_, e, _)| *e == r_37j);
    let mut r_37m = Vec::new();
    if let Some(r_37l) = page_count {
        if r_37l >= 1 && !r_37k(VanBanField::SoTrang) {
            r_37m.push((VanBanField::SoTrang, r_37l.to_string(), 1.0));
        }
    }
    if !r_37k(VanBanField::NgonNgu) {
        r_37m.push((VanBanField::NgonNgu, "VGnhur9uZyBWaeG7h3Q=".to_owned(), 0.5));
    }
    r_37m
}
pub async fn f_s3(
    r_37u: &PgPool,
    r_37p: Uuid,
    r_37q: Uuid,
    r_37r: &[(i32, VanBanField, String)],
    r_37o: &[(VanBanField, String, f32)],
) -> sqlx::Result<u64> {
    let mut r_37s = 0u64;
    for (r_37t, field, value) in r_37r {
        ocr_fields::upsert(
            r_37u,
            Tdw {
                document_id: r_37p,
                page_no: *r_37t,
                engine_version_id: r_37q,
                text_source: "ocr_raw",
                field_key: field.as_str(),
                value,
                bbox: None,
                confidence: None,
                verified: false,
            },
        )
        .await?;
        r_37s += 1;
    }
    for (field, value, r_37n) in r_37o {
        ocr_fields::upsert(
            r_37u,
            Tdw {
                document_id: r_37p,
                page_no: 1,
                engine_version_id: r_37q,
                text_source: "ocr_raw",
                field_key: field.as_str(),
                value,
                bbox: None,
                confidence: Some(*r_37n),
                verified: false,
            },
        )
        .await?;
        r_37s += 1;
    }
    if r_37s > 0 {
        review_queue::enqueue(r_37u, r_37p, 1, review_queue::C_I6).await?;
    }
    Ok(r_37s)
}
