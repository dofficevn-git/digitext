// Copyright (c) 2026 DigiText. All rights reserved.
//! Nộp hồ sơ vào lưu trữ hiện hành (lưu trữ cơ quan) — Điều 19 TT 05/2025/TT-BNV (26/11/2026, bản Lưu trữ cho cơ quan).
//! Dữ liệu và máy trạng thái ở `digitext_db::nop_luu`; ở đây chỉ kiểm quyền / phạm vi, dựng biên bản, phát thông báo.
//!
//! Hai vai: người PHÒNG BAN (`luutru.submit`) nộp hồ sơ phòng mình, xem phiếu phòng mình, rút phiếu mình lập; CÁN BỘ LƯU TRỮ
//! (`luutru.receive`) thấy toàn cơ quan, chấm, tiếp nhận / trả lại, sổ nộp lưu. `luutru.unlock` mở khoá ngoại lệ (ghi audit).
//!
//! - `GET  /nop-luu/tong-quan?projectId` · `GET /nop-luu/ho-so-can-nop?projectId&phongBanId&loc=den_han|qua_han&limit&offset`
//! - `POST /nop-luu {projectId, phongBanId?, hoSoIds[], ghiChu?}` · `GET /nop-luu?projectId&trangThai&phongBanId&nam&limit&offset`
//! - `GET  /nop-luu/{id}` (phiếu + hồ sơ) · `POST /nop-luu/{id}/rut` · `POST /nop-luu/{id}/ho-so/{hoSoId}/ket-qua {ketQua, lyDo?}`
//! - `POST /nop-luu/{id}/tiep-nhan {ghiChu?}` (tự lập biên bản giao nhận) · `POST /nop-luu/{id}/tra-lai {lyDo}`
//! - `POST /nop-luu/{id}/bien-ban` (lập lại) · `POST /nop-luu/{id}/mo-khoa {hoSoId, lyDo}`
//! - `GET  /nop-luu/so-nop-luu?projectId&nam&phongBanId` · `POST /nop-luu/so-nop-luu/bien-ban {projectId, nam, phongBanId?}`
use super::chinh_ly::{pham_vi_cua, project_access};
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Datelike;
use digitext_core::Permission;
use digitext_db::{audit, nop_luu, notifications, projects};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/nop-luu", get(list).post(lap))
        .route("/nop-luu/tong-quan", get(tong_quan))
        .route("/nop-luu/ho-so-can-nop", get(ho_so_can_nop))
        .route("/nop-luu/so-nop-luu", get(so_nop_luu))
        .route("/nop-luu/so-nop-luu/bien-ban", post(so_nop_luu_bien_ban))
        .route("/nop-luu/{id}", get(get_one))
        .route("/nop-luu/{id}/rut", post(rut))
        .route("/nop-luu/{id}/ho-so/{ho_so_id}/ket-qua", post(cham))
        .route("/nop-luu/{id}/tiep-nhan", post(tiep_nhan))
        .route("/nop-luu/{id}/tra-lai", post(tra_lai))
        .route("/nop-luu/{id}/bien-ban", post(lap_bien_ban))
        .route("/nop-luu/{id}/mo-khoa", post(mo_khoa))
}

/// Phòng ban bị ép theo người gọi (None = cán bộ / quản trị: tự chọn hoặc toàn cơ quan).
fn phong_ban_ep(ts: &Ts) -> Option<Uuid> {
    pham_vi_cua(ts).map(|p| p.phong_ban_id)
}

async fn quyen_xem(st: &AppState, ts: &Ts, project: Uuid) -> Result<(), ApiError> {
    if !(ts.has_perm(Permission::LuutruSubmit) || ts.has_perm(Permission::LuutruReceive)) {
        return Err(ApiError::forbidden());
    }
    project_access(st, ts, project).await
}

async fn quyen_nhan(st: &AppState, ts: &Ts, project: Uuid) -> Result<(), ApiError> {
    ts.require_perm(Permission::LuutruReceive)?;
    project_access(st, ts, project).await
}

