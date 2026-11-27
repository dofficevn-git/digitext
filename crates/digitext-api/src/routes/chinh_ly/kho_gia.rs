// Copyright (c) 2026 DigiText. All rights reserved.
//! KHO / GIÁ có sức chứa + gán hộp cho hồ sơ nhập gói (0035, 27/11/2026 — đợt 3 đối chiếu Digifile TT05, nhóm 4B).
//! - `GET  /cl/kho-gia/tong-hop?projectId&khoId&phongId` — mỗi giá một dòng: sức chứa, hộp / hồ sơ đang chứa, % đầy, gần đầy
//! - `GET  /cl/kho-gia/ho-so?projectId&khoId|giaId&format=xlsx` — danh sách hồ sơ theo kho / giá (JSON hoặc Excel)
//! - `GET  /cl/kho-gia/chua-hop?projectId&phongId&limit` — hồ sơ chưa gán hộp của phông (ưu tiên hồ sơ nhập gói)
//! - `POST /cl/ho-so/gan-hop {projectId, phongBanId, rows: [{hoSoId, hopSo}]}` — đặt phòng ban cho hồ sơ chưa có rồi gán hộp
//!   (he_thong_hoa::boxing: tạo hộp nếu chưa có, ghi cl_lich_su 'boxing'); lỗi theo dòng.
//! Quyền: đọc `dossier.read`; gán hộp `dossier.systematize` hoặc `khogia.manage`.
use super::{ctx_parts, kiem_phong, op_ctx, pham_vi_phong, phong_hieu_dung, project_access, read_access};
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use crate::xlsx::{self, CotBang};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use digitext_core::Permission;
use digitext_db::chinh_ly::he_thong_hoa as hth;
use digitext_db::chinh_ly::{ho_so, kho_gia as db};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cl/kho-gia/tong-hop", get(tong_hop))
        .route("/cl/kho-gia/ho-so", get(ho_so_theo_kho))
        .route("/cl/kho-gia/chua-hop", get(chua_hop))
        .route("/cl/ho-so/gan-hop", post(gan_hop))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TongHopQ {
    project_id: Uuid,
    #[serde(default)]
    kho_id: Option<Uuid>,
    #[serde(default)]
    phong_id: Option<Uuid>,
}

