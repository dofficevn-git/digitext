// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::Permission;
use digitext_db::access::{self, Ta, Tdq, T9};
use digitext_db::packages::{self, Tdy};
use digitext_db::{documents, notifications, projects};
use digitext_package::model::{T4n, T4o};
use digitext_package::wire::{Tq4, WireMeta};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/access", get(list_requests).post(create_request))
        .route("/access/fees", get(f_1el).put(f_24w))
        .route("/access/{id}", get(get_request))
        .route("/access/{id}/items/{item_id}/review", post(review_item))
        .route("/access/{id}/approve", post(approve_request))
        .route("/access/{id}/refuse", post(refuse_request))
        .route("/access/{id}/pay", post(confirm_payment))
        .route("/access/{id}/items/{item_id}/read", get(f_25r))
        .route("/access/{id}/items/{item_id}/download", get(f_178))
        .route("/access/{id}/dip", post(f_127))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T35 {
    pub project_id: Option<Uuid>,
    pub kind: String,
    pub muc_dich: String,
    pub phuong_thuc: Option<String>,
    pub uu_tien: Option<String>,
    pub nguoi_nhan: Option<String>,
    pub so_dinh_danh: Option<String>,
    pub document_ids: Vec<Uuid>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub requester: Option<Uuid>,
    pub kind: String,
    pub muc_dich: String,
    pub phuong_thuc: Option<String>,
    pub uu_tien: Option<String>,
    pub nguoi_nhan: Option<String>,
    pub status: String,
    pub review_note: Option<String>,
    pub phi: Option<i64>,
    pub paid_at: Option<DateTime<Utc>>,
    pub han_su_dung: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl From<Ta> for AccessDto {
    fn from(r_76: Ta) -> Self {
        AccessDto {
            id: r_76.id,
            project_id: r_76.project_id,
            requester: r_76.requester,
            kind: r_76.kind,
            muc_dich: r_76.muc_dich,
            phuong_thuc: r_76.phuong_thuc,
            uu_tien: r_76.uu_tien,
            nguoi_nhan: r_76.nguoi_nhan,
            status: r_76.status,
            review_note: r_76.review_note,
            phi: r_76.phi,
            paid_at: r_76.paid_at,
            han_su_dung: r_76.han_su_dung,
            approved_at: r_76.approved_at,
            created_at: r_76.created_at,
            updated_at: r_76.updated_at,
        }
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T8 {
    pub id: Uuid,
    pub document_id: Uuid,
    pub stt: i32,
    pub verdict: String,
    pub ly_do: Option<String>,
    pub dieu_kien: bool,
    pub filename: Option<String>,
    pub ho_so_ref: Option<String>,
    pub page_count: Option<i32>,
    pub deleted: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T7 {
    #[serde(flatten)]
    pub request: AccessDto,
    pub items: Vec<T8>,
}
fn can_touch_project(r_77: &Ts, r_79: &Scope, r_78: Uuid) -> bool {
    r_77.can_touch_project(r_79, r_78)
}
async fn load_in_scope(r_7d: &AppState, r_7a: &Ts, id: Uuid) -> Result<Ta, ApiError> {
    let r_7b = access::get(&r_7d.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let r_7c = f_29a(&r_7d.pool, &r_7a.user).await?;
    if !can_touch_project(r_7a, &r_7c, r_7b.project_id) {
        return Err(ApiError::forbidden());
    }
    Ok(r_7b)
}
fn require_owner_or_reviewer(r_7e: &Ts, r_7f: &Ta) -> Result<(), ApiError> {
    if r_7f.requester == Some(r_7e.user.id) || r_7e.has_perm(Permission::AccessReview) {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}
fn note_of(r_7h: &Option<String>) -> Option<&str> {
    r_7h.as_deref()
        .map(str::trim)
        .filter(|r_7g| !r_7g.is_empty())
}
async fn detail(r_7k: &AppState, r_7j: Ta) -> Result<T7, ApiError> {
    let mut items = Vec::new();
    for r_7i in access::items(&r_7k.pool, r_7j.id).await? {
        let doc = documents::get(&r_7k.pool, r_7i.document_id).await?;
        items.push(T8 {
            id: r_7i.id,
            document_id: r_7i.document_id,
            stt: r_7i.stt,
            verdict: r_7i.verdict,
            ly_do: r_7i.ly_do,
            dieu_kien: r_7i.dieu_kien,
            filename: doc.as_ref().map(|d| d.filename.clone()),
            ho_so_ref: doc.as_ref().and_then(|d| d.ho_so_ref.clone()),
            page_count: doc.as_ref().and_then(|d| d.page_count),
            deleted: r_7i.deleted_at.is_some(),
        });
    }
    Ok(T7 {
        request: r_7j.into(),
        items,
    })
}
async fn notify_requester(r_7n: &AppState, r_7m: &Ta, r_7l: &str) {
    if let Some(r_7o) = r_7m.requester {
        if let Err(e) = notifications::f_1v8(&r_7n.pool, r_7o, None, "access_status", r_7l).await {
            tracing::warn!(
                error = % e, "không ghi được thông báo khai thác"
            );
        }
    }
}
async fn create_request(
    State(r_7u): State<AppState>,
    r_7p: Ts,
    Json(r_7r): Json<T35>,
) -> Result<(StatusCode, Json<T7>), ApiError> {
    r_7p.require_perm(Permission::AccessRequest)?;
    let project_id = r_7r.project_id.or(r_7p.user.project_id).ok_or_else(|| {
        ApiError::bad_request(
            "thiếu projectId (người gọi không thuộc dự án nào)",
        )
    })?;
    let r_7t = f_29a(&r_7u.pool, &r_7p.user).await?;
    if !can_touch_project(&r_7p, &r_7t, project_id) {
        return Err(ApiError::forbidden());
    }
    if !matches!(r_7r.kind.as_str(), "doc" | "ban_sao" | "ban_sao_ky" | "dip") {
        return Err(
            ApiError::bad_request(
                "kind nhận 'doc' | 'ban_sao' | 'ban_sao_ky' | 'dip' (Điều 35/40/41/42)",
            ),
        );
    }
    if r_7r.kind == "ban_sao_ky" {
        match crate::copy_sign::f_1ov() {
            Ok(Some(_)) => {}
            Ok(None) => {
                return Err(
                    ApiError::service_unavailable(
                        "chưa cấu hình khóa ký bản sao — đặt DIGITEXT_COPY_SIGN_KEY/CERT (Điều 41)",
                    ),
                );
            }
            Err(e) => {
                tracing::error!(error = % e, "khóa ký bản sao lỗi");
                return Err(ApiError::internal());
            }
        }
    }
    if r_7r.muc_dich.trim().is_empty() {
        return Err(ApiError::bad_request(
            "phải khai mục đích sử dụng (Điều 35.4)",
        ));
    }
    if r_7r.kind == "dip"
        && (note_of(&r_7r.nguoi_nhan).is_none() || note_of(&r_7r.so_dinh_danh).is_none())
    {
        return Err(
            ApiError::bad_request(
                "bản sao DIP phải khai người nhận + mã định danh (Điều 42.2.c)",
            ),
        );
    }
    if r_7r.document_ids.is_empty() {
        return Err(ApiError::bad_request(
            "chọn tối thiểu 01 tài liệu",
        ));
    }
    for id in &r_7r.document_ids {
        let doc = documents::get(&r_7u.pool, *id).await?.ok_or_else(|| {
            ApiError::bad_request(format!("tài liệu {id} không tồn tại"))
        })?;
        if doc.project_id != project_id {
            return Err(ApiError::bad_request(format!(
                "tài liệu {id} không thuộc dự án"
            )));
        }
    }
    let (r_7s, _) = match access::create(
        &r_7u.pool,
        Tdq {
            project_id,
            requester: Some(r_7p.user.id),
            kind: &r_7r.kind,
            muc_dich: r_7r.muc_dich.trim(),
            phuong_thuc: note_of(&r_7r.phuong_thuc),
            uu_tien: note_of(&r_7r.uu_tien),
            nguoi_nhan: note_of(&r_7r.nguoi_nhan),
            so_dinh_danh: note_of(&r_7r.so_dinh_danh),
        },
        &r_7r.document_ids,
    )
    .await
    {
        Ok(r_7v) => r_7v,
        Err(e) if digitext_db::f_1jn(&e) => {
            return Err(ApiError::conflict(
                "một tài liệu xuất hiện hai lần trong yêu cầu"
                    .to_string(),
            ));
        }
        Err(e) => return Err(e.into()),
    };
    let r_7q = detail(&r_7u, r_7s).await?;
    Ok((StatusCode::CREATED, Json(r_7q)))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
async fn list_requests(
    State(r_80): State<AppState>,
    r_7w: Ts,
    Query(r_7x): Query<ListQuery>,
) -> Result<Json<Vec<AccessDto>>, ApiError> {
    r_7w.require_perm(Permission::AccessView)?;
    let limit = r_7x.limit.unwrap_or(50).clamp(1, 200);
    let offset = r_7x.offset.unwrap_or(0).max(0);
    let r_7z = f_29a(&r_80.pool, &r_7w.user).await?;
    let projects = r_7w.list_projects(r_7z);
    let rows = access::list(&r_80.pool, projects.as_deref(), limit, offset).await?;
    let rows: Vec<Ta> = if r_7w.has_perm(Permission::AccessReview) {
        rows
    } else {
        rows.into_iter()
            .filter(|r_7y| r_7y.requester == Some(r_7w.user.id))
            .collect()
    };
    Ok(Json(rows.into_iter().map(AccessDto::from).collect()))
}
async fn get_request(
    State(r_83): State<AppState>,
    r_81: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<T7>, ApiError> {
    r_81.require_perm(Permission::AccessView)?;
    let r_82 = load_in_scope(&r_83, &r_81, id).await?;
    require_owner_or_reviewer(&r_81, &r_82)?;
    Ok(Json(detail(&r_83, r_82).await?))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewBody {
    pub verdict: String,
    pub ly_do: Option<String>,
    #[serde(default)]
    pub dieu_kien: bool,
}
async fn review_item(
    State(r_89): State<AppState>,
    r_84: Ts,
    Path((id, item_id)): Path<(Uuid, Uuid)>,
    Json(r_85): Json<ReviewBody>,
) -> Result<StatusCode, ApiError> {
    r_84.require_perm(Permission::AccessReview)?;
    let r_88 = load_in_scope(&r_89, &r_84, id).await?;
    if r_85.verdict == "tu_choi" && note_of(&r_85.ly_do).is_none() {
        return Err(ApiError::bad_request(
            "từ chối phải nêu rõ lý do (Điều 36.1.c)",
        ));
    }
    let r_87 = access::items(&r_89.pool, r_88.id)
        .await?
        .iter()
        .any(|r_86| r_86.id == item_id);
    if !r_87 {
        return Err(ApiError::not_found());
    }
    if !access::review_item(
        &r_89.pool,
        item_id,
        &r_85.verdict,
        note_of(&r_85.ly_do),
        r_85.dieu_kien,
    )
    .await?
    {
        return Err(
            ApiError::bad_request(
                "verdict nhận 'duyet' hoặc 'tu_choi'; yêu cầu phải còn chờ duyệt"
                    .to_string(),
            ),
        );
    }
    Ok(StatusCode::NO_CONTENT)
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteBody {
    pub note: Option<String>,
}
async fn approve_request(
    State(r_8e): State<AppState>,
    r_8a: Ts,
    Path(id): Path<Uuid>,
    Json(r_8b): Json<NoteBody>,
) -> Result<Json<T7>, ApiError> {
    r_8a.require_perm(Permission::AccessReview)?;
    let r_8d = load_in_scope(&r_8e, &r_8a, id).await?;
    let Some(approved) = access::approve(&r_8e.pool, r_8d.id, note_of(&r_8b.note)).await? else {
        return Err(
            ApiError::conflict(
                format!(
                    "chỉ duyệt được yêu cầu chờ duyệt có ít nhất 01 tài liệu ĐẠT (đang là '{}')",
                    r_8d.status
                ),
            ),
        );
    };
    let phi = approved.phi.unwrap_or(0);
    let han = approved
        .han_su_dung
        .map(|r_8c| r_8c.format("%d/%m/%Y").to_string())
        .unwrap_or_default();
    notify_requester(
            &r_8e,
            &approved,
            &format!(
                "Yêu cầu khai thác đã DUYỆT — phí {phi} đồng, hạn sử dụng đến {han} (Điều 36.1.c)"
            ),
        )
        .await;
    Ok(Json(detail(&r_8e, approved).await?))
}
async fn refuse_request(
    State(r_8j): State<AppState>,
    r_8f: Ts,
    Path(id): Path<Uuid>,
    Json(r_8g): Json<NoteBody>,
) -> Result<Json<T7>, ApiError> {
    r_8f.require_perm(Permission::AccessReview)?;
    let r_8i = load_in_scope(&r_8j, &r_8f, id).await?;
    let note = note_of(&r_8g.note).ok_or_else(|| {
        ApiError::bad_request(
            "từ chối phải nêu rõ lý do (Điều 36.1.c)",
        )
    })?;
    if !access::refuse(&r_8j.pool, r_8i.id, note).await? {
        return Err(
            ApiError::conflict(
                format!(
                    "chỉ từ chối được yêu cầu chờ duyệt (đang là '{}')",
                    r_8i.status
                ),
            ),
        );
    }
    let r_8h = access::get(&r_8j.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    notify_requester(
        &r_8j,
        &r_8h,
        &format!("Yêu cầu khai thác bị TỪ CHỐI: {note}"),
    )
    .await;
    Ok(Json(detail(&r_8j, r_8h).await?))
}
async fn confirm_payment(
    State(r_8n): State<AppState>,
    r_8k: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<T7>, ApiError> {
    r_8k.require_perm(Permission::AccessReview)?;
    let r_8m = load_in_scope(&r_8n, &r_8k, id).await?;
    if !access::confirm_payment(&r_8n.pool, r_8m.id).await? {
        return Err(
            ApiError::conflict(
                "chỉ xác nhận thanh toán cho yêu cầu đã duyệt và chưa thanh toán"
                    .to_string(),
            ),
        );
    }
    let r_8l = access::get(&r_8n.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    notify_requester(
        &r_8n,
        &r_8l,
        "Đã xác nhận thanh toán phí (Điều 36.4)",
    )
    .await;
    Ok(Json(detail(&r_8n, r_8l).await?))
}
fn user_copy_key(project_id: Uuid, request_id: Uuid, item_id: Uuid) -> String {
    format!("{project_id}-access/{request_id}/{item_id}.pdf")
}
async fn f_2od(
    r_91: &AppState,
    r_8o: &Ts,
    request_id: Uuid,
    item_id: Uuid,
) -> Result<(Ta, Vec<u8>, Uuid), ApiError> {
    let r_8w = load_in_scope(r_91, r_8o, request_id).await?;
    require_owner_or_reviewer(r_8o, &r_8w)?;
    if r_8w.status != "approved" {
        return Err(ApiError::conflict(format!(
            "bản người dùng chỉ có sau khi duyệt (đang là '{}')",
            r_8w.status
        )));
    }
    if !r_8w.fee_settled() {
        return Err(ApiError::conflict(
            "chưa thanh toán phí (Điều 36.3.b/36.4)".to_string(),
        ));
    }
    if !r_8w.within_deadline() {
        return Err(ApiError::conflict(
            "đã quá hạn sử dụng 15 ngày (Điều 36.1.c/40.5)"
                .to_string(),
        ));
    }
    let r_8t: T9 = access::items(&r_91.pool, r_8w.id)
        .await?
        .into_iter()
        .find(|r_8s| r_8s.id == item_id)
        .ok_or_else(ApiError::not_found)?;
    if r_8t.verdict != "duyet" {
        return Err(ApiError::forbidden());
    }
    if r_8t.deleted_at.is_some() {
        return Err(
            ApiError::conflict(
                "bản người dùng đã bị xóa theo hạn (Điều 37.5/40.6)"
                    .to_string(),
            ),
        );
    }
    let r_92 = r_91
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let key = user_copy_key(r_8w.project_id, r_8w.id, r_8t.id);
    if let Some(r_8p) = r_92.get(&key).await.map_err(|e| {
        tracing::error!(error = % e, "đọc bản người dùng lỗi");
        ApiError::internal()
    })? {
        return Ok((r_8w, r_8p, r_8t.document_id));
    }
    let r_8x = digitext_storage::f_2fc(r_8w.project_id, r_8t.document_id);
    let r_93 = digitext_storage::f_2qs(r_8w.project_id, r_8t.document_id);
    let r_8z = match r_92.get(&r_8x).await {
        Ok(Some(b)) => b,
        Ok(None) => r_92
            .get(&r_93)
            .await
            .map_err(|e| {
                tracing::error!(error = % e, "đọc PDF nguồn lỗi");
                ApiError::internal()
            })?
            .ok_or_else(|| {
                ApiError::conflict(
                    "tài liệu chưa có PDF trong kho (chưa dựng/ký)"
                        .to_string(),
                )
            })?,
        Err(e) => {
            tracing::error!(error = % e, "đọc PDF nguồn lỗi");
            return Err(ApiError::internal());
        }
    };
    let r_8u = projects::get(&r_91.pool, r_8w.project_id)
        .await?
        .map(|r_8v| r_8v.name)
        .unwrap_or_else(|| "Lưu trữ lịch sử".to_string());
    let r_8r = digitext_pdf::f_1cq()
        .ok_or_else(|| {
            ApiError::service_unavailable(
                "không tìm thấy font TTF cho watermark — đặt DIGITEXT_CERT_FONT",
            )
        })?;
    let font = std::fs::read(&r_8r).map_err(|e| {
        tracing::error!(
            path = % r_8r.display(), error = % e, "đọc font lỗi"
        );
        ApiError::internal()
    })?;
    let ban_sao = r_8w.kind != "doc";
    let mut r_90 = digitext_pdf::f_2h7(&r_8z, &r_8u, ban_sao, &font).map_err(|e| {
        ApiError::bad_request(format!(
            "dựng bản người dùng lỗi: {e}"
        ))
    })?;
    if r_8w.kind == "ban_sao_ky" {
        let r_8y = crate::copy_sign::f_1ov().map_err(|e| {
            tracing::error!(error = % e, "khóa ký bản sao lỗi");
            ApiError::internal()
        })?;
        let Some((key, r_8q)) = r_8y else {
            return Err(
                ApiError::service_unavailable(
                    "chưa cấu hình khóa ký bản sao — đặt DIGITEXT_COPY_SIGN_KEY/CERT (Điều 41)",
                ),
            );
        };
        r_90 = crate::copy_sign::f_2f1(&r_90, &key, &r_8q).map_err(|e| {
            tracing::error!(error = % e, "ký bản sao lỗi");
            ApiError::internal()
        })?;
    }
    r_92.put(&key, &r_90).await.map_err(|e| {
        tracing::error!(error = % e, "PUT bản người dùng lỗi");
        ApiError::internal()
    })?;
    access::f_2ej(&r_91.pool, r_8t.id, &key).await?;
    Ok((r_8w, r_90, r_8t.document_id))
}
async fn f_25r(
    State(r_96): State<AppState>,
    r_94: Ts,
    Path((id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, ApiError> {
    r_94.require_perm(Permission::AccessView)?;
    let (r_yc, r_95, doc_id) = f_2od(&r_96, &r_94, id, item_id).await?;
    crate::routes::tra_cuu::ghi_nhat_ky(&r_96, &r_94, "doc", r_yc.project_id, Some(doc_id), serde_json::json!({ "yeuCau": id, "hinhThuc": r_yc.kind })).await;
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (header::CONTENT_DISPOSITION, "inline".to_string()),
        ],
        Body::from(r_95),
    )
        .into_response())
}
async fn f_178(
    State(r_9a): State<AppState>,
    r_97: Ts,
    Path((id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, ApiError> {
    r_97.require_perm(Permission::AccessView)?;
    let (r_99, r_98, doc_id) = f_2od(&r_9a, &r_97, id, item_id).await?;
    if !matches!(r_99.kind.as_str(), "ban_sao" | "ban_sao_ky") {
        return Err(
            ApiError::conflict(
                "bản đọc trên Hệ thống không cho phép tải (Điều 37.4)"
                    .to_string(),
            ),
        );
    }
    crate::routes::tra_cuu::ghi_nhat_ky(&r_9a, &r_97, "tai", r_99.project_id, Some(doc_id), serde_json::json!({ "yeuCau": id, "hinhThuc": r_99.kind })).await;
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"ban-sao-{item_id}.pdf\""),
            ),
        ],
        Body::from(r_98),
    )
        .into_response())
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T4p {
    pub package_id: Uuid,
    pub ma_xac_thuc: String,
}
async fn f_127(
    State(r_9q): State<AppState>,
    r_9d: Ts,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<T4p>), ApiError> {
    r_9d.require_perm(Permission::AccessReview)?;
    let r_9o = load_in_scope(&r_9q, &r_9d, id).await?;
    if r_9o.kind != "dip" {
        return Err(ApiError::bad_request(
            "chỉ yêu cầu kind 'dip' mới cấp gói DIP (Điều 42)",
        ));
    }
    if r_9o.status != "approved" || !r_9o.fee_settled() {
        return Err(ApiError::conflict(
            "gói DIP chỉ cấp sau khi duyệt và thanh toán phí"
                .to_string(),
        ));
    }
    let items = access::items(&r_9q.pool, r_9o.id).await?;
    let approved: Vec<&T9> = items
        .iter()
        .filter(|r_9j| r_9j.verdict == "duyet")
        .collect();
    if approved.is_empty() {
        return Err(ApiError::conflict(
            "không có tài liệu nào được duyệt".to_string(),
        ));
    }
    let r_9b = packages::list(&r_9q.pool, Some(&[r_9o.project_id]), 1000, 0).await?;
    let mut r_9h: Option<String> = None;
    let mut r_9c: Option<String> = None;
    let mut r_9g = Vec::new();
    let mut r_9l = Vec::new();
    let mut r_9t = Vec::new();
    for r_9k in &approved {
        let doc = documents::get(&r_9q.pool, r_9k.document_id)
            .await?
            .ok_or_else(ApiError::not_found)?;
        // Bản Lưu trữ: tài liệu nhập từ gói AIP (không có request_json dựng gói) → EAD văn bản lấy từ mục lục đã nhập.
        if let Some(g) = crate::nhap_goi::dip_tu_goi(&r_9q.pool, r_9k.document_id).await? {
            r_9h.get_or_insert(g.fond_code);
            r_9c.get_or_insert(g.archive_org_code);
            r_9l.push(g.ma);
            r_9t.push(g.trich_yeu.unwrap_or_else(|| doc.filename.clone()));
            r_9g.push(g.doc);
            continue;
        }
        let aip = r_9b
            .iter()
            .filter(|p| {
                p.status == "built"
                    && (p.kind == "aip_hoso" || p.kind == "aip_tailieu")
                    && p.ho_so_ref.is_some()
                    && p.ho_so_ref == doc.ho_so_ref
            })
            .max_by_key(|p| p.created_at);
        let Some(aip) = aip else {
            return Err(
                ApiError::conflict(
                    format!(
                        "tài liệu {} chưa có gói AIP trong Kho — không cấp DIP được (Điều 42.1)",
                        doc.filename
                    ),
                ),
            );
        };
        let wire: Tq4 = serde_json::from_value(aip.request_json.clone()).map_err(|e| {
            tracing::error!(
                package = % aip.id, error = % e,
                "request_json AIP không đọc được"
            );
            ApiError::internal()
        })?;
        r_9h.get_or_insert(wire.fond_code.clone());
        r_9c.get_or_insert(wire.archive_org_code.clone());
        let mut r_9m: Vec<_> = wire
            .documents
            .iter()
            .filter(|d| d.document_id == Some(r_9k.document_id))
            .cloned()
            .collect();
        if r_9m.is_empty() {
            return Err(ApiError::conflict(format!(
                "gói AIP của hồ sơ {} không chứa tài liệu {}",
                aip.label, doc.filename
            )));
        }
        for d in &r_9m {
            use digitext_package::ead::DocEad;
            let (r_9e, r_9r) = match &d.metadata {
                DocEad::Doc(f) => (f.doc_code.clone(), f.subject.clone()),
                DocEad::Pic(f) => (f.doc_code.clone(), f.image_title.clone()),
                DocEad::Media(f) => (f.doc_code.clone(), f.movie_title.clone()),
            };
            r_9l.push(r_9e);
            r_9t.push(r_9r.unwrap_or_else(|| doc.filename.clone()));
        }
        r_9g.append(&mut r_9m);
    }
    let r_9s = Utc::now()
        .with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00"))
        .format("%d/%m/%Y")
        .to_string();
    let ma_xac_thuc = format!(
        "XT.{}.{}",
        Utc::now().format("%Y"),
        r_9o.id.simple().to_string()[..8].to_uppercase()
    );
    let r_9n = projects::get(&r_9q.pool, r_9o.project_id)
        .await?
        .map(|p| p.name)
        .unwrap_or_else(|| "Lưu trữ lịch sử".to_string());
    let wire = Tq4 {
        kind: "dip".into(),
        fond_code: r_9h.unwrap_or_default(),
        archive_org_code: r_9c.unwrap_or_default(),
        fond_name: None,
        archive_org_name: Some(r_9n.clone()),
        packager_name: None,
        packager_code: None,
        allow_non_pdfa: true,
        label: Some(format!("DIP khai thác {ma_xac_thuc}")),
        meta: WireMeta::Dip(T4o {
            request_id: ma_xac_thuc.clone(),
            request_date: r_9s.clone(),
            purpose: "01".into(),
            purpose_content: Some(r_9o.muc_dich.clone()),
            fee_object_type: None,
            research_topic: None,
            description: None,
        }),
        submission_agreement: None,
        dissemination_request_id: Some(ma_xac_thuc.clone()),
        related_hoso_mets_id: None,
        include_premis: false,
        from_sip: false,
        zip_ordinal: None,
        content_information_type: None,
        dip_cert: Some(T4n {
            ma_xac_thuc: ma_xac_thuc.clone(),
            ten_luu_tru_lich_su: r_9n,
            nguoi_nhan_ho_ten: r_9o.nguoi_nhan.clone().unwrap_or_default(),
            nguoi_nhan_dinh_danh: r_9o.so_dinh_danh.clone().unwrap_or_default(),
            ma_luu_tru_goc: r_9l.join("; "),
            so_ky_hieu_goc: None,
            ten_loai: "Tài liệu lưu trữ số".into(),
            trich_yeu: r_9t.join("; "),
            muc_dich: r_9o.muc_dich.clone(),
            ngay_cap: r_9s,
            thoi_han_su_dung: r_9o
                .han_su_dung
                .map(|r_9i| r_9i.format("%d/%m/%Y").to_string()),
            so_luong_ban: 1,
        }),
        documents: r_9g,
    };
    let r_9p = wire.shape_errors();
    if !r_9p.is_empty() {
        return Err(ApiError::bad_request(format!(
            "yêu cầu DIP không hợp lệ: {}",
            r_9p.join("; ")
        )));
    }
    let request_json = serde_json::to_value(&wire).map_err(|_| ApiError::internal())?;
    let r_9f = packages::create(
        &r_9q.pool,
        Tdy {
            project_id: r_9o.project_id,
            requested_by: Some(r_9d.user.id),
            kind: "dip",
            ho_so_ref: None,
            label: &wire.display_label(),
            request_json: &request_json,
            submission_id: None,
            converted_from: None,
            stt_range: None,
        },
    )
    .await?;
    notify_requester(
            &r_9q,
            &r_9o,
            &format!(
                "Bản sao DIP {ma_xac_thuc} đang được đóng gói (Điều 42)"
            ),
        )
        .await;
    crate::routes::tra_cuu::ghi_nhat_ky(
        &r_9q,
        &r_9d,
        "dip",
        r_9o.project_id,
        None,
        serde_json::json!({ "yeuCau": id, "maXacThuc": ma_xac_thuc, "goi": r_9f.id, "soTaiLieu": approved.len() }),
    )
    .await;
    Ok((
        StatusCode::CREATED,
        Json(T4p {
            package_id: r_9f.id,
            ma_xac_thuc,
        }),
    ))
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T7y {
    pub project_id: Option<Uuid>,
    pub phi_doc_tai_lieu: i64,
    pub phi_sao_trang: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T7x {
    pub project_id: Option<Uuid>,
}
async fn f_1bz(r_9x: &AppState, r_9u: &Ts, project_id: Option<Uuid>) -> Result<Uuid, ApiError> {
    let r_9v = project_id.or(r_9u.user.project_id).ok_or_else(|| {
        ApiError::bad_request(
            "thiếu projectId (người gọi không thuộc dự án nào)",
        )
    })?;
    let r_9w = f_29a(&r_9x.pool, &r_9u.user).await?;
    if !can_touch_project(r_9u, &r_9w, r_9v) {
        return Err(ApiError::forbidden());
    }
    Ok(r_9v)
}
async fn f_1el(
    State(r_a3): State<AppState>,
    r_9y: Ts,
    Query(r_a2): Query<T7x>,
) -> Result<Json<T7y>, ApiError> {
    r_9y.require_perm(Permission::AccessView)?;
    let r_a1 = f_1bz(&r_a3, &r_9y, r_a2.project_id).await?;
    let (r_9z, r_a0) = access::f_1c0(&r_a3.pool, r_a1).await?;
    Ok(Json(T7y {
        project_id: Some(r_a1),
        phi_doc_tai_lieu: r_9z,
        phi_sao_trang: r_a0,
    }))
}
async fn f_24w(
    State(r_a7): State<AppState>,
    r_a4: Ts,
    Json(r_a5): Json<T7y>,
) -> Result<Json<T7y>, ApiError> {
    r_a4.require_perm(Permission::AccessReview)?;
    let r_a6 = f_1bz(&r_a7, &r_a4, r_a5.project_id).await?;
    if r_a5.phi_doc_tai_lieu < 0 || r_a5.phi_sao_trang < 0 {
        return Err(ApiError::bad_request("đơn giá không âm"));
    }
    access::f_2e2(&r_a7.pool, r_a6, r_a5.phi_doc_tai_lieu, r_a5.phi_sao_trang).await?;
    Ok(Json(T7y {
        project_id: Some(r_a6),
        phi_doc_tai_lieu: r_a5.phi_doc_tai_lieu,
        phi_sao_trang: r_a5.phi_sao_trang,
    }))
}
