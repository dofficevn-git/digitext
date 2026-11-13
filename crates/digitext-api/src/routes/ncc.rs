// Copyright (c) 2026 DigiText. All rights reserved.
//! Cửa đọc sản lượng cho **cổng nhà cung cấp** của hệ thống ngoài (10/11/2026).
//!
//! `POST /api/v1/ncc/san-luong` — hệ thống quản lý hợp đồng hỏi "dự án này, khoảng ngày
//! này, đội này làm được bao nhiêu", rồi tự vẽ bằng giao diện của nó. Nhà thầu phụ hết
//! phải gõ lại sản lượng lần hai, và số nghiệm thu không còn lệch với số đã làm thật.
//!
//! **Vì sao POST chứ không GET**: chuỗi ký là `<timestamp>.<thân gói>`. Để tham số trong
//! thân JSON thì toàn bộ tham số nằm trong vùng ký, không lọt vào nhật ký proxy, và bên
//! gọi dùng lại y nguyên hàm ký đã viết cho `POST /hooks/du-an` — không phải viết thêm
//! mã ký nào. Dùng GET thì query nằm ngoài vùng ký: ai bắt được một gói tin có thể đổi
//! `maNgoai` sang dự án khác trong 300 giây mà chữ ký vẫn hợp lệ.
//!
//! **Bên gọi là người NGOÀI.** Mọi thứ ở đây là số gộp mức dự án. Không trả tên người,
//! không trả KPI cá nhân, không trả ngưỡng quản lý, không trả ghi chú nội bộ — xem mục
//! "Không được để lọt" trong `docs/WEBHOOK-NCC.md`.
//!
//! Xác thực dùng chung khoá `api_clients` với webhook (`routes::hooks::xac_thuc`), nên
//! một hệ thống chỉ đọc được dự án nó đã tạo: `nguon` lấy từ khoá, không từ thân gói.

use crate::app::AppState;
use crate::client_ip::T2t;
use crate::error::ApiError;
use crate::routes::dashboard::{health_of, thresholds_of};
use crate::routes::hooks::{doc_ngay_opt, xac_thuc};
use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Json, Router};
use chrono::NaiveDate;
use digitext_db::dashboard::{self as dash, Metrics};
use digitext_db::chinh_ly::ncc as cl_ncc;
use digitext_db::documents as docs;
use digitext_db::progress;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// Rollup chạy mỗi 120 giây (xem `routes::dashboard::spawn_rollup`), nên số của hôm nay
/// luôn trễ tới chừng này. Nói ra để bên kia không tưởng là số tức thời.
const DO_TRE_TOI_DA_GIAY: i64 = 120;

/// Khoảng ngày mặc định khi gói tin không nói.
const SO_NGAY_MAC_DINH: i64 = 30;

/// Trần khoảng ngày — chặn một lời gọi kéo cả lịch sử.
const SO_NGAY_TOI_DA: i64 = 366;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ncc/san-luong", post(san_luong))
        .route("/ncc/tu-kiem/da-nop", post(tu_kiem_da_nop))
        .route("/ncc/tu-kiem/muc-luc", post(tu_kiem_muc_luc).layer(axum::extract::DefaultBodyLimit::max(12 * 1024 * 1024)))
        .route("/ncc/tu-kiem/muc-luc/mau", post(muc_luc_mau))
        .route("/ncc/tu-kiem/pdf", post(tu_kiem_pdf).layer(axum::extract::DefaultBodyLimit::max(48 * 1024 * 1024)))
        .route("/ncc/tu-kiem/goi", post(tu_kiem_goi).layer(axum::extract::DefaultBodyLimit::max(24 * 1024 * 1024)))
}

// ---------------------------------------------------------------- nhóm chỉ số

/// `(cột dashboard_daily, khoá JSON, nhãn tiếng Việt, đơn vị)`.
type ChiSo = (&'static str, &'static str, &'static str, &'static str);

/// Đội "Số hoá" bên hệ thống ngoài → các cột đo của dây chuyền số hoá.
const NHOM_SO_HOA: &[ChiSo] = &[
    ("pages_scanned", "trangQuet", "Trang đã quét", "trang"),
    ("docs_uploaded", "tepDuaLen", "Tệp đưa lên", "tệp"),
    ("pages_ocr_done", "trangOcrXong", "Trang OCR xong", "trang"),
    ("docs_pdfa", "tepPdfa", "Tệp đã ghép PDF/A", "tệp"),
    ("docs_signed", "tepKySo", "Tệp đã ký số", "tệp"),
    ("docs_reviewed_approved", "tepDuyetDat", "Tệp duyệt đạt", "tệp"),
    ("docs_reviewed_rejected", "tepTraLai", "Tệp bị trả lại", "tệp"),
];

/// Đội "Chỉnh lý" → các cột đo của dây chuyền chỉnh lý.
const NHOM_CHINH_LY: &[ChiSo] = &[
    ("cl_ho_so_new", "hoSoMoi", "Hồ sơ lập mới", "hồ sơ"),
    ("cl_ho_so_edited", "hoSoSua", "Hồ sơ có sửa", "hồ sơ"),
    ("cl_van_ban_new", "vanBanMoi", "Văn bản lập mới", "văn bản"),
    ("cl_fields_entered", "truongDaNhap", "Trường đã nhập", "trường"),
];

/// Tên đội bên ngoài → mã nội bộ. Nhận cả dấu gạch, không dấu và có dấu.
const DOI: &[(&str, &[&str])] = &[
    (
        "so_hoa",
        &[
            "so_hoa", "so-hoa", "sohoa", "so hoa", "số hoá", "số hóa", "sô hoa", "scan", "quét",
            "quet", "digitize", "digitise",
        ],
    ),
    (
        "chinh_ly",
        &[
            "chinh_ly",
            "chinh-ly",
            "chinhly",
            "chinh ly",
            "chỉnh lý",
            "chinh lý",
            "systematize",
        ],
    ),
    (
        "tat_ca",
        &["tat_ca", "tat-ca", "tatca", "tat ca", "tất cả", "all", "*"],
    ),
];

/// Chuẩn hoá tên đội. `None` = không hiểu.
fn chuan_doi(v: &str) -> Option<&'static str> {
    let v = v.trim().to_lowercase();
    if v.is_empty() {
        return None;
    }
    DOI.iter()
        .find(|(_, bidanh)| bidanh.iter().any(|b| *b == v))
        .map(|(ma, _)| *ma)
}

