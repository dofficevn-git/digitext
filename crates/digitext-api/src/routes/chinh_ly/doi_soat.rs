// Copyright (c) 2026 DigiText. All rights reserved.
//! Kiểm tra sản phẩm số hoá theo phông (khoản 5 Điều 9 TT 05/2025, 3/11/2026): đối soát mục lục hồ sơ chỉnh lý với tài
//! liệu đã lên máy chủ — mỗi hồ sơ: số tờ kê khai ↔ số trang đã quét, số tệp, PDF/A-2b, ký số, bị từ chối; tài liệu
//! không ghép được hồ sơ. Chỉ ĐỌC. Ghép theo (hộp, hồ sơ số) đúng như `tu_ocr.rs` (`khoa`); `thuMuc` = tên thư mục
//! phông trên máy chủ (tầng 1 của rel_path) để chỉ xét tài liệu trong thư mục đó khi dự án có nhiều phông.
//!
//! `GET /cl/doi-soat?projectId&phongId[&thuMuc]` → `DoiSoatResp`.

use super::project_access;
use super::tu_ocr::BoGhep;
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use digitext_core::Permission;
use digitext_db::chinh_ly as cl;
use digitext_db::documents;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/cl/doi-soat", get(doi_soat))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Q {
    pub project_id: Uuid,
    pub phong_id: Uuid,
    /// Thư mục phông trên máy chủ (tầng 1 rel_path); trống = mọi tài liệu của dự án.
    #[serde(default)]
    pub thu_muc: Option<String>,
}

