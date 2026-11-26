// Copyright (c) 2026 DigiText. All rights reserved.
//! Bảo quản đủ Điều 29–31 TT 05/2025/TT-BNV (0032, 26/11/2026). Dữ liệu ở `digitext_db::bao_quan`; ở đây kiểm quyền, chạy kiểm,
//! dựng nội dung điền sẵn cho biên bản Mẫu 01 / 04 (desktop lập qua sổ biên bản rồi gắn lại bằng `/bien-ban {recordId}`).
//! Màn cũ `/ops/*` giữ nguyên. Prefix `/bao-quan`:
//! - `GET tong-quan` · kế hoạch: `GET/POST ke-hoach`, `POST ke-hoach/xem-truoc`, `GET/PUT ke-hoach/{id}`, `POST …/trinh | duyet |
//!   tu-choi | huy | chay?limit | bien-ban`, `GET …/tep?trangThai`, `GET …/ket-luan` (điền sẵn Mẫu 01)
//! - lỗi: `GET loi?projectId&keHoachId&mo`, `POST loi/{id}/khac-phuc | kiem-lai | bo-qua`
//! - sao lưu: `GET lich`, `POST lich`, `PUT lich/{id}`, `GET sao-luu?lichId`, `POST sao-luu` (đủ cột 0032), `GET sao-luu/tinh-trang`,
//!   `POST sao-luu/{runId}/da-xoa`
//! - `GET/POST phuong-tien`, `PUT phuong-tien/{id}` · `GET/POST su-co`, `GET/PUT su-co/{id}`, `GET su-co/{id}/prefill`, `POST su-co/{id}/bien-ban`
//! Quyền: xem `bk.view`; ghi / chạy `bk.manage`; duyệt kế hoạch `bk.approve` (người duyệt khác người lập, trừ super admin).
use super::chinh_ly::project_access;
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Ts};
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use digitext_core::bao_quan::{self as bq, PhamViKiemTra, TinhTrangLich};
use digitext_core::{Permission, Role};
use digitext_db::{bao_quan as db, notifications, projects};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/bao-quan/tong-quan", get(tong_quan))
        .route("/bao-quan/ke-hoach", get(kh_list).post(kh_tao))
        .route("/bao-quan/ke-hoach/xem-truoc", post(kh_xem_truoc))
        .route("/bao-quan/ke-hoach/{id}", get(kh_get).put(kh_sua))
        .route("/bao-quan/ke-hoach/{id}/trinh", post(kh_trinh))
        .route("/bao-quan/ke-hoach/{id}/duyet", post(kh_duyet))
        .route("/bao-quan/ke-hoach/{id}/tu-choi", post(kh_tu_choi))
        .route("/bao-quan/ke-hoach/{id}/huy", post(kh_huy))
        .route("/bao-quan/ke-hoach/{id}/chay", post(kh_chay))
        .route("/bao-quan/ke-hoach/{id}/bien-ban", post(kh_bien_ban))
        .route("/bao-quan/ke-hoach/{id}/tep", get(kh_tep))
        .route("/bao-quan/ke-hoach/{id}/ket-luan", get(kh_ket_luan))
        .route("/bao-quan/loi", get(loi_list))
        .route("/bao-quan/loi/{id}/khac-phuc", post(loi_khac_phuc))
        .route("/bao-quan/loi/{id}/kiem-lai", post(loi_kiem_lai))
        .route("/bao-quan/loi/{id}/bo-qua", post(loi_bo_qua))
        .route("/bao-quan/lich", get(lich_list).post(lich_tao))
        .route("/bao-quan/lich/{id}", axum::routing::put(lich_sua))
        .route("/bao-quan/sao-luu", get(runs).post(run_ghi))
        .route("/bao-quan/sao-luu/tinh-trang", get(sao_luu_tinh_trang))
        .route("/bao-quan/sao-luu/{id}/da-xoa", post(run_da_xoa))
        .route("/bao-quan/phuong-tien", get(pt_list).post(pt_tao))
        .route("/bao-quan/phuong-tien/{id}", axum::routing::put(pt_sua))
        .route("/bao-quan/su-co", get(sc_list).post(sc_tao))
        .route("/bao-quan/su-co/{id}", get(sc_get).put(sc_sua))
        .route("/bao-quan/su-co/{id}/prefill", get(sc_prefill))
        .route("/bao-quan/su-co/{id}/bien-ban", post(sc_bien_ban))
}

async fn scope_projects(st: &AppState, ts: &Ts) -> Result<Option<Vec<Uuid>>, ApiError> {
    let sc = f_29a(&st.pool, &ts.user).await?;
    Ok(ts.list_projects(sc))
}

fn loi_nghiep_vu<T>(r: Result<T, String>) -> Result<T, ApiError> {
    r.map_err(ApiError::bad_request)
}

fn ngay_vn(t: DateTime<Utc>) -> String {
    t.with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00")).format("%d/%m/%Y").to_string()
}
fn gio_vn(t: DateTime<Utc>) -> String {
    t.with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00")).format("%d/%m/%Y %H:%M").to_string()
}

// --------------------------------------------------------------------------------------------------------------- tổng quan

