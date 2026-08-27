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
use digitext_db::records::{self, RecordRow, Tdz};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/records", get(f_1o9).post(f_12d))
        .route("/records/{id}/download", get(f_17e))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3g {
    pub project_id: Option<Uuid>,
    pub kind: String,
    pub payload: serde_json::Value,
    pub submission_id: Option<Uuid>,
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
        }
    }
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
    let (title, r_pf) = bienban::render(kind, payload).map_err(ApiError::bad_request)?;
    let r_pk = r_pj
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("a2hvIG9iamVjdCBjaMawYSBj4bqldSBow6xuaA=="))?;
    let r_pg = digitext_pdf::f_1cq()
        .ok_or_else(|| {
            ApiError::service_unavailable(
                "a2jDtG5nIHTDrG0gdGjhuqV5IGZvbnQgVFRGIGNobyBiacOqbiBi4bqjbiDigJQgxJHhurd0IERJR0lURVhUX0NFUlRfRk9OVA==",
            )
        })?;
    let font = std::fs::read(&r_pg).map_err(|e| {
        tracing::error!(
            path = % r_pg.display(), error = % e,
            "xJHhu41jIGZvbnQgYmnDqm4gYuG6o24gbOG7l2k="
        );
        ApiError::internal()
    })?;
    let pdf =
        digitext_pdf::f_ve(&title, Some(bienban::f_2i2(kind)), &r_pf, &font).map_err(|e| {
            ApiError::bad_request(format!("c2luaCBQREYgYmnDqm4gYuG6o24gbOG7l2k6IA=={e}"))
        })?;
    let record_id = Uuid::now_v7();
    let r_ph = f_26d(project_id, record_id);
    r_pk.put(&r_ph, &pdf).await.map_err(|e| {
        tracing::error!(
            error = % e, "UFVUIFBERiBiacOqbiBi4bqjbiB2w6BvIGtobyBs4buXaQ=="
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
            "dGhp4bq/dSBwcm9qZWN0SWQgKG5nxrDhu51pIGfhu41pIGtow7RuZyB0aHXhu5ljIGThu7Egw6FuIG7DoG8p",
        )
    })?;
    let r_po = f_29a(&r_pp.pool, &r_pl.user).await?;
    if !can_touch_project(&r_pl, &r_po, project_id) {
        return Err(ApiError::forbidden());
    }
    let r_pm = f_12e(
        &r_pp,
        &r_pl,
        project_id,
        &r_pn.kind,
        &r_pn.payload,
        r_pn.submission_id,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(r_pm)))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
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
    let r_pr = r_pq.list_projects(r_pu);
    let r_pt = records::list(&r_pv.pool, r_pr.as_deref(), limit, offset).await?;
    Ok(Json(r_pt.into_iter().map(RecordDto::from).collect()))
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
        .ok_or_else(|| ApiError::service_unavailable("a2hvIG9iamVjdCBjaMawYSBj4bqldSBow6xuaA=="))?;
    let r_px = r_q1
        .get(&r_py.pdf_key)
        .await
        .map_err(|e| {
            tracing::error!(
                record = % id, error = % e, "xJHhu41jIFBERiBiacOqbiBi4bqjbiBs4buXaQ=="
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
