// Copyright (c) 2026 DigiText. All rights reserved.
//! Nhập gói AIP/SIP vào Hệ thống từ desktop (bản Lưu trữ GĐ3, 4/11/2026) — màn "Nhập gói vào Hệ thống". Việc nhập nằm ở
//! `crate::nhap_goi` (dùng chung với lệnh `digitext-api nhap-goi`). Gói tải lên theo khúc vào `<data>/ingest-tmp/<lô>/`
//! (đứt mạng tải tiếp từ `offset` máy chủ báo), đủ rồi mới gọi `nhap`; tệp tạm xoá sau khi nhập.
//!
//! - `POST /ingest/batches` `{projectId, ghiChu?}` → `{id}`
//! - `GET  /ingest/batches?projectId` — các lô gần đây kèm đếm kết quả
//! - `GET  /ingest/batches/{id}` — từng gói trong lô
//! - `POST /ingest/batches/{id}/chunk?ten&offset` (thân = byte) → `{offset}`; lệch offset → 409 `{offset}`
//! - `POST /ingest/batches/{id}/nhap?ten&nhanCaGoiLoi` → kết quả một gói
//! - `POST /ingest/batches/{id}/finish`
//!
//! Quyền: `submission.approve` (nhận tài liệu vào lưu trữ) + `dossier.create`, và thuộc dự án.
use super::chinh_ly::project_access;
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use crate::nhap_goi::{self, NhapCtx};
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::{Permission, Role};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

const KHUC_TOI_DA: usize = 64 * 1024 * 1024;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ingest/batches", get(ds_lo).post(tao_lo))
        .route("/ingest/batches/{id}", get(chi_tiet_lo))
        .route("/ingest/batches/{id}/chunk", post(khuc).layer(DefaultBodyLimit::max(KHUC_TOI_DA)))
        .route("/ingest/batches/{id}/nhap", post(nhap))
        .route("/ingest/batches/{id}/finish", post(ket_thuc))
}

fn kiem_quyen(ts: &Ts) -> Result<(), ApiError> {
    ts.require_perm(Permission::SubmissionApprove)?;
    ts.require_perm(Permission::DossierCreate)?;
    Ok(())
}

async fn lo_cua(st: &AppState, ts: &Ts, id: Uuid) -> Result<Uuid, ApiError> {
    kiem_quyen(ts)?;
    let project: Uuid = sqlx::query_scalar("SELECT project_id FROM ingest_batches WHERE id = $1").bind(id).fetch_optional(&st.pool).await?.ok_or_else(ApiError::not_found)?;
    project_access(st, ts, project).await?;
    Ok(project)
}

/// Tên tệp an toàn để đặt trong thư mục tạm: chữ, số, `.-_`; phải là .zip.
fn ten_an_toan(ten: &str) -> Result<String, ApiError> {
    let t: String = ten.trim().chars().map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '_' }).collect();
    if t.is_empty() || t.len() > 200 || t.starts_with('.') || t.contains("..") || !t.to_ascii_lowercase().ends_with(".zip") {
        return Err(ApiError::bad_request("tên gói phải là tệp .zip"));
    }
    Ok(t)
}

