// Copyright (c) 2026 DigiText. All rights reserved.
//! Tạo văn bản chỉnh lý hàng loạt từ trường OCR của tài liệu đã quét (2/11/2026).
//!
//! `POST /cl/van-ban/tu-ocr` `{projectId, phongId, hoSoIds?: [uuid], after?: uuid}`:
//! - Hồ sơ của phông (hoặc `hoSoIds`) ghép với tài liệu máy chủ theo (hộp, hồ sơ số) — `ho_so_ref` "hộp/hồ sơ" hoặc
//!   tầng 2/3 của `rel_path`, chuẩn hoá bỏ tiền tố HỘP / HỒ SƠ và số 0 đầu. Hai hồ sơ (vĩnh viễn / có thời hạn) cùng
//!   khoá → tài liệu bị bỏ, báo lý do, không đoán.
//! - Mỗi tài liệu một văn bản: số thứ tự = đuôi `.{7 chữ số}.pdf` của tên tệp, không có → `thuTu` OCR → số kế tiếp
//!   trống trong hồ sơ. Trường OCR (18 khoá `VanBanField`) đổi sang cột `cl_van_ban`; "Số/KH" tách ở `/` đầu thành
//!   `so_van_ban` + `so_ky_hieu`; loại VB tên → mã, không khớp → "32" + ghi chú.
//! - Văn bản đã có: nối theo `cl_van_ban_file.document_id` trước, rồi theo (hồ sơ, số thứ tự). Dòng nhập tay hoặc đã
//!   có người chạm (`nguon` trống, `duyet_luc` có, `updated_by ≠ created_by`) chỉ được nối tệp, không ghi đè trường.
//! - `nguon = 'ocr'`, chưa duyệt; nguồn op `"ocr"` → lịch sử ghi hành động 'ocr', không vào tổng kết nhân viên.
//! - Tối đa `TU_OCR_MAX_DOCS` tài liệu một lần (mỗi `apply_op` một giao dịch); hết lô trả `conTiep` + `after` để
//!   desktop gọi tiếp.

use super::import::{eq_ci, lookup, op_err, resolve_van_ban, Lookup, RowError};
use super::{ctx_parts, op_ctx, project_access};
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use digitext_core::chinh_ly::{self as core_cl, codes};
use digitext_core::Permission;
use digitext_db::chinh_ly::{self as cl, OpKind, OpResult, Row, SyncOp};
use digitext_db::{documents, ocr_fields};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/cl/van-ban/tu-ocr", post(tu_ocr))
}