fn ds_doi() -> String {
    DOI.iter()
        .map(|(ma, _)| *ma)
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Bộ chỉ số của một đội. `tat_ca` = cả hai dây chuyền.
fn chi_so_cua(doi: &str) -> Vec<ChiSo> {
    match doi {
        "so_hoa" => NHOM_SO_HOA.to_vec(),
        "chinh_ly" => NHOM_CHINH_LY.to_vec(),
        _ => NHOM_SO_HOA
            .iter()
            .chain(NHOM_CHINH_LY.iter())
            .copied()
            .collect(),
    }
}

// ---------------------------------------------------------------- gói tin

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoiHoi {
    #[serde(default, alias = "maHopDong", alias = "maDa", alias = "maDuAnNgoai")]
    ma_ngoai: Option<String>,
    #[serde(default, alias = "to", alias = "nhom", alias = "doiThiCong")]
    doi: Option<String>,
    #[serde(default, alias = "tu", alias = "tuNgayLam")]
    tu_ngay: Option<String>,
    #[serde(default, alias = "den", alias = "denNgayLam")]
    den_ngay: Option<String>,
}

// ---------------------------------------------------------------- handler

async fn san_luong(
    State(st): State<AppState>,
    ip: T2t,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let kh = xac_thuc(&st, &headers, &ip, &body).await?;
    let g: GoiHoi = serde_json::from_slice(&body)
        .map_err(|e| ApiError::bad_request(format!("thân gói không phải JSON hợp lệ: {e}")))?;

    let ma_ngoai = g
        .ma_ngoai
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            ApiError::bad_request("thiếu maNgoai — mã dự án bên hệ thống của bạn, đúng mã đã gửi khi tạo dự án")
        })?;

    let doi = match g.doi.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        None => "tat_ca",
        Some(v) => chuan_doi(v).ok_or_else(|| {
            ApiError::bad_request(format!("doi = {v:?} không hiểu — dùng {}", ds_doi()))
        })?,
    };

    let (tu, den) = khoang_ngay(&g)?;

    // Phạm vi đọc do KHOÁ quyết định: `nguon` lấy từ `kh`, không lấy từ thân gói. Dự án
    // của nguồn khác và dự án không tồn tại trả cùng một lỗi — không để dò được sự tồn tại.
    let du_an = digitext_db::webhooks::tim_theo_ma_ngoai(&st.pool, &kh.nguon, ma_ngoai)
        .await?
        .ok_or_else(|| ApiError {
            status: axum::http::StatusCode::NOT_FOUND,
            code: "ncc_du_an_khong_thay",
            message: format!("không có dự án nào mang maNgoai {ma_ngoai:?}"),
        })?;
    // Mọi hàm bên dưới nhận `Option<&[Uuid]>` mà None = MỌI DỰ ÁN. Đi qua đúng một chỗ này.
    let pham_vi = [du_an.id];

    let chi_so = chi_so_cua(doi);
    let theo_ngay = dash::project_daily(&st.pool, Some(&pham_vi), tu, den).await?;
    let lich: BTreeMap<NaiveDate, Metrics> = theo_ngay.iter().map(|r| (r.day, r.m)).collect();

    // `dashboard_rollup` xoá dòng toàn 0, nên ngày nghỉ không có dòng. Lấp đủ để đồ thị
    // bên kia không nhảy ngày.
    let mut ngay: Vec<Value> = Vec::new();
    let mut tong = Map::new();
    let mut cong: Vec<i64> = vec![0; chi_so.len()];
    let mut d = tu;
    loop {
        let m = lich.get(&d).copied().unwrap_or_default();
        let mut dong = Map::new();
        dong.insert("ngay".into(), json!(d.to_string()));
        for (i, (cot, khoa, _, _)) in chi_so.iter().enumerate() {
            let v = m.get(cot);
            cong[i] += v;
            dong.insert((*khoa).into(), json!(v));
        }
        ngay.push(Value::Object(dong));
        match d.succ_opt() {
            Some(ke) if ke <= den => d = ke,
            _ => break,
        }
    }
    for (i, (_, khoa, _, _)) in chi_so.iter().enumerate() {
        tong.insert((*khoa).into(), json!(cong[i]));
    }

    let mut ra = Map::new();
    ra.insert("maNgoai".into(), json!(ma_ngoai));
    ra.insert("maDuAn".into(), json!(du_an.code));
    ra.insert("tenDuAn".into(), json!(du_an.name));
    ra.insert("loaiDuAn".into(), json!(du_an.loai));
    ra.insert("dangHoatDong".into(), json!(du_an.is_active));
    ra.insert("doi".into(), json!(doi));
    ra.insert("tuNgay".into(), json!(tu.to_string()));
    ra.insert("denNgay".into(), json!(den.to_string()));
    ra.insert(
        "chiSo".into(),
        json!(chi_so
            .iter()
            .map(|(_, khoa, ten, dv)| json!({ "ma": khoa, "ten": ten, "donVi": dv }))
            .collect::<Vec<_>>()),
    );
    ra.insert("ngay".into(), Value::Array(ngay));
    ra.insert("tong".into(), Value::Object(tong));

    // Luỹ kế + hợp đồng + tiến độ.
    //
    // `project_progress` có `WHERE p.is_active` nên dự án đã đóng trả RỖNG. Lúc đó lấy
    // luỹ kế từ `pipeline_counts` và bỏ phần tiến độ — tuyệt đối không để thành 500.
    let dong_td = progress::project_progress(&st.pool, Some(&pham_vi), None, None)
        .await?
        .into_iter()
        .next();

    match dong_td {
        Some(r) => {
            ra.insert(
                "luyKe".into(),
                json!({
                    "tep": r.files, "trang": r.pages, "tepPdfa": r.pdfa_files,
                    "tepKySo": r.signed_files, "tepOcrXong": r.ocr_files,
                    "tepDuyetDat": r.approved_files, "tepTraLai": r.rejected_files,
                }),
            );
            ra.insert("hopDong".into(), hop_dong_json(&r));

            let today = digitext_core::dashboard::vn_today();
            let (rag, _) = thresholds_of(&dash::get_kpi_settings(&st.pool, du_an.id).await?);
            let h = health_of(&r, &lich, today, &rag);
            // Vắng mặt khi không có mẫu số — đừng trả 0% làm người ta tưởng chưa làm gì.
            if h.progress_pct.is_some() || h.time_pct.is_some() {
                ra.insert(
                    "tienDo".into(),
                    json!({
                        "phanTram": h.progress_pct,
                        "phanTramThoiGian": h.time_pct,
                        "lech": h.lech,
                        "conLaiNgay": h.days_remaining,
                        "duKienXongNgay": h.projected_finish_on,
                        "congThuc": "tb-4-buoc-may-chu",
                    }),
                );
            }
        }
        None => {
            let p = dash::pipeline_counts(&st.pool, du_an.id).await?;
            ra.insert(
                "luyKe".into(),
                json!({
                    "tep": p.files, "trang": p.pages, "tepPdfa": p.pdfa_files,
                    "tepKySo": p.signed_files, "tepOcrXong": p.ocr_done_docs,
                    "tepDuyetDat": p.approved_files, "tepTraLai": p.rejected_files,
                }),
            );
            ra.insert("hopDong".into(), Value::Null);
            ra.insert(
                "ghiChu".into(),
                json!("dự án đã đóng — không tính tiến độ hợp đồng"),
            );
        }
    }

    ra.insert("capNhatLuc".into(), json!(chrono::Utc::now().to_rfc3339()));
    ra.insert("doTreToiDaGiay".into(), json!(DO_TRE_TOI_DA_GIAY));
    // Hệ thống này không biết nhà thầu nào làm phần nào: số trả về là của CẢ dự án trong
    // nhóm chỉ số đó. Một đội có hai nhà thầu thì phải tách thành hai dự án (maNgoai khác).
    ra.insert("gopNhieuNhaThau".into(), json!(true));

    Ok(Json(Value::Object(ra)))
}

/// Hợp đồng — bỏ `note` (ghi chú nội bộ). `null` khi chưa khai gì.
fn hop_dong_json(r: &progress::ProgressRow) -> Value {
    let trong = r.signed_on.is_none()
        && r.end_on.is_none()
        && r.deadline_on.is_none()
        && r.contract_pages.is_none()
        && r.contract_files.is_none()
        && r.contract_boxes.is_none()
        && r.contract_dossiers.is_none()
        && r.contract_sheets.is_none();
    if trong {
        return Value::Null;
    }
    json!({
        "ngayKy": r.signed_on,
        "ngayKetThuc": r.end_on,
        "hanChot": r.deadline_on,
        "soTrang": r.contract_pages,
        "soTep": r.contract_files,
        "soHop": r.contract_boxes,
        "soHoSo": r.contract_dossiers,
        "soTo": r.contract_sheets,
        "soMet": r.contract_meters,
        "nguoiPhuTrach": r.manager_name,
        "nguon": "digitext",
    })
}

/// Khoảng ngày từ gói tin, đã kẹp và kiểm. Mặc định 30 ngày tính ngược từ hôm nay (giờ VN).
fn khoang_ngay(g: &GoiHoi) -> Result<(NaiveDate, NaiveDate), ApiError> {
    let hom_nay = digitext_core::dashboard::vn_today();
    let den = doc_ngay_opt(&g.den_ngay, "denNgay")?.unwrap_or(hom_nay);
    let tu = doc_ngay_opt(&g.tu_ngay, "tuNgay")?
        .unwrap_or_else(|| den - chrono::Duration::days(SO_NGAY_MAC_DINH - 1));
    kep_khoang(tu, den)
}

fn kep_khoang(tu: NaiveDate, den: NaiveDate) -> Result<(NaiveDate, NaiveDate), ApiError> {
    if tu > den {
        return Err(ApiError::bad_request(format!(
            "tuNgay ({tu}) sau denNgay ({den})"
        )));
    }
    let so_ngay = (den - tu).num_days() + 1;
    if so_ngay > SO_NGAY_TOI_DA {
        return Err(ApiError::bad_request(format!(
            "khoảng {so_ngay} ngày quá dài — tối đa {SO_NGAY_TOI_DA} ngày một lần hỏi"
        )));
    }
    Ok((tu, den))
}

// ------------------------------------------------- tự kiểm: lỗi máy chủ đã bắt trên thứ đã nộp

/// Trần số dòng trả về một lần, và mặc định khi gói tin không nói.
const GIOI_HAN_MAC_DINH: i64 = 200;
const GIOI_HAN_TOI_DA: i64 = 1000;