/// Tình trạng từng lịch sao lưu (tính bằng digitext_core::bao_quan) + bản cũ cần xoá.
async fn lich_tinh_trang(st: &AppState) -> Result<Vec<Value>, ApiError> {
    let bay_gio = Utc::now();
    let mut out = Vec::new();
    for l in db::lich_list(&st.pool).await? {
        let tt = bq::tinh_trang_lich(l.lan_dat, i64::from(l.chu_ky_gio), i64::from(l.do_tre_cho_phep_gio), bay_gio);
        let (ma, han, tre) = match tt {
            TinhTrangLich::ChuaCo => ("chua_co", None, 0),
            TinhTrangLich::DungHan { han_ke_tiep } => ("dung_han", Some(han_ke_tiep), 0),
            TinhTrangLich::Tre { han_ke_tiep, gio } => ("tre", Some(han_ke_tiep), gio),
        };
        let can_xoa = bq::so_ban_can_xoa(l.so_ban, i64::from(l.giu_so_ban));
        let ban_cu = if can_xoa > 0 { db::ban_cu_can_xoa(&st.pool, l.id, l.giu_so_ban).await? } else { vec![] };
        out.push(json!({
            "id": l.id, "loai": l.loai, "ten": l.ten, "chuKyGio": l.chu_ky_gio, "doTreChoPhepGio": l.do_tre_cho_phep_gio,
            "giuSoBan": l.giu_so_ban, "bat": l.bat, "nguon": l.nguon, "phuongTienId": l.phuong_tien_id, "phuongTienMa": l.phuong_tien_ma,
            "ghiChu": l.ghi_chu, "lanDat": l.lan_dat, "soBan": l.so_ban,
            "tinhTrang": if l.bat { ma } else { "tat" }, "hanKeTiep": han, "treGio": tre, "soBanCanXoa": can_xoa, "banCuCanXoa": ban_cu,
        }));
    }
    Ok(out)
}

async fn tong_quan(State(st): State<AppState>, ts: Ts) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    let sc = scope_projects(&st, &ts).await?;
    let (kh_cho_duyet, kh_dang_chay, loi_mo, su_co_mo) = db::tong_quan(&st.pool, sc.as_deref()).await?;
    let lich = lich_tinh_trang(&st).await?;
    let lich_tre = lich.iter().filter(|l| l["tinhTrang"] == "tre" || l["tinhTrang"] == "chua_co").count();
    let ban_cu = lich.iter().map(|l| l["soBanCanXoa"].as_i64().unwrap_or(0)).sum::<i64>();
    let hom_nay = digitext_db::nop_luu::hom_nay_vn();
    let pt_sap_het = db::pt_sap_het(&st.pool, hom_nay, bq::NHAC_PHUONG_TIEN_TRUOC_NGAY).await?.len();
    Ok(Json(json!({
        "khChoDuyet": kh_cho_duyet, "khDangChay": kh_dang_chay, "loiMo": loi_mo, "suCoMo": su_co_mo,
        "lichTre": lich_tre, "banCuCanXoa": ban_cu, "phuongTienSapHet": pt_sap_het, "lich": lich,
    })))
}

// --------------------------------------------------------------------------------------------------------------- kế hoạch

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KhQ {
    project_id: Uuid,
    #[serde(default)]
    nam: Option<i32>,
}

async fn kh_list(State(st): State<AppState>, ts: Ts, Query(q): Query<KhQ>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    project_access(&st, &ts, q.project_id).await?;
    Ok(Json(json!({ "rows": db::kh_list(&st.pool, q.project_id, q.nam).await? })))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct KhBody {
    #[serde(default)]
    project_id: Option<Uuid>,
    #[serde(default)]
    nam: Option<i32>,
    #[serde(default)]
    ten: String,
    #[serde(default)]
    pham_vi: Value,
    #[serde(default)]
    phuong_phap: Option<String>,
    #[serde(default)]
    ghi_chu: Option<String>,
}

fn doc_pham_vi(v: &Value) -> Result<(PhamViKiemTra, Value), ApiError> {
    let pv: PhamViKiemTra = if v.is_null() { PhamViKiemTra::default() } else { serde_json::from_value(v.clone()).map_err(|e| ApiError::bad_request(format!("phamVi không hợp lệ: {e}")))? };
    if matches!(pv.xac_suat, Some(0)) {
        return Err(ApiError::bad_request("xacSuat phải từ 1 đến 100"));
    }
    let chuan = serde_json::to_value(&pv).unwrap_or_default();
    Ok((pv, chuan))
}

async fn kh_tao(State(st): State<AppState>, ts: Ts, Json(b): Json<KhBody>) -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    ts.require_perm(Permission::BkManage)?;
    let project = b.project_id.ok_or_else(|| ApiError::bad_request("thiếu projectId"))?;
    project_access(&st, &ts, project).await?;
    if b.ten.trim().is_empty() {
        return Err(ApiError::bad_request("Chưa đặt tên kế hoạch"));
    }
    let nam = b.nam.unwrap_or_else(|| digitext_db::nop_luu::hom_nay_vn().format("%Y").to_string().parse().unwrap_or(2026));
    let (_, pv) = doc_pham_vi(&b.pham_vi)?;
    let k = db::kh_tao(&st.pool, db::KhMoi { project_id: project, nam, ten: &b.ten, pham_vi: &pv, phuong_phap: b.phuong_phap.as_deref(), ghi_chu: b.ghi_chu.as_deref(), nguoi_lap: ts.user.id }).await?;
    let v = db::kh_get_json(&st.pool, k.id).await?.unwrap_or_default();
    Ok((axum::http::StatusCode::CREATED, Json(v)))
}

