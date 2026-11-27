// Copyright (c) 2026 DigiText. All rights reserved.
//! MƯỢN – TRẢ hồ sơ giấy CÓ DUYỆT (0034, 27/11/2026 — đợt 3 đối chiếu Digifile TT05, nhóm 4A). Dữ liệu ở
//! `digitext_db::chinh_ly::muon_tra`, máy trạng thái ở `digitext_core::chinh_ly::muon_tra`; ở đây kiểm quyền, thông báo, phiếu PDF.
//! - `GET  /cl/muon-tra?projectId&trangThai&q&limit&offset` — danh sách dòng (trangThai: cho_duyet | da_duyet | dang_muon | qua_han | da_tra | tu_choi | huy | mo)
//! - `POST /cl/muon-tra {projectId, hoSoIds[], nguoiMuon, donVi?, dienThoai?, email?, noiDung?, ngayMuon?, ngayDuKienTra?, ghiChu?, giaoNgay?}`
//! - `GET  /cl/muon-tra/tong-quan?projectId` · `GET /cl/muon-tra/{id}` · `GET /cl/muon-tra/{id}/phieu.pdf`
//! - `POST /cl/muon-tra/{id}/duyet | tu-choi | giao | gia-han | tra | huy {lyDo?, ngay?, hanMoi?}`
//! - `POST /cl/muon-tra/nhom/{nhomId}/duyet | tu-choi | huy {lyDo?}` — cả phiếu (mọi dòng còn chờ duyệt)
//! Quyền: xem `loan.view`; lập / giao / gia hạn / trả / huỷ `loan.manage`; duyệt / từ chối `loan.approve`; lập và giao luôn cần `loan.approve`.
use super::project_access;
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{Datelike, NaiveDate, Utc};
use digitext_core::chinh_ly::muon_tra as mt;
use digitext_core::Permission;
use digitext_db::chinh_ly::muon_tra as db;
use digitext_db::chinh_ly::thong_ke as tk;
use digitext_db::{notifications, projects};
use digitext_pdf::manual::ManualBlock;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cl/muon-tra", get(list).post(lap))
        .route("/cl/muon-tra/tong-quan", get(tong_quan))
        .route("/cl/muon-tra/nhom/{nhom_id}", get(nhom_get))
        .route("/cl/muon-tra/nhom/{nhom_id}/duyet", post(nhom_duyet))
        .route("/cl/muon-tra/nhom/{nhom_id}/tu-choi", post(nhom_tu_choi))
        .route("/cl/muon-tra/nhom/{nhom_id}/huy", post(nhom_huy))
        .route("/cl/muon-tra/{id}", get(get_one))
        .route("/cl/muon-tra/{id}/phieu.pdf", get(phieu_pdf))
        .route("/cl/muon-tra/{id}/duyet", post(duyet))
        .route("/cl/muon-tra/{id}/tu-choi", post(tu_choi))
        .route("/cl/muon-tra/{id}/giao", post(giao))
        .route("/cl/muon-tra/{id}/gia-han", post(gia_han))
        .route("/cl/muon-tra/{id}/tra", post(tra))
        .route("/cl/muon-tra/{id}/huy", post(huy))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQ {
    project_id: Uuid,
    #[serde(default)]
    trang_thai: Option<String>,
    #[serde(default)]
    q: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}
fn default_limit() -> i64 {
    500
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DuAnQ {
    project_id: Uuid,
}

fn loi_nghiep_vu<T>(r: Result<T, String>) -> Result<T, ApiError> {
    r.map_err(ApiError::bad_request)
}

fn uuid_cua(v: &Value, k: &str) -> Option<Uuid> {
    v.get(k).and_then(Value::as_str).and_then(|s| Uuid::parse_str(s).ok())
}

async fn list(State(st): State<AppState>, ts: Ts, Query(q): Query<ListQ>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::LoanView)?;
    project_access(&st, &ts, q.project_id).await?;
    let (rows, total) = tk::phieu_muon(&st.pool, q.project_id, q.trang_thai.as_deref().filter(|s| !s.is_empty()), q.q.as_deref(), q.limit, q.offset).await?;
    Ok(Json(json!({ "rows": rows, "total": total })))
}

async fn tong_quan(State(st): State<AppState>, ts: Ts, Query(q): Query<DuAnQ>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::LoanView)?;
    project_access(&st, &ts, q.project_id).await?;
    Ok(Json(db::tong_quan(&st.pool, q.project_id).await?))
}

