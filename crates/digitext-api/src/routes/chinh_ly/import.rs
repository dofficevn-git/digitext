// Copyright (c) 2026 DigiText. All rights reserved.
//! Nhập hồ sơ / văn bản theo dòng (desktop đọc Excel mẫu cũ rồi gửi JSON). Mỗi dòng một op,
//! lỗi trả theo dòng — không nuốt lỗi như bản cũ. Tên danh mục (THBQ, phòng ban, mảng, ngôn ngữ,
//! loại VB) trong dòng được đổi sang id / mã.

use super::{ctx_parts, op_ctx, project_access};
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use digitext_core::Permission;
use digitext_db::chinh_ly::he_thong_hoa as hth;
use digitext_db::chinh_ly::{self as cl, OpCtx, OpKind, OpResult, Row, SyncOp};
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/cl/ho-so/import", post(import_ho_so)).route("/cl/ho-so/{id}/van-ban/import", post(import_van_ban))
}

pub const IMPORT_MAX: usize = 5000;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HoSoImportReq {
    project_id: Uuid,
    phong_id: Uuid,
    /// create: dòng trùng số tạm → lỗi · upsert: ghi đè theo số tạm.
    #[serde(default = "default_mode")]
    mode: String,
    /// true: phòng ban / mảng ghi bằng tên (`ten_phong_ban`, `ten_mang`) chưa có trong phông → tự tạo
    /// (mã lấy từ `ma_don_vi` / `ma_mang` nếu có). Nhập mục lục hồ sơ dùng; mẫu cũ để mặc định false.
    #[serde(default)]
    create_missing: bool,
    rows: Vec<Row>,
}
fn default_mode() -> String {
    "upsert".into()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RowError {
    pub(super) row: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) field: Option<String>,
    pub(super) message: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportResp {
    created: usize,
    updated: usize,
    skipped: usize,
    /// Phòng ban / mảng tạo mới khi `create_missing`.
    created_phong_ban: usize,
    created_mang: usize,
    /// Nhập mục lục: số hồ sơ được xếp hộp theo cột Hộp số, số hộp tạo mới.
    gan_hop: usize,
    hop_tao_moi: usize,
    /// Ghi chú cho người nhập (vd. mã trong tệp đã bị phòng ban khác dùng → đặt mã khác).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notes: Vec<String>,
    errors: Vec<RowError>,
}

/// Bảng tra danh mục của dự án: (bảng → [(id, mã, tên, cha)]).
pub(super) struct Lookup {
    pub(super) thbq: Vec<(Uuid, String)>,
    /// THBQ vĩnh viễn (trang_thai = 1) — để biết loại của dòng (số tạm / hộp duy nhất theo loại, migration 0015).
    pub(super) thbq_vv: HashSet<Uuid>,
    pub(super) phong_ban: Vec<(Uuid, String, String)>,          // id, ma_don_vi, ten
    pub(super) mang: Vec<(Uuid, Uuid, String, String)>,         // id, phong_ban_id, ma_mang, ten
    pub(super) ngon_ngu: Vec<(String, String)>,                 // ma, ten
    pub(super) loai_vb: Vec<(String, String)>,
}

fn s<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(Value::as_str).map(str::trim).filter(|x| !x.is_empty())
}
fn u(v: &Value, k: &str) -> Option<Uuid> {
    s(v, k).and_then(|x| Uuid::parse_str(x).ok())
}
pub(super) fn eq_ci(a: &str, b: &str) -> bool {
    let norm = |s: &str| s.trim().to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
    norm(a) == norm(b)
}

pub(super) async fn lookup(st: &AppState, project: Uuid, phong: Uuid) -> Result<Lookup, ApiError> {
    let c = cl::danh_muc::catalog(&st.pool, project).await?;
    let arr = |k: &str| c.get(k).and_then(Value::as_array).cloned().unwrap_or_default();
    let thbq = arr("thbq").iter().filter_map(|v| Some((u(v, "id")?, s(v, "ten")?.to_owned()))).collect();
    let thbq_vv = arr("thbq").iter().filter(|v| v.get("trang_thai").and_then(Value::as_i64) == Some(1)).filter_map(|v| u(v, "id")).collect();
    let phong_ban = arr("phong_ban")
        .iter()
        .filter(|v| u(v, "phong_id") == Some(phong))
        .filter_map(|v| Some((u(v, "id")?, s(v, "ma_don_vi").unwrap_or("").to_owned(), s(v, "ten")?.to_owned())))
        .collect();
    let mang = arr("mang").iter().filter_map(|v| Some((u(v, "id")?, u(v, "phong_ban_id")?, s(v, "ma_mang").unwrap_or("00").to_owned(), s(v, "ten")?.to_owned()))).collect();
    let ngon_ngu = arr("ngon_ngu").iter().filter_map(|v| Some((s(v, "ma")?.to_owned(), s(v, "ten")?.to_owned()))).collect();
    let loai_vb = arr("loai_van_ban").iter().filter_map(|v| Some((s(v, "ma")?.to_owned(), s(v, "ten")?.to_owned()))).collect();
    Ok(Lookup { thbq, thbq_vv, phong_ban, mang, ngon_ngu, loai_vb })
}

