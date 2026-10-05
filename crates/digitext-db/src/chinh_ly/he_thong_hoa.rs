// Copyright (c) 2026 DigiText. All rights reserved.
//! Hệ thống hoá hồ sơ (chỉ online): lọc tập hồ sơ trong một phông, sắp theo 1 trong 6 thứ tự,
//! đánh số hồ sơ chính thức 1..n; gán hộp / bó / tập từ file "mục lục tạm" đã điền hoặc gán lẻ.
//! Bản cũ: đánh số qua Excel rồi nhập lại; hộp tạo theo (phòng ban, số hộp), dải từ/đến hồ sơ.

use super::lich_su;
use super::rows;
use super::sync::OpCtx;
use chrono::Datelike;
use digitext_core::chinh_ly::{natural_cmp, sort_items, SortItem, SortOrder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgConnection, PgPool, Row as _};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    #[serde(default)]
    pub phong_ban_id: Option<Uuid>,
    #[serde(default)]
    pub mang_ids: Vec<Uuid>,
    #[serde(default)]
    pub thbq_ids: Vec<Uuid>,
    /// VV | GL | HET
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub trung_thua: Option<bool>,
    /// Chỉ hồ sơ chưa có số chính thức.
    #[serde(default)]
    pub chua_danh_so: bool,
    /// Chỉ các hồ sơ này (chọn tay).
    #[serde(default)]
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRow {
    pub stt: i64,
    /// Loại bảo quản: true vĩnh viễn · false có thời hạn — mỗi loại đánh số riêng từ `start_at`.
    pub vinh_vien: bool,
    pub ho_so_id: Uuid,
    pub so_tam: String,
    pub tieu_de: String,
    pub ngay_bat_dau: Option<String>,
    pub ngay_ket_thuc: Option<String>,
    pub so_to: Option<String>,
    pub thbq_id: Option<Uuid>,
    pub ten_thbq: Option<String>,
    pub thbq_status: Option<String>,
    pub phong_ban_id: Option<Uuid>,
    pub ma_don_vi: Option<String>,
    pub ten_phong_ban: Option<String>,
    pub mang_id: Option<Uuid>,
    pub ma_mang: Option<String>,
    pub ten_mang: Option<String>,
    pub ghi_chu: Option<String>,
    pub ho_so_so_cu: Option<String>,
    pub hop_so: Option<String>,
    pub bo_so: Option<i64>,
    pub tap_so: Option<i64>,
    pub ma_ho_so_moi: Option<String>,
    pub ver: i64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub snapshot_ver: i64,
    pub order: i32,
    pub file_type: i32,
    pub rows: Vec<PreviewRow>,
}

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(Value::as_str).map(str::to_owned)
}
fn i(v: &Value, k: &str) -> Option<i64> {
    v.get(k).and_then(Value::as_i64)
}
fn u(v: &Value, k: &str) -> Option<Uuid> {
    s(v, k).and_then(|x| Uuid::parse_str(&x).ok())
}

/// Tập hồ sơ của phông theo bộ lọc (dòng view), chưa sắp.
async fn candidates(conn: &mut PgConnection, project: Uuid, phong: Uuid, f: &Filter) -> sqlx::Result<Vec<Value>> {
    let rows = sqlx::query(
        "SELECT to_jsonb(v) AS row FROM cl_ho_so_v v WHERE v.project_id = $1 AND v.phong_id = $2 AND v.deleted_at IS NULL \
           AND ($3::uuid IS NULL OR v.phong_ban_id = $3) \
           AND (cardinality($4::uuid[]) = 0 OR v.mang_id = ANY($4)) \
           AND (cardinality($5::uuid[]) = 0 OR v.thbq_id = ANY($5)) \
           AND ($6::text IS NULL OR v.thbq_status = $6) \
           AND ($7::bool IS NULL OR v.trung_thua = $7) \
           AND (NOT $8::bool OR v.ho_so_so IS NULL OR btrim(v.ho_so_so) = '') \
           AND (cardinality($9::uuid[]) = 0 OR v.id = ANY($9))",
    )
    .bind(project)
    .bind(phong)
    .bind(f.phong_ban_id)
    .bind(&f.mang_ids)
    .bind(&f.thbq_ids)
    .bind(f.status.as_deref())
    .bind(f.trung_thua)
    .bind(f.chua_danh_so)
    .bind(&f.ids)
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect())
}

