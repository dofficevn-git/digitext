// Copyright (c) 2026 DigiText. All rights reserved.
use crate::error::WorkerError;
use crate::infer::Tile;
use crate::tiling::f_2l0;
use crate::worker::Tgj;
use digitext_storage::{f_1xr, ObjectStore};
use image::RgbImage;
use sqlx::PgPool;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;
#[derive(Debug, Clone)]
pub struct FetchedPage {
    pub bytes: Vec<u8>,
    pub dpi: Option<u32>,
}
impl FetchedPage {
    pub fn bytes_only(bytes: Vec<u8>) -> Self {
        FetchedPage { bytes, dpi: None }
    }
}
pub trait Tgd: Send + Sync {
    fn fetch(
        &self,
        document_id: Uuid,
        page_no: i32,
    ) -> impl Future<Output = Result<FetchedPage, WorkerError>> + Send;
}
#[derive(Debug, Clone)]
pub struct Tgf {
    pub jpeg: Vec<u8>,
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: Option<u32>,
}
pub fn f_1hv(bytes: &[u8]) -> Result<(u32, u32), WorkerError> {
    image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| {
            WorkerError::CorruptInput(format!(
                "đoán định dạng ảnh trang lỗi: {e}"
            ))
        })?
        .into_dimensions()
        .map_err(|e| {
            WorkerError::CorruptInput(format!(
                "đọc kích thước ảnh trang lỗi: {e}"
            ))
        })
}
/// Trần bộ nhớ khi giải mã MỘT ảnh trang. Mặc định của `image` là 512 MiB → bản quét màu khổ lớn
/// độ phân giải cao (LTLSBN, 20/10/2026: 443 trang / 212 tài liệu, ~8–9 MB JPEG mỗi trang) báo
/// "Memory limit exceeded" và bị đánh hỏng VĨNH VIỄN. 4 GiB đủ cho ảnh ~20.000 × 20.000 px RGB.
const DECODE_MAX_ALLOC: u64 = 4 << 30;
pub fn f_13f(bytes: &[u8]) -> Result<RgbImage, WorkerError> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| WorkerError::CorruptInput(format!("đoán định dạng ảnh trang lỗi: {e}")))?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(DECODE_MAX_ALLOC);
    reader.limits(limits);
    let r_3di = reader.decode().map_err(|e| {
        WorkerError::CorruptInput(format!("giải mã ảnh trang lỗi: {e}"))
    })?;
    Ok(r_3di.to_rgb8())
}
/// JPEG gửi thẳng cho dịch vụ OCR được không: phải là JPEG 1 hoặc 3 kênh (không CMYK) và
/// KHÔNG mang EXIF — OpenCV tự xoay ảnh theo EXIF Orientation còn `image` thì không, toạ độ
/// khung chữ sẽ lệch khỏi ảnh mà lớp PDF/A dùng.
pub fn jpeg_passthrough_ok(b: &[u8]) -> bool {
    if b.len() < 4 || b[0] != 0xFF || b[1] != 0xD8 {
        return false;
    }
    let mut i = 2;
    while i + 4 <= b.len() {
        if b[i] != 0xFF {
            return false;
        }
        let m = b[i + 1];
        if m == 0xFF {
            i += 1;
            continue;
        }
        if m == 0x01 || (0xD0..=0xD7).contains(&m) {
            i += 2;
            continue;
        }
        let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
        let seg = &b[(i + 4).min(b.len())..(i + 2 + len).min(b.len())];
        match m {
            0xE1 if seg.starts_with(b"Exif\0") => return false,
            0xC0..=0xCF if !matches!(m, 0xC4 | 0xC8 | 0xCC) => {
                return matches!(seg.get(5), Some(1) | Some(3));
            }
            0xDA | 0xD9 => return false,
            _ => {}
        }
        i += 2 + len;
    }
    false
}
pub fn f_2l3(r_3dk: &RgbImage, tile_size: u32, r_3dl: u32) -> Vec<Tile> {
    let (r_3dn, r_3dj) = (r_3dk.width(), r_3dk.height());
    f_2l0(r_3dn, r_3dj, tile_size, r_3dl)
        .into_iter()
        .map(|r| {
            let mut r_3dm = Vec::with_capacity((r.w * r.h * 3) as usize);
            for r_3dp in r.y..r.y + r.h {
                for r_3do in r.x..r.x + r.w {
                    r_3dm.extend_from_slice(&r_3dk.get_pixel(r_3do, r_3dp).0);
                }
            }
            Tile {
                x: r.x,
                y: r.y,
                width: r.w,
                height: r.h,
                pixels: r_3dm,
                encoded: None,
            }
        })
        .collect()
}
pub struct ImagePageSource<F> {
    bytes: F,
    tile_size: u32,
    tile_overlap: u32,
}
impl<F: Tgd> ImagePageSource<F> {
    pub fn new(bytes: F, tile_size: u32, tile_overlap: u32) -> Self {
        ImagePageSource {
            bytes,
            tile_size,
            tile_overlap,
        }
    }
}
impl<F: Tgd> Tgj for ImagePageSource<F> {
    async fn load_tiles(&self, document_id: Uuid, page_no: i32) -> Result<Vec<Tile>, WorkerError> {
        let r_3dr = self.bytes.fetch(document_id, page_no).await?;
        let (tile_size, overlap) = (self.tile_size, self.tile_overlap);
        // Giải mã + cắt ô là việc CPU: chạy ở luồng blocking, không chặn các làn khác (mọi làn
        // chung một tác vụ tokio).
        tokio::task::spawn_blocking(move || {
            let r_3dq = f_13f(&r_3dr.bytes)?;
            let mut tiles = f_2l3(&r_3dq, tile_size, overlap);
            if let [t] = tiles.as_mut_slice() {
                if t.x == 0
                    && t.y == 0
                    && t.width == r_3dq.width()
                    && t.height == r_3dq.height()
                    && jpeg_passthrough_ok(&r_3dr.bytes)
                {
                    t.encoded = Some(Arc::new(r_3dr.bytes));
                }
            }
            Ok(tiles)
        })
        .await
        .map_err(|e| WorkerError::CorruptInput(format!("cắt ô trang {page_no}: {e}")))?
    }
    async fn load_page_image(&self, document_id: Uuid, page_no: i32) -> Result<Tgf, WorkerError> {
        let r_3ds = self.bytes.fetch(document_id, page_no).await?;
        let (width_px, height_px) = f_1hv(&r_3ds.bytes)?;
        Ok(Tgf {
            jpeg: r_3ds.bytes,
            width_px,
            height_px,
            dpi: r_3ds.dpi,
        })
    }
}
pub struct LocalDirPageBytes {
    dir: PathBuf,
}
impl LocalDirPageBytes {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        LocalDirPageBytes { dir: dir.into() }
    }
}
impl Tgd for LocalDirPageBytes {
    async fn fetch(&self, document_id: Uuid, page_no: i32) -> Result<FetchedPage, WorkerError> {
        let path = self
            .dir
            .join(document_id.to_string())
            .join(format!("{page_no}.jpg"));
        match tokio::fs::read(&path).await {
            Ok(bytes) => Ok(FetchedPage::bytes_only(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(WorkerError::StorageUnavailable(format!(
                    "chưa thấy ảnh trang {path:?}"
                )))
            }
            Err(e) => Err(WorkerError::StorageUnavailable(format!(
                "đọc {path:?}: {e}"
            ))),
        }
    }
}
pub struct MinioPageBytes {
    store: Arc<dyn ObjectStore>,
}
impl MinioPageBytes {
    pub fn new(store: Arc<dyn ObjectStore>) -> Self {
        MinioPageBytes { store }
    }
}
impl Tgd for MinioPageBytes {
    async fn fetch(&self, document_id: Uuid, page_no: i32) -> Result<FetchedPage, WorkerError> {
        let key = f_1xr(document_id, page_no);
        match self.store.get(&key).await {
            Ok(Some(bytes)) => Ok(FetchedPage::bytes_only(bytes)),
            Ok(None) => Err(WorkerError::StorageUnavailable(format!(
                "chưa thấy ảnh trang '{key}' trong kho"
            ))),
            Err(e) => Err(WorkerError::StorageUnavailable(format!(
                "fetch ảnh trang '{key}': {e}"
            ))),
        }
    }
}
pub struct PdfMinioPageBytes {
    store: Arc<dyn ObjectStore>,
    pool: PgPool,
    raster: Tid,
    cache: SourcePdfCache,
}
/// PDF nguồn đã tải + đã đọc, dùng chung cho mọi trang của một tài liệu.
struct SourcePdf {
    bytes: Vec<u8>,
    parsed: Result<digitext_pdf::ParsedPdf, String>,
}
type SourceCell = Arc<tokio::sync::OnceCell<Arc<SourcePdf>>>;
/// Bộ đệm PDF nguồn theo (storage_path, blake3). Trước đây MỖI TRANG tải nguyên tệp PDF từ
/// MinIO rồi `lopdf` đọc lại cả tệp — tài liệu N trang tốn N lần tải + N lần đọc khi OCR, thêm
/// N lần nữa khi ghép PDF/A. Các làn cùng xin một tài liệu thì chờ chung một lần tải
/// (OnceCell). Có trần theo dung lượng (DIGITEXT_PDF_CACHE_MB, mặc định 512; 0 = tắt): bỏ
/// tài liệu cũ nhất khi vượt; tệp lớn hơn trần không giữ lại.
struct SourcePdfCache {
    budget: usize,
    entries: std::sync::Mutex<std::collections::VecDeque<(String, SourceCell)>>,
}
impl SourcePdfCache {
    fn from_env() -> Self {
        let mb = std::env::var("DIGITEXT_PDF_CACHE_MB")
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(512);
        SourcePdfCache {
            budget: mb.saturating_mul(1024 * 1024),
            entries: Default::default(),
        }
    }
    fn cell(&self, key: &str) -> SourceCell {
        let mut q = self.entries.lock().unwrap();
        if let Some(pos) = q.iter().position(|(k, _)| k == key) {
            let e = q.remove(pos).unwrap();
            let cell = e.1.clone();
            q.push_back(e);
            return cell;
        }
        let cell: SourceCell = Default::default();
        q.push_back((key.to_string(), cell.clone()));
        cell
    }
    /// Bỏ tài liệu cũ nhất tới khi tổng dung lượng (bytes PDF ×2 — bản đọc sẵn giữ lại các
    /// luồng ảnh) nằm dưới trần. Ô chưa tải xong không tính và không bị bỏ.
    fn trim(&self) {
        let mut q = self.entries.lock().unwrap();
        let size = |c: &SourceCell| c.get().map_or(0, |p| p.bytes.len() * 2);
        let mut total: usize = q.iter().map(|(_, c)| size(c)).sum();
        let mut i = 0;
        while total > self.budget && i < q.len() {
            if q[i].1.initialized() {
                total -= size(&q[i].1);
                q.remove(i);
            } else {
                i += 1;
            }
        }
    }
    fn forget(&self, key: &str) {
        self.entries.lock().unwrap().retain(|(k, _)| k != key);
    }
}
#[derive(Debug, Clone)]
pub struct Tid {
    pub dpi: u32,
    pub pdftoppm_bin: String,
}
impl PdfMinioPageBytes {
    pub fn new(store: Arc<dyn ObjectStore>, pool: PgPool, raster: Tid) -> Self {
        PdfMinioPageBytes {
            store,
            pool,
            raster,
            cache: SourcePdfCache::from_env(),
        }
    }
}
pub async fn f_1xk(
    pdf: &[u8],
    page_no: i32,
    key: &str,
    raster: &Tid,
) -> Result<FetchedPage, WorkerError> {
    f_1xk_from(digitext_pdf::f_1b5(pdf, page_no), pdf, page_no, key, raster).await
}
async fn f_1xk_from(
    r_3e1: Result<(Vec<u8>, Option<u32>), digitext_pdf::PdfError>,
    pdf: &[u8],
    page_no: i32,
    key: &str,
    raster: &Tid,
) -> Result<FetchedPage, WorkerError> {
    match r_3e1 {
        Ok((bytes, dpi)) => Ok(FetchedPage { bytes, dpi }),
        Err(digitext_pdf::PdfError::PageNotJpeg { .. }) => {
            let bytes = f_25k(pdf, page_no, raster).await?;
            tracing::info!(
                key, page = page_no, dpi = raster.dpi, bytes = bytes.len(),
                "trang không có JPEG nhúng (CCITT/JBIG2/vector) — đã raster hoá"
            );
            Ok(FetchedPage {
                bytes,
                dpi: Some(raster.dpi),
            })
        }
        Err(e) => Err(WorkerError::CorruptInput(format!(
            "tách trang {page_no} từ '{key}': {e}"
        ))),
    }
}
pub async fn f_25k(pdf: &[u8], page_no: i32, r_3dt: &Tid) -> Result<Vec<u8>, WorkerError> {
    use tokio::io::AsyncWriteExt;
    let mut r_3du = tokio::process::Command::new(&r_3dt.pdftoppm_bin)
        .args([
            "-f",
            &page_no.to_string(),
            "-l",
            &page_no.to_string(),
            "-r",
            &r_3dt.dpi.to_string(),
            "-jpeg",
            "-singlefile",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            WorkerError::StorageUnavailable(
                format!(
                    "không chạy được '{}' để raster hoá trang (cài poppler-utils hoặc đặt {}): {e}",
                    r_3dt.pdftoppm_bin, crate ::config::C_63,
                ),
            )
        })?;
    let mut stdin = r_3du.stdin.take().expect("đã .stdin(piped)");
    let r_3dx = pdf.to_vec();
    let r_3dy = tokio::spawn(async move {
        let _ = stdin.write_all(&r_3dx).await;
        let _ = stdin.shutdown().await;
    });
    let r_3dw = r_3du.wait_with_output().await.map_err(|e| {
        WorkerError::StorageUnavailable(format!("chờ '{}': {e}", r_3dt.pdftoppm_bin))
    })?;
    let _ = r_3dy.await;
    if !r_3dw.status.success() {
        let r_3dv = String::from_utf8_lossy(&r_3dw.stderr);
        return Err(WorkerError::CorruptInput(format!(
            "raster hoá trang {page_no} thất bại ({}): {}",
            r_3dw.status,
            r_3dv.trim()
        )));
    }
    if r_3dw.stdout.is_empty() {
        return Err(WorkerError::CorruptInput(format!(
            "raster hoá trang {page_no} ra 0 byte (trang không tồn tại?)"
        )));
    }
    Ok(r_3dw.stdout)
}
impl PdfMinioPageBytes {
    /// `keys`: PDF gốc, rồi các bản thay thế khi PDF gốc đã bị xoá sau PDF/A (migration 0016) — lấy
    /// bản đầu tiên còn trong kho. Bản PDF/A nhúng nguyên ảnh JPEG gốc nên ảnh trang y như cũ.
    async fn source(&self, keys: &[String], tag: &str) -> Result<Arc<SourcePdf>, WorkerError> {
        let key = keys.first().map(String::as_str).unwrap_or_default();
        let cache_key = format!("{key}#{tag}");
        let load = || async {
            let mut found = None;
            for k in keys {
                match self.store.get(k).await {
                    Ok(Some(bytes)) => {
                        found = Some(bytes);
                        break;
                    }
                    Ok(None) => continue,
                    Err(e) => {
                        return Err(WorkerError::StorageUnavailable(format!(
                            "fetch PDF '{k}': {e}"
                        )));
                    }
                }
            }
            let Some(pdf) = found else {
                return Err(WorkerError::StorageUnavailable(format!(
                    "chưa thấy PDF '{key}' trong kho (cả bản PDF/A thay thế)"
                )));
            };
            tokio::task::spawn_blocking(move || {
                let parsed = digitext_pdf::ParsedPdf::parse(&pdf).map_err(|e| e.to_string());
                Arc::new(SourcePdf { bytes: pdf, parsed })
            })
            .await
            .map_err(|e| WorkerError::CorruptInput(format!("đọc PDF '{key}': {e}")))
        };
        if self.cache.budget == 0 {
            return load().await;
        }
        let cell = self.cache.cell(&cache_key);
        let src = cell.get_or_try_init(load).await.cloned();
        match &src {
            Ok(p) if p.bytes.len() * 2 > self.cache.budget => self.cache.forget(&cache_key),
            Ok(_) => self.cache.trim(),
            Err(_) => self.cache.forget(&cache_key),
        }
        src
    }
}
impl Tgd for PdfMinioPageBytes {
    async fn fetch(&self, document_id: Uuid, page_no: i32) -> Result<FetchedPage, WorkerError> {
        let r_3dz = digitext_db::documents::get(&self.pool, document_id)
            .await
            .map_err(|e| WorkerError::StorageUnavailable(format!("tra document: {e}")))?
            .ok_or_else(|| {
                WorkerError::CorruptInput(format!("không thấy document {document_id}"))
            })?;
        let key = r_3dz.storage_path.clone();
        // PDF/A trước bản đã ký: bản ký có thể thêm ảnh con dấu chữ ký vào trang, làm sai "ảnh JPEG
        // đầu tiên của trang".
        let keys = [
            key.clone(),
            digitext_storage::f_2qs(r_3dz.project_id, document_id),
            digitext_storage::f_2fc(r_3dz.project_id, document_id),
        ];
        let src = self.source(&keys, &r_3dz.blake3).await?;
        let r_3e0 = src.clone();
        let jpeg = tokio::task::spawn_blocking(move || match &r_3e0.parsed {
            Ok(parsed) => digitext_pdf::page_jpeg(parsed, page_no),
            Err(e) => Err(digitext_pdf::PdfError::NoPageImage(e.clone())),
        })
        .await
        .map_err(|e| WorkerError::CorruptInput(format!("tách trang {page_no}: {e}")))?;
        f_1xk_from(jpeg, &src.bytes, page_no, &key, &self.raster).await
    }
}
pub enum PageBytesSource {
    Local(LocalDirPageBytes),
    Minio(MinioPageBytes),
    Pdf(PdfMinioPageBytes),
}
impl Tgd for PageBytesSource {
    async fn fetch(&self, document_id: Uuid, page_no: i32) -> Result<FetchedPage, WorkerError> {
        match self {
            PageBytesSource::Local(s) => s.fetch(document_id, page_no).await,
            PageBytesSource::Minio(s) => s.fetch(document_id, page_no).await,
            PageBytesSource::Pdf(s) => s.fetch(document_id, page_no).await,
        }
    }
}
#[cfg(test)]
mod passthrough_tests {
    use super::jpeg_passthrough_ok;
    fn jpeg(rgb: bool) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(8, 8, image::Rgb([200, 10, 10]));
        let mut out = Vec::new();
        if rgb {
            image::codecs::jpeg::JpegEncoder::new(&mut out).encode_image(&img).unwrap();
        } else {
            let g = image::DynamicImage::ImageRgb8(img).to_luma8();
            image::codecs::jpeg::JpegEncoder::new(&mut out).encode_image(&g).unwrap();
        }
        out
    }
    #[test]
    fn jpeg_rgb_xam_duoc_gui_thang() {
        assert!(jpeg_passthrough_ok(&jpeg(true)));
        assert!(jpeg_passthrough_ok(&jpeg(false)));
    }
    #[test]
    fn exif_png_rac_khong_gui_thang() {
        let mut j = jpeg(true);
        let exif: &[u8] = &[0xFF, 0xE1, 0x00, 0x08, b'E', b'x', b'i', b'f', 0, 0];
        j.splice(2..2, exif.iter().copied());
        assert!(!jpeg_passthrough_ok(&j));
        assert!(!jpeg_passthrough_ok(&[0x89, b'P', b'N', b'G', 13, 10, 26, 10]));
        assert!(!jpeg_passthrough_ok(&[0xFF, 0xD8]));
    }
}