/// Tạo một dòng danh mục (phòng ban / mảng) qua `apply_op` — cùng kiểm tra, quyền, lịch sử như mọi đường ghi.
async fn tao_danh_muc(st: &AppState, ctx: &OpCtx<'_>, table: &str, new: Row, field: &str) -> Result<Result<Uuid, RowError>, ApiError> {
    let id = Uuid::now_v7();
    let op = SyncOp { op_id: None, table: table.into(), id, op: OpKind::Upsert, base: None, new, if_ver: None, edited_at: None };
    let r = cl::apply_op(&st.pool, ctx, &op).await?;
    Ok(match r {
        OpResult::Applied { .. } | OpResult::Merged { .. } | OpResult::NoChange { .. } => Ok(id),
        other => Err(op_err(0, &other).map(|mut e| {
            e.field = Some(field.to_owned());
            e
        }).unwrap_or(RowError { row: 0, field: Some(field.to_owned()), message: "không tạo được dòng danh mục".into() })),
    })
}

/// Số thứ tự từ mã "01"/"2" (để `so_tt` giữ đúng thứ tự trong tệp).
fn so_tt_tu_ma(ma: Option<&str>) -> Option<i64> {
    ma.and_then(|m| m.trim().parse::<i64>().ok()).filter(|n| *n > 0)
}

/// Mã muốn dùng đã có trong `da_dung` → mã số kế tiếp sau mã số lớn nhất (đệm 2 chữ số). Trả (mã, có đổi không).
/// Mỗi tệp mục lục đánh I, II… / 1, 2… riêng: hai tệp cùng phông có thể đều ghi "II." cho hai phòng ban khác nhau.
fn ma_con_trong(muon: Option<&str>, da_dung: &[&str]) -> (Option<String>, bool) {
    let Some(m) = muon else { return (None, false) };
    if !da_dung.iter().any(|d| eq_ci(d, m)) {
        return (Some(m.to_owned()), false);
    }
    let max = da_dung.iter().filter_map(|d| d.trim().parse::<i64>().ok()).max().unwrap_or(0);
    (Some(format!("{:02}", max + 1)), true)
}