/// Có trả nguyên văn ghi chú của người duyệt (`documents.review_note`) ra ngoài hay không.
///
/// Mặc định KHÔNG, và đó là chủ ý: ô ghi chú đó hiện được viết với giả định "chỉ người trong nhà
/// đọc" — người duyệt có thể gõ câu không định cho nhà thầu thấy. Muốn bật thì đổi nhãn ô nhập
/// bên desktop TRƯỚC, rồi đặt `DIGITEXT_NCC_LY_DO=true`. Số đếm và loại lỗi thì luôn trả.
fn tra_ly_do() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| {
        std::env::var("DIGITEXT_NCC_LY_DO").map(|v| v.trim().eq_ignore_ascii_case("true") || v.trim() == "1").unwrap_or(false)
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoiKiem {
    #[serde(alias = "ma_ngoai", alias = "maDuAnNgoai")]
    ma_ngoai: Option<String>,
    #[serde(alias = "gioi_han", alias = "soDong")]
    gioi_han: Option<i64>,
}

/// `POST /api/v1/ncc/tu-kiem/da-nop` — những gì máy chủ ĐÃ bắt lỗi trên tài liệu nhà thầu đã nộp.
///
/// Không tải tệp lên, không chạy bộ kiểm nào: ba con số này đã nằm trong cơ sở dữ liệu từ lúc
/// tải lên và lúc người duyệt bấm. Nhà thầu hôm nay chỉ biết mình sai sau khi bị trả lại cả đợt;
/// cửa này cho họ thấy ngay từng tệp.
async fn tu_kiem_da_nop(
    State(st): State<AppState>,
    ip: T2t,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let kh = xac_thuc(&st, &headers, &ip, &body).await?;
    let g: GoiKiem = serde_json::from_slice(&body)
        .map_err(|e| ApiError::bad_request(format!("thân gói không phải JSON hợp lệ: {e}")))?;

    let ma_ngoai = g
        .ma_ngoai
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            ApiError::bad_request("thiếu maNgoai — mã dự án bên hệ thống của bạn, đúng mã đã gửi khi tạo dự án")
        })?;

    let gioi_han = kep_gioi_han(g.gioi_han);

    // Phạm vi do KHOÁ quyết định, y như `/ncc/san-luong`. Dự án của nguồn khác và dự án không
    // tồn tại trả cùng một lỗi.
    let du_an = digitext_db::webhooks::tim_theo_ma_ngoai(&st.pool, &kh.nguon, ma_ngoai)
        .await?
        .ok_or_else(|| ApiError {
            status: axum::http::StatusCode::NOT_FOUND,
            code: "ncc_du_an_khong_thay",
            message: format!("không có dự án nào mang maNgoai {ma_ngoai:?}"),
        })?;
    let pham_vi = [du_an.id];

    let tong = docs::ncc_loi_tong(&st.pool, &pham_vi).await?;
    // Lấy thừa một dòng để biết có bị cắt hay không — nói ra để bên kia bày "còn nữa".
    let mut ds = docs::ncc_loi_ds(&st.pool, &pham_vi, gioi_han + 1).await?;
    let bi_cat = ds.len() as i64 > gioi_han;
    ds.truncate(gioi_han as usize);

    let co_ly_do = tra_ly_do();
    let tep: Vec<Value> = ds
        .iter()
        .map(|r| {
            let mut o = Map::new();
            o.insert("loai".into(), json!(r.loai));
            o.insert("ten".into(), json!(r.filename));
            o.insert("phong".into(), json!(r.phong));
            o.insert("hop".into(), json!(r.hop));
            o.insert("hoSo".into(), json!(r.ho_so));
            o.insert("soTrang".into(), json!(r.page_count));
            o.insert("luc".into(), json!(r.reviewed_at.map(|t| t.to_rfc3339())));
            if co_ly_do {
                o.insert("lyDo".into(), json!(r.review_note));
            }
            Value::Object(o)
        })
        .collect();

    let dem = |ma: &str| match ma {
        "tra_lai" => tong.tra_lai,
        "pdfa_cu" => tong.pdfa_cu,
        _ => tong.boc_truong_loi,
    };
    let loai: Vec<Value> = docs::NCC_LOAI_LOI
        .iter()
        .map(|(ma, ten)| json!({ "ma": ma, "ten": ten, "soTep": dem(ma) }))
        .collect();

    Ok(Json(json!({
        "maNgoai": ma_ngoai,
        "maDuAn": du_an.code,
        "tenDuAn": du_an.name,
        "dangHoatDong": du_an.is_active,
        "tongTep": tong.tong_tep,
        "loai": loai,
        "tep": tep,
        "biCat": bi_cat,
        "gioiHan": gioi_han,
        "coLyDo": co_ly_do,
    })))
}

/// Trần cứng, không báo lỗi: hỏi quá thì cắt xuống, vì đây là lệnh đọc, cắt bớt không sai nghiệp vụ.
fn kep_gioi_han(v: Option<i64>) -> i64 {
    match v {
        None => GIOI_HAN_MAC_DINH,
        Some(n) if n < 1 => 1,
        Some(n) => n.min(GIOI_HAN_TOI_DA),
    }
}

// ------------------------------------------------- tự kiểm: mục lục hồ sơ (tệp Excel tải lên)

/// Trần tệp Excel nhận vào. Mục lục lớn nhất trong bộ 23 tệp thật của Bắc Giang là 504 KiB
/// (2.829 dòng), nên 8 MiB là rất rộng rồi; để rộng hơn chỉ mở đường cho người ta bắn tệp rác.
const MUC_LUC_BYTE_TOI_DA: usize = 8 * 1024 * 1024;

/// Trần số dòng hồ sơ trả chi tiết. Nhập thật chặn ở `IMPORT_MAX = 5000` dòng
/// (`routes::chinh_ly::import`) nên kiểm quá con số đó là vô nghĩa — nói thẳng thay vì cắt im.
const MUC_LUC_DONG_TOI_DA: usize = 5000;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoiMucLuc {
    #[serde(alias = "ma_ngoai", alias = "maDuAnNgoai")]
    ma_ngoai: Option<String>,
    #[serde(alias = "ten_tep")]
    ten_tep: Option<String>,
    /// Nội dung tệp `.xlsx` / `.xls`, mã hoá base64.
    #[serde(alias = "noi_dung", alias = "tep")]
    noi_dung: Option<String>,
    /// Sheet muốn đọc; không có → sheet đầu tiên tìm được dòng tiêu đề cột.
    sheet: Option<String>,
    /// Tên THBQ dùng cho ô THBQ trống; không có → ô trống là lỗi dòng.
    #[serde(alias = "thbq_mac_dinh")]
    thbq_mac_dinh: Option<String>,
    /// Có trả danh sách dòng có vấn đề hay không (mặc định có).
    #[serde(alias = "chi_tiet")]
    chi_tiet: Option<bool>,
}

