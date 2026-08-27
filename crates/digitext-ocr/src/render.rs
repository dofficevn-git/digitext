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
                "xJFvw6FuIMSR4buLbmggZOG6oW5nIOG6o25oIHRyYW5nIGzhu5dpOiA={e}"
            ))
        })?
        .into_dimensions()
        .map_err(|e| {
            WorkerError::CorruptInput(format!(
                "xJHhu41jIGvDrWNoIHRoxrDhu5tjIOG6o25oIHRyYW5nIGzhu5dpOiA={e}"
            ))
        })
}
pub fn f_13f(bytes: &[u8]) -> Result<RgbImage, WorkerError> {
    let r_3di = image::load_from_memory(bytes).map_err(|e| {
        WorkerError::CorruptInput(format!("Z2nhuqNpIG3DoyDhuqNuaCB0cmFuZyBs4buXaTog{e}"))
    })?;
    Ok(r_3di.to_rgb8())
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
        let r_3dq = f_13f(&r_3dr.bytes)?;
        Ok(f_2l3(&r_3dq, self.tile_size, self.tile_overlap))
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
                    "Y2jGsGEgdGjhuqV5IOG6o25oIHRyYW5nIA=={path:?}"
                )))
            }
            Err(e) => Err(WorkerError::StorageUnavailable(format!(
                "xJHhu41jIA=={path:?}: {e}"
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
                "Y2jGsGEgdGjhuqV5IOG6o25oIHRyYW5nICc={key}' trong kho"
            ))),
            Err(e) => Err(WorkerError::StorageUnavailable(format!(
                "ZmV0Y2gg4bqjbmggdHJhbmcgJw=={key}': {e}"
            ))),
        }
    }
}
pub struct PdfMinioPageBytes {
    store: Arc<dyn ObjectStore>,
    pool: PgPool,
    raster: Tid,
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
        }
    }
}
pub async fn f_1xk(
    pdf: &[u8],
    page_no: i32,
    key: &str,
    raster: &Tid,
) -> Result<FetchedPage, WorkerError> {
    match digitext_pdf::f_1b5(pdf, page_no) {
        Ok((bytes, dpi)) => Ok(FetchedPage { bytes, dpi }),
        Err(digitext_pdf::PdfError::PageNotJpeg { .. }) => {
            let bytes = f_25k(pdf, page_no, raster).await?;
            tracing::info!(
                key, page = page_no, dpi = raster.dpi, bytes = bytes.len(),
                "dHJhbmcga2jDtG5nIGPDsyBKUEVHIG5ow7puZyAoQ0NJVFQvSkJJRzIvdmVjdG9yKSDigJQgxJHDoyByYXN0ZXIgaG/DoQ=="
            );
            Ok(FetchedPage {
                bytes,
                dpi: Some(raster.dpi),
            })
        }
        Err(e) => Err(WorkerError::CorruptInput(format!(
            "dMOhY2ggdHJhbmcg{page_no}IHThu6sgJw=={key}': {e}"
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
                    "a2jDtG5nIGNo4bqheSDEkcaw4bujYyAn{}JyDEkeG7gyByYXN0ZXIgaG/DoSB0cmFuZyAoY8OgaSBwb3BwbGVyLXV0aWxzIGhv4bq3YyDEkeG6t3Qg{}): {e}",
                    r_3dt.pdftoppm_bin, crate ::config::C_63,
                ),
            )
        })?;
    let mut stdin = r_3du.stdin.take().expect("xJHDoyAuc3RkaW4ocGlwZWQp");
    let r_3dx = pdf.to_vec();
    let r_3dy = tokio::spawn(async move {
        let _ = stdin.write_all(&r_3dx).await;
        let _ = stdin.shutdown().await;
    });
    let r_3dw = r_3du.wait_with_output().await.map_err(|e| {
        WorkerError::StorageUnavailable(format!("Y2jhu50gJw=={}': {e}", r_3dt.pdftoppm_bin))
    })?;
    let _ = r_3dy.await;
    if !r_3dw.status.success() {
        let r_3dv = String::from_utf8_lossy(&r_3dw.stderr);
        return Err(WorkerError::CorruptInput(format!(
            "cmFzdGVyIGhvw6EgdHJhbmcg{page_no}IHRo4bqldCBi4bqhaSAo{}): {}",
            r_3dw.status,
            r_3dv.trim()
        )));
    }
    if r_3dw.stdout.is_empty() {
        return Err(WorkerError::CorruptInput(format!(
            "cmFzdGVyIGhvw6EgdHJhbmcg{page_no}IHJhIDAgYnl0ZSAodHJhbmcga2jDtG5nIHThu5NuIHThuqFpPyk="
        )));
    }
    Ok(r_3dw.stdout)
}
impl Tgd for PdfMinioPageBytes {
    async fn fetch(&self, document_id: Uuid, page_no: i32) -> Result<FetchedPage, WorkerError> {
        let r_3dz = digitext_db::documents::get(&self.pool, document_id)
            .await
            .map_err(|e| WorkerError::StorageUnavailable(format!("tra document: {e}")))?
            .ok_or_else(|| {
                WorkerError::CorruptInput(format!("a2jDtG5nIHRo4bqleSBkb2N1bWVudCA={document_id}"))
            })?;
        let key = &r_3dz.storage_path;
        let pdf = match self.store.get(key).await {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                return Err(WorkerError::StorageUnavailable(format!(
                    "Y2jGsGEgdGjhuqV5IFBERiAn{key}' trong kho"
                )));
            }
            Err(e) => {
                return Err(WorkerError::StorageUnavailable(format!(
                    "fetch PDF '{key}': {e}"
                )));
            }
        };
        f_1xk(&pdf, page_no, key, &self.raster).await
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
