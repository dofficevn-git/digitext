// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::bienban;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::Permission;
use digitext_db::records::{self, ListFilter, RecordRow, Tdz};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/records", get(f_1o9).post(f_12d))
        .route("/records/{id}", get(get_record))
        .route("/records/{id}/download", get(f_17e))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3g {
    pub project_id: Option<Uuid>,
    pub kind: String,
    pub payload: serde_json::Value,
    pub submission_id: Option<Uuid>,
    /// Khoá phụ (0024): mã phông / ngày / lần bàn giao.
    #[serde(default, rename = "ref")]
    pub ref_: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub kind: String,
    pub so_bien_ban: String,
    pub title: String,
    pub submission_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    #[serde(rename = "ref")]
    pub ref_: Option<String>,
}
impl From<RecordRow> for RecordDto {
    fn from(r_pa: RecordRow) -> Self {
        RecordDto {
            id: r_pa.id,
            project_id: r_pa.project_id,
            kind: r_pa.kind,
            so_bien_ban: r_pa.so_bien_ban,
            title: r_pa.title,
            submission_id: r_pa.submission_id,
            created_at: r_pa.created_at,
            ref_: r_pa.ref_,
        }
    }
}
/// Chi tiết một biên bản kèm payload (xem lại, nhân bản, tính tổng ở máy trạm).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDetailDto {
    #[serde(flatten)]
    pub row: RecordDto,
    pub payload: serde_json::Value,
}
fn can_touch_project(r_pb: &Ts, r_pd: &Scope, r_pc: Uuid) -> bool {
    r_pb.can_touch_project(r_pd, r_pc)
}
fn f_26d(project_id: Uuid, record_id: Uuid) -> String {
    format!("{project_id}-records/{record_id}.pdf")
}
pub async fn f_12e(
    r_pj: &AppState,
    r_pe: &Ts,
    project_id: Uuid,
    kind: &str,
    payload: &serde_json::Value,
    submission_id: Option<Uuid>,
) -> Result<RecordDto, ApiError> {
    f_12e_ref(r_pj, r_pe, project_id, kind, payload, submission_id, None).await
}
pub async fn f_12e_ref(
    r_pj: &AppState,
    r_pe: &Ts,
    project_id: Uuid,
    kind: &str,
    payload: &serde_json::Value,
    submission_id: Option<Uuid>,
    ref_: Option<&str>,
) -> Result<RecordDto, ApiError> {
    let (title, r_pf) = bienban::render(kind, payload).map_err(ApiError::bad_request)?;
    let r_pk = r_pj
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let r_pg = digitext_pdf::f_1cq()
        .ok_or_else(|| {
            ApiError::service_unavailable(
                "không tìm thấy font TTF cho biên bản — đặt DIGITEXT_CERT_FONT",
            )
        })?;
    let font = std::fs::read(&r_pg).map_err(|e| {
        tracing::error!(
            path = % r_pg.display(), error = % e,
            "đọc font biên bản lỗi"
        );
        ApiError::internal()
    })?;
    let pdf =
        digitext_pdf::f_ve(&title, Some(bienban::f_2i2(kind)), &r_pf, &font).map_err(|e| {
            ApiError::bad_request(format!("sinh PDF biên bản lỗi: {e}"))
        })?;
    let record_id = Uuid::now_v7();
    let r_ph = f_26d(project_id, record_id);
    r_pk.put(&r_ph, &pdf).await.map_err(|e| {
        tracing::error!(
            error = % e, "PUT PDF biên bản vào kho lỗi"
        );
        ApiError::internal()
    })?;
    let r_pi = records::create(
        &r_pj.pool,
        Tdz {
            project_id,
            created_by: Some(r_pe.user.id),
            kind,
            title: &title,
            submission_id,
            payload,
            pdf_key: &r_ph,
            ref_: ref_.map(str::trim).filter(|s| !s.is_empty()),
        },
    )
    .await?;
    Ok(r_pi.into())
}
async fn f_12d(
    State(r_pp): State<AppState>,
    r_pl: Ts,
    Json(r_pn): Json<T3g>,
) -> Result<(StatusCode, Json<RecordDto>), ApiError> {
    r_pl.require_perm(Permission::RecordCreate)?;
    let project_id = r_pn.project_id.or(r_pl.user.project_id).ok_or_else(|| {
        ApiError::bad_request(
            "thiếu projectId (người gọi không thuộc dự án nào)",
        )
    })?;
    let r_po = f_29a(&r_pp.pool, &r_pl.user).await?;
    if !can_touch_project(&r_pl, &r_po, project_id) {
        return Err(ApiError::forbidden());
    }
    let r_pm = f_12e_ref(
        &r_pp,
        &r_pl,
        project_id,
        &r_pn.kind,
        &r_pn.payload,
        r_pn.submission_id,
        r_pn.ref_.as_deref(),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(r_pm)))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    /// Lọc (0024): `kind` = một hay nhiều loại cách nhau bằng dấu phẩy; `ref` = đúng khoá phụ; `projectId` = một dự án.
    pub kind: Option<String>,
    #[serde(rename = "ref")]
    pub ref_: Option<String>,
    pub project_id: Option<Uuid>,
}
async fn f_1o9(
    State(r_pv): State<AppState>,
    r_pq: Ts,
    Query(r_ps): Query<ListQuery>,
) -> Result<Json<Vec<RecordDto>>, ApiError> {
    r_pq.require_perm(Permission::RecordView)?;
    let limit = r_ps.limit.unwrap_or(50).clamp(1, 200);
    let offset = r_ps.offset.unwrap_or(0).max(0);
    let r_pu = f_29a(&r_pv.pool, &r_pq.user).await?;
    if let Some(p) = r_ps.project_id {
        if !can_touch_project(&r_pq, &r_pu, p) {
            return Err(ApiError::forbidden());
        }
    }
    let r_pr = r_pq.list_projects(r_pu);
    let kinds: Vec<String> = r_ps
        .kind
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(str::to_string)
        .collect();
    let f = ListFilter { kinds: &kinds, ref_: r_ps.ref_.as_deref().map(str::trim).filter(|s| !s.is_empty()), project: r_ps.project_id };
    let r_pt = records::list_filtered(&r_pv.pool, r_pr.as_deref(), &f, limit, offset).await?;
    Ok(Json(r_pt.into_iter().map(RecordDto::from).collect()))
}
async fn get_record(
    State(st): State<AppState>,
    ts: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<RecordDetailDto>, ApiError> {
    ts.require_perm(Permission::RecordView)?;
    let row = records::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    let scope = f_29a(&st.pool, &ts.user).await?;
    if !can_touch_project(&ts, &scope, row.project_id) {
        return Err(ApiError::forbidden());
    }
    let payload = row.payload.clone();
    Ok(Json(RecordDetailDto { row: row.into(), payload }))
}
async fn f_17e(
    State(r_q0): State<AppState>,
    r_pw: Ts,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    r_pw.require_perm(Permission::RecordView)?;
    let r_py = records::get(&r_q0.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let r_pz = f_29a(&r_q0.pool, &r_pw.user).await?;
    if !can_touch_project(&r_pw, &r_pz, r_py.project_id) {
        return Err(ApiError::forbidden());
    }
    let r_q1 = r_q0
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let r_px = r_q1
        .get(&r_py.pdf_key)
        .await
        .map_err(|e| {
            tracing::error!(
                record = % id, error = % e, "đọc PDF biên bản lỗi"
            );
            ApiError::internal()
        })?
        .ok_or_else(ApiError::not_found)?;
    let safe_name: String = format!("{}.pdf", r_py.so_bien_ban.replace('/', "-"));
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{safe_name}\""),
            ),
        ],
        Body::from(r_px),
    )
        .into_response())
}
