// Copyright (c) 2026 DigiText. All rights reserved.
use anyhow::Context;
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind: String,
    pub jwt_key_path: String,
    pub data_dir: String,
    pub behind_proxy: bool,
    pub pdf_store_dir: Option<String>,
    pub pdf_store_bucket: Option<String>,
    pub tsa_url: Option<String>,
    pub sign_trust_roots: Option<String>,
    pub tsa_trust_roots: Option<String>,
    pub pades_lt: bool,
    pub pades_lta: bool,
    pub minio: Option<digitext_storage::Tde>,
    pub sign_visible: bool,
    pub sign_stamp: bool,
}
fn f_1a5(r_3e: &str, r_3d: bool) -> bool {
    match std::env::var(r_3e) {
        Ok(r_3f) => match r_3f.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => r_3d,
        },
        Err(_) => r_3d,
    }
}
fn minio_from_env() -> Option<digitext_storage::Tde> {
    let r_3h = |r_3g: &str| {
        std::env::var(r_3g)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    Some(digitext_storage::Tde {
        endpoint: r_3h("DIGITEXT_MINIO_ENDPOINT")?,
        bucket: r_3h("DIGITEXT_MINIO_BUCKET")?,
        access_key: r_3h("DIGITEXT_MINIO_ACCESS_KEY")?,
        secret_key: r_3h("DIGITEXT_MINIO_SECRET_KEY")?,
        region: r_3h("DIGITEXT_MINIO_REGION").unwrap_or_else(|| "us-east-1".into()),
    })
}
impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            database_url: std::env::var("DATABASE_URL")
                .context("thiếu biến môi trường DATABASE_URL")?,
            bind: std::env::var("DIGITEXT_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into()),
            jwt_key_path: std::env::var("DIGITEXT_JWT_KEY_PATH")
                .unwrap_or_else(|_| "jwt-ed25519.pem".into()),
            data_dir: std::env::var("DIGITEXT_DATA_DIR").unwrap_or_else(|_| "data".into()),
            behind_proxy: std::env::var("DIGITEXT_BEHIND_PROXY")
                .map(|v| {
                    matches!(
                        v.trim().to_ascii_lowercase().as_str(),
                        "1" | "true" | "yes" | "on"
                    )
                })
                .unwrap_or(false),
            pdf_store_dir: std::env::var("DIGITEXT_PDF_STORE_DIR")
                .ok()
                .filter(|s| !s.is_empty()),
            pdf_store_bucket: std::env::var("DIGITEXT_PDF_STORE_BUCKET")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            tsa_url: std::env::var("DIGITEXT_TSA_URL")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            sign_trust_roots: std::env::var("DIGITEXT_SIGN_TRUST_ROOTS")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            tsa_trust_roots: std::env::var("DIGITEXT_TSA_TRUST_ROOTS")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            pades_lt: std::env::var("DIGITEXT_PADES_LT")
                .map(|v| {
                    matches!(
                        v.trim().to_ascii_lowercase().as_str(),
                        "1" | "true" | "yes" | "on"
                    )
                })
                .unwrap_or(false),
            pades_lta: std::env::var("DIGITEXT_PADES_LTA")
                .map(|v| {
                    matches!(
                        v.trim().to_ascii_lowercase().as_str(),
                        "1" | "true" | "yes" | "on"
                    )
                })
                .unwrap_or(false),
            minio: minio_from_env(),
            sign_visible: f_1a5("DIGITEXT_SIGN_VISIBLE", true),
            sign_stamp: f_1a5("DIGITEXT_SIGN_STAMP", false),
        })
    }
}
