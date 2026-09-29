// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::body::{Body, Bytes};
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, head, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::{DocStatus, Perm, Permission};
use digitext_db::documents::{self, DocRow, Tdt};
use digitext_db::ocr_jobs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;
const C_C7: usize = 200 * 1024 * 1024;
/// Tải lên chia khúc (tus) ghi dần ra đĩa nên cho tệp lớn hơn đường một lần: bản quét hồ sơ
/// dày có tệp 500+ MB (25/9/2026: 3 tệp LTLSBN 224–551 MB bị chặn ở 200 MB).
const MAX_RESUMABLE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/documents",
            post(f_2nl).layer(DefaultBodyLimit::max(C_C7)).get(f_1nb),
        )
        .route(
            "/documents/upload",
            head(f_1g9).patch(f_20h).layer(DefaultBodyLimit::max(C_1U)),
        )
        .route("/documents/{id}/download", get(f_173))
        .route("/documents/tree", get(f_2m3))
        .route("/documents/ocr-all", post(ocr_all))
        .route("/documents/ocr-retry-failed", post(ocr_retry_failed))
        .route("/documents/make-pdf2-all", post(make_pdf2_all))
        .route("/documents/extract-fields-all", post(extract_fields_all))
        .route("/documents/rebuild-non-pdfa", post(rebuild_non_pdfa))
        .route("/documents/progress", get(progress))
        .route("/documents/pdfa-report", get(f_21r))
        .route("/documents/report", get(report))
        .route("/documents/{id}/review", post(f_29u))
        .route("/documents/review-bulk", post(review_bulk))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tpe {
    pub project_id: Uuid,
    pub filename: String,
    #[serde(default)]
    pub ho_so_ref: Option<String>,
    #[serde(default)]
    pub page_count: Option<i32>,
    pub blake3: String,
    #[serde(default)]
    pub on_behalf_of: Option<Uuid>,
    #[serde(default)]
    pub batch_id: Option<Uuid>,
    #[serde(default)]
    pub rel_path: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tpf {
    pub id: Uuid,
    pub duplicate: bool,
}
const C_BV: usize = 8;
const C_C2: usize = 120;
fn f_zy(r_e3: &str) -> Option<String> {
    let r_e4: Vec<String> = r_e3
        .split(['/', '\\'])
        .map(|s| {
            let r_e2: String = s.chars().filter(|c| !c.is_control()).collect();
            r_e2.trim().chars().take(C_C2).collect()
        })
        .filter(|s: &String| !s.is_empty() && s != "." && s != "..")
        .take(C_BV)
        .collect();
    if r_e4.is_empty() {
        None
    } else {
        Some(r_e4.join("/"))
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub filename: String,
    pub ho_so_ref: Option<String>,
    pub page_count: Option<i32>,
    pub size_bytes: i64,
    pub status: String,
    pub uploaded_by_name: Option<String>,
    pub uploaded_at: DateTime<Utc>,
    pub review_status: String,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_note: Option<String>,
}
impl From<DocRow> for DocDto {
    fn from(r: DocRow) -> Self {
        DocDto {
            id: r.id,
            project_id: r.project_id,
            filename: r.filename,
            ho_so_ref: r.ho_so_ref,
            page_count: r.page_count,
            size_bytes: r.size_bytes,
            status: r.status,
            uploaded_by_name: r.uploaded_by_name,
            uploaded_at: r.uploaded_at,
            review_status: r.review_status,
            reviewed_at: r.reviewed_at,
            review_note: r.review_note,
        }
    }
}
async fn authorize_project(state: &AppState, r_e5: &Ts, project_id: Uuid) -> Result<(), ApiError> {
    let r_e6 = f_29a(&state.pool, &r_e5.user).await?;
    if !r_e5.can_touch_project(&r_e6, project_id) {
        return Err(ApiError::forbidden());
    }
    Ok(())
}
async fn f_29f(
    state: &AppState,
    r_e7: &Ts,
    project_id: Uuid,
    on_behalf_of: Option<Uuid>,
) -> Result<Option<Uuid>, ApiError> {
    match on_behalf_of {
        None => Ok(Some(r_e7.user.id)),
        Some(uid) if uid == r_e7.user.id => Ok(Some(r_e7.user.id)),
        Some(uid) => {
            if !r_e7.user.role.allows(Perm::Admin) {
                return Err(ApiError::forbidden());
            }
            let r_e9 = digitext_db::users::f_1eg(&state.pool, uid)
                .await?
                .ok_or_else(ApiError::forbidden)?;
            let r_e8 = f_29a(&state.pool, &r_e7.user).await?;
            if r_e9.project_id != Some(project_id) || !r_e8.allows_opt(r_e9.project_id) {
                return Err(ApiError::forbidden());
            }
            Ok(Some(uid))
        }
    }
}
async fn f_28s(
    state: &AppState,
    r_ea: &Ts,
    batch_id: Option<Uuid>,
) -> Result<Option<Uuid>, ApiError> {
    let Some(id) = batch_id else {
        return Ok(None);
    };
    if digitext_db::upload_batches::f_1j9(&state.pool, id, Some(r_ea.user.id)).await? {
        Ok(Some(id))
    } else {
        Err(ApiError::forbidden())
    }
}
async fn f_28v(
    state: &AppState,
    project_id: uuid::Uuid,
    blake3: &str,
    filename: &str,
    rel_path: Option<&str>,
    r_eb: &std::path::Path,
    r_ec: &str,
) -> Result<digitext_db::documents::Document, ApiError> {
    let _ = tokio::fs::remove_file(r_eb).await;
    if let Some(r_0) = state.source_store.as_ref() {
        if let Err(e) = r_0.delete(r_ec).await {
            tracing::warn!(
                key = % r_ec, error = % e,
                "không xoá được object mồ côi sau khi đua insert"
            );
        }
    }
    documents::find_same_file(&state.pool, project_id, blake3, filename, rel_path)
        .await?
        .ok_or_else(|| {
            ApiError::conflict(
                "tài liệu trùng vừa bị xoá, hãy gửi lại",
            )
        })
}
/// Bản sao thô trong `data/` chỉ là bước đệm để PUT lên kho object. Kho object có bản chính
/// (worker OCR, PDF/A, đóng gói, tải xuống đều đọc từ đó) nên giữ thêm bản thô là tốn gấp đôi
/// đĩa (25/9/2026: 45 GB thừa). Chỉ giữ khi `DIGITEXT_KEEP_SOURCE_COPY=true` hoặc không có kho
/// object (khi đó `data/` là nơi duy nhất còn tệp).
async fn drop_local_copy(state: &AppState, path: &std::path::Path) {
    if state.source_store.is_none() {
        return;
    }
    let keep = std::env::var("DIGITEXT_KEEP_SOURCE_COPY")
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);
    if keep {
        return;
    }
    if let Err(e) = tokio::fs::remove_file(path).await {
        tracing::warn!(error = % e, path = % path.display(), "không xoá được bản sao thô sau khi đẩy kho");
    }
}
async fn f_24y(state: &AppState, r_1: &str, r_ed: &std::path::Path) -> Result<(), ApiError> {
    let Some(r_2) = &state.source_store else {
        return Ok(());
    };
    let bytes = tokio::fs::read(r_ed).await.map_err(|e| {
        ApiError::from(anyhow::anyhow!(
            "đọc file để đẩy kho lỗi: {e}"
        ))
    })?;
    r_2.put(r_1, &bytes).await.map_err(|e| {
        tracing::error!(
            error = % e, key = % r_1, "PUT PDF nguồn lên kho lỗi"
        );
        ApiError::from(anyhow::anyhow!("đẩy PDF lên kho lỗi: {e}"))
    })?;
    tracing::info!(
        key = % r_1, bytes = bytes.len(),
        "PUT PDF nguồn lên kho (worker OCR fetch được)"
    );
    Ok(())
}
async fn f_2nl(
    State(state): State<AppState>,
    r_ef: Ts,
    Query(r_eo): Query<Tpe>,
    body: Body,
) -> Result<(StatusCode, Json<Tpf>), ApiError> {
    r_ef.require_perm(Permission::UploadExecute)?;
    authorize_project(&state, &r_ef, r_eo.project_id).await?;
    if !valid_blake3(&r_eo.blake3) {
        return Err(ApiError::bad_request("blake3 không hợp lệ"));
    }
    let r_4 = f_29f(&state, &r_ef, r_eo.project_id, r_eo.on_behalf_of).await?;
    let batch_id = f_28s(&state, &r_ef, r_eo.batch_id).await?;
    let rel_norm = r_eo.rel_path.as_deref().and_then(f_zy);
    if let Some(existing) = documents::find_same_file(
        &state.pool,
        r_eo.project_id,
        &r_eo.blake3,
        &r_eo.filename,
        rel_norm.as_deref(),
    )
    .await?
    {
        return Ok((
            StatusCode::OK,
            Json(Tpf {
                id: existing.id,
                duplicate: true,
            }),
        ));
    }
    let r_el = state.upload_lock(r_eo.project_id, &r_eo.blake3);
    let _guard = r_el.lock().await;
    let part = part_path(&state, r_eo.project_id, &r_eo.blake3);
    if let Some(r_em) = part.parent() {
        tokio::fs::create_dir_all(r_em).await.map_err(|e| {
            ApiError::from(anyhow::anyhow!(
                "tạo thư mục tạm lỗi: {e}"
            ))
        })?;
    }
    let (size, r_ee) = {
        use http_body_util::BodyExt;
        use tokio::io::AsyncWriteExt;
        let mut r_ei = tokio::fs::File::create(&part)
            .await
            .map_err(|e| ApiError::from(anyhow::anyhow!("mở file tạm lỗi: {e}")))?;
        let mut r_ej = blake3::Hasher::new();
        let mut size: u64 = 0;
        let mut body = body;
        while let Some(frame) = body.frame().await {
            let frame = frame.map_err(|e| {
                ApiError::bad_request_retryable(format!("nhận body lỗi: {e}"))
            })?;
            let Ok(chunk) = frame.into_data() else {
                continue;
            };
            size += chunk.len() as u64;
            if size as usize > C_C7 {
                let _ = tokio::fs::remove_file(&part).await;
                return Err(ApiError::bad_request(
                    "file vượt quá giới hạn",
                ));
            }
            r_ej.update(&chunk);
            r_ei.write_all(&chunk)
                .await
                .map_err(|e| ApiError::from(anyhow::anyhow!("ghi file lỗi: {e}")))?;
        }
        r_ei.sync_all().await.ok();
        (size, r_ej.finalize().to_hex().to_string())
    };
    if size == 0 {
        let _ = tokio::fs::remove_file(&part).await;
        return Err(ApiError::bad_request("file rỗng"));
    }
    if r_ee != r_eo.blake3 {
        let _ = tokio::fs::remove_file(&part).await;
        return Err(ApiError::bad_request_retryable(
            "hash không khớp (file hỏng khi truyền) — gửi lại",
        ));
    }
    if let Some(existing) = documents::find_same_file(
        &state.pool,
        r_eo.project_id,
        &r_eo.blake3,
        &r_eo.filename,
        rel_norm.as_deref(),
    )
    .await?
    {
        let _ = tokio::fs::remove_file(&part).await;
        return Ok((
            StatusCode::OK,
            Json(Tpf {
                id: existing.id,
                duplicate: true,
            }),
        ));
    }
    let doc_id = Uuid::now_v7();
    let r_eg = state.data_dir.join(r_eo.project_id.to_string());
    tokio::fs::create_dir_all(&r_eg).await.map_err(|e| {
        ApiError::from(anyhow::anyhow!(
            "tạo thư mục lưu lỗi: {e}"
        ))
    })?;
    let r_en = r_eg.join(format!("{doc_id}.pdf"));
    tokio::fs::rename(&part, &r_en)
        .await
        .map_err(|e| ApiError::from(anyhow::anyhow!("chốt file lỗi: {e}")))?;
    let r_3 = format!("{}/{doc_id}.pdf", r_eo.project_id);
    if let Err(e) = f_24y(&state, &r_3, &r_en).await {
        let _ = tokio::fs::remove_file(&r_en).await;
        return Err(e);
    }
    let rel_path = r_eo.rel_path.as_deref().and_then(f_zy);
    let r_ek = documents::insert(
        &state.pool,
        Tdt {
            project_id: r_eo.project_id,
            uploaded_by: r_4,
            filename: &r_eo.filename,
            ho_so_ref: r_eo.ho_so_ref.as_deref(),
            page_count: r_eo.page_count,
            size_bytes: size as i64,
            blake3: &r_eo.blake3,
            storage_path: &r_3,
            batch_id,
            rel_path: rel_path.as_deref(),
        },
    )
    .await?;
    let (r_eh, duplicate) = match r_ek {
        Some(d) => {
            drop_local_copy(&state, &r_en).await;
            (d, false)
        }
        None => (
            f_28v(&state, r_eo.project_id, &r_eo.blake3, &r_eo.filename, rel_path.as_deref(), &r_en, &r_3).await?,
            true,
        ),
    };
    digitext_db::audit::append(
        &state.pool,
        digitext_db::audit::Tp {
            actor_id: Some(r_ef.user.id),
            device_id: r_ef.claims.device_id,
            action: "document.upload",
            target: Some(format!("document:{}", r_eh.id)),
            detail: Some(serde_json::json!({ "filename" : r_eo.filename, "bytes" : size })),
            ip: r_ef.ip(),
        },
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(Tpf {
            id: r_eh.id,
            duplicate,
        }),
    ))
}
const C_3T: i64 = 200;
const C_BM: i64 = 500;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tax {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tof {
    pub id: Uuid,
    pub filename: String,
    pub rel_path: Option<String>,
    pub ho_so_ref: Option<String>,
    pub page_count: Option<i32>,
    pub size_bytes: i64,
    pub status: String,
    pub project_id: Uuid,
    pub project_code: String,
    pub project_name: String,
    pub fields_status: Option<String>,
    pub review_status: String,
    pub ocr_pages: i64,
    pub field_pages: i64,
    pub pdfa_pages: Option<i32>,
    pub pdfa2b: Option<bool>,
}
async fn f_2m3(State(state): State<AppState>, r_ep: Ts) -> Result<Json<Vec<Tof>>, ApiError> {
    r_ep.require_perm(Permission::DocumentList)?;
    let r_eq = match f_29a(&state.pool, &r_ep.user).await? {
        Scope::All => documents::f_1ol(&state.pool, None).await?,
        Scope::Projects(ids) => documents::f_1ol(&state.pool, Some(&ids)).await?,
    };
    Ok(Json(
        r_eq.into_iter()
            .map(|r| Tof {
                id: r.id,
                filename: r.filename,
                rel_path: r.rel_path,
                ho_so_ref: r.ho_so_ref,
                page_count: r.page_count,
                size_bytes: r.size_bytes,
                status: r.status,
                project_id: r.project_id,
                project_code: r.project_code,
                project_name: r.project_name,
                fields_status: r.fields_status,
                review_status: r.review_status,
                ocr_pages: r.ocr_pages,
                field_pages: r.field_pages,
                pdfa_pages: r.pdfa_pages,
                pdfa2b: r.pdfa_2b,
            })
            .collect(),
    ))
}
async fn f_1nb(
    State(state): State<AppState>,
    r_er: Ts,
    Query(r_es): Query<Tax>,
) -> Result<Json<Vec<DocDto>>, ApiError> {
    r_er.require_perm(Permission::DocumentList)?;
    let limit = r_es.limit.unwrap_or(C_3T).clamp(1, C_BM);
    let offset = r_es.offset.unwrap_or(0).max(0);
    let r_et = match f_29a(&state.pool, &r_er.user).await? {
        Scope::All => documents::list_all(&state.pool, limit, offset).await?,
        Scope::Projects(ids) => {
            documents::list_for_projects(&state.pool, &ids, limit, offset).await?
        }
    };
    Ok(Json(r_et.into_iter().map(DocDto::from).collect()))
}
#[derive(Debug, Default, Deserialize)]
pub struct DownloadQuery {
    /// `pdfa` → trả bản PDF/A (lớp chữ) từ kho object; mặc định trả bản gốc đã tải lên.
    pub variant: Option<String>,
}
async fn f_173(
    State(state): State<AppState>,
    r_eu: Ts,
    Path(id): Path<Uuid>,
    Query(q): Query<DownloadQuery>,
) -> Result<Response, ApiError> {
    let r_ev = documents::get(&state.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let ctv_ok = !r_eu.has_perm(Permission::DocumentDownload)
        && r_eu.has_perm(Permission::WorkExecute)
        && digitext_db::work_packages::can_work_on(&state.pool, r_eu.user.id, id).await?;
    if !ctv_ok {
        r_eu.require_perm(Permission::DocumentDownload)?;
        if !f_29a(&state.pool, &r_eu.user)
            .await?
            .allows_project(r_ev.project_id)
        {
            return Err(ApiError::forbidden());
        }
    }
    if q.variant.as_deref() == Some("signed") {
        // Bản ĐÃ KÝ SỐ (PAdES) — chỉ có khi tài liệu đã qua bước ký (signed/archived).
        if !matches!(r_ev.status.as_str(), "signed" | "archived") {
            return Err(ApiError::bad_request("tài liệu chưa ký số — ký ở mục “Ký số” trước"));
        }
        let key = digitext_storage::f_2fc(r_ev.project_id, id);
        let store = state
            .store
            .as_ref()
            .ok_or_else(|| ApiError::service_unavailable("kho lưu trữ chưa cấu hình"))?;
        let bytes = store
            .get(&key)
            .await
            .map_err(|e| {
                tracing::error!(error = % e, key, "lỗi đọc bản đã ký số từ kho object");
                ApiError::service_unavailable("kho lưu trữ không sẵn sàng")
            })?
            .ok_or_else(ApiError::not_found)?;
        let n = bytes.len() as u64;
        return Ok((f_177(&r_ev.filename, Some(n)), Body::from(bytes)).into_response());
    }
    if q.variant.as_deref() == Some("pdfa") {
        let key = documents::pdfa_key(&state.pool, id)
            .await?
            .ok_or_else(|| ApiError::bad_request("tài liệu chưa có bản PDF/A — chạy “PDF 2 lớp” trước"))?;
        let store = state
            .store
            .as_ref()
            .ok_or_else(|| ApiError::service_unavailable("kho lưu trữ chưa cấu hình"))?;
        let bytes = store
            .get(&key)
            .await
            .map_err(|e| {
                tracing::error!(error = % e, key, "lỗi đọc bản PDF/A từ kho object");
                ApiError::service_unavailable("kho lưu trữ không sẵn sàng")
            })?
            .ok_or_else(ApiError::not_found)?;
        let n = bytes.len() as u64;
        return Ok((f_177(&r_ev.filename, Some(n)), Body::from(bytes)).into_response());
    }
    let r_ey = state.data_dir.join(&r_ev.storage_path);
    let file = match tokio::fs::File::open(&r_ey).await {
        Ok(r_ew) => r_ew,
        Err(_) => match tokio::fs::File::open(&r_ev.storage_path).await {
            Ok(f) => f,
            Err(_) => {
                // Không còn bản sao thô trên đĩa (từ 25/9/2026 không giữ nữa — MinIO là bản chính).
                let Some(store) = state.source_store.as_ref().or(state.store.as_ref()) else {
                    return Err(ApiError::not_found());
                };
                let bytes = store
                    .get(&r_ev.storage_path)
                    .await
                    .map_err(|e| {
                        tracing::error!(error = % e, key = % r_ev.storage_path, "lỗi đọc PDF gốc từ kho object");
                        ApiError::service_unavailable("kho lưu trữ không sẵn sàng")
                    })?
                    .ok_or_else(ApiError::not_found)?;
                let n = bytes.len() as u64;
                return Ok((f_177(&r_ev.filename, Some(n)), Body::from(bytes)).into_response());
            }
        },
    };
    let r_ex = file.metadata().await.ok().map(|m| m.len());
    let body = Body::from_stream(tokio_util::io::ReaderStream::new(file));
    Ok((f_177(&r_ev.filename, r_ex), body).into_response())
}
fn f_177(filename: &str, r_f0: Option<u64>) -> HeaderMap {
    let safe_name: String = filename
        .chars()
        .filter(|c| !matches!(c, '"' | '\\' | '\r' | '\n'))
        .collect();
    let mut r_ez = HeaderMap::new();
    r_ez.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/pdf"),
    );
    if let Ok(r_f1) = header::HeaderValue::try_from(format!("attachment; filename=\"{safe_name}\""))
    {
        r_ez.insert(header::CONTENT_DISPOSITION, r_f1);
    }
    if let Some(n) = r_f0 {
        r_ez.insert(header::CONTENT_LENGTH, header::HeaderValue::from(n));
    }
    r_ez
}
const C_1U: usize = 8 * 1024 * 1024;
fn valid_blake3(r_f2: &str) -> bool {
    r_f2.len() == 64 && r_f2.bytes().all(|b| b.is_ascii_hexdigit())
}
fn part_path(state: &AppState, project_id: Uuid, blake3: &str) -> PathBuf {
    state
        .data_dir
        .join("_tmp")
        .join(project_id.to_string())
        .join(format!("{blake3}.part"))
}
fn f_1ge(r_f3: &HeaderMap, name: &str) -> Option<u64> {
    r_f3.get(name)?.to_str().ok()?.parse().ok()
}
fn f_1w0(status: StatusCode, offset: u64) -> Response {
    Response::builder()
        .status(status)
        .header("Upload-Offset", offset.to_string())
        .body(Body::empty())
        .expect("response hợp lệ")
}
const C_8Z: &str = "upload-duplicate";
fn f_10s(id: Uuid, duplicate: bool) -> Response {
    let mut b = Response::builder()
        .status(StatusCode::OK)
        .header("Upload-Complete", "1")
        .header("Document-Id", id.to_string());
    if duplicate {
        b = b.header(C_8Z, "1");
    }
    b.body(Body::empty()).expect("response hợp lệ")
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thk {
    pub project_id: Uuid,
    pub blake3: String,
    /// Client mới gửi kèm tên + đường dẫn để "đã có" nghĩa là đúng TỆP này đã lên, không phải
    /// một tệp khác cùng nội dung. Client cũ không gửi → so theo nội dung như trước.
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub rel_path: Option<String>,
}
async fn f_1g9(
    State(state): State<AppState>,
    r_f4: Ts,
    Query(r_f6): Query<Thk>,
) -> Result<Response, ApiError> {
    r_f4.require_perm(Permission::UploadExecute)?;
    authorize_project(&state, &r_f4, r_f6.project_id).await?;
    if !valid_blake3(&r_f6.blake3) {
        return Err(ApiError::bad_request("blake3 không hợp lệ"));
    }
    let r_f5 = match r_f6.filename.as_deref() {
        Some(name) => {
            let rel_norm = r_f6.rel_path.as_deref().and_then(f_zy);
            documents::find_same_file(&state.pool, r_f6.project_id, &r_f6.blake3, name, rel_norm.as_deref())
                .await?
        }
        None => documents::find_by_hash(&state.pool, r_f6.project_id, &r_f6.blake3).await?,
    };
    if let Some(r_f5) = r_f5 {
        return Ok(f_10s(r_f5.id, true));
    }
    let offset = tokio::fs::metadata(part_path(&state, r_f6.project_id, &r_f6.blake3))
        .await
        .map(|m| m.len())
        .unwrap_or(0);
    Ok(f_1w0(StatusCode::OK, offset))
}
async fn f_20h(
    State(state): State<AppState>,
    r_f8: Ts,
    Query(r_fi): Query<Tpe>,
    r_fe: HeaderMap,
    bytes: Bytes,
) -> Result<Response, ApiError> {
    r_f8.require_perm(Permission::UploadExecute)?;
    authorize_project(&state, &r_f8, r_fi.project_id).await?;
    if !valid_blake3(&r_fi.blake3) {
        return Err(ApiError::bad_request("blake3 không hợp lệ"));
    }
    let r_6 = f_29f(&state, &r_f8, r_fi.project_id, r_fi.on_behalf_of).await?;
    let offset = f_1ge(&r_fe, "upload-offset")
        .ok_or_else(|| ApiError::bad_request("thiếu Upload-Offset"))?;
    let r_fk = f_1ge(&r_fe, "upload-length")
        .ok_or_else(|| ApiError::bad_request("thiếu Upload-Length"))?;
    if r_fk == 0 || r_fk > MAX_RESUMABLE_BYTES {
        return Err(ApiError::bad_request(
            "kích thước file không hợp lệ",
        ));
    }
    if offset == 0 {
        f_28s(&state, &r_f8, r_fi.batch_id).await?;
    }
    let rel_norm = r_fi.rel_path.as_deref().and_then(f_zy);
    if let Some(existing) = documents::find_same_file(
        &state.pool,
        r_fi.project_id,
        &r_fi.blake3,
        &r_fi.filename,
        rel_norm.as_deref(),
    )
    .await?
    {
        let lock = state.upload_lock(r_fi.project_id, &r_fi.blake3);
        let _guard = lock.lock().await;
        let part = part_path(&state, r_fi.project_id, &r_fi.blake3);
        if tokio::fs::remove_file(&part).await.is_ok() {
            tracing::debug!(
                key = % format!("{}/{}", r_fi.project_id, r_fi.blake3),
                "thu .part sót lại vì tài liệu đã có trong kho"
            );
        }
        return Ok(f_10s(existing.id, true));
    }
    let lock = state.upload_lock(r_fi.project_id, &r_fi.blake3);
    let _guard = lock.lock().await;
    let part = part_path(&state, r_fi.project_id, &r_fi.blake3);
    if let Some(r_fh) = part.parent() {
        tokio::fs::create_dir_all(r_fh).await.map_err(|e| {
            ApiError::from(anyhow::anyhow!(
                "tạo thư mục tạm lỗi: {e}"
            ))
        })?;
    }
    let r_f9 = tokio::fs::metadata(&part)
        .await
        .map(|m| m.len())
        .unwrap_or(0);
    if offset != r_f9 {
        return Ok(f_1w0(StatusCode::CONFLICT, r_f9));
    }
    if r_f9 + bytes.len() as u64 > r_fk {
        return Err(ApiError::bad_request(
            "chunk vượt quá tổng kích thước",
        ));
    }
    {
        use tokio::io::AsyncWriteExt;
        let mut r_fc = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&part)
            .await
            .map_err(|e| ApiError::from(anyhow::anyhow!("mở file tạm lỗi: {e}")))?;
        r_fc.write_all(&bytes)
            .await
            .map_err(|e| ApiError::from(anyhow::anyhow!("ghi chunk lỗi: {e}")))?;
        r_fc.sync_all().await.ok();
    }
    let r_fg = r_f9 + bytes.len() as u64;
    if r_fg < r_fk {
        return Ok(f_1w0(StatusCode::OK, r_fg));
    }
    let r_f7 = f_us(&part).await?;
    if r_f7 != r_fi.blake3 {
        let _ = tokio::fs::remove_file(&part).await;
        return Err(
            ApiError::bad_request_retryable(
                "hash không khớp sau khi nhận đủ (dữ liệu hỏng) — gửi lại",
            ),
        );
    }
    if let Some(existing) = documents::find_same_file(
        &state.pool,
        r_fi.project_id,
        &r_fi.blake3,
        &r_fi.filename,
        rel_norm.as_deref(),
    )
    .await?
    {
        let _ = tokio::fs::remove_file(&part).await;
        return Ok(f_10s(existing.id, true));
    }
    let doc_id = Uuid::now_v7();
    let r_fa = state.data_dir.join(r_fi.project_id.to_string());
    tokio::fs::create_dir_all(&r_fa).await.map_err(|e| {
        ApiError::from(anyhow::anyhow!(
            "tạo thư mục lưu lỗi: {e}"
        ))
    })?;
    let r_fd = r_fa.join(format!("{doc_id}.pdf"));
    tokio::fs::rename(&part, &r_fd)
        .await
        .map_err(|e| ApiError::from(anyhow::anyhow!("chốt file lỗi: {e}")))?;
    let r_5 = format!("{}/{doc_id}.pdf", r_fi.project_id);
    if let Err(e) = f_24y(&state, &r_5, &r_fd).await {
        let _ = tokio::fs::remove_file(&r_fd).await;
        return Err(e);
    }
    let batch_id = f_28s(&state, &r_f8, r_fi.batch_id).await?;
    let rel_path = r_fi.rel_path.as_deref().and_then(f_zy);
    let r_ff = documents::insert(
        &state.pool,
        Tdt {
            project_id: r_fi.project_id,
            uploaded_by: r_6,
            filename: &r_fi.filename,
            ho_so_ref: r_fi.ho_so_ref.as_deref(),
            page_count: r_fi.page_count,
            size_bytes: r_fk as i64,
            blake3: &r_fi.blake3,
            storage_path: &r_5,
            batch_id,
            rel_path: rel_path.as_deref(),
        },
    )
    .await?;
    let (r_fb, duplicate) = match r_ff {
        Some(d) => {
            drop_local_copy(&state, &r_fd).await;
            (d, false)
        }
        None => (
            f_28v(&state, r_fi.project_id, &r_fi.blake3, &r_fi.filename, rel_path.as_deref(), &r_fd, &r_5).await?,
            true,
        ),
    };
    digitext_db::audit::append(
        &state.pool,
        digitext_db::audit::Tp {
            actor_id: Some(r_f8.user.id),
            device_id: r_f8.claims.device_id,
            action: "document.upload.resumable",
            target: Some(format!("document:{}", r_fb.id)),
            detail: Some(serde_json::json!({ "filename" : r_fi.filename, "bytes" : r_fk })),
            ip: r_f8.ip(),
        },
    )
    .await?;
    let mut r_fj = (
        StatusCode::CREATED,
        Json(Tpf {
            id: r_fb.id,
            duplicate,
        }),
    )
        .into_response();
    if duplicate {
        r_fj.headers_mut().insert(
            axum::http::HeaderName::from_static(C_8Z),
            axum::http::HeaderValue::from_static("1"),
        );
    }
    Ok(r_fj)
}
const C_BU: std::time::Duration = std::time::Duration::from_secs(24 * 3600);
pub async fn f_2id(r_fl: &std::path::Path) {
    f_2ic(r_fl, C_BU).await
}
async fn f_2ic(r_fm: &std::path::Path, r_fp: std::time::Duration) {
    let r_fv = r_fm.join("_tmp");
    let r_fq = std::time::SystemTime::now();
    let mut r_ft = 0u32;
    let Ok(mut projects) = tokio::fs::read_dir(&r_fv).await else {
        return;
    };
    while let Ok(Some(r_fs)) = projects.next_entry().await {
        let Ok(mut r_fo) = tokio::fs::read_dir(r_fs.path()).await else {
            continue;
        };
        while let Ok(Some(r_fn)) = r_fo.next_entry().await {
            let r_fr = r_fn.path();
            if r_fr.extension().and_then(|e| e.to_str()) != Some("part") {
                continue;
            }
            let r_fw = r_fn
                .metadata()
                .await
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|r_fu| r_fq.duration_since(r_fu).ok())
                .map(|age| age > r_fp)
                .unwrap_or(false);
            if r_fw && tokio::fs::remove_file(&r_fr).await.is_ok() {
                r_ft += 1;
            }
        }
    }
    if r_ft > 0 {
        tracing::info!(
            r_ft,
            ?r_fp,
            "dọn .part upload dở quá hạn"
        );
    }
}
async fn f_us(r_g3: &std::path::Path) -> Result<String, ApiError> {
    use tokio::io::AsyncReadExt;
    let mut r_g1 = tokio::fs::File::open(r_g3)
        .await
        .map_err(|e| ApiError::from(anyhow::anyhow!("mở file verify lỗi: {e}")))?;
    let mut r_g2 = blake3::Hasher::new();
    let mut r_g0 = vec![0u8; 1 << 20];
    loop {
        let n = r_g1.read(&mut r_g0).await.map_err(|e| {
            ApiError::from(anyhow::anyhow!("đọc file verify lỗi: {e}"))
        })?;
        if n == 0 {
            break;
        }
        r_g2.update(&r_g0[..n]);
    }
    Ok(r_g2.finalize().to_hex().to_string())
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tfa {
    #[serde(default)]
    pub project_id: Option<Uuid>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrAllResult {
    pub documents: usize,
    pub pages: i64,
    pub skipped_no_page_count: usize,
}
/// Trần tài liệu mỗi lần «OCR toàn bộ» (chỉ chặn bất thường; xếp theo lô nên 30.000+ vẫn vài giây).
const C_EE: i64 = 500_000;
async fn ocr_all(
    State(state): State<AppState>,
    r_g4: Ts,
    Json(r_g8): Json<Tfa>,
) -> Result<Json<OcrAllResult>, ApiError> {
    r_g4.require_perm(Permission::DocumentList)?;
    let projects: Option<Vec<Uuid>> = match r_g8.project_id {
        Some(r_g7) => {
            authorize_project(&state, &r_g4, r_g7).await?;
            Some(vec![r_g7])
        }
        None => match f_29a(&state.pool, &r_g4.user).await? {
            Scope::All => None,
            Scope::Projects(ids) => Some(ids),
        },
    };
    let r_g9 = documents::f_1o0(&state.pool, projects.as_deref(), C_EE).await?;
    let batch: Vec<(Uuid, i32)> = r_g9
        .iter()
        .filter_map(|r| r.page_count.filter(|n| *n > 0).map(|n| (r.id, n)))
        .collect();
    let r_g5 = batch.len();
    let skipped = r_g9.len() - r_g5;
    let pages = ocr_jobs::enqueue_ocr_bulk(&state.pool, &batch).await?;
    digitext_db::audit::append(
        &state.pool,
        digitext_db::audit::Tp {
            actor_id: Some(r_g4.user.id),
            device_id: r_g4.claims.device_id,
            action: "document.ocr_all",
            target: None,
            detail: Some(serde_json::json!(
                { "documents" : r_g5, "pages" : pages, "skipped" : skipped,
                "projectId" : r_g8.project_id, }
            )),
            ip: r_g4.ip(),
        },
    )
    .await?;
    Ok(Json(OcrAllResult {
        documents: r_g5,
        pages,
        skipped_no_page_count: skipped,
    }))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tcw {
    #[serde(default)]
    pub project_id: Option<Uuid>,
    #[serde(default)]
    pub document_ids: Option<Vec<Uuid>>,
    /// Gửi lại cả tài liệu đã bị từ chối vĩnh viễn (vd. < 200 dpi) — mặc định KHÔNG, vì kết
    /// quả chắc chắn vẫn bị từ chối cho tới khi quét lại.
    #[serde(default)]
    pub retry_rejected: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MakePdf2AllResult {
    pub documents: usize,
    /// Bỏ qua vì đã bị từ chối vĩnh viễn trước đó (vd. bản quét < 200 dpi — cần quét lại).
    pub skipped_rejected: usize,
}
const C_BF: i64 = 5000;
const MAX_SCOPE_IDS: usize = 20_000;
fn scope_ids(ids: Option<&Vec<Uuid>>) -> Result<Option<&[Uuid]>, ApiError> {
    match ids {
        None => Ok(None),
        Some(v) if v.len() > MAX_SCOPE_IDS => {
            Err(
                ApiError::bad_request(
                    format!(
                        "phạm vi quá lớn ({} tài liệu, trần {MAX_SCOPE_IDS}) — thu hẹp bộ lọc rồi bấm lại",
                        v.len()
                    ),
                ),
            )
        }
        Some(v) => Ok(Some(v.as_slice())),
    }
}
async fn make_pdf2_all(
    State(state): State<AppState>,
    r_ga: Ts,
    Json(r_gd): Json<Tcw>,
) -> Result<Json<MakePdf2AllResult>, ApiError> {
    r_ga.require_perm(Permission::DocumentList)?;
    let projects: Option<Vec<Uuid>> = match r_gd.project_id {
        Some(r_gc) => {
            authorize_project(&state, &r_ga, r_gc).await?;
            Some(vec![r_gc])
        }
        None => match f_29a(&state.pool, &r_ga.user).await? {
            Scope::All => None,
            Scope::Projects(ids) => Some(ids),
        },
    };
    let docs = scope_ids(r_gd.document_ids.as_ref())?;
    let r_ge = documents::f_1n9(
        &state.pool,
        DocStatus::OcrDone.as_str(),
        projects.as_deref(),
        docs,
        C_BF,
    )
    .await?;
    let rejected: std::collections::HashSet<Uuid> = if r_gd.retry_rejected {
        Default::default()
    } else {
        ocr_jobs::pdf2_rejected(&state.pool, projects.as_deref())
            .await?
            .into_iter()
            .map(|(id, _)| id)
            .collect()
    };
    let mut r_gb = 0usize;
    let mut skipped_rejected = 0usize;
    for r in &r_ge {
        if rejected.contains(&r.id) {
            skipped_rejected += 1;
            continue;
        }
        ocr_jobs::f_19r(&state.pool, r.id).await?;
        r_gb += 1;
    }
    digitext_db::audit::append(
        &state.pool,
        digitext_db::audit::Tp {
            actor_id: Some(r_ga.user.id),
            device_id: r_ga.claims.device_id,
            action: "document.make_pdf2_all",
            target: None,
            detail: Some(serde_json::json!(
                { "documents" : r_gb, "projectId" : r_gd.project_id,
                "scopedDocuments" : docs.map(< [Uuid] >::len), }
            )),
            ip: r_ga.ip(),
        },
    )
    .await?;
    Ok(Json(MakePdf2AllResult { documents: r_gb, skipped_rejected }))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrRetryFailedReq {
    #[serde(default)]
    pub project_id: Option<Uuid>,
    #[serde(default)]
    pub document_ids: Option<Vec<Uuid>>,
    /// Kèm cả trang `page_failed` (lỗi vĩnh viễn: ảnh hỏng, định dạng lạ). Mặc định
    /// chỉ chạy lại `failed_transient` — trang thất bại vì dịch vụ OCR chết.
    #[serde(default)]
    pub include_page_failed: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrRetryFailedResult {
    pub documents: u64,
    pub pages: u64,
    pub skipped_signed: u64,
}
/// Xếp hàng lại các trang OCR lỗi. "OCR toàn bộ" cố ý không đụng tới chúng (chỉ lấy
/// tài liệu chưa có job), nên đây là lối duy nhất ngoài sửa tay CSDL.
async fn ocr_retry_failed(
    State(state): State<AppState>,
    r_ga: Ts,
    Json(req): Json<OcrRetryFailedReq>,
) -> Result<Json<OcrRetryFailedResult>, ApiError> {
    r_ga.require_perm(Permission::DocumentList)?;
    let projects: Option<Vec<Uuid>> = match req.project_id {
        Some(p) => {
            authorize_project(&state, &r_ga, p).await?;
            Some(vec![p])
        }
        None => match f_29a(&state.pool, &r_ga.user).await? {
            Scope::All => None,
            Scope::Projects(ids) => Some(ids),
        },
    };
    let docs = scope_ids(req.document_ids.as_ref())?;
    let r = ocr_jobs::requeue_failed(
        &state.pool,
        projects.as_deref(),
        docs,
        req.include_page_failed,
    )
    .await?;
    digitext_db::audit::append(
        &state.pool,
        digitext_db::audit::Tp {
            actor_id: Some(r_ga.user.id),
            device_id: r_ga.claims.device_id,
            action: "document.ocr_retry_failed",
            target: None,
            detail: Some(serde_json::json!(
                { "documents" : r.documents, "pages" : r.pages, "skippedSigned" : r.skipped_signed,
                "includePageFailed" : req.include_page_failed, "projectId" : req.project_id,
                "scopedDocuments" : docs.map(< [Uuid] >::len), }
            )),
            ip: r_ga.ip(),
        },
    )
    .await?;
    Ok(Json(OcrRetryFailedResult {
        documents: r.documents,
        pages: r.pages,
        skipped_signed: r.skipped_signed,
    }))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T74 {
    #[serde(default)]
    pub project_id: Option<Uuid>,
    #[serde(default)]
    pub retry_failed: bool,
    #[serde(default)]
    pub document_ids: Option<Vec<Uuid>>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractFieldsAllResult {
    pub documents: usize,
    pub skipped_no_page_count: i64,
    pub skipped_page_failed: i64,
}
const C_6P: i64 = 5000;
async fn extract_fields_all(
    State(state): State<AppState>,
    r_gf: Ts,
    Json(r_gi): Json<T74>,
) -> Result<Json<ExtractFieldsAllResult>, ApiError> {
    r_gf.require_perm(Permission::DocumentList)?;
    let projects: Option<Vec<Uuid>> = match r_gi.project_id {
        Some(r_gh) => {
            authorize_project(&state, &r_gf, r_gh).await?;
            Some(vec![r_gh])
        }
        None => match f_29a(&state.pool, &r_gf.user).await? {
            Scope::All => None,
            Scope::Projects(ids) => Some(ids),
        },
    };
    let docs = scope_ids(r_gi.document_ids.as_ref())?;
    let r_gj = documents::f_1ny(
        &state.pool,
        projects.as_deref(),
        docs,
        C_6P,
        r_gi.retry_failed,
    )
    .await?;
    let mut r_gg = 0usize;
    for r in &r_gj {
        ocr_jobs::f_19p(&state.pool, r.id).await?;
        r_gg += 1;
    }
    let skipped = documents::f_121(&state.pool, projects.as_deref(), docs).await?;
    let skipped_page_failed = if r_gi.retry_failed {
        0
    } else {
        ocr_jobs::f_11y(&state.pool, "extract_fields", projects.as_deref(), docs).await?
    };
    digitext_db::audit::append(
        &state.pool,
        digitext_db::audit::Tp {
            actor_id: Some(r_gf.user.id),
            device_id: r_gf.claims.device_id,
            action: "document.extract_fields_all",
            target: None,
            detail: Some(serde_json::json!(
                { "documents" : r_gg, "skippedNoPageCount" : skipped,
                "skippedPageFailed" : skipped_page_failed, "retryFailed" : r_gi
                .retry_failed, "projectId" : r_gi.project_id, "scopedDocuments" :
                docs.map(< [Uuid] >::len), }
            )),
            ip: r_gf.ip(),
        },
    )
    .await?;
    Ok(Json(ExtractFieldsAllResult {
        documents: r_gg,
        skipped_no_page_count: skipped,
        skipped_page_failed,
    }))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RebuildResult {
    pub checked: usize,
    pub enqueued: usize,
    pub skipped_signed: usize,
}
fn f_264(status: &str) -> bool {
    matches!(status, "ocr_done" | "to_sign")
}
async fn rebuild_non_pdfa(
    State(state): State<AppState>,
    r_gk: Ts,
    Json(r_gm): Json<Tfa>,
) -> Result<Json<RebuildResult>, ApiError> {
    r_gk.require_perm(Permission::DocumentList)?;
    let r_8 = state
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let projects: Option<Vec<Uuid>> = match r_gm.project_id {
        Some(r_gl) => {
            authorize_project(&state, &r_gk, r_gl).await?;
            Some(vec![r_gl])
        }
        None => match f_29a(&state.pool, &r_gk.user).await? {
            Scope::All => None,
            Scope::Projects(ids) => Some(ids),
        },
    };
    let r_gn = match &projects {
        None => documents::list_all(&state.pool, C_FV, 0).await?,
        Some(ids) => documents::list_for_projects(&state.pool, ids, C_FV, 0).await?,
    };
    let (mut checked, mut enqueued, mut skipped_signed) = (0usize, 0usize, 0usize);
    for r in r_gn {
        if !f_1fz(&r.status) {
            continue;
        }
        let r_7 = digitext_storage::f_2qs(r.project_id, r.id);
        let Some(bytes) = r_8.get(&r_7).await.map_err(storage_err)? else {
            continue;
        };
        checked += 1;
        if digitext_pdf::pdfa::f_1rh(&bytes).is_pdfa2b() {
            continue;
        }
        if f_264(&r.status) {
            ocr_jobs::f_19r(&state.pool, r.id).await?;
            enqueued += 1;
        } else {
            skipped_signed += 1;
        }
    }
    digitext_db::audit::append(
        &state.pool,
        digitext_db::audit::Tp {
            actor_id: Some(r_gk.user.id),
            device_id: r_gk.claims.device_id,
            action: "document.rebuild_non_pdfa",
            target: None,
            detail: Some(serde_json::json!(
                { "checked" : checked, "enqueued" : enqueued, "skippedSigned" :
                skipped_signed, "projectId" : r_gm.project_id, }
            )),
            ip: r_gk.ip(),
        },
    )
    .await?;
    Ok(Json(RebuildResult {
        checked,
        enqueued,
        skipped_signed,
    }))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressDto {
    pub docs: i64,
    pub pages_total: i64,
    pub pages_ocr_done: i64,
    pub docs_ocr_pending: i64,
    pub docs_ocr_done: i64,
    pub docs_pdf2_done: i64,
    pub docs_fields_done: i64,
    pub docs_fields_active: i64,
    pub docs_fields_failed: i64,
    /// Trang OCR lỗi (tạm thời hết lượt + vĩnh viễn). Trước đây không có số nào
    /// cho lỗi OCR ở cấp trang nên 35 trang thất bại hôm 2026-09-17 không ai thấy.
    pub pages_ocr_failed: i64,
    pub pages_ocr_transient_failed: i64,
    pub pages_ocr_queued: i64,
    pub pages_ocr_running: i64,
    pub docs_ocr_failed: i64,
    /// Chốt "đã bắt hết chữ chưa": số trang dịch vụ OCR đã đo độ phủ mực, và số trang có độ
    /// phủ dưới ngưỡng (còn chữ chưa có khung → chưa vào lớp text PDF/A).
    pub pages_ink_measured: i64,
    pub pages_ink_low: i64,
    /// OCR xong nhưng ghép PDF 2 lớp bị từ chối vĩnh viễn (vd. < 200 dpi) — cần quét lại.
    pub docs_pdf2_rejected: i64,
    pub by_status: std::collections::HashMap<String, i64>,
}
/// Cùng biến môi trường + mặc định với worker (`digitext-ocr/src/config.rs`).
fn min_ink_coverage_threshold() -> f32 {
    std::env::var("DIGITEXT_REVIEW_MIN_INK_COVERAGE")
        .ok()
        .and_then(|v| v.trim().parse::<f32>().ok())
        .filter(|v| (0.0..=1.0).contains(v))
        .unwrap_or(0.90)
}
async fn progress(State(state): State<AppState>, r_go: Ts) -> Result<Json<ProgressDto>, ApiError> {
    r_go.require_perm(Permission::DocumentList)?;
    let projects: Option<Vec<Uuid>> = match f_29a(&state.pool, &r_go.user).await? {
        Scope::All => None,
        Scope::Projects(ids) => Some(ids),
    };
    let counts = documents::status_counts(&state.pool, projects.as_deref()).await?;
    let pages_ocr_done = digitext_db::ocr_results::f_1y3(&state.pool, projects.as_deref()).await?;
    let r_gq = ocr_jobs::f_1ax(&state.pool, projects.as_deref()).await?;
    let r_gs = ocr_jobs::ocr_failed_counts(&state.pool, projects.as_deref()).await?;
    let (pages_ink_measured, pages_ink_low) = digitext_db::ocr_results::ink_coverage_counts(
        &state.pool,
        projects.as_deref(),
        min_ink_coverage_threshold(),
    )
    .await?;
    let mut r_gp = ProgressDto {
        docs: 0,
        pages_total: 0,
        pages_ocr_done,
        docs_ocr_pending: 0,
        docs_ocr_done: 0,
        docs_pdf2_done: 0,
        docs_fields_done: r_gq.done,
        docs_fields_active: r_gq.active,
        docs_fields_failed: r_gq.failed,
        pages_ocr_failed: r_gs.pages_failed_transient + r_gs.pages_page_failed,
        pages_ocr_transient_failed: r_gs.pages_failed_transient,
        pages_ocr_queued: r_gs.pages_queued,
        pages_ocr_running: r_gs.pages_running,
        docs_ocr_failed: r_gs.docs_failed,
        pages_ink_measured,
        pages_ink_low,
        docs_pdf2_rejected: ocr_jobs::pdf2_rejected(&state.pool, projects.as_deref()).await?.len() as i64,
        by_status: std::collections::HashMap::new(),
    };
    for (status, docs, pages) in counts {
        r_gp.by_status.insert(status.clone(), docs);
        let Ok(r_gr) = status.parse::<DocStatus>() else {
            continue;
        };
        if r_gr == DocStatus::Disposed {
            continue;
        }
        r_gp.docs += docs;
        r_gp.pages_total += pages;
        if matches!(r_gr, DocStatus::Uploaded | DocStatus::OcrPending) {
            r_gp.docs_ocr_pending += docs;
        }
        if r_gr.ocr_finished() {
            r_gp.docs_ocr_done += docs;
        }
        if r_gr.pdf2_built() {
            r_gp.docs_pdf2_done += docs;
        }
    }
    Ok(Json(r_gp))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfaRow {
    pub id: Uuid,
    pub filename: String,
    pub ho_so_ref: Option<String>,
    pub status: String,
    pub state: String,
    pub pages: usize,
    pub expected_pages: Option<i32>,
    pub has_text_layer: bool,
    pub has_image: bool,
    pub pages_with_text: usize,
    pub pages_without_text: usize,
    pub pdfa2b: bool,
    pub reason: Option<String>,
}
const C_FV: i64 = 1000;
fn f_1fz(status: &str) -> bool {
    matches!(
        status,
        "to_sign" | "signing" | "signed" | "sign_rejected" | "archived"
    )
}
fn storage_err(e: digitext_storage::StorageError) -> ApiError {
    tracing::error!(error = % e, "lỗi kho object (pdfa-report)");
    ApiError::service_unavailable("kho lưu trữ không sẵn sàng")
}
async fn f_21r(State(state): State<AppState>, r_gs: Ts) -> Result<Json<Vec<PdfaRow>>, ApiError> {
    r_gs.require_perm(Permission::DocumentList)?;
    let r_a = state.store.as_ref().ok_or_else(|| {
        ApiError::service_unavailable("kho lưu trữ chưa cấu hình")
    })?;
    let r_gv = match f_29a(&state.pool, &r_gs.user).await? {
        Scope::All => documents::list_all(&state.pool, C_FV, 0).await?,
        Scope::Projects(ids) => documents::list_for_projects(&state.pool, &ids, C_FV, 0).await?,
    };
    // Tài liệu ghép PDF 2 lớp bị từ chối vĩnh viễn (vd. < 200 dpi): báo đúng lý do + đường dẫn để
    // quét lại, thay vì "thiếu tệp PDF" chung chung.
    let rejected: std::collections::HashMap<Uuid, String> =
        ocr_jobs::pdf2_rejected(&state.pool, None).await?.into_iter().collect();
    let mut r_gt = Vec::with_capacity(r_gv.len());
    for r in r_gv {
        if !f_1fz(&r.status) {
            r_gt.push(PdfaRow {
                id: r.id,
                filename: r.filename,
                ho_so_ref: r.ho_so_ref,
                status: r.status,
                state: "pending".into(),
                pages: 0,
                expected_pages: r.page_count,
                has_text_layer: false,
                has_image: false,
                pages_with_text: 0,
                pages_without_text: 0,
                pdfa2b: false,
                reason: None,
            });
            continue;
        }
        let r_9 = digitext_storage::f_2qs(r.project_id, r.id);
        let r_gu = match r_a.get(&r_9).await.map_err(storage_err)? {
            None if rejected.contains_key(&r.id) => PdfaRow {
                id: r.id,
                filename: r.filename,
                ho_so_ref: r.ho_so_ref,
                status: r.status,
                state: "rejected".into(),
                pages: 0,
                expected_pages: r.page_count,
                has_text_layer: false,
                has_image: false,
                pages_with_text: 0,
                pages_without_text: 0,
                pdfa2b: false,
                reason: rejected.get(&r.id).cloned(),
            },
            None => PdfaRow {
                id: r.id,
                filename: r.filename,
                ho_so_ref: r.ho_so_ref,
                status: r.status,
                state: "missing".into(),
                pages: 0,
                expected_pages: r.page_count,
                has_text_layer: false,
                has_image: false,
                pages_with_text: 0,
                pages_without_text: 0,
                pdfa2b: false,
                reason: Some(
                    "chưa tìm thấy PDF 2 lớp trong kho lưu trữ".into(),
                ),
            },
            Some(bytes) => {
                let c = digitext_pdf::f_2p9(&bytes, r.page_count);
                PdfaRow {
                    id: r.id,
                    filename: r.filename,
                    ho_so_ref: r.ho_so_ref,
                    status: r.status,
                    state: if c.ok { "ok" } else { "invalid" }.into(),
                    pages: c.pages,
                    expected_pages: r.page_count,
                    has_text_layer: c.has_text_layer,
                    has_image: c.has_image,
                    pages_with_text: c.pages_with_text,
                    pages_without_text: c.pages_without_text.len(),
                    pdfa2b: digitext_pdf::pdfa::f_1rh(&bytes).is_pdfa2b(),
                    reason: c.reason,
                }
            }
        };
        r_gt.push(r_gu);
    }
    Ok(Json(r_gt))
}
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportCounts {
    pub docs: i64,
    pub pages: i64,
    pub ocr_done: i64,
    pub ocr_pages: i64,
    pub field_docs: i64,
    pub pdfa_docs: i64,
    pub pdfa_old: i64,
    pub approved: i64,
    pub rejected: i64,
    pub pending: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Thp {
    pub project_id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    #[serde(flatten)]
    pub counts: ReportCounts,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportDto {
    pub projects_count: i64,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub totals: ReportCounts,
    pub projects: Vec<Thp>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tiv {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}
fn f_1yf(raw: &str, r_gw: bool) -> Option<DateTime<Utc>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if let Ok(r_gx) = DateTime::parse_from_rfc3339(raw) {
        return Some(r_gx.with_timezone(&Utc));
    }
    let d = chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d").ok()?;
    let d = if r_gw { d.succ_opt()? } else { d };
    Some(DateTime::from_naive_utc_and_offset(
        d.and_hms_opt(0, 0, 0)?,
        Utc,
    ))
}
fn f_2a6(
    projects: Vec<digitext_db::projects::Project>,
    r_gz: Vec<digitext_db::documents::Tiu>,
) -> (Vec<Thp>, ReportCounts) {
    let mut r_h2: Vec<Thp> = projects
        .into_iter()
        .map(|r_h1| Thp {
            project_id: r_h1.id,
            code: r_h1.code,
            name: r_h1.name,
            is_active: r_h1.is_active,
            counts: ReportCounts::default(),
        })
        .collect();
    let r_h0: std::collections::HashMap<Uuid, usize> = r_h2
        .iter()
        .enumerate()
        .map(|(i, r)| (r.project_id, i))
        .collect();
    let mut totals = ReportCounts::default();
    for c in r_gz {
        if c.status.parse::<DocStatus>() == Ok(DocStatus::Disposed) {
            continue;
        }
        let review = c.review_status.parse::<digitext_core::ReviewStatus>().ok();
        let r_gy = |r_h3: &mut ReportCounts| {
            r_h3.docs += c.docs;
            r_h3.pages += c.pages;
            r_h3.ocr_done += c.docs_ocr_full;
            r_h3.ocr_pages += c.ocr_pages;
            r_h3.field_docs += c.docs_fields;
            r_h3.pdfa_docs += c.docs_pdfa;
            r_h3.pdfa_old += c.docs_pdfa_old;
            match review {
                Some(digitext_core::ReviewStatus::Approved) => r_h3.approved += c.docs,
                Some(digitext_core::ReviewStatus::Rejected) => r_h3.rejected += c.docs,
                _ => r_h3.pending += c.docs,
            }
        };
        r_gy(&mut totals);
        if let Some(i) = r_h0.get(&c.project_id) {
            r_gy(&mut r_h2[*i].counts);
        }
    }
    (r_h2, totals)
}
async fn report(
    State(state): State<AppState>,
    r_h4: Ts,
    Query(r_h6): Query<Tiv>,
) -> Result<Json<ReportDto>, ApiError> {
    r_h4.require_perm(Permission::DocumentList)?;
    let from = match r_h6
        .from
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        None => None,
        Some(s) => Some(f_1yf(s, false).ok_or_else(|| {
            ApiError::bad_request(
                "mốc 'from' không đọc được (YYYY-MM-DD)",
            )
        })?),
    };
    let to = match r_h6.to.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        None => None,
        Some(s) => Some(f_1yf(s, true).ok_or_else(|| {
            ApiError::bad_request(
                "mốc 'to' không đọc được (YYYY-MM-DD)",
            )
        })?),
    };
    if let (Some(a), Some(b)) = (from, to) {
        if a >= b {
            return Err(
                ApiError::bad_request(
                    "khoảng ngày rỗng: ngày bắt đầu phải trước ngày kết thúc",
                ),
            );
        }
    }
    let (projects, scope_ids) = match f_29a(&state.pool, &r_h4.user).await? {
        Scope::All => (digitext_db::projects::list(&state.pool).await?, None),
        Scope::Projects(ids) => (
            digitext_db::projects::f_1n7(&state.pool, &ids).await?,
            Some(ids),
        ),
    };
    let r_h5 = documents::f_281(&state.pool, scope_ids.as_deref(), from, to).await?;
    let (r_h7, totals) = f_2a6(projects, r_h5);
    Ok(Json(ReportDto {
        projects_count: r_h7.len() as i64,
        from,
        to,
        totals,
        projects: r_h7,
    }))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tj9 {
    pub decision: digitext_core::ReviewStatus,
    #[serde(default)]
    pub note: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResult {
    pub id: Uuid,
    pub review_status: String,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_note: Option<String>,
}
const C_BY: usize = 2000;
async fn f_29u(
    State(state): State<AppState>,
    r_h8: Ts,
    Path(id): Path<Uuid>,
    Json(r_ha): Json<Tj9>,
) -> Result<Json<ReviewResult>, ApiError> {
    r_h8.require_perm(Permission::DocumentReview)?;
    let r_h9 = documents::get(&state.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !f_29a(&state.pool, &r_h8.user)
        .await?
        .allows_project(r_h9.project_id)
    {
        return Err(ApiError::forbidden());
    }
    let note: Option<String> = r_ha
        .note
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(C_BY).collect());
    if r_ha.decision == digitext_core::ReviewStatus::Rejected && note.is_none() {
        return Err(ApiError::bad_request(
            "phải nêu lý do khi từ chối kết quả bóc tách",
        ));
    }
    let r_hb = documents::f_2ef(
        &state.pool,
        id,
        r_ha.decision,
        Some(r_h8.user.id),
        note.as_deref(),
    )
    .await?
    .ok_or_else(ApiError::not_found)?;
    digitext_db::audit::append(
        &state.pool,
        digitext_db::audit::Tp {
            actor_id: Some(r_h8.user.id),
            device_id: r_h8.claims.device_id,
            action: "document.review",
            target: Some(id.to_string()),
            detail: Some(serde_json::json!(
                { "decision" : r_ha.decision.as_str(), "projectId" : r_h9
                .project_id, "note" : note, }
            )),
            ip: r_h8.ip(),
        },
    )
    .await?;
    Ok(Json(ReviewResult {
        id,
        review_status: r_hb.review_status,
        reviewed_at: r_hb.reviewed_at,
        review_note: r_hb.review_note,
    }))
}
const MAX_REVIEW_BULK: usize = 500;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tj6 {
    pub ids: Vec<Uuid>,
    pub decision: digitext_core::ReviewStatus,
    #[serde(default)]
    pub note: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewBulkResult {
    pub updated: i64,
    pub skipped: i64,
    pub review_status: String,
}
async fn review_bulk(
    State(state): State<AppState>,
    r_hc: Ts,
    Json(r_he): Json<Tj6>,
) -> Result<Json<ReviewBulkResult>, ApiError> {
    r_hc.require_perm(Permission::DocumentReview)?;
    if r_he.ids.is_empty() {
        return Err(ApiError::bad_request(
            "chưa chọn tài liệu nào để duyệt",
        ));
    }
    if r_he.ids.len() > MAX_REVIEW_BULK {
        return Err(ApiError::bad_request(format!(
            "một lượt duyệt tối đa {MAX_REVIEW_BULK} tài liệu"
        )));
    }
    let ids: Vec<Uuid> = {
        let mut r_hf = std::collections::BTreeSet::new();
        r_he.ids
            .iter()
            .copied()
            .filter(|id| r_hf.insert(*id))
            .collect()
    };
    let note: Option<String> = r_he
        .note
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(C_BY).collect());
    if r_he.decision == digitext_core::ReviewStatus::Rejected && note.is_none() {
        return Err(ApiError::bad_request(
            "phải nêu lý do khi từ chối kết quả bóc tách",
        ));
    }
    let projects = match f_29a(&state.pool, &r_hc.user).await? {
        Scope::All => None,
        Scope::Projects(ids) => Some(ids),
    };
    let r_hd = documents::f_2eg(
        &state.pool,
        &ids,
        projects.as_deref(),
        r_he.decision,
        Some(r_hc.user.id),
        note.as_deref(),
    )
    .await?;
    digitext_db::audit::append(
        &state.pool,
        digitext_db::audit::Tp {
            actor_id: Some(r_hc.user.id),
            device_id: r_hc.claims.device_id,
            action: "document.review_bulk",
            target: None,
            detail: Some(serde_json::json!(
                { "decision" : r_he.decision.as_str(), "updated" : r_hd.len(),
                "skipped" : ids.len() - r_hd.len(), "ids" : r_hd, "note" : note,
                }
            )),
            ip: r_hc.ip(),
        },
    )
    .await?;
    Ok(Json(ReviewBulkResult {
        updated: r_hd.len() as i64,
        skipped: (ids.len() - r_hd.len()) as i64,
        review_status: r_he.decision.as_str().to_owned(),
    }))
}
