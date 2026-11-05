// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::Permission;
use digitext_db::disposal::{self, T4v, T4w, T4x};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/disposal", get(f_1nd))
        .route("/disposal/sweep", post(f_2be))
        .route("/disposal/retention", post(set_retention))
        .route("/disposal/entries/{id}/note", post(set_note))
        .route("/disposal/entries/{id}/keep", post(keep_entry))
        .route("/disposal/batches", get(list_batches).post(create_batch))
        .route("/disposal/batches/{id}", get(f_1ed))
        .route("/disposal/batches/{id}/council", post(council))
        .route("/disposal/batches/{id}/appraise", post(appraise))
        .route("/disposal/batches/{id}/decide", post(decide))
        .route("/disposal/batches/{id}/execute", post(execute))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub document_id: Uuid,
    pub batch_id: Option<Uuid>,
    pub ly_do: String,
    pub dup_of: Option<Uuid>,
    pub ghi_chu: Option<String>,
    pub status: String,
    pub filename: String,
    pub ho_so_ref: Option<String>,
    pub page_count: Option<i32>,
    pub het_han_luu_tru: Option<DateTime<Utc>>,
    pub doc_status: String,
    pub created_at: DateTime<Utc>,
}
impl From<T4w> for EntryDto {
    fn from(r_c0: T4w) -> Self {
        EntryDto {
            id: r_c0.id,
            project_id: r_c0.project_id,
            document_id: r_c0.document_id,
            batch_id: r_c0.batch_id,
            ly_do: r_c0.ly_do,
            dup_of: r_c0.dup_of,
            ghi_chu: r_c0.ghi_chu,
            status: r_c0.status,
            filename: r_c0.filename,
            ho_so_ref: r_c0.ho_so_ref,
            page_count: r_c0.page_count,
            het_han_luu_tru: r_c0.het_han_luu_tru,
            doc_status: r_c0.doc_status,
            created_at: r_c0.created_at,
        }
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub nam: i32,
    pub status: String,
    pub hoi_dong_note: Option<String>,
    pub tham_dinh_note: Option<String>,
    pub tu_quyet: bool,
    pub quyet_dinh_so: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
impl From<T4v> for BatchDto {
    fn from(r_c1: T4v) -> Self {
        BatchDto {
            id: r_c1.id,
            project_id: r_c1.project_id,
            nam: r_c1.nam,
            status: r_c1.status,
            hoi_dong_note: r_c1.hoi_dong_note,
            tham_dinh_note: r_c1.tham_dinh_note,
            tu_quyet: r_c1.tu_quyet,
            quyet_dinh_so: r_c1.quyet_dinh_so,
            decided_at: r_c1.decided_at,
            executed_at: r_c1.executed_at,
            created_at: r_c1.created_at,
        }
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogDto {
    pub action: String,
    pub actor: Option<Uuid>,
    pub detail: Option<String>,
    pub document_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
impl From<T4x> for LogDto {
    fn from(r_c2: T4x) -> Self {
        LogDto {
            action: r_c2.action,
            actor: r_c2.actor,
            detail: r_c2.detail,
            document_id: r_c2.document_id,
            created_at: r_c2.created_at,
        }
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T16 {
    #[serde(flatten)]
    pub batch: BatchDto,
    pub entries: Vec<EntryDto>,
    pub log: Vec<LogDto>,
}
fn can_touch_project(r_c3: &Ts, r_c5: &Scope, r_c4: Uuid) -> bool {
    r_c3.can_touch_project(r_c5, r_c4)
}
async fn scope_projects(r_c8: &AppState, r_c6: &Ts) -> Result<Option<Vec<Uuid>>, ApiError> {
    let r_c7 = f_29a(&r_c8.pool, &r_c6.user).await?;
    Ok(r_c6.list_projects(r_c7))
}
async fn f_1a1(r_cc: &AppState, r_c9: &Ts, id: Uuid) -> Result<T4w, ApiError> {
    let r_ca = disposal::entry(&r_cc.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let r_cb = f_29a(&r_cc.pool, &r_c9.user).await?;
    if !can_touch_project(r_c9, &r_cb, r_ca.project_id) {
        return Err(ApiError::forbidden());
    }
    Ok(r_ca)
}
async fn f_u8(r_cg: &AppState, r_cd: &Ts, id: Uuid) -> Result<T4v, ApiError> {
    let r_ce = disposal::batch(&r_cg.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let r_cf = f_29a(&r_cg.pool, &r_cd.user).await?;
    if !can_touch_project(r_cd, &r_cf, r_ce.project_id) {
        return Err(ApiError::forbidden());
    }
    Ok(r_ce)
}
fn note_of(r_ci: &Option<String>) -> Option<&str> {
    r_ci.as_deref()
        .map(str::trim)
        .filter(|r_ch| !r_ch.is_empty())
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
async fn f_1nd(
    State(r_cn): State<AppState>,
    r_cj: Ts,
    Query(r_cl): Query<ListQuery>,
) -> Result<Json<Vec<EntryDto>>, ApiError> {
    r_cj.require_perm(Permission::DisposalView)?;
    let r_ck = scope_projects(&r_cn, &r_cj).await?;
    let r_cm = disposal::entries(
        &r_cn.pool,
        r_ck.as_deref(),
        r_cl.limit.unwrap_or(200).clamp(1, 500),
        r_cl.offset.unwrap_or(0).max(0),
    )
    .await?;
    Ok(Json(r_cm.into_iter().map(EntryDto::from).collect()))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tmb {
    pub trung_lap: usize,
    pub het_thoi_han: usize,
}
async fn f_2be(State(r_cr): State<AppState>, r_co: Ts) -> Result<Json<Tmb>, ApiError> {
    r_co.require_perm(Permission::DisposalReview)?;
    let r_cp = disposal::f_18b(&r_cr.pool).await?;
    let r_cq = disposal::f_1ah(&r_cr.pool).await?;
    Ok(Json(Tmb {
        trung_lap: r_cp.len(),
        het_thoi_han: r_cq.len(),
    }))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tj2 {
    pub document_id: Uuid,
    pub het_han: Option<DateTime<Utc>>,
}
async fn set_retention(
    State(r_cw): State<AppState>,
    r_cs: Ts,
    Json(r_ct): Json<Tj2>,
) -> Result<StatusCode, ApiError> {
    r_cs.require_perm(Permission::DisposalReview)?;
    let r_cu = digitext_db::documents::get(&r_cw.pool, r_ct.document_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let r_cv = f_29a(&r_cw.pool, &r_cs.user).await?;
    if !can_touch_project(&r_cs, &r_cv, r_cu.project_id) {
        return Err(ApiError::forbidden());
    }
    if !disposal::set_retention(&r_cw.pool, r_ct.document_id, r_ct.het_han).await? {
        return Err(ApiError::conflict(
            "tài liệu đã hủy — không đặt hạn".to_string(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteBody {
    pub note: Option<String>,
}
async fn set_note(
    State(r_d0): State<AppState>,
    r_cx: Ts,
    Path(id): Path<Uuid>,
    Json(r_cy): Json<NoteBody>,
) -> Result<StatusCode, ApiError> {
    r_cx.require_perm(Permission::DisposalReview)?;
    let r_cz = f_1a1(&r_d0, &r_cx, id).await?;
    disposal::set_note(&r_d0.pool, r_cz.id, note_of(&r_cy.note)).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn keep_entry(
    State(r_d4): State<AppState>,
    r_d1: Ts,
    Path(id): Path<Uuid>,
    Json(r_d2): Json<NoteBody>,
) -> Result<StatusCode, ApiError> {
    r_d1.require_perm(Permission::DisposalReview)?;
    let r_d3 = f_1a1(&r_d4, &r_d1, id).await?;
    let note = note_of(&r_d2.note).ok_or_else(|| {
        ApiError::bad_request("giữ lại phải ghi rõ lý do (lưu vết)")
    })?;
    if !disposal::keep_entry(&r_d4.pool, r_d3.id, Some(r_d1.user.id), note).await? {
        return Err(
            ApiError::conflict(
                "chỉ giữ lại được dòng đang 'Xem xét hủy' và chưa vào đợt"
                    .to_string(),
            ),
        );
    }
    Ok(StatusCode::NO_CONTENT)
}
async fn list_batches(
    State(r_d9): State<AppState>,
    r_d5: Ts,
    Query(r_d7): Query<ListQuery>,
) -> Result<Json<Vec<BatchDto>>, ApiError> {
    r_d5.require_perm(Permission::DisposalView)?;
    let r_d6 = scope_projects(&r_d9, &r_d5).await?;
    let r_d8 = disposal::batches(
        &r_d9.pool,
        r_d6.as_deref(),
        r_d7.limit.unwrap_or(50).clamp(1, 200),
        r_d7.offset.unwrap_or(0).max(0),
    )
    .await?;
    Ok(Json(r_d8.into_iter().map(BatchDto::from).collect()))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T37 {
    pub project_id: Option<Uuid>,
    pub nam: i32,
    pub entry_ids: Vec<Uuid>,
}
async fn create_batch(
    State(r_dd): State<AppState>,
    r_da: Ts,
    Json(r_db): Json<T37>,
) -> Result<(StatusCode, Json<BatchDto>), ApiError> {
    r_da.require_perm(Permission::DisposalReview)?;
    let project_id = r_db.project_id.or(r_da.user.project_id).ok_or_else(|| {
        ApiError::bad_request(
            "thiếu projectId (người gọi không thuộc dự án nào)",
        )
    })?;
    let r_dc = f_29a(&r_dd.pool, &r_da.user).await?;
    if !can_touch_project(&r_da, &r_dc, project_id) {
        return Err(ApiError::forbidden());
    }
    if r_db.entry_ids.is_empty() {
        return Err(ApiError::bad_request(
            "chọn tối thiểu 01 dòng Danh mục",
        ));
    }
    let Some(batch) = disposal::create_batch(
        &r_dd.pool,
        project_id,
        r_db.nam,
        &r_db.entry_ids,
        Some(r_da.user.id),
    )
    .await?
    else {
        return Err(
            ApiError::conflict(
                "có dòng không hợp lệ (khác dự án / đã vào đợt khác / đã xử lý)"
                    .to_string(),
            ),
        );
    };
    Ok((StatusCode::CREATED, Json(batch.into())))
}
async fn f_1ed(
    State(r_dg): State<AppState>,
    r_de: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<T16>, ApiError> {
    r_de.require_perm(Permission::DisposalView)?;
    let r_df = f_u8(&r_dg, &r_de, id).await?;
    let entries = disposal::f_u7(&r_dg.pool, r_df.id).await?;
    let log = disposal::f_1pf(&r_dg.pool, r_df.id).await?;
    Ok(Json(T16 {
        batch: r_df.into(),
        entries: entries.into_iter().map(EntryDto::from).collect(),
        log: log.into_iter().map(LogDto::from).collect(),
    }))
}
async fn council(
    State(r_dk): State<AppState>,
    r_dh: Ts,
    Path(id): Path<Uuid>,
    Json(r_dj): Json<NoteBody>,
) -> Result<StatusCode, ApiError> {
    r_dh.require_perm(Permission::DisposalReview)?;
    let r_di = f_u8(&r_dk, &r_dh, id).await?;
    let note = note_of(&r_dj.note)
        .ok_or_else(|| {
            ApiError::bad_request(
                "phải ghi kết quả làm việc của Hội đồng (Điều 49.2.c)",
            )
        })?;
    if !disposal::council(&r_dk.pool, r_di.id, note, Some(r_dh.user.id)).await? {
        return Err(ApiError::conflict(format!(
            "chỉ chốt Hội đồng khi đợt đang '{}' ≠ hoi_dong",
            r_di.status
        )));
    }
    Ok(StatusCode::NO_CONTENT)
}
async fn appraise(
    State(r_do): State<AppState>,
    r_dl: Ts,
    Path(id): Path<Uuid>,
    Json(r_dn): Json<NoteBody>,
) -> Result<StatusCode, ApiError> {
    r_dl.require_perm(Permission::DisposalReview)?;
    let r_dm = f_u8(&r_do, &r_dl, id).await?;
    let note = note_of(&r_dn.note).ok_or_else(|| {
        ApiError::bad_request("phải ghi nội dung ý kiến thẩm định")
    })?;
    if !disposal::appraise(&r_do.pool, r_dm.id, note, Some(r_dl.user.id)).await? {
        return Err(
            ApiError::conflict(
                format!(
                    "chỉ nhập thẩm định ở bước tham_dinh (đang là '{}')",
                    r_dm.status
                ),
            ),
        );
    }
    Ok(StatusCode::NO_CONTENT)
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T4k {
    pub quyet_dinh_so: String,
    #[serde(default)]
    pub tu_quyet: bool,
}
async fn decide(
    State(r_ds): State<AppState>,
    r_dp: Ts,
    Path(id): Path<Uuid>,
    Json(r_dr): Json<T4k>,
) -> Result<StatusCode, ApiError> {
    r_dp.require_perm(Permission::DisposalDecide)?;
    let r_dq = f_u8(&r_ds, &r_dp, id).await?;
    if r_dr.quyet_dinh_so.trim().is_empty() {
        return Err(ApiError::bad_request(
            "thiếu số Quyết định hủy",
        ));
    }
    if !disposal::decide(
        &r_ds.pool,
        r_dq.id,
        r_dr.quyet_dinh_so.trim(),
        r_dr.tu_quyet,
        Some(r_dp.user.id),
    )
    .await?
    {
        return Err(
            ApiError::conflict(
                "quyết định cần đợt ở bước tham_dinh và ĐÃ có ý kiến thẩm định (hoặc tuQuyet=true theo Điều 49.8)"
                    .to_string(),
            ),
        );
    }
    Ok(StatusCode::NO_CONTENT)
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T70 {
    pub disposed: usize,
}
async fn execute(
    State(r_e0): State<AppState>,
    r_dt: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<T70>, ApiError> {
    r_dt.require_perm(Permission::DisposalDecide)?;
    let r_du = f_u8(&r_e0, &r_dt, id).await?;
    let r_e1 = r_e0
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let Some(entries) = disposal::f_1ad(&r_e0.pool, r_du.id, Some(r_dt.user.id)).await? else {
        return Err(ApiError::conflict(format!(
            "chỉ thao tác hủy sau Quyết định (đang là '{}')",
            r_du.status
        )));
    };
    let mut disposed = 0usize;
    let mut goi: std::collections::BTreeSet<Uuid> = std::collections::BTreeSet::new();
    for r_dv in &entries {
        let mut r_dy = vec![
            digitext_storage::f_2fc(r_dv.project_id, r_dv.document_id),
            digitext_storage::f_2qs(r_dv.project_id, r_dv.document_id),
            digitext_storage::f_23q(r_dv.project_id, r_dv.document_id),
        ];
        for r_dz in 1..=r_dv.page_count.unwrap_or(0).max(0) {
            r_dy.push(digitext_storage::f_1xr(r_dv.document_id, r_dz));
        }
        if disposal::f_1d0(
            &r_e0.pool,
            r_dv,
            Some(r_dt.user.id),
            &format!("hủy {} object khỏi kho", r_dy.len()),
        )
        .await?
        {
            disposed += 1;
        }
        for r_dx in &r_dy {
            if let Err(r_dw) = r_e1.delete(r_dx).await {
                tracing::error!(
                    key = % r_dx, error = % r_dw,
                    "purge object sau khi chốt hủy lỗi — mồ côi, sẽ purge lại lần chạy sau"
                );
            }
        }
        let g: Option<Uuid> = sqlx::query_scalar("SELECT goi_id FROM documents WHERE id = $1")
            .bind(r_dv.document_id)
            .fetch_optional(&r_e0.pool)
            .await?
            .flatten();
        goi.extend(g);
    }
    let quyet_dinh = r_du.quyet_dinh_so.clone().unwrap_or_default();
    for g in goi {
        huy_goi_aip(&r_e0, r_e1.as_ref(), &r_du, g, &quyet_dinh, r_dt.user.id).await?;
    }
    Ok(Json(T70 { disposed }))
}
/// Zip gói AIP nhập (bản Lưu trữ) theo quyết định tiêu huỷ: hết tài liệu còn giữ trong gói → xoá zip khỏi kho WORM qua
/// `xoa_theo_quyet_dinh`, đánh dấu `packages.huy_*`; còn tài liệu chưa huỷ → giữ gói và ghi nhật ký (zip vẫn chứa bản của
/// tài liệu đã huỷ cho tới khi huỷ đủ hồ sơ). Chốt với anh Hiếu 5/11/2026.
async fn huy_goi_aip(
    st: &AppState,
    store: &dyn digitext_storage::ObjectStore,
    dot: &digitext_db::disposal::T4v,
    goi: Uuid,
    quyet_dinh: &str,
    actor: Uuid,
) -> Result<(), ApiError> {
    let row: Option<(Option<String>, i64, Option<String>)> = sqlx::query_as(
        "SELECT p.zip_key, (SELECT count(*) FROM documents d WHERE d.goi_id = p.id AND d.status <> 'disposed'), p.label \
         FROM packages p WHERE p.id = $1 AND p.huy_at IS NULL",
    )
    .bind(goi)
    .fetch_optional(&st.pool)
    .await?;
    let Some((zip_key, con_lai, label)) = row else { return Ok(()) };
    let (action, detail) = if con_lai > 0 {
        ("giu_goi_aip", format!("Giữ gói {}: còn {con_lai} tài liệu chưa huỷ — zip chỉ xoá khi huỷ hết tài liệu của gói", label.unwrap_or_default()))
    } else {
        if let Some(key) = zip_key.as_deref() {
            if let Err(e) = store.xoa_theo_quyet_dinh(key).await {
                tracing::error!(key, error = %e, "xoá zip gói theo quyết định tiêu huỷ lỗi");
                return Err(ApiError::internal());
            }
        }
        sqlx::query("UPDATE packages SET huy_at = now(), huy_quyet_dinh = $2 WHERE id = $1")
            .bind(goi)
            .bind(quyet_dinh)
            .execute(&st.pool)
            .await?;
        ("huy_goi_aip", format!("Xoá zip gói {} khỏi kho lưu trữ theo Quyết định {quyet_dinh}", label.unwrap_or_default()))
    };
    sqlx::query(
        "INSERT INTO disposal_log (id, project_id, batch_id, action, actor, detail) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::now_v7())
    .bind(dot.project_id)
    .bind(dot.id)
    .bind(action)
    .bind(actor)
    .bind(detail)
    .execute(&st.pool)
    .await?;
    Ok(())
}