async fn kh_xem_truoc(State(st): State<AppState>, ts: Ts, Json(b): Json<KhBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    let project = b.project_id.ok_or_else(|| ApiError::bad_request("thiếu projectId"))?;
    project_access(&st, &ts, project).await?;
    let (pv, _) = doc_pham_vi(&b.pham_vi)?;
    let (tai_lieu, goi, uoc) = db::kh_xem_truoc(&st.pool, project, &pv).await?;
    let ten_phong = db::ten_phong(&st.pool, &pv.phong_ids).await?;
    Ok(Json(json!({ "taiLieu": tai_lieu, "goi": goi, "uocLuong": uoc, "moTa": pv.mo_ta(&ten_phong) })))
}

async fn nap(st: &AppState, ts: &Ts, id: Uuid) -> Result<db::KhRow, ApiError> {
    let k = db::kh_get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    project_access(st, ts, k.project_id).await?;
    Ok(k)
}

async fn kh_get(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    nap(&st, &ts, id).await?;
    Ok(Json(db::kh_get_json(&st.pool, id).await?.ok_or_else(ApiError::not_found)?))
}

async fn kh_sua(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<KhBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    nap(&st, &ts, id).await?;
    if b.ten.trim().is_empty() {
        return Err(ApiError::bad_request("Chưa đặt tên kế hoạch"));
    }
    let (_, pv) = doc_pham_vi(&b.pham_vi)?;
    if !db::kh_sua(&st.pool, id, &b.ten, &pv, b.phuong_phap.as_deref(), b.ghi_chu.as_deref()).await? {
        return Err(ApiError::bad_request("Chỉ sửa được kế hoạch còn dự thảo"));
    }
    Ok(Json(db::kh_get_json(&st.pool, id).await?.unwrap_or_default()))
}

async fn bao_quan_tri(st: &AppState, project: Uuid, kind: &str, msg: &str, ref_: &str) -> Result<(), ApiError> {
    for a in projects::f_re(&st.pool, project).await? {
        let _ = notifications::f_1v8_ref(&st.pool, a, kind, msg, ref_).await?;
    }
    Ok(())
}

async fn kh_trinh(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    let k = nap(&st, &ts, id).await?;
    let pv: PhamViKiemTra = serde_json::from_value(k.pham_vi.clone()).unwrap_or_default();
    let (tai_lieu, goi, _) = db::kh_xem_truoc(&st.pool, k.project_id, &pv).await?;
    if tai_lieu + goi == 0 {
        return Err(ApiError::bad_request("Phạm vi không có tệp nào để kiểm — nới phạm vi trước khi trình"));
    }
    loi_nghiep_vu(db::kh_trinh(&st.pool, id).await?)?;
    bao_quan_tri(&st, k.project_id, "plan_submitted", &format!("Kế hoạch kiểm tra {} ({}) chờ duyệt — Điều 29", k.ma, k.ten), &format!("kh:{id}:trinh")).await?;
    Ok(Json(db::kh_get_json(&st.pool, id).await?.unwrap_or_default()))
}

async fn kh_duyet(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkApprove)?;
    let k = nap(&st, &ts, id).await?;
    if k.nguoi_lap == Some(ts.user.id) && ts.user.role != Role::SuperAdmin {
        return Err(ApiError::bad_request("Người lập không tự duyệt kế hoạch của mình"));
    }
    let so_tep = loi_nghiep_vu(db::kh_duyet(&st.pool, id, ts.user.id).await?)?;
    if let Some(nl) = k.nguoi_lap {
        let _ = notifications::f_1v8_ref(&st.pool, nl, "plan_approved", &format!("Kế hoạch kiểm tra {} đã duyệt: {so_tep} tệp / gói sẽ kiểm tự động", k.ma), &format!("kh:{id}:duyet")).await?;
    }
    Ok(Json(db::kh_get_json(&st.pool, id).await?.unwrap_or_default()))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LyDoBody {
    #[serde(default)]
    ly_do: String,
}

async fn kh_tu_choi(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<LyDoBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkApprove)?;
    let k = nap(&st, &ts, id).await?;
    if b.ly_do.trim().is_empty() {
        return Err(ApiError::bad_request("Ghi lý do từ chối"));
    }
    loi_nghiep_vu(db::kh_tu_choi(&st.pool, id, ts.user.id, &b.ly_do).await?)?;
    if let Some(nl) = k.nguoi_lap {
        let _ = notifications::f_1v8(&st.pool, nl, None, "plan_rejected", &format!("Kế hoạch kiểm tra {} bị trả lại: {}", k.ma, b.ly_do.trim())).await?;
    }
    Ok(Json(db::kh_get_json(&st.pool, id).await?.unwrap_or_default()))
}

