// Copyright (c) 2026 DigiText. All rights reserved.
#[cfg(feature = "mock-scanner")]
mod mock;
pub mod paper;
#[cfg(all(
    any(target_os = "linux", target_os = "macos"),
    not(feature = "mock-scanner")
))]
mod sane;
#[cfg(all(windows, not(feature = "mock-scanner"), feature = "twain"))]
mod twain;
#[cfg(all(windows, not(feature = "mock-scanner"), not(feature = "twain")))]
mod wia;
use base64::Engine as _;
use image::RgbImage;
use paper::f_150;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tkw {
    pub id: String,
    pub name: String,
    pub model: String,
    pub connected: bool,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    Rgb,
    Gray,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaperSource {
    Adf,
    Glass,
}
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tko {
    #[serde(default)]
    pub device_id: Option<String>,
    pub dpi: u32,
    pub color: ColorMode,
    pub source: PaperSource,
    #[serde(default)]
    pub duplex: bool,
    #[serde(default = "default_start_index")]
    pub start_index: u32,
    #[serde(default)]
    pub max_pages: Option<u32>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub enhance: crate::storage::EnhanceFlags,
}
fn default_start_index() -> u32 {
    1
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tkt {
    pub index: u32,
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: u32,
    pub width_mm: f32,
    pub height_mm: f32,
    pub paper: String,
    pub paper_known: bool,
    pub dpi_ok: bool,
    pub thumbnail: String,
    pub preview: String,
    pub source_path: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum BatchState {
    Starting,
    Capturing {
        device: String,
    },
    #[allow(dead_code)]
    Disconnected {
        at_page: u32,
    },
    Completed {
        pages: u32,
    },
    Cancelled {
        pages: u32,
    },
}
#[derive(Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);
impl CancelToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
pub trait Tkr: Send {
    fn state(&self, batch_id: &str, state: BatchState);
    fn page(&self, batch_id: &str, page: Tkt);
    fn error(&self, batch_id: &str, message: &str);
}
pub trait Tkv: Send + Sync {
    fn list_devices(&self) -> Result<Vec<Tkw>, String>;
    fn scan_batch(&self, batch_id: &str, opts: &Tko, sink: &dyn Tkr, cancel: &CancelToken);
}
fn backend() -> Box<dyn Tkv> {
    #[cfg(feature = "mock-scanner")]
    {
        return Box::new(mock::Tdi::new());
    }
    #[cfg(all(windows, not(feature = "mock-scanner"), feature = "twain"))]
    {
        return Box::new(twain::Tov::new());
    }
    #[cfg(all(windows, not(feature = "mock-scanner"), not(feature = "twain")))]
    {
        return Box::new(wia::Tq1::new());
    }
    #[cfg(all(
        any(target_os = "linux", target_os = "macos"),
        not(feature = "mock-scanner")
    ))]
    {
        return Box::new(sane::Tki::new());
    }
    #[allow(unreachable_code)]
    {
        Box::new(Ted)
    }
}
struct Ted;
impl Tkv for Ted {
    fn list_devices(&self) -> Result<Vec<Tkw>, String> {
        Ok(Vec::new())
    }
    fn scan_batch(&self, r_2da: &str, _opts: &Tko, sink: &dyn Tkr, _cancel: &CancelToken) {
        sink.error(
            r_2da,
            "Nền tảng này không hỗ trợ quét tài liệu (chỉ có trên Windows/macOS/Linux).",
        );
        sink.state(r_2da, BatchState::Cancelled { pages: 0 });
    }
}
pub(crate) fn f_19b(r_2dc: &RgbImage) -> String {
    let mut r_2db = Vec::new();
    if r_2dc
        .write_to(&mut Cursor::new(&mut r_2db), image::ImageFormat::Png)
        .is_err()
    {
        return String::new();
    }
    let b64 = base64::engine::general_purpose::STANDARD.encode(&r_2db);
    format!("data:image/png;base64,{b64}")
}
pub(crate) const C_G1: u32 = 1500;
pub(crate) fn f_199(r_2de: &RgbImage) -> String {
    let mut r_2dd = Vec::new();
    if r_2de
        .write_to(&mut Cursor::new(&mut r_2dd), image::ImageFormat::Jpeg)
        .is_err()
    {
        return String::new();
    }
    let b64 = base64::engine::general_purpose::STANDARD.encode(&r_2dd);
    format!("data:image/jpeg;base64,{b64}")
}
fn f_1xl() -> std::path::PathBuf {
    std::env::temp_dir().join("digitext-pages")
}
pub(crate) fn f_2bw(r_2dh: &image::DynamicImage, r_2df: &str, index: u32) -> String {
    let r_2dg = f_1xl().join(r_2df);
    if std::fs::create_dir_all(&r_2dg).is_err() {
        return String::new();
    }
    let r_2di = r_2dg.join(format!("page-{index:05}.png"));
    if r_2dh.save(&r_2di).is_err() {
        return String::new();
    }
    r_2di.to_string_lossy().into_owned()
}
pub(crate) fn f_2bx(r_2dm: &image::DynamicImage, r_2dj: &str, index: u32) -> String {
    let r_2dl = f_1xl().join(r_2dj);
    if std::fs::create_dir_all(&r_2dl).is_err() {
        return String::new();
    }
    let r_2dn = r_2dl.join(format!("page-{index:05}.jpg"));
    match crate::storage::f_198(r_2dm, 92) {
        Ok((r_2dk, _, _)) if std::fs::write(&r_2dn, &r_2dk).is_ok() => {
            r_2dn.to_string_lossy().into_owned()
        }
        _ => String::new(),
    }
}
pub fn f_2ib() {
    let _ = std::fs::remove_dir_all(f_1xl());
}
#[derive(Default)]
pub struct Tkx {
    batches: Arc<Mutex<HashMap<String, CancelToken>>>,
}
impl Tkx {
    pub fn has_active_batches(&self) -> bool {
        self.batches
            .lock()
            .map(|r_2do| !r_2do.is_empty())
            .unwrap_or(false)
    }
}
struct Tnw {
    app: AppHandle,
}
impl Tkr for Tnw {
    fn state(&self, r_2dp: &str, state: BatchState) {
        let _ = self.app.emit(
            "scan://state",
            serde_json::json!({ "batchId" : r_2dp, "state" : state }),
        );
    }
    fn page(&self, r_2dq: &str, page: Tkt) {
        let _ = self.app.emit(
            "scan://page",
            serde_json::json!({ "batchId" : r_2dq, "page" : page }),
        );
    }
    fn error(&self, r_2dr: &str, message: &str) {
        let _ = self.app.emit(
            "scan://error",
            serde_json::json!({ "batchId" : r_2dr, "message" : message }),
        );
    }
}
const C_5C: usize = 3;
struct T6u {
    index: u32,
    source: std::path::PathBuf,
}
pub(crate) struct EnhancePool {
    tx: Mutex<Option<std::sync::mpsc::Sender<T6u>>>,
    handles: Mutex<Vec<std::thread::JoinHandle<()>>>,
    queued: Arc<AtomicU64>,
}
impl EnhancePool {
    pub(crate) fn new<F>(r_2dt: crate::storage::EnhanceFlags, on_done: F) -> Self
    where
        F: Fn(u32, Option<Thm>, u64, u64) + Send + Sync + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel::<T6u>();
        let rx = Arc::new(Mutex::new(rx));
        let queued = Arc::new(AtomicU64::new(0));
        let done = Arc::new(AtomicU64::new(0));
        let on_done = Arc::new(on_done);
        let r_2e2 = std::thread::available_parallelism()
            .map(|r_2e1| r_2e1.get().saturating_sub(1).max(1))
            .unwrap_or(2)
            .min(C_5C);
        let mut handles = Vec::with_capacity(r_2e2);
        for _ in 0..r_2e2 {
            let rx = rx.clone();
            let queued = queued.clone();
            let done = done.clone();
            let on_done = on_done.clone();
            let r_2dv = std::thread::Builder::new()
                .name("scan-enhance".into())
                .spawn(move || loop {
                    let r_2dx = match rx.lock() {
                        Ok(r_2du) => match r_2du.recv() {
                            Ok(r_2dw) => r_2dw,
                            Err(_) => break,
                        },
                        Err(_) => break,
                    };
                    let r_2e0 = std::time::Instant::now();
                    let r_2dz = match f_19g(&r_2dx.source, r_2dt) {
                        Ok(r_2dy) => {
                            tracing::info!(
                                page = r_2dx.index,
                                ms = r_2e0.elapsed().as_millis() as u64,
                                "làm sạch trang xong"
                            );
                            Some(r_2dy)
                        }
                        Err(e) => {
                            tracing::warn!(
                                page = r_2dx.index, error = % e,
                                "làm sạch trang LỖI"
                            );
                            None
                        }
                    };
                    let r_2ds = done.fetch_add(1, Ordering::SeqCst) + 1;
                    on_done(r_2dx.index, r_2dz, r_2ds, queued.load(Ordering::SeqCst));
                });
            match r_2dv {
                Ok(h) => handles.push(h),
                Err(e) => {
                    tracing::warn!(
                        error = % e,
                        "không tạo được luồng làm sạch"
                    );
                    break;
                }
            }
        }
        Self {
            tx: Mutex::new(Some(tx)),
            handles: Mutex::new(handles),
            queued,
        }
    }
    pub(crate) fn submit(&self, index: u32, source: std::path::PathBuf) {
        tracing::debug!(
            page = index, path = % source.display(),
            "xếp trang vào hàng làm sạch"
        );
        self.queued.fetch_add(1, Ordering::SeqCst);
        if let Ok(r_2e3) = self.tx.lock() {
            if let Some(tx) = r_2e3.as_ref() {
                let _ = tx.send(T6u { index, source });
            }
        }
    }
    pub(crate) fn finish(&self) {
        if let Ok(mut r_2e4) = self.tx.lock() {
            r_2e4.take();
        }
        let handles = self
            .handles
            .lock()
            .map(|mut h| std::mem::take(&mut *h))
            .unwrap_or_default();
        for h in handles {
            let _ = h.join();
        }
    }
}
fn f_19g(source: &std::path::Path, r_2e5: crate::storage::EnhanceFlags) -> Result<Thm, String> {
    if !r_2e5.any() {
        return Err("không có bước làm sạch nào được bật".into());
    }
    let lock = f_1xt(source);
    let _held = lock
        .lock()
        .map_err(|_| "khoá trang hỏng".to_string())?;
    let work = f_2rc(source);
    let r_2e7 = if work.exists() { &work } else { source };
    let r_2e6 = image::open(r_2e7)
        .map_err(|e| format!("không mở được {}: {e}", r_2e7.display()))?;
    let r_2e8 = crate::storage::f_t1(&r_2e6, r_2e5);
    f_2by(&r_2e8, &work)?;
    f_24u(source, &work);
    Ok(f_29l(&r_2e8, source, 0))
}
struct EnhancingSink<'a, S: Tkr + Sync> {
    inner: &'a S,
    pool: &'a EnhancePool,
    pending_terminal: Mutex<Option<BatchState>>,
}
impl<'a, S: Tkr + Sync> EnhancingSink<'a, S> {
    fn new(inner: &'a S, pool: &'a EnhancePool) -> Self {
        Self {
            inner,
            pool,
            pending_terminal: Mutex::new(None),
        }
    }
    fn take_terminal(&self) -> Option<BatchState> {
        self.pending_terminal
            .lock()
            .ok()
            .and_then(|mut r_2e9| r_2e9.take())
    }
}
impl<S: Tkr + Sync> Tkr for EnhancingSink<'_, S> {
    fn state(&self, r_2ea: &str, state: BatchState) {
        if matches!(
            state,
            BatchState::Completed { .. } | BatchState::Cancelled { .. }
        ) {
            if let Ok(mut r_2eb) = self.pending_terminal.lock() {
                *r_2eb = Some(state);
                return;
            }
        }
        self.inner.state(r_2ea, state);
    }
    fn page(&self, r_2ec: &str, page: Tkt) {
        let (index, source) = (page.index, page.source_path.clone());
        self.inner.page(r_2ec, page);
        if !source.is_empty() {
            self.pool.submit(index, std::path::PathBuf::from(source));
        }
    }
    fn error(&self, r_2ed: &str, message: &str) {
        self.inner.error(r_2ed, message);
    }
}
static C_W: AtomicU64 = AtomicU64::new(1);
fn f_1u1() -> String {
    format!("batch-{}", C_W.fetch_add(1, Ordering::Relaxed))
}
pub(crate) fn f_1dq(prefix: &str) -> String {
    format!("{prefix}-{}", C_W.fetch_add(1, Ordering::Relaxed))
}
#[tauri::command]
pub async fn list_scanners(auth: State<'_, crate::auth::AuthState>) -> Result<Vec<Tkw>, String> {
    auth.require_perm_str(digitext_core::Permission::ScanView)?;
    tauri::async_runtime::spawn_blocking(|| backend().list_devices())
        .await
        .map_err(|e| format!("luồng liệt kê máy quét lỗi: {e}"))?
}
struct T17 {
    batches: Arc<Mutex<HashMap<String, CancelToken>>>,
    id: String,
}
impl Drop for T17 {
    fn drop(&mut self) {
        if let Ok(mut r_2ee) = self.batches.lock() {
            r_2ee.remove(&self.id);
        }
    }
}
#[tauri::command]
pub fn start_scan_batch(
    app: AppHandle,
    auth: State<'_, crate::auth::AuthState>,
    state: State<'_, Tkx>,
    opts: Tko,
) -> Result<String, String> {
    auth.require_perm_str(digitext_core::Permission::ScanExecute)?;
    let r_2ef = f_1u1();
    let cancel = CancelToken::default();
    {
        let mut map = state.batches.lock().map_err(|_| "scanner state poisoned")?;
        if !map.is_empty() {
            return Err(
                "Đang có một lô quét chạy — kết thúc lô hiện tại trước khi bắt đầu lô mới."
                    .into(),
            );
        }
        map.insert(r_2ef.clone(), cancel.clone());
    }
    let batches = state.batches.clone();
    let id = r_2ef.clone();
    let r_2ej = std::thread::Builder::new()
        .name("scan-batch".into())
        .spawn(move || {
            let _guard = T17 {
                batches,
                id: id.clone(),
            };
            let sink = Tnw { app: app.clone() };
            let r_2eh = opts.enhance;
            tracing::info!(batch = % id, ? r_2eh, "bắt đầu lô quét");
            if !r_2eh.any() {
                tracing::info!(
                    batch = % id,
                    "làm sạch tự động TẮT cho lô này"
                );
                backend().scan_batch(&id, &opts, &sink, &cancel);
                return;
            }
            let r_2eg = id.clone();
            let pool = EnhancePool::new(r_2eh, move |index, result, done, total| {
                let r_2ei = app.emit(
                    "scan://enhanced",
                    serde_json::json!(
                        { "batchId" : r_2eg, "index" : index, "result" : result,
                        "done" : done, "total" : total, }
                    ),
                );
                match r_2ei {
                    Ok(()) => {
                        tracing::info!(
                            page = index,
                            done,
                            total,
                            "đã phát ảnh đã làm sạch"
                        )
                    }
                    Err(e) => {
                        tracing::warn!(
                            page = index, error = % e,
                            "PHÁT event làm sạch LỖI"
                        )
                    }
                }
            });
            let r_2em = EnhancingSink::new(&sink, &pool);
            backend().scan_batch(&id, &opts, &r_2em, &cancel);
            let r_2el = std::time::Instant::now();
            pool.finish();
            tracing::info!(
                batch = % id, pages = pool.queued.load(Ordering::SeqCst), drain_ms =
                r_2el.elapsed().as_millis() as u64,
                "hàng đợi làm sạch đã cạn"
            );
            if let Some(r_2ek) = r_2em.take_terminal() {
                sink.state(&id, r_2ek);
            }
        });
    if let Err(e) = r_2ej {
        if let Ok(mut map) = state.batches.lock() {
            map.remove(&r_2ef);
        }
        return Err(e.to_string());
    }
    Ok(r_2ef)
}
#[tauri::command]
pub fn cancel_scan_batch(
    auth: State<'_, crate::auth::AuthState>,
    state: State<'_, Tkx>,
    batch_id: String,
) -> Result<(), String> {
    auth.require_perm_str(digitext_core::Permission::ScanExecute)?;
    if let Some(r_2en) = state
        .batches
        .lock()
        .map_err(|_| "scanner state poisoned")?
        .get(&batch_id)
    {
        r_2en.cancel();
    }
    Ok(())
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thl {
    #[serde(default)]
    pub rotate_quarters: i32,
    #[serde(default)]
    pub brightness: i32,
    #[serde(default)]
    pub contrast: f32,
    #[serde(default = "default_gamma")]
    pub gamma: f32,
    #[serde(default = "default_color_mode")]
    pub color_mode: String,
    #[serde(default = "default_threshold")]
    pub bw_threshold: u8,
    #[serde(default)]
    pub flip_h: bool,
    #[serde(default)]
    pub flip_v: bool,
    #[serde(default)]
    pub deskew_deg: f32,
    #[serde(default)]
    pub crop: Option<T3l>,
    #[serde(default)]
    pub auto_crop: bool,
    #[serde(default)]
    pub auto_deskew: bool,
    #[serde(default)]
    pub align: Option<String>,
    #[serde(default)]
    pub denoise: bool,
    #[serde(default)]
    pub despeckle: bool,
    #[serde(default)]
    pub sharpen: bool,
    #[serde(default)]
    pub punch_holes: bool,
    #[serde(default)]
    pub auto_tone: bool,
    #[serde(default)]
    pub source_dpi: u32,
    #[serde(default)]
    pub target_dpi: u32,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3l {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
fn default_gamma() -> f32 {
    1.0
}
fn default_color_mode() -> String {
    "color".into()
}
fn default_threshold() -> u8 {
    128
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Thm {
    pub preview: String,
    pub thumbnail: String,
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: u32,
    pub can_undo: bool,
    pub can_redo: bool,
}
fn f_2rc(source: &std::path::Path) -> std::path::PathBuf {
    source.with_extension("work.png")
}
static C_FQ: Mutex<Option<HashMap<std::path::PathBuf, Arc<Mutex<()>>>>> = Mutex::new(None);
fn f_2by(r_2ep: &image::DynamicImage, work: &std::path::Path) -> Result<(), String> {
    let mut r_2eo = Vec::new();
    r_2ep
        .write_to(&mut Cursor::new(&mut r_2eo), image::ImageFormat::Png)
        .map_err(|e| format!("mã hoá ảnh lỗi: {e}"))?;
    crate::storage::f_2rn(work, &r_2eo)
}
fn f_1xt(source: &std::path::Path) -> Arc<Mutex<()>> {
    let r_2eq = source
        .canonicalize()
        .unwrap_or_else(|_| source.to_path_buf());
    let mut g = match C_FQ.lock() {
        Ok(g) => g,
        Err(r_2er) => r_2er.into_inner(),
    };
    g.get_or_insert_with(HashMap::new)
        .entry(r_2eq)
        .or_default()
        .clone()
}
const HISTORY_MAX: i64 = 5;
fn f_1h2(source: &std::path::Path) -> std::path::PathBuf {
    source.with_extension("hist")
}
fn f_1h3(r_2es: &std::path::Path, n: i64) -> std::path::PathBuf {
    r_2es.join(format!("{n:05}.png"))
}
fn f_25p(r_2et: &std::path::Path) -> i64 {
    std::fs::read_to_string(r_2et.join("cursor"))
        .ok()
        .and_then(|r_2eu| r_2eu.trim().parse().ok())
        .unwrap_or(-1)
}
fn f_2rq(r_2ev: &std::path::Path, n: i64) {
    let _ = std::fs::write(r_2ev.join("cursor"), n.to_string());
}
fn f_24u(source: &std::path::Path, work: &std::path::Path) {
    let r_2ex = f_1h2(source);
    if std::fs::create_dir_all(&r_2ex).is_err() {
        return;
    }
    let r_2ew = f_25p(&r_2ex);
    let mut n = r_2ew + 1;
    while f_1h3(&r_2ex, n).exists() {
        let _ = std::fs::remove_file(f_1h3(&r_2ex, n));
        n += 1;
    }
    let r_2ey = r_2ew + 1;
    if std::fs::copy(work, f_1h3(&r_2ex, r_2ey)).is_err() {
        return;
    }
    f_2rq(&r_2ex, r_2ey);
    let mut r_2ez = r_2ey - HISTORY_MAX;
    while r_2ez >= 0 && f_1h3(&r_2ex, r_2ez).exists() {
        let _ = std::fs::remove_file(f_1h3(&r_2ex, r_2ez));
        r_2ez -= 1;
    }
}
fn can_undo(r_2f1: &std::path::Path) -> bool {
    let r_2f0 = f_25p(r_2f1);
    r_2f0 == 0 || (r_2f0 > 0 && f_1h3(r_2f1, r_2f0 - 1).exists())
}
fn can_redo(r_2f2: &std::path::Path) -> bool {
    f_1h3(r_2f2, f_25p(r_2f2) + 1).exists()
}
fn f_1h8(source: &std::path::Path, r_2f3: bool) -> Result<(), String> {
    let r_2f5 = f_1h2(source);
    let r_2f4 = f_25p(&r_2f5);
    let work = f_2rc(source);
    if r_2f3 {
        if r_2f4 < 0 {
            return Err("không còn bước nào để hoàn tác".into());
        }
        if r_2f4 == 0 {
            let _ = std::fs::remove_file(&work);
            f_2rq(&r_2f5, -1);
            return Ok(());
        }
        let r_2f7 = f_1h3(&r_2f5, r_2f4 - 1);
        if !r_2f7.exists() {
            return Err(
                format!(
                    "chỉ giữ được {HISTORY_MAX} bước gần nhất — không lùi thêm được"
                ),
            );
        }
        std::fs::copy(&r_2f7, &work).map_err(|e| {
            format!("không khôi phục được bước trước: {e}")
        })?;
        f_2rq(&r_2f5, r_2f4 - 1);
    } else {
        let r_2f6 = f_1h3(&r_2f5, r_2f4 + 1);
        if !r_2f6.exists() {
            return Err("không còn bước nào để làm lại".into());
        }
        std::fs::copy(&r_2f6, &work)
            .map_err(|e| format!("không khôi phục được bước sau: {e}"))?;
        f_2rq(&r_2f5, r_2f4 + 1);
    }
    Ok(())
}
fn f_19y(source: &str) -> Result<std::path::PathBuf, String> {
    let r_2fa = std::path::Path::new(source);
    let r_2f9 = f_1xl().canonicalize().unwrap_or_else(|_| f_1xl());
    let r_2f8 = r_2fa
        .canonicalize()
        .map_err(|_| "ảnh nguồn không tồn tại".to_string())?;
    if !r_2f8.starts_with(&r_2f9) {
        return Err("đường dẫn ảnh không hợp lệ".into());
    }
    Ok(r_2f8)
}
pub fn f_295(source_path: &str) -> Result<std::path::PathBuf, String> {
    let r_2fb = f_19y(source_path)?;
    let work = f_2rc(&r_2fb);
    Ok(if work.exists() { work } else { r_2fb })
}
fn f_29l(r_2fd: &image::DynamicImage, source: &std::path::Path, dpi: u32) -> Thm {
    let r_2fc = f_1h2(source);
    Thm {
        preview: f_199(&r_2fd.thumbnail(C_G1, C_G1).to_rgb8()),
        thumbnail: f_19b(&r_2fd.thumbnail(96, 136).to_rgb8()),
        width_px: r_2fd.width(),
        height_px: r_2fd.height(),
        dpi,
        can_undo: can_undo(&r_2fc),
        can_redo: can_redo(&r_2fc),
    }
}
#[tauri::command]
pub async fn process_scan_page(
    auth: State<'_, crate::auth::AuthState>,
    source_path: String,
    ops: Thl,
) -> Result<Thm, String> {
    auth.require_perm_str(digitext_core::Permission::ScanExecute)?;
    tauri::async_runtime::spawn_blocking(move || {
        let source = f_19y(&source_path)?;
        let lock = f_1xt(&source);
        let _held = lock.lock().map_err(|_| "khoá trang hỏng")?;
        let work = f_2rc(&source);
        let r_2fh = if work.exists() { &work } else { &source };
        let mut r_2fg = image::open(r_2fh)
            .map_err(|e| format!("không mở được ảnh: {e}"))?;
        if let Some(r_2fe) = ops.crop {
            r_2fg = crate::storage::f_12p(&r_2fg, r_2fe.x, r_2fe.y, r_2fe.w, r_2fe.h);
        }
        match ops.rotate_quarters.rem_euclid(4) {
            1 => r_2fg = r_2fg.rotate90(),
            2 => r_2fg = r_2fg.rotate180(),
            3 => r_2fg = r_2fg.rotate270(),
            _ => {}
        }
        if ops.flip_h {
            r_2fg = r_2fg.fliph();
        }
        if ops.flip_v {
            r_2fg = r_2fg.flipv();
        }
        if ops.deskew_deg.abs() > 0.01 {
            r_2fg = crate::storage::f_2ad(&r_2fg, ops.deskew_deg);
        }
        if ops.auto_crop {
            r_2fg = crate::storage::auto_crop(&r_2fg);
        }
        if ops.auto_deskew {
            r_2fg = crate::storage::auto_deskew(&r_2fg);
        }
        match ops.align.as_deref() {
            Some("center") => {
                r_2fg = crate::storage::f_231(&r_2fg, crate::storage::Placement::Center);
            }
            Some("left") => {
                r_2fg = crate::storage::f_231(&r_2fg, crate::storage::Placement::TopLeft);
            }
            _ => {}
        }
        if ops.punch_holes {
            r_2fg = crate::storage::f_277(&r_2fg);
        }
        if ops.despeckle {
            r_2fg = crate::storage::despeckle(&r_2fg);
        }
        if ops.denoise {
            r_2fg = crate::storage::denoise(&r_2fg);
        }
        if ops.sharpen {
            r_2fg = crate::storage::sharpen(&r_2fg);
        }
        if ops.auto_tone {
            r_2fg = crate::storage::auto_tone(&r_2fg);
        }
        if ops.brightness != 0 {
            r_2fg = r_2fg.brighten(ops.brightness);
        }
        if ops.contrast != 0.0 {
            r_2fg = r_2fg.adjust_contrast(ops.contrast);
        }
        if (ops.gamma - 1.0).abs() > 0.001 {
            let r_2fi = 1.0 / ops.gamma.max(0.01);
            let r_2fj: [u8; 256] = std::array::from_fn(|i| {
                ((i as f32 / 255.0).powf(r_2fi) * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8
            });
            let mut r_2fl = r_2fg.to_rgba8();
            for px in r_2fl.pixels_mut() {
                px[0] = r_2fj[px[0] as usize];
                px[1] = r_2fj[px[1] as usize];
                px[2] = r_2fj[px[2] as usize];
            }
            r_2fg = image::DynamicImage::ImageRgba8(r_2fl);
        }
        match ops.color_mode.as_str() {
            "gray" => r_2fg = image::DynamicImage::ImageLuma8(r_2fg.to_luma8()),
            "bw" => {
                let mut r_2ff = r_2fg.to_luma8();
                for px in r_2ff.pixels_mut() {
                    px[0] = if px[0] >= ops.bw_threshold { 255 } else { 0 };
                }
                r_2fg = image::DynamicImage::ImageLuma8(r_2ff);
            }
            _ => {}
        }
        let r_2fk = if ops.target_dpi > 0 && ops.source_dpi > 0 && ops.target_dpi != ops.source_dpi
        {
            r_2fg = crate::storage::f_28l(&r_2fg, ops.source_dpi, ops.target_dpi);
            ops.target_dpi
        } else {
            0
        };
        f_2by(&r_2fg, &work)?;
        f_24u(&source, &work);
        Ok(f_29l(&r_2fg, &source, r_2fk))
    })
    .await
    .map_err(|e| format!("luồng xử lý ảnh lỗi: {e}"))?
}
#[tauri::command]
pub async fn reset_scan_page(
    auth: State<'_, crate::auth::AuthState>,
    source_path: String,
) -> Result<Thm, String> {
    auth.require_perm_str(digitext_core::Permission::ScanExecute)?;
    tauri::async_runtime::spawn_blocking(move || {
        let source = f_19y(&source_path)?;
        let lock = f_1xt(&source);
        let _held = lock.lock().map_err(|_| "khoá trang hỏng")?;
        let _ = std::fs::remove_file(f_2rc(&source));
        let _ = std::fs::remove_dir_all(f_1h2(&source));
        let r_2fm = image::open(&source)
            .map_err(|e| format!("không mở được ảnh gốc: {e}"))?;
        Ok(f_29l(&r_2fm, &source, 0))
    })
    .await
    .map_err(|e| format!("luồng xử lý ảnh lỗi: {e}"))?
}
fn f_2hi(source_path: &str, r_2fn: bool) -> Result<Thm, String> {
    let source = f_19y(source_path)?;
    let lock = f_1xt(&source);
    let _held = lock.lock().map_err(|_| "khoá trang hỏng")?;
    f_1h8(&source, r_2fn)?;
    let work = f_2rc(&source);
    let r_2fp = if work.exists() { &work } else { &source };
    let r_2fo =
        image::open(r_2fp).map_err(|e| format!("không mở được ảnh: {e}"))?;
    Ok(f_29l(&r_2fo, &source, 0))
}
#[tauri::command]
pub async fn undo_scan_page(
    auth: State<'_, crate::auth::AuthState>,
    source_path: String,
) -> Result<Thm, String> {
    auth.require_perm_str(digitext_core::Permission::ScanExecute)?;
    tauri::async_runtime::spawn_blocking(move || f_2hi(&source_path, true))
        .await
        .map_err(|e| format!("luồng xử lý ảnh lỗi: {e}"))?
}
#[tauri::command]
pub async fn redo_scan_page(
    auth: State<'_, crate::auth::AuthState>,
    source_path: String,
) -> Result<Thm, String> {
    auth.require_perm_str(digitext_core::Permission::ScanExecute)?;
    tauri::async_runtime::spawn_blocking(move || f_2hi(&source_path, false))
        .await
        .map_err(|e| format!("luồng xử lý ảnh lỗi: {e}"))?
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T6r {
    pub can_undo: bool,
    pub can_redo: bool,
}
#[tauri::command]
pub async fn scan_page_history(
    auth: State<'_, crate::auth::AuthState>,
    source_path: String,
) -> Result<T6r, String> {
    auth.require_perm_str(digitext_core::Permission::ScanExecute)?;
    let source = f_19y(&source_path)?;
    let r_2fq = f_1h2(&source);
    Ok(T6r {
        can_undo: can_undo(&r_2fq),
        can_redo: can_redo(&r_2fq),
    })
}
pub(crate) fn f_1xn(r_2fr: &image::DynamicImage, index: u32, dpi: u32, source_path: String) -> Tkt {
    let preview = f_199(&r_2fr.thumbnail(C_G1, C_G1).to_rgb8());
    let thumbnail = f_19b(&r_2fr.thumbnail(96, 136).to_rgb8());
    let mut r_2fs = f_vo(
        index,
        r_2fr.width(),
        r_2fr.height(),
        dpi,
        thumbnail,
        preview,
    );
    r_2fs.source_path = source_path;
    r_2fs
}
pub(crate) fn f_vo(
    index: u32,
    width_px: u32,
    height_px: u32,
    dpi: u32,
    thumbnail: String,
    preview: String,
) -> Tkt {
    let (r_2fu, r_2fv, r_2ft) = f_150(width_px, height_px, dpi);
    Tkt {
        index,
        width_px,
        height_px,
        dpi,
        width_mm: r_2fv,
        height_mm: r_2ft,
        paper: r_2fu.label().to_string(),
        paper_known: r_2fu.is_known(),
        dpi_ok: paper::f_17i(dpi),
        thumbnail,
        preview,
        source_path: String::new(),
    }
}
