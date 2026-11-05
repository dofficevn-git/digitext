// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::Permission;
use digitext_db::packages::{self, Tdy, Tgc};
use digitext_package::wire::Tq4;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/packages", get(f_1o6).post(f_12b))
        .route("/packages/{id}", get(f_1es))
        .route("/packages/{id}/download", get(f_17c))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3b {
    pub project_id: Option<Uuid>,
    pub ho_so_ref: Option<String>,
    pub submission_id: Option<Uuid>,
    pub stt_from: Option<i32>,
    pub stt_to: Option<i32>,
    pub request: Tq4,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub kind: String,
    pub status: String,
    pub ho_so_ref: Option<String>,
    pub label: String,
    /// Mã và tên phông lấy từ yêu cầu đóng gói (`fondCode`/`fondName`) — bảng packages không
    /// có cột riêng. Có hai trường này thì màn Gói tin mới lọc và tải theo phông được.
    pub fond_code: Option<String>,
    pub fond_name: Option<String>,
    pub objid: Option<String>,
    pub zip_name: Option<String>,
    pub zip_sha256: Option<String>,
    pub zip_size: Option<i64>,
    pub document_count: i32,
    pub error: Option<String>,
    pub virus_scan: Option<String>,
    pub submission_id: Option<Uuid>,
    pub converted_from: Option<Uuid>,
    pub stt_from: Option<i32>,
    pub stt_to: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
fn chuoi_json(r_mf1: &serde_json::Value, r_mf2: &str) -> Option<String> {
    r_mf1
        .get(r_mf2)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}
impl From<digitext_db::packages::Tgd> for PackageDto {
    fn from(r: digitext_db::packages::Tgd) -> Self {
        PackageDto {
            id: r.id,
            project_id: r.project_id,
            kind: r.kind,
            status: r.status,
            ho_so_ref: r.ho_so_ref,
            label: r.label,
            fond_code: r.fond_code,
            fond_name: r.fond_name,
            objid: r.objid,
            zip_name: r.zip_name,
            zip_sha256: r.zip_sha256,
            zip_size: r.zip_size,
            document_count: r.document_count,
            error: r.error,
            virus_scan: r.virus_scan,
            submission_id: r.submission_id,
            converted_from: r.converted_from,
            stt_from: r.stt_from,
            stt_to: r.stt_to,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
impl From<Tgc> for PackageDto {
    fn from(r: Tgc) -> Self {
        PackageDto {
            id: r.id,
            project_id: r.project_id,
            kind: r.kind,
            status: r.status,
            ho_so_ref: r.ho_so_ref,
            label: r.label,
            fond_code: chuoi_json(&r.request_json, "fondCode"),
            fond_name: chuoi_json(&r.request_json, "fondName"),
            objid: r.objid,
            zip_name: r.zip_name,
            zip_sha256: r.zip_sha256,
            zip_size: r.zip_size,
            document_count: r.document_count,
            error: r.error,
            virus_scan: r.virus_scan,
            submission_id: r.submission_id,
            converted_from: r.converted_from,
            stt_from: r.stt_from,
            stt_to: r.stt_to,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
fn can_touch_project(r_mg: &Ts, r_mi: &Scope, r_mh: Uuid) -> bool {
    r_mg.can_touch_project(r_mi, r_mh)
}
async fn f_12b(
    State(r_mv): State<AppState>,
    r_mj: Ts,
    Json(r_mq): Json<T3b>,
) -> Result<(StatusCode, Json<PackageDto>), ApiError> {
    r_mj.require_perm(Permission::PackageCreate)?;
    let project_id = r_mq.project_id.or(r_mj.user.project_id).ok_or_else(|| {
        ApiError::bad_request(
            "thiếu projectId (người gọi không thuộc dự án nào)",
        )
    })?;
    let r_mt = f_29a(&r_mv.pool, &r_mj.user).await?;
    if !can_touch_project(&r_mj, &r_mt, project_id) {
        return Err(ApiError::forbidden());
    }
    let r_mu = r_mq.request.shape_errors();
    if !r_mu.is_empty() {
        return Err(ApiError::bad_request(format!(
            "yêu cầu đóng gói không hợp lệ: {}",
            r_mu.join("; ")
        )));
    }
    for r_ml in &r_mq.request.documents {
        if let Some(doc_id) = r_ml.document_id {
            let r_mm = digitext_db::documents::get(&r_mv.pool, doc_id)
                .await?
                .ok_or_else(|| {
                    ApiError::bad_request(format!(
                        "tài liệu {doc_id} không tồn tại"
                    ))
                })?;
            if r_mm.project_id != project_id {
                return Err(ApiError::bad_request(format!(
                    "tài liệu {doc_id} không thuộc dự án của gói"
                )));
            }
        }
    }
    let mut r_mx = match (r_mq.stt_from, r_mq.stt_to) {
        (None, None) => None,
        (Some(a), Some(b)) if a >= 1 && a <= b => Some((a, b)),
        _ => {
            return Err(
                ApiError::bad_request(
                    "sttFrom/sttTo phải đi thành cặp và 1 ≤ từ ≤ đến (PK-5, Điều 22.3.c)"
                        .to_string(),
                ),
            );
        }
    };
    if let Some(sid) = r_mq.submission_id {
        let r_my = digitext_db::submissions::get(&r_mv.pool, sid)
            .await?
            .ok_or_else(|| {
                ApiError::bad_request(format!("lần nộp {sid} không tồn tại"))
            })?;
        if r_my.project_id != project_id {
            return Err(ApiError::bad_request(
                "lần nộp không thuộc dự án của gói".to_string(),
            ));
        }
        if r_my.status != "confirmed" && r_my.status != "awaiting_reception" {
            return Err(
                ApiError::conflict(
                    format!(
                        "lần nộp chưa nhận gói (trạng thái: {}) — cần được xác nhận trước (Điều 21)",
                        r_my.status
                    ),
                ),
            );
        }
        if !r_mq.request.kind.starts_with("sip_") {
            return Err(
                ApiError::bad_request(
                    "chỉ gói SIP mới gắn được vào lần nộp (Điều 22.3.c)"
                        .to_string(),
                ),
            );
        }
        match r_mx {
            Some((a, b)) => {
                let r_mk = digitext_db::submissions::f_2hx(&r_mv.pool, sid).await?;
                let r_mp = r_mk.is_some_and(|(r_mo, r_mn)| a >= r_mo && b <= r_mn);
                if !r_mp {
                    return Err(
                        ApiError::bad_request(
                            format!(
                                "dải STT {a}–{b} nằm ngoài Mục lục của lần nộp (PK-5)"
                            ),
                        ),
                    );
                }
            }
            None => {
                if let digitext_package::wire::WireMeta::Hoso(h) = &r_mq.request.meta {
                    if let Some(r_mw) =
                        digitext_db::submissions::f_2hy(&r_mv.pool, sid, &h.file_code).await?
                    {
                        r_mx = Some((r_mw, r_mw));
                    }
                }
            }
        }
    }
    let r_mr = serde_json::to_value(&r_mq.request).map_err(|e| {
        ApiError::bad_request(format!(
            "request không serialize được: {e}"
        ))
    })?;
    let r_ms = packages::create(
        &r_mv.pool,
        Tdy {
            project_id,
            requested_by: Some(r_mj.user.id),
            kind: &r_mq.request.kind,
            ho_so_ref: r_mq.ho_so_ref.as_deref(),
            label: &r_mq.request.display_label(),
            request_json: &r_mr,
            submission_id: r_mq.submission_id,
            converted_from: None,
            stt_range: r_mx,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(r_ms.into())))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
// 5/11/2026: mot phong Bac Giang sinh hon 140 goi mot luot. Muc 50 cu lam man Goi tin
// chi thay 50 dong moi nhat (toan 'cho dung'), loc 'Xong' ra 0 goi du da dung xong.
const DEFAULT_LIMIT: i64 = 2000;
const MAX_LIMIT: i64 = 5000;
async fn f_1o6(
    State(r_n4): State<AppState>,
    r_mz: Ts,
    Query(r_n1): Query<ListQuery>,
) -> Result<Json<Vec<PackageDto>>, ApiError> {
    r_mz.require_perm(Permission::PackageView)?;
    let limit = r_n1.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = r_n1.offset.unwrap_or(0).max(0);
    let r_n3 = f_29a(&r_n4.pool, &r_mz.user).await?;
    let r_n0 = r_mz.list_projects(r_n3);
    let r_n2 = packages::list_gon(&r_n4.pool, r_n0.as_deref(), limit, offset).await?;
    Ok(Json(r_n2.into_iter().map(PackageDto::from).collect()))
}
async fn load_in_scope(r_n8: &AppState, r_n5: &Ts, id: Uuid) -> Result<Tgc, ApiError> {
    let r_n6 = packages::get(&r_n8.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let r_n7 = f_29a(&r_n8.pool, &r_n5.user).await?;
    if !can_touch_project(r_n5, &r_n7, r_n6.project_id) {
        return Err(ApiError::forbidden());
    }
    Ok(r_n6)
}
async fn f_1es(
    State(r_nb): State<AppState>,
    r_n9: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<PackageDto>, ApiError> {
    r_n9.require_perm(Permission::PackageView)?;
    let r_na = load_in_scope(&r_nb, &r_n9, id).await?;
    Ok(Json(r_na.into()))
}
#[derive(Debug, PartialEq, Eq)]
enum RangeParse {
    Full,
    Partial(u64, u64),
    Unsatisfiable,
}
fn f_1zq(header: Option<&str>, total: u64) -> RangeParse {
    let Some(r_nd) = header.and_then(|h| h.strip_prefix("bytes=")) else {
        return RangeParse::Full;
    };
    if r_nd.contains(',') {
        return RangeParse::Full;
    }
    let Some((a, b)) = r_nd.split_once('-') else {
        return RangeParse::Full;
    };
    match (a, b) {
        ("", n) => match n.parse::<u64>() {
            Ok(n) if n > 0 && total > 0 => RangeParse::Partial(total.saturating_sub(n), total - 1),
            Ok(_) => RangeParse::Unsatisfiable,
            Err(_) => RangeParse::Full,
        },
        (a, b) => {
            let Ok(r_ne) = a.parse::<u64>() else {
                return RangeParse::Full;
            };
            let r_nc = if b.is_empty() {
                total.saturating_sub(1)
            } else {
                match b.parse::<u64>() {
                    Ok(e) => e.min(total.saturating_sub(1)),
                    Err(_) => return RangeParse::Full,
                }
            };
            if r_ne >= total || r_nc < r_ne {
                RangeParse::Unsatisfiable
            } else {
                RangeParse::Partial(r_ne, r_nc)
            }
        }
    }
}
async fn f_17c(
    State(r_no): State<AppState>,
    r_nf: Ts,
    Path(id): Path<Uuid>,
    r_nh: HeaderMap,
) -> Result<Response, ApiError> {
    r_nf.require_perm(Permission::PackageDownload)?;
    let r_nm = load_in_scope(&r_no, &r_nf, id).await?;
    if r_nm.status != "built" {
        return Err(ApiError::conflict(format!(
            "gói chưa sẵn sàng (trạng thái: {})",
            r_nm.status
        )));
    }
    let (Some(r_ns), Some(zip_name)) = (r_nm.zip_key.as_deref(), r_nm.zip_name.as_deref()) else {
        return Err(ApiError::internal());
    };
    let r_np = r_no
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let etag = r_nm
        .zip_sha256
        .as_deref()
        .map(|h| format!("\"sha256:{h}\""));
    if let (Some(etag), Some(r_ni)) = (
        etag.as_deref(),
        r_nh.get(header::IF_NONE_MATCH)
            .and_then(|v| v.to_str().ok()),
    ) {
        if r_ni
            .split(',')
            .any(|r_nr| r_nr.trim() == etag || r_nr.trim() == "*")
        {
            let mut r_nl = StatusCode::NOT_MODIFIED.into_response();
            if let Ok(v) = etag.parse() {
                r_nl.headers_mut().insert(header::ETAG, v);
            }
            return Ok(r_nl);
        }
    }
    let total = r_nm.zip_size.unwrap_or(0).max(0) as u64;
    let r_nk = r_nh.get(header::RANGE).and_then(|v| v.to_str().ok());
    let r_nj = match f_1zq(r_nk, total) {
        RangeParse::Unsatisfiable => {
            return Ok((
                StatusCode::RANGE_NOT_SATISFIABLE,
                [(header::CONTENT_RANGE, format!("bytes */{total}"))],
            )
                .into_response());
        }
        RangeParse::Full => None,
        RangeParse::Partial(r_nn, e) => Some((r_nn, Some(e))),
    };
    let r_nq = r_np
        .get_stream(r_ns, r_nj)
        .await
        .map_err(|e| {
            tracing::error!(
                package = % id, error = % e,
                "đọc ZIP gói tin từ kho lỗi"
            );
            ApiError::internal()
        })?
        .ok_or_else(ApiError::not_found)?;
    let safe_name: String = zip_name
        .chars()
        .filter(|c| !matches!(c, '"' | '\\' | '\r' | '\n'))
        .collect();
    let mut r_ng = Response::builder()
        .status(if r_nj.is_some() {
            StatusCode::PARTIAL_CONTENT
        } else {
            StatusCode::OK
        })
        .header(header::CONTENT_TYPE, "application/zip")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{safe_name}\""),
        )
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, r_nq.len);
    if let Some(etag) = &etag {
        r_ng = r_ng.header(header::ETAG, etag);
    }
    if r_nj.is_some() {
        r_ng = r_ng.header(
            header::CONTENT_RANGE,
            format!(
                "bytes {}-{}/{}",
                r_nq.start,
                r_nq.start + r_nq.len.saturating_sub(1),
                r_nq.total_len
            ),
        );
    }
    r_ng.body(Body::from_stream(r_nq.body))
        .map_err(|_| ApiError::internal())
}