async fn tong_hop(State(st): State<AppState>, ts: Ts, Query(q): Query<TongHopQ>) -> Result<Json<Value>, ApiError> {
    read_access(&st, &ts, q.project_id).await?;
    let scope = pham_vi_phong(&st, &ts, q.project_id).await?;
    let phong_id = phong_hieu_dung(scope.as_deref(), q.phong_id)?;
    let rows = db::tong_hop(&st.pool, q.project_id, q.kho_id, phong_id, scope.as_deref()).await?;
    let gan_day = rows.iter().filter(|r| r["gan_day"].as_bool() == Some(true)).count();
    let vuot = rows.iter().filter(|r| r["vuot"].as_bool() == Some(true)).count();
    Ok(Json(json!({ "rows": rows, "ganDay": gan_day, "vuot": vuot })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HoSoQ {
    project_id: Uuid,
    #[serde(default)]
    kho_id: Option<Uuid>,
    #[serde(default)]
    gia_id: Option<Uuid>,
    #[serde(default)]
    format: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}
fn default_limit() -> i64 {
    2000
}

fn chu(v: &Value, k: &str) -> String {
    match v.get(k) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

async fn ho_so_theo_kho(State(st): State<AppState>, ts: Ts, Query(q): Query<HoSoQ>) -> Result<Response, ApiError> {
    read_access(&st, &ts, q.project_id).await?;
    if q.kho_id.is_none() && q.gia_id.is_none() {
        return Err(ApiError::bad_request("Chọn kho hoặc giá"));
    }
    let xlsx_ = q.format.as_deref().map(str::trim).map(str::to_lowercase).as_deref() == Some("xlsx");
    let f = ho_so::Filter {
        project_id: q.project_id,
        pham_vi: super::pham_vi_cua(&ts),
        phong_ids: pham_vi_phong(&st, &ts, q.project_id).await?,
        kho_id: q.kho_id,
        gia_id: q.gia_id,
        sort: Some("ho_so_so".into()),
        limit: if xlsx_ { ho_so::LIST_MAX } else { q.limit.clamp(1, ho_so::LIST_MAX) },
        offset: if xlsx_ { 0 } else { q.offset.max(0) },
        ..ho_so::Filter::default()
    };
    let (rows, total) = ho_so::list(&st.pool, &f).await?;
    if !xlsx_ {
        return Ok(Json(json!({ "rows": rows, "total": total })).into_response());
    }
    let ten_pham_vi = match (q.gia_id, q.kho_id) {
        (Some(g), _) => sqlx::query_scalar::<_, String>("SELECT k.ten || ' / ' || g.ten FROM cl_gia g JOIN cl_kho k ON k.id = g.kho_id WHERE g.id = $1").bind(g).fetch_optional(&st.pool).await?.unwrap_or_default(),
        (None, Some(k)) => sqlx::query_scalar::<_, String>("SELECT ten FROM cl_kho WHERE id = $1").bind(k).fetch_optional(&st.pool).await?.unwrap_or_default(),
        _ => String::new(),
    };
    let ten_cq = digitext_db::projects::get(&st.pool, q.project_id).await?.map(|p| p.name).unwrap_or_default();
    let cot = [
        CotBang::chu("Kho", 18.0),
        CotBang::chu("Giá", 16.0),
        CotBang::ma("Hộp", 8.0),
        CotBang::ma("Hồ sơ số", 10.0),
        CotBang::ma("Mã hồ sơ", 20.0),
        CotBang::chu("Phông", 26.0),
        CotBang::chu("Phòng ban", 22.0),
        CotBang::chu("Tiêu đề hồ sơ", 50.0),
        CotBang::ma("Bắt đầu", 11.0),
        CotBang::ma("Kết thúc", 11.0),
        CotBang::chu("THBQ", 14.0),
        CotBang::so("Số tờ", 8.0),
    ];
    let bang: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            vec![
                chu(r, "ten_kho"), chu(r, "ten_gia"), chu(r, "hop_so"), chu(r, "ho_so_so"), chu(r, "ma_ho_so"), chu(r, "ten_phong"), chu(r, "ten_phong_ban"),
                chu(r, "tieu_de"), chu(r, "ngay_bat_dau"), chu(r, "ngay_ket_thuc"), chu(r, "ten_thbq"), chu(r, "so_to"),
            ]
        })
        .collect();
    let dau = vec![ten_cq, "DANH SÁCH HỒ SƠ THEO KHO / GIÁ".to_string(), format!("Phạm vi: {ten_pham_vi} — {} hồ sơ", rows.len())];
    let sheet = xlsx::sheet_bang("Hồ sơ theo kho giá", &dau, &cot, &bang, None);
    let bytes = xlsx::dong_goi(&[sheet]).map_err(|e| {
        tracing::error!(error = %e, "đóng gói Excel hồ sơ theo kho lỗi");
        ApiError::internal()
    })?;
    Ok(crate::routes::bao_cao::tep(bytes, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "ho-so-theo-kho-gia.xlsx"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChuaHopQ {
    project_id: Uuid,
    phong_id: Uuid,
    #[serde(default = "default_limit")]
    limit: i64,
}

async fn chua_hop(State(st): State<AppState>, ts: Ts, Query(q): Query<ChuaHopQ>) -> Result<Json<Value>, ApiError> {
    read_access(&st, &ts, q.project_id).await?;
    kiem_phong(pham_vi_phong(&st, &ts, q.project_id).await?.as_deref(), q.phong_id)?;
    Ok(Json(json!({ "rows": db::ho_so_chua_hop(&st.pool, q.project_id, q.phong_id, q.limit).await? })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GanHopBody {
    project_id: Uuid,
    phong_ban_id: Uuid,
    rows: Vec<GanHopRow>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GanHopRow {
    ho_so_id: Uuid,
    hop_so: String,
}

async fn gan_hop(State(st): State<AppState>, ts: Ts, Json(b): Json<GanHopBody>) -> Result<Json<Value>, ApiError> {
    if !(ts.has_perm(Permission::DossierSystematize) || ts.has_perm(Permission::KhoGiaManage)) {
        return Err(ApiError::forbidden());
    }
    project_access(&st, &ts, b.project_id).await?;
    if b.rows.is_empty() {
        return Err(ApiError::bad_request("Chưa có dòng nào"));
    }
    if b.rows.len() > 5000 {
        return Err(ApiError::bad_request("Tối đa 5000 dòng mỗi lần"));
    }
    let ids: Vec<Uuid> = b.rows.iter().map(|r| r.ho_so_id).collect();
    if let Some(sc) = pham_vi_phong(&st, &ts, b.project_id).await? {
        let ngoai: i64 = sqlx::query_scalar("SELECT count(*) FROM cl_ho_so WHERE id = ANY($1) AND NOT (phong_id = ANY($2))").bind(&ids).bind(&sc).fetch_one(&st.pool).await?;
        if ngoai > 0 {
            return Err(ApiError::forbidden());
        }
    }
    let (da_dat_pb, loi_pb) = db::dat_phong_ban(&st.pool, b.project_id, b.phong_ban_id, &ids, ts.user.id).await?;
    let loi_ids: std::collections::HashSet<Uuid> = loi_pb.iter().map(|(id, _)| *id).collect();
    let box_rows: Vec<hth::BoxRow> = b
        .rows
        .iter()
        .filter(|r| !loi_ids.contains(&r.ho_so_id))
        .map(|r| hth::BoxRow { ho_so_id: r.ho_so_id, hop_so: Some(r.hop_so.trim().to_string()), bo_so: None, tap_so: None, clear: false })
        .collect();
    let parts = ctx_parts(&ts);
    let ctx = op_ctx(&ts, b.project_id, "online", &parts);
    let kq = if box_rows.is_empty() { hth::BoxResult::default() } else { hth::boxing(&st.pool, &ctx, 1, &box_rows).await? };
    let mut errors: Vec<Value> = loi_pb.iter().map(|(id, m)| json!({ "hoSoId": id, "message": m })).collect();
    errors.extend(kq.errors.iter().map(|e| json!({ "hoSoId": e.ho_so_id, "row": e.row, "message": e.message })));
    digitext_db::audit::append(&st.pool, super::audit(&ts, "cl.gan_hop", b.project_id.to_string(), json!({ "phongBanId": b.phong_ban_id, "applied": kq.applied, "hopsCreated": kq.hops_created, "phongBanDat": da_dat_pb, "errors": errors.len() }))).await?;
    Ok(Json(json!({ "applied": kq.applied, "hopsCreated": kq.hops_created, "phongBanDat": da_dat_pb, "errors": errors })))
}
