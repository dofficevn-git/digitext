// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
pub mod chain;
pub mod cms;
pub mod dss;
pub mod nhieu;
pub mod ocsp;
pub mod pdf;
pub mod tsa;
pub use nhieu::{co_chu_ky, kiem_tat_ca, ket_luan_tep, ChuKyPdf, KetLuanTep, KetQuaNhieu};
pub use pdf::{f_1cm, f_1cn, f_1co, f_2q1, f_2q2, f_xy, finalize, prepare, signer_info, thoi_gian_ky, verify, SignerInfo, Thg, Tpr};
#[derive(Debug, thiserror::Error)]
pub enum SignError {
    #[error("lỗi DER/ASN.1: {0}")]
    Der(String),
    #[error("lỗi CMS: {0}")]
    Cms(String),
    #[error("lỗi PDF chữ ký: {0}")]
    Pdf(String),
}
impl SignError {
    pub fn der(r_4sw: impl core::fmt::Display) -> Self {
        SignError::Der(r_4sw.to_string())
    }
}