/// Kết luận một hồ sơ, theo thứ tự ưu tiên khi nhiều lỗi cùng lúc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KetLuan {
    ChuaQuet,
    ThieuTrang,
    ThuaTrang,
    KhongPdfa,
    PdfaCu,
    BiTuChoi,
    Dat,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoSoDoiSoat {
    pub ho_so_id: Uuid,
    pub hop_so: String,
    pub ho_so_so: String,
    pub ma_ho_so: Option<String>,
    pub tieu_de: String,
    pub vinh_vien: bool,
    /// Số tờ kê khai trong mục lục (None = chưa kê / không phải số).
    pub so_to: Option<i64>,
    pub files: i64,
    pub pages: i64,
    pub pdfa_docs: i64,
    pub pdfa_cu: i64,
    pub signed_docs: i64,
    pub rejected_docs: i64,
    pub ket_luan: KetLuan,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KhongKhop {
    pub document_id: Uuid,
    pub filename: String,
    pub hop: String,
    pub ho_so: String,
    pub ly_do: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TongKet {
    pub ho_so_muc_luc: usize,
    pub ho_so_co_tep: usize,
    pub dat: usize,
    pub chua_quet: usize,
    pub lech_so_to: usize,
    pub khong_pdfa: usize,
    pub pdfa_cu: usize,
    pub bi_tu_choi: usize,
    pub tep: i64,
    pub trang: i64,
    pub so_to_muc_luc: i64,
    pub tep_khong_khop: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoiSoatResp {
    pub tong_ket: TongKet,
    pub ho_so: Vec<HoSoDoiSoat>,
    pub khong_khop: Vec<KhongKhop>,
    /// Các thư mục phông đang có tài liệu trong dự án (để màn hình chọn `thuMuc`).
    pub thu_muc_co: Vec<String>,
}

/// "215" / "215 tờ" / " 0215" → 215; không có chữ số → None.
fn so_to_tu_chuoi(s: Option<&str>) -> Option<i64> {
    let d: String = s?.chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
    d.parse::<i64>().ok().filter(|n| *n > 0)
}

pub fn ket_luan(so_to: Option<i64>, files: i64, pages: i64, pdfa_docs: i64, pdfa_cu: i64, rejected: i64) -> KetLuan {
    if files == 0 {
        return KetLuan::ChuaQuet;
    }
    if let Some(t) = so_to {
        if pages < t {
            return KetLuan::ThieuTrang;
        }
        if pages > t * 2 {
            return KetLuan::ThuaTrang;
        }
    }
    if pdfa_docs < files {
        return KetLuan::KhongPdfa;
    }
    if pdfa_cu > 0 {
        return KetLuan::PdfaCu;
    }
    if rejected > 0 {
        return KetLuan::BiTuChoi;
    }
    KetLuan::Dat
}

async fn doi_soat(State(st): State<AppState>, ts: Ts, Query(q): Query<Q>) -> Result<Json<DoiSoatResp>, ApiError> {
    project_access(&st, &ts, q.project_id).await?;
    if !ts.has_perm(Permission::DossierRead) || !ts.has_perm(Permission::DocumentList) {
        return Err(ApiError::forbidden());
    }
    Ok(Json(tinh(&st.pool, q.project_id, q.phong_id, q.thu_muc.as_deref()).await?))
}

/// Phần tính (không kiểm quyền): đọc hai bảng rồi ghép.
pub async fn tinh(pool: &sqlx::PgPool, project_id: Uuid, phong_id: Uuid, thu_muc: Option<&str>) -> Result<DoiSoatResp, ApiError> {
    let ho_sos = cl::ho_so::doi_soat_ho_so_cua_phong(pool, project_id, phong_id).await?;
    let docs = documents::list_for_project_qc(pool, project_id).await?;
    Ok(ghep(&ho_sos, &docs, thu_muc))
}

/// Ghép mục lục với tài liệu máy chủ — hàm thuần, chạy thử được trên dữ liệu xuất ra (test `chay_that_json`).
pub fn ghep(ho_sos: &[cl::ho_so::HoSoDoiSoatRow], docs: &[documents::DocQcRow], thu_muc: Option<&str>) -> DoiSoatResp {
    let bo_ghep = BoGhep::new(ho_sos.iter().map(|h| (h.hop_so.as_deref(), h.ho_so_so.as_deref().filter(|s| !s.trim().is_empty()).unwrap_or(h.so_tam.as_str()))));
    let mut thu_muc_co: Vec<String> = Vec::new();
    for d in docs {
        if !d.phong.is_empty() && !thu_muc_co.iter().any(|t| t == &d.phong) {
            thu_muc_co.push(d.phong.clone());
        }
    }
    thu_muc_co.sort();
    let loc = thu_muc.map(str::trim).filter(|s| !s.is_empty()).map(str::to_lowercase);

    #[derive(Default)]
    struct Gop {
        files: i64,
        pages: i64,
        pdfa: i64,
        pdfa_cu: i64,
        signed: i64,
        rejected: i64,
    }
    let mut gop: HashMap<usize, Gop> = HashMap::new();
    let mut khong_khop: Vec<KhongKhop> = Vec::new();
    for d in docs {
        if d.status == "disposed" {
            continue;
        }
        if let Some(l) = &loc {
            if d.phong.to_lowercase() != *l {
                continue;
            }
        }
        match bo_ghep.tim(&d.hop, &d.ho_so) {
            [i] => {
                let g = gop.entry(*i).or_default();
                g.files += 1;
                g.pages += i64::from(d.page_count.unwrap_or(0));
                if d.pdfa_pages.unwrap_or(0) > 0 {
                    g.pdfa += 1;
                    if d.pdfa_2b == Some(false) {
                        g.pdfa_cu += 1;
                    }
                }
                if d.status == "signed" || d.status == "archived" {
                    g.signed += 1;
                }
                if d.review_status.as_deref() == Some("rejected") {
                    g.rejected += 1;
                }
            }
            [_, _, ..] => khong_khop.push(KhongKhop { document_id: d.id, filename: d.filename.clone(), hop: d.hop.clone(), ho_so: d.ho_so.clone(), ly_do: "hộp/hồ sơ trùng giữa hồ sơ vĩnh viễn và có thời hạn".into() }),
            [] => khong_khop.push(KhongKhop { document_id: d.id, filename: d.filename.clone(), hop: d.hop.clone(), ho_so: d.ho_so.clone(), ly_do: "không có hồ sơ này trong mục lục".into() }),
        }
    }
    // Đếm trước khi cắt danh sách (4/11/2026: trước đây tổng kết báo đúng 2.000 khi phông có nhiều tệp lệch hơn).
    let so_khong_khop = khong_khop.len();
    if khong_khop.len() > 2000 {
        khong_khop.truncate(2000);
    }

    let mut tk = TongKet { ho_so_muc_luc: ho_sos.len(), tep_khong_khop: so_khong_khop, ..Default::default() };
    let mut out = Vec::with_capacity(ho_sos.len());
    for (i, h) in ho_sos.iter().enumerate() {
        let g = gop.remove(&i).unwrap_or_default();
        let so_to = so_to_tu_chuoi(h.so_to.as_deref());
        let kl = ket_luan(so_to, g.files, g.pages, g.pdfa, g.pdfa_cu, g.rejected);
        if g.files > 0 {
            tk.ho_so_co_tep += 1;
        }
        match kl {
            KetLuan::Dat => tk.dat += 1,
            KetLuan::ChuaQuet => tk.chua_quet += 1,
            KetLuan::ThieuTrang | KetLuan::ThuaTrang => tk.lech_so_to += 1,
            KetLuan::KhongPdfa => tk.khong_pdfa += 1,
            KetLuan::PdfaCu => tk.pdfa_cu += 1,
            KetLuan::BiTuChoi => tk.bi_tu_choi += 1,
        }
        tk.tep += g.files;
        tk.trang += g.pages;
        tk.so_to_muc_luc += so_to.unwrap_or(0);
        out.push(HoSoDoiSoat {
            ho_so_id: h.id,
            hop_so: h.hop_so.clone().unwrap_or_default(),
            ho_so_so: h.ho_so_so.clone().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| h.so_tam.clone()),
            ma_ho_so: h.ma_ho_so.clone(),
            tieu_de: h.tieu_de.clone(),
            vinh_vien: h.vinh_vien,
            so_to,
            files: g.files,
            pages: g.pages,
            pdfa_docs: g.pdfa,
            pdfa_cu: g.pdfa_cu,
            signed_docs: g.signed,
            rejected_docs: g.rejected,
            ket_luan: kl,
        });
    }
    DoiSoatResp { tong_ket: tk, ho_so: out, khong_khop, thu_muc_co }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ket_luan_theo_uu_tien() {
        assert_eq!(ket_luan(Some(10), 0, 0, 0, 0, 0), KetLuan::ChuaQuet);
        assert_eq!(ket_luan(Some(10), 2, 8, 2, 0, 0), KetLuan::ThieuTrang);
        assert_eq!(ket_luan(Some(10), 2, 25, 2, 0, 0), KetLuan::ThuaTrang);
        assert_eq!(ket_luan(Some(10), 2, 12, 1, 0, 0), KetLuan::KhongPdfa);
        assert_eq!(ket_luan(Some(10), 2, 12, 2, 1, 0), KetLuan::PdfaCu);
        assert_eq!(ket_luan(None, 2, 12, 2, 0, 1), KetLuan::BiTuChoi);
        assert_eq!(ket_luan(Some(10), 2, 12, 2, 0, 0), KetLuan::Dat);
        assert_eq!(so_to_tu_chuoi(Some(" 0215 tờ")), Some(215));
        assert_eq!(so_to_tu_chuoi(Some("-")), None);
    }

    /// Chạy thử trên dữ liệu thật đã xuất (chỉ đọc, 4/11/2026):
    ///   DIGITEXT_THU_DOCS = JSON các dòng `list_for_project_qc` (psql json_agg trên máy chủ)
    ///   DIGITEXT_THU_CAP  = JSON `[{"hoSo": "<tệp JSON hồ sơ>", "thuMuc": "<thư mục phông>"}]` (hồ sơ từ kiem:mucluc KIEM_XUAT_JSON)
    ///   cargo test -p digitext-api --lib chay_that_json -- --ignored --nocapture
    #[test]
    #[ignore]
    fn chay_that_json() {
        use serde_json::Value;
        let doc_txt = std::fs::read_to_string(std::env::var("DIGITEXT_THU_DOCS").expect("DOCS")).unwrap();
        let docs_v: Vec<Value> = serde_json::from_str(doc_txt.trim_start_matches('\u{feff}')).unwrap();
        let s = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        let i = |v: &Value, k: &str| v.get(k).and_then(Value::as_i64).map(|n| n as i32);
        let docs: Vec<documents::DocQcRow> = docs_v
            .iter()
            .map(|v| documents::DocQcRow {
                id: s(v, "id").parse().unwrap_or(Uuid::nil()),
                filename: s(v, "filename"),
                phong: s(v, "phong"),
                hop: s(v, "hop"),
                ho_so: s(v, "ho_so"),
                page_count: i(v, "page_count"),
                status: s(v, "status"),
                pdfa_pages: i(v, "pdfa_pages"),
                pdfa_2b: v.get("pdfa_2b").and_then(Value::as_bool),
                review_status: v.get("review_status").and_then(Value::as_str).map(str::to_string),
            })
            .collect();
        let cap: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(std::env::var("DIGITEXT_THU_CAP").expect("CAP")).unwrap()).unwrap();
        println!("TAI_LIEU={}", docs.len());
        for c in &cap {
            let tep = s(c, "hoSo");
            let thu_muc = s(c, "thuMuc");
            let hs_v: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(&tep).unwrap()).unwrap();
            let ho_sos: Vec<cl::ho_so::HoSoDoiSoatRow> = hs_v
                .iter()
                .enumerate()
                .map(|(k, v)| cl::ho_so::HoSoDoiSoatRow {
                    id: Uuid::from_u128(k as u128 + 1),
                    hop_so: v.get("hop_so").and_then(Value::as_str).map(str::to_string),
                    ho_so_so: v.get("ho_so_so").and_then(Value::as_str).map(str::to_string),
                    so_tam: s(v, "so_tam"),
                    vinh_vien: v.get("vinh_vien").and_then(Value::as_bool).unwrap_or(false),
                    ma_ho_so: None,
                    tieu_de: s(v, "tieu_de"),
                    so_to: v.get("so_to").and_then(Value::as_str).map(str::to_string),
                })
                .collect();
            let t0 = std::time::Instant::now();
            let r = ghep(&ho_sos, &docs, Some(&thu_muc));
            let ms = t0.elapsed().as_millis();
            let mut kl: std::collections::BTreeMap<String, usize> = Default::default();
            for h in &r.ho_so {
                *kl.entry(serde_json::to_string(&h.ket_luan).unwrap().trim_matches('"').to_string()).or_default() += 1;
            }
            let mut ly_do: std::collections::BTreeMap<&str, usize> = Default::default();
            for k in &r.khong_khop {
                *ly_do.entry(k.ly_do.as_str()).or_default() += 1;
            }
            let tk = &r.tong_ket;
            println!(
                "CAP|{}|{}|hs={}|coTep={}|dat={}|tep={}|trang={}|soTo={}|khongKhop={}|{}ms|{:?}|{:?}",
                std::path::Path::new(&tep).file_stem().unwrap().to_string_lossy(),
                thu_muc,
                tk.ho_so_muc_luc,
                tk.ho_so_co_tep,
                tk.dat,
                tk.tep,
                tk.trang,
                tk.so_to_muc_luc,
                tk.tep_khong_khop,
                ms,
                kl,
                ly_do
            );
            for h in r.ho_so.iter().filter(|h| h.files > 0).take(3) {
                println!("  HS hop={} hs={} soTo={:?} tep={} trang={} pdfa={} -> {:?}", h.hop_so, h.ho_so_so, h.so_to, h.files, h.pages, h.pdfa_docs, h.ket_luan);
            }
            for k in r.khong_khop.iter().take(3) {
                println!("  KK hop={:?} hs={:?} {}", k.hop, k.ho_so, k.filename);
            }
        }
    }
}
