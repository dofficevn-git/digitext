// Copyright (c) 2026 DigiText. All rights reserved.
//! Tra cứu tài liệu lưu trữ (bản Lưu trữ GĐ4, 5/11/2026) — tìm không dấu trên mục lục (văn bản + hồ sơ) và lớp chữ,
//! đọc bản có watermark, nhật ký khai thác. Chỉ mục và hàm `khong_dau` / `cl_vb_tim` / `cl_hs_tim` ở migration 0026.
//!
//! - `GET /tra-cuu?projectId&q&noiDung&phongId&hoSoId&thbqId&loaiVb&namTu&namDen&cheDo&daHuy&limit&offset` — mỗi dòng
//!   một văn bản; các từ của `q` phải có mặt theo đúng thứ tự (mẫu `%tu1%tu2%`), khớp mục lục hoặc lớp chữ.
//! - `GET /tra-cuu/bo-loc?projectId` — phông, THBQ, loại văn bản, khoảng năm để dựng bộ lọc.
//! - `GET /tra-cuu/tai-lieu/{documentId}/doc` — bản đọc có watermark (không tải). Cán bộ lưu trữ (`access.review`) đọc
//!   mọi tài liệu; người khai thác chỉ đọc thẳng tài liệu CÔNG KHAI khi phí đọc của dự án bằng 0 — còn lại gửi yêu cầu
//!   khai thác (Điều 35–37). Ghi `audit_log` `khai_thac.xem`.
//! - `GET /tra-cuu/nhat-ky?projectId&limit` — nhật ký khai thác (`audit_log` `khai_thac.*`), cần `access.review`.
use super::chinh_ly::project_access;
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::Permission;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tra-cuu", get(tim))
        .route("/tra-cuu/bo-loc", get(bo_loc))
        .route("/tra-cuu/tai-lieu/{id}/doc", get(doc_tai_lieu))
        .route("/tra-cuu/nhat-ky", get(nhat_ky))
}

/// Chế độ sử dụng "hạn chế" hoặc "mật": mã TT05 02/03 hoặc chữ (có/không dấu). Dùng chung SQL ↔ Rust.
const HAN_CHE_SQL: &str = "(che IN ('02', '03') OR khong_dau(che) LIKE 'han che%' OR khong_dau(che) LIKE 'mat%')";

pub fn han_che(che_do: Option<&str>) -> bool {
    let Some(c) = che_do.map(str::trim).filter(|c| !c.is_empty()) else { return false };
    if c == "02" || c == "03" {
        return true;
    }
    let c = c.to_lowercase();
    c.starts_with("hạn chế") || c.starts_with("han che") || c.starts_with("mật") || c.starts_with("mat")
}

