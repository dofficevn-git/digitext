// Copyright (c) 2026 DigiText. All rights reserved.
//! Danh mục hồ sơ dự kiến của cơ quan — Điều 19.4 TT 05/2025/TT-BNV (26/11/2026). Dữ liệu / máy trạng thái ở
//! `digitext_db::danh_muc_ho_so`; ở đây kiểm quyền, ép phạm vi phòng ban, bản in, thông báo.
//!
//! Quyền: `danhmuc.edit` soạn dòng (người phòng ban chỉ dòng phòng mình), `danhmuc.submit` tạo / trình / phiên bản mới,
//! `danhmuc.approve` duyệt / trả lại / ban hành.
//!
//! - `GET /danh-muc-ho-so?projectId&phongId&nam` · `POST /danh-muc-ho-so {projectId, phongId, nam, saoChepTu?}`
//! - `GET /danh-muc-ho-so/hieu-luc?phongId&nam&phongBanId` (ô "Dòng danh mục" của biểu mẫu hồ sơ)
//! - `GET /danh-muc-ho-so/{id}` (kèm dòng) · `GET /{id}/doi-chieu` · `POST /{id}/bien-ban`
//! - `POST /{id}/dong {…}` · `PUT /{id}/dong/{dongId}` · `DELETE /{id}/dong/{dongId}` · `POST /{id}/dong/nhap {rows: […]}`
//! - `POST /{id}/trinh` · `/duyet` · `/tra-lai {lyDo}` · `/ban-hanh {soQuyetDinh, ngayBanHanh}` · `/phien-ban-moi`
use super::chinh_ly::{pham_vi_cua, project_access};
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::NaiveDate;
use digitext_core::Permission;
use digitext_db::{danh_muc_ho_so as dm, notifications, projects};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/danh-muc-ho-so", get(list).post(tao))
        .route("/danh-muc-ho-so/hieu-luc", get(hieu_luc))
        .route("/danh-muc-ho-so/{id}", get(get_one))
        .route("/danh-muc-ho-so/{id}/doi-chieu", get(doi_chieu))
        .route("/danh-muc-ho-so/{id}/bien-ban", post(bien_ban))
        .route("/danh-muc-ho-so/{id}/dong", post(them_dong))
        .route("/danh-muc-ho-so/{id}/dong/nhap", post(nhap_dong))
        .route("/danh-muc-ho-so/{id}/dong/{dong_id}", axum::routing::put(sua_dong).delete(xoa_dong))
        .route("/danh-muc-ho-so/{id}/trinh", post(trinh))
        .route("/danh-muc-ho-so/{id}/duyet", post(duyet))
        .route("/danh-muc-ho-so/{id}/tra-lai", post(tra_lai))
        .route("/danh-muc-ho-so/{id}/ban-hanh", post(ban_hanh))
        .route("/danh-muc-ho-so/{id}/phien-ban-moi", post(phien_ban_moi))
}

fn phong_ban_ep(ts: &Ts) -> Option<Uuid> {
    pham_vi_cua(ts).map(|p| p.phong_ban_id)
}

async fn quyen_xem(st: &AppState, ts: &Ts, project: Uuid) -> Result<(), ApiError> {
    if !(ts.has_perm(Permission::DanhmucEdit) || ts.has_perm(Permission::DanhmucSubmit) || ts.has_perm(Permission::DanhmucApprove) || ts.has_perm(Permission::DossierRead)) {
        return Err(ApiError::forbidden());
    }
    project_access(st, ts, project).await
}

