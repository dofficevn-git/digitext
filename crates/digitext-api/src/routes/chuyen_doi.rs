// Copyright (c) 2026 DigiText. All rights reserved.
//! CHUYỂN ĐỔI tài liệu lưu trữ số ra GIẤY — Điều 13–16, Phụ lục VI TT 05/2025/TT-BNV (0037, 27/11/2026; đợt 4 nhóm 3B).
//! Lấy PDF gốc trong kho (bản đã ký nếu có), kiểm chữ ký số ngay lúc chuyển đổi (`digitext_sign_verify::nhieu`), dựng trang thể thức
//! "TÀI LIỆU LƯU TRỮ CHUYỂN ĐỔI" (Mục 1 PL VI: sau phần nội dung, trang cuối) bằng `digitext_pdf::manual`, ghép (`ghep::noi_truoc`),
//! in chân trang "Bản in từ tài liệu lưu trữ số … trang i/n", lưu `{project}-chuyen-doi/{id}.pdf`, ghi sổ `chuyen_doi_giay` và nhật
//! ký khai thác `khai_thac.chuyen_doi`. Bản in KHÔNG ký số lại — người có thẩm quyền ký tay + đóng dấu (theo đề xuất, anh chưa chốt 17).
//! - `POST /chuyen-doi {projectId, documentId, mucDich?, nguoiNhan?, canCu?, chucVu?}` → dòng sổ (quyền `access.review`)
//! - `GET  /chuyen-doi?projectId&from&to&limit&offset[&format=xlsx]` — sổ chuyển đổi (quyền `access.view`)
//! - `GET  /chuyen-doi/{id}/download` — PDF đã ghép
use super::chinh_ly::{kiem_phong, pham_vi_phong, project_access};
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use crate::xlsx::{self, CotBang};
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use digitext_core::Permission;
use digitext_db::chuyen_doi as db;
use digitext_pdf::manual::ManualBlock;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row as _;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/chuyen-doi", get(so).post(tao)).route("/chuyen-doi/{id}/download", get(tai))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaoBody {
    project_id: Uuid,
    document_id: Uuid,
    #[serde(default)]
    muc_dich: Option<String>,
    #[serde(default)]
    nguoi_nhan: Option<String>,
    #[serde(default)]
    can_cu: Option<String>,
    /// Chức vụ người có thẩm quyền ký bản in (mặc định "LÃNH ĐẠO CƠ QUAN").
    #[serde(default)]
    chuc_vu: Option<String>,
}

fn ngay_vn(t: DateTime<Utc>) -> chrono::DateTime<chrono::FixedOffset> {
    t.with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00"))
}

fn row_json(r: &db::ChuyenDoiRow) -> Value {
    json!({
        "id": r.id, "project_id": r.project_id, "document_id": r.document_id, "ho_so_id": r.ho_so_id, "van_ban_id": r.van_ban_id, "phong_id": r.phong_id,
        "nam": r.nam, "so_chuyen_doi": r.so_chuyen_doi, "ma_tai_lieu": r.ma_tai_lieu, "ten_tai_lieu": r.ten_tai_lieu, "ma_ho_so": r.ma_ho_so,
        "so_trang": r.so_trang, "muc_dich": r.muc_dich, "nguoi_nhan": r.nguoi_nhan, "can_cu": r.can_cu, "ky_so_ket_luan": r.ky_so_ket_luan,
        "ky_so_chi_tiet": r.ky_so_chi_tiet, "nguoi_thuc_hien": r.nguoi_thuc_hien, "chuc_vu": r.chuc_vu, "pdf_key": r.pdf_key,
        "dung_luong_bytes": r.dung_luong_bytes, "created_at": r.created_at,
    })
}

