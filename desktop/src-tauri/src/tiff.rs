// Copyright (c) 2026 DigiText. All rights reserved.
//! Tách TIFF (tài liệu Đảng) — bước giữa Ký số và Đóng gói. Nguồn: bản đã ký trên máy chủ hoặc thư mục PDF
//! trên máy. Mỗi trang một TIFF lấy nguyên ảnh quét (`digitext_pdf::tach_tiff`), đặt tên theo mẫu của đơn vị
//! (`digitext_core::tiff_naming`). Chạy song song theo thư mục nguồn (trong một thư mục phải tuần tự vì STT
//! chạy liên tục qua các PDF). Ghi `.part` rồi đổi tên; ổ đích mất kết nối (ổ USB rớt) thì chờ cắm lại rồi
//! ghi tiếp chứ không đánh lỗi hàng loạt.
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::tiff_naming::{group, natural_cmp, sanitize_segment, FolderPlan, TiffNaming};
use digitext_core::Permission;
use digitext_pdf::tach_tiff::{page_count, page_tiff, TiffColor, TiffCompression, TiffOpts};
use digitext_pdf::ParsedPdf;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};

static CANCEL: AtomicBool = AtomicBool::new(false);
static PAUSE: AtomicBool = AtomicBool::new(false);
static RUNNING: AtomicBool = AtomicBool::new(false);
const EV: &str = "tiff://progress";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TiffInput {
    /// Chuỗi thư mục nguồn (gồm tên thư mục gốc), phần tử cuối = thư mục chứa PDF.
    pub source_dirs: Vec<String>,
    pub file_name: String,
    /// Thư mục trên máy: đường dẫn tuyệt đối tới PDF.
    #[serde(default)]
    pub path: Option<String>,
    /// Máy chủ: id tài liệu (tải bản đã ký).
    #[serde(default)]
    pub doc_id: Option<String>,
    /// Số trang đã biết (máy chủ) — giữ được STT các PDF sau khi một PDF lỗi.
    #[serde(default)]
    pub pages: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalPdf {
    pub path: String,
    pub source_dirs: Vec<String>,
    pub file_name: String,
}

/// Quét đệ quy thư mục, lấy mọi PDF. `sourceDirs` bắt đầu bằng tên thư mục được chọn (như tool cũ).
#[tauri::command]
pub async fn tiff_scan_local(auth: State<'_, AuthState>, folder: String) -> Result<Vec<LocalPdf>, AppError> {
    auth.require_perm(Permission::SignView)?;
    let root = PathBuf::from(folder.trim());
    if !root.is_dir() {
        return Err(AppError::validation(format!("không thấy thư mục: {}", root.display())));
    }
    tauri::async_runtime::spawn_blocking(move || scan_local(&root))
        .await
        .map_err(|e| AppError::internal(format!("quét thư mục lỗi: {e}")))
}

fn scan_local(root: &Path) -> Vec<LocalPdf> {
    let root_name = root.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "PDF".into());
    let mut out = Vec::new();
    let mut stack = vec![(root.to_path_buf(), vec![root_name])];
    while let Some((dir, chain)) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            if p.is_dir() {
                let mut c = chain.clone();
                c.push(name);
                stack.push((p, c));
            } else if name.to_lowercase().ends_with(".pdf") {
                out.push(LocalPdf { path: p.to_string_lossy().into_owned(), source_dirs: chain.clone(), file_name: name });
            }
        }
    }
    out.sort_by(|a, b| a.source_dirs.cmp(&b.source_dirs).then_with(|| natural_cmp(&a.file_name, &b.file_name)));
    out
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewArgs {
    pub inputs: Vec<TiffInput>,
    pub naming: TiffNaming,
    #[serde(default)]
    pub out_dir: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewFolder {
    pub source_dirs: Vec<String>,
    pub output_dirs: Vec<String>,
    pub files: usize,
    pub first_pdf: String,
    pub first_tiff: String,
    /// Thư mục xuất đã có sẵn (lần chạy trước / trùng).
    pub exists: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub folder_count: usize,
    pub file_count: usize,
    pub folders: Vec<PreviewFolder>,
    pub collisions: Vec<String>,
}

fn stem(name: &str) -> &str {
    match name.rsplit_once('.') {
        Some((s, e)) if e.eq_ignore_ascii_case("pdf") => s,
        _ => name,
    }
}

fn plan_of(naming: &TiffNaming, inputs: &[TiffInput]) -> (Vec<FolderPlan>, Vec<String>) {
    let g: Vec<(Vec<String>, String)> = inputs
        .iter()
        .map(|i| (i.source_dirs.iter().map(|s| sanitize_segment(s)).collect(), i.file_name.clone()))
        .collect();
    group(naming, &g)
}

fn out_path(root: &Path, dirs: &[String]) -> PathBuf {
    dirs.iter().fold(root.to_path_buf(), |p, d| p.join(d))
}

/// Xem trước cây thư mục / tên TIFF đầu tiên của từng thư mục (tối đa 300 thư mục) + xung đột tên.
#[tauri::command]
pub async fn tiff_preview(auth: State<'_, AuthState>, args: PreviewArgs) -> Result<Preview, AppError> {
    auth.require_perm(Permission::SignView)?;
    args.naming.validate().map_err(AppError::validation)?;
    let (folders, collisions) = plan_of(&args.naming, &args.inputs);
    let root = args.out_dir.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(PathBuf::from);
    let list = folders
        .iter()
        .take(300)
        .map(|f| {
            let first = &args.inputs[f.files[0]].file_name;
            PreviewFolder {
                source_dirs: f.source_dirs.clone(),
                output_dirs: f.output_dirs.clone(),
                files: f.files.len(),
                first_pdf: first.clone(),
                first_tiff: args.naming.file_name(stem(first), args.naming.start),
                exists: root.as_ref().is_some_and(|r| out_path(r, &f.output_dirs).is_dir()),
            }
        })
        .collect();
    Ok(Preview { folder_count: folders.len(), file_count: args.inputs.len(), folders: list, collisions })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunArgs {
    pub inputs: Vec<TiffInput>,
    pub naming: TiffNaming,
    pub out_dir: String,
    /// "skip" (bỏ qua PDF đã đủ TIFF — chạy lại được) | "overwrite".
    #[serde(default = "skip")]
    pub on_exist: String,
    #[serde(default)]
    pub force_rgb: bool,
    #[serde(default)]
    pub uncompressed: bool,
    #[serde(default)]
    pub concurrency: Option<usize>,
}
fn skip() -> String {
    "skip".into()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub done_files: usize,
    pub total_files: usize,
    pub done_pages: usize,
    pub file: String,
    /// "ok" | "skipped" | "failed" | "waiting_dest" (ổ đích mất kết nối, đang chờ)
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedRow {
    pub file_name: String,
    pub source: String,
    pub error: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    pub total_files: usize,
    pub ok: usize,
    pub skipped: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub pages: usize,
    pub bytes: u64,
    pub failures: Vec<FailedRow>,
    pub report_path: Option<String>,
}

struct Row {
    source: String,
    out_dir: String,
    file_name: String,
    status: &'static str,
    pages: u32,
    tiffs: Vec<(String, String)>,
    dpi: String,
    color: String,
    bytes: u64,
    error: String,
}

/// Nguồn bản đã ký trên máy chủ (None: chỉ PDF trên máy — dùng cho test).
type Server<'a, 'r> = Option<(&'a State<'r, Db>, &'a State<'r, AuthState>)>;

struct Shared<'a> {
    on_progress: &'a (dyn Fn(Progress) + Sync),
    root: &'a Path,
    total: usize,
    done: AtomicUsize,
    pages: AtomicUsize,
    rows: Mutex<Vec<Row>>,
}

impl Shared<'_> {
    fn emit(&self, file: &str, status: &str, error: Option<String>) {
        (self.on_progress)(Progress {
            done_files: self.done.load(Ordering::SeqCst),
            total_files: self.total,
            done_pages: self.pages.load(Ordering::SeqCst),
            file: file.to_string(),
            status: status.into(),
            error,
        });
    }

    /// Chờ khi ổ đích mất kết nối (thư mục gốc xuất không còn). Trả false nếu người dùng dừng.
    async fn wait_dest(&self, file: &str) -> bool {
        let mut announced = false;
        while !self.root.is_dir() {
            if CANCEL.load(Ordering::SeqCst) {
                return false;
            }
            if !announced {
                self.emit(file, "waiting_dest", Some(format!("ổ đích mất kết nối: {}", self.root.display())));
                announced = true;
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        true
    }
}

async fn wait_pause() -> bool {
    while PAUSE.load(Ordering::SeqCst) && !CANCEL.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    !CANCEL.load(Ordering::SeqCst)
}

/// Ghi một tệp an toàn: `.part` → đổi tên. Lỗi khi ổ đích biến mất → chờ cắm lại rồi ghi lại.
async fn write_file(sh: &Shared<'_>, dir: &Path, name: &str, bytes: &[u8], file_label: &str) -> Result<(), String> {
    loop {
        let part = dir.join(format!("{name}.part"));
        let r = async {
            tokio::fs::create_dir_all(dir).await?;
            tokio::fs::write(&part, bytes).await?;
            tokio::fs::rename(&part, dir.join(name)).await
        }
        .await;
        match r {
            Ok(()) => return Ok(()),
            Err(e) => {
                let _ = std::fs::remove_file(&part);
                if !sh.root.is_dir() {
                    if !sh.wait_dest(file_label).await {
                        return Err("đã dừng trong lúc chờ ổ đích".into());
                    }
                    continue;
                }
                return Err(format!("ghi {name}: {e}"));
            }
        }
    }
}

async fn load_pdf(server: Server<'_, '_>, inp: &TiffInput) -> Result<Vec<u8>, String> {
    if let Some(p) = &inp.path {
        return tokio::fs::read(p).await.map_err(|e| format!("đọc PDF: {e}"));
    }
    let id = inp.doc_id.as_deref().ok_or("thiếu nguồn PDF")?;
    let (db, auth) = server.ok_or("không có kết nối máy chủ")?;
    let tmp = std::env::temp_dir().join("digitext-tiff").join(format!("{id}.pdf"));
    crate::docs::download_variant_to(db, auth, id, &tmp, "signed").await.map_err(|e| format!("tải bản đã ký: {e}"))?;
    let b = tokio::fs::read(&tmp).await.map_err(|e| format!("đọc bản tải về: {e}"));
    let _ = tokio::fs::remove_file(&tmp).await;
    b
}

async fn process_folder(
    sh: &Shared<'_>,
    server: Server<'_, '_>,
    args: &RunArgs,
    folder: &FolderPlan,
    opts: TiffOpts,
) {
    let dir = out_path(sh.root, &folder.output_dirs);
    let out_dir = folder.output_dirs.join("/");
    let mut seq = args.naming.start;
    let mut broken: Option<String> = None;
    for &i in &folder.files {
        let inp = &args.inputs[i];
        let source = format!("{}/{}", inp.source_dirs.join("/"), inp.file_name);
        let mut row = Row {
            source: source.clone(),
            out_dir: out_dir.clone(),
            file_name: inp.file_name.clone(),
            status: "failed",
            pages: 0,
            tiffs: Vec::new(),
            dpi: String::new(),
            color: String::new(),
            bytes: 0,
            error: String::new(),
        };
        if !wait_pause().await {
            return;
        }
        let res: Result<&'static str, String> = async {
            if let Some(b) = &broken {
                return Err(format!("bỏ qua: {b} lỗi trước đó nên không xác định được STT"));
            }
            let s = stem(&inp.file_name).to_string();
            // Đã đủ TIFF từ lần chạy trước (biết số trang) → bỏ qua, khỏi tải/giải mã.
            if args.on_exist == "skip" {
                if let Some(n) = inp.pages.filter(|n| *n > 0) {
                    let names: Vec<String> = (0..n).map(|k| args.naming.file_name(&s, seq + k)).collect();
                    if names.iter().all(|nm| std::fs::metadata(dir.join(nm)).is_ok_and(|m| m.len() > 0)) {
                        row.pages = n;
                        return Ok("skipped");
                    }
                }
            }
            let pdf = load_pdf(server, inp).await?;
            let parsed = tauri::async_runtime::spawn_blocking(move || ParsedPdf::parse(&pdf).map_err(|e| e.to_string()))
                .await
                .map_err(|e| e.to_string())??;
            let parsed = std::sync::Arc::new(parsed);
            let n = page_count(&parsed);
            if n == 0 {
                return Err("PDF không có trang".into());
            }
            row.pages = n;
            let names: Vec<String> = (0..n).map(|k| args.naming.file_name(&s, seq + k)).collect();
            if args.on_exist == "skip"
                && names.iter().all(|nm| std::fs::metadata(dir.join(nm)).is_ok_and(|m| m.len() > 0))
            {
                return Ok("skipped");
            }
            for (k, name) in names.iter().enumerate() {
                if CANCEL.load(Ordering::SeqCst) {
                    return Err("người dùng dừng giữa tệp".into());
                }
                let p = parsed.clone();
                let page = k as u32 + 1;
                let t = tauri::async_runtime::spawn_blocking(move || page_tiff(&p, page, opts).map_err(|e| e.to_string()))
                    .await
                    .map_err(|e| e.to_string())??;
                write_file(sh, &dir, name, &t.bytes, &inp.file_name).await?;
                if k == 0 {
                    row.dpi = if t.dpi_guessed { format!("{}?", t.dpi) } else { t.dpi.to_string() };
                    row.color = match t.color {
                        TiffColor::Rgb24 => "RGB 24 bit".into(),
                        TiffColor::Gray8 => "Xám 8 bit".into(),
                    };
                }
                row.bytes += t.bytes.len() as u64;
                row.tiffs.push((name.clone(), t.sha256));
                sh.pages.fetch_add(1, Ordering::SeqCst);
            }
            Ok("ok")
        }
        .await;
        match res {
            Ok(st) => {
                row.status = st;
                seq += row.pages;
            }
            Err(e) => {
                row.error = e.clone();
                match inp.pages.filter(|n| *n > 0) {
                    // Biết số trang (máy chủ) → các PDF sau vẫn đúng STT.
                    Some(n) => seq += n,
                    None if broken.is_none() => broken = Some(inp.file_name.clone()),
                    None => {}
                }
            }
        }
        sh.done.fetch_add(1, Ordering::SeqCst);
        sh.emit(&inp.file_name, row.status, (!row.error.is_empty()).then(|| row.error.clone()));
        sh.rows.lock().expect("rows").push(row);
        if CANCEL.load(Ordering::SeqCst) {
            return;
        }
    }
}

fn csv(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Biên bản CSV (mở bằng Excel) + manifest SHA-256, đặt tên kèm thời điểm để không đè lần chạy trước.
fn write_reports(root: &Path, rows: &[Row]) -> std::io::Result<PathBuf> {
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let report = root.join(format!("BIEN-BAN-TACH-TIFF-{ts}.csv"));
    let mut s = String::from("\u{feff}Tệp PDF,Thư mục nguồn,Thư mục TIFF,Trạng thái,Số trang,Số TIFF,TIFF đầu,TIFF cuối,Dpi,Màu,Dung lượng (MB),Lỗi\n");
    let mut man = String::new();
    for r in rows {
        let st = match r.status {
            "ok" => "Đã tách",
            "skipped" => "Đã có (bỏ qua)",
            _ => "Lỗi",
        };
        s.push_str(
            &[
                csv(&r.file_name),
                csv(r.source.rsplit_once('/').map_or("", |x| x.0)),
                csv(&r.out_dir),
                st.into(),
                r.pages.to_string(),
                r.tiffs.len().to_string(),
                csv(r.tiffs.first().map_or("", |t| t.0.as_str())),
                csv(r.tiffs.last().map_or("", |t| t.0.as_str())),
                r.dpi.clone(),
                r.color.clone(),
                format!("{:.1}", r.bytes as f64 / 1e6),
                csv(&r.error),
            ]
            .join(","),
        );
        s.push('\n');
        for (name, sha) in &r.tiffs {
            man.push_str(&format!("{sha}  {}/{name}\n", r.out_dir));
        }
    }
    std::fs::write(&report, s)?;
    if !man.is_empty() {
        std::fs::write(root.join(format!("manifest-sha256-{ts}.txt")), man)?;
    }
    Ok(report)
}

#[tauri::command]
pub async fn tiff_run(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: RunArgs,
) -> Result<RunResult, AppError> {
    auth.require_perm(Permission::SignExecute)?;
    args.naming.validate().map_err(AppError::validation)?;
    let root = PathBuf::from(args.out_dir.trim());
    if !root.is_absolute() || !root.is_dir() {
        return Err(AppError::validation(format!("thư mục xuất không tồn tại: {}", root.display())));
    }
    if args.inputs.is_empty() {
        return Err(AppError::validation("chưa chọn PDF nào"));
    }
    let (folders, collisions) = plan_of(&args.naming, &args.inputs);
    if !collisions.is_empty() {
        return Err(AppError::validation(format!(
            "{} xung đột tên TIFF — sửa trước khi chạy: {}",
            collisions.len(),
            collisions.iter().take(5).cloned().collect::<Vec<_>>().join("; ")
        )));
    }
    if RUNNING.swap(true, Ordering::SeqCst) {
        return Err(AppError::validation("đang có một đợt tách TIFF chạy"));
    }
    CANCEL.store(false, Ordering::SeqCst);
    PAUSE.store(false, Ordering::SeqCst);
    tracing::info!(files = args.inputs.len(), folders = folders.len(), "tiff_run");
    let emit = |p: Progress| {
        let _ = app.emit(EV, p);
    };
    let out = run_batch(&args, &folders, &root, &emit, Some((&db, &auth))).await;
    RUNNING.store(false, Ordering::SeqCst);
    tracing::info!(ok = out.ok, skipped = out.skipped, failed = out.failed, cancelled = out.cancelled, "tiff_run xong");
    Ok(out)
}

async fn run_batch(
    args: &RunArgs,
    folders: &[FolderPlan],
    root: &Path,
    on_progress: &(dyn Fn(Progress) + Sync),
    server: Server<'_, '_>,
) -> RunResult {
    let opts = TiffOpts {
        force_rgb: args.force_rgb,
        compression: if args.uncompressed { TiffCompression::None } else { TiffCompression::Lzw },
    };
    let sh = Shared {
        on_progress,
        root,
        total: args.inputs.len(),
        done: AtomicUsize::new(0),
        pages: AtomicUsize::new(0),
        rows: Mutex::new(Vec::new()),
    };
    let conc = args.concurrency.unwrap_or(3).clamp(1, 8);
    let shr = &sh;
    futures_util::stream::iter(0..folders.len())
        .map(move |i| process_folder(shr, server, args, &folders[i], opts))
        .buffer_unordered(conc)
        .collect::<Vec<()>>()
        .await;

    let rows = sh.rows.into_inner().expect("rows");
    let mut out = RunResult { total_files: args.inputs.len(), ..Default::default() };
    for r in &rows {
        match r.status {
            "ok" => out.ok += 1,
            "skipped" => out.skipped += 1,
            _ => {
                out.failed += 1;
                out.failures.push(FailedRow { file_name: r.file_name.clone(), source: r.source.clone(), error: r.error.clone() });
            }
        }
        out.pages += r.tiffs.len();
        out.bytes += r.bytes;
    }
    out.cancelled = out.total_files - rows.len();
    if root.is_dir() && !rows.is_empty() {
        match write_reports(root, &rows) {
            Ok(p) => out.report_path = Some(p.to_string_lossy().into_owned()),
            Err(e) => tracing::warn!(error = %e, "tiff_run: ghi biên bản lỗi"),
        }
    }
    out
}

#[tauri::command]
pub async fn tiff_cancel(auth: State<'_, AuthState>) -> Result<(), AppError> {
    auth.require_perm(Permission::SignView)?;
    CANCEL.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn tiff_pause(auth: State<'_, AuthState>, paused: bool) -> Result<(), AppError> {
    auth.require_perm(Permission::SignView)?;
    PAUSE.store(paused, Ordering::SeqCst);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use digitext_core::tiff_naming::{TiffNamingRule, TiffNaming};

    fn pdf(pages: u32, gray: bool) -> Vec<u8> {
        let mut jpeg = Vec::new();
        if gray {
            let img = image::GrayImage::from_pixel(120, 160, image::Luma([90]));
            image::codecs::jpeg::JpegEncoder::new(&mut jpeg).encode_image(&img).unwrap();
        } else {
            let img = image::RgbImage::from_pixel(120, 160, image::Rgb([200, 10, 10]));
            image::codecs::jpeg::JpegEncoder::new(&mut jpeg).encode_image(&img).unwrap();
        }
        let p: Vec<digitext_pdf::Tgh> = (0..pages)
            .map(|_| digitext_pdf::Tgh { jpeg: jpeg.clone(), width_px: 120, height_px: 160, dpi: 300, spans: vec![] })
            .collect();
        digitext_pdf::f_vz(&p).unwrap()
    }

    fn run(args: &RunArgs, root: &Path) -> RunResult {
        let (folders, coll) = plan_of(&args.naming, &args.inputs);
        assert!(coll.is_empty(), "{coll:?}");
        tauri::async_runtime::block_on(run_batch(args, &folders, root, &|_| {}, None))
    }

    #[test]
    fn tach_thu_muc_tren_may_dat_ten_v2_va_chay_lai_bo_qua() {
        let tmp = std::env::temp_dir().join(format!("digitext-tiff-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let src = tmp.join("R");
        let out = tmp.join("OUT");
        std::fs::create_dir_all(src.join("06").join("42")).unwrap();
        std::fs::create_dir_all(src.join("06").join("43")).unwrap();
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(src.join("06/42/H49.44-001-01-0042-002.pdf"), pdf(1, false)).unwrap();
        std::fs::write(src.join("06/42/H49.44-001-01-0042-001.pdf"), pdf(2, false)).unwrap();
        std::fs::write(src.join("06/43/H49.44-ĐU-NQ-0001-2002.pdf"), pdf(1, true)).unwrap();
        std::fs::write(src.join("06/43/hong.pdf"), b"%PDF-1.4 hong").unwrap();

        let inputs: Vec<TiffInput> = scan_local(&src)
            .into_iter()
            .map(|f| TiffInput { source_dirs: f.source_dirs, file_name: f.file_name, path: Some(f.path), doc_id: None, pages: None })
            .collect();
        assert_eq!(inputs.len(), 4);
        let mut args = RunArgs {
            inputs,
            naming: TiffNaming::preset(TiffNamingRule::V2),
            out_dir: out.to_string_lossy().into_owned(),
            on_exist: "skip".into(),
            force_rgb: false,
            uncompressed: false,
            concurrency: Some(2),
        };
        let r = run(&args, &out);
        assert_eq!((r.ok, r.skipped, r.failed, r.pages), (3, 0, 1, 4), "{:?}", r.failures);
        let d42 = out.join("R/06/H49.44-001-01-0042");
        for n in ["H49.44-001-01-0042-001-001.tif", "H49.44-001-01-0042-001-002.tif", "H49.44-001-01-0042-002-003.tif"] {
            assert!(d42.join(n).is_file(), "thiếu {n}");
        }
        // Văn bản Đảng: giữ tên thư mục gốc "43".
        assert!(out.join("R/06/43/H49.44-ĐU-NQ-0001-2002-001.tif").is_file());
        let report = r.report_path.clone().unwrap();
        let csv = std::fs::read_to_string(&report).unwrap();
        assert!(csv.contains("hong.pdf") && csv.contains("Lỗi"));
        let man: Vec<_> = std::fs::read_dir(&out)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("manifest-sha256"))
            .collect();
        assert_eq!(std::fs::read_to_string(man[0].path()).unwrap().lines().count(), 4);
        assert!(std::fs::read_dir(&d42).unwrap().flatten().all(|e| !e.file_name().to_string_lossy().ends_with(".part")));

        // Chạy lại: PDF đã đủ TIFF được bỏ qua, không ghi lại.
        std::thread::sleep(Duration::from_millis(1100));
        let r2 = run(&args, &out);
        assert_eq!((r2.ok, r2.skipped, r2.failed), (0, 3, 1));
        args.on_exist = "overwrite".into();
        std::thread::sleep(Duration::from_millis(1100));
        assert_eq!(run(&args, &out).ok, 3);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