async fn nap(st: &AppState, ts: &Ts, id: Uuid) -> Result<dm::DanhMucRow, ApiError> {
    let d = dm::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    quyen_xem(st, ts, d.project_id).await?;
    Ok(d)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQ {
    project_id: Uuid,
    #[serde(default)]
    phong_id: Option<Uuid>,
    #[serde(default)]
    nam: Option<i32>,
}

async fn list(State(st): State<AppState>, ts: Ts, Query(q): Query<ListQ>) -> Result<Json<Value>, ApiError> {
    quyen_xem(&st, &ts, q.project_id).await?;
    Ok(Json(json!({ "rows": dm::list(&st.pool, q.project_id, q.phong_id, q.nam).await? })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HieuLucQ {
    phong_id: Uuid,
    nam: i32,
    #[serde(default)]
    phong_ban_id: Option<Uuid>,
}

async fn hieu_luc(State(st): State<AppState>, ts: Ts, Query(q): Query<HieuLucQ>) -> Result<Json<Value>, ApiError> {
    let project: Option<Uuid> = sqlx::query_scalar("SELECT project_id FROM cl_phong WHERE id = $1").bind(q.phong_id).fetch_optional(&st.pool).await?;
    let project = project.ok_or_else(ApiError::not_found)?;
    quyen_xem(&st, &ts, project).await?;
    let pb = phong_ban_ep(&ts).or(q.phong_ban_id);
    Ok(Json(json!({ "rows": dm::dong_hieu_luc(&st.pool, q.phong_id, q.nam, pb).await? })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaoBody {
    project_id: Uuid,
    phong_id: Uuid,
    nam: i32,
    #[serde(default)]
    sao_chep_tu: Option<Uuid>,
}

async fn tao(State(st): State<AppState>, ts: Ts, Json(b): Json<TaoBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::DanhmucSubmit)?;
    project_access(&st, &ts, b.project_id).await?;
    if !(2000..=2100).contains(&b.nam) {
        return Err(ApiError::bad_request("năm không hợp lệ"));
    }
    let d = dm::tao(&st.pool, b.project_id, b.phong_id, b.nam, ts.user.id, b.sao_chep_tu).await?.map_err(ApiError::conflict)?;
    Ok(Json(dm::get_json(&st.pool, d.id).await?.unwrap_or(Value::Null)))
}

async fn get_one(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    let d = nap(&st, &ts, id).await?;
    // Người phòng ban vẫn xem được cả danh mục (bản chung của cơ quan), chỉ sửa dòng phòng mình.
    Ok(Json(json!({ "danhMuc": dm::get_json(&st.pool, d.id).await?, "dong": dm::dong_cua(&st.pool, d.id, None).await?, "phongBanCuaToi": phong_ban_ep(&ts) })))
}

async fn doi_chieu(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    nap(&st, &ts, id).await?;
    Ok(Json(dm::doi_chieu(&st.pool, id).await?.unwrap_or(Value::Null)))
}

async fn them_dong(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<dm::DongMoi>) -> Result<Json<Value>, ApiError> {
    let d = nap(&st, &ts, id).await?;
    ts.require_perm(Permission::DanhmucEdit)?;
    let dong_id = dm::them_dong(&st.pool, id, d.project_id, ts.user.id, phong_ban_ep(&ts), &b).await?.map_err(ApiError::bad_request)?;
    Ok(Json(json!({ "id": dong_id, "dong": dm::dong_cua(&st.pool, id, None).await? })))
}

async fn sua_dong(State(st): State<AppState>, ts: Ts, Path((id, dong_id)): Path<(Uuid, Uuid)>, Json(b): Json<dm::DongMoi>) -> Result<Json<Value>, ApiError> {
    nap(&st, &ts, id).await?;
    ts.require_perm(Permission::DanhmucEdit)?;
    dm::sua_dong(&st.pool, id, dong_id, phong_ban_ep(&ts), &b).await?.map_err(ApiError::bad_request)?;
    Ok(Json(json!({ "dong": dm::dong_cua(&st.pool, id, None).await? })))
}

async fn xoa_dong(State(st): State<AppState>, ts: Ts, Path((id, dong_id)): Path<(Uuid, Uuid)>) -> Result<Json<Value>, ApiError> {
    nap(&st, &ts, id).await?;
    ts.require_perm(Permission::DanhmucEdit)?;
    dm::xoa_dong(&st.pool, id, dong_id, phong_ban_ep(&ts)).await?.map_err(ApiError::bad_request)?;
    Ok(Json(json!({ "dong": dm::dong_cua(&st.pool, id, None).await? })))
}

#[derive(Deserialize)]
struct NhapBody {
    rows: Vec<dm::DongMoi>,
}

async fn nhap_dong(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<NhapBody>) -> Result<Json<Value>, ApiError> {
    let d = nap(&st, &ts, id).await?;
    ts.require_perm(Permission::DanhmucEdit)?;
    if b.rows.len() > 5000 {
        return Err(ApiError::bad_request("mỗi lần nhập tối đa 5.000 dòng"));
    }
    let (them, sua, loi) = dm::nhap_dong(&st.pool, id, d.project_id, ts.user.id, phong_ban_ep(&ts), &b.rows).await?.map_err(ApiError::conflict)?;
    Ok(Json(json!({
        "them": them, "sua": sua,
        "loi": loi.into_iter().map(|(i, m)| json!({ "dong": i, "loi": m })).collect::<Vec<_>>(),
        "dong": dm::dong_cua(&st.pool, id, None).await?,
    })))
}

async fn bao_quan_tri(st: &AppState, d: &dm::DanhMucRow, kind: &str, msg: &str, ref_: &str) {
    if let Ok(ids) = projects::f_re(&st.pool, d.project_id).await {
        for u in ids {
            let _ = notifications::f_1v8_ref(&st.pool, u, kind, msg, ref_).await;
        }
    }
}

async fn bao_nguoi_trinh(st: &AppState, d: &dm::DanhMucRow, msg: &str, ref_: &str) {
    for u in [d.trinh_boi, d.created_by].into_iter().flatten() {
        let _ = notifications::f_1v8_ref(&st.pool, u, "danh_muc_ket_qua", msg, ref_).await;
    }
}

fn ten(d: &dm::DanhMucRow) -> String {
    format!("Danh mục hồ sơ năm {} (phiên bản {})", d.nam, d.phien_ban)
}

async fn trinh(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    nap(&st, &ts, id).await?;
    ts.require_perm(Permission::DanhmucSubmit)?;
    let d = dm::trinh(&st.pool, id, ts.user.id).await?.map_err(ApiError::conflict)?;
    bao_quan_tri(&st, &d, "danh_muc_cho_duyet", &format!("{} đã trình, chờ duyệt", ten(&d)), &format!("danh_muc:{id}:trinh:{}", d.trinh_luc.map(|t| t.timestamp()).unwrap_or(0))).await;
    Ok(Json(dm::get_json(&st.pool, id).await?.unwrap_or(Value::Null)))
}

async fn duyet(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    let truoc = nap(&st, &ts, id).await?;
    ts.require_perm(Permission::DanhmucApprove)?;
    if truoc.trinh_boi == Some(ts.user.id) && !ts.user.role.allows(digitext_core::Perm::Super) {
        return Err(ApiError::conflict("người trình không tự duyệt danh mục của mình"));
    }
    let d = dm::duyet(&st.pool, id, ts.user.id).await?.map_err(ApiError::conflict)?;
    bao_nguoi_trinh(&st, &d, &format!("{} đã được duyệt — chờ ban hành", ten(&d)), &format!("danh_muc:{id}:duyet:{}", d.duyet_luc.map(|t| t.timestamp()).unwrap_or(0))).await;
    Ok(Json(dm::get_json(&st.pool, id).await?.unwrap_or(Value::Null)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LyDoBody {
    ly_do: String,
}

async fn tra_lai(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<LyDoBody>) -> Result<Json<Value>, ApiError> {
    nap(&st, &ts, id).await?;
    ts.require_perm(Permission::DanhmucApprove)?;
    let d = dm::tra_lai(&st.pool, id, ts.user.id, &b.ly_do).await?.map_err(ApiError::conflict)?;
    bao_nguoi_trinh(&st, &d, &format!("{} bị trả lại: {}", ten(&d), b.ly_do.trim()), &format!("danh_muc:{id}:tra:{}", d.updated_at.timestamp())).await;
    Ok(Json(dm::get_json(&st.pool, id).await?.unwrap_or(Value::Null)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BanHanhBody {
    so_quyet_dinh: String,
    ngay_ban_hanh: NaiveDate,
}

async fn ban_hanh(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<BanHanhBody>) -> Result<Json<Value>, ApiError> {
    nap(&st, &ts, id).await?;
    ts.require_perm(Permission::DanhmucApprove)?;
    let d = dm::ban_hanh(&st.pool, id, ts.user.id, &b.so_quyet_dinh, b.ngay_ban_hanh).await?.map_err(ApiError::conflict)?;
    bao_nguoi_trinh(&st, &d, &format!("{} đã ban hành theo QĐ {}", ten(&d), b.so_quyet_dinh.trim()), &format!("danh_muc:{id}:ban_hanh")).await;
    Ok(Json(dm::get_json(&st.pool, id).await?.unwrap_or(Value::Null)))
}

async fn phien_ban_moi(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    nap(&st, &ts, id).await?;
    ts.require_perm(Permission::DanhmucSubmit)?;
    let d = dm::phien_ban_moi(&st.pool, id, ts.user.id).await?.map_err(ApiError::conflict)?;
    Ok(Json(dm::get_json(&st.pool, d.id).await?.unwrap_or(Value::Null)))
}

/// Bản in danh mục (biên bản generic `danh_muc_ho_so`, ref = <mã phông>-<năm>): bảng theo phòng ban.
async fn bien_ban(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<super::records::RecordDto>, ApiError> {
    let d = nap(&st, &ts, id).await?;
    ts.require_perm(Permission::DanhmucSubmit)?;
    let dm_json = dm::get_json(&st.pool, id).await?.unwrap_or(Value::Null);
    let dong = dm::dong_cua(&st.pool, id, None).await?;
    let chu = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let ten_cq = projects::get(&st.pool, d.project_id).await?.map(|x| x.name).unwrap_or_default();
    let rows: Vec<Vec<String>> = dong
        .iter()
        .enumerate()
        .map(|(i, r)| vec![(i + 1).to_string(), chu(r, "ten_phong_ban"), chu(r, "ma_dong"), chu(r, "tieu_de"), chu(r, "ten_thbq"), chu(r, "nguoi_lap"), chu(r, "ghi_chu")])
        .collect();
    let payload = json!({
        "title": format!("Danh mục hồ sơ năm {}", d.nam),
        "blocks": [
            {"t": "field", "label": "Cơ quan", "value": ten_cq},
            {"t": "field", "label": "Phông", "value": format!("{} ({})", chu(&dm_json, "ten_phong"), chu(&dm_json, "ma_phong"))},
            {"t": "field", "label": "Phiên bản / trạng thái", "value": format!("{} / {}", d.phien_ban, d.trang_thai)},
            {"t": "field", "label": "Quyết định ban hành", "value": format!("{} ngày {}", d.so_quyet_dinh.clone().unwrap_or_else(|| "—".into()), d.ngay_ban_hanh.map(|n| n.format("%d/%m/%Y").to_string()).unwrap_or_else(|| "—".into()))},
            {"t": "table", "columns": ["STT", "Đơn vị", "Số ký hiệu HS", "Tiêu đề hồ sơ dự kiến", "THBQ", "Người lập", "Ghi chú"], "rows": rows},
            {"t": "signatures", "left": ["NGƯỜI LẬP", "(Ký, ghi rõ họ tên)"], "right": ["THỦ TRƯỞNG CƠ QUAN", "(Ký, ghi rõ họ tên, đóng dấu)"]}
        ]
    });
    let ref_ = format!("{}-{}", chu(&dm_json, "ma_phong"), d.nam);
    Ok(Json(super::records::f_12e_ref(&st, &ts, d.project_id, "danh_muc_ho_so", &payload, None, Some(&ref_)).await?))
}
