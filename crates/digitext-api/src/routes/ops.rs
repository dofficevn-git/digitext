// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Ts};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::Permission;
use digitext_db::ops::{self, Tdr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ops/integrity", get(f_1ik))
        .route("/ops/integrity/run", post(f_1il))
        .route("/ops/backups", get(f_1n6).post(f_27y))
}
async fn scope_projects(r_ll: &AppState, r_lj: &Ts) -> Result<Option<Vec<Uuid>>, ApiError> {
    let r_lk = f_29a(&r_ll.pool, &r_lj.user).await?;
    Ok(r_lj.list_projects(r_lk))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T7r {
    pub document_id: Uuid,
    pub filename: String,
    pub ho_so_ref: Option<String>,
    pub chi_tiet: Option<String>,
    pub checked_at: DateTime<Utc>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ta0 {
    pub tong_tai_lieu: i64,
    pub da_kiem_3_nam: i64,
    pub so_loi: i64,
    pub tep_loi: Vec<T7r>,
    /// Gói zip trong kho (bản Lưu trữ: gói AIP nhập; bản thi công: gói đã dựng) — kiểm sha256 (GĐ4, 5/11/2026).
    pub tong_goi: i64,
    pub goi_da_kiem: i64,
    pub goi_so_loi: i64,
    pub goi_loi: Vec<GoiLoiDto>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoiLoiDto {
    pub package_id: Uuid,
    pub label: String,
    pub objid: Option<String>,
    pub chi_tiet: Option<String>,
    pub checked_at: DateTime<Utc>,
}
async fn f_1ik(State(r_lp): State<AppState>, r_lm: Ts) -> Result<Json<Ta0>, ApiError> {
    r_lm.require_perm(Permission::BkView)?;
    let r_ln = scope_projects(&r_lp, &r_lm).await?;
    let (r_lq, da_kiem, so_loi) = ops::stats(&r_lp.pool, r_ln.as_deref()).await?;
    let tep_loi = ops::f_1bp(&r_lp.pool, r_ln.as_deref(), 200)
        .await?
        .into_iter()
        .map(|r_lo| T7r {
            document_id: r_lo.document_id,
            filename: r_lo.filename,
            ho_so_ref: r_lo.ho_so_ref,
            chi_tiet: r_lo.chi_tiet,
            checked_at: r_lo.checked_at,
        })
        .collect();
    let (tong_goi, goi_da_kiem, goi_so_loi) = ops::stats_goi(&r_lp.pool, r_ln.as_deref()).await?;
    let goi_loi = ops::goi_loi(&r_lp.pool, r_ln.as_deref(), 200)
        .await?
        .into_iter()
        .map(|g| GoiLoiDto { package_id: g.package_id, label: g.label, objid: g.objid, chi_tiet: g.chi_tiet, checked_at: g.checked_at })
        .collect();
    Ok(Json(Ta0 {
        tong_tai_lieu: r_lq,
        da_kiem_3_nam: da_kiem,
        so_loi,
        tep_loi,
        tong_goi,
        goi_da_kiem,
        goi_so_loi,
        goi_loi,
    }))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tjc {
    pub limit: Option<i64>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tjd {
    pub da_kiem: usize,
    pub loi: usize,
    pub goi_da_kiem: usize,
    pub goi_loi: usize,
}
async fn f_1il(
    State(r_lz): State<AppState>,
    r_lr: Ts,
    Query(r_ly): Query<Tjc>,
) -> Result<Json<Tjd>, ApiError> {
    r_lr.require_perm(Permission::BkManage)?;
    let r_m0 = r_lz
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let limit = r_ly.limit.unwrap_or(50).clamp(1, 500);
    let r_lu = ops::f_17z(&r_lz.pool, limit).await?;
    let mut loi = 0usize;
    let r_lv = r_lu.len();
    for (r_lt, r_lx, _filename, r_lw, b3) in r_lu {
        let r_m1 = check_document(r_m0.as_ref(), r_lx, r_lt, r_lw, b3.as_deref()).await;
        match r_m1 {
            Ok(()) => {
                ops::f_26a(&r_lz.pool, r_lx, r_lt, "dat", None).await?;
            }
            Err(r_ls) => {
                loi += 1;
                ops::f_26a(&r_lz.pool, r_lx, r_lt, "loi", Some(&r_ls)).await?;
            }
        }
    }
    let goi = ops::goi_can_kiem(&r_lz.pool, (limit / 10).max(1)).await?;
    let goi_da_kiem = goi.len();
    let mut goi_loi = 0usize;
    for (pkg, project, key, sha) in goi {
        match check_package(r_m0.as_ref(), &key, &sha).await {
            Ok(()) => ops::ghi_kiem_goi(&r_lz.pool, project, pkg, "dat", None).await?,
            Err(e) => {
                goi_loi += 1;
                ops::ghi_kiem_goi(&r_lz.pool, project, pkg, "loi", Some(&e)).await?;
            }
        }
    }
    Ok(Json(Tjd { da_kiem: r_lv, loi, goi_da_kiem, goi_loi }))
}
/// Zip gói trong kho còn đúng sha256 lúc nhập/dựng.
pub async fn check_package(store: &dyn digitext_storage::ObjectStore, key: &str, sha256: &str) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let bytes = match store.get(key).await {
        Ok(Some(b)) => b,
        Ok(None) => return Err(format!("thiếu zip gói trong kho ({key})")),
        Err(e) => return Err(format!("kho object lỗi: {e}")),
    };
    let thuc = format!("{:x}", Sha256::digest(&bytes));
    if !thuc.eq_ignore_ascii_case(sha256) {
        return Err(format!("{key}: sha256 sai — lưu {sha256}, băm lại {thuc}"));
    }
    Ok(())
}
pub async fn check_document(
    r_m7: &dyn digitext_storage::ObjectStore,
    r_m6: Uuid,
    r_m4: Uuid,
    r_m5: Option<i32>,
    blake3_goc: Option<&str>,
) -> Result<(), String> {
    let signed = digitext_storage::f_2fc(r_m6, r_m4);
    let warm = digitext_storage::f_2qs(r_m6, r_m4);
    let (key, r_m2) = match r_m7.get(&signed).await {
        Ok(Some(b)) => (signed, b),
        Ok(None) => match r_m7.get(&warm).await {
            Ok(Some(b)) => (warm.clone(), b),
            Ok(None) => {
                return Err(format!(
                    "thiếu object trong kho ({signed} và {warm})"
                ));
            }
            Err(e) => return Err(format!("kho object lỗi: {e}")),
        },
        Err(e) => return Err(format!("kho object lỗi: {e}")),
    };
    if let Some(goc) = blake3_goc {
        let thuc = blake3::hash(&r_m2).to_hex().to_string();
        if !thuc.eq_ignore_ascii_case(goc) {
            return Err(format!("{key}: blake3 sai — lúc nhập {goc}, băm lại {thuc}"));
        }
    }
    let r_m3 = digitext_pdf::f_2p9(&r_m2, r_m5);
    if !r_m3.ok {
        return Err(format!(
            "{key}: {}",
            r_m3.reason.unwrap_or_else(|| "không đạt".into())
        ));
    }
    Ok(())
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T12 {
    pub id: Uuid,
    pub loai: String,
    pub nguon: String,
    pub phuong_tien: Option<String>,
    pub ket_qua: String,
    pub chi_tiet: Option<String>,
    pub created_at: DateTime<Utc>,
}
async fn f_1n6(State(r_mb): State<AppState>, r_m8: Ts) -> Result<Json<Vec<T12>>, ApiError> {
    r_m8.require_perm(Permission::BkView)?;
    let r_ma = ops::backups(&r_mb.pool, 200).await?;
    Ok(Json(
        r_ma.into_iter()
            .map(|r_m9| T12 {
                id: r_m9.id,
                loai: r_m9.loai,
                nguon: r_m9.nguon,
                phuong_tien: r_m9.phuong_tien,
                ket_qua: r_m9.ket_qua,
                chi_tiet: r_m9.chi_tiet,
                created_at: r_m9.created_at,
            })
            .collect(),
    ))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tit {
    pub loai: String,
    pub nguon: String,
    pub phuong_tien: Option<String>,
    pub ket_qua: String,
    pub chi_tiet: Option<String>,
}
async fn f_27y(
    State(r_mf): State<AppState>,
    r_mc: Ts,
    Json(r_md): Json<Tit>,
) -> Result<(StatusCode, Json<T12>), ApiError> {
    r_mc.require_perm(Permission::BkManage)?;
    if !matches!(r_md.loai.as_str(), "gia_tang" | "day_du" | "day_du_3_nam") {
        return Err(
            ApiError::bad_request(
                "loai nhận 'gia_tang' | 'day_du' | 'day_du_3_nam' (Điều 30.1)",
            ),
        );
    }
    if !matches!(r_md.ket_qua.as_str(), "dat" | "khong_dat") {
        return Err(ApiError::bad_request(
            "ketQua nhận 'dat' | 'khong_dat'",
        ));
    }
    if r_md.nguon.trim().is_empty() {
        return Err(ApiError::bad_request(
            "thiếu nguồn sao lưu (Điều 30.3.a)",
        ));
    }
    let r_me = ops::f_269(
        &r_mf.pool,
        Tdr {
            loai: &r_md.loai,
            nguon: r_md.nguon.trim(),
            phuong_tien: r_md.phuong_tien.as_deref(),
            ket_qua: &r_md.ket_qua,
            chi_tiet: r_md.chi_tiet.as_deref(),
            reported_by: Some(r_mc.user.id),
        },
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(T12 {
            id: r_me.id,
            loai: r_me.loai,
            nguon: r_me.nguon,
            phuong_tien: r_me.phuong_tien,
            ket_qua: r_me.ket_qua,
            chi_tiet: r_me.chi_tiet,
            created_at: r_me.created_at,
        }),
    ))
}