fn tep_tam(st: &AppState, lo: Uuid, ten: &str) -> std::path::PathBuf {
    st.data_dir.join("ingest-tmp").join(lo.to_string()).join(ten)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaoLo {
    project_id: Uuid,
    #[serde(default)]
    ghi_chu: Option<String>,
}

async fn tao_lo(State(st): State<AppState>, ts: Ts, Json(b): Json<TaoLo>) -> Result<Json<Value>, ApiError> {
    kiem_quyen(&ts)?;
    project_access(&st, &ts, b.project_id).await?;
    if st.store.is_none() {
        return Err(ApiError::service_unavailable("kho lưu trữ chưa cấu hình — không nhập gói được"));
    }
    let id = nhap_goi::tao_lo(&st.pool, b.project_id, ts.user.id, "tai_len", b.ghi_chu.as_deref().map(str::trim).filter(|s| !s.is_empty())).await?;
    Ok(Json(json!({ "id": id })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QDsLo {
    project_id: Uuid,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct LoRow {
    id: Uuid,
    nguon: String,
    ghi_chu: Option<String>,
    nguoi_tao: Option<String>,
    created_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    tong: i64,
    nhap: i64,
    da_co: i64,
    tu_choi: i64,
    loi: i64,
    so_van_ban: i64,
    so_chua_ky: i64,
}

async fn ds_lo(State(st): State<AppState>, ts: Ts, Query(q): Query<QDsLo>) -> Result<Json<Vec<LoRow>>, ApiError> {
    kiem_quyen(&ts)?;
    project_access(&st, &ts, q.project_id).await?;
    let rows = sqlx::query_as::<_, LoRow>(
        "SELECT b.id, b.nguon, b.ghi_chu, u.display_name AS nguoi_tao, b.created_at, b.finished_at, \
         count(i.id) AS tong, count(*) FILTER (WHERE i.ket_qua = 'nhap') AS nhap, count(*) FILTER (WHERE i.ket_qua = 'da_co') AS da_co, \
         count(*) FILTER (WHERE i.ket_qua = 'tu_choi') AS tu_choi, count(*) FILTER (WHERE i.ket_qua = 'loi') AS loi, \
         COALESCE(sum(i.so_van_ban), 0)::bigint AS so_van_ban, COALESCE(sum(i.so_chua_ky), 0)::bigint AS so_chua_ky \
         FROM ingest_batches b LEFT JOIN ingest_items i ON i.batch_id = b.id LEFT JOIN users u ON u.id = b.created_by \
         WHERE b.project_id = $1 GROUP BY b.id, u.display_name ORDER BY b.created_at DESC LIMIT 100",
    )
    .bind(q.project_id)
    .fetch_all(&st.pool)
    .await?;
    Ok(Json(rows))
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct GoiRow {
    id: Uuid,
    ten_tep: String,
    objid: Option<String>,
    loai_goi: Option<String>,
    size_bytes: Option<i64>,
    ket_qua: String,
    ket_luan_kiem: Option<String>,
    so_loi: i32,
    so_canh_bao: i32,
    package_id: Option<Uuid>,
    ho_so_id: Option<Uuid>,
    ma_ho_so: Option<String>,
    tieu_de: Option<String>,
    so_van_ban: i32,
    so_chua_ky: i32,
    thong_bao: Option<String>,
    chi_tiet: Option<Value>,
    created_at: DateTime<Utc>,
}

async fn chi_tiet_lo(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Vec<GoiRow>>, ApiError> {
    lo_cua(&st, &ts, id).await?;
    let rows = sqlx::query_as::<_, GoiRow>(
        "SELECT i.id, i.ten_tep, i.objid, i.loai_goi, i.size_bytes, i.ket_qua, i.ket_luan_kiem, i.so_loi, i.so_canh_bao, i.package_id, i.ho_so_id, \
         COALESCE(g.ma_ho_so_goc, p.ho_so_ref) AS ma_ho_so, h.tieu_de, i.so_van_ban, i.so_chua_ky, i.thong_bao, i.chi_tiet, i.created_at \
         FROM ingest_items i LEFT JOIN cl_ho_so h ON h.id = i.ho_so_id LEFT JOIN cl_ho_so_goi g ON g.ho_so_id = i.ho_so_id \
         LEFT JOIN packages p ON p.id = i.package_id WHERE i.batch_id = $1 ORDER BY i.created_at",
    )
    .bind(id)
    .fetch_all(&st.pool)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QKhuc {
    ten: String,
    offset: u64,
}

async fn khuc(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Query(q): Query<QKhuc>, body: Bytes) -> Result<Response, ApiError> {
    lo_cua(&st, &ts, id).await?;
    let ten = ten_an_toan(&q.ten)?;
    let p = tep_tam(&st, id, &ten);
    if let Some(d) = p.parent() {
        tokio::fs::create_dir_all(d).await.map_err(|e| ApiError::from(anyhow::anyhow!("tạo thư mục tạm: {e}")))?;
    }
    let co = tokio::fs::metadata(&p).await.map(|m| m.len()).unwrap_or(0);
    if q.offset != co {
        return Ok((StatusCode::CONFLICT, Json(json!({ "code": "offset", "message": "lệch vị trí tải lên", "offset": co }))).into_response());
    }
    let mut f = tokio::fs::OpenOptions::new().create(true).append(true).open(&p).await.map_err(|e| ApiError::from(anyhow::anyhow!("mở tệp tạm: {e}")))?;
    f.write_all(&body).await.map_err(|e| ApiError::from(anyhow::anyhow!("ghi tệp tạm: {e}")))?;
    f.flush().await.map_err(|e| ApiError::from(anyhow::anyhow!("ghi tệp tạm: {e}")))?;
    Ok(Json(json!({ "offset": co + body.len() as u64 })).into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QNhap {
    ten: String,
    #[serde(default)]
    nhan_ca_goi_loi: bool,
}

async fn nhap(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Query(q): Query<QNhap>) -> Result<Json<nhap_goi::KetQuaGoi>, ApiError> {
    let project = lo_cua(&st, &ts, id).await?;
    let ten = ten_an_toan(&q.ten)?;
    let p = tep_tam(&st, id, &ten);
    if !p.is_file() {
        return Err(ApiError::bad_request("chưa tải gói lên"));
    }
    let store = st.store.clone().ok_or_else(|| ApiError::service_unavailable("kho lưu trữ chưa cấu hình"))?;
    let has_perm = |perm: Permission| ts.has_perm(perm);
    let ctx = NhapCtx {
        pool: &st.pool,
        store: store.as_ref(),
        project_id: project,
        user_id: ts.user.id,
        user_name: &ts.user.display_name,
        device_id: ts.claims.device_id,
        has_perm: &has_perm,
        is_super: matches!(ts.user.role, Role::SuperAdmin),
        batch_id: id,
        cho_phep_loi: q.nhan_ca_goi_loi,
    };
    let kq = nhap_goi::nhap_goi(&ctx, &p).await;
    let _ = tokio::fs::remove_file(&p).await;
    if let Some(d) = p.parent() {
        let _ = tokio::fs::remove_dir(d).await;
    }
    Ok(Json(kq?))
}

async fn ket_thuc(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    lo_cua(&st, &ts, id).await?;
    nhap_goi::ket_thuc_lo(&st.pool, id).await?;
    let _ = tokio::fs::remove_dir_all(st.data_dir.join("ingest-tmp").join(id.to_string())).await;
    Ok(Json(json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use super::ten_an_toan;

    #[test]
    fn ten_goi_an_toan() {
        assert_eq!(ten_an_toan("urn_H49.132.02_uuid_0A6A.zip").unwrap(), "urn_H49.132.02_uuid_0A6A.zip");
        assert_eq!(ten_an_toan("gói 1.zip").unwrap(), "g_i_1.zip");
        assert!(ten_an_toan("../x.zip").is_err());
        assert!(ten_an_toan("x.pdf").is_err());
        assert!(ten_an_toan(".zip").is_err());
    }
}