/// Đổi tên/mã danh mục trong dòng hồ sơ sang id: `thbq` (tên, hoặc số năm "10"), `ten_phong_ban`/`ma_don_vi`
/// (tên khớp trước, mã sau), `ten_mang`/`ma_mang`, `ngon_ngu` (tên → mã). `create_missing` → phòng ban / mảng
/// ghi bằng tên mà chưa có thì tạo trong phông rồi dùng luôn cho các dòng sau.
async fn resolve_ho_so(st: &AppState, ctx: &OpCtx<'_>, phong: Uuid, lk: &mut Lookup, row: &mut Row, create_missing: bool, out: &mut ImportResp) -> Result<Result<(), RowError>, ApiError> {
    let v = Value::Object(row.clone());
    if row.get("thbq_id").map_or(true, Value::is_null) {
        if let Some(t) = s(&v, "thbq") {
            let found = lk.thbq.iter().find(|(_, ten)| eq_ci(ten, t)).or_else(|| {
                // "10" → "10 năm"; "vv" → "Vĩnh viễn"
                let alt = if t.eq_ignore_ascii_case("vv") { "Vĩnh viễn".to_owned() } else { format!("{t} năm") };
                lk.thbq.iter().find(|(_, ten)| eq_ci(ten, &alt))
            });
            match found {
                Some((id, _)) => {
                    row.insert("thbq_id".into(), json!(id));
                }
                None => return Ok(Err(RowError { row: 0, field: Some("thbq".into()), message: format!("Thời hạn bảo quản {t:?} không có trong danh mục") })),
            }
        }
    }
    row.remove("thbq");
    let mut pb_id = u(&v, "phong_ban_id");
    if pb_id.is_none() {
        let ten = s(&v, "ten_phong_ban");
        let ma = s(&v, "ma_don_vi");
        if let Some(t) = ten.or(ma) {
            // Nhập mục lục (create_missing + có tên): chỉ khớp theo TÊN. Mã trong tệp là số thứ tự I, II… của riêng
            // tệp đó — khớp theo mã sẽ gắn nhầm "II. Tư pháp" (tệp B) vào "II. Văn hoá" (tệp A) mà không báo gì.
            let chi_theo_ten = create_missing && ten.is_some();
            let found = ten
                .and_then(|tn| lk.phong_ban.iter().find(|(_, _, ten)| eq_ci(ten, tn)))
                .or_else(|| if chi_theo_ten { None } else { ma.and_then(|m| lk.phong_ban.iter().find(|(_, mc, ten)| eq_ci(mc, m) || eq_ci(ten, m))) })
                .or_else(|| if chi_theo_ten { None } else { lk.phong_ban.iter().find(|(_, mc, ten)| eq_ci(mc, t) || eq_ci(ten, t)) });
            match found {
                Some((id, _, _)) => {
                    pb_id = Some(*id);
                    row.insert("phong_ban_id".into(), json!(id));
                }
                None if create_missing && ten.is_some() => {
                    let tn = ten.unwrap_or_default();
                    let da_dung: Vec<&str> = lk.phong_ban.iter().map(|(_, mc, _)| mc.as_str()).filter(|m| !m.is_empty()).collect();
                    let (ma_moi, doi) = ma_con_trong(ma, &da_dung);
                    if doi {
                        let cu = lk.phong_ban.iter().find(|(_, mc, _)| ma.is_some_and(|m| eq_ci(mc, m))).map(|(_, _, t)| t.as_str()).unwrap_or("");
                        out.notes.push(format!(
                            "Phòng ban {tn:?}: mã {} đã dùng cho {cu:?} → đặt mã {}",
                            ma.unwrap_or(""),
                            ma_moi.as_deref().unwrap_or("")
                        ));
                    }
                    let ma = ma_moi.as_deref();
                    let mut nr = Row::new();
                    nr.insert("phong_id".into(), json!(phong));
                    nr.insert("ten".into(), json!(tn));
                    if let Some(m) = ma {
                        nr.insert("ma_don_vi".into(), json!(m));
                    }
                    if let Some(n) = so_tt_tu_ma(ma) {
                        nr.insert("so_tt".into(), json!(n));
                    }
                    match tao_danh_muc(st, ctx, "cl_phong_ban", nr, "ma_don_vi").await? {
                        Ok(id) => {
                            lk.phong_ban.push((id, ma.unwrap_or("").to_owned(), tn.to_owned()));
                            out.created_phong_ban += 1;
                            pb_id = Some(id);
                            row.insert("phong_ban_id".into(), json!(id));
                        }
                        Err(mut e) => {
                            e.message = format!("Không tạo được phòng ban {tn:?}: {}", e.message);
                            return Ok(Err(e));
                        }
                    }
                }
                None => return Ok(Err(RowError { row: 0, field: Some("ma_don_vi".into()), message: format!("Phòng ban {t:?} không có trong phông") })),
            }
        }
    }
    row.remove("ma_don_vi");
    row.remove("ten_phong_ban");
    if row.get("mang_id").map_or(true, Value::is_null) {
        let ten = s(&v, "ten_mang");
        let ma = s(&v, "ma_mang");
        if let Some(t) = ten.or(ma) {
            let trong_pb = |pb: &Uuid| pb_id.map_or(true, |p| *pb == p);
            let chi_theo_ten = create_missing && ten.is_some();
            let found = ten
                .and_then(|tn| lk.mang.iter().find(|(_, pb, _, ten)| trong_pb(pb) && eq_ci(ten, tn)))
                .or_else(|| if chi_theo_ten { None } else { ma.and_then(|m| lk.mang.iter().find(|(_, pb, mc, ten)| trong_pb(pb) && (eq_ci(mc, m) || eq_ci(ten, m)))) })
                .or_else(|| if chi_theo_ten { None } else { lk.mang.iter().find(|(_, pb, mc, ten)| trong_pb(pb) && (eq_ci(mc, t) || eq_ci(ten, t))) });
            match found {
                Some((id, pb, _, _)) => {
                    row.insert("mang_id".into(), json!(id));
                    if pb_id.is_none() {
                        row.insert("phong_ban_id".into(), json!(pb));
                    }
                }
                None if create_missing && ten.is_some() => {
                    let Some(pb) = pb_id else {
                        return Ok(Err(RowError { row: 0, field: Some("ma_mang".into()), message: format!("Mảng {t:?} cần có phòng ban để tạo") }));
                    };
                    let tn = ten.unwrap_or_default();
                    let da_dung: Vec<&str> = lk.mang.iter().filter(|(_, p, _, _)| *p == pb).map(|(_, _, mc, _)| mc.as_str()).collect();
                    let (ma_moi, doi) = ma_con_trong(ma, &da_dung);
                    if doi {
                        let cu = lk.mang.iter().find(|(_, p, mc, _)| *p == pb && ma.is_some_and(|m| eq_ci(mc, m))).map(|(_, _, _, t)| t.as_str()).unwrap_or("");
                        out.notes.push(format!(
                            "Mảng {tn:?}: mã {} đã dùng cho {cu:?} trong cùng phòng ban → đặt mã {}",
                            ma.unwrap_or(""),
                            ma_moi.as_deref().unwrap_or("")
                        ));
                    }
                    let ma = ma_moi.as_deref();
                    let mut nr = Row::new();
                    nr.insert("phong_ban_id".into(), json!(pb));
                    nr.insert("ten".into(), json!(tn));
                    if let Some(m) = ma {
                        nr.insert("ma_mang".into(), json!(m));
                    }
                    if let Some(n) = so_tt_tu_ma(ma) {
                        nr.insert("so_tt".into(), json!(n));
                    }
                    match tao_danh_muc(st, ctx, "cl_mang", nr, "ma_mang").await? {
                        Ok(id) => {
                            lk.mang.push((id, pb, ma.unwrap_or("00").to_owned(), tn.to_owned()));
                            out.created_mang += 1;
                            row.insert("mang_id".into(), json!(id));
                        }
                        Err(mut e) => {
                            e.message = format!("Không tạo được mảng {tn:?}: {}", e.message);
                            return Ok(Err(e));
                        }
                    }
                }
                None => return Ok(Err(RowError { row: 0, field: Some("ma_mang".into()), message: format!("Mảng công việc {t:?} không có trong phòng ban") })),
            }
        }
    }
    row.remove("ma_mang");
    row.remove("ten_mang");
    if let Some(t) = s(&v, "ngon_ngu") {
        if let Some((ma, _)) = lk.ngon_ngu.iter().find(|(ma, ten)| ma == t || eq_ci(ten, t)) {
            row.insert("ngon_ngu".into(), json!(ma));
        }
    }
    Ok(Ok(()))
}