/// Số tài liệu xử lý tối đa một lần gọi (≈ 40 ms/op, `post_long` 600 s).
pub const TU_OCR_MAX_DOCS: usize = 2000;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Req {
    project_id: Uuid,
    phong_id: Uuid,
    /// Chỉ các hồ sơ này (trống = cả phông).
    #[serde(default)]
    ho_so_ids: Vec<Uuid>,
    /// Con trỏ: bỏ qua hồ sơ có id ≤ `after` (hồ sơ xét theo thứ tự id).
    #[serde(default)]
    after: Option<Uuid>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KhongKhop {
    document_id: Uuid,
    filename: String,
    ly_do: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Resp {
    /// Hồ sơ xét trong lần gọi này / có ít nhất một tài liệu / không có tài liệu nào.
    ho_so_xet: usize,
    ho_so_khop: usize,
    ho_so_khong_tep: usize,
    tai_lieu_xet: usize,
    /// Tài liệu không ghép được hồ sơ (chỉ trả ở lần gọi đầu, `after` trống).
    tai_lieu_khong_khop: Vec<KhongKhop>,
    created: usize,
    updated: usize,
    /// Văn bản nhập tay / đã duyệt: chỉ nối tệp, giữ nguyên trường.
    da_lien_ket_tep: usize,
    /// Không có gì đổi so với lần trước.
    giu_nguyen: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notes: Vec<String>,
    errors: Vec<RowError>,
    con_tiep: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<Uuid>,
}

/// Khoá ghép: chữ thường, bỏ tiền tố "hộp"/"hop"/"hồ sơ"/"ho so"/"hs", bỏ ký tự không phải chữ số/chữ cái ở hai đầu,
/// toàn chữ số thì bỏ số 0 đầu ("01" = "1"; "572A" giữ nguyên).
pub(super) fn khoa(s: &str) -> String {
    let mut t = s.trim().to_lowercase();
    // bỏ dấu đơn giản cho hai từ tiền tố thường gặp
    t = t.replace("hộp", "hop").replace("hồ sơ", "ho so");
    for p in ["ho so", "hop", "hs"] {
        if t.starts_with(p) {
            t = t[p.len()..].to_owned();
            break;
        }
    }
    let t = t.trim_matches(|c: char| !c.is_alphanumeric()).to_owned();
    if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
        let z = t.trim_start_matches('0');
        return if z.is_empty() { "0".into() } else { z.to_owned() };
    }
    t
}

/// Số thứ tự từ đuôi tên tệp quét `{stem}.{seq:07}.pdf` (archive/build.rs) — None nếu không đúng dạng.
pub(super) fn stt_tu_ten_tep(filename: &str) -> Option<i32> {
    let stem = filename.strip_suffix(".pdf").or_else(|| filename.strip_suffix(".PDF")).unwrap_or(filename);
    let last = stem.rsplit('.').next()?;
    if last.len() == 7 && last.chars().all(|c| c.is_ascii_digit()) {
        last.parse::<i32>().ok().filter(|n| *n >= 1)
    } else {
        None
    }
}

/// "123/QĐ-UBND" → ("123", "QĐ-UBND"); không có "/" → (None, cả chuỗi).
pub(super) fn tach_so_ky_hieu(s: &str) -> (Option<String>, Option<String>) {
    let s = s.trim();
    if s.is_empty() {
        return (None, None);
    }
    match s.split_once('/') {
        Some((a, b)) if !a.trim().is_empty() && !b.trim().is_empty() => (Some(a.trim().to_owned()), Some(b.trim().to_owned())),
        _ => (None, Some(s.to_owned())),
    }
}

fn chu_so(s: &str) -> Option<i64> {
    let d: String = s.chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
    d.parse::<i64>().ok().filter(|n| *n > 0)
}

fn ma_hop_le(s: &str, bang: &[(&str, &str)]) -> Option<String> {
    let t = s.trim();
    bang.iter().find(|(m, ten)| *m == t || eq_ci(ten, t)).map(|(m, _)| (*m).to_owned())
}

/// Dòng `cl_van_ban` từ trường OCR của một tài liệu (không gồm `ho_so_id` / `so_thu_tu`).
fn dong_tu_truong(lk: &Lookup, f: &HashMap<String, String>, page_count: Option<i32>) -> Row {
    let mut row = Row::new();
    let mut ghi_chu: Vec<String> = Vec::new();
    let g = |k: &str| f.get(k).map(|v| v.trim()).filter(|v| !v.is_empty());
    if let Some(l) = g("loaiVanBan") {
        let mut r = Row::new();
        r.insert("loai_vb".into(), json!(l));
        resolve_van_ban(lk, &mut r);
        match r.get("loai_vb").and_then(Value::as_str) {
            Some(m) if lk.loai_vb.iter().any(|(ma, _)| ma == m) => row.insert("loai_vb".into(), json!(m)),
            _ => {
                ghi_chu.push(format!("Loại VB theo OCR: {l}"));
                row.insert("loai_vb".into(), json!("32"))
            }
        };
    } else {
        row.insert("loai_vb".into(), json!("32"));
    }
    if let Some(s) = g("soKyHieu") {
        let (so, kh) = tach_so_ky_hieu(s);
        row.insert("so_van_ban".into(), json!(so));
        row.insert("so_ky_hieu".into(), json!(kh));
    }
    if let Some(d) = g("ngayThang") {
        match core_cl::normalize_date(d) {
            Ok(Some(x)) => {
                row.insert("ngay_thang".into(), json!(x));
            }
            _ => ghi_chu.push(format!("Ngày tháng theo OCR: {d}")),
        }
    }
    for (k, c) in [("coQuanBanHanh", "co_quan_ban_hanh"), ("coQuanChuQuan", "co_quan_chu_quan"), ("nguoiKy", "nguoi_ky"), ("trichYeu", "trich_yeu"), ("butTich", "but_tich"), ("taiLieuSo", "tai_lieu_so")] {
        if let Some(v) = g(k) {
            row.insert(c.into(), json!(v));
        }
    }
    for (k, c) in [("toSo", "to_so"), ("trangSo", "trang_so")] {
        if let Some(n) = g(k).and_then(chu_so) {
            row.insert(c.into(), json!(n));
        }
    }
    match g("soTrang").and_then(chu_so).or_else(|| page_count.filter(|n| *n > 0).map(i64::from)) {
        Some(n) => {
            row.insert("so_trang".into(), json!(n));
        }
        None => {}
    }
    if let Some(l) = g("ngonNgu") {
        let mut r = Row::new();
        r.insert("ngon_ngu".into(), json!(l));
        resolve_van_ban(lk, &mut r);
        if let Some(m) = r.get("ngon_ngu").and_then(Value::as_str).filter(|m| codes::NGON_NGU.iter().any(|(ma, _)| ma == m) || lk.ngon_ngu.iter().any(|(ma, _)| ma == m)) {
            row.insert("ngon_ngu".into(), json!(m));
        }
    }
    row.entry("ngon_ngu".to_owned()).or_insert(json!("01"));
    row.insert("che_do_su_dung".into(), json!(g("cheDoSuDung").and_then(|v| ma_hop_le(v, codes::TIEP_CAN)).unwrap_or_else(|| "01".into())));
    row.insert("do_tin_cay".into(), json!(g("doTinCay").and_then(|v| ma_hop_le(v, codes::TIN_CAY)).unwrap_or_else(|| "03".into())));
    row.insert("tinh_trang_vat_ly".into(), json!(g("tinhTrangVatLy").and_then(|v| ma_hop_le(v, codes::TINH_TRANG_VAT_LY)).unwrap_or_else(|| "02".into())));
    if let Some(v) = g("ghiChu") {
        ghi_chu.insert(0, v.to_owned());
    }
    if f.is_empty() {
        ghi_chu.push("Chưa có kết quả OCR".into());
    }
    if !ghi_chu.is_empty() {
        row.insert("ghi_chu".into(), json!(ghi_chu.join("; ")));
    }
    row.insert("nguon".into(), json!("ocr"));
    row
}

struct VbCu {
    id: Uuid,
    /// Người đã chạm (nhập tay, đã duyệt, hoặc có người sửa sau khi máy tạo) → không ghi đè trường.
    da_cham: bool,
}

async fn tu_ocr(State(st): State<AppState>, ts: Ts, Json(b): Json<Req>) -> Result<Json<Resp>, ApiError> {
    project_access(&st, &ts, b.project_id).await?;
    if !ts.has_perm(Permission::DossierCreate) || !ts.has_perm(Permission::DossierUpdate) {
        return Err(ApiError::forbidden());
    }
    let lk = lookup(&st, b.project_id, b.phong_id).await?;
    let parts = ctx_parts(&ts);
    let ctx = op_ctx(&ts, b.project_id, "ocr", &parts);
    let mut out = Resp::default();

    // 1. Hồ sơ của phông → khoá (hộp, hồ sơ số); trùng khoá giữa hai loại → không ghép.
    let mut ho_sos = cl::ho_so::khoa_ho_so_cua_phong(&st.pool, b.project_id, b.phong_id).await?;
    if !b.ho_so_ids.is_empty() {
        let chon: BTreeSet<Uuid> = b.ho_so_ids.iter().copied().collect();
        ho_sos.retain(|h| chon.contains(&h.id));
    }
    let mut theo_khoa: HashMap<(String, String), Vec<usize>> = HashMap::new();
    for (i, h) in ho_sos.iter().enumerate() {
        let hs = h.ho_so_so.as_deref().filter(|s| !s.trim().is_empty()).unwrap_or(h.so_tam.as_str());
        theo_khoa.entry((khoa(h.hop_so.as_deref().unwrap_or("")), khoa(hs))).or_default().push(i);
    }

    // 2. Tài liệu của dự án → hồ sơ.
    let docs = documents::list_for_project_refs(&st.pool, b.project_id).await?;
    let mut tep_cua: HashMap<usize, Vec<&documents::DocRefRow>> = HashMap::new();
    for d in &docs {
        let k = (khoa(&d.hop), khoa(&d.ho_so));
        match theo_khoa.get(&k) {
            Some(v) if v.len() == 1 => tep_cua.entry(v[0]).or_default().push(d),
            Some(_) => {
                if b.after.is_none() {
                    out.tai_lieu_khong_khop.push(KhongKhop { document_id: d.id, filename: d.filename.clone(), ly_do: format!("Hộp {}/hồ sơ {} trùng giữa hồ sơ vĩnh viễn và có thời hạn — chọn hồ sơ và chạy cho riêng hồ sơ đó", d.hop, d.ho_so) });
                }
            }
            None => {
                if b.after.is_none() && b.ho_so_ids.is_empty() {
                    out.tai_lieu_khong_khop.push(KhongKhop { document_id: d.id, filename: d.filename.clone(), ly_do: format!("Không có hồ sơ hộp {:?} / hồ sơ số {:?} trong phông", d.hop, d.ho_so) });
                }
            }
        }
    }

    // 3. Lô hồ sơ của lần gọi này (theo id tăng dần, sau `after`), gom tối đa TU_OCR_MAX_DOCS tài liệu.
    let mut lo: Vec<usize> = Vec::new();
    let mut so_tep = 0usize;
    for (i, h) in ho_sos.iter().enumerate() {
        if b.after.is_some_and(|a| h.id <= a) {
            continue;
        }
        let n = tep_cua.get(&i).map_or(0, Vec::len);
        if !lo.is_empty() && so_tep + n > TU_OCR_MAX_DOCS {
            out.con_tiep = true;
            break;
        }
        lo.push(i);
        so_tep += n;
    }
    out.ho_so_xet = lo.len();
    out.after = lo.last().map(|i| ho_sos[*i].id);
    if lo.is_empty() {
        out.con_tiep = false;
        return Ok(Json(out));
    }
    let doc_ids: Vec<Uuid> = lo.iter().flat_map(|i| tep_cua.get(i).map(|v| v.iter().map(|d| d.id).collect::<Vec<_>>()).unwrap_or_default()).collect();
    out.tai_lieu_xet = doc_ids.len();

    // 4. Trường tốt nhất mỗi tài liệu.
    let mut truong: HashMap<Uuid, HashMap<String, String>> = HashMap::new();
    for chunk in doc_ids.chunks(1000) {
        for f in ocr_fields::best_fields_for_docs(&st.pool, chunk).await? {
            truong.entry(f.document_id).or_default().insert(f.field_key, f.value);
        }
    }

    // 5. Văn bản / tệp đã có của phông (một truy vấn mỗi bảng).
    let mut vb_theo_stt: HashMap<(Uuid, i32), VbCu> = HashMap::new();
    let mut stt_da_dung: HashMap<Uuid, BTreeSet<i32>> = HashMap::new();
    let mut vb_cham: HashMap<Uuid, bool> = HashMap::new();
    for v in cl::ho_so::van_ban_cua_phong(&st.pool, b.phong_id).await? {
        let da_cham = v.nguon.as_deref() != Some("ocr") || v.duyet_luc.is_some() || (v.updated_by.is_some() && v.updated_by != v.created_by);
        vb_cham.insert(v.id, da_cham);
        stt_da_dung.entry(v.ho_so_id).or_default().insert(v.so_thu_tu);
        vb_theo_stt.insert((v.ho_so_id, v.so_thu_tu), VbCu { id: v.id, da_cham });
    }
    let mut vb_cua_tep: HashMap<Uuid, Uuid> = HashMap::new();
    for t in cl::ho_so::van_ban_tep_cua_phong(&st.pool, b.phong_id).await? {
        vb_cua_tep.insert(t.document_id, t.van_ban_id);
    }

    // 6. Từng hồ sơ, từng tài liệu.
    for i in lo {
        let h = &ho_sos[i];
        let Some(teps) = tep_cua.get(&i) else {
            out.ho_so_khong_tep += 1;
            continue;
        };
        out.ho_so_khop += 1;
        let dung = stt_da_dung.entry(h.id).or_default();
        for d in teps {
            let n = out.created + out.updated + out.da_lien_ket_tep + out.giu_nguyen + out.errors.len() + 1;
            let rong = HashMap::new();
            let f = truong.get(&d.id).unwrap_or(&rong);
            // Văn bản đích: theo tệp đã nối → theo số thứ tự → tạo mới.
            let (vb_id, stt, da_cham, moi) = if let Some(id) = vb_cua_tep.get(&d.id).copied() {
                let stt = vb_theo_stt.iter().find(|(_, v)| v.id == id).map(|((_, s), _)| *s).unwrap_or(0);
                (id, stt, vb_cham.get(&id).copied().unwrap_or(true), false)
            } else {
                let muon = stt_tu_ten_tep(&d.filename).or_else(|| f.get("thuTu").and_then(|v| chu_so(v)).map(|v| v as i32));
                match muon.and_then(|s| vb_theo_stt.get(&(h.id, s)).map(|v| (s, v))) {
                    Some((s, v)) if !vb_cua_tep.values().any(|x| *x == v.id) => (v.id, s, v.da_cham, false),
                    Some((s, _)) => {
                        // Số thứ tự đã thuộc tài liệu khác (quét lại, bản ký) → số kế tiếp còn trống.
                        let mut k = dung.iter().next_back().copied().unwrap_or(0) + 1;
                        while dung.contains(&k) {
                            k += 1;
                        }
                        out.notes.push(format!("{}: số thứ tự {s} đã thuộc tài liệu khác trong hồ sơ → dùng số {k}", d.filename));
                        (Uuid::now_v7(), k, false, true)
                    }
                    None => {
                        let mut k = muon.unwrap_or_else(|| dung.iter().next_back().copied().unwrap_or(0) + 1);
                        while dung.contains(&k) {
                            k += 1;
                        }
                        (Uuid::now_v7(), k, false, true)
                    }
                }
            };
            dung.insert(stt);
            if da_cham {
                // Nhập tay / đã duyệt: chỉ nối tệp.
                if !vb_cua_tep.contains_key(&d.id) {
                    noi_tep(&st, &ctx, vb_id, d, n, &mut out).await?;
                    vb_cua_tep.insert(d.id, vb_id);
                }
                out.da_lien_ket_tep += 1;
                continue;
            }
            let mut row = dong_tu_truong(&lk, f, d.page_count);
            row.insert("ho_so_id".into(), json!(h.id));
            row.insert("so_thu_tu".into(), json!(stt));
            let op = SyncOp { op_id: None, table: "cl_van_ban".into(), id: vb_id, op: OpKind::Upsert, base: None, new: row, if_ver: None, edited_at: None };
            let r = cl::apply_op(&st.pool, &ctx, &op).await?;
            match &r {
                OpResult::Applied { .. } if moi => out.created += 1,
                OpResult::Applied { .. } | OpResult::Merged { .. } => out.updated += 1,
                OpResult::NoChange { .. } => out.giu_nguyen += 1,
                _ => {
                    if let Some(mut e) = op_err(n, &r) {
                        e.message = format!("{}: {}", d.filename, e.message);
                        out.errors.push(e);
                    }
                    continue;
                }
            }
            vb_theo_stt.insert((h.id, stt), VbCu { id: vb_id, da_cham: false });
            vb_cham.insert(vb_id, false);
            if !vb_cua_tep.contains_key(&d.id) {
                noi_tep(&st, &ctx, vb_id, d, n, &mut out).await?;
                vb_cua_tep.insert(d.id, vb_id);
            }
        }
    }
    digitext_db::audit::append(
        &st.pool,
        super::audit(&ts, "cl.van_ban_tu_ocr", b.phong_id.to_string(), json!({"ho_so_xet": out.ho_so_xet, "tai_lieu_xet": out.tai_lieu_xet, "created": out.created, "updated": out.updated, "da_lien_ket_tep": out.da_lien_ket_tep, "giu_nguyen": out.giu_nguyen, "khong_khop": out.tai_lieu_khong_khop.len(), "errors": out.errors.len(), "con_tiep": out.con_tiep})),
    )
    .await?;
    Ok(Json(out))
}

/// Nối tài liệu máy chủ vào văn bản (`cl_van_ban_file`, một dòng / tài liệu).
async fn noi_tep(st: &AppState, ctx: &cl::OpCtx<'_>, vb_id: Uuid, d: &documents::DocRefRow, n: usize, out: &mut Resp) -> Result<(), ApiError> {
    let mut row = Row::new();
    row.insert("van_ban_id".into(), json!(vb_id));
    row.insert("document_id".into(), json!(d.id));
    row.insert("name".into(), json!(d.filename));
    row.insert("format".into(), json!("pdf"));
    if let Some(p) = d.page_count.filter(|p| *p > 0) {
        row.insert("so_trang".into(), json!(p));
    }
    row.insert("so_tt".into(), json!(1));
    let op = SyncOp { op_id: None, table: "cl_van_ban_file".into(), id: Uuid::now_v7(), op: OpKind::Upsert, base: None, new: row, if_ver: None, edited_at: None };
    let r = cl::apply_op(&st.pool, ctx, &op).await?;
    if let Some(mut e) = op_err(n, &r) {
        e.message = format!("{}: nối tệp: {}", d.filename, e.message);
        out.errors.push(e);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{khoa, stt_tu_ten_tep, tach_so_ky_hieu};

    #[test]
    fn khoa_bo_tien_to_va_so_khong() {
        assert_eq!(khoa("01"), "1");
        assert_eq!(khoa("HOP 01"), "1");
        assert_eq!(khoa("Hộp 007"), "7");
        assert_eq!(khoa("HO SO 12"), "12");
        assert_eq!(khoa("Hồ sơ 12"), "12");
        assert_eq!(khoa("572A"), "572a");
        assert_eq!(khoa(""), "");
        assert_eq!(khoa("000"), "0");
    }

    #[test]
    fn stt_tu_ten_tep_quet() {
        assert_eq!(stt_tu_ten_tep("H05.13.24.01.2005.01.01.0000001.pdf"), Some(1));
        assert_eq!(stt_tu_ten_tep("UBND.01.1997.12.0000034.PDF"), Some(34));
        assert_eq!(stt_tu_ten_tep("tailieu.pdf"), None);
        assert_eq!(stt_tu_ten_tep("x.0000000.pdf"), None);
    }

    #[test]
    fn tach_so_va_ky_hieu() {
        assert_eq!(tach_so_ky_hieu("123/QĐ-UBND"), (Some("123".into()), Some("QĐ-UBND".into())));
        assert_eq!(tach_so_ky_hieu("QĐ-UBND"), (None, Some("QĐ-UBND".into())));
        assert_eq!(tach_so_ky_hieu("  "), (None, None));
    }
}
