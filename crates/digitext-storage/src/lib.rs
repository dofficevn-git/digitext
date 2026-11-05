// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use uuid::Uuid;
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("key không hợp lệ: {0:?}")]
    InvalidKey(String),
    #[error("lỗi I/O kho object: {0}")]
    Io(String),
    #[error("kho WORM: không được xoá object gói tin {0:?}")]
    Immutable(String),
}
pub type Tm3<'a, T> = Pin<Box<dyn Future<Output = Result<T, StorageError>> + Send + 'a>>;
pub type T1k = Pin<Box<dyn futures_core::Stream<Item = Result<bytes::Bytes, StorageError>> + Send>>;
pub struct Tf8 {
    pub total_len: u64,
    pub start: u64,
    pub len: u64,
    pub body: T1k,
}
pub type ByteRange = (u64, Option<u64>);
pub trait ObjectStore: Send + Sync {
    /// Chuẩn bị kho trước khi dùng (tạo bucket nếu thiếu). Mặc định không làm gì.
    fn ensure_ready<'a>(&'a self) -> Tm3<'a, ()> {
        Box::pin(async { Ok(()) })
    }
    fn put<'a>(&'a self, key: &'a str, bytes: &'a [u8]) -> Tm3<'a, ()>;
    fn get<'a>(&'a self, key: &'a str) -> Tm3<'a, Option<Vec<u8>>>;
    fn delete<'a>(&'a self, key: &'a str) -> Tm3<'a, ()>;
    /// Xoá khi THỰC HIỆN QUYẾT ĐỊNH TIÊU HUỶ — đường duy nhất được xoá gói lưu trữ ghi một lần (`WormGuard` chặn `delete`
    /// với `*-packages/`). Chỉ gọi từ `POST /disposal/batches/{id}/execute` (chốt với anh Hiếu 5/11/2026).
    fn xoa_theo_quyet_dinh<'a>(&'a self, key: &'a str) -> Tm3<'a, ()> {
        self.delete(key)
    }
    fn list_prefix<'a>(&'a self, prefix: &'a str) -> Tm3<'a, Vec<String>>;
    fn exists<'a>(&'a self, key: &'a str) -> Tm3<'a, bool> {
        Box::pin(async move { Ok(self.get(key).await?.is_some()) })
    }
    fn get_stream<'a>(&'a self, key: &'a str, range: Option<ByteRange>) -> Tm3<'a, Option<Tf8>> {
        Box::pin(async move {
            let Some(bytes) = self.get(key).await? else {
                return Ok(None);
            };
            let total = bytes.len() as u64;
            let (start, len) = f_296(range, total).ok_or_else(|| {
                StorageError::Io(format!("dải byte ngoài phạm vi ({key})"))
            })?;
            let part = bytes[start as usize..(start + len) as usize].to_vec();
            Ok(Some(Tf8 {
                total_len: total,
                start,
                len,
                body: Box::pin(futures_util::stream::iter([Ok(bytes::Bytes::from(part))])),
            }))
        })
    }
}
fn f_296(range: Option<ByteRange>, total: u64) -> Option<(u64, u64)> {
    match range {
        None => Some((0, total)),
        Some((start, _)) if start >= total => None,
        Some((start, end)) => {
            let end = end.map_or(total - 1, |e| e.min(total - 1));
            if end < start {
                None
            } else {
                Some((start, end - start + 1))
            }
        }
    }
}
pub fn f_2qs(project_id: Uuid, document_id: Uuid) -> String {
    format!("{project_id}-warm/{document_id}.pdf")
}
pub fn f_23q(project_id: Uuid, document_id: Uuid) -> String {
    format!("{project_id}-warm/{document_id}.prepared.pdf")
}
pub fn f_2fc(project_id: Uuid, document_id: Uuid) -> String {
    format!("{project_id}-warm/{document_id}.signed.pdf")
}
pub fn f_1xr(document_id: Uuid, page_no: i32) -> String {
    format!("{document_id}/{page_no}.jpg")
}
pub fn f_1x5(project_id: Uuid, package_id: Uuid) -> String {
    format!("{project_id}-packages/{package_id}.zip")
}
fn f_2p4(key: &str) -> Result<(), StorageError> {
    if key.is_empty() || key.split('/').any(|c| c == ".." || c.is_empty()) {
        return Err(StorageError::InvalidKey(key.to_owned()));
    }
    Ok(())
}
fn f_2pa(prefix: &str) -> Result<(), StorageError> {
    if prefix.split('/').any(|c| c == "..") {
        return Err(StorageError::InvalidKey(prefix.to_owned()));
    }
    Ok(())
}
pub struct FsObjectStore {
    root: PathBuf,
}
impl FsObjectStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        FsObjectStore { root: root.into() }
    }
    fn path_of(&self, key: &str) -> Result<PathBuf, StorageError> {
        f_2p4(key)?;
        Ok(self.root.join(key))
    }
}
impl ObjectStore for FsObjectStore {
    fn put<'a>(&'a self, key: &'a str, bytes: &'a [u8]) -> Tm3<'a, ()> {
        Box::pin(async move {
            let r_5fu = self.path_of(key)?;
            if let Some(r_5ft) = r_5fu.parent() {
                tokio::fs::create_dir_all(r_5ft)
                    .await
                    .map_err(|e| StorageError::Io(e.to_string()))?;
            }
            let tmp = f_2l9(&r_5fu);
            tokio::fs::write(&tmp, bytes)
                .await
                .map_err(|e| StorageError::Io(e.to_string()))?;
            tokio::fs::rename(&tmp, &r_5fu)
                .await
                .map_err(|e| StorageError::Io(e.to_string()))
        })
    }
    fn delete<'a>(&'a self, key: &'a str) -> Tm3<'a, ()> {
        Box::pin(async move {
            let r_5fv = self.path_of(key)?;
            match tokio::fs::remove_file(&r_5fv).await {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(StorageError::Io(e.to_string())),
            }
        })
    }
    fn get<'a>(&'a self, key: &'a str) -> Tm3<'a, Option<Vec<u8>>> {
        Box::pin(async move {
            let r_5fw = self.path_of(key)?;
            match tokio::fs::read(&r_5fw).await {
                Ok(bytes) => Ok(Some(bytes)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(StorageError::Io(e.to_string())),
            }
        })
    }
    fn exists<'a>(&'a self, key: &'a str) -> Tm3<'a, bool> {
        Box::pin(async move {
            let r_5fy = self.path_of(key)?;
            match tokio::fs::metadata(&r_5fy).await {
                Ok(r_5fx) => Ok(r_5fx.is_file()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
                Err(e) => Err(StorageError::Io(e.to_string())),
            }
        })
    }
    fn get_stream<'a>(&'a self, key: &'a str, range: Option<ByteRange>) -> Tm3<'a, Option<Tf8>> {
        Box::pin(async move {
            use futures_util::TryStreamExt;
            use tokio::io::{AsyncReadExt, AsyncSeekExt};
            let r_5g1 = self.path_of(key)?;
            let mut r_5g0 = match tokio::fs::File::open(&r_5g1).await {
                Ok(r_5fz) => r_5fz,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(StorageError::Io(e.to_string())),
            };
            let total = r_5g0
                .metadata()
                .await
                .map_err(|e| StorageError::Io(e.to_string()))?
                .len();
            let (start, len) = f_296(range, total).ok_or_else(|| {
                StorageError::Io(format!("dải byte ngoài phạm vi ({key})"))
            })?;
            if start > 0 {
                r_5g0
                    .seek(std::io::SeekFrom::Start(start))
                    .await
                    .map_err(|e| StorageError::Io(e.to_string()))?;
            }
            let body = tokio_util::io::ReaderStream::new(r_5g0.take(len))
                .map_err(|e| StorageError::Io(e.to_string()));
            Ok(Some(Tf8 {
                total_len: total,
                start,
                len,
                body: Box::pin(body),
            }))
        })
    }
    fn list_prefix<'a>(&'a self, prefix: &'a str) -> Tm3<'a, Vec<String>> {
        Box::pin(async move {
            f_2pa(prefix)?;
            let mut r_5g5 = Vec::new();
            let mut r_5g6 = vec![(self.root.clone(), String::new())];
            while let Some((r_5g2, rel)) = r_5g6.pop() {
                let mut rd = match tokio::fs::read_dir(&r_5g2).await {
                    Ok(rd) => rd,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(e) => {
                        return Err(StorageError::Io(format!(
                            "đọc {}: {e}",
                            r_5g2.display()
                        )));
                    }
                };
                while let Some(r_5g3) = rd.next_entry().await.map_err(|e| {
                    StorageError::Io(format!("duyệt {}: {e}", r_5g2.display()))
                })? {
                    let name = r_5g3.file_name().to_string_lossy().into_owned();
                    let child = if rel.is_empty() {
                        name
                    } else {
                        format!("{rel}/{name}")
                    };
                    let r_5g4 = r_5g3
                        .file_type()
                        .await
                        .map_err(|e| StorageError::Io(format!("stat {child}: {e}")))?;
                    if r_5g4.is_dir() {
                        r_5g6.push((r_5g3.path(), child));
                    } else if child.starts_with(prefix) {
                        r_5g5.push(child);
                    }
                }
            }
            Ok(r_5g5)
        })
    }
}
fn f_2l9(r_5g7: &Path) -> PathBuf {
    let mut r_5g8 = r_5g7.as_os_str().to_owned();
    r_5g8.push(".tmp");
    PathBuf::from(r_5g8)
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tde {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}
pub fn f_vr(
    minio: Option<&Tde>,
    pdf_bucket: Option<&str>,
    r_5ga: Option<&str>,
) -> Result<Option<std::sync::Arc<dyn ObjectStore>>, StorageError> {
    let pdf_bucket = pdf_bucket.filter(|b| !b.is_empty());
    if let Some(bucket) = pdf_bucket {
        let Some(mc) = minio else {
            return Err(
                StorageError::Io(
                    "khai bucket kho PDF 2 lớp nhưng THIẾU cấu hình MinIO (DIGITEXT_MINIO_*)"
                        .into(),
                ),
            );
        };
        let r_5g9 = Tde {
            bucket: bucket.to_owned(),
            ..mc.clone()
        };
        return Ok(Some(std::sync::Arc::new(MinioObjectStore::new(&r_5g9)?)));
    }
    Ok(r_5ga
        .filter(|d| !d.is_empty())
        .map(|d| std::sync::Arc::new(FsObjectStore::new(d)) as std::sync::Arc<dyn ObjectStore>))
}
pub struct MinioObjectStore {
    bucket: rusty_s3::Bucket,
    creds: rusty_s3::Credentials,
    http: reqwest::Client,
    sign_ttl: std::time::Duration,
}
impl MinioObjectStore {
    pub fn new(r_5gb: &Tde) -> Result<Self, StorageError> {
        let endpoint = url::Url::parse(&r_5gb.endpoint).map_err(|e| {
            StorageError::Io(format!(
                "endpoint MinIO không hợp lệ ({}): {e}",
                r_5gb.endpoint
            ))
        })?;
        let bucket = rusty_s3::Bucket::new(
            endpoint,
            rusty_s3::UrlStyle::Path,
            r_5gb.bucket.clone(),
            r_5gb.region.clone(),
        )
        .map_err(|e| StorageError::Io(format!("bucket MinIO '{}': {e}", r_5gb.bucket)))?;
        Ok(MinioObjectStore {
            bucket,
            creds: rusty_s3::Credentials::new(r_5gb.access_key.clone(), r_5gb.secret_key.clone()),
            http: reqwest::Client::new(),
            sign_ttl: std::time::Duration::from_secs(300),
        })
    }
}
impl ObjectStore for MinioObjectStore {
    fn ensure_ready<'a>(&'a self) -> Tm3<'a, ()> {
        Box::pin(async move {
            use rusty_s3::S3Action;
            let name = self.bucket.name().to_owned();
            let signed = self.bucket.create_bucket(&self.creds).sign(self.sign_ttl);
            let r = self
                .http
                .put(signed)
                .send()
                .await
                .map_err(|e| StorageError::Io(format!("tạo bucket {name}: {e}")))?;
            let st = r.status();
            // 409 = bucket đã có (BucketAlreadyOwnedByYou) — mục tiêu đã đạt.
            if st.is_success() || st == reqwest::StatusCode::CONFLICT {
                Ok(())
            } else {
                let body = r.text().await.unwrap_or_default();
                Err(StorageError::Io(format!(
                    "tạo bucket {name}: HTTP {st} {}",
                    body.chars().take(200).collect::<String>()
                )))
            }
        })
    }
    fn put<'a>(&'a self, key: &'a str, bytes: &'a [u8]) -> Tm3<'a, ()> {
        Box::pin(async move {
            f_2p4(key)?;
            use rusty_s3::S3Action;
            let signed = self
                .bucket
                .put_object(Some(&self.creds), key)
                .sign(self.sign_ttl);
            self.http
                .put(signed)
                .body(bytes.to_vec())
                .send()
                .await
                .map_err(|e| StorageError::Io(format!("PUT {key}: {e}")))?
                .error_for_status()
                .map_err(|e| StorageError::Io(format!("PUT {key}: {e}")))?;
            Ok(())
        })
    }
    fn delete<'a>(&'a self, key: &'a str) -> Tm3<'a, ()> {
        Box::pin(async move {
            f_2p4(key)?;
            use rusty_s3::S3Action;
            let signed = self
                .bucket
                .delete_object(Some(&self.creds), key)
                .sign(self.sign_ttl);
            let r_5gc = self
                .http
                .delete(signed)
                .send()
                .await
                .map_err(|e| StorageError::Io(format!("DELETE {key}: {e}")))?;
            if r_5gc.status().is_success() || r_5gc.status() == reqwest::StatusCode::NOT_FOUND {
                Ok(())
            } else {
                Err(StorageError::Io(format!(
                    "DELETE {key}: {}",
                    r_5gc.status()
                )))
            }
        })
    }
    fn get<'a>(&'a self, key: &'a str) -> Tm3<'a, Option<Vec<u8>>> {
        Box::pin(async move {
            f_2p4(key)?;
            use rusty_s3::S3Action;
            let signed = self
                .bucket
                .get_object(Some(&self.creds), key)
                .sign(self.sign_ttl);
            let resp = self
                .http
                .get(signed)
                .send()
                .await
                .map_err(|e| StorageError::Io(format!("GET {key}: {e}")))?;
            if resp.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }
            let resp = resp
                .error_for_status()
                .map_err(|e| StorageError::Io(format!("GET {key}: {e}")))?;
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| StorageError::Io(format!("GET {key} đọc body: {e}")))?;
            Ok(Some(bytes.to_vec()))
        })
    }
    fn exists<'a>(&'a self, key: &'a str) -> Tm3<'a, bool> {
        Box::pin(async move {
            f_2p4(key)?;
            use rusty_s3::S3Action;
            let signed = self
                .bucket
                .head_object(Some(&self.creds), key)
                .sign(self.sign_ttl);
            let r_5gd = self
                .http
                .head(signed)
                .send()
                .await
                .map_err(|e| StorageError::Io(format!("HEAD {key}: {e}")))?;
            if r_5gd.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(false);
            }
            r_5gd
                .error_for_status()
                .map_err(|e| StorageError::Io(format!("HEAD {key}: {e}")))?;
            Ok(true)
        })
    }
    fn get_stream<'a>(&'a self, key: &'a str, range: Option<ByteRange>) -> Tm3<'a, Option<Tf8>> {
        Box::pin(async move {
            f_2p4(key)?;
            use futures_util::TryStreamExt;
            use rusty_s3::S3Action;
            let signed = self
                .bucket
                .get_object(Some(&self.creds), key)
                .sign(self.sign_ttl);
            let mut r_5gf = self.http.get(signed);
            if let Some((start, end)) = range {
                let r_5gg = match end {
                    Some(e) => format!("bytes={start}-{e}"),
                    None => format!("bytes={start}-"),
                };
                r_5gf = r_5gf.header(reqwest::header::RANGE, r_5gg);
            }
            let resp = r_5gf
                .send()
                .await
                .map_err(|e| StorageError::Io(format!("GET {key}: {e}")))?;
            if resp.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }
            let resp = resp
                .error_for_status()
                .map_err(|e| StorageError::Io(format!("GET {key}: {e}")))?;
            let (total, start, len) = if resp.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                f_1yp(
                    resp.headers()
                        .get(reqwest::header::CONTENT_RANGE)
                        .and_then(|r_5gh| r_5gh.to_str().ok())
                        .unwrap_or(""),
                )
                .ok_or_else(|| {
                    StorageError::Io(format!(
                        "GET {key}: Content-Range không đọc được"
                    ))
                })?
            } else {
                let r_5ge = resp.content_length().ok_or_else(|| {
                    StorageError::Io(format!("GET {key}: thiếu Content-Length"))
                })?;
                (r_5ge, 0, r_5ge)
            };
            let body = resp
                .bytes_stream()
                .map_err(move |e| StorageError::Io(format!("đọc body stream: {e}")));
            Ok(Some(Tf8 {
                total_len: total,
                start,
                len,
                body: Box::pin(body),
            }))
        })
    }
    fn list_prefix<'a>(&'a self, prefix: &'a str) -> Tm3<'a, Vec<String>> {
        Box::pin(async move {
            f_2pa(prefix)?;
            use rusty_s3::S3Action;
            let mut r_5gj = Vec::new();
            let mut r_5gm: Option<String> = None;
            loop {
                let mut r_5gi = self.bucket.list_objects_v2(Some(&self.creds));
                if !prefix.is_empty() {
                    r_5gi.with_prefix(prefix);
                }
                if let Some(r_5gl) = &r_5gm {
                    r_5gi.with_continuation_token(r_5gl.as_str());
                }
                let signed = r_5gi.sign(self.sign_ttl);
                let body = self
                    .http
                    .get(signed)
                    .send()
                    .await
                    .map_err(|e| StorageError::Io(format!("LIST {prefix:?}: {e}")))?
                    .error_for_status()
                    .map_err(|e| StorageError::Io(format!("LIST {prefix:?}: {e}")))?
                    .text()
                    .await
                    .map_err(|e| {
                        StorageError::Io(format!("LIST {prefix:?} đọc body: {e}"))
                    })?;
                let r_5gk =
                    rusty_s3::actions::ListObjectsV2::parse_response(&body).map_err(|e| {
                        StorageError::Io(format!("LIST {prefix:?} phân tích XML: {e}"))
                    })?;
                for c in r_5gk.contents {
                    r_5gj.push(f_13g(&c.key));
                }
                r_5gm = r_5gk.next_continuation_token;
                if r_5gm.is_none() {
                    break;
                }
            }
            Ok(r_5gj)
        })
    }
}
fn f_1yp(r_5gp: &str) -> Option<(u64, u64, u64)> {
    let r_5gn = r_5gp.strip_prefix("bytes ")?;
    let (r_5go, total) = r_5gn.split_once('/')?;
    let (a, b) = r_5go.split_once('-')?;
    let (a, b, total) = (
        a.parse::<u64>().ok()?,
        b.parse::<u64>().ok()?,
        total.parse::<u64>().ok()?,
    );
    if b < a || total <= b {
        return None;
    }
    Some((total, a, b - a + 1))
}
pub fn f_1jb(key: &str) -> bool {
    key.split('/')
        .next()
        .is_some_and(|r_5gq| r_5gq.ends_with("-packages"))
}
pub struct WormGuard {
    inner: std::sync::Arc<dyn ObjectStore>,
}
impl WormGuard {
    pub fn new(inner: std::sync::Arc<dyn ObjectStore>) -> Self {
        WormGuard { inner }
    }
}
impl ObjectStore for WormGuard {
    fn ensure_ready<'a>(&'a self) -> Tm3<'a, ()> {
        self.inner.ensure_ready()
    }
    /// Gói lưu trữ (`*-packages/`) ghi một lần: đã có thì chỉ chấp nhận ghi lại ĐÚNG nội dung cũ (worker thử lại, nhập gói
    /// chạy lại), khác nội dung → từ chối (4/11/2026, trước đó chỉ chặn xoá).
    fn put<'a>(&'a self, key: &'a str, bytes: &'a [u8]) -> Tm3<'a, ()> {
        if f_1jb(key) {
            return Box::pin(async move {
                match self.inner.get(key).await? {
                    Some(cu) if cu.as_slice() == bytes => Ok(()),
                    Some(_) => Err(StorageError::Immutable(key.to_owned())),
                    None => self.inner.put(key, bytes).await,
                }
            });
        }
        self.inner.put(key, bytes)
    }
    fn delete<'a>(&'a self, key: &'a str) -> Tm3<'a, ()> {
        if f_1jb(key) {
            let key = key.to_owned();
            return Box::pin(async move { Err(StorageError::Immutable(key)) });
        }
        self.inner.delete(key)
    }
    fn xoa_theo_quyet_dinh<'a>(&'a self, key: &'a str) -> Tm3<'a, ()> {
        self.inner.delete(key)
    }
    fn get<'a>(&'a self, key: &'a str) -> Tm3<'a, Option<Vec<u8>>> {
        self.inner.get(key)
    }
    fn list_prefix<'a>(&'a self, prefix: &'a str) -> Tm3<'a, Vec<String>> {
        self.inner.list_prefix(prefix)
    }
    fn exists<'a>(&'a self, key: &'a str) -> Tm3<'a, bool> {
        self.inner.exists(key)
    }
    fn get_stream<'a>(&'a self, key: &'a str, range: Option<ByteRange>) -> Tm3<'a, Option<Tf8>> {
        self.inner.get_stream(key, range)
    }
}
fn f_13g(r_5gr: &str) -> String {
    percent_encoding::percent_decode_str(r_5gr)
        .decode_utf8()
        .map(|r_5gs| r_5gs.into_owned())
        .unwrap_or_else(|_| r_5gr.to_owned())
}

#[cfg(test)]
mod worm_tests {
    use super::*;

    #[tokio::test]
    async fn goi_ghi_mot_lan_nhung_ghi_lai_dung_noi_dung_duoc() {
        let dir = std::env::temp_dir().join(format!("worm-{}", Uuid::now_v7()));
        let g = WormGuard::new(std::sync::Arc::new(FsObjectStore::new(&dir)));
        let k = "p-packages/a.zip";
        g.put(k, b"goi").await.unwrap();
        g.put(k, b"goi").await.unwrap();
        assert!(matches!(g.put(k, b"khac").await, Err(StorageError::Immutable(_))));
        assert!(matches!(g.delete(k).await, Err(StorageError::Immutable(_))));
        g.xoa_theo_quyet_dinh(k).await.unwrap();
        assert!(g.get(k).await.unwrap().is_none());
        g.put("p-warm/x.pdf", b"1").await.unwrap();
        g.put("p-warm/x.pdf", b"2").await.unwrap();
        let _ = std::fs::remove_dir_all(dir);
    }
}