/// Nạp phiếu và kiểm người gọi được chạm tới nó: cán bộ lưu trữ / quản trị thấy hết; người phòng ban chỉ phiếu phòng mình.
async fn nap(st: &AppState, ts: &Ts, id: Uuid) -> Result<nop_luu::PhieuRow, ApiError> {
    let p = nop_luu::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    quyen_xem(st, ts, p.project_id).await?;
    if let Some(pb) = phong_ban_ep(ts) {
        if p.phong_ban_id != Some(pb) && p.nguoi_nop != Some(ts.user.id) {
            return Err(ApiError::forbidden());
        }
    }
    Ok(p)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DuAnQ {
    project_id: Uuid,
    #[serde(default)]
    phong_ban_id: Option<Uuid>,
}

async fn tong_quan(State(st): State<AppState>, ts: Ts, Query(q): Query<DuAnQ>) -> Result<Json<nop_luu::TongQuan>, ApiError> {
    quyen_xem(&st, &ts, q.project_id).await?;
    let pb = phong_ban_ep(&ts).or(q.phong_ban_id);
    Ok(Json(nop_luu::tong_quan(&st.pool, q.project_id, pb).await?))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CanNopQ {
    project_id: Uuid,
    #[serde(default)]
    phong_ban_id: Option<Uuid>,
    /// den_han | qua_han | (trống = mọi hồ sơ chưa nộp)
    #[serde(default)]
    loc: Option<String>,
    #[serde(default = "gioi_han")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}
fn gioi_han() -> i64 {
    200
}

async fn ho_so_can_nop(State(st): State<AppState>, ts: Ts, Query(q): Query<CanNopQ>) -> Result<Json<Value>, ApiError> {
    quyen_xem(&st, &ts, q.project_id).await?;
    let pb = phong_ban_ep(&ts).or(q.phong_ban_id);
    let loc = q.loc.as_deref().filter(|l| matches!(*l, "den_han" | "qua_han"));
    let (rows, total) = nop_luu::ho_so_can_nop(&st.pool, q.project_id, pb, loc, q.limit, q.offset).await?;
    Ok(Json(json!({ "rows": rows, "total": total })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LapBody {
    project_id: Uuid,
    #[serde(default)]
    phong_ban_id: Option<Uuid>,
    ho_so_ids: Vec<Uuid>,
    #[serde(default)]
    ghi_chu: Option<String>,
}

async fn lap(State(st): State<AppState>, ts: Ts, Json(b): Json<LapBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::LuutruSubmit)?;
    project_access(&st, &ts, b.project_id).await?;
    if b.ho_so_ids.len() > 500 {
        return Err(ApiError::bad_request("mỗi phiếu tối đa 500 hồ sơ — tách thành nhiều phiếu"));
    }
    let phong_ban_id = phong_ban_ep(&ts).or(b.phong_ban_id);
    let p = nop_luu::lap(
        &st.pool,
        nop_luu::LapPhieu {
            project_id: b.project_id,
            phong_ban_id,
            nguoi_nop: ts.user.id,
            nguoi_nop_ten: &ts.user.display_name,
            ghi_chu: b.ghi_chu.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            ho_so_ids: &b.ho_so_ids,
        },
    )
    .await?
    .map_err(ApiError::bad_request)?;
    // Báo cán bộ lưu trữ cơ quan (quản trị dự án) có phiếu mới.
    let msg = format!("Phiếu nộp lưu {} ({} hồ sơ) đang chờ tiếp nhận", p.ma_phieu, p.so_ho_so);
    for u in projects::f_re(&st.pool, p.project_id).await? {
        let _ = notifications::f_1v8_ref(&st.pool, u, "nop_luu_cho_tiep_nhan", &msg, &format!("nop_luu:{}", p.id)).await;
    }
    Ok(Json(nop_luu::get_json(&st.pool, p.id).await?.unwrap_or(Value::Null)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQ {
    project_id: Uuid,
    #[serde(default)]
    trang_thai: Option<String>,
    #[serde(default)]
    phong_ban_id: Option<Uuid>,
    #[serde(default)]
    nam: Option<i32>,
    #[serde(default = "gioi_han")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

async fn list(State(st): State<AppState>, ts: Ts, Query(q): Query<ListQ>) -> Result<Json<Value>, ApiError> {
    quyen_xem(&st, &ts, q.project_id).await?;
    let f = nop_luu::ListFilter {
        project_id: q.project_id,
        trang_thai: q.trang_thai.filter(|t| matches!(t.as_str(), "cho_tiep_nhan" | "da_tiep_nhan" | "tra_lai" | "rut")),
        phong_ban_id: phong_ban_ep(&ts).or(q.phong_ban_id),
        nguoi_nop: None,
        nam: q.nam,
        limit: q.limit,
        offset: q.offset,
    };
    let (rows, total) = nop_luu::list(&st.pool, &f).await?;
    Ok(Json(json!({ "rows": rows, "total": total })))
}

async fn get_one(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    let p = nap(&st, &ts, id).await?;
    let phieu = nop_luu::get_json(&st.pool, p.id).await?.unwrap_or(Value::Null);
    let ho_so = nop_luu::ho_so_cua(&st.pool, p.id).await?;
    Ok(Json(json!({ "phieu": phieu, "hoSo": ho_so })))
}

async fn rut(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    let p = nap(&st, &ts, id).await?;
    // Người nộp rút phiếu mình; cán bộ lưu trữ cũng rút hộ được (vd nộp nhầm).
    if p.nguoi_nop != Some(ts.user.id) && !ts.has_perm(Permission::LuutruReceive) {
        return Err(ApiError::forbidden());
    }
    let p = nop_luu::rut(&st.pool, id, ts.user.id, &ts.user.display_name).await?.map_err(ApiError::conflict)?;
    Ok(Json(nop_luu::get_json(&st.pool, p.id).await?.unwrap_or(Value::Null)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChamBody {
    /// nhan | tra_lai | cho
    ket_qua: String,
    #[serde(default)]
    ly_do: Option<String>,
}

async fn cham(State(st): State<AppState>, ts: Ts, Path((id, ho_so_id)): Path<(Uuid, Uuid)>, Json(b): Json<ChamBody>) -> Result<Json<Value>, ApiError> {
    let p = nap(&st, &ts, id).await?;
    quyen_nhan(&st, &ts, p.project_id).await?;
    nop_luu::cham(&st.pool, id, ho_so_id, &b.ket_qua, b.ly_do.as_deref()).await?.map_err(ApiError::bad_request)?;
    Ok(Json(json!({ "hoSo": nop_luu::ho_so_cua(&st.pool, id).await? })))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct GhiChuBody {
    #[serde(default)]
    ghi_chu: Option<String>,
}

/// Thân POST tuỳ chọn: không gửi gì / rỗng → mặc định (axum `Option<Json<T>>` trả 415 khi thiếu content-type).
fn than_tuy_chon<T: serde::de::DeserializeOwned + Default>(b: &axum::body::Bytes) -> Result<T, ApiError> {
    if b.iter().all(u8::is_ascii_whitespace) {
        return Ok(T::default());
    }
    serde_json::from_slice(b).map_err(|e| ApiError::bad_request(format!("thân JSON không hợp lệ: {e}")))
}

async fn tiep_nhan(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, body: axum::body::Bytes) -> Result<Json<Value>, ApiError> {
    let p = nap(&st, &ts, id).await?;
    quyen_nhan(&st, &ts, p.project_id).await?;
    let b: GhiChuBody = than_tuy_chon(&body)?;
    let ghi_chu = b.ghi_chu.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());
    let p = nop_luu::tiep_nhan(&st.pool, id, ts.user.id, &ts.user.display_name, ghi_chu.as_deref()).await?.map_err(ApiError::conflict)?;
    sau_tiep_nhan(&st, &ts, &p).await;
    Ok(Json(json!({ "phieu": nop_luu::get_json(&st.pool, p.id).await?, "hoSo": nop_luu::ho_so_cua(&st.pool, p.id).await? })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LyDoBody {
    ly_do: String,
}

async fn tra_lai(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<LyDoBody>) -> Result<Json<Value>, ApiError> {
    let p = nap(&st, &ts, id).await?;
    quyen_nhan(&st, &ts, p.project_id).await?;
    let p = nop_luu::tra_lai(&st.pool, id, ts.user.id, &ts.user.display_name, &b.ly_do).await?.map_err(ApiError::conflict)?;
    sau_tiep_nhan(&st, &ts, &p).await;
    Ok(Json(json!({ "phieu": nop_luu::get_json(&st.pool, p.id).await?, "hoSo": nop_luu::ho_so_cua(&st.pool, p.id).await? })))
}

/// Sau tiếp nhận / trả lại: lập biên bản giao nhận (lỗi dựng PDF — vd máy chủ thiếu font — không làm hỏng việc tiếp nhận,
/// còn nút «Lập lại biên bản») và báo người nộp.
async fn sau_tiep_nhan(st: &AppState, ts: &Ts, p: &nop_luu::PhieuRow) {
    if p.trang_thai == "da_tiep_nhan" {
        match dung_bien_ban(st, ts, p).await {
            Ok(_) => {}
            Err(e) => tracing::warn!(phieu = %p.ma_phieu, error = ?e, "không lập được biên bản giao nhận nộp lưu — lập lại bằng POST /nop-luu/{{id}}/bien-ban"),
        }
    }
    if let Some(u) = p.nguoi_nop {
        let msg = match p.trang_thai.as_str() {
            "da_tiep_nhan" => format!("Phiếu nộp lưu {} đã được tiếp nhận", p.ma_phieu),
            _ => format!("Phiếu nộp lưu {} bị trả lại{}", p.ma_phieu, p.ly_do_tra.as_deref().map(|l| format!(": {l}")).unwrap_or_default()),
        };
        let _ = notifications::f_1v8_ref(&st.pool, u, "nop_luu_ket_qua", &msg, &format!("nop_luu:{}:kq", p.id)).await;
    }
}

async fn dung_bien_ban(st: &AppState, ts: &Ts, p: &nop_luu::PhieuRow) -> Result<super::records::RecordDto, ApiError> {
    let phieu = nop_luu::get_json(&st.pool, p.id).await?.unwrap_or(Value::Null);
    let ho_so = nop_luu::ho_so_cua(&st.pool, p.id).await?;
    let chu = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let so = |v: &Value, k: &str| v.get(k).map(|x| match x { Value::Number(n) => n.to_string(), Value::String(s) => s.clone(), _ => String::new() }).unwrap_or_default();
    let ten_cq = projects::get(&st.pool, p.project_id).await?.map(|x| x.name).unwrap_or_default();
    let ngay = p.tiep_nhan_luc.unwrap_or(p.updated_at).with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00"));
    let rows: Vec<Vec<String>> = ho_so
        .iter()
        .map(|h| {
            vec![
                so(h, "stt"),
                if chu(h, "hoSoSo").is_empty() { chu(h, "soTam") } else { chu(h, "hoSoSo") },
                chu(h, "tieuDe"),
                format!("{} – {}", chu(h, "ngayBatDau"), chu(h, "ngayKetThuc")).trim_matches(|c| c == ' ' || c == '–').to_string(),
                chu(h, "soTo"),
                chu(h, "tenThbq"),
                match chu(h, "ketQua").as_str() {
                    "nhan" => "Nhận".into(),
                    "tra_lai" => "Trả lại".into(),
                    k => k.to_string(),
                },
                chu(h, "lyDo"),
            ]
        })
        .collect();
    let so_nhan = so(&phieu, "so_nhan");
    let so_tra = so(&phieu, "so_tra_lai");
    let payload = json!({
        "title": "Biên bản giao nhận hồ sơ nộp lưu vào lưu trữ cơ quan",
        "blocks": [
            {"t": "paragraph", "text": format!("Hôm nay, ngày {} tháng {} năm {}, tại {}, chúng tôi gồm:", ngay.day(), ngay.month(), ngay.year(), ten_cq)},
            {"t": "field", "label": "Bên giao (đơn vị nộp)", "value": format!("{} — {}", chu(&phieu, "ten_phong_ban"), chu(&phieu, "nguoi_nop_ten"))},
            {"t": "field", "label": "Bên nhận (lưu trữ cơ quan)", "value": chu(&phieu, "tiep_nhan_ten")},
            {"t": "field", "label": "Số phiếu nộp lưu", "value": p.ma_phieu},
            {"t": "field", "label": "Phông", "value": format!("{} ({})", chu(&phieu, "ten_phong"), chu(&phieu, "ma_phong"))},
            {"t": "field", "label": "Số hồ sơ nộp / nhận / trả lại", "value": format!("{} / {} / {}", p.so_ho_so, so_nhan, so_tra)},
            {"t": "field", "label": "Số văn bản / số trang", "value": format!("{} / {}", p.so_van_ban.map(|n| n.to_string()).unwrap_or_default(), p.so_trang.map(|n| n.to_string()).unwrap_or_default())},
            {"t": "table", "columns": ["STT", "Hồ sơ số", "Tiêu đề hồ sơ", "Thời gian", "Số tờ", "THBQ", "Kết quả", "Lý do"], "rows": rows},
            {"t": "paragraph", "text": format!("Ghi chú: {}", p.ghi_chu.as_deref().unwrap_or("không"))},
            {"t": "paragraph", "text": "Biên bản lập thành 02 bản, mỗi bên giữ 01 bản. Hồ sơ đã tiếp nhận được khoá trên Hệ thống; đơn vị nộp được sử dụng trong 360 ngày (Điều 19 TT 05/2025/TT-BNV)."},
            {"t": "signatures", "left": ["ĐẠI DIỆN ĐƠN VỊ NỘP", "(Ký, ghi rõ họ tên)"], "right": ["LƯU TRỮ CƠ QUAN", "(Ký, ghi rõ họ tên)"]}
        ]
    });
    let rec = super::records::f_12e_ref(st, ts, p.project_id, "giao_nhan_nop_luu", &payload, None, Some(&p.ma_phieu)).await?;
    nop_luu::set_bien_ban(&st.pool, p.id, rec.id).await?;
    Ok(rec)
}

async fn lap_bien_ban(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<super::records::RecordDto>, ApiError> {
    let p = nap(&st, &ts, id).await?;
    quyen_nhan(&st, &ts, p.project_id).await?;
    if !matches!(p.trang_thai.as_str(), "da_tiep_nhan" | "tra_lai") {
        return Err(ApiError::conflict("biên bản giao nhận lập sau khi tiếp nhận"));
    }
    Ok(Json(dung_bien_ban(&st, &ts, &p).await?))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MoKhoaBody {
    ho_so_id: Uuid,
    ly_do: String,
}

async fn mo_khoa(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<MoKhoaBody>) -> Result<Json<Value>, ApiError> {
    let p = nap(&st, &ts, id).await?;
    ts.require_perm(Permission::LuutruUnlock)?;
    if b.ly_do.trim().is_empty() {
        return Err(ApiError::bad_request("mở khoá phải ghi lý do"));
    }
    let thuoc: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM lt_nop_luu_ho_so WHERE nop_luu_id = $1 AND ho_so_id = $2)").bind(id).bind(b.ho_so_id).fetch_one(&st.pool).await?;
    if !thuoc {
        return Err(ApiError::not_found());
    }
    if !nop_luu::mo_khoa(&st.pool, b.ho_so_id, ts.user.id).await? {
        return Err(ApiError::conflict("hồ sơ không ở trạng thái đã nộp / chờ tiếp nhận"));
    }
    audit::append(
        &st.pool,
        audit::Tp {
            actor_id: Some(ts.user.id),
            device_id: ts.claims.device_id,
            action: "nop_luu.mo_khoa",
            target: Some(format!("ho_so:{}", b.ho_so_id)),
            detail: Some(json!({ "projectId": p.project_id, "maPhieu": p.ma_phieu, "lyDo": b.ly_do.trim() })),
            ip: ts.ip(),
        },
    )
    .await?;
    Ok(Json(json!({ "hoSo": nop_luu::ho_so_cua(&st.pool, id).await? })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SoQ {
    project_id: Uuid,
    #[serde(default)]
    nam: Option<i32>,
    #[serde(default)]
    phong_ban_id: Option<Uuid>,
}

async fn so_nop_luu(State(st): State<AppState>, ts: Ts, Query(q): Query<SoQ>) -> Result<Json<Value>, ApiError> {
    quyen_xem(&st, &ts, q.project_id).await?;
    let nam = q.nam.unwrap_or_else(|| nop_luu::hom_nay_vn().year());
    let pb = phong_ban_ep(&ts).or(q.phong_ban_id);
    Ok(Json(json!({ "nam": nam, "rows": nop_luu::so_nop_luu(&st.pool, q.project_id, nam, pb).await? })))
}

async fn so_nop_luu_bien_ban(State(st): State<AppState>, ts: Ts, Json(q): Json<SoQ>) -> Result<Json<super::records::RecordDto>, ApiError> {
    quyen_nhan(&st, &ts, q.project_id).await?;
    let nam = q.nam.unwrap_or_else(|| nop_luu::hom_nay_vn().year());
    let rows = nop_luu::so_nop_luu(&st.pool, q.project_id, nam, q.phong_ban_id).await?;
    let ten_cq = projects::get(&st.pool, q.project_id).await?.map(|x| x.name).unwrap_or_default();
    let chu = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let so = |v: &Value, k: &str| v.get(k).map(|x| match x { Value::Number(n) => n.to_string(), Value::String(s) => s.clone(), _ => String::new() }).unwrap_or_default();
    let ngay = |v: &Value, k: &str| {
        v.get(k)
            .and_then(Value::as_str)
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00")).format("%d/%m/%Y").to_string())
            .unwrap_or_default()
    };
    let bang: Vec<Vec<String>> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            vec![
                (i + 1).to_string(),
                if chu(r, "tiep_nhan_luc").is_empty() { ngay(r, "created_at") } else { ngay(r, "tiep_nhan_luc") },
                chu(r, "ten_phong_ban"),
                chu(r, "ma_phieu"),
                so(r, "so_ho_so"),
                so(r, "so_nhan"),
                so(r, "so_tra_lai"),
                chu(r, "nguoi_nop_ten"),
                chu(r, "tiep_nhan_ten"),
                chu(r, "so_bien_ban"),
            ]
        })
        .collect();
    let payload = json!({
        "title": format!("Sổ nộp lưu hồ sơ vào lưu trữ cơ quan năm {nam}"),
        "blocks": [
            {"t": "field", "label": "Cơ quan", "value": ten_cq},
            {"t": "field", "label": "Năm", "value": nam.to_string()},
            {"t": "field", "label": "Số phiếu đã tiếp nhận / trả lại", "value": format!("{} / {}", rows.iter().filter(|r| chu(r, "trang_thai") == "da_tiep_nhan").count(), rows.iter().filter(|r| chu(r, "trang_thai") == "tra_lai").count())},
            {"t": "table", "columns": ["STT", "Ngày", "Đơn vị nộp", "Số phiếu", "HS nộp", "Nhận", "Trả lại", "Người giao", "Người nhận", "Biên bản"], "rows": bang},
            {"t": "signatures", "left": ["NGƯỜI LẬP SỔ", "(Ký, ghi rõ họ tên)"], "right": ["LƯU TRỮ CƠ QUAN", "(Ký, ghi rõ họ tên)"]}
        ]
    });
    Ok(Json(super::records::f_12e_ref(&st, &ts, q.project_id, "so_nop_luu", &payload, None, Some(&nam.to_string())).await?))
}
