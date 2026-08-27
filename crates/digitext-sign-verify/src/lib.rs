// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
pub mod chain;
pub mod cms;
pub mod dss;
pub mod ocsp;
pub mod pdf;
pub mod tsa;
pub use pdf::{f_1cm, f_1cn, f_1co, f_2q1, f_2q2, f_xy, finalize, prepare, verify, Thg, Tpr};
#[derive(Debug, thiserror::Error)]
pub enum SignError {
    #[error("bOG7l2kgREVSL0FTTi4xOiA={0}")]
    Der(String),
    #[error("bOG7l2kgQ01TOiA={0}")]
    Cms(String),
    #[error("bOG7l2kgUERGIGNo4buvIGvDvTog{0}")]
    Pdf(String),
}
impl SignError {
    pub fn der(r_4sw: impl core::fmt::Display) -> Self {
        SignError::Der(r_4sw.to_string())
    }
}