async fn tao(State(st): State<AppState>, ts: Ts, Json(b): Json<TaoBody>) -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    ts.require_perm(Permission::AccessReview)?;
    project_access(&st, &ts, b.project_id).await?;
    // Tài liệu + mục lục (văn bản / hồ sơ / phông) nếu đã nối.
    let r = sqlx::query(
        "SELECT d.project_id, d.status, d.filename, d.page_count, d.ky_so, \
                v.id AS van_ban_id, h.id AS ho_so_id, h.phong_id, p.ma_du_an, p.ten AS ten_phong, v.so_thu_tu, v.trich_yeu, v.so_ky_hieu, v.ngay_thang, v.co_quan_ban_hanh, \
                hv.ma_ho_so, g.ma_ho_so_goc, h.tieu_de AS tieu_de_ho_so \
         FROM documents d \
         LEFT JOIN LATERAL (SELECT van_ban_id FROM cl_van_ban_file WHERE document_id = d.id AND deleted_at IS NULL LIMIT 1) f ON true \
         LEFT JOIN cl_van_ban v ON v.id = f.van_ban_id \
         LEFT JOIN cl_ho_so h ON h.id = v.ho_so_id \
         LEFT JOIN cl_ho_so_v hv ON hv.id = h.id \
         LEFT JOIN cl_ho_so_goi g ON g.ho_so_id = h.id \
         LEFT JOIN cl_phong p ON p.id = h.phong_id \
         WHERE d.id = $1",
    )
    .bind(b.document_id)
    .fetch_optional(&st.pool)
    .await?
    .ok_or_else(ApiError::not_found)?;
    let project: Uuid = r.try_get("project_id")?;
    if project != b.project_id {
        return Err(ApiError::not_found());
    }
    let status: String = r.try_get("status")?;
    if status == "disposed" {
        return Err(ApiError::conflict("tài liệu đã tiêu huỷ theo quyết định — không chuyển đổi được"));
    }
    let phong_id: Option<Uuid> = r.try_get("phong_id")?;
    let scope = pham_vi_phong(&st, &ts, project).await?;
    match (scope.as_deref(), phong_id) {
        (Some(sc), Some(p)) => kiem_phong(Some(sc), p)?,
        (Some(_), None) => return Err(ApiError::forbidden()),
        _ => {}
    }
    let filename: String = r.try_get("filename")?;
    let page_count: Option<i32> = r.try_get("page_count")?;
    let van_ban_id: Option<Uuid> = r.try_get("van_ban_id")?;
    let ho_so_id: Option<Uuid> = r.try_get("ho_so_id")?;
    let so_thu_tu: Option<i32> = r.try_get("so_thu_tu")?;
    let trich_yeu: Option<String> = r.try_get("trich_yeu")?;
    let so_ky_hieu: Option<String> = r.try_get("so_ky_hieu")?;
    let ngay_thang: Option<String> = r.try_get("ngay_thang")?;
    let co_quan_bh: Option<String> = r.try_get("co_quan_ban_hanh")?;
    let ma_ho_so: Option<String> = r.try_get::<Option<String>, _>("ma_ho_so_goc")?.or(r.try_get("ma_ho_so")?);
    let tieu_de_ho_so: Option<String> = r.try_get("tieu_de_ho_so")?;
    let ten_phong: Option<String> = r.try_get("ten_phong")?;
    let ma_du_an: Option<String> = r.try_get("ma_du_an")?;
    let ma_tai_lieu = match (&ma_ho_so, so_thu_tu) {
        (Some(m), Some(stt)) => format!("{m}.{stt:07}"),
        _ => filename.clone(),
    };
    let ten_tai_lieu = trich_yeu.clone().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| filename.clone());

    // PDF gốc: bản đã ký trước, không có thì bản PDF/A.
    let store = st.store.as_ref().ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let mut goc = None;
    for key in [digitext_storage::f_2fc(project, b.document_id), digitext_storage::f_2qs(project, b.document_id)] {
        if let Some(bytes) = store.get(&key).await.map_err(|e| {
            tracing::error!(error = %e, key, "đọc PDF để chuyển đổi lỗi");
            ApiError::internal()
        })? {
            goc = Some(bytes);
            break;
        }
    }
    let goc = goc.ok_or_else(|| ApiError::conflict("tài liệu chưa có bản số trong kho"))?;
    let cq = digitext_db::co_quan::hieu_dung(&st.pool, project).await?;
    let ten_cq = cq.ten.clone();
    let font_path = digitext_pdf::f_1cq().ok_or_else(|| ApiError::service_unavailable("không tìm thấy font TTF — đặt DIGITEXT_CERT_FONT"))?;
    let font = std::fs::read(&font_path).map_err(|e| {
        tracing::error!(path = %font_path.display(), error = %e, "đọc font lỗi");
        ApiError::internal()
    })?;

    // Kiểm chữ ký số tại thời điểm chuyển đổi (chặn luồng, PDF lớn).
    let goc_cho_kiem = goc.clone();
    let bay_gio = Utc::now().timestamp();
    let (ket_luan, chi_tiet) = tokio::task::spawn_blocking(move || {
        let kq = digitext_sign_verify::nhieu::kiem_tat_ca(&goc_cho_kiem);
        let kl = digitext_sign_verify::nhieu::ket_luan_tep(&kq, bay_gio);
        let nguoi: Vec<String> = kq.chu_ky.iter().filter_map(|c| c.nguoi_ky.as_ref().map(|n| n.signer.clone())).collect();
        let mut ct = Vec::new();
        if !nguoi.is_empty() {
            ct.push(format!("Người ký: {}", nguoi.join("; ")));
        }
        ct.extend(kl.loi.iter().map(|l| format!("Lỗi: {l}")));
        ct.extend(kl.canh_bao.iter().map(|l| format!("Cảnh báo: {l}")));
        (kl.ket_luan, ct.join(" · "))
    })
    .await
    .map_err(|_| ApiError::internal())?;

    let id = Uuid::now_v7();
    let mut tx = st.pool.begin().await?;
    let (nam, so) = db::cap_so(&mut tx, project).await?;
    let luc = ngay_vn(Utc::now());
    let chuc_vu = b.chuc_vu.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| cq.chuc_vu_nguoi_ky.clone());
    let mo_ta_tl = {
        let mut v = vec![ten_tai_lieu.clone()];
        if let Some(s) = so_ky_hieu.as_deref().filter(|s| !s.is_empty()) {
            v.push(format!("Số, ký hiệu: {s}"));
        }
        if let Some(s) = ngay_thang.as_deref().filter(|s| !s.is_empty()) {
            v.push(format!("Ngày ban hành: {s}"));
        }
        if let Some(s) = co_quan_bh.as_deref().filter(|s| !s.is_empty()) {
            v.push(format!("Cơ quan ban hành: {s}"));
        }
        v.join(" — ")
    };
    let mut kv = vec![
        ("Số chuyển đổi".to_string(), so.clone()),
        ("Mã lưu trữ tài liệu gốc".to_string(), ma_tai_lieu.clone()),
        ("Tài liệu gốc".to_string(), mo_ta_tl),
    ];
    if let Some(h) = &tieu_de_ho_so {
        kv.push(("Hồ sơ".into(), format!("{} — {}", ma_ho_so.clone().unwrap_or_default(), h)));
    }
    if let Some(p) = &ten_phong {
        kv.push(("Phông".into(), format!("{} — {}", ma_du_an.clone().unwrap_or_default(), p)));
    }
    kv.push(("Số trang tài liệu".into(), page_count.map(|n| n.to_string()).unwrap_or_default()));
    kv.push(("Chữ ký số tại thời điểm chuyển đổi".into(), if chi_tiet.is_empty() { ket_luan.clone() } else { format!("{ket_luan} — {chi_tiet}") }));
    kv.push(("Nguồn".into(), format!("Hệ thống lưu trữ số DigiText của {ten_cq}")));
    if let Some(m) = b.muc_dich.as_deref().filter(|s| !s.trim().is_empty()) {
        kv.push(("Mục đích".into(), m.trim().into()));
    }
    if let Some(m) = b.nguoi_nhan.as_deref().filter(|s| !s.trim().is_empty()) {
        kv.push(("Người / đơn vị nhận".into(), m.trim().into()));
    }
    if let Some(m) = b.can_cu.as_deref().filter(|s| !s.trim().is_empty()) {
        kv.push(("Căn cứ".into(), m.trim().into()));
    }
    kv.push(("Thời điểm chuyển đổi".into(), luc.format("%d/%m/%Y %H:%M").to_string()));
    kv.push(("Người thực hiện".into(), ts.user.display_name.clone()));
    let blocks = vec![
        ManualBlock::Title("TÀI LIỆU LƯU TRỮ CHUYỂN ĐỔI".into()),
        ManualBlock::Subtitle("(Thể thức tài liệu lưu trữ chuyển đổi từ tài liệu số dạng văn bản — Mục 1 Phụ lục VI Thông tư 05/2025/TT-BNV)".into()),
        ManualBlock::Gap(6.0),
        ManualBlock::KeyValue(kv),
        ManualBlock::Gap(8.0),
        ManualBlock::Paragraph(format!(
            "Tài liệu này được chuyển đổi từ tài liệu lưu trữ số trong Hệ thống của {ten_cq}; nội dung in ra đúng với bản số tại thời điểm chuyển đổi. \
             Bản in có giá trị khi có chữ ký của người có thẩm quyền và dấu của cơ quan, tổ chức."
        )),
        ManualBlock::Gap(10.0),
        ManualBlock::Centered { text: format!("{}, ngày {} tháng {} năm {}", "……………", luc.day(), luc.month(), luc.year()), size: 10.0, bold: false },
        ManualBlock::Gap(6.0),
        ManualBlock::Signatures {
            left: ("NGƯỜI THỰC HIỆN CHUYỂN ĐỔI".into(), "(Ký, ghi rõ họ tên)".into(), ts.user.display_name.clone()),
            right: (chuc_vu.to_uppercase(), "(Chữ ký của người có thẩm quyền, dấu của cơ quan, tổ chức)".into(), if b.chuc_vu.is_some() { String::new() } else { cq.nguoi_ky.clone().unwrap_or_default() }),
        },
    ];
    let font2 = font.clone();
    let the_thuc = tokio::task::spawn_blocking(move || digitext_pdf::manual::render_manual(&blocks, &font2))
        .await
        .map_err(|_| ApiError::internal())?
        .map_err(|e| ApiError::bad_request(format!("dựng trang thể thức lỗi: {e}")))?;
    let so2 = so.clone();
    let ten_cq2 = ten_cq.clone();
    let ghep = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, digitext_pdf::PdfError> {
        let g = digitext_pdf::ghep::noi_truoc(&goc, &the_thuc)?;
        digitext_pdf::ghep::chan_trang(&g, &|i, n| format!("Bản in từ tài liệu lưu trữ số — {ten_cq2} — Số chuyển đổi {so2} — trang {i}/{n}"), &font)
    })
    .await
    .map_err(|_| ApiError::internal())?
    .map_err(|e| ApiError::bad_request(format!("ghép PDF chuyển đổi lỗi: {e}")))?;
    let key = format!("{project}-chuyen-doi/{id}.pdf");
    store.put(&key, &ghep).await.map_err(|e| {
        tracing::error!(error = %e, key, "ghi PDF chuyển đổi vào kho lỗi");
        ApiError::internal()
    })?;
    let row = db::ghi(
        &mut tx,
        &db::Moi {
            id,
            project_id: project,
            document_id: Some(b.document_id),
            ho_so_id,
            van_ban_id,
            phong_id,
            ma_tai_lieu: Some(&ma_tai_lieu),
            ten_tai_lieu: Some(&ten_tai_lieu),
            ma_ho_so: ma_ho_so.as_deref(),
            so_trang: page_count,
            muc_dich: b.muc_dich.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            nguoi_nhan: b.nguoi_nhan.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            can_cu: b.can_cu.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            ky_so_ket_luan: Some(&ket_luan),
            ky_so_chi_tiet: Some(chi_tiet.as_str()).filter(|s| !s.is_empty()),
            nguoi_thuc_hien: ts.user.id,
            chuc_vu: Some(&chuc_vu),
            pdf_key: &key,
            dung_luong_bytes: ghep.len() as i64,
        },
        nam,
        &so,
    )
    .await?;
    tx.commit().await?;
    super::tra_cuu::ghi_nhat_ky(&st, &ts, "chuyen_doi", project, Some(b.document_id), json!({ "soChuyenDoi": so, "maTaiLieu": ma_tai_lieu, "mucDich": b.muc_dich, "nguoiNhan": b.nguoi_nhan })).await;
    Ok((axum::http::StatusCode::CREATED, Json(row_json(&row))))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SoQ {
    project_id: Uuid,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
    #[serde(default)]
    format: Option<String>,
}
fn default_limit() -> i64 {
    500
}