fn build(conn_rows: Vec<Value>, order: SortOrder, file_type: i32, start_at: i64, taken: &HashMap<(bool, String), Uuid>) -> Preview {
    let mut items: Vec<SortItem<Value>> = conn_rows
        .into_iter()
        .map(|v| SortItem {
            thbq_sort: i(&v, "thbq_so_tt").unwrap_or(i64::MAX / 2) as i32,
            mang_sort: i(&v, "mang_so_tt").unwrap_or(i64::MAX / 2) as i32,
            mang_ten: s(&v, "ten_mang").unwrap_or_default(),
            ngay_kt: s(&v, "ngay_ket_thuc").and_then(|t| digitext_core::chinh_ly::ClDate::parse(&t).ok().flatten()).map(|d| d.sort_key()),
            so_tam: s(&v, "so_tam").unwrap_or_default(),
            key: v,
        })
        .collect();
    sort_items(&mut items, order);
    // Vĩnh viễn trước, có thời hạn sau (sort ổn định giữ thứ tự đã chọn trong mỗi loại); mỗi loại đánh từ start_at.
    items.sort_by_key(|it| !it.key.get("vinh_vien").and_then(Value::as_bool).unwrap_or(false));
    let mut snapshot_ver = 0i64;
    let mut rows = Vec::with_capacity(items.len());
    let (mut n_vv, mut n_cth) = (0i64, 0i64);
    for it in items.into_iter() {
        let v = it.key;
        let vinh_vien = v.get("vinh_vien").and_then(Value::as_bool).unwrap_or(false);
        let dem = if vinh_vien { &mut n_vv } else { &mut n_cth };
        let stt = start_at + *dem;
        *dem += 1;
        let id = u(&v, "id").unwrap_or_default();
        let ver = i(&v, "ver").unwrap_or(0);
        snapshot_ver = snapshot_ver.max(ver);
        let mut warnings = vec![];
        if s(&v, "ngay_ket_thuc").is_none() {
            warnings.push("Thiếu ngày kết thúc — không tạo được mã hồ sơ".into());
        }
        if s(&v, "phong_ban_id").is_none() {
            warnings.push("Chưa gán phòng ban — không gán được hộp".into());
        }
        if let Some(other) = taken.get(&(vinh_vien, stt.to_string())) {
            if *other != id {
                warnings.push(format!("Số hồ sơ {stt} đang thuộc hồ sơ khác ngoài tập"));
            }
        }
        let nam_kt = i(&v, "nam_ket_thuc").map(|n| n as i32);
        let mls = if vinh_vien { s(&v, "muc_luc_so") } else { s(&v, "muc_luc_so_cth").or_else(|| s(&v, "muc_luc_so")) };
        rows.push(PreviewRow {
            stt,
            vinh_vien,
            ho_so_id: id,
            so_tam: s(&v, "so_tam").unwrap_or_default(),
            tieu_de: s(&v, "tieu_de").unwrap_or_default(),
            ngay_bat_dau: s(&v, "ngay_bat_dau"),
            ngay_ket_thuc: s(&v, "ngay_ket_thuc"),
            so_to: s(&v, "so_to"),
            thbq_id: u(&v, "thbq_id"),
            ten_thbq: s(&v, "ten_thbq"),
            thbq_status: s(&v, "thbq_status"),
            phong_ban_id: u(&v, "phong_ban_id"),
            ma_don_vi: s(&v, "ma_don_vi"),
            ten_phong_ban: s(&v, "ten_phong_ban"),
            mang_id: u(&v, "mang_id"),
            ma_mang: s(&v, "ma_mang"),
            ten_mang: s(&v, "ten_mang"),
            ghi_chu: s(&v, "ghi_chu"),
            ho_so_so_cu: s(&v, "ho_so_so"),
            hop_so: s(&v, "hop_so"),
            bo_so: i(&v, "bo_so"),
            tap_so: i(&v, "tap_so"),
            ma_ho_so_moi: digitext_core::chinh_ly::ma_ho_so(s(&v, "ma_du_an").as_deref(), s(&v, "phong_so").as_deref(), nam_kt, Some(&stt.to_string()), mls.as_deref()),
            ver,
            warnings,
        });
    }
    Preview { snapshot_ver, order: order as i32, file_type, rows }
}

