// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{Scope, Ts};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::{Perm, Permission};
use digitext_db::upload_batches::{self, T18};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upload-batches", get(list_batches).post(open_batch))
        .route("/upload-batches/matrix", get(f_u9))
        .route("/upload-batches/{id}", axum::routing::patch(complete_batch))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tff {
    #[serde(default)]
    pub note: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T30 {
    pub forwarded: i32,
    pub skipped: i32,
    pub failed: i32,
    pub rejected: i32,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDto {
    pub id: Uuid,
    pub forwarded_by: Option<Uuid>,
    pub forwarded_by_name: Option<String>,
    pub device_id: Option<Uuid>,
    pub note: Option<String>,
    pub status: String,
    pub forwarded: i32,
    pub skipped: i32,
    pub failed: i32,
    pub rejected: i32,
    pub received: i64,
    pub project_count: i64,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
impl From<T18> for BatchDto {
    fn from(r_vb: T18) -> Self {
        BatchDto {
            id: r_vb.id,
            forwarded_by: r_vb.forwarded_by,
            forwarded_by_name: r_vb.forwarded_by_name,
            device_id: r_vb.device_id,
            note: r_vb.note,
            status: r_vb.status,
            forwarded: r_vb.forwarded,
            skipped: r_vb.skipped,
            failed: r_vb.failed,
            rejected: r_vb.rejected,
            received: r_vb.received,
            project_count: r_vb.project_count,
            created_at: r_vb.created_at,
            completed_at: r_vb.completed_at,
        }
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tfg {
    pub id: Uuid,
}
fn f_28g(r_vc: &Ts) -> Result<(), ApiError> {
    if r_vc.user.role.allows(Perm::Admin) {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}
async fn open_batch(
    State(r_vh): State<AppState>,
    r_vd: Ts,
    Json(r_vf): Json<Tff>,
) -> Result<(StatusCode, Json<Tfg>), ApiError> {
    r_vd.require_perm(Permission::UploadExecute)?;
    f_28g(&r_vd)?;
    let note = r_vf
        .note
        .as_deref()
        .map(str::trim)
        .filter(|r_vg| !r_vg.is_empty());
    let r_ve =
        upload_batches::open(&r_vh.pool, Some(r_vd.user.id), r_vd.claims.device_id, note).await?;
    Ok((StatusCode::CREATED, Json(Tfg { id: r_ve.id })))
}
async fn complete_batch(
    State(r_vk): State<AppState>,
    r_vi: Ts,
    Path(id): Path<Uuid>,
    Json(r_vj): Json<T30>,
) -> Result<Json<BatchDto>, ApiError> {
    r_vi.require_perm(Permission::UploadExecute)?;
    f_28g(&r_vi)?;
    if r_vj.forwarded < 0 || r_vj.skipped < 0 || r_vj.failed < 0 || r_vj.rejected < 0 {
        return Err(ApiError::bad_request(
            "c+G7kSBsaeG7h3UgdMOzbSB04bqvdCBraMO0bmcgxJHGsOG7o2Mgw6Jt",
        ));
    }
    upload_batches::complete(
        &r_vk.pool,
        id,
        Some(r_vi.user.id),
        r_vj.forwarded,
        r_vj.skipped,
        r_vj.failed,
        r_vj.rejected,
    )
    .await?
    .ok_or_else(ApiError::not_found)?;
    f_u6(&r_vk, id).await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    #[serde(default)]
    pub limit: Option<i64>,
}
const DEFAULT_LIMIT: i64 = 100;
const MAX_LIMIT: i64 = 500;
async fn list_batches(
    State(r_vp): State<AppState>,
    r_vl: Ts,
    Query(r_vn): Query<ListQuery>,
) -> Result<Json<Vec<BatchDto>>, ApiError> {
    r_vl.require_perm(Permission::DocumentList)?;
    let limit = r_vn.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let r_vm = match crate::extract::f_29a(&r_vp.pool, &r_vl.user).await? {
        Scope::All => None,
        _ => Some(r_vl.user.id),
    };
    let r_vo = upload_batches::list(&r_vp.pool, r_vm, limit).await?;
    Ok(Json(r_vo.into_iter().map(BatchDto::from).collect()))
}
async fn f_u6(r_vr: &AppState, id: Uuid) -> Result<Json<BatchDto>, ApiError> {
    upload_batches::f_1ev(&r_vr.pool, id)
        .await?
        .map(|r_vq| Json(BatchDto::from(r_vq)))
        .ok_or_else(ApiError::not_found)
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Td2 {
    pub id: Uuid,
    pub name: Option<String>,
    pub attention: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Td5 {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub cells: std::collections::HashMap<String, i64>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Td4 {
    pub admins: Vec<Td2>,
    pub projects: Vec<Td5>,
}
async fn f_u9(State(r_vz): State<AppState>, r_vs: Ts) -> Result<Json<Td4>, ApiError> {
    r_vs.require_perm(Permission::DocumentList)?;
    let cells = match crate::extract::f_29a(&r_vz.pool, &r_vs.user).await? {
        Scope::All => upload_batches::matrix(&r_vz.pool, None).await?,
        Scope::Projects(r_vu) => upload_batches::matrix(&r_vz.pool, Some(&r_vu)).await?,
    };
    let r_vy: std::collections::HashSet<Uuid> = upload_batches::f_rf(&r_vz.pool)
        .await?
        .into_iter()
        .collect();
    let mut admins: Vec<Td2> = Vec::new();
    let mut r_vx: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    let mut projects: Vec<Td5> = Vec::new();
    let mut r_vw: std::collections::HashMap<Uuid, usize> = std::collections::HashMap::new();
    for r_vt in cells {
        if r_vx.insert(r_vt.admin_id) {
            admins.push(Td2 {
                id: r_vt.admin_id,
                name: r_vt.admin_name.clone(),
                attention: r_vy.contains(&r_vt.admin_id),
            });
        }
        let r_vv = *r_vw.entry(r_vt.project_id).or_insert_with(|| {
            projects.push(Td5 {
                id: r_vt.project_id,
                code: r_vt.project_code.clone(),
                name: r_vt.project_name.clone(),
                cells: std::collections::HashMap::new(),
            });
            projects.len() - 1
        });
        projects[r_vv]
            .cells
            .insert(r_vt.admin_id.to_string(), r_vt.count);
    }
    Ok(Json(Td4 { admins, projects }))
}