/// Mẫu LIKE từ chuỗi đã bỏ dấu: các từ nối bằng `%` (đúng thứ tự), thoát `\ % _`. Rỗng → None.
fn mau_tim(q_khong_dau: &str) -> Option<String> {
    let tu: Vec<String> = q_khong_dau
        .split_whitespace()
        .map(|w| w.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"))
        .collect();
    if tu.is_empty() {
        None
    } else {
        Some(format!("%{}%", tu.join("%")))
    }
}

async fn kiem_quyen(st: &AppState, ts: &Ts, project: Uuid) -> Result<(), ApiError> {
    if !(ts.has_perm(Permission::AccessRequest) || ts.has_perm(Permission::AccessView) || ts.has_perm(Permission::AccessReview)) {
        return Err(ApiError::forbidden());
    }
    project_access(st, ts, project).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimQuery {
    project_id: Uuid,
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    noi_dung: Option<bool>,
    #[serde(default)]
    phong_id: Option<Uuid>,
    #[serde(default)]
    ho_so_id: Option<Uuid>,
    #[serde(default)]
    thbq_id: Option<Uuid>,
    #[serde(default)]
    loai_vb: Option<String>,
    #[serde(default)]
    nam_tu: Option<i32>,
    #[serde(default)]
    nam_den: Option<i32>,
    /// "cong_khai" | "han_che"
    #[serde(default)]
    che_do: Option<String>,
    #[serde(default)]
    da_huy: Option<bool>,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    offset: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct DongTraCuu {
    van_ban_id: Uuid,
    ho_so_id: Uuid,
    phong_id: Uuid,
    ma_phong: String,
    ten_phong: String,
    ma_ho_so: String,
    /// Gói AIP gốc (hồ sơ nhập từ gói) — để hiển thị nguồn.
    objid: Option<String>,
    tieu_de_ho_so: String,
    thbq: Option<String>,
    so_thu_tu: i32,
    loai_vb: Option<String>,
    so_ky_hieu: Option<String>,
    ngay_thang: Option<String>,
    co_quan: Option<String>,
    trich_yeu: Option<String>,
    so_trang: Option<i32>,
    che_do: Option<String>,
    han_che: bool,
    document_id: Option<Uuid>,
    ky_so: Option<String>,
    da_huy: bool,
    /// "muc_luc" | "noi_dung" | null (không có từ tìm)
    khop: Option<String>,
    #[sqlx(skip)]
    trich_doan: Option<String>,
    #[sqlx(skip)]
    trang: Option<i32>,
    #[serde(skip)]
    tong: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct KetQuaTraCuu {
    tong: i64,
    dong: Vec<DongTraCuu>,
}

async fn tim(State(st): State<AppState>, ts: Ts, Query(q): Query<TimQuery>) -> Result<Json<KetQuaTraCuu>, ApiError> {
    kiem_quyen(&st, &ts, q.project_id).await?;
    let mau = match q.q.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => {
            let kd: String = sqlx::query_scalar("SELECT khong_dau($1)").bind(s).fetch_one(&st.pool).await?;
            mau_tim(&kd)
        }
        None => None,
    };
    let noi_dung = q.noi_dung.unwrap_or(true) && mau.is_some();
    let da_huy = q.da_huy.unwrap_or(false) && ts.has_perm(Permission::AccessReview);
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let offset = q.offset.unwrap_or(0).max(0);
    let sql = format!(
        "WITH nd AS (SELECT DISTINCT document_id FROM ocr_results WHERE $10 AND khong_dau(content) LIKE $2) \
         SELECT v.id AS van_ban_id, h.id AS ho_so_id, p.id AS phong_id, p.ma_du_an AS ma_phong, p.ten AS ten_phong, \
                COALESCE(g.ma_ho_so_goc, p.ma_du_an || '.' || COALESCE(h.ho_so_so, h.so_tam)) AS ma_ho_so, g.objid, \
                h.tieu_de AS tieu_de_ho_so, t.ten AS thbq, v.so_thu_tu, COALESCE(lv.ten, v.loai_vb) AS loai_vb, v.so_ky_hieu, v.ngay_thang, \
                v.co_quan_ban_hanh AS co_quan, v.trich_yeu, v.so_trang, x.che AS che_do, {HAN_CHE_SQL} AS han_che, \
                f.document_id, d.ky_so, COALESCE(d.status = 'disposed', false) AS da_huy, \
                CASE WHEN $2::text IS NULL THEN NULL \
                     WHEN cl_vb_tim(v.trich_yeu, v.so_ky_hieu, v.co_quan_ban_hanh, v.loai_vb, v.tu_khoa) LIKE $2 \
                       OR cl_hs_tim(h.tieu_de, h.so_ky_hieu_hs, h.tu_khoa, h.so_tam) LIKE $2 THEN 'muc_luc' \
                     ELSE 'noi_dung' END AS khop, \
                count(*) OVER () AS tong \
         FROM cl_van_ban v \
         JOIN cl_ho_so h ON h.id = v.ho_so_id AND h.deleted_at IS NULL \
         JOIN cl_phong p ON p.id = h.phong_id AND p.deleted_at IS NULL \
         LEFT JOIN cl_ho_so_goi g ON g.ho_so_id = h.id \
         LEFT JOIN cl_thbq t ON t.id = h.thbq_id \
         LEFT JOIN LATERAL (SELECT ten FROM cl_loai_van_ban WHERE ma = btrim(v.loai_vb) AND deleted_at IS NULL \
                            AND (project_id IS NULL OR project_id = v.project_id) ORDER BY project_id NULLS LAST LIMIT 1) lv ON true \
         LEFT JOIN LATERAL (SELECT document_id FROM cl_van_ban_file WHERE van_ban_id = v.id AND deleted_at IS NULL \
                            AND document_id IS NOT NULL ORDER BY so_tt NULLS LAST, created_at LIMIT 1) f ON true \
         LEFT JOIN documents d ON d.id = f.document_id \
         CROSS JOIN LATERAL (SELECT COALESCE(NULLIF(btrim(v.che_do_su_dung), ''), NULLIF(btrim(h.che_do_su_dung), '')) AS che) x \
         WHERE v.project_id = $1 AND v.deleted_at IS NULL \
           AND ($2::text IS NULL \
                OR cl_vb_tim(v.trich_yeu, v.so_ky_hieu, v.co_quan_ban_hanh, v.loai_vb, v.tu_khoa) LIKE $2 \
                OR cl_hs_tim(h.tieu_de, h.so_ky_hieu_hs, h.tu_khoa, h.so_tam) LIKE $2 \
                OR f.document_id IN (SELECT document_id FROM nd)) \
           AND ($3::uuid IS NULL OR p.id = $3) \
           AND ($4::uuid IS NULL OR h.id = $4) \
           AND ($5::uuid IS NULL OR h.thbq_id = $5) \
           AND ($6::text IS NULL OR lower(btrim(v.loai_vb)) = lower(btrim($6))) \
           AND ($7::int IS NULL OR COALESCE(extract(year FROM v.ngay_thang_sort)::int, h.nam_ket_thuc) >= $7) \
           AND ($8::int IS NULL OR COALESCE(extract(year FROM v.ngay_thang_sort)::int, h.nam_ket_thuc) <= $8) \
           AND ($9::text IS NULL OR ($9 = 'han_che') = {HAN_CHE_SQL}) \
           AND ($11 OR d.id IS NULL OR d.status <> 'disposed') \
         ORDER BY p.ma_du_an, g.ma_ho_so_goc NULLS LAST, h.so_tam, v.so_thu_tu \
         LIMIT $12 OFFSET $13"
    );
    let mut dong: Vec<DongTraCuu> = sqlx::query_as(&sql)
        .bind(q.project_id)
        .bind(mau.as_deref())
        .bind(q.phong_id)
        .bind(q.ho_so_id)
        .bind(q.thbq_id)
        .bind(q.loai_vb.as_deref().map(str::trim).filter(|s| !s.is_empty()))
        .bind(q.nam_tu)
        .bind(q.nam_den)
        .bind(q.che_do.as_deref().filter(|c| matches!(*c, "cong_khai" | "han_che")))
        .bind(noi_dung)
        .bind(da_huy)
        .bind(limit)
        .bind(offset)
        .fetch_all(&st.pool)
        .await?;
    let tong = dong.first().map(|d| d.tong).unwrap_or(0);

    // Trích đoạn lớp chữ quanh từ đầu tiên cho dòng khớp nội dung. Bỏ dấu tiếng Việt giữ nguyên số ký tự nên vị trí trong
    // chuỗi không dấu dùng được cho chuỗi gốc.
    if let (true, Some(m)) = (noi_dung, mau.as_deref()) {
        let ids: Vec<Uuid> = dong.iter().filter(|d| d.khop.as_deref() == Some("noi_dung")).filter_map(|d| d.document_id).collect();
        if !ids.is_empty() {
            let tu_dau = m.trim_matches('%').split('%').next().unwrap_or("").replace("\\%", "%").replace("\\_", "_").replace("\\\\", "\\");
            let doan: Vec<(Uuid, i32, String)> = sqlx::query_as(
                "SELECT DISTINCT ON (document_id) document_id, page_no, \
                        substring(content FROM greatest(1, strpos(khong_dau(content), $3) - 80) FOR 240) \
                 FROM ocr_results WHERE document_id = ANY($1) AND khong_dau(content) LIKE $2 \
                 ORDER BY document_id, page_no, (text_source = 'ocr_corrected') DESC",
            )
            .bind(&ids)
            .bind(m)
            .bind(&tu_dau)
            .fetch_all(&st.pool)
            .await?;
            for (id, trang, chu) in doan {
                for d in dong.iter_mut().filter(|d| d.document_id == Some(id)) {
                    d.trang = Some(trang);
                    d.trich_doan = Some(chu.split_whitespace().collect::<Vec<_>>().join(" "));
                }
            }
        }
    }
    Ok(Json(KetQuaTraCuu { tong, dong }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DuAnQuery {
    project_id: Uuid,
    #[serde(default)]
    limit: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct MucChon {
    id: Uuid,
    ten: String,
    so: i64,
}

/// Loại văn bản: `ma` là giá trị lưu trong mục lục (gói nhập ghi mã TT05, vd "32"), `ten` tra bảng cl_loai_van_ban.
#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct LoaiVb {
    ma: String,
    ten: String,
    so: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BoLoc {
    phong: Vec<MucChon>,
    thbq: Vec<MucChon>,
    loai_vb: Vec<LoaiVb>,
    nam_min: Option<i32>,
    nam_max: Option<i32>,
    tong_van_ban: i64,
    tong_ho_so: i64,
    /// Người gọi đọc thẳng được tài liệu công khai (phí đọc = 0 hoặc là cán bộ lưu trữ).
    doc_thang: bool,
    can_bo: bool,
}

async fn bo_loc(State(st): State<AppState>, ts: Ts, Query(q): Query<DuAnQuery>) -> Result<Json<BoLoc>, ApiError> {
    kiem_quyen(&st, &ts, q.project_id).await?;
    let pid = q.project_id;
    let phong: Vec<MucChon> = sqlx::query_as(
        "SELECT p.id, p.ma_du_an || ' — ' || p.ten AS ten, count(h.id) AS so FROM cl_phong p \
         LEFT JOIN cl_ho_so h ON h.phong_id = p.id AND h.deleted_at IS NULL \
         WHERE p.project_id = $1 AND p.deleted_at IS NULL GROUP BY p.id ORDER BY p.so_tt NULLS LAST, p.ma_du_an",
    )
    .bind(pid)
    .fetch_all(&st.pool)
    .await?;
    let thbq: Vec<MucChon> = sqlx::query_as(
        "SELECT t.id, t.ten, count(*) AS so FROM cl_ho_so h JOIN cl_thbq t ON t.id = h.thbq_id \
         WHERE h.project_id = $1 AND h.deleted_at IS NULL GROUP BY t.id ORDER BY t.trang_thai, t.gia_tri DESC",
    )
    .bind(pid)
    .fetch_all(&st.pool)
    .await?;
    let loai_vb: Vec<LoaiVb> = sqlx::query_as(
        "SELECT x.ma, COALESCE((SELECT l.ten FROM cl_loai_van_ban l WHERE l.ma = x.ma AND l.deleted_at IS NULL \
                                AND (l.project_id IS NULL OR l.project_id = $1) ORDER BY l.project_id NULLS LAST LIMIT 1), x.ma) AS ten, x.so \
         FROM (SELECT btrim(loai_vb) AS ma, count(*) AS so FROM cl_van_ban WHERE project_id = $1 AND deleted_at IS NULL \
               AND NULLIF(btrim(loai_vb), '') IS NOT NULL GROUP BY 1) x ORDER BY x.so DESC LIMIT 60",
    )
    .bind(pid)
    .fetch_all(&st.pool)
    .await?;
    let (nam_min, nam_max, tong_ho_so): (Option<i32>, Option<i32>, i64) =
        sqlx::query_as("SELECT min(nam_ket_thuc), max(nam_ket_thuc), count(*) FROM cl_ho_so WHERE project_id = $1 AND deleted_at IS NULL")
            .bind(pid)
            .fetch_one(&st.pool)
            .await?;
    let tong_van_ban: i64 = sqlx::query_scalar("SELECT count(*) FROM cl_van_ban WHERE project_id = $1 AND deleted_at IS NULL")
        .bind(pid)
        .fetch_one(&st.pool)
        .await?;
    let can_bo = ts.has_perm(Permission::AccessReview);
    let doc_thang = can_bo || phi_doc(&st, pid).await? == 0;
    Ok(Json(BoLoc { phong, thbq, loai_vb, nam_min, nam_max, tong_van_ban, tong_ho_so, doc_thang, can_bo }))
}

async fn phi_doc(st: &AppState, project: Uuid) -> Result<i64, ApiError> {
    let phi: Option<i64> = sqlx::query_scalar("SELECT phi_doc_tai_lieu FROM access_fee_rates WHERE project_id = $1")
        .bind(project)
        .fetch_optional(&st.pool)
        .await?;
    Ok(phi.unwrap_or(0))
}

/// Ghi một dòng nhật ký khai thác (`audit_log` action `khai_thac.<viec>`); lỗi ghi chỉ cảnh báo, không chặn việc chính.
pub async fn ghi_nhat_ky(st: &AppState, ts: &Ts, viec: &str, project: Uuid, document_id: Option<Uuid>, chi_tiet: serde_json::Value) {
    let mut detail = json!({ "projectId": project });
    if let (Some(o), serde_json::Value::Object(c)) = (detail.as_object_mut(), chi_tiet) {
        o.extend(c);
    }
    if let Err(e) = digitext_db::audit::append(
        &st.pool,
        digitext_db::audit::Tp {
            actor_id: Some(ts.user.id),
            device_id: ts.claims.device_id,
            action: &format!("khai_thac.{viec}"),
            target: document_id.map(|d| format!("document:{d}")),
            detail: Some(detail),
            ip: ts.ip(),
        },
    )
    .await
    {
        tracing::warn!(error = %e, "ghi nhật ký khai thác lỗi");
    }
}

async fn doc_tai_lieu(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Response, ApiError> {
    let row: Option<(Uuid, String, String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT d.project_id, d.status, d.filename, \
                COALESCE(NULLIF(btrim(v.che_do_su_dung), ''), NULLIF(btrim(h.che_do_su_dung), '')), v.trich_yeu, \
                COALESCE(g.ma_ho_so_goc, h.so_tam) \
         FROM documents d \
         LEFT JOIN LATERAL (SELECT van_ban_id FROM cl_van_ban_file WHERE document_id = d.id AND deleted_at IS NULL LIMIT 1) f ON true \
         LEFT JOIN cl_van_ban v ON v.id = f.van_ban_id \
         LEFT JOIN cl_ho_so h ON h.id = v.ho_so_id \
         LEFT JOIN cl_ho_so_goi g ON g.ho_so_id = h.id \
         WHERE d.id = $1",
    )
    .bind(id)
    .fetch_optional(&st.pool)
    .await?;
    let (project, status, filename, che_do, trich_yeu, ma_ho_so) = row.ok_or_else(ApiError::not_found)?;
    kiem_quyen(&st, &ts, project).await?;
    if status == "disposed" {
        return Err(ApiError::conflict("tài liệu đã tiêu huỷ theo quyết định — chỉ còn mục lục"));
    }
    if !ts.has_perm(Permission::AccessReview) {
        if han_che(che_do.as_deref()) {
            return Err(ApiError::conflict("tài liệu hạn chế sử dụng — chỉ xem mục lục; muốn đọc bản số hãy gửi yêu cầu khai thác (Điều 35)"));
        }
        if phi_doc(&st, project).await? > 0 {
            return Err(ApiError::conflict("đọc tài liệu có thu phí — gửi yêu cầu khai thác để đọc (Điều 36)"));
        }
    }
    let store = st.store.as_ref().ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let mut goc = None;
    for key in [digitext_storage::f_2fc(project, id), digitext_storage::f_2qs(project, id)] {
        if let Some(b) = store.get(&key).await.map_err(|e| {
            tracing::error!(error = %e, key, "đọc PDF tra cứu lỗi");
            ApiError::internal()
        })? {
            goc = Some(b);
            break;
        }
    }
    let goc = goc.ok_or_else(|| ApiError::conflict("tài liệu chưa có bản số trong kho"))?;
    let ten_luu_tru = digitext_db::projects::get(&st.pool, project).await?.map(|p| p.name).unwrap_or_else(|| "Lưu trữ lịch sử".to_string());
    let font_path = digitext_pdf::f_1cq().ok_or_else(|| ApiError::service_unavailable("không tìm thấy font TTF cho watermark — đặt DIGITEXT_CERT_FONT"))?;
    let font = std::fs::read(&font_path).map_err(|e| {
        tracing::error!(path = %font_path.display(), error = %e, "đọc font lỗi");
        ApiError::internal()
    })?;
    let ban_doc = tokio::task::spawn_blocking(move || digitext_pdf::f_2h7(&goc, &ten_luu_tru, false, &font))
        .await
        .map_err(|_| ApiError::internal())?
        .map_err(|e| ApiError::bad_request(format!("dựng bản đọc lỗi: {e}")))?;
    ghi_nhat_ky(&st, &ts, "xem", project, Some(id), json!({ "tep": filename, "maHoSo": ma_ho_so, "trichYeu": trich_yeu, "nguon": "tra_cuu" })).await;
    Ok(([(header::CONTENT_TYPE, "application/pdf".to_string()), (header::CONTENT_DISPOSITION, "inline".to_string())], Body::from(ban_doc)).into_response())
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct DongNhatKy {
    id: i64,
    at: DateTime<Utc>,
    action: String,
    nguoi: Option<String>,
    ten_dang_nhap: Option<String>,
    document_id: Option<String>,
    detail: Option<serde_json::Value>,
    ip: Option<String>,
}

async fn nhat_ky(State(st): State<AppState>, ts: Ts, Query(q): Query<DuAnQuery>) -> Result<Json<Vec<DongNhatKy>>, ApiError> {
    ts.require_perm(Permission::AccessReview)?;
    project_access(&st, &ts, q.project_id).await?;
    let rows: Vec<DongNhatKy> = sqlx::query_as(
        "SELECT a.id, a.at, a.action, u.display_name AS nguoi, u.username AS ten_dang_nhap, \
                substring(a.target FROM 'document:(.*)') AS document_id, a.detail, a.ip \
         FROM audit_log a LEFT JOIN users u ON u.id = a.actor_id \
         WHERE a.action LIKE 'khai_thac.%' AND a.detail->>'projectId' = $1::text \
         ORDER BY a.at DESC LIMIT $2",
    )
    .bind(q.project_id)
    .bind(q.limit.unwrap_or(500).clamp(1, 5000))
    .fetch_all(&st.pool)
    .await?;
    Ok(Json(rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mau_tim_noi_tu_theo_thu_tu() {
        assert_eq!(mau_tim("quyet  dinh dat"), Some("%quyet%dinh%dat%".into()));
        assert_eq!(mau_tim("50%_a\\b"), Some("%50\\%\\_a\\\\b%".into()));
        assert_eq!(mau_tim("   "), None);
    }

    #[test]
    fn nhan_che_do_han_che() {
        assert!(han_che(Some("02")));
        assert!(han_che(Some("Hạn chế")));
        assert!(han_che(Some("Mật")));
        assert!(!han_che(Some("01")));
        assert!(!han_che(Some("Công khai")));
        assert!(!han_che(None));
    }
}
