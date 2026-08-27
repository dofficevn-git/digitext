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
use digitext_db::packages::{self, Tdy};
use digitext_db::submissions::{self, Tdv, Te0, Tm7, Tm8};
use digitext_package::wire::Tq4;
use serde::{Deserialize, Serialize};
use std::io::Write as _;
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/submissions", get(f_1oe).post(f_26p))
        .route("/submissions/{id}", get(f_1ex))
        .route("/submissions/{id}/confirm", post(f_116))
        .route("/submissions/{id}/refuse", post(f_26m))
        .route("/submissions/{id}/reconcile", post(f_267))
        .route("/submissions/{id}/receive", post(f_266))
        .route(
            "/submissions/{id}/items/{item_id}/review",
            post(review_item),
        )
        .route("/submissions/{id}/approve", post(f_s6))
        .route("/submissions/{id}/complete", post(f_10r))
        .route("/submissions/{id}/returned", get(f_29t))
        .route("/submissions/{id}/report", post(f_12f))
        .route("/submissions/{id}/dossier", get(f_175))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tin {
    pub ma_ho_so: String,
    pub tieu_de: String,
    pub thoi_gian_bat_dau: Option<String>,
    pub thoi_gian_ket_thuc: Option<String>,
    pub so_trang: Option<i32>,
    pub muc_do_tiep_can: Option<String>,
    pub ghi_chu: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tio {
    pub project_id: Option<Uuid>,
    pub org_name: String,
    pub org_code: String,
    #[serde(default)]
    pub da_nop_giay: bool,
    pub summary: Option<String>,
    pub fond_code: String,
    pub phuong_an_phan_loai: Option<String>,
    pub cach_thuc: Option<String>,
    pub du_kien_thoi_gian: Option<String>,
    pub lien_he: Option<String>,
    pub ghi_chu: Option<String>,
    pub total_pages: Option<i64>,
    pub total_bytes: Option<i64>,
    pub han_nop: Option<String>,
    pub items: Vec<Tin>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub ma_yeu_cau: String,
    pub org_name: String,
    pub org_code: String,
    pub da_nop_giay: bool,
    pub so_lan_nop: i32,
    pub summary: Option<String>,
    pub fond_code: String,
    pub cach_thuc: String,
    pub status: String,
    pub decision_note: Option<String>,
    pub reconcile_note: Option<String>,
    pub reception_note: Option<String>,
    pub approval_doc: Option<String>,
    pub approval_note: Option<String>,
    pub total_pages: Option<i64>,
    pub total_bytes: Option<i64>,
    pub han_nop: Option<DateTime<Utc>>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub received_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl From<Tm8> for SubmissionDto {
    fn from(r_s7: Tm8) -> Self {
        SubmissionDto {
            id: r_s7.id,
            project_id: r_s7.project_id,
            ma_yeu_cau: r_s7.ma_yeu_cau,
            org_name: r_s7.org_name,
            org_code: r_s7.org_code,
            da_nop_giay: r_s7.da_nop_giay,
            so_lan_nop: r_s7.so_lan_nop,
            summary: r_s7.summary,
            fond_code: r_s7.fond_code,
            cach_thuc: r_s7.cach_thuc,
            status: r_s7.status,
            decision_note: r_s7.decision_note,
            reconcile_note: r_s7.reconcile_note,
            reception_note: r_s7.reception_note,
            approval_doc: r_s7.approval_doc,
            approval_note: r_s7.approval_note,
            total_pages: r_s7.total_pages,
            total_bytes: r_s7.total_bytes,
            han_nop: r_s7.han_nop,
            confirmed_at: r_s7.confirmed_at,
            received_at: r_s7.received_at,
            approved_at: r_s7.approved_at,
            completed_at: r_s7.completed_at,
            created_at: r_s7.created_at,
            updated_at: r_s7.updated_at,
        }
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDto {
    pub id: Uuid,
    pub stt: i32,
    pub ma_ho_so: String,
    pub tieu_de: String,
    pub thoi_gian_bat_dau: Option<String>,
    pub thoi_gian_ket_thuc: Option<String>,
    pub so_trang: Option<i32>,
    pub muc_do_tiep_can: Option<String>,
    pub ghi_chu: Option<String>,
    pub review_result: String,
    pub review_note: Option<String>,
}
impl From<Tm7> for ItemDto {
    fn from(r_s8: Tm7) -> Self {
        ItemDto {
            id: r_s8.id,
            stt: r_s8.stt,
            ma_ho_so: r_s8.ma_ho_so,
            tieu_de: r_s8.tieu_de,
            thoi_gian_bat_dau: r_s8.thoi_gian_bat_dau,
            thoi_gian_ket_thuc: r_s8.thoi_gian_ket_thuc,
            so_trang: r_s8.so_trang,
            muc_do_tiep_can: r_s8.muc_do_tiep_can,
            ghi_chu: r_s8.ghi_chu,
            review_result: r_s8.review_result,
            review_note: r_s8.review_note,
        }
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tm6 {
    #[serde(flatten)]
    pub submission: SubmissionDto,
    pub items: Vec<ItemDto>,
    pub packages: Vec<super::packages::PackageDto>,
    pub duplicate_in_system: Vec<String>,
}
fn can_touch_project(r_s9: &Ts, r_sb: &Scope, r_sa: Uuid) -> bool {
    r_s9.can_touch_project(r_sb, r_sa)
}
async fn load_in_scope(r_sf: &AppState, r_sc: &Ts, id: Uuid) -> Result<Tm8, ApiError> {
    let r_sd = submissions::get(&r_sf.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let r_se = f_29a(&r_sf.pool, &r_sc.user).await?;
    if !can_touch_project(r_sc, &r_se, r_sd.project_id) {
        return Err(ApiError::forbidden());
    }
    Ok(r_sd)
}
fn note_of(v: &Option<String>) -> Option<&str> {
    v.as_deref().map(str::trim).filter(|r_sg| !r_sg.is_empty())
}
fn f_1z3(r_sh: &str) -> Result<DateTime<Utc>, ApiError> {
    if let Ok(d) = chrono::NaiveDate::parse_from_str(r_sh, "%Y-%m-%d") {
        let r_sj = chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00");
        return Ok(d
            .and_hms_opt(0, 0, 0)
            .expect("00:00:00")
            .and_local_timezone(r_sj)
            .single()
            .expect("+07:00 không có DST")
            .with_timezone(&Utc));
    }
    DateTime::parse_from_rfc3339(r_sh)
        .map(|r_si| r_si.with_timezone(&Utc))
        .map_err(|_| {
            ApiError::bad_request("hanNop nhận 'YYYY-MM-DD' hoặc RFC 3339")
        })
}
fn f_1d7(n: i64) -> String {
    const C_OX: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = n as f64;
    let mut r_sk = 0;
    while v >= 1024.0 && r_sk < C_OX.len() - 1 {
        v /= 1024.0;
        r_sk += 1;
    }
    if r_sk == 0 {
        format!("{n} B")
    } else {
        format!("{v:.1} {}", C_OX[r_sk])
    }
}
async fn f_26p(
    State(r_st): State<AppState>,
    r_sl: Ts,
    Json(r_sq): Json<Tio>,
) -> Result<(StatusCode, Json<Tm6>), ApiError> {
    r_sl.require_perm(Permission::SubmissionCreate)?;
    let project_id = r_sq.project_id.or(r_sl.user.project_id).ok_or_else(|| {
        ApiError::bad_request(
            "thiếu projectId (người gọi không thuộc dự án nào)",
        )
    })?;
    let r_ss = f_29a(&r_st.pool, &r_sl.user).await?;
    if !can_touch_project(&r_sl, &r_ss, project_id) {
        return Err(ApiError::forbidden());
    }
    if r_sq.org_name.trim().is_empty() || r_sq.org_code.trim().is_empty() {
        return Err(ApiError::bad_request(
            "thiếu tên/mã cơ quan nộp",
        ));
    }
    if r_sq.fond_code.trim().is_empty() {
        return Err(ApiError::bad_request("thiếu mã phông"));
    }
    if r_sq.items.is_empty() {
        return Err(
            ApiError::bad_request(
                "Mục lục hồ sơ nộp cần tối thiểu 01 dòng (Điều 20.3.k)",
            ),
        );
    }
    let cach_thuc = r_sq.cach_thuc.as_deref().unwrap_or("truc_tuyen");
    if cach_thuc != "truc_tiep" && cach_thuc != "truc_tuyen" {
        return Err(ApiError::bad_request(
            "cachThuc nhận 'truc_tiep' hoặc 'truc_tuyen' (Điều 18)",
        ));
    }
    let han_nop = match note_of(&r_sq.han_nop) {
        Some(r_sp) => Some(f_1z3(r_sp)?),
        None => None,
    };
    let items: Vec<Tdv<'_>> = r_sq
        .items
        .iter()
        .map(|r_sn| Tdv {
            ma_ho_so: r_sn.ma_ho_so.as_str(),
            tieu_de: r_sn.tieu_de.as_str(),
            thoi_gian_bat_dau: r_sn.thoi_gian_bat_dau.as_deref(),
            thoi_gian_ket_thuc: r_sn.thoi_gian_ket_thuc.as_deref(),
            so_trang: r_sn.so_trang,
            muc_do_tiep_can: r_sn.muc_do_tiep_can.as_deref(),
            ghi_chu: r_sn.ghi_chu.as_deref(),
        })
        .collect();
    let result = submissions::register(
        &r_st.pool,
        Te0 {
            project_id,
            created_by: Some(r_sl.user.id),
            org_name: r_sq.org_name.trim(),
            org_code: r_sq.org_code.trim(),
            da_nop_giay: r_sq.da_nop_giay,
            summary: note_of(&r_sq.summary),
            fond_code: r_sq.fond_code.trim(),
            phuong_an_phan_loai: note_of(&r_sq.phuong_an_phan_loai),
            cach_thuc,
            du_kien_thoi_gian: note_of(&r_sq.du_kien_thoi_gian),
            lien_he: note_of(&r_sq.lien_he),
            ghi_chu: note_of(&r_sq.ghi_chu),
            total_pages: r_sq.total_pages,
            total_bytes: r_sq.total_bytes,
            han_nop,
        },
        &items,
    )
    .await;
    let (r_sr, r_so) = match result {
        Ok(v) => v,
        Err(e) if digitext_db::f_1jn(&e) => {
            return Err(
                ApiError::conflict(
                    "Mục lục có mã hồ sơ trùng lặp trong cùng lần nộp (Điều 24.1.a)"
                        .to_string(),
                ),
            );
        }
        Err(e) => return Err(e.into()),
    };
    if let Some(r_sm) = r_sr.han_nop {
        if r_sm < Utc::now() + chrono::Months::new(12) {
            if let Err(e) = digitext_db::notifications::f_1v8(
                    &r_st.pool,
                    r_sl.user.id,
                    Some(r_sr.id),
                    "sla_register_late",
                    &format!(
                        "Lần nộp {}: đăng ký MUỘN hơn 12 tháng trước hạn nộp {} (Điều 20.2)",
                        r_sr.ma_yeu_cau, r_sm.format("%d/%m/%Y")
                    ),
                )
                .await
            {
                tracing::warn!(
                    error = % e,
                    "không ghi được cảnh báo Điều 20.2"
                );
            }
        }
    }
    Ok((
        StatusCode::CREATED,
        Json(Tm6 {
            submission: r_sr.into(),
            items: r_so.into_iter().map(ItemDto::from).collect(),
            packages: Vec::new(),
            duplicate_in_system: Vec::new(),
        }),
    ))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
async fn f_1oe(
    State(r_sz): State<AppState>,
    r_su: Ts,
    Query(r_sw): Query<ListQuery>,
) -> Result<Json<Vec<SubmissionDto>>, ApiError> {
    r_su.require_perm(Permission::SubmissionView)?;
    let limit = r_sw.limit.unwrap_or(50).clamp(1, 200);
    let offset = r_sw.offset.unwrap_or(0).max(0);
    let r_sy = f_29a(&r_sz.pool, &r_su.user).await?;
    let r_sv = r_su.list_projects(r_sy);
    let r_sx = submissions::list(&r_sz.pool, r_sv.as_deref(), limit, offset).await?;
    Ok(Json(r_sx.into_iter().map(SubmissionDto::from).collect()))
}
async fn f_1ex(
    State(r_t4): State<AppState>,
    r_t0: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<Tm6>, ApiError> {
    r_t0.require_perm(Permission::SubmissionView)?;
    let r_t3 = load_in_scope(&r_t4, &r_t0, id).await?;
    let items = submissions::items(&r_t4.pool, id).await?;
    let packages = packages::list(&r_t4.pool, Some(&[r_t3.project_id]), 500, 0)
        .await?
        .into_iter()
        .filter(|r_t2| r_t2.submission_id == Some(id))
        .collect::<Vec<_>>();
    let r_t1 = submissions::f_18c(&r_t4.pool, r_t3.project_id, id).await?;
    Ok(Json(Tm6 {
        submission: r_t3.into(),
        items: items.into_iter().map(ItemDto::from).collect(),
        packages: packages
            .into_iter()
            .map(super::packages::PackageDto::from)
            .collect(),
        duplicate_in_system: r_t1,
    }))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteBody {
    pub note: Option<String>,
}
async fn f_116(
    State(r_t8): State<AppState>,
    r_t5: Ts,
    Path(id): Path<Uuid>,
    Json(r_t6): Json<NoteBody>,
) -> Result<Json<SubmissionDto>, ApiError> {
    r_t5.require_perm(Permission::SubmissionReview)?;
    let r_t7 = load_in_scope(&r_t8, &r_t5, id).await?;
    if !submissions::confirm(&r_t8.pool, r_t7.id, note_of(&r_t6.note)).await? {
        return Err(ApiError::conflict(format!(
            "không xác nhận được từ trạng thái '{}'",
            r_t7.status
        )));
    }
    refreshed(&r_t8, id).await
}
async fn f_26m(
    State(r_tc): State<AppState>,
    r_t9: Ts,
    Path(id): Path<Uuid>,
    Json(r_ta): Json<NoteBody>,
) -> Result<Json<SubmissionDto>, ApiError> {
    r_t9.require_perm(Permission::SubmissionReview)?;
    let r_tb = load_in_scope(&r_tc, &r_t9, id).await?;
    let note = note_of(&r_ta.note).ok_or_else(|| {
        ApiError::bad_request(
            "từ chối thu phải ghi rõ lý do (Điều 21.3.d)",
        )
    })?;
    if !submissions::refuse(&r_tc.pool, r_tb.id, Some(note)).await? {
        return Err(ApiError::conflict(format!(
            "không từ chối được từ trạng thái '{}'",
            r_tb.status
        )));
    }
    refreshed(&r_tc, id).await
}
async fn f_267(
    State(r_te): State<AppState>,
    r_td: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<SubmissionDto>, ApiError> {
    r_td.require_perm(Permission::SubmissionView)?;
    load_in_scope(&r_te, &r_td, id).await?;
    submissions::reconcile(&r_te.pool, id).await?;
    refreshed(&r_te, id).await
}
async fn f_266(
    State(r_ti): State<AppState>,
    r_tf: Ts,
    Path(id): Path<Uuid>,
    Json(r_tg): Json<NoteBody>,
) -> Result<Json<SubmissionDto>, ApiError> {
    r_tf.require_perm(Permission::SubmissionReview)?;
    let r_th = load_in_scope(&r_ti, &r_tf, id).await?;
    if !submissions::receive(&r_ti.pool, r_th.id, note_of(&r_tg.note)).await? {
        return Err(
            ApiError::conflict(
                format!(
                    "chỉ tiếp nhận được khi 'Chờ tiếp nhận' (đang là '{}')",
                    r_th.status
                ),
            ),
        );
    }
    refreshed(&r_ti, id).await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewBody {
    pub result: String,
    pub note: Option<String>,
}
async fn review_item(
    State(r_tn): State<AppState>,
    r_tj: Ts,
    Path((id, item_id)): Path<(Uuid, Uuid)>,
    Json(r_tk): Json<ReviewBody>,
) -> Result<StatusCode, ApiError> {
    r_tj.require_perm(Permission::SubmissionReview)?;
    load_in_scope(&r_tn, &r_tj, id).await?;
    if r_tk.result == "khong_dat" && note_of(&r_tk.note).is_none() {
        return Err(ApiError::bad_request(
            "không đạt phải nêu rõ lý do (Điều 24.2)",
        ));
    }
    let r_tm = submissions::items(&r_tn.pool, id)
        .await?
        .iter()
        .any(|r_tl| r_tl.id == item_id);
    if !r_tm {
        return Err(ApiError::not_found());
    }
    if !submissions::review_item(&r_tn.pool, item_id, &r_tk.result, note_of(&r_tk.note)).await? {
        return Err(ApiError::bad_request(
            "result nhận 'dat' hoặc 'khong_dat'".to_string(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tk {
    pub quyet_dinh_so: String,
    pub note: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tl {
    pub submission: SubmissionDto,
    pub converted_aip_ids: Vec<Uuid>,
}
async fn f_s6(
    State(r_tv): State<AppState>,
    r_tp: Ts,
    Path(id): Path<Uuid>,
    Json(r_tq): Json<Tk>,
) -> Result<Json<Tl>, ApiError> {
    r_tp.require_perm(Permission::SubmissionApprove)?;
    let r_tt = load_in_scope(&r_tv, &r_tp, id).await?;
    if r_tq.quyet_dinh_so.trim().is_empty() {
        return Err(ApiError::bad_request(
            "thiếu số Quyết định phê duyệt",
        ));
    }
    if !submissions::approve(
        &r_tv.pool,
        r_tt.id,
        r_tq.quyet_dinh_so.trim(),
        note_of(&r_tq.note),
    )
    .await?
    {
        return Err(
            ApiError::conflict(
                format!(
                    "chỉ phê duyệt được sau khi tiếp nhận (đang là '{}')",
                    r_tt.status
                ),
            ),
        );
    }
    let mut r_tr = Vec::new();
    for r_tu in packages::f_w8(&r_tv.pool, r_tt.id).await? {
        let r_tx: Tq4 = match serde_json::from_value(r_tu.request_json.clone()) {
            Ok(r_tw) => r_tw,
            Err(e) => {
                tracing::error!(
                    package = % r_tu.id, error = % e,
                    "request_json gói SIP không đọc được — bỏ qua chuyển hóa"
                );
                continue;
            }
        };
        let r_to = match r_tx.to_aip() {
            Ok(a) => a,
            Err(e) => {
                tracing::error!(
                    package = % r_tu.id, error = % e,
                    "chuyển SIP→AIP thất bại"
                );
                continue;
            }
        };
        let request_json = serde_json::to_value(&r_to).map_err(|_| ApiError::internal())?;
        let r_ts = packages::create(
            &r_tv.pool,
            Tdy {
                project_id: r_tt.project_id,
                requested_by: Some(r_tp.user.id),
                kind: &r_to.kind,
                ho_so_ref: r_tu.ho_so_ref.as_deref(),
                label: &r_to.display_label(),
                request_json: &request_json,
                submission_id: Some(r_tt.id),
                converted_from: Some(r_tu.id),
                stt_range: r_tu.stt_from.zip(r_tu.stt_to),
            },
        )
        .await?;
        r_tr.push(r_ts.id);
    }
    let refreshed = submissions::get(&r_tv.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(Tl {
        submission: refreshed.into(),
        converted_aip_ids: r_tr,
    }))
}
async fn f_10r(
    State(r_u0): State<AppState>,
    r_ty: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<SubmissionDto>, ApiError> {
    r_ty.require_perm(Permission::SubmissionApprove)?;
    let r_tz = load_in_scope(&r_u0, &r_ty, id).await?;
    if !submissions::complete(&r_u0.pool, r_tz.id).await? {
        return Err(ApiError::conflict(format!(
            "chỉ hoàn thành được sau phê duyệt (đang là '{}')",
            r_tz.status
        )));
    }
    refreshed(&r_u0, id).await
}
async fn refreshed(r_u2: &AppState, id: Uuid) -> Result<Json<SubmissionDto>, ApiError> {
    let r_u1 = submissions::get(&r_u2.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(r_u1.into()))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tj4 {
    #[serde(flatten)]
    pub item: ItemDto,
    pub sip_package_id: Option<Uuid>,
    pub sip_zip_name: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tj3 {
    pub submission: SubmissionDto,
    pub total_registered: usize,
    pub total_approved: usize,
    pub total_returned: usize,
    pub returned_items: Vec<Tj4>,
    pub sip_packages: Vec<super::packages::PackageDto>,
}
async fn f_29t(
    State(r_u7): State<AppState>,
    r_u3: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<Tj3>, ApiError> {
    r_u3.require_perm(Permission::SubmissionView)?;
    let r_u4 = load_in_scope(&r_u7, &r_u3, id).await?;
    if r_u4.status != "approved" && r_u4.status != "completed" {
        return Err(
            ApiError::conflict(
                format!(
                    "danh sách trả lại chỉ có sau phê duyệt (đang là '{}')",
                    r_u4.status
                ),
            ),
        );
    }
    let items = submissions::items(&r_u7.pool, id).await?;
    let total_registered = items.len();
    let total_approved = items.iter().filter(|i| i.review_result == "dat").count();
    let r_u6 = packages::f_w8(&r_u7.pool, id).await?;
    let returned: Vec<Tj4> = items
        .into_iter()
        .filter(|i| i.review_result == "khong_dat")
        .map(|i| {
            let r_u5 = r_u6
                .iter()
                .find(|p| p.ho_so_ref.as_deref() == Some(&i.ma_ho_so));
            Tj4 {
                sip_package_id: r_u5.map(|p| p.id),
                sip_zip_name: r_u5.and_then(|p| p.zip_name.clone()),
                item: i.into(),
            }
        })
        .collect();
    Ok(Json(Tj3 {
        total_registered,
        total_approved,
        total_returned: returned.len(),
        returned_items: returned,
        sip_packages: r_u6
            .into_iter()
            .map(super::packages::PackageDto::from)
            .collect(),
        submission: r_u4.into(),
    }))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct To4 {
    pub ghi_chu: Option<String>,
    pub loai: Option<String>,
}
async fn f_12f(
    State(r_uf): State<AppState>,
    r_u8: Ts,
    Path(id): Path<Uuid>,
    Json(r_u9): Json<To4>,
) -> Result<(StatusCode, Json<super::records::RecordDto>), ApiError> {
    r_u8.require_perm(Permission::SubmissionReview)?;
    let r_ue = load_in_scope(&r_uf, &r_u8, id).await?;
    if r_ue.status != "approved" && r_ue.status != "completed" {
        return Err(ApiError::conflict(format!(
            "biên bản thu nộp lập sau phê duyệt (đang là '{}')",
            r_ue.status
        )));
    }
    let items = submissions::items(&r_uf.pool, id).await?;
    let r_uh = items.iter().filter(|i| i.review_result == "dat").count() as i64;
    let r_uj = items
        .iter()
        .filter(|i| i.review_result == "khong_dat")
        .count() as i64;
    let r_ug = r_ue
        .approved_at
        .unwrap_or(r_ue.updated_at)
        .format("%d/%m/%Y")
        .to_string();
    let ghi_chu = note_of(&r_u9.ghi_chu).map(str::to_owned);
    let loai = r_u9.loai.as_deref().unwrap_or("thu_nop");
    let r_uc = match loai {
        "thu_nop" => {
            let r_ui: i64 = packages::f_w8(&r_uf.pool, id)
                .await?
                .iter()
                .map(|r_ub| r_ub.document_count as i64)
                .sum();
            serde_json::to_value(crate::bienban::To3 {
                ma_dang_ky: r_ue.ma_yeu_cau.clone(),
                ma_co_quan: r_ue.org_code.clone(),
                ten_co_quan: r_ue.org_name.clone(),
                so_lan_nop: r_ue.so_lan_nop,
                tong_dang_ky: items.len() as i64,
                tong_thu_nop: r_ui,
                tong_duoc_duyet: r_uh,
                tong_tra_lai: r_uj,
                thoi_gian_thu_nop: r_ug,
                nguoi_thu_nop: r_u8.user.display_name.clone(),
                ghi_chu,
            })
        }
        "giao_nhan" => {
            serde_json::to_value(crate::bienban::T8q {
                ma_dang_ky: r_ue.ma_yeu_cau.clone(),
                ma_co_quan: r_ue.org_code.clone(),
                ten_co_quan: r_ue.org_name.clone(),
                so_lan_nop: r_ue.so_lan_nop,
                tong_giao_nhan: r_uh,
                tong_dung_luong: r_ue.total_bytes.map(f_1d7),
                thoi_gian: r_ug,
                ben_giao: r_ue.org_name.clone(),
                ben_nhan: r_u8.user.display_name.clone(),
                ghi_chu,
            })
        }
        "tra_lai" => {
            let r_ud = if r_ue.cach_thuc == "truc_tiep" {
                "Chuyển vào thiết bị lưu trữ để bàn giao (Điều 25.4.a)"
            } else {
                "Tải từ tài khoản cơ quan nộp trên Hệ thống (Điều 25.4.b)"
            };
            serde_json::to_value(crate::bienban::Tob {
                ma_dang_ky: r_ue.ma_yeu_cau.clone(),
                ma_co_quan: r_ue.org_code.clone(),
                ten_co_quan: r_ue.org_name.clone(),
                so_lan_nop: r_ue.so_lan_nop,
                tong_tra_lai: r_uj,
                danh_sach: items
                    .iter()
                    .filter(|i| i.review_result == "khong_dat")
                    .map(|i| crate::bienban::Toc {
                        ma_ho_so: i.ma_ho_so.clone(),
                        tieu_de: i.tieu_de.clone(),
                        ly_do: i.review_note.clone(),
                    })
                    .collect(),
                phuong_thuc_tra: r_ud.to_string(),
                thoi_gian: r_ug,
                ben_giao: r_u8.user.display_name.clone(),
                ben_nhan: r_ue.org_name.clone(),
                ghi_chu,
            })
        }
        other => {
            return Err(
                ApiError::bad_request(
                    format!(
                        "loai nhận 'thu_nop' | 'giao_nhan' | 'tra_lai' (không nhận '{other}')"
                    ),
                ),
            );
        }
    }
        .map_err(|_| ApiError::internal())?;
    let r_ua =
        super::records::f_12e(&r_uf, &r_u8, r_ue.project_id, loai, &r_uc, Some(r_ue.id)).await?;
    Ok((StatusCode::CREATED, Json(r_ua)))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct To8 {
    kind: String,
    message: String,
    created_at: DateTime<Utc>,
}
fn f_2qf(r_uk: DateTime<Utc>) -> String {
    r_uk.with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00"))
        .format("%d/%m/%Y %H:%M")
        .to_string()
}
fn f_16w(
    r_uu: &Tm8,
    items: &[Tm7],
    r_ur: &[digitext_db::packages::Tgc],
    r_ut: &[digitext_db::records::RecordRow],
    timeline: &[To8],
) -> Vec<digitext_pdf::FormBlock> {
    use digitext_pdf::FormBlock;
    let r_ul = |r_uo: &str, r_uv: String| FormBlock::Field {
        label: r_uo.to_string(),
        value: r_uv,
    };
    let mut b = vec![
        r_ul(
            "Mã đăng ký yêu cầu thu nộp",
            r_uu.ma_yeu_cau.clone(),
        ),
        r_ul(
            "Cơ quan, tổ chức nộp",
            format!("{} ({})", r_uu.org_name, r_uu.org_code),
        ),
        r_ul("Mã phông", r_uu.fond_code.clone()),
        r_ul(
            "Cách thức nộp (Điều 18)",
            r_uu.cach_thuc.clone(),
        ),
        r_ul("Số lần nộp", r_uu.so_lan_nop.to_string()),
        r_ul("Trạng thái", r_uu.status.clone()),
    ];
    if let Some(r_um) = r_uu.han_nop {
        b.push(r_ul(
            "Hạn nộp tài liệu (Điều 20.2)",
            f_2qf(r_um),
        ));
    }
    b.push(FormBlock::SectionHeading(
        "I. CÁC MỐC QUÁ TRÌNH".into(),
    ));
    let mut r_up = vec![format!(
        "Đăng ký nộp (Điều 20): {}",
        f_2qf(r_uu.created_at)
    )];
    if let Some(t) = r_uu.confirmed_at {
        let note = r_uu.decision_note.as_deref().unwrap_or("");
        r_up.push(format!(
            "Xác nhận đồng ý thu (Điều 21.2): {} {note}",
            f_2qf(t)
        ));
    }
    if let Some(n) = &r_uu.reconcile_note {
        r_up.push(format!(
            "Đối chiếu Mục lục (Điều 22.3.d): {n}"
        ));
    }
    if let Some(t) = r_uu.received_at {
        r_up.push(format!(
            "Tiếp nhận (Điều 23): {}",
            f_2qf(t)
        ));
    }
    if let Some(t) = r_uu.approved_at {
        r_up.push(format!(
            "Phê duyệt (Điều 25.1): {} — Quyết định {}",
            f_2qf(t),
            r_uu.approval_doc.as_deref().unwrap_or("")
        ));
    }
    if let Some(t) = r_uu.completed_at {
        r_up.push(format!(
            "Hoàn thành thu nộp (Điều 25.5): {}",
            f_2qf(t)
        ));
    }
    b.push(FormBlock::Bullets(r_up));
    b.push(FormBlock::SectionHeading(
        "II. KẾT QUẢ XỬ LÝ NGHIỆP VỤ".into(),
    ));
    let dat = items.iter().filter(|i| i.review_result == "dat").count();
    let r_un = items
        .iter()
        .filter(|i| i.review_result == "khong_dat")
        .count();
    b.push(r_ul(
        "Tổng số hồ sơ theo đăng ký (Mục lục)",
        items.len().to_string(),
    ));
    b.push(r_ul(
        "Tổng số được duyệt",
        dat.to_string(),
    ));
    b.push(r_ul("Tổng số trả lại", r_un.to_string()));
    if !r_ur.is_empty() {
        b.push(FormBlock::Paragraph(
            "Các gói tin của lần nộp:".into(),
        ));
        b.push(FormBlock::Bullets(
            r_ur.iter()
                .map(|r_uq| {
                    format!(
                        "{} — {} — {}{}",
                        r_uq.kind,
                        r_uq.label,
                        r_uq.status,
                        r_uq.objid
                            .as_deref()
                            .map(|o| format!(" — {o}"))
                            .unwrap_or_default()
                    )
                })
                .collect(),
        ));
    }
    b.push(FormBlock::SectionHeading(
        "III. BIÊN BẢN ĐÃ LẬP".into(),
    ));
    if r_ut.is_empty() {
        b.push(FormBlock::Paragraph(
            "(Chưa lập biên bản nào.)".into(),
        ));
    } else {
        b.push(FormBlock::Bullets(
            r_ut.iter()
                .map(|r_us| {
                    format!(
                        "{} — {} ({})",
                        r_us.so_bien_ban,
                        r_us.title,
                        f_2qf(r_us.created_at)
                    )
                })
                .collect(),
        ));
    }
    b.push(FormBlock::SectionHeading(
        "IV. NHẬT KÝ THÔNG BÁO".into(),
    ));
    if timeline.is_empty() {
        b.push(FormBlock::Paragraph(
            "(Không có thông báo.)".into(),
        ));
    } else {
        b.push(FormBlock::Bullets(
            timeline
                .iter()
                .map(|t| format!("{}: {}", f_2qf(t.created_at), t.message))
                .collect(),
        ));
    }
    b
}
async fn f_175(
    State(r_v7): State<AppState>,
    r_uw: Ts,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    r_uw.require_perm(Permission::SubmissionView)?;
    let r_v6 = load_in_scope(&r_v7, &r_uw, id).await?;
    if r_v6.status != "approved" && r_v6.status != "completed" {
        return Err(ApiError::conflict(format!(
            "hồ sơ quá trình lập sau phê duyệt (đang là '{}')",
            r_v6.status
        )));
    }
    let r_v8 = r_v7
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let items = submissions::items(&r_v7.pool, id).await?;
    let r_v3: Vec<_> = packages::list(&r_v7.pool, Some(&[r_v6.project_id]), 500, 0)
        .await?
        .into_iter()
        .filter(|r_v2| r_v2.submission_id == Some(id))
        .collect();
    let r_v5 = digitext_db::records::list_for_submission(&r_v7.pool, id).await?;
    let timeline: Vec<To8> = digitext_db::notifications::list_for_submission(&r_v7.pool, id)
        .await?
        .into_iter()
        .map(|n| To8 {
            kind: n.kind,
            message: n.message,
            created_at: n.created_at,
        })
        .collect();
    let r_uy = digitext_pdf::f_1cq()
        .ok_or_else(|| {
            ApiError::service_unavailable(
                "không tìm thấy font TTF cho hồ sơ quá trình — đặt DIGITEXT_CERT_FONT",
            )
        })?;
    let font = std::fs::read(&r_uy).map_err(|e| {
        tracing::error!(
            path = % r_uy.display(), error = % e, "đọc font lỗi"
        );
        ApiError::internal()
    })?;
    let r_ux = f_16w(&r_v6, &items, &r_v3, &r_v5, &timeline);
    let pdf = digitext_pdf::f_ve(
            "Hồ sơ quá trình thu nộp hồ sơ, tài liệu",
            Some(
                "(Theo điểm d khoản 5 Điều 25 — Thông tư 05/2025/TT-BNV)",
            ),
            &r_ux,
            &font,
        )
        .map_err(|e| {
            tracing::error!(
                error = % e, "sinh PDF hồ sơ quá trình lỗi"
            );
            ApiError::internal()
        })?;
    let json = serde_json::json!(
        { "submission" : SubmissionDto::from(r_v6.clone()), "items" : items.into_iter()
        .map(ItemDto::from).collect::< Vec < _ >> (), "packages" : r_v3.into_iter()
        .map(super::packages::PackageDto::from).collect::< Vec < _ >> (), "records" :
        r_v5.iter().cloned().map(super::records::RecordDto::from).collect::< Vec < _ >>
        (), "timeline" : timeline, }
    );
    let r_v9 = |e: zip::result::ZipError| {
        tracing::error!(
            error = % e, "dựng ZIP hồ sơ quá trình lỗi"
        );
        ApiError::internal()
    };
    let r_uz = |e: std::io::Error| {
        tracing::error!(error = % e, "ghi ZIP hồ sơ quá trình lỗi");
        ApiError::internal()
    };
    let mut r_va = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let r_v1 = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    r_va.start_file("qua-trinh.json", r_v1).map_err(r_v9)?;
    r_va.write_all(&serde_json::to_vec_pretty(&json).map_err(|_| ApiError::internal())?)
        .map_err(r_uz)?;
    r_va.start_file("ho-so-qua-trinh.pdf", r_v1).map_err(r_v9)?;
    r_va.write_all(&pdf).map_err(r_uz)?;
    for r_v4 in &r_v5 {
        match r_v8.get(&r_v4.pdf_key).await {
            Ok(Some(bytes)) => {
                let r_v0 = format!("bien-ban/{}.pdf", r_v4.so_bien_ban.replace('/', "-"));
                r_va.start_file(r_v0, r_v1).map_err(r_v9)?;
                r_va.write_all(&bytes).map_err(r_uz)?;
            }
            Ok(None) => {
                tracing::warn!(
                    record = % r_v4.id, key = % r_v4.pdf_key,
                    "PDF biên bản không còn trong kho"
                );
            }
            Err(e) => {
                tracing::error!(
                    record = % r_v4.id, error = % e,
                    "đọc PDF biên bản lỗi"
                );
                return Err(ApiError::internal());
            }
        }
    }
    let bytes = r_va.finish().map_err(r_v9)?.into_inner();
    let safe_ma: String = r_v6.ma_yeu_cau.replace('/', "-");
    Ok((
        [
            (header::CONTENT_TYPE, "application/zip".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"ho-so-qua-trinh-{safe_ma}.zip\""),
            ),
        ],
        Body::from(bytes),
    )
        .into_response())
}
