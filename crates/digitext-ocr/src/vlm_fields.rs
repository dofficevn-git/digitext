// Copyright (c) 2026 DigiText. All rights reserved.
use crate::fields::T76;
use digitext_core::VanBanField;
use digitext_db::ocr_fields::{self, Tdw};
use digitext_db::review_queue;
use sqlx::PgPool;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;
#[derive(Debug, Clone, PartialEq)]
pub struct Tpx {
    pub field: VanBanField,
    pub value: String,
    pub bbox: Option<[f32; 4]>,
    pub confidence: f32,
}
#[derive(Debug)]
pub enum VlmError {
    Backend(String),
}
impl std::fmt::Display for VlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VlmError::Backend(m) => write!(f, "bOG7l2kgYmFja2VuZCBWTE06IA=={m}"),
        }
    }
}
impl std::error::Error for VlmError {}
pub trait Tpw {
    fn extract(
        &self,
        image_jpeg: &[u8],
        page_w: u32,
        page_h: u32,
    ) -> impl Future<Output = Result<Vec<Tpx>, VlmError>> + Send;
}
pub struct T4t;
impl Tpw for T4t {
    #[allow(clippy::manual_async_fn)]
    fn extract(
        &self,
        _image_jpeg: &[u8],
        _page_w: u32,
        _page_h: u32,
    ) -> impl Future<Output = Result<Vec<Tpx>, VlmError>> + Send {
        async { Ok(Vec::new()) }
    }
}
pub trait T5b: Send + Sync {
    fn extract_dyn<'a>(
        &'a self,
        image_jpeg: &'a [u8],
        page_w: u32,
        page_h: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Tpx>, VlmError>> + Send + 'a>>;
}
impl<T: Tpw + Send + Sync> T5b for T {
    fn extract_dyn<'a>(
        &'a self,
        r_3iz: &'a [u8],
        r_3j1: u32,
        r_3j0: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Tpx>, VlmError>> + Send + 'a>> {
        Box::pin(self.extract(r_3iz, r_3j1, r_3j0))
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Tpy {
    pub min_fields: usize,
    pub tau: f32,
    pub max_low_conf: f32,
    pub keep_heuristic_conf: f32,
}
pub fn f_2ey(r_3j3: usize, r_3j5: f32, r_3j4: f32, r_3j2: Tpy) -> bool {
    r_3j3 < r_3j2.min_fields || r_3j5 < r_3j2.tau || r_3j4 > r_3j2.max_low_conf
}
pub fn f_1j1(f: VanBanField) -> bool {
    matches!(
        f,
        VanBanField::SoKyHieu
            | VanBanField::NgayThang
            | VanBanField::ToSo
            | VanBanField::SoTrang
            | VanBanField::TrangSo
            | VanBanField::ThuTu
    )
}
pub fn f_1sk(r_3j7: &[T76], r_3j9: Vec<Tpx>, keep_heuristic_conf: f32) -> Vec<Tpx> {
    let r_3j6: HashSet<VanBanField> = r_3j7
        .iter()
        .filter(|f| f.confidence >= keep_heuristic_conf)
        .map(|f| f.field)
        .collect();
    r_3j9
        .into_iter()
        .filter(|r_3j8| !r_3j8.value.trim().is_empty() && !r_3j6.contains(&r_3j8.field))
        .collect()
}
pub async fn f_1b6<V: Tpw>(
    r_3ja: &V,
    r_3jc: &[u8],
    r_3je: u32,
    r_3jd: u32,
    r_3jb: &[T76],
    keep_heuristic_conf: f32,
) -> Result<Vec<Tpx>, VlmError> {
    let r_3jf = r_3ja.extract(r_3jc, r_3je, r_3jd).await?;
    Ok(f_1sk(r_3jb, r_3jf, keep_heuristic_conf))
}
pub async fn f_s5(
    r_3jm: &PgPool,
    r_3jh: Uuid,
    r_3jl: i32,
    r_3ji: Uuid,
    r_3jj: &[Tpx],
) -> sqlx::Result<u64> {
    let mut r_3jk = 0u64;
    for f in r_3jj {
        let bbox = f
            .bbox
            .map(|r_3jg| serde_json::json!([r_3jg[0], r_3jg[1], r_3jg[2], r_3jg[3]]));
        ocr_fields::upsert(
            r_3jm,
            Tdw {
                document_id: r_3jh,
                page_no: r_3jl,
                engine_version_id: r_3ji,
                text_source: "ocr_corrected",
                field_key: f.field.as_str(),
                value: &f.value,
                bbox,
                confidence: Some(f.confidence),
                verified: false,
            },
        )
        .await?;
        r_3jk += 1;
    }
    if r_3jk > 0 {
        review_queue::enqueue(r_3jm, r_3jh, r_3jl, review_queue::C_I5).await?;
    }
    Ok(r_3jk)
}
