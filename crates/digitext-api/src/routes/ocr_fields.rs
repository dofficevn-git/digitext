// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::extract::{Path, Query, State};
use axum::http::header::{HeaderValue, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::{Permission, VanBanField};
use digitext_db::ocr_fields::{self, Tdw, Tfb};
use digitext_db::{documents, engine_versions};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/documents/{id}/fields", get(f_1ne).patch(f_11n))
        .route("/ocr/fields/export", get(f_1aj))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDto {
    pub page_no: i32,
    pub engine_version_id: Uuid,
    pub text_source: String,
    pub field_key: String,
    pub value: String,
    pub bbox: Option<serde_json::Value>,
    pub confidence: Option<f32>,
    pub verified: bool,
    pub updated_at: DateTime<Utc>,
}
impl From<Tfb> for FieldDto {
    fn from(r_ki: Tfb) -> Self {
        FieldDto {
            page_no: r_ki.page_no,
            engine_version_id: r_ki.engine_version_id,
            text_source: r_ki.text_source,
            field_key: r_ki.field_key,
            value: r_ki.value,
            bbox: r_ki.bbox,
            confidence: r_ki.confidence,
            verified: r_ki.verified,
            updated_at: r_ki.updated_at,
        }
    }
}
async fn f_sv(r_kl: &AppState, r_kj: &Ts, id: Uuid) -> Result<documents::Document, ApiError> {
    let r_kk = documents::get(&r_kl.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !f_29a(&r_kl.pool, &r_kj.user)
        .await?
        .allows_project(r_kk.project_id)
    {
        return Err(ApiError::forbidden());
    }
    Ok(r_kk)
}
async fn f_1ne(
    State(r_kp): State<AppState>,
    r_km: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<FieldDto>>, ApiError> {
    r_km.require_perm(Permission::DocumentList)?;
    let r_kn = f_sv(&r_kp, &r_km, id).await?;
    let r_ko = ocr_fields::f_1nh(&r_kp.pool, r_kn.id).await?;
    Ok(Json(r_ko.into_iter().map(FieldDto::from).collect()))
}
fn default_true() -> bool {
    true
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T34 {
    pub page_no: i32,
    pub field_key: String,
    pub value: String,
    #[serde(default)]
    pub engine_version_id: Option<Uuid>,
    #[serde(default)]
    pub bbox: Option<serde_json::Value>,
    #[serde(default = "default_true")]
    pub verified: bool,
}
fn f_ud(r_kt: Option<&serde_json::Value>) -> Option<Vec<f64>> {
    let r_kq = r_kt?.as_array()?;
    let r_ks: Vec<f64> = r_kq.iter().filter_map(|r_kr| r_kr.as_f64()).collect();
    (r_ks.len() == r_kq.len()).then_some(r_ks)
}
fn f_1j8(r_ku: &Tfb, value: &str, bbox: Option<&serde_json::Value>, verified: bool) -> bool {
    r_ku.value == value
        && r_ku.verified == verified
        && digitext_core::f_ue(f_ud(r_ku.bbox.as_ref()).as_deref(), f_ud(bbox).as_deref())
}
async fn f_11n(
    State(r_l1): State<AppState>,
    r_kv: Ts,
    Path(id): Path<Uuid>,
    Json(r_kw): Json<T34>,
) -> Result<Json<FieldDto>, ApiError> {
    r_kv.require_perm(Permission::DocumentList)?;
    let r_kx = f_sv(&r_l1, &r_kv, id).await?;
    if r_kw.page_no < 1 {
        return Err(ApiError::bad_request("page_no phải ≥ 1"));
    }
    let field: VanBanField = r_kw
        .field_key
        .parse()
        .map_err(|_| ApiError::bad_request("field_key không hợp lệ"))?;
    let engine = match r_kw.engine_version_id {
        Some(e) => e,
        None => {
            engine_versions::f_ql(&r_l1.pool, "recognition")
                .await?
                .ok_or_else(|| {
                    ApiError::bad_request(
                        "chưa có engine recognition active để gắn",
                    )
                })?
                .id
        }
    };
    let value = digitext_core::f_x2(&r_kw.value);
    if let Some(r_ky) = ocr_fields::get(
        &r_l1.pool,
        r_kx.id,
        r_kw.page_no,
        engine,
        "ocr_corrected",
        field.as_str(),
    )
    .await?
    {
        if f_1j8(&r_ky, &value, r_kw.bbox.as_ref(), r_kw.verified) {
            return Ok(Json(FieldDto::from(r_ky)));
        }
    }
    let r_l0 = match ocr_fields::upsert(
        &r_l1.pool,
        Tdw {
            document_id: r_kx.id,
            page_no: r_kw.page_no,
            engine_version_id: engine,
            text_source: "ocr_corrected",
            field_key: field.as_str(),
            value: &value,
            bbox: r_kw.bbox.clone(),
            confidence: None,
            verified: r_kw.verified,
        },
    )
    .await
    {
        Ok(r_kz) => r_kz,
        Err(e)
            if matches!(
                & e, sqlx::Error::Database(db) if db.code().as_deref() == Some("23503")
            ) =>
        {
            return Err(ApiError::bad_request(
                "engineVersionId không tồn tại",
            ));
        }
        Err(e) => return Err(e.into()),
    };
    digitext_db::audit::append(
        &r_l1.pool,
        digitext_db::audit::Tp {
            actor_id: Some(r_kv.user.id),
            device_id: r_kv.claims.device_id,
            action: "ocr.field.correct",
            target: Some(format!("document:{}", r_kx.id)),
            detail: Some(serde_json::json!(
                { "pageNo" : r_kw.page_no, "fieldKey" : field.as_str(),
                "verified" : r_kw.verified }
            )),
            ip: r_kv.ip(),
        },
    )
    .await?;
    Ok(Json(FieldDto::from(r_l0)))
}
const C_3V: i64 = 1000;
const C_BO: i64 = 10000;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T72 {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub after_doc: Option<Uuid>,
    #[serde(default)]
    pub after_page: Option<i32>,
    #[serde(default)]
    pub after_key: Option<String>,
}
async fn f_1aj(
    State(r_lh): State<AppState>,
    r_l3: Ts,
    Query(r_ld): Query<T72>,
) -> Result<Response, ApiError> {
    r_l3.require_perm(Permission::DocumentList)?;
    let limit = r_ld.limit.unwrap_or(C_3V).clamp(1, C_BO);
    let r_lc: Option<Vec<Uuid>> = match f_29a(&r_lh.pool, &r_l3.user).await? {
        Scope::All => None,
        Scope::Projects(r_l7) => Some(r_l7),
    };
    let r_l2 = match (r_ld.after_doc, r_ld.after_page, r_ld.after_key.as_deref()) {
        (Some(r_l6), Some(r_lb), Some(r_l8)) => Some((r_l6, r_lb, r_l8)),
        _ => None,
    };
    let r_lg = ocr_fields::f_1ak(&r_lh.pool, r_lc.as_deref(), r_l2, limit).await?;
    let mut r_l4 = String::new();
    for r_le in &r_lg {
        let r_la = serde_json::json!(
            { "documentId" : r_le.document_id, "pageNo" : r_le.page_no, "fieldKey" : r_le
            .field_key, "value" : r_le.value, "bbox" : r_le.bbox, }
        );
        r_l4.push_str(&r_la.to_string());
        r_l4.push('\n');
    }
    let mut r_lf = r_l4.into_response();
    r_lf.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-ndjson"),
    );
    if r_lg.len() as i64 == limit {
        if let Some(r_l9) = r_lg.last() {
            let r_l5 = format!("{},{},{}", r_l9.document_id, r_l9.page_no, r_l9.field_key);
            if let Ok(r_li) = HeaderValue::from_str(&r_l5) {
                r_lf.headers_mut().insert("X-Next-Cursor", r_li);
            }
        }
    }
    Ok(r_lf)
}
