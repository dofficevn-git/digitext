// Copyright (c) 2026 DigiText. All rights reserved.
mod builder;
pub mod codes;
pub mod ead;
pub mod fetch;
pub mod ids;
mod mets;
pub mod mime;
pub mod model;
mod premis;
pub mod schemas;
pub mod tt02;
pub mod validate;
pub mod wire;
mod xml;
pub use builder::{f_vn, T1g};
pub use model::{
    DocumentContent, OrgIdent, PackageKind, PackageMeta, PackageSourceEvent, T4o, T7z, T8s, Tga, Tlv, To, T51,
};
pub use validate::{f_2p7, Severity, Violation};
#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("invalid package request: {0:?}")]
    InvalidRequest(Vec<String>),
    #[error("zip error: {0}")]
    Zip(String),
    #[error("sinh văn bản xác thực DIP lỗi: {0}")]
    Cert(String),
}