/// Số hồ sơ đã dùng trong phông (ngoài tập) theo loại → id, để cảnh báo trùng.
async fn taken_numbers(conn: &mut PgConnection, phong: Uuid) -> sqlx::Result<HashMap<(bool, String), Uuid>> {
    let rows = sqlx::query("SELECT id, vinh_vien, btrim(ho_so_so) AS so FROM cl_ho_so WHERE phong_id = $1 AND deleted_at IS NULL AND ho_so_so IS NOT NULL AND btrim(ho_so_so) <> ''")
        .bind(phong)
        .fetch_all(&mut *conn)
        .await?;
    Ok(rows.into_iter().filter_map(|r| Some(((r.try_get::<bool, _>("vinh_vien").ok()?, r.try_get::<String, _>("so").ok()?), r.try_get::<Uuid, _>("id").ok()?))).collect())
}

pub async fn preview(pool: &PgPool, project: Uuid, phong: Uuid, f: &Filter, order: SortOrder, file_type: i32, start_at: i64) -> sqlx::Result<Preview> {
    let mut conn = pool.acquire().await?;
    let cands = candidates(&mut conn, project, phong, f).await?;
    let taken = taken_numbers(&mut conn, phong).await?;
    Ok(build(cands, order, file_type, start_at.max(1), &taken))
}

#[derive(Debug)]
pub enum ApplyErr {
    Stale,
}

/// Đánh số: ghi `ho_so_so = stt`, `thu_tu_sap_xep = stt` cho tập đã xem trước (từ chối nếu tập đã đổi
/// sau `snapshot_ver`). Trả kết quả như xem trước (để desktop xuất mục lục tạm).
pub async fn apply(pool: &PgPool, ctx: &OpCtx<'_>, phong: Uuid, f: &Filter, order: SortOrder, file_type: i32, start_at: i64, snapshot_ver: i64) -> sqlx::Result<Result<Preview, ApplyErr>> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(cl_lock_key($1))").bind(ctx.project_id).execute(&mut *tx).await?;
    let cands = candidates(&mut tx, ctx.project_id, phong, f).await?;
    let taken = taken_numbers(&mut tx, phong).await?;
    let p = build(cands, order, file_type, start_at.max(1), &taken);
    if p.snapshot_ver != snapshot_ver {
        return Ok(Err(ApplyErr::Stale));
    }
    for r in &p.rows {
        let before = rows::get(&mut tx, "cl_ho_so", r.ho_so_id).await?.unwrap_or_default();
        let new_so = r.stt.to_string();
        if before.get("ho_so_so").and_then(Value::as_str) == Some(new_so.as_str()) && before.get("thu_tu_sap_xep").and_then(Value::as_i64) == Some(r.stt) {
            continue;
        }
        sqlx::query("UPDATE cl_ho_so SET ho_so_so = $2, thu_tu_sap_xep = $3, updated_by = $4 WHERE id = $1")
            .bind(r.ho_so_id)
            .bind(&new_so)
            .bind(r.stt as i32)
            .bind(ctx.user_id)
            .execute(&mut *tx)
            .await?;
        let after = rows::get(&mut tx, "cl_ho_so", r.ho_so_id).await?.unwrap_or_default();
        let (diffs, n_new, n_edited) = lich_su::diff_rows(Some(&before), &after, &["ho_so_so", "thu_tu_sap_xep"]);
        lich_su::insert(
            &mut tx,
            lich_su::NewEntry {
                project_id: ctx.project_id,
                entity: "ho_so",
                entity_id: r.ho_so_id,
                ho_so_id: r.ho_so_id,
                phong_id: Some(phong),
                action: "systematize",
                actor_id: Some(ctx.user_id),
                actor_name: Some(ctx.user_name),
                at_client: None,
                source: "online",
                title: Some(&r.tieu_de),
                diffs,
                n_new,
                n_edited,
            },
        )
        .await?;
    }
    // Trả bản mới (ver đã đổi).
    let cands = candidates(&mut tx, ctx.project_id, phong, f).await?;
    let taken = taken_numbers(&mut tx, phong).await?;
    let out = build(cands, order, file_type, start_at.max(1), &taken);
    tx.commit().await?;
    Ok(Ok(out))
}