/// `POST /api/v1/ncc/tu-kiem/muc-luc` — nhà thầu tự kiểm tệp mục lục hồ sơ **trước khi nộp**.
///
/// Toàn bộ việc đọc và kiểm chạy ở máy chủ DigiText (chốt 10/11/2026): cổng nhà cung cấp chỉ
/// gửi tệp lên và bày kết quả, không giữ một dòng luật nào. Bộ đọc là
/// `digitext-mucluc`, đã đối chứng khớp 23/23 mục lục thật và 10.171/10.171 dòng với bản TS
/// đang chạy trong desktop.
///
/// Hai tầng kiểm, và tầng thứ hai là tầng không thể làm ở trình duyệt:
/// 1. **Trong tệp** — cột, tiêu đề, thời gian, THBQ, trùng hồ sơ số trong chính tệp.
/// 2. **Với kho** — hồ sơ số đã có trên máy chủ, đếm theo từng phông của dự án.
async fn tu_kiem_muc_luc(
    State(st): State<AppState>,
    ip: T2t,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let kh = xac_thuc(&st, &headers, &ip, &body).await?;
    let g: GoiMucLuc = serde_json::from_slice(&body)
        .map_err(|e| ApiError::bad_request(format!("thân gói không phải JSON hợp lệ: {e}")))?;

    let ma_ngoai = g
        .ma_ngoai
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ApiError::bad_request("thiếu maNgoai — mã dự án bên hệ thống của bạn"))?;
    let b64 = g
        .noi_dung
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ApiError::bad_request("thiếu noiDung — nội dung tệp Excel mã hoá base64"))?;

    let tep: Vec<u8> = {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|_| ApiError::bad_request("noiDung không phải base64 hợp lệ"))?
    };
    if tep.is_empty() {
        return Err(ApiError::bad_request("tệp rỗng"));
    }
    if tep.len() > MUC_LUC_BYTE_TOI_DA {
        return Err(ApiError::bad_request(format!(
            "tệp {} MiB vượt trần {} MiB",
            tep.len() / 1024 / 1024,
            MUC_LUC_BYTE_TOI_DA / 1024 / 1024
        )));
    }

    let du_an = digitext_db::webhooks::tim_theo_ma_ngoai(&st.pool, &kh.nguon, ma_ngoai)
        .await?
        .ok_or_else(|| ApiError {
            status: axum::http::StatusCode::NOT_FOUND,
            code: "ncc_du_an_khong_thay",
            message: format!("không có dự án nào mang maNgoai {ma_ngoai:?}"),
        })?;

    // Danh mục THBQ của chính dự án — bộ đọc cần nó để khớp cột THBQ và biết loại bảo quản.
    let dm = cl_ncc::thbq_cua_du_an(&st.pool, du_an.id).await?;
    let ten_dm: Vec<String> = dm.iter().map(|t| t.ten.clone()).collect();
    let ten_vv: Vec<String> = dm.iter().filter(|t| t.vinh_vien).map(|t| t.ten.clone()).collect();
    // Chốt an toàn, không phải việc của nhà thầu: migration 0012 gieo 14 dòng THBQ dùng chung
    // nên nhánh này chỉ nổ khi bộ gieo bị xoá ⇒ là lỗi cấu hình máy chủ, nói đúng như vậy.
    if ten_dm.is_empty() {
        return Err(ApiError::bad_request(
            "máy chủ không có Danh mục thời hạn bảo quản nào — lỗi cấu hình phía DigiText, báo quản trị hệ thống",
        ));
    }

    // Đọc + phân tích là việc nặng CPU trên tệp vài nghìn dòng ⇒ đẩy sang luồng chặn, đừng
    // chiếm worker Tokio (cùng lý do với `password::f_2pn` ở `routes::hooks`).
    let ten_tep = g.ten_tep.as_deref().unwrap_or("muc-luc.xlsx").trim().to_owned();
    let sheet = g.sheet.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned);
    let thbq_md = g.thbq_mac_dinh.clone();
    let pt = tokio::task::spawn_blocking(move || {
        let so = digitext_mucluc::doc_so_tu_bytes(&tep, sheet.as_deref())?;
        // Khác bản TS một chủ ý: hộp thoại bên desktop chỉ cho chọn trong danh sách sheet có
        // thật nên không bao giờ xin sai, còn qua API thì xin sai là chuyện thường. Im lặng đọc
        // sheet khác rồi trả số của sheet đó là cách làm người ta tin vào số sai.
        if let Some(xin) = sheet.as_deref() {
            if so.sheet != xin {
                return Err(format!("tệp không có sheet {xin:?} — các sheet có trong tệp: {}", so.sheets.join(", ")));
            }
        }
        Ok::<_, String>(digitext_mucluc::phan_tich::phan_tich_muc_luc(
            &so.rows,
            &digitext_mucluc::phan_tich::TuyChon {
                ten_tep: &ten_tep,
                sheet: &so.sheet,
                sheets: &so.sheets,
                thbq_danh_muc: &ten_dm,
                thbq_vinh_vien: &ten_vv,
                thbq_mac_dinh: thbq_md.as_deref(),
            },
        ))
    })
    .await
    .map_err(|_| ApiError::internal())?
    .map_err(ApiError::bad_request)?;

    if pt.rows.len() > MUC_LUC_DONG_TOI_DA {
        return Err(ApiError::bad_request(format!(
            "tệp có {} dòng hồ sơ, vượt trần {MUC_LUC_DONG_TOI_DA} dòng một lần nhập — tách tệp theo phông hoặc theo hộp",
            pt.rows.len()
        )));
    }

    // Tầng 2: đối chiếu với hồ sơ ĐÃ có trên máy chủ. Khoá duy nhất thật là theo phông, mà cổng
    // không biết sẽ nhập vào phông nào ⇒ đếm cho từng phông, bên kia tự bày.
    let (vv, st_key): (Vec<bool>, Vec<String>) = pt
        .rows
        .iter()
        .filter(|r| !r.ho_so_so.is_empty())
        .map(|r| (r.vinh_vien.unwrap_or(false), r.ho_so_so.trim().to_lowercase()))
        .unzip();
    let trung = cl_ncc::trung_so_tam_theo_phong(&st.pool, du_an.id, &vv, &st_key).await?;

    // Phông: tệp mục lục ghi "Phông: …" ở phần đầu. Bên desktop dùng chính tên đó để CẢNH BÁO
    // khi nó lệch phông người dùng đang chọn (`NhapMucLucDialog.tsx:84`), chứ không bao giờ dùng
    // để chọn phông — phông do cán bộ lưu trữ mở và chọn lúc nhập. Ở đây cũng vậy: khớp được thì
    // trả số trùng của đúng phông ấy, không khớp thì để nguyên bảng cho bên kia bày.
    let phong_ds = cl_ncc::phong_cua_du_an(&st.pool, du_an.id).await?;
    let trong_tep = pt.ten_phong_trong_tep.trim();
    let khop = (!trong_tep.is_empty())
        .then(|| phong_ds.iter().find(|p| gon_ten(p) == gon_ten(trong_tep)).cloned())
        .flatten();
    let so_trung_phong = khop
        .as_ref()
        .map(|k| trung.iter().find(|t| &t.phong_ten == k).map(|t| t.so_trung).unwrap_or(0));

    let chi_tiet = g.chi_tiet.unwrap_or(true);
    let (dong, bi_cat) = if chi_tiet { dong_co_van_de(&pt) } else { (Vec::new(), false) };

    Ok(Json(json!({
        "maNgoai": ma_ngoai,
        "maDuAn": du_an.code,
        "tenDuAn": du_an.name,
        "tenTep": pt.ten_tep,
        "sheet": pt.sheet,
        "sheets": pt.sheets,
        "dongTieuDeCot": pt.dong_tieu_de_cot,
        "cot": pt.cot.khoa(),
        "soHoSo": pt.rows.len(),
        "soPhongBan": pt.phong_bans.len(),
        "khongChiaPhongBan": pt.khong_chia_phong_ban,
        "soHop": pt.so_hop,
        "thbqTep": pt.thbq_tep,
        "soLoi": pt.so_loi,
        "soCanhBao": pt.so_canh_bao,
        "soDaDien": pt.so_da_dien,
        "nhapDuoc": !digitext_mucluc::phan_tich::co_loi_chan(&pt),
        "vanDe": pt.van_de.iter().map(|v| json!({
            "dong": v.dong,
            "muc": if v.muc == digitext_mucluc::phan_tich::Muc::Loi { "loi" } else { "canh_bao" },
            "noiDung": v.noi_dung,
        })).collect::<Vec<_>>(),
        "dong": dong,
        "biCat": bi_cat,
        "trungVoiKho": trung.iter().map(|t| json!({ "phong": t.phong_ten, "soTrung": t.so_trung })).collect::<Vec<_>>(),
        "phong": {
            "trongTep": pt.ten_phong_trong_tep,
            "khop": khop,
            "soTrung": so_trung_phong,
            "soPhongTrongDuAn": phong_ds.len(),
        },
        "thbqDanhMuc": dm.iter().map(|t| json!({ "ten": t.ten, "vinhVien": t.vinh_vien })).collect::<Vec<_>>(),
    })))
}

/// Các dòng hồ sơ **có vấn đề**, kèm cờ "danh sách bị cắt".
///
/// Phép lọc ở đây phải phủ **đúng** tập dòng mà `so_loi` / `so_canh_bao` đếm, nếu không màn hình
/// bên kia hiện "Cảnh báo 59" rồi mở ra không có dòng nào. Đã sập một lần (13/11/2026): bản cũ
/// lọc theo `loi` / `canh_bao`, trong khi `so_canh_bao` còn đếm cả dòng có **kết luận thời gian**
/// là cảnh báo/lỗi — mà lời giải thích của loại đó nằm trong `tg.ghi_chu`, không nằm trong
/// `canh_bao`.
///
/// Vì vậy `canhBao` trả ra là `canh_bao` **cộng** `tg.ghi_chu` khi kết luận thời gian không
/// sạch: đúng bằng thứ `so_canh_bao` đã tính, nên hai con số không thể lệch nhau. Chi tiết
/// thời gian vẫn trả nguyên trong `thoiGian` cho bên nào muốn bày riêng.
fn dong_co_van_de(pt: &digitext_mucluc::phan_tich::PhanTich) -> (Vec<Value>, bool) {
    use digitext_mucluc::thoi_gian::KetLuan;
    let tg_co_y = |k: KetLuan| matches!(k, KetLuan::CanhBao | KetLuan::Loi);

    let mut dong: Vec<Value> = Vec::new();
    let mut bi_cat = false;
    for r in pt
        .rows
        .iter()
        .filter(|r| !r.loi.is_empty() || !r.canh_bao.is_empty() || tg_co_y(r.tg.ket_luan))
    {
        if dong.len() >= GIOI_HAN_TOI_DA as usize {
            bi_cat = true;
            break;
        }
        let mut canh_bao = r.canh_bao.clone();
        if tg_co_y(r.tg.ket_luan) {
            canh_bao.extend(r.tg.ghi_chu.iter().cloned());
        }
        dong.push(json!({
            "dong": r.dong,
            "hoSoSo": r.ho_so_so,
            "hopSo": r.hop_so,
            "loi": r.loi,
            "canhBao": canh_bao,
            "thoiGian": {
                "batDau": r.tg.bat_dau,
                "ketThuc": r.tg.ket_thuc,
                "ketLuan": r.tg.ket_luan,
                "ghiChu": r.tg.ghi_chu,
            },
        }));
    }
    (dong, bi_cat)
}

/// So tên phông: hạ chữ thường + nén khoảng trắng. Đúng phép so bên desktop dùng để cảnh báo
/// lệch phông (`NhapMucLucDialog.tsx:84`) — hai nơi so khác nhau là một nơi báo khớp, nơi kia báo
/// lệch, trên cùng một tệp.
fn gon_ten(s: &str) -> String {
    s.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}

// Thiếu `rename_all` ở đây là lỗi đã xảy ra HAI lần (GoiKiem, rồi GoiMau — bên gọi nhận 400
// "thiếu maNgoai" dù gói tin có maNgoai). Trình biên dịch không bắt được, vì struct vẫn hợp lệ.
// Test `moi_cua_deu_nhan_ma_ngoai_camel_case` ở cuối tệp canh cả năm cửa.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoiMau {
    #[serde(alias = "ma_ngoai", alias = "maDuAnNgoai")]
    ma_ngoai: Option<String>,
}

