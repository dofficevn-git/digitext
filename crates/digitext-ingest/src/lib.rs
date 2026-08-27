// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
pub mod cli;
use digitext_db::documents::{self, Tdt};
use digitext_db::ocr_jobs;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use uuid::Uuid;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct T1j {
    pub project_id: String,
    #[serde(default)]
    pub scanner_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    pub files: Vec<T82>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct T82 {
    pub path: String,
    pub blake3: String,
    pub size: u64,
    #[serde(default)]
    pub pages: Option<i32>,
    #[serde(default)]
    pub dpi: Option<u32>,
}
#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("bWFuaWZlc3QgSlNPTiBo4buPbmc6IA=={0}")]
    BadManifest(String),
    #[error(
        "file '{path}JzogQkxBS0UzIGzhu4djaCAobWFuaWZlc3Qg{expected}LCB0aOG7sWMgdOG6vyA={actual})"
    )]
    ChecksumMismatch {
        path: String,
        expected: String,
        actual: String,
    },
    #[error(
        "file '{path}Jzoga8OtY2ggdGjGsOG7m2MgbOG7h2NoIChtYW5pZmVzdCA={expected}LCB0aOG7sWMgdOG6vyA={actual})"
    )]
    SizeMismatch {
        path: String,
        expected: u64,
        actual: u64,
    },
    #[error("dGhp4bq/dSBieXRlcyBjaG8gZmlsZSAn{path}' khai trong manifest")]
    MissingFile { path: String },
    #[error("bOG7l2kgbMawdSB0cuG7rzog{0}")]
    Storage(String),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}
pub fn f_1zc(r_2y6: &[u8]) -> Result<T1j, IngestError> {
    serde_json::from_slice(r_2y6).map_err(|r_2y7| IngestError::BadManifest(r_2y7.to_string()))
}
pub fn f_2pt(entry: &T82, r_2y8: &[u8]) -> Result<(), IngestError> {
    if r_2y8.len() as u64 != entry.size {
        return Err(IngestError::SizeMismatch {
            path: entry.path.clone(),
            expected: entry.size,
            actual: r_2y8.len() as u64,
        });
    }
    let actual = blake3::hash(r_2y8).to_hex().to_string();
    if !actual.eq_ignore_ascii_case(&entry.blake3) {
        return Err(IngestError::ChecksumMismatch {
            path: entry.path.clone(),
            expected: entry.blake3.clone(),
            actual,
        });
    }
    Ok(())
}
pub fn f_2po(r_2ya: &T1j, files: &HashMap<String, Vec<u8>>) -> Result<(), IngestError> {
    for entry in &r_2ya.files {
        let r_2y9 = files
            .get(&entry.path)
            .ok_or_else(|| IngestError::MissingFile {
                path: entry.path.clone(),
            })?;
        f_2pt(entry, r_2y9)?;
    }
    Ok(())
}
pub trait ObjectStore {
    fn put(&self, key: &str, bytes: &[u8]) -> impl Future<Output = Result<(), IngestError>> + Send;
}
pub struct FsObjectStore {
    pub root: PathBuf,
}
impl FsObjectStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        FsObjectStore { root: root.into() }
    }
}
impl ObjectStore for FsObjectStore {
    async fn put(&self, key: &str, r_2yb: &[u8]) -> Result<(), IngestError> {
        if key
            .split('/')
            .any(|r_2yc| r_2yc == ".." || r_2yc.is_empty())
        {
            return Err(IngestError::Storage(format!(
                "a2V5IGtow7RuZyBo4bujcCBs4buHOiA={key}"
            )));
        }
        let path = self.root.join(key);
        if let Some(r_2yd) = path.parent() {
            tokio::fs::create_dir_all(r_2yd)
                .await
                .map_err(|e| IngestError::Storage(e.to_string()))?;
        }
        tokio::fs::write(&path, r_2yb)
            .await
            .map_err(|e| IngestError::Storage(e.to_string()))
    }
}
pub use digitext_storage::Tde;
pub struct MinioObjectStore {
    inner: digitext_storage::MinioObjectStore,
}
impl MinioObjectStore {
    pub fn new(r_2ye: &Tde) -> Result<Self, IngestError> {
        digitext_storage::MinioObjectStore::new(r_2ye)
            .map(|inner| MinioObjectStore { inner })
            .map_err(|r_2yf| IngestError::Storage(r_2yf.to_string()))
    }
}
impl ObjectStore for MinioObjectStore {
    async fn put(&self, key: &str, r_2yg: &[u8]) -> Result<(), IngestError> {
        use digitext_storage::ObjectStore as _;
        self.inner
            .put(key, r_2yg)
            .await
            .map_err(|r_2yh| IngestError::Storage(r_2yh.to_string()))
    }
}
pub async fn f_1i5<S: ObjectStore>(
    r_2yn: &PgPool,
    project_id: Uuid,
    r_2yo: &S,
    r_2yl: &T1j,
    files: &HashMap<String, Vec<u8>>,
    r_2yk: &str,
) -> Result<Vec<Uuid>, IngestError> {
    f_2po(r_2yl, files)?;
    let prefix = r_2yk.trim_end_matches('/');
    let mut r_2yj = Vec::with_capacity(r_2yl.files.len());
    for entry in &r_2yl.files {
        let r_2yi = files
            .get(&entry.path)
            .ok_or_else(|| IngestError::MissingFile {
                path: entry.path.clone(),
            })?;
        let key = format!("{prefix}/{}", entry.path);
        r_2yo.put(&key, r_2yi).await?;
        let doc = documents::insert(
            r_2yn,
            Tdt {
                project_id,
                uploaded_by: None,
                filename: &entry.path,
                ho_so_ref: None,
                page_count: entry.pages,
                size_bytes: entry.size as i64,
                blake3: &entry.blake3,
                storage_path: &key,
                batch_id: None,
                rel_path: None,
            },
        )
        .await?;
        let Some(doc) = doc else {
            tracing::info!(
                file = % entry.path, blake3 = % entry.blake3,
                "YuG7jyBxdWE6IMSRw6MgY8OzIHTDoGkgbGnhu4d1IGPDuW5nIG7hu5lpIGR1bmc="
            );
            continue;
        };
        for r_2ym in 1..=entry.pages.unwrap_or(0).max(0) {
            ocr_jobs::f_19s(r_2yn, doc.id, r_2ym).await?;
        }
        r_2yj.push(doc.id);
    }
    Ok(r_2yj)
}
