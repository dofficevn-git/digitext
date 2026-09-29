// Copyright (c) 2026 DigiText. All rights reserved.
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};
use tauri::{AppHandle, Emitter, State};
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocRow {
    pub id: String,
    pub project_id: String,
    pub filename: String,
    pub ho_so_ref: Option<String>,
    pub page_count: Option<i32>,
    pub size_bytes: i64,
    pub status: String,
    pub uploaded_by_name: Option<String>,
    pub uploaded_at: String,
    #[serde(default = "review_pending")]
    pub review_status: String,
    #[serde(default)]
    pub reviewed_at: Option<String>,
    #[serde(default)]
    pub review_note: Option<String>,
}
fn review_pending() -> String {
    "pending".to_owned()
}
#[tauri::command]
pub async fn docs_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Vec<DocRow>, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::DocumentList,
        move |r_1j7, r_1j8| async move {
            client::f_1eo::<Vec<DocRow>>(&r_1j7, &r_1j8, "/documents").await
        },
    )
    .await
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeDocRow {
    pub id: String,
    pub filename: String,
    pub rel_path: Option<String>,
    pub ho_so_ref: Option<String>,
    pub page_count: Option<i32>,
    pub size_bytes: i64,
    pub status: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub project_code: Option<String>,
    #[serde(default)]
    pub project_name: Option<String>,
    #[serde(default)]
    pub fields_status: Option<String>,
    #[serde(default = "review_pending")]
    pub review_status: String,
    #[serde(default)]
    pub ocr_pages: Option<i64>,
    #[serde(default)]
    pub field_pages: Option<i64>,
    #[serde(default)]
    pub pdfa_pages: Option<i32>,
    #[serde(default)]
    pub pdfa2b: Option<bool>,
}
#[tauri::command]
pub async fn docs_tree(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Vec<TreeDocRow>, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::DocumentList,
        move |r_1j9, r_1ja| async move {
            client::f_1eo::<Vec<TreeDocRow>>(&r_1j9, &r_1ja, "/documents/tree").await
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadArgs {
    pub id: String,
    pub filename: String,
    #[serde(default)]
    pub rel_path: Option<String>,
    /// `true` → tải bản PDF/A (lớp chữ) về `_pdfa/…` thay vì bản gốc về `_nhan_ve/…`.
    #[serde(default)]
    pub pdfa: bool,
    /// Loại bản tải: "original" | "pdfa" | "signed". Có thì ghi đè `pdfa`.
    /// "signed" → bản ĐÃ KÝ SỐ về `_da_ky_so/…` (hoặc thư mục người dùng chọn).
    #[serde(default)]
    pub variant: Option<String>,
    /// Thư mục người dùng CHỌN để lưu (tuyệt đối). Có → lưu `<dest>/<rel_path>/<tên tệp>`,
    /// không qua Kho `_nhan_ve`/`_pdfa`. Không có → như cũ (Kho trong Documents).
    #[serde(default)]
    pub dest_dir: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tpa {
    pub project_id: String,
    pub path: String,
    #[serde(default)]
    pub ho_so_ref: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tpb {
    pub id: String,
    pub filename: String,
    pub duplicate: bool,
}
#[tauri::command]
pub async fn docs_upload_local(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tpa,
) -> Result<Tpb, AppError> {
    let project_id = args.project_id.trim().to_string();
    if project_id.is_empty() {
        return Err(
            AppError::validation(
                "Chưa xác định dự án — hãy đăng nhập vào một dự án rồi thử lại.",
            ),
        );
    }
    let path = PathBuf::from(&args.path);
    let filename = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .ok_or_else(|| {
            AppError::validation("Đường dẫn tệp không hợp lệ.")
        })?;
    if !filename.to_lowercase().ends_with(".pdf") {
        return Err(AppError::validation("Chỉ hỗ trợ tệp PDF."));
    }
    let (bytes, blake3_hex, page_count) = tauri::async_runtime::spawn_blocking(
        move || -> std::io::Result<(Vec<u8>, String, Option<i64>)> {
            let bytes = std::fs::read(&path)?;
            let mut r_1jd = blake3::Hasher::new();
            r_1jd.update(&bytes);
            let r_1jc = r_1jd.finalize().to_hex().to_string();
            let pages = lopdf::Document::load_mem(&bytes)
                .ok()
                .map(|d| d.get_pages().len() as i64);
            Ok((bytes, r_1jc, pages))
        },
    )
    .await
    .map_err(|e| AppError::internal(format!("luồng đọc tệp lỗi: {e}")))?
    .map_err(|e| AppError::internal(format!("đọc tệp lỗi: {e}")))?;
    let ho_so = args
        .ho_so_ref
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let fname = filename.clone();
    let r_1je = f_2r8(
        &db,
        &auth,
        Permission::UploadExecute,
        move |r_1jb, r_1jf| {
            let (project_id, fname, ho_so, blake3_hex, bytes) = (
                project_id.clone(),
                fname.clone(),
                ho_so.clone(),
                blake3_hex.clone(),
                bytes.clone(),
            );
            async move {
                client::f_2nn(
                    &r_1jb,
                    &r_1jf,
                    &project_id,
                    &fname,
                    ho_so.as_deref(),
                    page_count,
                    &blake3_hex,
                    bytes,
                )
                .await
            }
        },
    )
    .await?;
    tracing::info!(
        document_id = % r_1je.id, duplicate = r_1je.duplicate, page_count = page_count
        .unwrap_or(- 1),
        "docs_upload_local: tải PDF lên máy chủ xong"
    );
    Ok(Tpb {
        id: r_1je.id,
        filename,
        duplicate: r_1je.duplicate,
    })
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tf9 {
    #[serde(default)]
    pub project_id: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrAllResult {
    pub documents: u32,
    pub pages: i64,
    pub skipped_no_page_count: u32,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfaRow {
    pub id: String,
    pub filename: String,
    pub ho_so_ref: Option<String>,
    pub status: String,
    pub state: String,
    pub pages: u32,
    pub expected_pages: Option<i32>,
    pub has_text_layer: bool,
    pub has_image: bool,
    pub pages_with_text: u32,
    pub pages_without_text: u32,
    pub pdfa2b: bool,
    pub reason: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebuildResult {
    pub checked: u32,
    pub enqueued: u32,
    pub skipped_signed: u32,
}
#[tauri::command]
pub async fn docs_ocr_all(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tf9,
) -> Result<OcrAllResult, AppError> {
    let project_id = args.project_id.clone();
    f_2r8(&db, &auth, Permission::DocumentList, move |r_1jg, r_1ji| {
        let project_id = project_id.clone();
        async move {
            let r_1jh = serde_json::json!({ "projectId" : project_id });
            client::f_23d::<OcrAllResult>(&r_1jg, &r_1ji, "/documents/ocr-all", &r_1jh).await
        }
    })
    .await
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MakePdf2AllResult {
    pub documents: u32,
    #[serde(default)]
    pub skipped_rejected: u32,
}
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tcv {
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub document_ids: Option<Vec<String>>,
}
#[tauri::command]
pub async fn docs_make_pdf2_all(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tcv,
) -> Result<MakePdf2AllResult, AppError> {
    let (project_id, document_ids) = (args.project_id.clone(), args.document_ids.clone());
    f_2r8(&db, &auth, Permission::DocumentList, move |r_1jj, r_1jl| {
        let project_id = project_id.clone();
        let document_ids = document_ids.clone();
        async move {
            let r_1jk = serde_json::json!(
                { "projectId" : project_id, "documentIds" : document_ids, }
            );
            client::f_23d::<MakePdf2AllResult>(&r_1jj, &r_1jl, "/documents/make-pdf2-all", &r_1jk)
                .await
        }
    })
    .await
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrRetryFailedArgs {
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub document_ids: Option<Vec<String>>,
    #[serde(default)]
    pub include_page_failed: bool,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrRetryFailedResult {
    pub documents: u64,
    pub pages: u64,
    pub skipped_signed: u64,
}
#[tauri::command]
pub async fn docs_ocr_retry_failed(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: OcrRetryFailedArgs,
) -> Result<OcrRetryFailedResult, AppError> {
    let (project_id, document_ids, include_page_failed) = (
        args.project_id.clone(),
        args.document_ids.clone(),
        args.include_page_failed,
    );
    f_2r8(&db, &auth, Permission::DocumentList, move |r_1jj, r_1jl| {
        let project_id = project_id.clone();
        let document_ids = document_ids.clone();
        async move {
            let body = serde_json::json!(
                { "projectId" : project_id, "documentIds" : document_ids,
                "includePageFailed" : include_page_failed, }
            );
            client::f_23d::<OcrRetryFailedResult>(
                &r_1jj,
                &r_1jl,
                "/documents/ocr-retry-failed",
                &body,
            )
            .await
        }
    })
    .await
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractFieldsAllResult {
    pub documents: u32,
    pub skipped_no_page_count: i64,
    #[serde(default)]
    pub skipped_page_failed: i64,
}
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T73 {
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub retry_failed: bool,
    #[serde(default)]
    pub document_ids: Option<Vec<String>>,
}
#[tauri::command]
pub async fn docs_extract_fields_all(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T73,
) -> Result<ExtractFieldsAllResult, AppError> {
    let (project_id, retry_failed) = (args.project_id.clone(), args.retry_failed);
    let document_ids = args.document_ids.clone();
    f_2r8(&db, &auth, Permission::DocumentList, move |r_1jm, r_1jo| {
        let project_id = project_id.clone();
        let document_ids = document_ids.clone();
        async move {
            let r_1jn = serde_json::json!(
                { "projectId" : project_id, "retryFailed" : retry_failed,
                "documentIds" : document_ids, }
            );
            client::f_23d::<ExtractFieldsAllResult>(
                &r_1jm,
                &r_1jo,
                "/documents/extract-fields-all",
                &r_1jn,
            )
            .await
        }
    })
    .await
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressDto {
    pub docs: i64,
    pub pages_total: i64,
    pub pages_ocr_done: i64,
    pub docs_ocr_pending: i64,
    pub docs_ocr_done: i64,
    pub docs_pdf2_done: i64,
    #[serde(default)]
    pub docs_pdf2_rejected: i64,
    #[serde(default)]
    pub docs_fields_done: i64,
    #[serde(default)]
    pub docs_fields_active: i64,
    #[serde(default)]
    pub docs_fields_failed: i64,
    #[serde(default)]
    pub pages_ocr_failed: i64,
    #[serde(default)]
    pub pages_ocr_transient_failed: i64,
    #[serde(default)]
    pub pages_ocr_queued: i64,
    #[serde(default)]
    pub pages_ocr_running: i64,
    #[serde(default)]
    pub docs_ocr_failed: i64,
    #[serde(default)]
    pub pages_ink_measured: i64,
    #[serde(default)]
    pub pages_ink_low: i64,
    pub by_status: std::collections::HashMap<String, i64>,
}
#[tauri::command]
pub async fn docs_progress(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<ProgressDto, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::DocumentList,
        move |r_1jp, r_1jq| async move {
            client::f_1eo::<ProgressDto>(&r_1jp, &r_1jq, "/documents/progress").await
        },
    )
    .await
}
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportCounts {
    pub docs: i64,
    pub pages: i64,
    pub ocr_done: i64,
    #[serde(default)]
    pub ocr_pages: Option<i64>,
    #[serde(default)]
    pub field_docs: Option<i64>,
    #[serde(default)]
    pub pdfa_docs: Option<i64>,
    #[serde(default)]
    pub pdfa_old: Option<i64>,
    pub approved: i64,
    pub rejected: i64,
    pub pending: i64,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thq {
    pub project_id: String,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    #[serde(flatten)]
    pub counts: ReportCounts,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportDto {
    pub projects_count: i64,
    pub from: Option<String>,
    pub to: Option<String>,
    pub totals: ReportCounts,
    pub projects: Vec<Thq>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportArgs {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}
fn f_1ue(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(r_1jr, c)| r_1jr == 4 || r_1jr == 7 || c.is_ascii_digit())
}
fn f_289(r_1js: &ReportArgs) -> String {
    let mut r_1jt: Vec<String> = Vec::new();
    for (ten, r_1ju) in [("from", &r_1js.from), ("to", &r_1js.to)] {
        if let Some(s) = r_1ju.as_deref().map(str::trim).filter(|s| f_1ue(s)) {
            r_1jt.push(format!("{ten}={s}"));
        }
    }
    if r_1jt.is_empty() {
        String::new()
    } else {
        format!("?{}", r_1jt.join("&"))
    }
}
#[tauri::command]
pub async fn docs_report(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: ReportArgs,
) -> Result<ReportDto, AppError> {
    let path = format!("/documents/report{}", f_289(&args));
    f_2r8(&db, &auth, Permission::DocumentList, move |r_1jv, r_1jw| {
        let path = path.clone();
        async move { client::f_1eo::<ReportDto>(&r_1jv, &r_1jw, &path).await }
    })
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewArgs {
    pub id: String,
    pub decision: String,
    #[serde(default)]
    pub note: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResult {
    pub id: String,
    pub review_status: String,
    pub reviewed_at: Option<String>,
    pub review_note: Option<String>,
}
fn f_2oo(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && b.iter().enumerate().all(|(r_1jx, c)| match r_1jx {
            8 | 13 | 18 | 23 => *c == b'-',
            _ => c.is_ascii_hexdigit(),
        })
}
#[tauri::command]
pub async fn docs_review(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: ReviewArgs,
) -> Result<ReviewResult, AppError> {
    if !f_2oo(&args.id) {
        return Err(AppError::validation(
            "mã tài liệu không hợp lệ",
        ));
    }
    let (id, decision, note) = (args.id.clone(), args.decision.clone(), args.note.clone());
    f_2r8(
        &db,
        &auth,
        Permission::DocumentReview,
        move |r_1jy, r_1k0| {
            let (id, decision, note) = (id.clone(), decision.clone(), note.clone());
            async move {
                let r_1jz = serde_json::json!(
                    { "decision" : decision, "note" : note }
                );
                let path = format!("/documents/{id}/review");
                client::f_23d::<ReviewResult>(&r_1jy, &r_1k0, &path, &r_1jz).await
            }
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tj5 {
    pub ids: Vec<String>,
    pub decision: String,
    #[serde(default)]
    pub note: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewBulkResult {
    pub updated: i64,
    pub skipped: i64,
    pub review_status: String,
}
#[tauri::command]
pub async fn docs_review_bulk(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tj5,
) -> Result<ReviewBulkResult, AppError> {
    if args.ids.is_empty() {
        return Err(AppError::validation(
            "chưa chọn tài liệu nào để duyệt",
        ));
    }
    if !args.ids.iter().all(|id| f_2oo(id)) {
        return Err(AppError::validation(
            "mã tài liệu không hợp lệ",
        ));
    }
    let (ids, decision, note) = (args.ids.clone(), args.decision.clone(), args.note.clone());
    f_2r8(
        &db,
        &auth,
        Permission::DocumentReview,
        move |r_1k1, r_1k3| {
            let (ids, decision, note) = (ids.clone(), decision.clone(), note.clone());
            async move {
                let r_1k2 = serde_json::json!(
                    { "ids" : ids, "decision" : decision, "note" : note }
                );
                client::f_23d::<ReviewBulkResult>(&r_1k1, &r_1k3, "/documents/review-bulk", &r_1k2)
                    .await
            }
        },
    )
    .await
}
#[tauri::command]
pub async fn docs_pdfa_check(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Vec<PdfaRow>, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::DocumentList,
        move |r_1k4, r_1k5| async move {
            client::f_1eo::<Vec<PdfaRow>>(&r_1k4, &r_1k5, "/documents/pdfa-report").await
        },
    )
    .await
}
#[tauri::command]
pub async fn docs_rebuild_non_pdfa(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tf9,
) -> Result<RebuildResult, AppError> {
    let project_id = args.project_id.clone();
    f_2r8(&db, &auth, Permission::DocumentList, move |r_1k6, r_1k8| {
        let project_id = project_id.clone();
        async move {
            let r_1k7 = serde_json::json!({ "projectId" : project_id });
            client::f_23d::<RebuildResult>(&r_1k6, &r_1k8, "/documents/rebuild-non-pdfa", &r_1k7)
                .await
        }
    })
    .await
}
const C_PK: &str = "_xem_tam";
const C_PM: usize = 40;
const C_PL: u64 = 4 * 1024 * 1024 * 1024;
pub const C_6M: &str = "docs://download";
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T57 {
    pub id: String,
    pub received: u64,
    pub total: Option<u64>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tps {
    pub id: String,
    #[serde(default)]
    pub size_bytes: Option<i64>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tpt {
    pub path: String,
    pub cached: bool,
    pub size_bytes: u64,
}
fn f_wz(id: &str) -> String {
    let r_1k9: String = id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(80)
        .collect();
    if r_1k9.is_empty() {
        "tai-lieu".into()
    } else {
        r_1k9
    }
}
fn f_2o9(r_1kc: Option<std::fs::Metadata>, r_1ka: Option<u64>) -> Option<u64> {
    let r_1kb = r_1kc?;
    if !r_1kb.is_file() || r_1kb.len() == 0 {
        return None;
    }
    match r_1ka {
        Some(r_1kd) if r_1kd != r_1kb.len() => None,
        _ => Some(r_1kb.len()),
    }
}
fn f_24j(mut r_1kf: Vec<(PathBuf, SystemTime, u64)>, r_1kj: usize, r_1ki: u64) -> Vec<PathBuf> {
    r_1kf.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut r_1kh = 0u64;
    let mut r_1ke = Vec::new();
    for (r_1kg, (path, _, r_1kk)) in r_1kf.into_iter().enumerate() {
        let r_1kl = r_1kg >= r_1kj || r_1kh.saturating_add(r_1kk) > r_1ki;
        if r_1kl {
            r_1ke.push(path);
        } else {
            r_1kh += r_1kk;
        }
    }
    r_1ke
}
const C_FT: Duration = Duration::from_secs(3600);
fn f_1jl(name: &str, r_1kn: SystemTime, r_1ko: SystemTime, r_1km: Duration) -> bool {
    if !name.contains(".part") {
        return false;
    }
    r_1ko
        .duration_since(r_1kn)
        .map(|d| d >= r_1km)
        .unwrap_or(false)
}
fn f_24k(r_1kp: &Path) {
    let Ok(r_1ku) = std::fs::read_dir(r_1kp) else {
        return;
    };
    let r_1kt = SystemTime::now();
    let mut r_1kq = Vec::new();
    for e in r_1ku.flatten() {
        let path = e.path();
        let Ok(r_1kr) = e.metadata() else { continue };
        if !r_1kr.is_file() {
            continue;
        }
        let r_1ks = r_1kr.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let name = e.file_name().to_string_lossy().into_owned();
        if f_1jl(&name, r_1ks, r_1kt, C_FT) {
            let _ = std::fs::remove_file(&path);
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
            r_1kq.push((path, r_1ks, r_1kr.len()));
        }
    }
    for path in f_24j(r_1kq, C_PM, C_PL) {
        let _ = std::fs::remove_file(path);
    }
}
fn part_path(r_1kv: &Path) -> PathBuf {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    r_1kv.with_extension(format!("pdf.part{n}"))
}
async fn f_174(
    r_1kx: &State<'_, Db>,
    auth: &State<'_, AuthState>,
    id: &str,
    r_1l0: &Path,
    r_1kz: &(dyn Fn(u64, Option<u64>) + Sync),
) -> Result<u64, AppError> {
    f_174_variant(r_1kx, auth, id, r_1l0, r_1kz, false).await
}
async fn f_174_variant(
    r_1kx: &State<'_, Db>,
    auth: &State<'_, AuthState>,
    id: &str,
    r_1l0: &Path,
    r_1kz: &(dyn Fn(u64, Option<u64>) + Sync),
    pdfa: bool,
) -> Result<u64, AppError> {
    f_174_kind(r_1kx, auth, id, r_1l0, r_1kz, if pdfa { "pdfa" } else { "original" }).await
}
async fn f_174_kind(
    r_1kx: &State<'_, Db>,
    auth: &State<'_, AuthState>,
    id: &str,
    r_1l0: &Path,
    r_1kz: &(dyn Fn(u64, Option<u64>) + Sync),
    kind: &str,
) -> Result<u64, AppError> {
    let r_1ky = r_1l0.parent().ok_or_else(|| {
        AppError::internal("đường dẫn lưu không hợp lệ")
    })?;
    tokio::fs::create_dir_all(r_1ky)
        .await
        .map_err(|e| AppError::internal(format!("tạo thư mục lưu lỗi: {e}")))?;
    let part = part_path(r_1l0);
    let n = crate::auth::admin::f_2r8_any(r_1kx, auth, &[Permission::DocumentDownload, Permission::WorkExecute], |r_1kw, r_1l1| {
        let (id, part) = (id.to_string(), part.clone());
        let route = match kind {
            "pdfa" => format!("/documents/{id}/download?variant=pdfa"),
            "signed" => format!("/documents/{id}/download?variant=signed"),
            _ => format!("/documents/{id}/download"),
        };
        async move { client::f_1ez(&r_1kw, &r_1l1, &route, &part, r_1kz).await }
    })
    .await
    .inspect_err(|_| {
        let _ = std::fs::remove_file(&part);
    })?;
    tokio::fs::rename(&part, r_1l0)
        .await
        .map_err(|e| AppError::internal(format!("lưu file lỗi: {e}")))?;
    Ok(n)
}
const C_BN: usize = 8;
fn f_179(r_1l3: &Path, rel_path: Option<&str>, filename: &str) -> PathBuf {
    f_179_in(r_1l3, "_nhan_ve", rel_path, filename)
}
fn f_179_in(r_1l3: &Path, bucket: &str, rel_path: Option<&str>, filename: &str) -> PathBuf {
    let r_1l5: Vec<&str> = rel_path
        .unwrap_or("")
        .split(['/', '\\'])
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .take(C_BN)
        .collect();
    let base = if bucket.is_empty() { r_1l3.to_path_buf() } else { r_1l3.join(bucket) };
    let r_1l2 = crate::storage::f_1jt(&base, &r_1l5, false);
    let r_1l4 = Path::new(filename)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tai-lieu.pdf".into());
    r_1l2.join(crate::storage::sanitize(&r_1l4, 120, false))
}
#[tauri::command]
pub async fn docs_download(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: DownloadArgs,
) -> Result<String, AppError> {
    let kind: &str = match args.variant.as_deref().map(str::trim) {
        Some("signed") => "signed",
        Some("pdfa") => "pdfa",
        Some("original") => "original",
        _ if args.pdfa => "pdfa",
        _ => "original",
    };
    let r_1l6 = match args.dest_dir.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(dest) => {
            let dest = Path::new(dest);
            if !dest.is_absolute() {
                return Err(AppError::validation("thư mục lưu phải là đường dẫn tuyệt đối"));
            }
            if !dest.is_dir() {
                return Err(AppError::validation(format!("thư mục lưu không tồn tại: {}", dest.display())));
            }
            f_179_in(dest, "", args.rel_path.as_deref(), &args.filename)
        }
        None => {
            let r_1l7 = crate::archive::paths::f_29d(&app, crate::archive::paths::f_112(&db).as_deref())?;
            let bucket = match kind {
                "pdfa" => "_pdfa",
                "signed" => "_da_ky_so",
                _ => "_nhan_ve",
            };
            f_179_in(&r_1l7, bucket, args.rel_path.as_deref(), &args.filename)
        }
    };
    let gate = std::sync::Mutex::new(crate::forward_progress::ProgressGate::new(u64::MAX));
    let id = args.id.clone();
    let n = f_174_kind(
        &db,
        &auth,
        &args.id,
        &r_1l6,
        &|received, total| f_18y(&app, &gate, &id, received, total, false),
        kind,
    )
    .await?;
    f_18y(&app, &gate, &args.id, n, Some(n), true);
    Ok(r_1l6.to_string_lossy().into_owned())
}
fn f_18y(
    r_1l8: &AppHandle,
    gate: &std::sync::Mutex<crate::forward_progress::ProgressGate>,
    id: &str,
    received: u64,
    total: Option<u64>,
    xong: bool,
) {
    if !xong && !gate.lock().expect("gate").allow(received, Instant::now()) {
        return;
    }
    let _ = r_1l8.emit(
        C_6M,
        T57 {
            id: id.to_string(),
            received,
            total,
        },
    );
}
#[tauri::command]
pub async fn docs_pdf_for_view(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tps,
) -> Result<Tpt, AppError> {
    let r_1lc = crate::archive::paths::f_29d(&app, crate::archive::paths::f_112(&db).as_deref())?;
    let r_1l9 = r_1lc.join(C_PK);
    let r_1lb = r_1l9.join(format!("{}.pdf", f_wz(&args.id)));
    let r_1la = args.size_bytes.filter(|v| *v > 0).map(|v| v as u64);
    if let Some(size_bytes) = f_2o9(std::fs::metadata(&r_1lb).ok(), r_1la) {
        return Ok(Tpt {
            path: r_1lb.to_string_lossy().into_owned(),
            cached: true,
            size_bytes,
        });
    }
    let gate = std::sync::Mutex::new(crate::forward_progress::ProgressGate::new(u64::MAX));
    let id = args.id.clone();
    let size_bytes = f_174(&db, &auth, &args.id, &r_1lb, &|received, total| {
        f_18y(&app, &gate, &id, received, total, false)
    })
    .await?;
    f_18y(&app, &gate, &args.id, size_bytes, Some(size_bytes), true);
    f_24k(&r_1l9);
    tracing::info!(
        document_id = % args.id, size_bytes,
        "docs_pdf_for_view: đã tải PDF về cache xem tạm"
    );
    Ok(Tpt {
        path: r_1lb.to_string_lossy().into_owned(),
        cached: false,
        size_bytes,
    })
}