/// `POST /api/v1/ncc/tu-kiem/muc-luc/mau` — phát **tệp mẫu mục lục hồ sơ** + hướng dẫn điền.
///
/// Tệp mẫu do máy chủ sinh ra từ chính bảng nhận cột của bộ đọc (`digitext_mucluc::mau`), nên
/// không thể lệch khỏi thứ bộ đọc chấp nhận. Danh mục thời hạn bảo quản lấy từ **chính dự án**,
/// nên mẫu phát cho dự án nào là đúng danh mục dự án ấy.
///
/// Mẫu này KHÁC "mẫu lập hồ sơ" của desktop (`desktop/src/features/chinhly/excel.ts`): mẫu lập
/// hồ sơ đọc **theo vị trí cột** với 14 cột cố định và ba dòng đầu cố định; mục lục hồ sơ đọc
/// **theo tên cột**, phòng ban/mảng nằm trong cột Tiêu đề. Đưa nhầm mẫu lập hồ sơ cho nhà thầu
/// thì bộ đọc MLHS vẫn nhận dòng tiêu đề nhưng đẻ ra hồ sơ rác từ dòng số thứ tự và dòng kiểu
/// dữ liệu.
///
/// Trả base64 trong JSON chứ không trả thẳng byte: bên gọi dùng lại hàm ký + đọc JSON đã có cho
/// các cửa khác, không phải viết nhánh xử lý nhị phân riêng.
async fn muc_luc_mau(
    State(st): State<AppState>,
    ip: T2t,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let kh = xac_thuc(&st, &headers, &ip, &body).await?;
    let g: GoiMau = serde_json::from_slice(&body)
        .map_err(|e| ApiError::bad_request(format!("thân gói không phải JSON hợp lệ: {e}")))?;
    let ma_ngoai = g
        .ma_ngoai
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ApiError::bad_request("thiếu maNgoai — mã dự án bên hệ thống của bạn"))?;

    let du_an = digitext_db::webhooks::tim_theo_ma_ngoai(&st.pool, &kh.nguon, ma_ngoai)
        .await?
        .ok_or_else(|| ApiError {
            status: axum::http::StatusCode::NOT_FOUND,
            code: "ncc_du_an_khong_thay",
            message: format!("không có dự án nào mang maNgoai {ma_ngoai:?}"),
        })?;

    let dm = cl_ncc::thbq_cua_du_an(&st.pool, du_an.id).await?;
    let thbq: Vec<(String, bool)> = dm.iter().map(|t| (t.ten.clone(), t.vinh_vien)).collect();
    let phong_ds = cl_ncc::phong_cua_du_an(&st.pool, du_an.id).await?;
    // Điền sẵn tên phông chỉ khi dự án có đúng MỘT phông. Hai phông trở lên mà điền bừa một cái
    // thì nhà thầu nộp mục lục phông kia với tên phông này.
    let ten_phong = (phong_ds.len() == 1).then(|| phong_ds[0].as_str());

    let sheets = digitext_mucluc::mau::tep_mau(ten_phong, &thbq);
    let tep = crate::xlsx::dong_goi(&sheets).map_err(|_| ApiError::internal())?;

    use base64::Engine;
    Ok(Json(json!({
        "maNgoai": ma_ngoai,
        "maDuAn": du_an.code,
        "tenDuAn": du_an.name,
        "tenTep": format!("mau-muc-luc-ho-so-{}.xlsx", du_an.code),
        "kieu": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "noiDung": base64::engine::general_purpose::STANDARD.encode(&tep),
        "cot": digitext_mucluc::MO_TA_COT,
        "huongDan": digitext_mucluc::mau::huong_dan(),
        "thbqDanhMuc": dm.iter().map(|t| json!({ "ten": t.ten, "vinhVien": t.vinh_vien })).collect::<Vec<_>>(),
        "phongs": phong_ds,
        "gioiHan": { "mib": digitext_mucluc::mau::MIB_TOI_DA, "dong": digitext_mucluc::mau::DONG_TOI_DA },
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doi_nhan_nhieu_cach_viet() {
        for v in ["so-hoa", "Số hoá", "SO_HOA", " scan ", "số hóa"] {
            assert_eq!(chuan_doi(v), Some("so_hoa"), "{v}");
        }
        for v in ["chinh-ly", "Chỉnh lý", "chinh_ly"] {
            assert_eq!(chuan_doi(v), Some("chinh_ly"), "{v}");
        }
        assert_eq!(chuan_doi("tất cả"), Some("tat_ca"));
        assert_eq!(chuan_doi("đóng gói"), None);
        assert_eq!(chuan_doi("  "), None);
    }

    /// Nhóm chỉ số không được để lọt cột nội bộ (gói việc, đóng gói, nộp lưu).
    #[test]
    fn nhom_chi_so_khong_lot_cot_noi_bo() {
        let cam = ["work_items", "packages_built", "submissions", "fields_"];
        for doi in ["so_hoa", "chinh_ly", "tat_ca"] {
            for (cot, khoa, ten, dv) in chi_so_cua(doi) {
                assert!(
                    !cam.iter().any(|c| cot.starts_with(c)),
                    "{doi} để lọt cột {cot}"
                );
                assert!(!khoa.is_empty() && !ten.is_empty() && !dv.is_empty());
                assert!(
                    digitext_db::dashboard::METRIC_COLS.contains(&cot),
                    "{cot} không có trong dashboard_daily"
                );
            }
        }
        assert_eq!(chi_so_cua("so_hoa").len(), 7);
        assert_eq!(chi_so_cua("chinh_ly").len(), 4);
        assert_eq!(chi_so_cua("tat_ca").len(), 11);
    }

    /// Hướng dẫn phát cho nhà thầu nói trần tệp là bao nhiêu. Nói một đằng mà cửa chặn một nẻo
    /// thì nhà thầu tách tệp theo con số sai rồi vẫn bị từ chối.
    /// Dựng một phân tích thật rồi kiểm **bất biến**: số đếm bày trên thẻ và số dòng bày trong
    /// bảng phải nói cùng một chuyện.
    ///
    /// Bản cũ sai đúng chỗ này: thẻ "Cảnh báo 59" mà bảng rỗng, vì `so_canh_bao` đếm cả dòng có
    /// kết luận **thời gian** là cảnh báo, còn danh sách thì chỉ lấy dòng có `canh_bao`. Người
    /// dùng không có cách nào biết 59 cảnh báo đó là gì.
    #[test]
    fn so_dem_va_danh_sach_khong_duoc_lech_nhau() {
        use calamine::Data;
        let o = |v: &str| Data::String(v.to_string());
        let rows: Vec<Vec<Data>> = vec![
            vec![o("Hộp số"), o("Hồ sơ số"), o("Tiêu đề hồ sơ"), o("Thời gian"), o("Số tờ"), o("THBQ")],
            vec![o(""), o(""), o("I. VĂN PHÒNG"), o(""), o(""), o("")],
            vec![o(""), o(""), o("1. Công tác chung"), o(""), o(""), o("")],
            // Thời gian đọc được, sạch sẽ — không được đếm vào đâu cả.
            vec![o("1"), o("01"), o("Hồ sơ A"), o("05/01/2020 - 28/12/2020"), o("10"), o("Vĩnh viễn")],
            // Ô thời gian trống, tiêu đề có năm ⇒ máy tự suy: kết luận "đã điền", KHÔNG phải cảnh báo.
            vec![o("1"), o("02"), o("Hồ sơ B năm 2019"), o(""), o("10"), o("Vĩnh viễn")],
            // Năm ngoài dải ⇒ kết luận cảnh báo, lời giải thích nằm trong tg.ghi_chu chứ không
            // phải trong canh_bao — ĐÂY là loại dòng mà bản cũ đếm nhưng không bày.
            vec![o("1"), o("03"), o("Hồ sơ C năm 1700"), o("01/01/1700 - 31/12/1700"), o("10"), o("Vĩnh viễn")],
            // Số tờ đơn vị lạ ⇒ cảnh báo thường, nằm trong canh_bao.
            vec![o("1"), o("04"), o("Hồ sơ D năm 2020"), o("2020"), o("18 quyển"), o("Vĩnh viễn")],
            // THBQ không có trong danh mục ⇒ lỗi chặn.
            vec![o("1"), o("05"), o("Hồ sơ E năm 2020"), o("2020"), o("10"), o("Không có thật")],
        ];
        let dm = vec!["Vĩnh viễn".to_string(), "10 năm".to_string()];
        let vv = vec!["Vĩnh viễn".to_string()];
        let pt = digitext_mucluc::phan_tich::phan_tich_muc_luc(
            &rows,
            &digitext_mucluc::phan_tich::TuyChon {
                ten_tep: "t.xlsx",
                sheet: "Sheet1",
                sheets: &["Sheet1".to_string()],
                thbq_danh_muc: &dm,
                thbq_vinh_vien: &vv,
                thbq_mac_dinh: None,
            },
        );
        let (dong, bi_cat) = dong_co_van_de(&pt);
        assert!(!bi_cat);

        // Bộ dữ liệu phải thực sự chạm tới cả ba loại, nếu không test xanh mà chẳng kiểm gì.
        assert!(pt.so_loi > 0, "bộ dữ liệu thử phải có lỗi chặn");
        assert!(pt.so_canh_bao > 0, "bộ dữ liệu thử phải có cảnh báo");
        assert!(
            pt.rows.iter().any(|r| r.canh_bao.is_empty()
                && matches!(
                    r.tg.ket_luan,
                    digitext_mucluc::thoi_gian::KetLuan::CanhBao | digitext_mucluc::thoi_gian::KetLuan::Loi
                )),
            "phải có dòng chỉ cảnh báo THỜI GIAN — đúng loại bản cũ bỏ sót"
        );

        let co = |d: &Value, k: &str| d[k].as_array().map(|a| !a.is_empty()).unwrap_or(false);
        let dong_loi = dong.iter().filter(|d| co(d, "loi")).count();
        let dong_canh_bao = dong.iter().filter(|d| co(d, "canhBao")).count();

        // `so_loi` còn cộng cả lỗi CẤU TRÚC (vanDe), nên chỉ so phần lỗi theo dòng.
        let loi_theo_dong = pt.rows.iter().filter(|r| !r.loi.is_empty()).count();
        assert_eq!(dong_loi, loi_theo_dong, "số dòng có lỗi trong danh sách lệch với số đếm");
        assert_eq!(
            dong_canh_bao, pt.so_canh_bao,
            "thẻ báo {} cảnh báo nhưng bảng chỉ có {} dòng cảnh báo",
            pt.so_canh_bao, dong_canh_bao
        );

        // Mỗi dòng cảnh báo phải có CHỮ để đọc, không được là mảng rỗng.
        for d in &dong {
            if co(d, "canhBao") {
                assert!(
                    d["canhBao"].as_array().unwrap().iter().all(|v| !v.as_str().unwrap_or("").trim().is_empty()),
                    "có cảnh báo rỗng ở dòng {}",
                    d["dong"]
                );
            }
        }
    }

    /// **Cả sáu cửa phải nhận `maNgoai`** — tên khoá tài liệu công bố, và tên bên gọi dùng.
    ///
    /// Quên `#[serde(rename_all = "camelCase")]` trên một struct gói tin là lỗi đã xảy ra HAI
    /// lần (`GoiKiem`, rồi `GoiMau` ngày 13/11/2026). Nó không làm hỏng biên dịch, không làm
    /// hỏng test nào khác, và hiện ra ở đầu kia dưới dạng **400 "thiếu maNgoai" dù gói tin có
    /// maNgoai** — bên gọi đi tìm lỗi trong mã của họ. Test này đóng lớp lỗi đó lại.
    #[test]
    fn moi_cua_deu_nhan_ma_ngoai_camel_case() {
        let j = r#"{"maNgoai":"PRJ-26-017"}"#;
        let mong = Some("PRJ-26-017");
        assert_eq!(serde_json::from_str::<GoiHoi>(j).unwrap().ma_ngoai.as_deref(), mong, "ncc/san-luong");
        assert_eq!(serde_json::from_str::<GoiKiem>(j).unwrap().ma_ngoai.as_deref(), mong, "ncc/tu-kiem/da-nop");
        assert_eq!(serde_json::from_str::<GoiMucLuc>(j).unwrap().ma_ngoai.as_deref(), mong, "ncc/tu-kiem/muc-luc");
        assert_eq!(serde_json::from_str::<GoiMau>(j).unwrap().ma_ngoai.as_deref(), mong, "ncc/tu-kiem/muc-luc/mau");
        assert_eq!(serde_json::from_str::<GoiPdf>(j).unwrap().ma_ngoai.as_deref(), mong, "ncc/tu-kiem/pdf");
        assert_eq!(serde_json::from_str::<GoiGoiTin>(j).unwrap().ma_ngoai.as_deref(), mong, "ncc/tu-kiem/goi");
    }

    /// Các khoá nhiều chữ khác cũng phải nhận dạng camelCase, không riêng `maNgoai` — cùng một
    /// thuộc tính `rename_all` chi phối cả nhóm, nên kiểm thêm một khoá mỗi cửa là đủ.
    #[test]
    fn khoa_nhieu_chu_khac_cung_nhan_camel_case() {
        let g: GoiHoi = serde_json::from_str(r#"{"maNgoai":"X","tuNgay":"2026-10-01"}"#).unwrap();
        assert_eq!(g.tu_ngay.as_deref(), Some("2026-10-01"));
        let g: GoiMucLuc = serde_json::from_str(r#"{"maNgoai":"X","tenTep":"a.xlsx","thbqMacDinh":"Vĩnh viễn"}"#).unwrap();
        assert_eq!(g.ten_tep.as_deref(), Some("a.xlsx"));
        assert_eq!(g.thbq_mac_dinh.as_deref(), Some("Vĩnh viễn"));
        let g: GoiPdf = serde_json::from_str(r#"{"maNgoai":"X","tenTep":"a.pdf"}"#).unwrap();
        assert_eq!(g.ten_tep.as_deref(), Some("a.pdf"));
        let g: GoiGoiTin = serde_json::from_str(
            r#"{"maNgoai":"X","tenGoi":"AIP.zip","coGoi":131723288,"tep":[{"duongDan":"a/METS.xml","noiDung":"PHgvPg=="}]}"#,
        )
        .unwrap();
        assert_eq!(g.ten_goi.as_deref(), Some("AIP.zip"));
        assert_eq!(g.co_goi, Some(131_723_288));
        assert_eq!(g.tep[0].duong_dan.as_deref(), Some("a/METS.xml"));
        assert!(g.tep[0].noi_dung.is_some());
    }

    #[test]
    fn tran_trong_huong_dan_khop_tran_that_cua_cua() {
        assert_eq!(digitext_mucluc::mau::MIB_TOI_DA, MUC_LUC_BYTE_TOI_DA / 1024 / 1024);
        assert_eq!(digitext_mucluc::mau::DONG_TOI_DA, MUC_LUC_DONG_TOI_DA);
    }

    /// Phép so tên phông phải bỏ qua hoa thường và khoảng trắng thừa — đúng như bên desktop,
    /// nơi cùng phép so này quyết định có hiện cảnh báo lệch phông hay không.
    #[test]
    fn so_ten_phong_bo_qua_hoa_thuong_va_khoang_trang() {
        assert_eq!(gon_ten("  Sở  Nông   nghiệp "), gon_ten("sở nông nghiệp"));
        assert_eq!(gon_ten("UBND tỉnh Bắc Giang"), gon_ten("ubnd  tỉnh bắc giang"));
        assert_ne!(gon_ten("Phông A"), gon_ten("Phông B"));
        // Dấu tiếng Việt KHÔNG bị bỏ: "Phong" và "Phông" là hai phông khác nhau.
        assert_ne!(gon_ten("Phong Nha"), gon_ten("Phông Nha"));
    }

    /// Tệp mẫu phải đóng gói được và chính bộ đọc phải nhập lại sạch — phép thử trọn vòng của
    /// cửa `muc-luc/mau`, không cần CSDL.
    #[test]
    fn tep_mau_dung_goi_duoc_va_doc_lai_sach() {
        let dm = vec![("Vĩnh viễn".to_string(), true), ("20 năm".to_string(), false)];
        let tep = crate::xlsx::dong_goi(&digitext_mucluc::mau::tep_mau(Some("Phông thử"), &dm)).unwrap();
        let so = digitext_mucluc::doc_so_tu_bytes(&tep, None).unwrap();
        let ten: Vec<String> = dm.iter().map(|(t, _)| t.clone()).collect();
        let vv: Vec<String> = dm.iter().filter(|(_, v)| *v).map(|(t, _)| t.clone()).collect();
        let pt = digitext_mucluc::phan_tich::phan_tich_muc_luc(
            &so.rows,
            &digitext_mucluc::phan_tich::TuyChon {
                ten_tep: "mau.xlsx",
                sheet: &so.sheet,
                sheets: &so.sheets,
                thbq_danh_muc: &ten,
                thbq_vinh_vien: &vv,
                thbq_mac_dinh: None,
            },
        );
        assert!(!digitext_mucluc::phan_tich::co_loi_chan(&pt));
        // Tên phông điền sẵn phải khớp chính nó qua phép so của cửa.
        assert_eq!(gon_ten(&pt.ten_phong_trong_tep), gon_ten("Phông thử"));
    }

    #[test]
    fn khoa_json_khong_trung_nhau() {
        let mut k: Vec<&str> = chi_so_cua("tat_ca").iter().map(|c| c.1).collect();
        k.sort_unstable();
        let n = k.len();
        k.dedup();
        assert_eq!(k.len(), n, "có khoá JSON trùng nhau");
    }

    #[test]
    fn khoang_ngay_mac_dinh_va_kep() {
        let d = |s: &str| NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap();
        // mặc định: 30 ngày tính cả hai đầu
        let (tu, den) = khoang_ngay(&GoiHoi {
            ma_ngoai: None,
            doi: None,
            tu_ngay: None,
            den_ngay: Some("2026-11-10".into()),
        })
        .unwrap();
        assert_eq!(den, d("2026-11-10"));
        assert_eq!(tu, d("2026-10-12"));
        assert_eq!((den - tu).num_days() + 1, 30);

        assert!(kep_khoang(d("2026-11-11"), d("2026-11-10")).is_err());
        assert!(kep_khoang(d("2025-01-01"), d("2026-11-10")).is_err());
        assert!(kep_khoang(d("2026-11-10"), d("2026-11-10")).is_ok());
    }

    #[test]
    fn gioi_han_bi_kep_chu_khong_bao_loi() {
        assert_eq!(kep_gioi_han(None), GIOI_HAN_MAC_DINH);
        assert_eq!(kep_gioi_han(Some(50)), 50);
        assert_eq!(kep_gioi_han(Some(999_999)), GIOI_HAN_TOI_DA);
        assert_eq!(kep_gioi_han(Some(0)), 1);
        assert_eq!(kep_gioi_han(Some(-7)), 1);
    }

    /// Mã loại lỗi là khoá bên DO365 dùng để dịch nhãn — trùng mã là họ gộp sai hai loại.
    #[test]
    fn ma_loai_loi_khong_trung_nhau() {
        let mut ma: Vec<&str> = docs::NCC_LOAI_LOI.iter().map(|(m, _)| *m).collect();
        let truoc = ma.len();
        ma.sort_unstable();
        ma.dedup();
        assert_eq!(ma.len(), truoc, "mã loại lỗi bị trùng: {ma:?}");
        assert!(ma.iter().all(|m| !m.is_empty()));
    }

    /// Chặn hồi quy: ghi chú người duyệt phải TẮT khi không ai đặt biến môi trường.
    #[test]
    fn ly_do_mac_dinh_tat() {
        if std::env::var("DIGITEXT_NCC_LY_DO").is_err() {
            assert!(!tra_ly_do());
        }
    }
}

// ------------------------------------------------- tự kiểm: một tệp PDF lẻ (chưa nộp)

/// Trần một tệp PDF. Một tài liệu hành chính 300 dpi vài chục trang hiếm khi quá chừng này; để
/// rộng hơn thì base64 trong JSON thành gánh nặng cho cả hai bên.
/// Trần của **bản kê gói**. Gói AIP thật đo ngày 13/11/2026: 131 MB nội dung / 326 KB siêu dữ
/// liệu — tỉ lệ 0,247%. Nên 12 MiB siêu dữ liệu đủ cho gói cỡ vài GB, còn 20.000 mục đủ cho
/// gói nhiều hồ sơ. Vượt thì tách gói, đừng nới trần.
const GOI_SIEU_DU_LIEU_TOI_DA: usize = 12 * 1024 * 1024;
const GOI_MUC_TOI_DA: usize = 20_000;

const PDF_BYTE_TOI_DA: usize = 32 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoiGoiTin {
    #[serde(alias = "ma_ngoai", alias = "maDuAnNgoai")]
    ma_ngoai: Option<String>,
    /// Tên tệp gói, chỉ để hiện lại.
    #[serde(alias = "ten_goi", alias = "tenTep")]
    ten_goi: Option<String>,
    /// Cỡ gói thật, byte — chỉ để hiện lại, máy chủ không kiểm.
    #[serde(alias = "co_goi")]
    co_goi: Option<u64>,
    #[serde(default)]
    tep: Vec<MucGoi>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MucGoi {
    #[serde(alias = "duong_dan", alias = "d")]
    duong_dan: Option<String>,
    /// Cỡ tệp, byte. Tuỳ chọn.
    #[serde(alias = "c")]
    co: Option<u64>,
    /// Nội dung base64 — **chỉ gửi cho tệp siêu dữ liệu** (.xml/.xsd). Tệp nội dung để trống.
    #[serde(alias = "noi_dung")]
    noi_dung: Option<String>,
}

/// `POST /api/v1/ncc/tu-kiem/goi` — nhà thầu tự kiểm **cấu trúc gói tin** trước khi giao.
///
/// Không tải cả gói lên. Cổng đọc mục lục của tệp `.zip` ngay trên máy nhà thầu, gửi lên **bản
/// kê**: đường dẫn của mọi tệp, kèm nội dung của riêng các tệp siêu dữ liệu. Đo trên gói AIP
/// thật: 131 MB nội dung / 326 KB siêu dữ liệu — **0,247%**. Gói 1 GB gửi lên chừng 2,5 MB.
///
/// Chạy đủ phần kiểm cấu trúc: METS, PREMIS, EAD, TT02, mã cơ quan, định dạng ngày, `href` có
/// trỏ đúng tệp có thật không, tệp mồ côi, đối chiếu EAD ↔ METS.
///
/// **Ba phép kiểm cần byte thật thì KHÔNG chạy** — PDF/A từng tệp, mã băm METS, mã băm PREMIS.
/// Kết quả nói thẳng số phép kiểm đã bỏ, và kết luận khi ấy là `CHƯA KIỂM ĐỦ`, **không bao giờ**
/// là `ĐẠT` trơn. Tệp PDF lẻ thì dùng cửa `tu-kiem/pdf`.
async fn tu_kiem_goi(
    State(st): State<AppState>,
    ip: T2t,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let kh = xac_thuc(&st, &headers, &ip, &body).await?;
    let g: GoiGoiTin = serde_json::from_slice(&body)
        .map_err(|e| ApiError::bad_request(format!("thân gói không phải JSON hợp lệ: {e}")))?;

    let ma_ngoai = g
        .ma_ngoai
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ApiError::bad_request("thiếu maNgoai — mã dự án bên hệ thống của bạn"))?;
    if g.tep.is_empty() {
        return Err(ApiError::bad_request("thiếu tep[] — bản kê các tệp trong gói"));
    }
    if g.tep.len() > GOI_MUC_TOI_DA {
        return Err(ApiError::bad_request(format!(
            "bản kê có {} mục, vượt trần {GOI_MUC_TOI_DA} — tách gói nhỏ hơn",
            g.tep.len()
        )));
    }

    let du_an = digitext_db::webhooks::tim_theo_ma_ngoai(&st.pool, &kh.nguon, ma_ngoai)
        .await?
        .ok_or_else(|| ApiError {
            status: axum::http::StatusCode::NOT_FOUND,
            code: "ncc_du_an_khong_thay",
            message: format!("không có dự án nào mang maNgoai {ma_ngoai:?}"),
        })?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD;
    let mut tong_noi_dung = 0usize;
    let mut tep: Vec<(String, Option<Vec<u8>>, Option<u64>)> = Vec::with_capacity(g.tep.len());
    for (i, m) in g.tep.iter().enumerate() {
        let d = m
            .duong_dan
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ApiError::bad_request(format!("tep[{i}] thiếu duongDan")))?;
        // Đường dẫn là dữ liệu của người ngoài và được dùng để tra bảng — chặn mọi lối đi lên
        // thư mục cha, kể cả khi nó chỉ nằm trong bộ nhớ.
        if d.starts_with('/') || d.contains("..") || d.contains('\\') {
            return Err(ApiError::bad_request(format!(
                "tep[{i}] duongDan {d:?} không hợp lệ — dùng dấu / và đường dẫn tương đối trong gói"
            )));
        }
        let noi = match m.noi_dung.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            None => None,
            Some(v) => {
                let b = b64
                    .decode(v)
                    .map_err(|_| ApiError::bad_request(format!("tep[{i}] noiDung không phải base64 hợp lệ")))?;
                tong_noi_dung += b.len();
                if tong_noi_dung > GOI_SIEU_DU_LIEU_TOI_DA {
                    return Err(ApiError::bad_request(format!(
                        "tổng siêu dữ liệu vượt trần {} MiB — chỉ gửi nội dung của tệp .xml/.xsd",
                        GOI_SIEU_DU_LIEU_TOI_DA / 1024 / 1024
                    )));
                }
                Some(b)
            }
        };
        tep.push((d.to_owned(), noi, m.co));
    }

    let co_sieu = tep.iter().filter(|(_, b, _)| b.is_some()).count();
    if co_sieu == 0 {
        return Err(ApiError::bad_request(
            "không mục nào có noiDung — phải gửi kèm nội dung của các tệp .xml trong gói (METS, PREMIS, EAD…)",
        ));
    }

    let ten_goi = g.ten_goi.as_deref().unwrap_or("goi.zip").trim().to_owned();
    let co_goi = g.co_goi.unwrap_or(0);
    let so_muc = tep.len();
    // Phân tích XML của cả gói là việc nặng CPU ⇒ sang luồng chặn, cùng lý do với cửa mục lục.
    let kq = tokio::task::spawn_blocking(move || {
        let goc = digitext_pkgcheck::PackageResult {
            source: String::new(), // KHÔNG đưa đường dẫn máy chủ vào kết quả của người ngoài
            name: ten_goi,
            kind: "?".into(),
            ..Default::default()
        };
        digitext_pkgcheck::check_loaded(goc, digitext_pkgcheck::tu_ban_ke(tep, co_goi)).0
    })
    .await
    .map_err(|_| ApiError::internal())?;

    let muc = |l: &digitext_pkgcheck::Level| match l {
        digitext_pkgcheck::Level::Err => "loi",
        digitext_pkgcheck::Level::Warn => "canh_bao",
        digitext_pkgcheck::Level::Ok => "dat",
    };
    Ok(Json(json!({
        "maNgoai": ma_ngoai,
        "maDuAn": du_an.code,
        "tenDuAn": du_an.name,
        "tenGoi": kq.name,
        "soMuc": so_muc,
        "soTepSieuDuLieu": co_sieu,
        "coGoi": co_goi,
        "loai": kq.kind,
        "objid": kq.objid,
        "profile": kq.profile,
        "coQuanTao": kq.creator_org,
        "phanMemTao": kq.creator_software,
        "ngayTao": kq.created,
        "maPhong": kq.file_code,
        "tieuDe": kq.title,
        "soTaiLieu": kq.total_doc,
        "soTrang": kq.pages,
        "soTepDuLieu": kq.data_files,
        "ketLuan": kq.verdict,
        "soLoi": kq.errors,
        "soCanhBao": kq.warnings,
        "chuaKiemNoiDung": kq.chua_kiem_noi_dung,
        "phatHien": kq.findings.iter().map(|f| json!({
            "muc": muc(&f.level),
            "nhom": f.group,
            "noiDung": f.message,
        })).collect::<Vec<_>>(),
        "ghiChu": "Cửa này kiểm CẤU TRÚC gói từ siêu dữ liệu. Không kiểm nội dung từng tệp PDF, không đối chiếu mã băm — dùng «Tự kiểm PDF/A» cho tệp mẫu. Bản kiểm đầy đủ chạy khi nộp gói thật.",
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoiPdf {
    #[serde(alias = "ma_ngoai", alias = "maDuAnNgoai")]
    ma_ngoai: Option<String>,
    #[serde(alias = "ten_tep")]
    ten_tep: Option<String>,
    /// Nội dung tệp PDF, mã hoá base64.
    #[serde(alias = "noi_dung", alias = "tep")]
    noi_dung: Option<String>,
    /// Tên loại tài liệu (nếu biết) — dùng để xếp bản đồ/bản vẽ, đổi ngưỡng dpi 200 → 300.
    #[serde(alias = "loai_tai_lieu")]
    loai_tai_lieu: Option<String>,
    /// Trích yếu (nếu biết) — chỉ sinh dấu hiệu NGHI, không đổi ngưỡng.
    #[serde(alias = "trich_yeu")]
    trich_yeu: Option<String>,
}

/// `POST /api/v1/ncc/tu-kiem/pdf` — nhà thầu tự kiểm **một tệp PDF** trước khi làm cả lô.
///
/// Trả lời: *"tệp PDF tôi làm ra có đạt Điều 8 khoản 2 điểm a TT 05/2025 không?"* Giá trị thật là
/// kiểm **một tệp mẫu** trước khi sản xuất mấy chục nghìn tệp cùng cách — sai cách ghép PDF/A thì
/// cả lô bị trả lại.
///
/// Phép kiểm là **chính hàm** `digitext_extimport::check::PdfFacts::tt05_failures` mà DigiText dùng
/// ở Tiện ích › Kiểm PDF/A và ở đường nhập gói, nên không có luật riêng cho cửa này. Cột
/// `documents.pdfa_2b` mà máy chủ ghi lúc tải lên cũng được trả ở đây (`datPdfa2b`) — đạt ở đây thì
/// tệp sẽ không bị đánh dấu `pdfa_cu` trong «Tự kiểm chất lượng».
///
/// ⚠️ **KHÔNG chạy veraPDF** (kết luận hợp chuẩn ISO 19005 đầy đủ): veraPDF cần Docker + ảnh
/// `verapdf/cli` trên máy chủ. Ở đây là kiểm **dấu hiệu cấu trúc** — đủ để bắt mọi lỗi hay gặp
/// (không khai pdfaid, thiếu hồ sơ màu, thiếu lớp chữ, dpi thấp, màu dưới 24 bit), nhưng không
/// thay được veraPDF.
async fn tu_kiem_pdf(
    State(st): State<AppState>,
    ip: T2t,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let kh = xac_thuc(&st, &headers, &ip, &body).await?;
    let g: GoiPdf = serde_json::from_slice(&body)
        .map_err(|e| ApiError::bad_request(format!("thân gói không phải JSON hợp lệ: {e}")))?;

    let ma_ngoai = g
        .ma_ngoai
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ApiError::bad_request("thiếu maNgoai — mã dự án bên hệ thống của bạn"))?;
    let b64 = g
        .noi_dung
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ApiError::bad_request("thiếu noiDung — nội dung tệp PDF mã hoá base64"))?;

    let tep: Vec<u8> = {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|_| ApiError::bad_request("noiDung không phải base64 hợp lệ"))?
    };
    if tep.is_empty() {
        return Err(ApiError::bad_request("tệp rỗng"));
    }
    if tep.len() > PDF_BYTE_TOI_DA {
        return Err(ApiError::bad_request(format!(
            "tệp {} MiB vượt trần {} MiB",
            tep.len() / 1024 / 1024,
            PDF_BYTE_TOI_DA / 1024 / 1024
        )));
    }
    if !tep.starts_with(b"%PDF-") {
        return Err(ApiError::bad_request("không phải tệp PDF (thiếu chữ ký %PDF- ở đầu tệp)"));
    }

    // Phạm vi vẫn do khoá quyết định — cửa này không đọc dữ liệu dự án, nhưng vẫn buộc khai
    // `maNgoai` để nhật ký và hạn mức về sau gắn được vào đúng dự án.
    let _du_an = digitext_db::webhooks::tim_theo_ma_ngoai(&st.pool, &kh.nguon, ma_ngoai)
        .await?
        .ok_or_else(|| ApiError {
            status: axum::http::StatusCode::NOT_FOUND,
            code: "ncc_du_an_khong_thay",
            message: format!("không có dự án nào mang maNgoai {ma_ngoai:?}"),
        })?;

    let so_byte = tep.len();
    let loai_tl = g.loai_tai_lieu.clone();
    let trich_yeu = g.trich_yeu.clone();
    // Soi PDF là việc nặng CPU (đọc mọi trang, mọi ảnh) ⇒ luồng chặn.
    let (facts, pdfa2b, xep) = tokio::task::spawn_blocking(move || {
        use digitext_extimport::check;
        let f = check::inspect_pdf(&tep);
        let pdfa2b = digitext_pdf::pdfa::f_1rh(&tep).is_pdfa2b();
        let xep = check::classify_map(loai_tl.as_deref(), trich_yeu.as_deref(), f.max_page_cm);
        (f, pdfa2b, xep)
    })
    .await
    .map_err(|_| ApiError::internal())?;

    use digitext_extimport::check::MapClass;
    let la_ban_ve = xep.is_map();
    let li_do_xep = match &xep {
        MapClass::Sure(s) | MapClass::Suspect(s) => Some(s.clone()),
        MapClass::No => None,
    };

    let mut chua_dat: Vec<Value> = Vec::new();
    let mut chua_ky_so = false;
    for (ma, noi_dung) in facts.tt05_failures(la_ban_ve) {
        if ma == "PDF-CHUA-KY-SO" {
            // Trong dây chuyền DigiText, ký số làm SAU khi nhà thầu giao sản phẩm — báo đây là
            // "chưa đạt" thì đúng luật nhưng là nhiễu với họ. Tách ra thành cờ riêng.
            chua_ky_so = true;
            continue;
        }
        chua_dat.push(json!({ "ma": ma, "noiDung": noi_dung }));
    }

    Ok(Json(json!({
        "maNgoai": ma_ngoai,
        "tenTep": g.ten_tep.as_deref().unwrap_or("").trim(),
        "soByte": so_byte,
        "soTrang": facts.pages,
        "soTrangCoLopChu": facts.text_pages,
        "dpiNhoNhat": facts.min_dpi,
        "doSauBitNhoNhat": facts.min_bit_depth,
        "tuKhaiPdfa": facts.declared,
        "coHoSoMau": facts.has_output_intent,
        "laPdfa": facts.pdfa,
        "daKySo": facts.signed,
        "canhDaiNhatCm": facts.max_page_cm,
        "laBanVe": la_ban_ve,
        "liDoXepLoai": li_do_xep,
        "dpiToiThieuCanCo": if la_ban_ve { digitext_extimport::check::MAP_MIN_DPI } else { digitext_pdf::FM1_MIN_DPI },
        "datPdfa2b": pdfa2b,
        "chuaDat": chua_dat,
        "datChuan": chua_dat.is_empty(),
        "chuaKySo": chua_ky_so,
        "tepLoi": facts.encrypted_or_broken,
        "ghiChu": "Kiểm dấu hiệu cấu trúc theo TT 05/2025 Điều 8.2.a. KHÔNG chạy veraPDF nên chưa phải kết luận hợp chuẩn ISO 19005 đầy đủ.",
    })))
}