/// Nạp một dòng phiếu và kiểm người gọi thuộc dự án của nó.
async fn nap(st: &AppState, ts: &Ts, id: Uuid) -> Result<Value, ApiError> {
    let r = db::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    let project = uuid_cua(&r, "project_id").ok_or_else(ApiError::not_found)?;
    project_access(st, ts, project).await?;
    Ok(r)
}

async fn get_one(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::LoanView)?;
    Ok(Json(nap(&st, &ts, id).await?))
}

async fn nhom_get(State(st): State<AppState>, ts: Ts, Path(nhom_id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::LoanView)?;
    let rows = db::nhom(&st.pool, nhom_id).await?;
    let Some(first) = rows.first() else { return Err(ApiError::not_found()) };
    let project = uuid_cua(first, "project_id").ok_or_else(ApiError::not_found)?;
    project_access(&st, &ts, project).await?;
    Ok(Json(json!({ "rows": rows })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LapBody {
    project_id: Uuid,
    ho_so_ids: Vec<Uuid>,
    nguoi_muon: String,
    #[serde(default)]
    don_vi: Option<String>,
    #[serde(default)]
    dien_thoai: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    noi_dung: Option<String>,
    #[serde(default)]
    ngay_muon: Option<NaiveDate>,
    #[serde(default)]
    ngay_du_kien_tra: Option<NaiveDate>,
    #[serde(default)]
    ghi_chu: Option<String>,
    /// Lập và giao luôn (bỏ bước duyệt) — cần `loan.approve`.
    #[serde(default)]
    giao_ngay: bool,
}

async fn lap(State(st): State<AppState>, ts: Ts, Json(b): Json<LapBody>) -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    ts.require_perm(Permission::LoanManage)?;
    project_access(&st, &ts, b.project_id).await?;
    if b.ho_so_ids.len() > 200 {
        return Err(ApiError::bad_request("Một phiếu tối đa 200 hồ sơ"));
    }
    let co_approve = ts.has_perm(Permission::LoanApprove);
    if b.giao_ngay && !co_approve {
        return Err(ApiError::forbidden());
    }
    let trang_thai = if b.giao_ngay { mt::DANG_MUON } else { mt::CHO_DUYET };
    let (nhom_id, so_phieu, rows) = loi_nghiep_vu(
        db::lap(
            &st.pool,
            db::LapPhieu {
                project_id: b.project_id,
                ho_so_ids: &b.ho_so_ids,
                nguoi_muon: &b.nguoi_muon,
                don_vi: b.don_vi.as_deref(),
                dien_thoai: b.dien_thoai.as_deref(),
                email: b.email.as_deref(),
                noi_dung: b.noi_dung.as_deref(),
                ngay_muon: b.ngay_muon,
                ngay_du_kien_tra: b.ngay_du_kien_tra,
                ghi_chu: b.ghi_chu.as_deref(),
                trang_thai,
                nguoi_lap: ts.user.id,
            },
        )
        .await?,
    )?;
    digitext_db::audit::append(&st.pool, super::audit(&ts, "cl.loan.create", so_phieu.clone(), json!({ "projectId": b.project_id, "nhomId": nhom_id, "hoSo": rows.len(), "trangThai": trang_thai }))).await?;
    if trang_thai == mt::CHO_DUYET {
        // Báo quản trị dự án có phiếu chờ duyệt (một thông báo một phiếu).
        let msg = format!("Phiếu mượn {so_phieu} ({} hồ sơ) của {} chờ duyệt — người lập: {}", rows.len(), b.nguoi_muon.trim(), ts.user.display_name);
        for u in projects::f_re(&st.pool, b.project_id).await? {
            if u != ts.user.id {
                let _ = notifications::f_1v8_ref(&st.pool, u, "loan_request", &msg, &nhom_id.to_string()).await;
            }
        }
    }
    Ok((axum::http::StatusCode::CREATED, Json(json!({ "nhomId": nhom_id, "soPhieu": so_phieu, "rows": rows }))))
}

/// Thân JSON tuỳ chọn của một bước: POST không có thân (hoặc `{}`) → mặc định. Không dùng `Option<Json<_>>` vì thiếu Content-Type
/// axum trả 415 thay vì None.
fn than(b: &Bytes) -> Result<BuocBody, ApiError> {
    if b.iter().all(|c| c.is_ascii_whitespace()) {
        return Ok(BuocBody::default());
    }
    serde_json::from_slice(b).map_err(|e| ApiError::bad_request(format!("thân JSON không hợp lệ: {e}")))
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct BuocBody {
    #[serde(default)]
    ly_do: Option<String>,
    #[serde(default)]
    ngay: Option<NaiveDate>,
    #[serde(default)]
    han_moi: Option<NaiveDate>,
}

/// Một bước chuyển trên MỘT dòng: kiểm quyền theo trạng thái đích, ghi audit, báo người lập khi duyệt / từ chối.
async fn buoc(st: &AppState, ts: &Ts, id: Uuid, den: &str, b: BuocBody) -> Result<Json<Value>, ApiError> {
    ts.require_perm(mt::quyen_chuyen(den))?;
    let truoc = nap(st, ts, id).await?;
    let sau = loi_nghiep_vu(db::chuyen(&st.pool, id, den, ts.user.id, db::Buoc { ly_do: b.ly_do.as_deref(), ngay: b.ngay }).await?)?;
    let so_phieu = truoc.get("so_phieu").and_then(Value::as_str).unwrap_or("").to_string();
    digitext_db::audit::append(&st.pool, super::audit(ts, "cl.loan.transition", id.to_string(), json!({ "soPhieu": so_phieu, "tu": truoc["trang_thai"], "den": den, "lyDo": b.ly_do }))).await?;
    if matches!(den, mt::DA_DUYET | mt::TU_CHOI) {
        if let Some(nguoi_lap) = uuid_cua(&truoc, "nguoi_lap_phieu") {
            if nguoi_lap != ts.user.id {
                let tieu_de = truoc.get("tieu_de").and_then(Value::as_str).unwrap_or("");
                let msg = if den == mt::DA_DUYET {
                    format!("Phiếu mượn {so_phieu} — hồ sơ \"{tieu_de}\" đã được duyệt, có thể giao hồ sơ")
                } else {
                    format!("Phiếu mượn {so_phieu} — hồ sơ \"{tieu_de}\" bị từ chối: {}", b.ly_do.as_deref().unwrap_or(""))
                };
                let _ = notifications::f_1v8_ref(&st.pool, nguoi_lap, "loan_decided", &msg, &format!("{id}:{den}")).await;
            }
        }
    }
    Ok(Json(sau))
}

async fn duyet(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, body: Bytes) -> Result<Json<Value>, ApiError> {
    buoc(&st, &ts, id, mt::DA_DUYET, than(&body)?).await
}
async fn tu_choi(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, body: Bytes) -> Result<Json<Value>, ApiError> {
    buoc(&st, &ts, id, mt::TU_CHOI, than(&body)?).await
}
async fn giao(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, body: Bytes) -> Result<Json<Value>, ApiError> {
    buoc(&st, &ts, id, mt::DANG_MUON, than(&body)?).await
}
async fn tra(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, body: Bytes) -> Result<Json<Value>, ApiError> {
    buoc(&st, &ts, id, mt::DA_TRA, than(&body)?).await
}
async fn huy(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, body: Bytes) -> Result<Json<Value>, ApiError> {
    buoc(&st, &ts, id, mt::HUY, than(&body)?).await
}

async fn gia_han(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<BuocBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::LoanManage)?;
    let truoc = nap(&st, &ts, id).await?;
    let han_moi = b.han_moi.ok_or_else(|| ApiError::bad_request("Thiếu hạn trả mới (hanMoi)"))?;
    let sau = loi_nghiep_vu(db::gia_han(&st.pool, id, ts.user.id, &ts.user.display_name, han_moi, b.ly_do.as_deref()).await?)?;
    digitext_db::audit::append(&st.pool, super::audit(&ts, "cl.loan.extend", id.to_string(), json!({ "soPhieu": truoc["so_phieu"], "tu": truoc["ngay_du_kien_tra"], "den": han_moi, "lyDo": b.ly_do }))).await?;
    Ok(Json(sau))
}

/// Cả phiếu: nạp một dòng của nhóm để kiểm dự án, rồi chuyển mọi dòng còn chờ duyệt.
async fn nhom_buoc(st: &AppState, ts: &Ts, nhom_id: Uuid, den: &str, b: BuocBody) -> Result<Json<Value>, ApiError> {
    ts.require_perm(mt::quyen_chuyen(den))?;
    let rows = db::nhom(&st.pool, nhom_id).await?;
    let Some(first) = rows.first() else { return Err(ApiError::not_found()) };
    let project = uuid_cua(first, "project_id").ok_or_else(ApiError::not_found)?;
    project_access(st, ts, project).await?;
    let n = loi_nghiep_vu(db::chuyen_nhom(&st.pool, nhom_id, den, ts.user.id, b.ly_do.as_deref()).await?)?;
    let so_phieu = first.get("so_phieu").and_then(Value::as_str).unwrap_or("").to_string();
    digitext_db::audit::append(&st.pool, super::audit(ts, "cl.loan.transition_group", nhom_id.to_string(), json!({ "soPhieu": so_phieu, "den": den, "dong": n, "lyDo": b.ly_do }))).await?;
    if let Some(nguoi_lap) = uuid_cua(first, "nguoi_lap_phieu") {
        if nguoi_lap != ts.user.id && matches!(den, mt::DA_DUYET | mt::TU_CHOI) {
            let msg = if den == mt::DA_DUYET { format!("Phiếu mượn {so_phieu} đã được duyệt ({n} hồ sơ), có thể giao hồ sơ") } else { format!("Phiếu mượn {so_phieu} bị từ chối ({n} hồ sơ): {}", b.ly_do.as_deref().unwrap_or("")) };
            let _ = notifications::f_1v8_ref(&st.pool, nguoi_lap, "loan_decided", &msg, &format!("{nhom_id}:{den}")).await;
        }
    }
    Ok(Json(json!({ "daChuyen": n, "rows": db::nhom(&st.pool, nhom_id).await? })))
}
async fn nhom_duyet(State(st): State<AppState>, ts: Ts, Path(nhom_id): Path<Uuid>, body: Bytes) -> Result<Json<Value>, ApiError> {
    nhom_buoc(&st, &ts, nhom_id, mt::DA_DUYET, than(&body)?).await
}
async fn nhom_tu_choi(State(st): State<AppState>, ts: Ts, Path(nhom_id): Path<Uuid>, body: Bytes) -> Result<Json<Value>, ApiError> {
    nhom_buoc(&st, &ts, nhom_id, mt::TU_CHOI, than(&body)?).await
}
async fn nhom_huy(State(st): State<AppState>, ts: Ts, Path(nhom_id): Path<Uuid>, body: Bytes) -> Result<Json<Value>, ApiError> {
    nhom_buoc(&st, &ts, nhom_id, mt::HUY, than(&body)?).await
}

fn chu(v: &Value, k: &str) -> String {
    match v.get(k) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => if *b { "Có".into() } else { String::new() },
        _ => String::new(),
    }
}
fn ngay_dep(s: &str) -> String {
    NaiveDate::parse_from_str(s.get(..10).unwrap_or(s), "%Y-%m-%d").map(|d| d.format("%d/%m/%Y").to_string()).unwrap_or_else(|_| s.to_string())
}

/// Phiếu mượn PDF (máy chủ dựng, A4 dọc): đầu văn bản hành chính, thông tin người mượn, bảng hồ sơ cùng phiếu, khối ký.
async fn phieu_pdf(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Response, ApiError> {
    ts.require_perm(Permission::LoanView)?;
    let r = nap(&st, &ts, id).await?;
    let project = uuid_cua(&r, "project_id").ok_or_else(ApiError::not_found)?;
    let rows = match uuid_cua(&r, "nhom_id") {
        Some(n) => db::nhom(&st.pool, n).await?,
        None => vec![r.clone()],
    };
    let ten_cq = projects::get(&st.pool, project).await?.map(|p| p.name).unwrap_or_default();
    let font_path = digitext_pdf::f_1cq().ok_or_else(|| ApiError::service_unavailable("không tìm thấy font TTF cho phiếu mượn — đặt DIGITEXT_CERT_FONT"))?;
    let font = std::fs::read(&font_path).map_err(|e| {
        tracing::error!(path = %font_path.display(), error = %e, "đọc font phiếu mượn lỗi");
        ApiError::internal()
    })?;
    let now_vn = Utc::now().with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00"));
    let so_phieu = chu(&r, "so_phieu");
    let mut kv = vec![
        ("Người mượn".to_string(), chu(&r, "nguoi_muon")),
        ("Đơn vị công tác".to_string(), chu(&r, "don_vi")),
        ("Điện thoại / email".to_string(), [chu(&r, "dien_thoai"), chu(&r, "email")].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ")),
        ("Nội dung, mục đích".to_string(), chu(&r, "noi_dung")),
        ("Ngày mượn".to_string(), ngay_dep(&chu(&r, "ngay_muon"))),
        ("Hạn trả".to_string(), ngay_dep(&chu(&r, "ngay_du_kien_tra"))),
        ("Trạng thái".to_string(), mt::ten_trang_thai(&chu(&r, "trang_thai")).to_string()),
    ];
    if !chu(&r, "nguoi_duyet_ten").is_empty() {
        kv.push(("Người duyệt".into(), format!("{} — {}", chu(&r, "nguoi_duyet_ten"), ngay_dep(&chu(&r, "duyet_at")))));
    }
    if !chu(&r, "giao_boi_ten").is_empty() {
        kv.push(("Người giao".into(), format!("{} — {}", chu(&r, "giao_boi_ten"), ngay_dep(&chu(&r, "giao_at")))));
    }
    if chu(&r, "so_lan_gia_han").parse::<i64>().unwrap_or(0) > 0 {
        kv.push(("Gia hạn".into(), format!("{} lần, hạn gốc {}", chu(&r, "so_lan_gia_han"), ngay_dep(&chu(&r, "han_tra_goc")))));
    }
    if !chu(&r, "ghi_chu").is_empty() {
        kv.push(("Ghi chú".into(), chu(&r, "ghi_chu")));
    }
    let bang: Vec<Vec<String>> = rows
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let ma = if chu(x, "ma_ho_so").is_empty() { format!("Số tạm {}", chu(x, "so_tam")) } else { chu(x, "ma_ho_so") };
            vec![(i + 1).to_string(), ma, chu(x, "tieu_de"), chu(x, "hop_so"), chu(x, "ten_phong_ban"), mt::ten_trang_thai(&chu(x, "trang_thai")).to_string()]
        })
        .collect();
    let blocks = vec![
        ManualBlock::OfficialHeader {
            left: vec![format!("*{}", ten_cq.to_uppercase()), format!("Số: {}", if so_phieu.is_empty() { "—".to_string() } else { so_phieu.clone() })],
            right: vec!["*CỘNG HOÀ XÃ HỘI CHỦ NGHĨA VIỆT NAM".into(), "*Độc lập - Tự do - Hạnh phúc".into()],
            dated: format!("Ngày {} tháng {} năm {}", now_vn.day(), now_vn.month(), now_vn.year()),
        },
        ManualBlock::Title("PHIẾU MƯỢN HỒ SƠ".into()),
        ManualBlock::Subtitle("(Điều 32 Thông tư 05/2025/TT-BNV)".into()),
        ManualBlock::Gap(6.0),
        ManualBlock::KeyValue(kv),
        ManualBlock::Gap(10.0),
        ManualBlock::SectionBar(format!("Hồ sơ mượn ({} hồ sơ)", bang.len())),
        ManualBlock::Table { headers: vec!["STT".into(), "Mã hồ sơ".into(), "Tiêu đề hồ sơ".into(), "Hộp".into(), "Phòng ban".into(), "Trạng thái".into()], widths: vec![4.0, 14.0, 40.0, 6.0, 16.0, 12.0], rows: bang },
        ManualBlock::Gap(18.0),
        ManualBlock::Signatures {
            left: ("NGƯỜI MƯỢN".into(), "(Ký, ghi rõ họ tên)".into(), chu(&r, "nguoi_muon")),
            right: ("CÁN BỘ LƯU TRỮ".into(), "(Ký, ghi rõ họ tên)".into(), chu(&r, "nguoi_lap_ten")),
        },
        ManualBlock::Footer(format!("DigiText — in lúc {}", now_vn.format("%d/%m/%Y %H:%M"))),
    ];
    let bytes = digitext_pdf::manual::render_manual(&blocks, &font).map_err(|e| ApiError::bad_request(format!("dựng PDF phiếu mượn lỗi: {e}")))?;
    let ten = format!("phieu-muon-{}.pdf", if so_phieu.is_empty() { id.to_string() } else { so_phieu.replace('/', "-") });
    Ok(crate::routes::bao_cao::tep(bytes, "application/pdf", &ten))
}