async fn kh_huy(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    nap(&st, &ts, id).await?;
    loi_nghiep_vu(db::kh_huy(&st.pool, id).await?)?;
    Ok(Json(db::kh_get_json(&st.pool, id).await?.unwrap_or_default()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LimitQ {
    #[serde(default)]
    limit: Option<i64>,
}

/// Kiểm ngay một lô tệp của kế hoạch (worker vẫn tự chạy mỗi giờ). Dùng chung hàm băm với `/ops/integrity/run`.
async fn kh_chay(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Query(q): Query<LimitQ>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    let k = nap(&st, &ts, id).await?;
    if !matches!(k.trang_thai.as_str(), "da_duyet" | "dang_thuc_hien") {
        return Err(ApiError::bad_request("Kế hoạch phải được duyệt rồi mới kiểm"));
    }
    let store = st.store.as_ref().ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let (n, loi, xong) = kiem_lo(&st.pool, store.as_ref(), &k, limit).await?;
    if xong {
        bao_quan_tri(&st, k.project_id, "plan_done", &format!("Kế hoạch kiểm tra {} đã kiểm hết: xem kết quả và lập biên bản Mẫu 01", k.ma), &format!("kh:{id}:xong")).await?;
    }
    Ok(Json(json!({ "daKiem": n, "loi": loi, "hoanThanh": xong })))
}

/// Kiểm `limit` tệp / gói còn chờ của kế hoạch; trả (đã kiểm, lỗi, kế hoạch vừa hoàn thành).
pub async fn kiem_lo(pool: &sqlx::PgPool, store: &dyn digitext_storage::ObjectStore, k: &db::KhRow, limit: i64) -> sqlx::Result<(usize, usize, bool)> {
    let tep = db::kh_tep_cho(pool, k.id, limit).await?;
    let (mut n, mut loi, mut xong) = (0usize, 0usize, false);
    for t in tep {
        let kq = match (t.document_id, t.package_id, t.zip_key.as_deref(), t.zip_sha256.as_deref()) {
            (Some(doc), _, _, _) => super::ops::check_document(store, k.project_id, doc, t.page_count, t.blake3.as_deref()).await,
            (None, Some(_), Some(key), Some(sha)) => super::ops::check_package(store, key, sha).await,
            _ => Err("gói không còn zip / sha256".to_string()),
        };
        n += 1;
        let x = match kq {
            Ok(()) => db::kh_ghi_ket_qua(pool, k, &t, "dat", None).await?,
            Err(e) => {
                loi += 1;
                db::kh_ghi_ket_qua(pool, k, &t, "loi", Some(&e)).await?
            }
        };
        xong |= x;
    }
    Ok((n, loi, xong))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RecordBody {
    record_id: Option<Uuid>,
}

async fn kh_bien_ban(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<RecordBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    nap(&st, &ts, id).await?;
    let rid = b.record_id.ok_or_else(|| ApiError::bad_request("thiếu recordId"))?;
    db::kh_set_record(&st.pool, id, rid).await?;
    Ok(Json(db::kh_get_json(&st.pool, id).await?.unwrap_or_default()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TepQ {
    #[serde(default)]
    trang_thai: Option<String>,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    offset: Option<i64>,
}

async fn kh_tep(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Query(q): Query<TepQ>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    nap(&st, &ts, id).await?;
    let tt = q.trang_thai.as_deref().filter(|s| !s.is_empty());
    let (rows, total) = db::kh_tep(&st.pool, id, tt, q.limit.unwrap_or(200).clamp(1, 1000), q.offset.unwrap_or(0).max(0)).await?;
    Ok(Json(json!({ "rows": rows, "total": total })))
}

/// Nội dung điền sẵn Mẫu 01 (Phụ lục VII) từ kết quả kế hoạch — dạng `{fields, lists}` của sổ biên bản desktop.
async fn kh_ket_luan(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    let k = nap(&st, &ts, id).await?;
    let kj = db::kh_get_json(&st.pool, id).await?.unwrap_or_default();
    let pv: PhamViKiemTra = serde_json::from_value(k.pham_vi.clone()).unwrap_or_default();
    let ten_phong = db::ten_phong(&st.pool, &pv.phong_ids).await?;
    let (loi_rows, _) = db::kh_tep(&st.pool, id, Some("loi"), 500, 0).await?;
    let loi_mo = db::loi_list(&st.pool, Some(&[k.project_id]), Some(id), false, 500, 0).await?;
    let khac_phuc_cua = |doc: &Value, goi: &Value| -> String {
        loi_mo
            .iter()
            .find(|l| (!doc.is_null() && l["document_id"] == *doc) || (!goi.is_null() && l["package_id"] == *goi))
            .and_then(|l| l["khac_phuc"].as_str().map(str::to_owned))
            .unwrap_or_default()
    };
    let tep_loi: Vec<Value> = loi_rows
        .iter()
        .map(|t| {
            let dia_chi = match (t["documentId"].as_str().and_then(|s| s.parse::<Uuid>().ok()), t["projectId"].as_str().and_then(|s| s.parse::<Uuid>().ok())) {
                (Some(doc), Some(p)) => digitext_storage::f_2fc(p, doc),
                _ => t["objid"].as_str().unwrap_or("").to_string(),
            };
            json!({ "tenTep": t["tenTep"], "diaChi": dia_chi, "moTaLoi": t["chiTiet"], "maHoSo": t["maHoSo"].as_str().unwrap_or(""), "khacPhuc": khac_phuc_cua(&t["documentId"], &t["packageId"]) })
        })
        .collect();
    let thoi_gian = match (kj["bat_dau_luc"].as_str(), kj["ket_thuc_luc"].as_str()) {
        (Some(a), Some(b)) => format!("{} – {}", a.get(..10).unwrap_or(a), b.get(..10).unwrap_or(b)),
        (Some(a), None) => format!("từ {}", a.get(..10).unwrap_or(a)),
        _ => String::new(),
    };
    Ok(Json(json!({
        "fields": {
            "keHoach": format!("{} — {}", k.ma, k.ten), "ky": format!("Năm {}", k.nam), "phamVi": pv.mo_ta(&ten_phong),
            "phuongPhap": kj["phuong_phap"].as_str().unwrap_or(if pv.xac_suat.is_some() { "Lấy mẫu xác suất, băm lại và kiểm PDF/A" } else { "Kiểm toàn bộ: băm lại và kiểm PDF/A" }),
            "thoiGian": thoi_gian, "tongSoTep": k.so_tep.to_string(), "soDat": k.so_dat.to_string(), "soLoi": k.so_loi.to_string(),
            "ngay": ngay_vn(Utc::now()), "nguoiLap": ts.user.display_name,
            "ketLuan": if k.so_loi == 0 { "Toàn bộ tệp kiểm tra đạt; dữ liệu chủ tiếp cận, sử dụng và cập nhật được; thông tin khớp tệp tin kèm theo." } else { "" },
        },
        "lists": { "tepLoi": tep_loi, "thanhPhan": [] },
        "keHoachId": id,
    })))
}

// --------------------------------------------------------------------------------------------------------------- lỗi

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoiQ {
    #[serde(default)]
    project_id: Option<Uuid>,
    #[serde(default)]
    ke_hoach_id: Option<Uuid>,
    #[serde(default)]
    mo: Option<bool>,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    offset: Option<i64>,
}

async fn loi_list(State(st): State<AppState>, ts: Ts, Query(q): Query<LoiQ>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    let sc = match q.project_id {
        Some(p) => {
            project_access(&st, &ts, p).await?;
            Some(vec![p])
        }
        None => scope_projects(&st, &ts).await?,
    };
    let rows = db::loi_list(&st.pool, sc.as_deref(), q.ke_hoach_id, q.mo.unwrap_or(true), q.limit.unwrap_or(200).clamp(1, 1000), q.offset.unwrap_or(0).max(0)).await?;
    Ok(Json(json!({ "rows": rows })))
}

async fn nap_loi(st: &AppState, ts: &Ts, id: Uuid) -> Result<db::LoiRow, ApiError> {
    let l = db::loi_get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    project_access(st, ts, l.project_id).await?;
    Ok(l)
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct KhacPhucBody {
    #[serde(default)]
    khac_phuc: String,
}

async fn loi_khac_phuc(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<KhacPhucBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    nap_loi(&st, &ts, id).await?;
    if b.khac_phuc.trim().is_empty() {
        return Err(ApiError::bad_request("Ghi biện pháp đã khắc phục (Điều 29.4.g)"));
    }
    if !db::loi_khac_phuc(&st.pool, id, ts.user.id, &b.khac_phuc).await? {
        return Err(ApiError::bad_request("Dòng lỗi không ở trạng thái mới / kiểm lại lỗi"));
    }
    Ok(Json(json!({ "ok": true })))
}

async fn loi_kiem_lai(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    let l = nap_loi(&st, &ts, id).await?;
    if !matches!(l.trang_thai.as_str(), "moi" | "da_khac_phuc" | "kiem_lai_loi") {
        return Err(ApiError::bad_request("Dòng lỗi đã đóng"));
    }
    let store = st.store.as_ref().ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let kq = match (l.document_id, l.zip_key.as_deref(), l.zip_sha256.as_deref()) {
        (Some(doc), _, _) => super::ops::check_document(store.as_ref(), l.project_id, doc, l.page_count, l.blake3.as_deref()).await,
        (None, Some(key), Some(sha)) => super::ops::check_package(store.as_ref(), key, sha).await,
        _ => Err("gói không còn zip / sha256".to_string()),
    };
    let (ket_qua, chi_tiet) = match &kq {
        Ok(()) => ("dat", None),
        Err(e) => ("loi", Some(e.as_str())),
    };
    db::loi_kiem_lai_ghi(&st.pool, &l, ket_qua, chi_tiet).await?;
    Ok(Json(json!({ "ketQua": ket_qua, "chiTiet": chi_tiet })))
}

async fn loi_bo_qua(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<LyDoBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    nap_loi(&st, &ts, id).await?;
    if b.ly_do.trim().is_empty() {
        return Err(ApiError::bad_request("Ghi lý do bỏ qua"));
    }
    if !db::loi_bo_qua(&st.pool, id, ts.user.id, &b.ly_do).await? {
        return Err(ApiError::bad_request("Dòng lỗi đã đóng"));
    }
    Ok(Json(json!({ "ok": true })))
}

// --------------------------------------------------------------------------------------------------------------- lịch sao lưu

async fn lich_list(State(st): State<AppState>, ts: Ts) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    Ok(Json(json!({ "rows": lich_tinh_trang(&st).await? })))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LichBody {
    #[serde(default)]
    loai: Option<String>,
    #[serde(default)]
    ten: String,
    #[serde(default)]
    chu_ky_gio: Option<i32>,
    #[serde(default)]
    do_tre_cho_phep_gio: Option<i32>,
    #[serde(default)]
    giu_so_ban: Option<i32>,
    #[serde(default = "mac_dinh_true")]
    bat: bool,
    #[serde(default)]
    nguon: Option<String>,
    #[serde(default)]
    phuong_tien_id: Option<Uuid>,
    #[serde(default)]
    ghi_chu: Option<String>,
}
fn mac_dinh_true() -> bool {
    true
}

fn lich_sua_tu(b: &LichBody) -> Result<db::LichSua<'_>, ApiError> {
    if b.ten.trim().is_empty() {
        return Err(ApiError::bad_request("Chưa đặt tên lịch"));
    }
    let chu_ky = b.chu_ky_gio.ok_or_else(|| ApiError::bad_request("thiếu chuKyGio"))?;
    if chu_ky <= 0 {
        return Err(ApiError::bad_request("chuKyGio phải > 0"));
    }
    Ok(db::LichSua {
        ten: &b.ten,
        chu_ky_gio: chu_ky,
        do_tre_cho_phep_gio: b.do_tre_cho_phep_gio.unwrap_or(2).max(0),
        giu_so_ban: b.giu_so_ban.unwrap_or(14).max(1),
        bat: b.bat,
        nguon: b.nguon.as_deref(),
        phuong_tien_id: b.phuong_tien_id,
        ghi_chu: b.ghi_chu.as_deref(),
    })
}

async fn lich_tao(State(st): State<AppState>, ts: Ts, Json(b): Json<LichBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    let loai = b.loai.as_deref().unwrap_or("");
    if !matches!(loai, "gia_tang" | "day_du" | "day_du_3_nam") {
        return Err(ApiError::bad_request("loai nhận 'gia_tang' | 'day_du' | 'day_du_3_nam' (Điều 30.1)"));
    }
    let s = lich_sua_tu(&b)?;
    let id = db::lich_tao(&st.pool, loai, s).await?;
    Ok(Json(json!({ "id": id })))
}

async fn lich_sua(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<LichBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    let s = lich_sua_tu(&b)?;
    if !db::lich_sua(&st.pool, id, s).await? {
        return Err(ApiError::not_found());
    }
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunQ {
    #[serde(default)]
    lich_id: Option<Uuid>,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    offset: Option<i64>,
}

async fn runs(State(st): State<AppState>, ts: Ts, Query(q): Query<RunQ>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    Ok(Json(json!({ "rows": db::runs(&st.pool, q.lich_id, q.limit.unwrap_or(200).clamp(1, 1000), q.offset.unwrap_or(0).max(0)).await? })))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RunBody {
    #[serde(default)]
    loai: String,
    #[serde(default)]
    nguon: String,
    #[serde(default)]
    phuong_tien: Option<String>,
    #[serde(default)]
    ket_qua: String,
    #[serde(default)]
    chi_tiet: Option<String>,
    #[serde(default)]
    lich_id: Option<Uuid>,
    #[serde(default)]
    phuong_tien_id: Option<Uuid>,
    #[serde(default)]
    duong_dan: Option<String>,
    #[serde(default)]
    dung_luong_bytes: Option<i64>,
    #[serde(default)]
    ma_bam: Option<String>,
    #[serde(default)]
    bat_dau_at: Option<DateTime<Utc>>,
    #[serde(default)]
    ket_thuc_at: Option<DateTime<Utc>>,
    #[serde(default)]
    nguoi_thuc_hien: Option<String>,
}

/// Ghi nhận một lần sao lưu đủ cột 0032 (script vận hành hoặc tay; `POST /ops/backups` cũ vẫn dùng được).
async fn run_ghi(State(st): State<AppState>, ts: Ts, Json(b): Json<RunBody>) -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    ts.require_perm(Permission::BkManage)?;
    if !matches!(b.loai.as_str(), "gia_tang" | "day_du" | "day_du_3_nam") {
        return Err(ApiError::bad_request("loai nhận 'gia_tang' | 'day_du' | 'day_du_3_nam' (Điều 30.1)"));
    }
    if !matches!(b.ket_qua.as_str(), "dat" | "khong_dat") {
        return Err(ApiError::bad_request("ketQua nhận 'dat' | 'khong_dat'"));
    }
    if b.nguon.trim().is_empty() {
        return Err(ApiError::bad_request("thiếu nguồn sao lưu (Điều 30.3.a)"));
    }
    let v = db::run_ghi(
        &st.pool,
        db::RunMoi {
            loai: &b.loai,
            nguon: b.nguon.trim(),
            phuong_tien: b.phuong_tien.as_deref(),
            ket_qua: &b.ket_qua,
            chi_tiet: b.chi_tiet.as_deref(),
            reported_by: Some(ts.user.id),
            lich_id: b.lich_id,
            phuong_tien_id: b.phuong_tien_id,
            duong_dan: b.duong_dan.as_deref(),
            dung_luong_bytes: b.dung_luong_bytes,
            ma_bam: b.ma_bam.as_deref(),
            bat_dau_at: b.bat_dau_at,
            ket_thuc_at: b.ket_thuc_at,
            nguoi_thuc_hien: b.nguoi_thuc_hien.as_deref().or(Some(ts.user.display_name.as_str())),
        },
    )
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(v)))
}

async fn sao_luu_tinh_trang(State(st): State<AppState>, ts: Ts) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    Ok(Json(json!({ "rows": lich_tinh_trang(&st).await? })))
}

async fn run_da_xoa(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    if !db::run_da_xoa(&st.pool, id, ts.user.id).await? {
        return Err(ApiError::bad_request("Bản sao lưu không có hoặc đã đánh dấu xoá"));
    }
    Ok(Json(json!({ "ok": true })))
}

// --------------------------------------------------------------------------------------------------------------- phương tiện

async fn pt_list(State(st): State<AppState>, ts: Ts) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    let hom_nay = digitext_db::nop_luu::hom_nay_vn();
    let rows: Vec<Value> = db::pt_list(&st.pool)
        .await?
        .into_iter()
        .map(|mut p| {
            let han = p["han_do_ben"].as_str().and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            let con = bq::ngay_toi_han_do_ben(han, hom_nay);
            p["con_ngay"] = json!(con);
            p["sap_het"] = json!(p["trang_thai"] == "dang_dung" && bq::can_nhac_phuong_tien(han, hom_nay));
            p
        })
        .collect();
    Ok(Json(json!({ "rows": rows })))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PtBody {
    #[serde(default)]
    ma: String,
    #[serde(default)]
    loai: String,
    #[serde(default)]
    mo_ta: Option<String>,
    #[serde(default)]
    dung_luong_bytes: Option<i64>,
    #[serde(default)]
    noi_bao_quan: Option<String>,
    #[serde(default)]
    ngay_dua_vao: Option<NaiveDate>,
    #[serde(default)]
    han_do_ben: Option<NaiveDate>,
    #[serde(default)]
    trang_thai: Option<String>,
    #[serde(default)]
    thay_boi_id: Option<Uuid>,
    #[serde(default)]
    ghi_chu: Option<String>,
}

fn pt_tu(b: &PtBody) -> Result<db::PtMoi<'_>, ApiError> {
    if b.ma.trim().is_empty() || b.loai.trim().is_empty() {
        return Err(ApiError::bad_request("Cần mã và loại phương tiện"));
    }
    let tt = b.trang_thai.as_deref().unwrap_or("dang_dung");
    if !matches!(tt, "dang_dung" | "da_thay" | "hong" | "loai_bo") {
        return Err(ApiError::bad_request("trangThai nhận dang_dung | da_thay | hong | loai_bo"));
    }
    Ok(db::PtMoi {
        ma: &b.ma,
        loai: &b.loai,
        mo_ta: b.mo_ta.as_deref(),
        dung_luong_bytes: b.dung_luong_bytes,
        noi_bao_quan: b.noi_bao_quan.as_deref(),
        ngay_dua_vao: b.ngay_dua_vao,
        han_do_ben: b.han_do_ben,
        trang_thai: tt,
        thay_boi_id: b.thay_boi_id,
        ghi_chu: b.ghi_chu.as_deref(),
    })
}

fn trung_ma(e: sqlx::Error) -> ApiError {
    match &e {
        sqlx::Error::Database(d) if d.code().as_deref() == Some("23505") => ApiError::conflict("Mã phương tiện đã có"),
        _ => e.into(),
    }
}

async fn pt_tao(State(st): State<AppState>, ts: Ts, Json(b): Json<PtBody>) -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    ts.require_perm(Permission::BkManage)?;
    let p = pt_tu(&b)?;
    let v = db::pt_tao(&st.pool, p).await.map_err(trung_ma)?;
    Ok((axum::http::StatusCode::CREATED, Json(v)))
}

async fn pt_sua(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<PtBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    let p = pt_tu(&b)?;
    Ok(Json(db::pt_sua(&st.pool, id, p).await.map_err(trung_ma)?.ok_or_else(ApiError::not_found)?))
}

// --------------------------------------------------------------------------------------------------------------- sự cố

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScQ {
    #[serde(default)]
    mo: Option<bool>,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    offset: Option<i64>,
}

async fn sc_list(State(st): State<AppState>, ts: Ts, Query(q): Query<ScQ>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    let sc = scope_projects(&st, &ts).await?;
    Ok(Json(json!({ "rows": db::sc_list(&st.pool, sc.as_deref(), q.mo.unwrap_or(false), q.limit.unwrap_or(200).clamp(1, 1000), q.offset.unwrap_or(0).max(0)).await? })))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ScBody {
    #[serde(default)]
    project_id: Option<Uuid>,
    #[serde(default)]
    thoi_diem: Option<DateTime<Utc>>,
    #[serde(default)]
    mo_ta: String,
    #[serde(default)]
    nguyen_nhan: Option<String>,
    #[serde(default)]
    muc_do: Option<String>,
    #[serde(default)]
    pham_vi: Option<String>,
    #[serde(default)]
    bien_phap: Option<String>,
    #[serde(default)]
    backup_run_id: Option<Uuid>,
    #[serde(default)]
    phuong_phap_phuc_hoi: Option<String>,
    #[serde(default)]
    phuc_hoi_luc: Option<DateTime<Utc>>,
    #[serde(default)]
    ket_qua_kiem_tra: Option<String>,
    #[serde(default)]
    du_lieu_mat: Option<String>,
    #[serde(default)]
    trang_thai: Option<String>,
    #[serde(default)]
    nguoi_xu_ly: Option<Uuid>,
}

fn sc_tu(b: &ScBody) -> Result<db::ScSua<'_>, ApiError> {
    if b.mo_ta.trim().is_empty() {
        return Err(ApiError::bad_request("Mô tả sự cố"));
    }
    let muc_do = b.muc_do.as_deref().unwrap_or("trung_binh");
    if !matches!(muc_do, "thap" | "trung_binh" | "cao" | "nghiem_trong") {
        return Err(ApiError::bad_request("mucDo nhận thap | trung_binh | cao | nghiem_trong"));
    }
    let tt = b.trang_thai.as_deref().unwrap_or("moi");
    if !matches!(tt, "moi" | "dang_xu_ly" | "da_phuc_hoi" | "dong") {
        return Err(ApiError::bad_request("trangThai nhận moi | dang_xu_ly | da_phuc_hoi | dong"));
    }
    Ok(db::ScSua {
        project_id: b.project_id,
        thoi_diem: b.thoi_diem.unwrap_or_else(Utc::now),
        mo_ta: &b.mo_ta,
        nguyen_nhan: b.nguyen_nhan.as_deref(),
        muc_do,
        pham_vi: b.pham_vi.as_deref(),
        bien_phap: b.bien_phap.as_deref(),
        backup_run_id: b.backup_run_id,
        phuong_phap_phuc_hoi: b.phuong_phap_phuc_hoi.as_deref(),
        phuc_hoi_luc: b.phuc_hoi_luc,
        ket_qua_kiem_tra: b.ket_qua_kiem_tra.as_deref(),
        du_lieu_mat: b.du_lieu_mat.as_deref(),
        trang_thai: tt,
        nguoi_xu_ly: b.nguoi_xu_ly,
    })
}

async fn sc_tao(State(st): State<AppState>, ts: Ts, Json(b): Json<ScBody>) -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    ts.require_perm(Permission::BkManage)?;
    if let Some(p) = b.project_id {
        project_access(&st, &ts, p).await?;
    }
    let s = sc_tu(&b)?;
    let v = db::sc_tao(&st.pool, s, ts.user.id).await?;
    // Sự cố nghiêm trọng báo mọi quản trị (Điều 31.1: thông báo kịp thời).
    if matches!(b.muc_do.as_deref(), Some("cao" | "nghiem_trong")) {
        let ma = v["ma"].as_str().unwrap_or("").to_string();
        for a in digitext_db::ops::f_rk(&st.pool).await? {
            let _ = notifications::f_1v8_ref(&st.pool, a, "incident", &format!("Sự cố {ma} ({}) — Điều 31: cách ly, phục hồi từ bản sao lưu, lập biên bản Mẫu 04", b.mo_ta.trim()), &format!("sc:{ma}")).await?;
        }
    }
    Ok((axum::http::StatusCode::CREATED, Json(v)))
}

async fn sc_get(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    Ok(Json(db::sc_get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?))
}

async fn sc_sua(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<ScBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    if let Some(p) = b.project_id {
        project_access(&st, &ts, p).await?;
    }
    let s = sc_tu(&b)?;
    Ok(Json(db::sc_sua(&st.pool, id, s).await?.ok_or_else(ApiError::not_found)?))
}

/// Điền sẵn Mẫu 04 từ sự cố: nguồn phục hồi lấy từ bản sao lưu đã gắn.
async fn sc_prefill(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkView)?;
    let s = db::sc_get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    let g = |k: &str| s[k].as_str().unwrap_or("").to_string();
    let thoi_diem = s["thoi_diem"].as_str().and_then(|x| x.parse::<DateTime<Utc>>().ok()).map(gio_vn).unwrap_or_default();
    let run = &s["backup_run"];
    let nguon = if run.is_null() {
        String::new()
    } else {
        let luc = run["created_at"].as_str().and_then(|x| x.parse::<DateTime<Utc>>().ok()).map(gio_vn).unwrap_or_default();
        format!("Bản sao lưu {} ({}) lúc {luc}{}", run["loai"].as_str().unwrap_or(""), run["nguon"].as_str().unwrap_or(""), run["duong_dan"].as_str().map(|d| format!(" — {d}")).unwrap_or_default())
    };
    let bien_phap: Vec<Value> = g("bien_phap").lines().map(str::trim).filter(|l| !l.is_empty()).map(|l| json!({ "value": l })).collect();
    Ok(Json(json!({
        "fields": {
            "maSuCo": g("ma"), "mucDo": match g("muc_do").as_str() { "thap" => "Thấp", "cao" => "Cao", "nghiem_trong" => "Nghiêm trọng", _ => "Trung bình" },
            "thoiDiemSuCo": thoi_diem, "moTaSuCo": g("mo_ta"), "nguyenNhan": g("nguyen_nhan"), "phamVi": g("pham_vi"),
            "nguonPhucHoi": nguon, "phuongPhapPhucHoi": g("phuong_phap_phuc_hoi"), "ketQuaKiemTra": g("ket_qua_kiem_tra"), "duLieuMat": g("du_lieu_mat"),
            "nguoiXuLy": g("nguoi_xu_ly_ten"), "ngay": ngay_vn(Utc::now()),
        },
        "lists": { "bienPhap": bien_phap },
        "suCoId": id,
    })))
}

async fn sc_bien_ban(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<RecordBody>) -> Result<Json<Value>, ApiError> {
    ts.require_perm(Permission::BkManage)?;
    let rid = b.record_id.ok_or_else(|| ApiError::bad_request("thiếu recordId"))?;
    db::sc_get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    db::sc_set_record(&st.pool, id, rid).await?;
    Ok(Json(db::sc_get(&st.pool, id).await?.unwrap_or_default()))
}
