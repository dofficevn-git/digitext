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
use digitext_db::chinh_ly::{self as cl, OpKind, OpResult, Row, SyncOp};
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
    rows: Vec<Row>,
}
fn default_mode() -> String {
    "upsert".into()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RowError {
    row: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
    message: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportResp {
    created: usize,
    updated: usize,
    skipped: usize,
    errors: Vec<RowError>,
}

/// Bảng tra danh mục của dự án: (bảng → [(id, mã, tên, cha)]).
struct Lookup {
    thbq: Vec<(Uuid, String)>,
    phong_ban: Vec<(Uuid, String, String)>,          // id, ma_don_vi, ten
    mang: Vec<(Uuid, Uuid, String, String)>,         // id, phong_ban_id, ma_mang, ten
    ngon_ngu: Vec<(String, String)>,                 // ma, ten
    loai_vb: Vec<(String, String)>,
}

fn s<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(Value::as_str).map(str::trim).filter(|x| !x.is_empty())
}
fn u(v: &Value, k: &str) -> Option<Uuid> {
    s(v, k).and_then(|x| Uuid::parse_str(x).ok())
}
fn eq_ci(a: &str, b: &str) -> bool {
    let norm = |s: &str| s.trim().to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
    norm(a) == norm(b)
}

async fn lookup(st: &AppState, project: Uuid, phong: Uuid) -> Result<Lookup, ApiError> {
    let c = cl::danh_muc::catalog(&st.pool, project).await?;
    let arr = |k: &str| c.get(k).and_then(Value::as_array).cloned().unwrap_or_default();
    let thbq = arr("thbq").iter().filter_map(|v| Some((u(v, "id")?, s(v, "ten")?.to_owned()))).collect();
    let phong_ban = arr("phong_ban")
        .iter()
        .filter(|v| u(v, "phong_id") == Some(phong))
        .filter_map(|v| Some((u(v, "id")?, s(v, "ma_don_vi").unwrap_or("").to_owned(), s(v, "ten")?.to_owned())))
        .collect();
    let mang = arr("mang").iter().filter_map(|v| Some((u(v, "id")?, u(v, "phong_ban_id")?, s(v, "ma_mang").unwrap_or("00").to_owned(), s(v, "ten")?.to_owned()))).collect();
    let ngon_ngu = arr("ngon_ngu").iter().filter_map(|v| Some((s(v, "ma")?.to_owned(), s(v, "ten")?.to_owned()))).collect();
    let loai_vb = arr("loai_van_ban").iter().filter_map(|v| Some((s(v, "ma")?.to_owned(), s(v, "ten")?.to_owned()))).collect();
    Ok(Lookup { thbq, phong_ban, mang, ngon_ngu, loai_vb })
}

/// Đổi tên/mã danh mục trong dòng hồ sơ sang id: `thbq` (tên, hoặc số năm "10"), `ma_don_vi`/`ten_phong_ban`,
/// `ma_mang`/`ten_mang`, `ngon_ngu` (tên → mã).
fn resolve_ho_so(lk: &Lookup, row: &mut Row) -> Result<(), RowError> {
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
                None => return Err(RowError { row: 0, field: Some("thbq".into()), message: format!("Thời hạn bảo quản {t:?} không có trong danh mục") }),
            }
        }
    }
    row.remove("thbq");
    let mut pb_id = u(&v, "phong_ban_id");
    if pb_id.is_none() {
        if let Some(t) = s(&v, "ma_don_vi").or(s(&v, "ten_phong_ban")) {
            match lk.phong_ban.iter().find(|(_, ma, ten)| eq_ci(ma, t) || eq_ci(ten, t)) {
                Some((id, _, _)) => {
                    pb_id = Some(*id);
                    row.insert("phong_ban_id".into(), json!(id));
                }
                None => return Err(RowError { row: 0, field: Some("ma_don_vi".into()), message: format!("Phòng ban {t:?} không có trong phông") }),
            }
        }
    }
    row.remove("ma_don_vi");
    row.remove("ten_phong_ban");
    if row.get("mang_id").map_or(true, Value::is_null) {
        if let Some(t) = s(&v, "ma_mang").or(s(&v, "ten_mang")) {
            match lk.mang.iter().find(|(_, pb, ma, ten)| pb_id.map_or(true, |p| *pb == p) && (eq_ci(ma, t) || eq_ci(ten, t))) {
                Some((id, pb, _, _)) => {
                    row.insert("mang_id".into(), json!(id));
                    if pb_id.is_none() {
                        row.insert("phong_ban_id".into(), json!(pb));
                    }
                }
                None => return Err(RowError { row: 0, field: Some("ma_mang".into()), message: format!("Mảng công việc {t:?} không có trong phòng ban") }),
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
    Ok(())
}

fn resolve_van_ban(lk: &Lookup, row: &mut Row) {
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

fn op_err(i: usize, r: &OpResult) -> Option<RowError> {
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
    let lk = lookup(&st, b.project_id, b.phong_id).await?;
    let parts = ctx_parts(&ts);
    let ctx = op_ctx(&ts, b.project_id, "import", &parts);
    let mut out = ImportResp::default();
    for (i, mut row) in b.rows.into_iter().enumerate() {
        let n = i + 1;
        row.insert("phong_id".into(), json!(b.phong_id));
        if let Err(mut e) = resolve_ho_so(&lk, &mut row) {
            e.row = n;
            out.errors.push(e);
            continue;
        }
        let so_tam = row.get("so_tam").and_then(Value::as_str).map(str::trim).unwrap_or("").to_owned();
        if so_tam.is_empty() {
            out.errors.push(RowError { row: n, field: Some("so_tam".into()), message: "Thiếu số tạm".into() });
            continue;
        }
        let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cl_ho_so WHERE phong_id = $1 AND lower(btrim(so_tam)) = lower(btrim($2)) AND deleted_at IS NULL")
            .bind(b.phong_id)
            .bind(&so_tam)
            .fetch_optional(&st.pool)
            .await?;
        let id = match (existing, b.mode.as_str()) {
            (Some(_), "create") => {
                out.errors.push(RowError { row: n, field: Some("so_tam".into()), message: format!("Số tạm {so_tam:?} đã có trong phông") });
                continue;
            }
            (Some(id), _) => id,
            (None, _) => Uuid::now_v7(),
        };
        let op = SyncOp { op_id: None, table: "cl_ho_so".into(), id, op: OpKind::Upsert, base: None, new: row, if_ver: None, edited_at: None };
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
    }
    digitext_db::audit::append(&st.pool, super::audit(&ts, "cl.import_ho_so", b.phong_id.to_string(), json!({"created": out.created, "updated": out.updated, "errors": out.errors.len()}))).await?;
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