// ------------------------------------------------------------------ gán hộp / bó / tập

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoxRow {
    pub ho_so_id: Uuid,
    #[serde(default)]
    pub hop_so: Option<String>,
    #[serde(default)]
    pub bo_so: Option<i64>,
    #[serde(default)]
    pub tap_so: Option<i64>,
    /// Xoá gán (hộp / bó / tập về trống).
    #[serde(default)]
    pub clear: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowError {
    pub row: usize,
    pub ho_so_id: Uuid,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoxResult {
    pub applied: usize,
    pub hops_created: usize,
    pub bo_tap_created: usize,
    pub errors: Vec<RowError>,
}

/// Hộp xác định bởi (phòng ban, loại vĩnh viễn / có thời hạn, số hộp) — migration 0015: mỗi loại đánh hộp từ 01.
pub async fn find_or_create_hop(tx: &mut PgConnection, project: Uuid, phong_ban: Uuid, vinh_vien: bool, hop_so: &str, by: Uuid, created: &mut usize) -> sqlx::Result<Uuid> {
    let found: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cl_hop WHERE phong_ban_id = $1 AND vinh_vien = $3 AND lower(btrim(hop_so)) = lower(btrim($2)) AND deleted_at IS NULL")
        .bind(phong_ban)
        .bind(hop_so)
        .bind(vinh_vien)
        .fetch_optional(&mut *tx)
        .await?;
    if let Some(id) = found {
        return Ok(id);
    }
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO cl_hop (id, project_id, phong_ban_id, hop_so, vinh_vien, created_by, updated_by) VALUES ($1, $2, $3, btrim($4), $6, $5, $5)")
        .bind(id)
        .bind(project)
        .bind(phong_ban)
        .bind(hop_so)
        .bind(by)
        .bind(vinh_vien)
        .execute(&mut *tx)
        .await?;
    *created += 1;
    Ok(id)
}

async fn find_or_create_bo_tap(tx: &mut PgConnection, project: Uuid, phong_ban: Uuid, loai: &str, so: i64, by: Uuid, created: &mut usize) -> sqlx::Result<Uuid> {
    let found: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cl_bo_tap WHERE phong_ban_id = $1 AND loai = $2 AND so = $3 AND deleted_at IS NULL")
        .bind(phong_ban)
        .bind(loai)
        .bind(so as i32)
        .fetch_optional(&mut *tx)
        .await?;
    if let Some(id) = found {
        return Ok(id);
    }
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO cl_bo_tap (id, project_id, phong_ban_id, loai, so, created_by, updated_by) VALUES ($1, $2, $3, $4, $5, $6, $6)")
        .bind(id)
        .bind(project)
        .bind(phong_ban)
        .bind(loai)
        .bind(so as i32)
        .bind(by)
        .execute(&mut *tx)
        .await?;
    *created += 1;
    Ok(id)
}

/// Tính lại từ hồ sơ / đến hồ sơ của các hộp. Loại (vinh_vien) là danh tính của hộp — không suy lại từ hồ sơ.
pub async fn refresh_hops(tx: &mut PgConnection, hop_ids: &[Uuid]) -> sqlx::Result<()> {
    for h in hop_ids {
        let rs = sqlx::query("SELECT h.ho_so_so FROM cl_ho_so h WHERE h.hop_id = $1 AND h.deleted_at IS NULL")
            .bind(h)
            .fetch_all(&mut *tx)
            .await?;
        let mut sos: Vec<String> = rs.iter().filter_map(|r| r.try_get::<Option<String>, _>("ho_so_so").ok().flatten()).map(|x| x.trim().to_owned()).filter(|x| !x.is_empty()).collect();
        sos.sort_by(|a, b| natural_cmp(a, b));
        sqlx::query("UPDATE cl_hop SET tu_ho_so = $2, den_ho_so = $3 WHERE id = $1")
            .bind(h)
            .bind(sos.first())
            .bind(sos.last())
            .execute(&mut *tx)
            .await?;
    }
    Ok(())
}

/// Gán hộp (file_type 1) hoặc bó/tập (file_type 2) cho nhiều hồ sơ; lỗi trả theo dòng, dòng hợp lệ vẫn áp.
pub async fn boxing(pool: &PgPool, ctx: &OpCtx<'_>, file_type: i32, rows_in: &[BoxRow]) -> sqlx::Result<BoxResult> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(cl_lock_key($1))").bind(ctx.project_id).execute(&mut *tx).await?;
    let mut out = BoxResult::default();
    let mut touched: BTreeMap<Uuid, ()> = BTreeMap::new();
    for (idx, r) in rows_in.iter().enumerate() {
        let n = idx + 1;
        let Some(before) = rows::get(&mut tx, "cl_ho_so", r.ho_so_id).await? else {
            out.errors.push(RowError { row: n, ho_so_id: r.ho_so_id, message: "Không thấy hồ sơ".into() });
            continue;
        };
        if rows::uuid_of(&before, "project_id") != Some(ctx.project_id) || !before.get("deleted_at").map_or(true, Value::is_null) {
            out.errors.push(RowError { row: n, ho_so_id: r.ho_so_id, message: "Hồ sơ không thuộc dự án hoặc đã xoá".into() });
            continue;
        }
        let Some(pb) = rows::uuid_of(&before, "phong_ban_id") else {
            out.errors.push(RowError { row: n, ho_so_id: r.ho_so_id, message: "Hồ sơ chưa gán phòng ban (hộp / bó / tập thuộc phòng ban)".into() });
            continue;
        };
        if let Some(old) = rows::uuid_of(&before, "hop_id") {
            touched.insert(old, ());
        }
        let mut patch = serde_json::Map::new();
        if r.clear {
            patch.insert("hop_id".into(), Value::Null);
            patch.insert("bo_id".into(), Value::Null);
            patch.insert("tap_id".into(), Value::Null);
        } else if file_type == 1 {
            let Some(hs) = r.hop_so.as_deref().map(str::trim).filter(|x| !x.is_empty()) else {
                out.errors.push(RowError { row: n, ho_so_id: r.ho_so_id, message: "Thiếu số hộp".into() });
                continue;
            };
            let vv = before.get("vinh_vien").and_then(Value::as_bool).unwrap_or(false);
            let hid = find_or_create_hop(&mut tx, ctx.project_id, pb, vv, hs, ctx.user_id, &mut out.hops_created).await?;
            touched.insert(hid, ());
            patch.insert("hop_id".into(), json!(hid));
        } else {
            if r.bo_so.is_none() && r.tap_so.is_none() {
                out.errors.push(RowError { row: n, ho_so_id: r.ho_so_id, message: "Thiếu số bó / số tập".into() });
                continue;
            }
            if let Some(b) = r.bo_so {
                if b < 1 {
                    out.errors.push(RowError { row: n, ho_so_id: r.ho_so_id, message: "Số bó phải là số nguyên dương".into() });
                    continue;
                }
                let id = find_or_create_bo_tap(&mut tx, ctx.project_id, pb, "bo", b, ctx.user_id, &mut out.bo_tap_created).await?;
                patch.insert("bo_id".into(), json!(id));
            }
            if let Some(t) = r.tap_so {
                if t < 1 {
                    out.errors.push(RowError { row: n, ho_so_id: r.ho_so_id, message: "Số tập phải là số nguyên dương".into() });
                    continue;
                }
                let id = find_or_create_bo_tap(&mut tx, ctx.project_id, pb, "tap", t, ctx.user_id, &mut out.bo_tap_created).await?;
                patch.insert("tap_id".into(), json!(id));
            }
        }
        patch.insert("updated_by".into(), json!(ctx.user_id));
        let sp = digitext_core::chinh_ly::spec::spec("cl_ho_so").expect("spec");
        let after = rows::update(&mut tx, sp, r.ho_so_id, &patch).await?;
        let (diffs, n_new, n_edited) = lich_su::diff_rows(Some(&before), &after, &["hop_id", "bo_id", "tap_id"]);
        if !diffs.is_empty() {
            lich_su::insert(
                &mut tx,
                lich_su::NewEntry {
                    project_id: ctx.project_id,
                    entity: "ho_so",
                    entity_id: r.ho_so_id,
                    ho_so_id: r.ho_so_id,
                    phong_id: rows::uuid_of(&before, "phong_id"),
                    action: "boxing",
                    actor_id: Some(ctx.user_id),
                    actor_name: Some(ctx.user_name),
                    at_client: None,
                    source: "online",
                    title: rows::str_of(&before, "tieu_de"),
                    diffs,
                    n_new,
                    n_edited,
                },
            )
            .await?;
        }
        out.applied += 1;
    }
    let ids: Vec<Uuid> = touched.keys().copied().collect();
    refresh_hops(&mut tx, &ids).await?;
    tx.commit().await?;
    Ok(out)
}

/// Năm hiện tại (giờ Việt Nam) cho trạng thái THBQ.
pub fn nam_now() -> i32 {
    (chrono::Utc::now() + chrono::Duration::hours(7)).year()
}
