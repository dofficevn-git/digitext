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
use digitext_db::info::{self, T9v, Tdu};
use digitext_db::{notifications, projects};
use digitext_pdf::FormBlock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/info", get(list_requests).post(create_request))
        .route("/info/{id}", get(get_request))
        .route("/info/{id}/approve", post(approve_request))
        .route("/info/{id}/refuse", post(refuse_request))
        .route("/info/{id}/pay", post(confirm_payment))
        .route("/info/{id}/generate", post(generate))
        .route("/info/{id}/fulfill", post(fulfill))
        .route("/info/{id}/download", get(download))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3a {
    pub project_id: Option<Uuid>,
    pub hinh_thuc: String,
    pub chu_de: String,
    pub muc_dich: String,
    pub phuong_thuc: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub requester: Option<Uuid>,
    pub hinh_thuc: String,
    pub chu_de: String,
    pub muc_dich: String,
    pub phuong_thuc: Option<String>,
    pub status: String,
    pub review_note: Option<String>,
    pub phi: Option<i64>,
    pub paid_at: Option<DateTime<Utc>>,
    pub co_ket_qua: bool,
    pub bien_tap: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl From<T9v> for InfoDto {
    fn from(r_i8: T9v) -> Self {
        InfoDto {
            id: r_i8.id,
            project_id: r_i8.project_id,
            requester: r_i8.requester,
            hinh_thuc: r_i8.hinh_thuc,
            chu_de: r_i8.chu_de,
            muc_dich: r_i8.muc_dich,
            phuong_thuc: r_i8.phuong_thuc,
            status: r_i8.status,
            review_note: r_i8.review_note,
            phi: r_i8.phi,
            paid_at: r_i8.paid_at,
            co_ket_qua: r_i8.ket_qua_key.is_some(),
            bien_tap: r_i8.bien_tap,
            approved_at: r_i8.approved_at,
            fulfilled_at: r_i8.fulfilled_at,
            created_at: r_i8.created_at,
            updated_at: r_i8.updated_at,
        }
    }
}
fn can_touch_project(r_i9: &Ts, r_ib: &Scope, r_ia: Uuid) -> bool {
    r_i9.can_touch_project(r_ib, r_ia)
}
async fn load_in_scope(r_if: &AppState, r_ic: &Ts, id: Uuid) -> Result<T9v, ApiError> {
    let r_id = info::get(&r_if.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let r_ie = f_29a(&r_if.pool, &r_ic.user).await?;
    if !can_touch_project(r_ic, &r_ie, r_id.project_id) {
        return Err(ApiError::forbidden());
    }
    Ok(r_id)
}
fn require_owner_or_reviewer(r_ig: &Ts, r_ih: &T9v) -> Result<(), ApiError> {
    if r_ih.requester == Some(r_ig.user.id) || r_ig.has_perm(Permission::AccessReview) {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}
fn note_of(r_ij: &Option<String>) -> Option<&str> {
    r_ij.as_deref()
        .map(str::trim)
        .filter(|r_ii| !r_ii.is_empty())
}
async fn notify_requester(r_im: &AppState, r_il: &T9v, r_ik: &str) {
    if let Some(r_in) = r_il.requester {
        if let Err(e) = notifications::f_1v8(&r_im.pool, r_in, None, "info_status", r_ik).await {
            tracing::warn!(
                error = % e,
                "không ghi được thông báo cung cấp thông tin"
            );
        }
    }
}
async fn create_request(
    State(r_is): State<AppState>,
    r_io: Ts,
    Json(r_ip): Json<T3a>,
) -> Result<(StatusCode, Json<InfoDto>), ApiError> {
    r_io.require_perm(Permission::AccessRequest)?;
    let project_id = r_ip.project_id.or(r_io.user.project_id).ok_or_else(|| {
        ApiError::bad_request(
            "thiếu projectId (người gọi không thuộc dự án nào)",
        )
    })?;
    let r_ir = f_29a(&r_is.pool, &r_io.user).await?;
    if !can_touch_project(&r_io, &r_ir, project_id) {
        return Err(ApiError::forbidden());
    }
    if !matches!(
        r_ip.hinh_thuc.as_str(),
        "danh_muc" | "trich_xuat" | "tong_hop"
    ) {
        return Err(
            ApiError::bad_request(
                "hinhThuc nhận 'danh_muc' | 'trich_xuat' | 'tong_hop' (Điều 44)",
            ),
        );
    }
    if r_ip.chu_de.trim().is_empty() {
        return Err(ApiError::bad_request(
            "thiếu chủ đề cụ thể (Điều 44.1.a)",
        ));
    }
    if r_ip.muc_dich.trim().is_empty() {
        return Err(ApiError::bad_request(
            "phải khai mục đích sử dụng (Điều 45.3)",
        ));
    }
    let r_iq = info::create(
        &r_is.pool,
        Tdu {
            project_id,
            requester: Some(r_io.user.id),
            hinh_thuc: &r_ip.hinh_thuc,
            chu_de: r_ip.chu_de.trim(),
            muc_dich: r_ip.muc_dich.trim(),
            phuong_thuc: note_of(&r_ip.phuong_thuc),
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(r_iq.into())))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
async fn list_requests(
    State(r_ix): State<AppState>,
    r_it: Ts,
    Query(r_iu): Query<ListQuery>,
) -> Result<Json<Vec<InfoDto>>, ApiError> {
    r_it.require_perm(Permission::AccessView)?;
    let limit = r_iu.limit.unwrap_or(50).clamp(1, 200);
    let offset = r_iu.offset.unwrap_or(0).max(0);
    let r_iw = f_29a(&r_ix.pool, &r_it.user).await?;
    let projects = r_it.list_projects(r_iw);
    let rows = info::list(&r_ix.pool, projects.as_deref(), limit, offset).await?;
    let rows: Vec<T9v> = if r_it.has_perm(Permission::AccessReview) {
        rows
    } else {
        rows.into_iter()
            .filter(|r_iv| r_iv.requester == Some(r_it.user.id))
            .collect()
    };
    Ok(Json(rows.into_iter().map(InfoDto::from).collect()))
}
async fn get_request(
    State(r_j0): State<AppState>,
    r_iy: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<InfoDto>, ApiError> {
    r_iy.require_perm(Permission::AccessView)?;
    let r_iz = load_in_scope(&r_j0, &r_iy, id).await?;
    require_owner_or_reviewer(&r_iy, &r_iz)?;
    Ok(Json(r_iz.into()))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteBody {
    pub note: Option<String>,
}
async fn approve_request(
    State(r_j4): State<AppState>,
    r_j1: Ts,
    Path(id): Path<Uuid>,
    Json(r_j2): Json<NoteBody>,
) -> Result<Json<InfoDto>, ApiError> {
    r_j1.require_perm(Permission::AccessReview)?;
    let r_j3 = load_in_scope(&r_j4, &r_j1, id).await?;
    let Some(approved) = info::approve(&r_j4.pool, r_j3.id, note_of(&r_j2.note)).await? else {
        return Err(ApiError::conflict(format!(
            "chỉ duyệt được yêu cầu chờ duyệt (đang là '{}')",
            r_j3.status
        )));
    };
    let phi = approved.phi.unwrap_or(0);
    notify_requester(
            &r_j4,
            &approved,
            &format!(
                "Yêu cầu cung cấp thông tin đã DUYỆT — phí {phi} đồng (Điều 46.1.c)"
            ),
        )
        .await;
    Ok(Json(approved.into()))
}
async fn refuse_request(
    State(r_j9): State<AppState>,
    r_j5: Ts,
    Path(id): Path<Uuid>,
    Json(r_j6): Json<NoteBody>,
) -> Result<Json<InfoDto>, ApiError> {
    r_j5.require_perm(Permission::AccessReview)?;
    let r_j8 = load_in_scope(&r_j9, &r_j5, id).await?;
    let note = note_of(&r_j6.note).ok_or_else(|| {
        ApiError::bad_request(
            "từ chối phải nêu rõ lý do (Điều 46.1.c)",
        )
    })?;
    if !info::refuse(&r_j9.pool, r_j8.id, note).await? {
        return Err(
            ApiError::conflict(
                format!(
                    "chỉ từ chối được yêu cầu chờ duyệt (đang là '{}')",
                    r_j8.status
                ),
            ),
        );
    }
    let r_j7 = info::get(&r_j9.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    notify_requester(
        &r_j9,
        &r_j7,
        &format!("Yêu cầu cung cấp thông tin bị TỪ CHỐI: {note}"),
    )
    .await;
    Ok(Json(r_j7.into()))
}
async fn confirm_payment(
    State(r_jd): State<AppState>,
    r_ja: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<InfoDto>, ApiError> {
    r_ja.require_perm(Permission::AccessReview)?;
    let r_jc = load_in_scope(&r_jd, &r_ja, id).await?;
    if !info::confirm_payment(&r_jd.pool, r_jc.id).await? {
        return Err(
            ApiError::conflict(
                "chỉ xác nhận thanh toán cho yêu cầu đã duyệt và chưa thanh toán"
                    .to_string(),
            ),
        );
    }
    let r_jb = info::get(&r_jd.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    notify_requester(
        &r_jd,
        &r_jb,
        "Đã xác nhận thanh toán phí (Điều 46.4)",
    )
    .await;
    Ok(Json(r_jb.into()))
}
fn f_2pl(r_je: &str, r_jh: &str, r_jg: usize) -> String {
    let r_jf = r_je.to_lowercase();
    let r_jj = r_jf.find(&r_jh.to_lowercase()).unwrap_or(0);
    let r_jl = r_je[..r_jj].chars().count().saturating_sub(r_jg / 4);
    let r_jk: String = r_je.chars().skip(r_jl).take(r_jg).collect();
    let mut r_ji = String::new();
    if r_jl > 0 {
        r_ji.push('…');
    }
    r_ji.push_str(r_jk.trim());
    if r_je.chars().count() > r_jl + r_jg {
        r_ji.push('…');
    }
    r_ji
}
fn f_1kc(project_id: Uuid, id: Uuid) -> String {
    format!("{project_id}-info/{id}.pdf")
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T8k {
    pub bien_tap: Option<String>,
}
async fn generate(
    State(r_jv): State<AppState>,
    r_jm: Ts,
    Path(id): Path<Uuid>,
    Json(r_jo): Json<T8k>,
) -> Result<Json<InfoDto>, ApiError> {
    r_jm.require_perm(Permission::AccessReview)?;
    let r_ju = load_in_scope(&r_jv, &r_jm, id).await?;
    if r_ju.status != "approved" {
        return Err(
            ApiError::conflict(
                format!(
                    "chỉ kết xuất sau khi duyệt và trước khi trả (đang là '{}')",
                    r_ju.status
                ),
            ),
        );
    }
    let r_jw = r_jv
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let (quota, _) = info::f_25g(&r_jv.pool, r_ju.project_id).await?;
    let org = projects::get(&r_jv.pool, r_ju.project_id)
        .await?
        .map(|r_js| r_js.name)
        .unwrap_or_else(|| "Lưu trữ lịch sử".to_string());
    let mut r_jn: Vec<FormBlock> = vec![
        FormBlock::Field {
            label: "Chủ đề".into(),
            value: r_ju.chu_de.clone(),
        },
        FormBlock::Field {
            label: "Mục đích sử dụng".into(),
            value: r_ju.muc_dich.clone(),
        },
    ];
    let r_jy;
    let r_jx = match r_ju.hinh_thuc.as_str() {
        "danh_muc" => {
            let hits = info::f_x8(&r_jv.pool, r_ju.project_id, &r_ju.chu_de, quota + 1).await?;
            r_jy = hits.len() as i64 > quota;
            let hits = &hits[..hits.len().min(quota as usize)];
            r_jn.push(FormBlock::SectionHeading(format!(
                "DANH MỤC TÀI LIỆU ({} mục)",
                hits.len()
            )));
            r_jn.push(FormBlock::Bullets(
                hits.iter()
                    .map(|h| {
                        format!(
                            "{} | {} | {} trang | {}",
                            h.ho_so_ref.as_deref().unwrap_or("—"),
                            h.filename,
                            h.page_count.unwrap_or(0),
                            h.uploaded_at.format("%d/%m/%Y"),
                        )
                    })
                    .collect(),
            ));
            "DANH MỤC TÀI LIỆU LƯU TRỮ THEO CHỦ ĐỀ"
        }
        "trich_xuat" => {
            let hits = info::f_1b9(&r_jv.pool, r_ju.project_id, &r_ju.chu_de, quota + 1).await?;
            r_jy = hits.len() as i64 > quota;
            let hits = &hits[..hits.len().min(quota as usize)];
            r_jn.push(FormBlock::SectionHeading(format!(
                "NỘI DUNG TRÍCH XUẤT NGUYÊN VĂN ({} đoạn)",
                hits.len()
            )));
            for h in hits {
                r_jn.push(FormBlock::Field {
                    label: "Địa chỉ lưu trữ".into(),
                    value: format!(
                        "{} — {} — trang {} (liên kết gốc: {})",
                        h.ho_so_ref.as_deref().unwrap_or("—"),
                        h.filename,
                        h.page_no,
                        h.document_id
                    ),
                });
                r_jn.push(FormBlock::Paragraph(format!(
                    "“{}”",
                    f_2pl(&h.content, &r_ju.chu_de, 400)
                )));
            }
            "THÔNG TIN TRÍCH XUẤT TỪ TÀI LIỆU LƯU TRỮ SỐ"
        }
        _ => {
            let rows_ = info::f_2i4(&r_jv.pool, r_ju.project_id, &r_ju.chu_de, quota + 1).await?;
            r_jy = rows_.len() as i64 > quota;
            let rows_ = &rows_[..rows_.len().min(quota as usize)];
            let tong_tl: i64 = rows_.iter().map(|r| r.so_tai_lieu).sum();
            let tong_trang: i64 = rows_.iter().map(|r| r.tong_trang).sum();
            r_jn.push(FormBlock::SectionHeading(
                "BÁO CÁO TỔNG HỢP TỰ ĐỘNG".into(),
            ));
            r_jn.push(FormBlock::Field {
                label: "Tổng số hồ sơ liên quan".into(),
                value: rows_.len().to_string(),
            });
            r_jn.push(FormBlock::Field {
                label: "Tổng số tài liệu / tổng số trang".into(),
                value: format!("{tong_tl} / {tong_trang}"),
            });
            r_jn.push(
                FormBlock::Bullets(
                    rows_
                        .iter()
                        .map(|r| {
                            format!(
                                "Hồ sơ {}: {} tài liệu, {} trang, {} trang khớp nội dung",
                                r.ho_so_ref.as_deref().unwrap_or("—"), r.so_tai_lieu, r
                                .tong_trang, r.so_trang_khop,
                            )
                        })
                        .collect(),
                ),
            );
            "THÔNG TIN TỔNG HỢP TỪ CSDL TÀI LIỆU LƯU TRỮ"
        }
    };
    if r_jy {
        r_jn.push(
            FormBlock::Paragraph(
                format!(
                    "(Kết quả vượt giới hạn {quota} mục cho một lần yêu cầu — Điều 44.4; thu hẹp chủ đề hoặc tách yêu cầu.)"
                ),
            ),
        );
    }
    if let Some(r_jp) = note_of(&r_jo.bien_tap) {
        r_jn.push(FormBlock::SectionHeading(
            "RÀ SOÁT, BIÊN TẬP (Điều 47.2)".into(),
        ));
        r_jn.push(FormBlock::Paragraph(r_jp.to_string()));
    }
    r_jn.push(FormBlock::SectionHeading("XÁC THỰC".into()));
    r_jn.push(
        FormBlock::Paragraph(
            format!(
                "Kết quả được kết xuất từ Hệ thống quản lý tài liệu lưu trữ số và được xác thực bởi {org}. Thời điểm kết xuất: {} (+07:00).",
                Utc::now().with_timezone(& chrono::FixedOffset::east_opt(7 * 3600)
                .expect("+07:00")).format("%d/%m/%Y %H:%M"),
            ),
        ),
    );
    let r_jq = digitext_pdf::f_1cq().ok_or_else(|| {
        ApiError::service_unavailable(
            "không tìm thấy font TTF — đặt DIGITEXT_CERT_FONT",
        )
    })?;
    let font = std::fs::read(&r_jq).map_err(|e| {
        tracing::error!(
            path = % r_jq.display(), error = % e, "đọc font lỗi"
        );
        ApiError::internal()
    })?;
    let pdf = digitext_pdf::f_ve(
            r_jx,
            Some(
                "(Cung cấp thông tin theo Mục 4 Chương VI — Thông tư 05/2025/TT-BNV)",
            ),
            &r_jn,
            &font,
        )
        .map_err(|e| ApiError::bad_request(
            format!("dựng PDF kết quả lỗi: {e}"),
        ))?;
    let r_jr = f_1kc(r_ju.project_id, r_ju.id);
    r_jw.put(&r_jr, &pdf).await.map_err(|e| {
        tracing::error!(
            error = % e,
            "PUT kết quả cung cấp thông tin lỗi"
        );
        ApiError::internal()
    })?;
    info::f_2ee(&r_jv.pool, r_ju.id, &r_jr, note_of(&r_jo.bien_tap)).await?;
    let r_jt = info::get(&r_jv.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(r_jt.into()))
}
async fn fulfill(
    State(r_k2): State<AppState>,
    r_jz: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<InfoDto>, ApiError> {
    r_jz.require_perm(Permission::AccessReview)?;
    let r_k1 = load_in_scope(&r_k2, &r_jz, id).await?;
    if !info::fulfill(&r_k2.pool, r_k1.id).await? {
        return Err(
            ApiError::conflict(
                "chỉ trả kết quả khi đã duyệt và ĐÃ kết xuất (Điều 47.3)"
                    .to_string(),
            ),
        );
    }
    let r_k0 = info::get(&r_k2.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    notify_requester(
            &r_k2,
            &r_k0,
            "Kết quả cung cấp thông tin đã sẵn sàng — tải từ tài khoản (Điều 47.4)",
        )
        .await;
    Ok(Json(r_k0.into()))
}
async fn download(
    State(r_k8): State<AppState>,
    r_k3: Ts,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    r_k3.require_perm(Permission::AccessView)?;
    let r_k7 = load_in_scope(&r_k8, &r_k3, id).await?;
    require_owner_or_reviewer(&r_k3, &r_k7)?;
    let r_k5 = r_k3.has_perm(Permission::AccessReview);
    if !r_k5 {
        if r_k7.status != "fulfilled" {
            return Err(ApiError::conflict(
                "kết quả chưa được trả (Điều 47.4)".to_string(),
            ));
        }
        if !r_k7.fee_settled() {
            return Err(ApiError::conflict(
                "chưa thanh toán phí (Điều 46.3.b/46.4)".to_string(),
            ));
        }
    }
    let r_k6 = r_k7.ket_qua_key.as_deref().ok_or_else(|| {
        ApiError::conflict(
            "chưa kết xuất kết quả (Điều 47.1)".to_string(),
        )
    })?;
    let r_k9 = r_k8
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let r_k4 = r_k9
        .get(r_k6)
        .await
        .map_err(|e| {
            tracing::error!(error = % e, "đọc kết quả lỗi");
            ApiError::internal()
        })?
        .ok_or_else(ApiError::not_found)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"cung-cap-thong-tin-{id}.pdf\""),
            ),
        ],
        Body::from(r_k4),
    )
        .into_response())
}