pub(super) fn resolve_van_ban(lk: &Lookup, row: &mut Row) {
    let v = Value::Object(row.clone());
    if let Some(t) = s(&v, "loai_vb").or(s(&v, "ten_loai_vb")) {
        if let Some((ma, _)) = lk.loai_vb.iter().find(|(ma, ten)| ma == t || eq_ci(ten, t)) {
            row.insert("loai_vb".into(), json!(ma));
        }
    }
    row.remove("ten_loai_vb");
    if let Some(t) = s(&v, "ngon_ngu") {
        if let Some((ma, _)) = lk.ngon_ngu.iter().find(|(ma, ten)| ma == t || eq_ci(ten, t)) {
            row.insert("ngon_ngu".into(), json!(ma));
        }
    }
}

pub(super) fn op_err(i: usize, r: &OpResult) -> Option<RowError> {
    match r {
        OpResult::Rejected { message, errors, .. } => Some(RowError { row: i, field: errors.first().map(|e| e.field.clone()), message: message.clone() }),
        OpResult::Conflict { conflict_fields, .. } => Some(RowError { row: i, field: conflict_fields.first().cloned(), message: "Dữ liệu đã bị người khác sửa".into() }),
        _ => None,
    }
}

async fn import_ho_so(State(st): State<AppState>, ts: Ts, Json(b): Json<HoSoImportReq>) -> Result<Json<ImportResp>, ApiError> {
    project_access(&st, &ts, b.project_id).await?;
    if b.rows.len() > IMPORT_MAX {
        return Err(ApiError::bad_request(format!("Tối đa {IMPORT_MAX} dòng mỗi lần — hãy chia nhỏ tệp")));
    }
    let mut lk = lookup(&st, b.project_id, b.phong_id).await?;
    let parts = ctx_parts(&ts);
    let ctx = op_ctx(&ts, b.project_id, "import", &parts);
    let mut out = ImportResp::default();
    // Nhập mục lục ghi luôn Hồ sơ số và xếp hộp theo cột Hộp số — việc của Hệ thống hoá nên cần quyền đó.
    let duoc_xep = ts.has_perm(Permission::DossierSystematize);
    let mut can_xep: Vec<(usize, hth::BoxRow)> = Vec::new();
    let mut bo_so_hop = false;
    for (i, mut row) in b.rows.into_iter().enumerate() {
        let n = i + 1;
        row.insert("phong_id".into(), json!(b.phong_id));
        let hop_so = row.remove("hop_so").and_then(|v| v.as_str().map(|x| x.trim().to_owned())).filter(|x| !x.is_empty());
        if !duoc_xep && (hop_so.is_some() || row.contains_key("ho_so_so")) {
            row.remove("ho_so_so");
            bo_so_hop = true;
        }
        if let Err(mut e) = resolve_ho_so(&st, &ctx, b.phong_id, &mut lk, &mut row, b.create_missing, &mut out).await? {
            e.row = n;
            out.errors.push(e);
            continue;
        }
        let so_tam = row.get("so_tam").and_then(Value::as_str).map(str::trim).unwrap_or("").to_owned();
        if so_tam.is_empty() {
            out.errors.push(RowError { row: n, field: Some("so_tam".into()), message: "Thiếu số tạm".into() });
            continue;
        }
        // Số tạm duy nhất trong (phông, loại vĩnh viễn / có thời hạn): mục lục mỗi loại đánh lại từ 1.
        let vinh_vien = u(&Value::Object(row.clone()), "thbq_id").is_some_and(|t| lk.thbq_vv.contains(&t));
        let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cl_ho_so WHERE phong_id = $1 AND vinh_vien = $3 AND lower(btrim(so_tam)) = lower(btrim($2)) AND deleted_at IS NULL")
            .bind(b.phong_id)
            .bind(&so_tam)
            .bind(vinh_vien)
            .fetch_optional(&st.pool)
            .await?;
        let id = match (existing, b.mode.as_str()) {
            (Some(_), "create") => {
                out.errors.push(RowError { row: n, field: Some("so_tam".into()), message: format!("Số tạm {so_tam:?} đã có trong phông (cùng loại {})", if vinh_vien { "vĩnh viễn" } else { "có thời hạn" }) });
                continue;
            }
            (Some(id), _) => id,
            (None, _) => Uuid::now_v7(),
        };
        let op = SyncOp { op_id: None, table: "cl_ho_so".into(), id, op: OpKind::Upsert, base: None, new: row, if_ver: None, edited_at: None };
        let r = cl::apply_op(&st.pool, &ctx, &op).await?;
        if matches!(r, OpResult::Applied { .. } | OpResult::Merged { .. } | OpResult::NoChange { .. }) && duoc_xep {
            if let Some(hs) = hop_so {
                can_xep.push((n, hth::BoxRow { ho_so_id: id, hop_so: Some(hs), bo_so: None, tap_so: None, clear: false }));
            }
        }
        match (&r, existing) {
            (OpResult::Applied { .. }, None) => out.created += 1,
            (OpResult::Applied { .. } | OpResult::Merged { .. }, Some(_)) => out.updated += 1,
            (OpResult::NoChange { .. }, _) => out.skipped += 1,
            _ => {
                if let Some(e) = op_err(n, &r) {
                    out.errors.push(e);
                }
            }
        }
    }
    if !can_xep.is_empty() {
        let rows: Vec<hth::BoxRow> = can_xep.iter().map(|(_, r)| r.clone()).collect();
        let br = hth::boxing(&st.pool, &ctx, 1, &rows).await?;
        out.gan_hop = br.applied;
        out.hop_tao_moi = br.hops_created;
        for e in br.errors {
            let n = can_xep.get(e.row.saturating_sub(1)).map_or(0, |(n, _)| *n);
            out.errors.push(RowError { row: n, field: Some("hop_so".into()), message: format!("Xếp hộp: {}", e.message) });
        }
    }
    if bo_so_hop {
        out.notes.push("Tài khoản chưa có quyền Hệ thống hoá — đã nhập hồ sơ nhưng bỏ qua Hồ sơ số và Hộp số".into());
    }
    digitext_db::audit::append(&st.pool, super::audit(&ts, "cl.import_ho_so", b.phong_id.to_string(), json!({"created": out.created, "updated": out.updated, "errors": out.errors.len(), "created_phong_ban": out.created_phong_ban, "created_mang": out.created_mang, "gan_hop": out.gan_hop, "hop_tao_moi": out.hop_tao_moi}))).await?;
    Ok(Json(out))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VanBanImportReq {
    project_id: Uuid,
    #[serde(default = "default_mode")]
    mode: String,
    rows: Vec<Row>,
}

async fn import_van_ban(State(st): State<AppState>, ts: Ts, Path(ho_so_id): Path<Uuid>, Json(b): Json<VanBanImportReq>) -> Result<Json<ImportResp>, ApiError> {
    project_access(&st, &ts, b.project_id).await?;
    if b.rows.len() > IMPORT_MAX {
        return Err(ApiError::bad_request(format!("Tối đa {IMPORT_MAX} dòng mỗi lần — hãy chia nhỏ tệp")));
    }
    let hs = cl::ho_so::get(&st.pool, ho_so_id).await?.ok_or_else(ApiError::not_found)?;
    let phong = u(&hs, "phong_id").ok_or_else(ApiError::not_found)?;
    if u(&hs, "project_id") != Some(b.project_id) {
        return Err(ApiError::forbidden());
    }
    let lk = lookup(&st, b.project_id, phong).await?;
    let parts = ctx_parts(&ts);
    let ctx = op_ctx(&ts, b.project_id, "import", &parts);
    let mut out = ImportResp::default();
    let mut next = cl::ho_so::next_stt(&st.pool, ho_so_id).await?;
    for (i, mut row) in b.rows.into_iter().enumerate() {
        let n = i + 1;
        row.insert("ho_so_id".into(), json!(ho_so_id));
        resolve_van_ban(&lk, &mut row);
        let stt = cl::rows::i64_of(&row, "so_thu_tu");
        let stt = match stt {
            Some(v) if v >= 1 => v as i32,
            _ => {
                row.insert("so_thu_tu".into(), json!(next));
                next += 1;
                next - 1
            }
        };
        let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cl_van_ban WHERE ho_so_id = $1 AND so_thu_tu = $2 AND deleted_at IS NULL")
            .bind(ho_so_id)
            .bind(stt)
            .fetch_optional(&st.pool)
            .await?;
        let id = match (existing, b.mode.as_str()) {
            (Some(_), "create") => {
                out.errors.push(RowError { row: n, field: Some("so_thu_tu".into()), message: format!("Số thứ tự {stt} đã có trong hồ sơ") });
                continue;
            }
            (Some(id), _) => id,
            (None, _) => Uuid::now_v7(),
        };
        let op = SyncOp { op_id: None, table: "cl_van_ban".into(), id, op: OpKind::Upsert, base: None, new: row, if_ver: None, edited_at: None };
        let r = cl::apply_op(&st.pool, &ctx, &op).await?;
        match (&r, existing) {
            (OpResult::Applied { .. }, None) => out.created += 1,
            (OpResult::Applied { .. } | OpResult::Merged { .. }, Some(_)) => out.updated += 1,
            (OpResult::NoChange { .. }, _) => out.skipped += 1,
            _ => {
                if let Some(e) = op_err(n, &r) {
                    out.errors.push(e);
                }
            }
        }
        if stt >= next {
            next = stt + 1;
        }
    }
    digitext_db::audit::append(&st.pool, super::audit(&ts, "cl.import_van_ban", ho_so_id.to_string(), json!({"created": out.created, "updated": out.updated, "errors": out.errors.len()}))).await?;
    Ok(Json(out))
}

#[cfg(test)]
mod ma_tests {
    use super::ma_con_trong;
    #[test]
    fn ma_trung_lay_ma_ke_tiep() {
        assert_eq!(ma_con_trong(Some("02"), &["01", "03"]), (Some("02".into()), false));
        assert_eq!(ma_con_trong(Some("02"), &["01", "02"]), (Some("03".into()), true));
        assert_eq!(ma_con_trong(Some("01"), &["01", "02", "05"]), (Some("06".into()), true));
        assert_eq!(ma_con_trong(None, &["01"]), (None, false));
    }
}
