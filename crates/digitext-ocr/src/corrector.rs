// Copyright (c) 2026 DigiText. All rights reserved.
use crate::infer::Tgi;
use digitext_db::ocr_results::Tdx;
use digitext_db::{ocr_results, review_queue};
pub use digitext_vi::Lexicon;
use sqlx::PgPool;
use std::future::Future;
use uuid::Uuid;
#[derive(Debug, Clone, PartialEq)]
pub struct Tm9 {
    pub corrected: String,
    pub confidence: f32,
}
#[derive(Debug)]
pub enum CorrectorError {
    Backend(String),
}
impl std::fmt::Display for CorrectorError {
    fn fmt(&self, r_31w: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorrectorError::Backend(m) => {
                write!(r_31w, "bOG7l2kgYmFja2VuZCBMTE0gY29ycmVjdG9yOiA={m}")
            }
        }
    }
}
impl std::error::Error for CorrectorError {}
pub trait Tb4 {
    fn suggest(
        &self,
        raw: &str,
    ) -> impl Future<Output = Result<Option<Tm9>, CorrectorError>> + Send;
}
pub struct T4s;
impl Tb4 for T4s {
    #[allow(clippy::manual_async_fn)]
    fn suggest(
        &self,
        _raw: &str,
    ) -> impl Future<Output = Result<Option<Tm9>, CorrectorError>> + Send {
        async { Ok(None) }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct T8j {
    pub tau: f32,
    pub max_edit: usize,
    pub min_llm_conf: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    KeepRaw,
    Accept(String),
    Review,
}
pub fn f_1jf(r_31x: &str) -> bool {
    r_31x.chars().any(|c| c.is_ascii_digit())
        || r_31x
            .split_whitespace()
            .any(|r_31y| f_1jh(r_31y.trim_matches(|c: char| !c.is_ascii_alphabetic())))
}
fn f_1jh(r_320: &str) -> bool {
    !r_320.is_empty()
        && r_320
            .chars()
            .all(|r_31z| matches!(r_31z, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
}
pub fn f_1mw(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut r_327: Vec<usize> = (0..=b.len()).collect();
    for (r_325, &r_321) in a.iter().enumerate() {
        let mut r_324 = vec![0usize; b.len() + 1];
        r_324[0] = r_325 + 1;
        for (r_326, &r_322) in b.iter().enumerate() {
            let r_323 = usize::from(r_321 != r_322);
            r_324[r_326 + 1] = (r_327[r_326 + 1] + 1)
                .min(r_324[r_326] + 1)
                .min(r_327[r_326] + r_323);
        }
        r_327 = r_324;
    }
    r_327[b.len()]
}
pub fn decide(r_32b: &str, r_32d: Option<&Tm9>, r_329: f32, r_328: T8j, r_32a: bool) -> Decision {
    let Some(r_32c) = r_32d else {
        return Decision::KeepRaw;
    };
    if r_32c.corrected == r_32b {
        return Decision::KeepRaw;
    }
    if f_1jf(r_32b) || f_1jf(&r_32c.corrected) {
        return Decision::Review;
    }
    if r_329 >= r_328.tau {
        return Decision::KeepRaw;
    }
    if r_32c.confidence < r_328.min_llm_conf {
        return Decision::Review;
    }
    if f_1mw(r_32b, &r_32c.corrected) <= r_328.max_edit && r_32a {
        Decision::Accept(r_32c.corrected.clone())
    } else {
        Decision::Review
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Tge {
    pub texts: Vec<String>,
    pub accepted: usize,
    pub needs_review: bool,
}
impl Tge {
    pub fn corrected_text(&self) -> String {
        self.texts.join(" ")
    }
}
pub async fn f_11o<C, L>(
    page: &Tgi,
    corrector: &C,
    r_32f: T8j,
    r_32j: L,
) -> Result<Tge, CorrectorError>
where
    C: Tb4,
    L: Fn(&str) -> bool,
{
    let mut texts = Vec::with_capacity(page.texts.len());
    let mut accepted = 0usize;
    let mut needs_review = false;
    for (r_32h, r_32k) in page.texts.iter().enumerate() {
        let r_32g = page.confidences.get(r_32h).copied().unwrap_or(1.0);
        if r_32g >= r_32f.tau {
            texts.push(r_32k.clone());
            continue;
        }
        let r_32m = corrector.suggest(r_32k).await?;
        let r_32i = r_32m
            .as_ref()
            .map(|r_32l| r_32j(&r_32l.corrected))
            .unwrap_or(false);
        match decide(r_32k, r_32m.as_ref(), r_32g, r_32f, r_32i) {
            Decision::Accept(r_32e) => {
                accepted += 1;
                texts.push(r_32e);
            }
            Decision::Review => {
                needs_review = true;
                texts.push(r_32k.clone());
            }
            Decision::KeepRaw => texts.push(r_32k.clone()),
        }
    }
    Ok(Tge {
        texts,
        accepted,
        needs_review,
    })
}
#[derive(Debug)]
pub enum ApplyError {
    Corrector(CorrectorError),
    Db(sqlx::Error),
}
impl From<CorrectorError> for ApplyError {
    fn from(r_32n: CorrectorError) -> Self {
        ApplyError::Corrector(r_32n)
    }
}
impl From<sqlx::Error> for ApplyError {
    fn from(r_32o: sqlx::Error) -> Self {
        ApplyError::Db(r_32o)
    }
}
#[allow(clippy::too_many_arguments)]
pub async fn f_s4<C, L>(
    r_32x: &PgPool,
    r_32s: Uuid,
    r_32w: i32,
    r_32t: Uuid,
    page: &Tgi,
    corrector: &C,
    r_32u: T8j,
    r_32p: f32,
    r_32v: L,
) -> Result<Tge, ApplyError>
where
    C: Tb4,
    L: Fn(&str) -> bool,
{
    let r_32r = f_11o(page, corrector, r_32u, r_32v).await?;
    if r_32r.accepted > 0 {
        let r_32q = digitext_vi::normalize(&r_32r.corrected_text());
        ocr_results::upsert(
            r_32x,
            Tdx {
                document_id: r_32s,
                page_no: r_32w,
                engine_version_id: r_32t,
                text_source: "ocr_corrected",
                content: &r_32q,
                mean_confidence: page.mean_confidence(),
                low_conf_ratio: page.low_conf_ratio(r_32p),
                boxes: None,
            },
        )
        .await?;
    }
    if r_32r.needs_review {
        review_queue::enqueue(r_32x, r_32s, r_32w, review_queue::C_I2).await?;
    }
    Ok(r_32r)
}