fn vn(s: Option<&str>, end: bool) -> Result<Option<DateTime<Utc>>, ApiError> {
    let Some(s) = s.map(str::trim).filter(|x| !x.is_empty()) else { return Ok(None) };
    let d = NaiveDate::parse_from_str(s, "%d/%m/%Y").or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d")).map_err(|_| ApiError::bad_request(format!("Ngày {s:?} không hợp lệ")))?;
    let d = if end { d.succ_opt().unwrap_or(d) } else { d };
    Ok(Some(d.and_hms_opt(0, 0, 0).unwrap().and_utc() - chrono::Duration::hours(7)))
}

async fn so(State(st): State<AppState>, ts: Ts, Query(q): Query<SoQ>) -> Result<Response, ApiError> {
    if !(ts.has_perm(Permission::AccessView) || ts.has_perm(Permission::AccessReview)) {
        return Err(ApiError::forbidden());
    }
    project_access(&st, &ts, q.project_id).await?;
    let scope = pham_vi_phong(&st, &ts, q.project_id).await?;
    let xlsx_ = q.format.as_deref().map(str::trim).map(str::to_lowercase).as_deref() == Some("xlsx");
    let (rows, total) = db::so(&st.pool, q.project_id, vn(q.from.as_deref(), false)?, vn(q.to.as_deref(), true)?, scope.as_deref(), if xlsx_ { 20_000 } else { q.limit }, if xlsx_ { 0 } else { q.offset }).await?;
    if !xlsx_ {
        return Ok(Json(json!({ "rows": rows, "total": total })).into_response());
    }
    let chu = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let ngay = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).and_then(|s| DateTime::parse_from_rfc3339(s).ok()).map(|d| ngay_vn(d.with_timezone(&Utc)).format("%d/%m/%Y %H:%M").to_string()).unwrap_or_default();
    let cot = [
        CotBang::ma("Số chuyển đổi", 16.0),
        CotBang::ma("Thời điểm", 16.0),
        CotBang::ma("Mã tài liệu gốc", 26.0),
        CotBang::chu("Tài liệu", 44.0),
        CotBang::ma("Mã hồ sơ", 20.0),
        CotBang::chu("Phông", 24.0),
        CotBang::so("Trang", 7.0),
        CotBang::chu("Mục đích", 24.0),
        CotBang::chu("Người nhận", 20.0),
        CotBang::chu("Chữ ký số", 18.0),
        CotBang::chu("Người thực hiện", 20.0),
    ];
    let bang: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            vec![
                chu(r, "so_chuyen_doi"), ngay(r, "created_at"), chu(r, "ma_tai_lieu"), chu(r, "ten_tai_lieu"), chu(r, "ma_ho_so"), chu(r, "ten_phong"),
                r.get("so_trang").and_then(Value::as_i64).map(|n| n.to_string()).unwrap_or_default(), chu(r, "muc_dich"), chu(r, "nguoi_nhan"), chu(r, "ky_so_ket_luan"), chu(r, "nguoi_thuc_hien_ten"),
            ]
        })
        .collect();
    let ten_cq = digitext_db::co_quan::hieu_dung(&st.pool, q.project_id).await?.ten;
    let sheet = xlsx::sheet_bang("Sổ chuyển đổi", &[ten_cq, "SỔ CHUYỂN ĐỔI TÀI LIỆU LƯU TRỮ SỐ RA GIẤY".into(), format!("{} lượt — Điều 13–16, Phụ lục VI TT 05/2025/TT-BNV", rows.len())], &cot, &bang, None);
    let bytes = xlsx::dong_goi(&[sheet]).map_err(|e| {
        tracing::error!(error = %e, "đóng gói Excel sổ chuyển đổi lỗi");
        ApiError::internal()
    })?;
    Ok(super::bao_cao::tep(bytes, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "so-chuyen-doi.xlsx"))
}

async fn tai(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Response, ApiError> {
    if !(ts.has_perm(Permission::AccessView) || ts.has_perm(Permission::AccessReview)) {
        return Err(ApiError::forbidden());
    }
    let row = db::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    project_access(&st, &ts, row.project_id).await?;
    if let Some(p) = row.phong_id {
        kiem_phong(pham_vi_phong(&st, &ts, row.project_id).await?.as_deref(), p)?;
    }
    let store = st.store.as_ref().ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    let bytes = store
        .get(&row.pdf_key)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, key = %row.pdf_key, "đọc PDF chuyển đổi lỗi");
            ApiError::internal()
        })?
        .ok_or_else(|| ApiError::conflict("bản PDF chuyển đổi không còn trong kho"))?;
    Ok(super::bao_cao::tep(bytes, "application/pdf", &format!("chuyen-doi-{}.pdf", row.so_chuyen_doi)))
}
