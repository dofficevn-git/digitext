// Copyright (c) 2026 DigiText. All rights reserved.
//! Cầu nối bảng SQLite cũ (`dossier`, `van_ban`, `phong_ban`, `mang_cong_viec`, `thoi_han_bq`, `kho`, `gia`)
//! với bản sao `cl_*`:
//! - **Chiếu** (`project_row`): mỗi dòng cl_* kéo về / ghi cục bộ được ghi lại vào bảng cũ (khoá `cl_id`), để các
//!   màn Số hoá (Tạo lập CSDL, Đóng gói, lọc OCR theo phòng ban / THBQ, ghép PDF quét) chạy như trước.
//! - **Chuyển tiếp** (`forward_*`, `push_unmigrated`): lệnh ghi cũ vẫn ghi bảng cũ rồi đẩy sang cl_* (tạo mới nếu
//!   dòng chưa có `cl_id`) → lên máy chủ qua outbox. Đây cũng là "wizard chuyển dữ liệu cũ".
//! `pdf_path` của văn bản là dữ liệu máy, không chiếu và không đẩy.

use super::store::{self, Row};
use digitext_core::chinh_ly::spec;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::{json, Value};

/// Bảng cũ + cột nhận `cl_id`.
pub const LEGACY_TABLES: &[&str] = &["dossier", "van_ban", "phong_ban", "mang_cong_viec", "thoi_han_bq", "kho", "gia"];

pub fn ensure_columns(conn: &Connection) -> rusqlite::Result<()> {
    for t in LEGACY_TABLES {
        let has: bool = {
            let mut st = conn.prepare(&format!("PRAGMA table_info(\"{t}\")"))?;
            let cols: Vec<String> = st.query_map([], |r| r.get::<_, String>(1))?.collect::<rusqlite::Result<_>>()?;
            cols.iter().any(|c| c == "cl_id")
        };
        if !has {
            conn.execute_batch(&format!("ALTER TABLE {t} ADD COLUMN cl_id TEXT;"))?;
        }
        conn.execute_batch(&format!("CREATE UNIQUE INDEX IF NOT EXISTS ux_{t}_cl_id ON {t} (cl_id) WHERE cl_id IS NOT NULL;"))?;
    }
    Ok(())
}

fn s(r: &Row, k: &str) -> Option<String> {
    match r.get(k) {
        Some(Value::String(v)) => Some(v.clone()).filter(|v| !v.trim().is_empty()),
        Some(Value::Number(n)) => Some(n.to_string()),
        Some(Value::Bool(b)) => Some(b.to_string()),
        _ => None,
    }
}
fn i(r: &Row, k: &str) -> Option<i64> {
    match r.get(k) {
        Some(Value::Number(n)) => n.as_i64(),
        Some(Value::String(v)) => v.trim().parse().ok(),
        _ => None,
    }
}
fn b(r: &Row, k: &str) -> bool {
    match r.get(k) {
        Some(Value::Bool(v)) => *v,
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0) != 0,
        Some(Value::String(v)) => matches!(v.as_str(), "1" | "true"),
        _ => false,
    }
}
fn deleted(r: &Row) -> bool {
    !r.get("deleted_at").map_or(true, Value::is_null)
}

fn cl_row(conn: &Connection, table: &str, id: &str) -> rusqlite::Result<Option<Row>> {
    Ok(store::get(conn, table, id)?.map(|e| e.row))
}

fn legacy_id(conn: &Connection, table: &str, cl_id: &str) -> rusqlite::Result<Option<i64>> {
    conn.query_row(&format!("SELECT rowid FROM {table} WHERE cl_id = ?1"), params![cl_id], |r| r.get(0)).optional()
}

// ------------------------------------------------------------------ chiếu cl_* → bảng cũ

/// Chiếu một dòng cl_* sang bảng cũ tương ứng (bỏ qua bảng không có bản cũ).
pub fn project_row(conn: &Connection, table: &str, row: &Row) -> rusqlite::Result<()> {
    match table {
        "cl_phong" => project_phong(conn, row),
        "cl_thbq" => project_thbq(conn, row),
        "cl_phong_ban" => project_phong_ban(conn, row),
        "cl_mang" => project_mang(conn, row),
        "cl_kho" => project_kho(conn, row),
        "cl_gia" => project_gia(conn, row),
        "cl_ho_so" => project_ho_so(conn, row),
        "cl_van_ban" => project_van_ban(conn, row),
        "cl_hop" | "cl_bo_tap" => {
            // số hộp / bó / tập nằm trên dossier → chiếu lại các hồ sơ dùng nó
            let col = if table == "cl_hop" { "hop_id" } else { "bo_id" };
            let ids: Vec<String> = {
                let mut st = conn.prepare(&format!("SELECT id FROM cl_ho_so WHERE {col} = ?1 OR tap_id = ?1"))?;
                let v = st.query_map(params![s(row, "id").unwrap_or_default()], |r| r.get(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
                v
            };
            for id in ids {
                if let Some(h) = cl_row(conn, "cl_ho_so", &id)? {
                    project_ho_so(conn, &h)?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Chiếu một lô dòng (cha trước con theo rank của spec).
pub fn project_rows(conn: &Connection, rows: &[(String, Row)]) -> rusqlite::Result<()> {
    let mut v: Vec<&(String, Row)> = rows.iter().collect();
    v.sort_by_key(|(t, _)| spec::spec(t).map(|s| s.rank).unwrap_or(200));
    for (t, r) in v {
        project_row(conn, t, r)?;
    }
    Ok(())
}

/// Xoá dòng cũ mà bản cl_* tương ứng đã xoá / không còn (sau xoá cục bộ hay kéo tombstone).
pub fn prune_legacy(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM van_ban WHERE cl_id IS NOT NULL AND cl_id NOT IN (SELECT id FROM cl_van_ban WHERE deleted_at IS NULL)", [])?;
    conn.execute("DELETE FROM dossier WHERE cl_id IS NOT NULL AND cl_id NOT IN (SELECT id FROM cl_ho_so WHERE deleted_at IS NULL)", [])?;
    Ok(())
}

/// Ngược lại: dòng cl_* còn sống mà dòng cũ đã bị lệnh cũ xoá → xoá cục bộ (đẩy lên máy chủ).
pub fn reconcile_deleted(conn: &Connection, who: &store::Who) -> rusqlite::Result<usize> {
    let mut n = 0;
    for (cl, legacy, sp) in [("cl_van_ban", "van_ban", "cl_van_ban"), ("cl_ho_so", "dossier", "cl_ho_so")] {
        let ids: Vec<String> = {
            let mut st = conn.prepare(&format!("SELECT id FROM {cl} WHERE deleted_at IS NULL AND project_id = ?1 AND id NOT IN (SELECT cl_id FROM {legacy} WHERE cl_id IS NOT NULL)"))?;
            let v = st.query_map(params![who.project_id], |r| r.get(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
            v
        };
        for id in ids {
            n += store::delete_local(conn, spec::spec(sp).expect("spec"), &id, who)? as usize;
        }
    }
    Ok(n)
}

/// Chiếu lại toàn bộ dự án (sau khi kéo lại từ đầu hoặc chạy wizard).
pub fn project_all(conn: &Connection, project_id: &str) -> rusqlite::Result<usize> {
    let mut n = 0;
    for t in ["cl_phong", "cl_thbq", "cl_phong_ban", "cl_mang", "cl_kho", "cl_gia", "cl_ho_so", "cl_van_ban"] {
        let (rows, _) = store::list(conn, t, project_id, &store::ListFilter { include_deleted: true, limit: Some(500_000), ..Default::default() })?;
        for r in rows {
            project_row(conn, t, &r)?;
            n += 1;
        }
    }
    Ok(n)
}

fn project_phong(conn: &Connection, r: &Row) -> rusqlite::Result<()> {
    if deleted(r) {
        return Ok(());
    }
    if let (Some(ma), Some(ten)) = (s(r, "ma_du_an"), s(r, "ten")) {
        crate::db::f_2o6(conn, &ma, &ten, None, s(r, "phong_so").as_deref(), s(r, "muc_luc_so").as_deref())?;
    }
    Ok(())
}

fn project_thbq(conn: &Connection, r: &Row) -> rusqlite::Result<()> {
    let id = s(r, "id").unwrap_or_default();
    let ten = s(r, "ten").unwrap_or_default();
    if deleted(r) {
        conn.execute("DELETE FROM thoi_han_bq WHERE cl_id = ?1", params![id])?;
        return Ok(());
    }
    let tt = i(r, "trang_thai").unwrap_or(2);
    let gt = i(r, "gia_tri").unwrap_or(0);
    let sort = i(r, "so_tt").unwrap_or(0);
    if let Some(lid) = legacy_id(conn, "thoi_han_bq", &id)? {
        conn.execute("UPDATE thoi_han_bq SET ten_thbq = ?2, trang_thai = ?3, gia_tri = ?4, sort = ?5 WHERE thbqid = ?1", params![lid, ten, tt, gt, sort])?;
        return Ok(());
    }
    // gắn vào dòng cũ cùng tên (chưa có cl_id) — tránh nhân đôi
    let same: Option<i64> = conn.query_row("SELECT thbqid FROM thoi_han_bq WHERE cl_id IS NULL AND lower(trim(ten_thbq)) = lower(trim(?1))", params![ten], |r| r.get(0)).optional()?;
    match same {
        Some(lid) => {
            conn.execute("UPDATE thoi_han_bq SET cl_id = ?2, trang_thai = ?3, gia_tri = ?4, sort = ?5 WHERE thbqid = ?1", params![lid, id, tt, gt, sort])?;
        }
        None => {
            conn.execute("INSERT INTO thoi_han_bq (ten_thbq, trang_thai, gia_tri, sort, cl_id) VALUES (?1, ?2, ?3, ?4, ?5)", params![ten, tt, gt, sort, id])?;
        }
    }
    Ok(())
}

fn ma_phong_of(conn: &Connection, phong_id: &str) -> rusqlite::Result<Option<String>> {
    Ok(cl_row(conn, "cl_phong", phong_id)?.and_then(|p| s(&p, "ma_du_an")))
}

fn project_phong_ban(conn: &Connection, r: &Row) -> rusqlite::Result<()> {
    let id = s(r, "id").unwrap_or_default();
    if deleted(r) {
        conn.execute("DELETE FROM phong_ban WHERE cl_id = ?1", params![id])?;
        return Ok(());
    }
    let Some(ma_phong) = s(r, "phong_id").and_then(|p| ma_phong_of(conn, &p).ok().flatten()) else { return Ok(()) };
    // đảm bảo bảng phong có dòng (FK)
    let exists: i64 = conn.query_row("SELECT count(*) FROM phong WHERE ma_du_an = ?1", params![ma_phong], |r| r.get(0))?;
    if exists == 0 {
        if let Some(p) = s(r, "phong_id").and_then(|p| cl_row(conn, "cl_phong", &p).ok().flatten()) {
            project_phong(conn, &p)?;
        }
    }
    let ma_dv: String = conn.query_row("SELECT ma_dv FROM phong_ban WHERE cl_id = ?1", params![id], |r| r.get(0)).optional()?.unwrap_or_else(|| id.clone());
    conn.execute(
        "INSERT INTO phong_ban (ma_dv, ma_phong, ma_don_vi, ten_don_vi, so_tt, cl_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(ma_dv) DO UPDATE SET ma_phong = excluded.ma_phong, ma_don_vi = excluded.ma_don_vi, ten_don_vi = excluded.ten_don_vi, so_tt = excluded.so_tt, cl_id = excluded.cl_id",
        params![ma_dv, ma_phong, s(r, "ma_don_vi"), s(r, "ten").unwrap_or_default(), i(r, "so_tt"), id],
    )?;
    Ok(())
}

fn project_mang(conn: &Connection, r: &Row) -> rusqlite::Result<()> {
    let id = s(r, "id").unwrap_or_default();
    if deleted(r) {
        conn.execute("DELETE FROM mang_cong_viec WHERE cl_id = ?1", params![id])?;
        return Ok(());
    }
    let Some(pb) = s(r, "phong_ban_id") else { return Ok(()) };
    let Some(ma_dv): Option<String> = conn.query_row("SELECT ma_dv FROM phong_ban WHERE cl_id = ?1", params![pb], |r| r.get(0)).optional()? else { return Ok(()) };
    match legacy_id(conn, "mang_cong_viec", &id)? {
        Some(lid) => {
            conn.execute("UPDATE mang_cong_viec SET ma_mang = ?2, ten_mang = ?3, ma_dv = ?4, so_tt = ?5 WHERE mang_cvid = ?1", params![lid, s(r, "ma_mang"), s(r, "ten").unwrap_or_default(), ma_dv, i(r, "so_tt")])?;
        }
        None => {
            conn.execute("INSERT INTO mang_cong_viec (ma_mang, ten_mang, ma_dv, so_tt, cl_id) VALUES (?1, ?2, ?3, ?4, ?5)", params![s(r, "ma_mang"), s(r, "ten").unwrap_or_default(), ma_dv, i(r, "so_tt"), id])?;
        }
    }
    Ok(())
}

fn project_kho(conn: &Connection, r: &Row) -> rusqlite::Result<()> {
    let id = s(r, "id").unwrap_or_default();
    if deleted(r) {
        conn.execute("DELETE FROM kho WHERE cl_id = ?1", params![id])?;
        return Ok(());
    }
    match legacy_id(conn, "kho", &id)? {
        Some(lid) => {
            conn.execute("UPDATE kho SET ten_kho = ?2, so_tt = ?3, ghi_chu = ?4 WHERE ma_kho = ?1", params![lid, s(r, "ten").unwrap_or_default(), i(r, "so_tt"), s(r, "ghi_chu")])?;
        }
        None => {
            conn.execute("INSERT INTO kho (ten_kho, so_tt, ghi_chu, cl_id) VALUES (?1, ?2, ?3, ?4)", params![s(r, "ten").unwrap_or_default(), i(r, "so_tt"), s(r, "ghi_chu"), id])?;
        }
    }
    Ok(())
}

fn project_gia(conn: &Connection, r: &Row) -> rusqlite::Result<()> {
    let id = s(r, "id").unwrap_or_default();
    if deleted(r) {
        conn.execute("DELETE FROM gia WHERE cl_id = ?1", params![id])?;
        return Ok(());
    }
    let Some(kho_lid) = s(r, "kho_id").and_then(|k| legacy_id(conn, "kho", &k).ok().flatten()) else { return Ok(()) };
    let ma_dv: Option<String> = match s(r, "phong_ban_id") {
        Some(pb) => conn.query_row("SELECT ma_dv FROM phong_ban WHERE cl_id = ?1", params![pb], |r| r.get(0)).optional()?,
        None => None,
    };
    let ma_da: Option<String> = match ma_dv.as_deref() {
        Some(dv) => conn.query_row("SELECT ma_phong FROM phong_ban WHERE ma_dv = ?1", params![dv], |r| r.get(0)).optional()?,
        None => None,
    };
    match legacy_id(conn, "gia", &id)? {
        Some(lid) => {
            conn.execute("UPDATE gia SET ten_gia = ?2, ma_kho = ?3, so_tt = ?4, ma_dv = ?5, tu_hop = ?6, den_hop = ?7, ghi_chu = ?8, ma_da = ?9 WHERE ma_gia = ?1", params![lid, s(r, "ten").unwrap_or_default(), kho_lid, i(r, "so_tt"), ma_dv, i(r, "tu_hop"), i(r, "den_hop"), s(r, "ghi_chu"), ma_da])?;
        }
        None => {
            conn.execute("INSERT INTO gia (ten_gia, ma_kho, so_tt, ma_dv, tu_hop, den_hop, ghi_chu, ma_da, cl_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", params![s(r, "ten").unwrap_or_default(), kho_lid, i(r, "so_tt"), ma_dv, i(r, "tu_hop"), i(r, "den_hop"), s(r, "ghi_chu"), ma_da, id])?;
        }
    }
    Ok(())
}

fn project_ho_so(conn: &Connection, r: &Row) -> rusqlite::Result<()> {
    let id = s(r, "id").unwrap_or_default();
    if deleted(r) {
        conn.execute("DELETE FROM dossier WHERE cl_id = ?1", params![id])?;
        return Ok(());
    }
    let phong = s(r, "phong_id").and_then(|p| cl_row(conn, "cl_phong", &p).ok().flatten());
    let ma_da = phong.as_ref().and_then(|p| s(p, "ma_du_an"));
    let ma_dv: Option<String> = match s(r, "phong_ban_id") {
        Some(pb) => conn.query_row("SELECT ma_dv FROM phong_ban WHERE cl_id = ?1", params![pb], |r| r.get(0)).optional()?,
        None => None,
    };
    let mang = match s(r, "mang_id") {
        Some(m) => legacy_id(conn, "mang_cong_viec", &m)?,
        None => None,
    };
    let thbq = match s(r, "thbq_id") {
        Some(t) => legacy_id(conn, "thoi_han_bq", &t)?,
        None => None,
    };
    let hop_so = match s(r, "hop_id") {
        Some(h) => cl_row(conn, "cl_hop", &h)?.and_then(|x| s(&x, "hop_so")),
        None => None,
    };
    let bo_so = match s(r, "bo_id") {
        Some(h) => cl_row(conn, "cl_bo_tap", &h)?.and_then(|x| s(&x, "so")),
        None => None,
    };
    let tap_so = match s(r, "tap_id") {
        Some(h) => cl_row(conn, "cl_bo_tap", &h)?.and_then(|x| s(&x, "so")),
        None => None,
    };
    let ma_gia = match s(r, "gia_id") {
        Some(g) => legacy_id(conn, "gia", &g)?,
        None => None,
    };
    let nam_kt = s(r, "ngay_ket_thuc").and_then(|t| digitext_core::chinh_ly::ClDate::parse(&t).ok().flatten()).map(|d| d.year);
    let ma_ho_so = digitext_core::chinh_ly::ma_ho_so(ma_da.as_deref(), phong.as_ref().and_then(|p| s(p, "phong_so")).as_deref(), nam_kt, s(r, "ho_so_so").as_deref(), phong.as_ref().and_then(|p| s(p, "muc_luc_so")).as_deref());
    let trang_thai = if s(r, "ho_so_so").is_some() { "final" } else { "temp" };
    let existing = legacy_id(conn, "dossier", &id)?;
    let p = params![
        ma_da,
        ma_dv,
        mang,
        thbq,
        s(r, "so_tam").unwrap_or_default(),
        ma_ho_so,
        s(r, "tieu_de").unwrap_or_default(),
        s(r, "ngay_bat_dau"),
        s(r, "ngay_ket_thuc"),
        s(r, "so_to"),
        b(r, "trung_thua") as i64,
        i(r, "tong_so_vb"),
        i(r, "so_luong_trang"),
        s(r, "che_do_su_dung"),
        s(r, "ngon_ngu"),
        s(r, "tinh_trang_vat_ly"),
        s(r, "ghi_chu"),
        hop_so,
        bo_so,
        tap_so,
        trang_thai,
        i(r, "thu_tu_sap_xep"),
        ma_gia,
        s(r, "created_by"),
        s(r, "updated_by"),
        id,
    ];
    match existing {
        Some(lid) => {
            let mut st = conn.prepare(
                "UPDATE dossier SET ma_da=?1, ma_dv=?2, mang_cvid=?3, thbq=?4, so_tam=?5, ma_ho_so=?6, tieu_de=?7, ngay_bat_dau=?8, ngay_ket_thuc=?9, so_to=?10, trung_thua=?11,
                    tong_so_van_ban=?12, so_luong_trang=?13, che_do_su_dung=?14, ngon_ngu=?15, tinh_trang_vat_ly=?16, ghi_chu=?17, hop_so=?18, bo_so=?19, tap_so=?20, trang_thai=?21,
                    thu_tu_sap_xep=?22, ma_gia=?23, created_by=COALESCE(created_by, ?24), updated_by=?25, updated_at=datetime('now') WHERE cl_id=?26 AND rowid=?27",
            )?;
            st.execute(rusqlite::params_from_iter(p.iter().map(|x| *x).chain(std::iter::once(&lid as &dyn rusqlite::ToSql))))?;
        }
        None => {
            // tránh trùng số tạm với dòng cũ chưa gắn cl_id: gắn luôn vào dòng đó
            let orphan: Option<i64> = conn
                .query_row("SELECT id FROM dossier WHERE cl_id IS NULL AND ma_da IS ?1 AND lower(trim(so_tam)) = lower(trim(?2))", params![ma_da, s(r, "so_tam").unwrap_or_default()], |r| r.get(0))
                .optional()?;
            match orphan {
                Some(lid) => {
                    conn.execute("UPDATE dossier SET cl_id = ?2 WHERE id = ?1", params![lid, id])?;
                    return project_ho_so(conn, r);
                }
                None => {
                    let mut st = conn.prepare(
                        "INSERT INTO dossier (ma_da, ma_dv, mang_cvid, thbq, so_tam, ma_ho_so, tieu_de, ngay_bat_dau, ngay_ket_thuc, so_to, trung_thua, tong_so_van_ban, so_luong_trang,
                            che_do_su_dung, ngon_ngu, tinh_trang_vat_ly, ghi_chu, hop_so, bo_so, tap_so, trang_thai, thu_tu_sap_xep, ma_gia, created_by, updated_by, cl_id)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26)",
                    )?;
                    st.execute(p)?;
                }
            }
        }
    }
    Ok(())
}

fn project_van_ban(conn: &Connection, r: &Row) -> rusqlite::Result<()> {
    let id = s(r, "id").unwrap_or_default();
    if deleted(r) {
        conn.execute("DELETE FROM van_ban WHERE cl_id = ?1", params![id])?;
        return Ok(());
    }
    let Some(dossier_id) = s(r, "ho_so_id").and_then(|h| legacy_id(conn, "dossier", &h).ok().flatten()) else { return Ok(()) };
    let loai = s(r, "loai_vb").map(|ma| {
        conn.query_row("SELECT ten FROM cl_loai_van_ban WHERE ma = ?1 AND deleted_at IS NULL LIMIT 1", params![ma], |x| x.get::<_, String>(0)).optional().ok().flatten().unwrap_or(ma)
    });
    let p = params![
        dossier_id,
        i(r, "so_thu_tu").unwrap_or(0),
        loai,
        [s(r, "so_van_ban"), s(r, "so_ky_hieu")].into_iter().flatten().collect::<Vec<_>>().join("/"),
        s(r, "ngay_thang"),
        s(r, "co_quan_ban_hanh"),
        s(r, "trich_yeu"),
        i(r, "to_so"),
        i(r, "so_trang"),
        i(r, "trang_so"),
        s(r, "ngon_ngu"),
        s(r, "che_do_su_dung"),
        s(r, "do_tin_cay"),
        s(r, "but_tich"),
        s(r, "tinh_trang_vat_ly"),
        s(r, "tai_lieu_so"),
        s(r, "ghi_chu"),
        s(r, "created_by"),
        id,
    ];
    match legacy_id(conn, "van_ban", &id)? {
        Some(lid) => {
            let mut st = conn.prepare(
                "UPDATE van_ban SET dossier_id=?1, thu_tu=?2, loai_van_ban=?3, so_ky_hieu=?4, ngay_thang=?5, co_quan_ban_hanh=?6, trich_yeu=?7, to_so=?8, so_trang=?9, trang_so=?10,
                    ngon_ngu=?11, che_do_su_dung=?12, do_tin_cay=?13, but_tich=?14, tinh_trang_vat_ly=?15, tai_lieu_so=?16, ghi_chu=?17, created_by=COALESCE(created_by, ?18), updated_at=datetime('now') WHERE cl_id=?19 AND rowid=?20",
            )?;
            st.execute(rusqlite::params_from_iter(p.iter().map(|x| *x).chain(std::iter::once(&lid as &dyn rusqlite::ToSql))))?;
        }
        None => {
            let orphan: Option<i64> = conn.query_row("SELECT id FROM van_ban WHERE cl_id IS NULL AND dossier_id = ?1 AND thu_tu = ?2", params![dossier_id, i(r, "so_thu_tu").unwrap_or(0)], |r| r.get(0)).optional()?;
            match orphan {
                Some(lid) => {
                    conn.execute("UPDATE van_ban SET cl_id = ?2 WHERE id = ?1", params![lid, id])?;
                    return project_van_ban(conn, r);
                }
                None => {
                    let mut st = conn.prepare(
                        "INSERT INTO van_ban (dossier_id, thu_tu, loai_van_ban, so_ky_hieu, ngay_thang, co_quan_ban_hanh, trich_yeu, to_so, so_trang, trang_so, ngon_ngu, che_do_su_dung,
                            do_tin_cay, but_tich, tinh_trang_vat_ly, tai_lieu_so, ghi_chu, created_by, cl_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
                    )?;
                    st.execute(p)?;
                }
            }
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ chuyển tiếp bảng cũ → cl_*

fn opt_str(conn: &Connection, sql: &str, p: impl rusqlite::Params) -> rusqlite::Result<Option<String>> {
    conn.query_row(sql, p, |r| r.get::<_, Option<String>>(0)).optional().map(|x| x.flatten())
}

/// cl_phong theo mã phông cũ; tạo mới trong dự án nếu chưa có.
fn cl_phong_for(conn: &Connection, ma_du_an: &str, who: &store::Who) -> rusqlite::Result<Option<String>> {
    if let Some(id) = opt_str(conn, "SELECT id FROM cl_phong WHERE lower(trim(ma_du_an)) = lower(trim(?1)) AND deleted_at IS NULL AND (project_id = ?2 OR project_id IS NULL) LIMIT 1", params![ma_du_an, who.project_id])? {
        return Ok(Some(id));
    }
    let p: Option<(String, Option<String>, Option<String>)> = conn
        .query_row("SELECT ten_du_an, phong_so, muc_luc_so FROM phong WHERE ma_du_an = ?1", params![ma_du_an], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .optional()?;
    let Some((ten, phong_so, muc_luc_so)) = p else { return Ok(None) };
    let id = uuid::Uuid::now_v7().to_string();
    let sp = spec::spec("cl_phong").expect("spec");
    let mut row = Row::new();
    row.insert("ma_du_an".into(), json!(ma_du_an.trim()));
    row.insert("ten".into(), json!(ten));
    row.insert("phong_so".into(), json!(phong_so));
    row.insert("muc_luc_so".into(), json!(muc_luc_so));
    store::write_local(conn, sp, &id, &row, who, None)?;
    Ok(Some(id))
}

fn cl_thbq_for(conn: &Connection, thbqid: i64, who: &store::Who) -> rusqlite::Result<Option<String>> {
    if let Some(id) = opt_str(conn, "SELECT cl_id FROM thoi_han_bq WHERE thbqid = ?1", params![thbqid])? {
        return Ok(Some(id));
    }
    let t: Option<(String, i64, i64, i64)> = conn.query_row("SELECT ten_thbq, trang_thai, gia_tri, sort FROM thoi_han_bq WHERE thbqid = ?1", params![thbqid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).optional()?;
    let Some((ten, tt, gt, sort)) = t else { return Ok(None) };
    // khớp tên với bảng mã (dùng chung hoặc của dự án)
    let found = opt_str(conn, "SELECT id FROM cl_thbq WHERE lower(trim(ten)) = lower(trim(?1)) AND deleted_at IS NULL AND (project_id = ?2 OR project_id IS NULL) ORDER BY project_id IS NULL LIMIT 1", params![ten, who.project_id])?;
    let id = match found {
        Some(id) => id,
        None => {
            let id = uuid::Uuid::now_v7().to_string();
            let mut row = Row::new();
            row.insert("ten".into(), json!(ten));
            row.insert("trang_thai".into(), json!(tt.clamp(1, 4)));
            row.insert("gia_tri".into(), json!(gt));
            row.insert("so_tt".into(), json!(sort));
            row.insert("ma_tt05".into(), json!(digitext_core::chinh_ly::codes::thbq_ma_tt05(tt as i16, gt as i32)));
            store::write_local(conn, spec::spec("cl_thbq").expect("spec"), &id, &row, who, None)?;
            id
        }
    };
    conn.execute("UPDATE thoi_han_bq SET cl_id = ?2 WHERE thbqid = ?1", params![thbqid, id])?;
    Ok(Some(id))
}

fn cl_phong_ban_for(conn: &Connection, ma_dv: &str, who: &store::Who) -> rusqlite::Result<Option<String>> {
    if let Some(id) = opt_str(conn, "SELECT cl_id FROM phong_ban WHERE ma_dv = ?1", params![ma_dv])? {
        return Ok(Some(id));
    }
    let pb: Option<(String, Option<String>, String, Option<i64>)> = conn.query_row("SELECT ma_phong, ma_don_vi, ten_don_vi, so_tt FROM phong_ban WHERE ma_dv = ?1", params![ma_dv], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).optional()?;
    let Some((ma_phong, ma_don_vi, ten, so_tt)) = pb else { return Ok(None) };
    let Some(phong_id) = cl_phong_for(conn, &ma_phong, who)? else { return Ok(None) };
    let id = uuid::Uuid::now_v7().to_string();
    let mut row = Row::new();
    row.insert("phong_id".into(), json!(phong_id));
    row.insert("ma_don_vi".into(), json!(ma_don_vi));
    row.insert("ten".into(), json!(ten));
    row.insert("so_tt".into(), json!(so_tt));
    store::write_local(conn, spec::spec("cl_phong_ban").expect("spec"), &id, &row, who, None)?;
    conn.execute("UPDATE phong_ban SET cl_id = ?2 WHERE ma_dv = ?1", params![ma_dv, id])?;
    Ok(Some(id))
}

fn cl_mang_for(conn: &Connection, mang_cvid: i64, who: &store::Who) -> rusqlite::Result<Option<String>> {
    if let Some(id) = opt_str(conn, "SELECT cl_id FROM mang_cong_viec WHERE mang_cvid = ?1", params![mang_cvid])? {
        return Ok(Some(id));
    }
    let m: Option<(Option<String>, String, String, Option<i64>)> = conn.query_row("SELECT ma_mang, ten_mang, ma_dv, so_tt FROM mang_cong_viec WHERE mang_cvid = ?1", params![mang_cvid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).optional()?;
    let Some((ma_mang, ten, ma_dv, so_tt)) = m else { return Ok(None) };
    let Some(pb) = cl_phong_ban_for(conn, &ma_dv, who)? else { return Ok(None) };
    let id = uuid::Uuid::now_v7().to_string();
    let mut row = Row::new();
    row.insert("phong_ban_id".into(), json!(pb));
    row.insert("ma_mang".into(), json!(ma_mang.filter(|x| !x.trim().is_empty()).unwrap_or_else(|| "00".into())));
    row.insert("ten".into(), json!(ten));
    row.insert("so_tt".into(), json!(so_tt));
    store::write_local(conn, spec::spec("cl_mang").expect("spec"), &id, &row, who, None)?;
    conn.execute("UPDATE mang_cong_viec SET cl_id = ?2 WHERE mang_cvid = ?1", params![mang_cvid, id])?;
    Ok(Some(id))
}

fn code_of(conn: &Connection, table: &str, v: Option<String>) -> Option<String> {
    let v = v?;
    let t = v.trim();
    if t.is_empty() {
        return None;
    }
    let ma = opt_str(conn, &format!("SELECT ma FROM {table} WHERE (ma = ?1 OR lower(trim(ten)) = lower(?1)) AND deleted_at IS NULL LIMIT 1"), params![t]).ok().flatten();
    if ma.is_some() {
        return ma;
    }
    // bản sao chưa kéo bảng mã về → tra bảng mã TT05 trong digitext-core
    let list = if table == "cl_loai_van_ban" { digitext_core::chinh_ly::codes::LOAI_VAN_BAN } else { digitext_core::chinh_ly::codes::NGON_NGU };
    let low = t.to_lowercase();
    list.iter().find(|(m, l)| *m == t || l.to_lowercase() == low).map(|(m, _)| (*m).to_owned()).or(Some(t.to_owned()))
}

/// Đẩy một hồ sơ cũ (theo id) sang cl_ho_so (tạo mới nếu chưa có cl_id). Trả cl_id.
pub fn forward_dossier(conn: &Connection, legacy_id_: i64, who: &store::Who) -> rusqlite::Result<Option<String>> {
    let d = conn
        .query_row(
            "SELECT ma_da, ma_dv, mang_cvid, thbq, so_tam, tieu_de, ngay_bat_dau, ngay_ket_thuc, so_to, trung_thua, tong_so_van_ban, so_luong_trang, che_do_su_dung, ngon_ngu, tinh_trang_vat_ly, ghi_chu, cl_id FROM dossier WHERE id = ?1",
            params![legacy_id_],
            |r| {
                Ok((
                    r.get::<_, Option<String>>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<i64>>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, Option<String>>(7)?,
                    r.get::<_, Option<String>>(8)?,
                    r.get::<_, i64>(9)?,
                    r.get::<_, Option<i64>>(10)?,
                    r.get::<_, Option<i64>>(11)?,
                    r.get::<_, Option<String>>(12)?,
                    r.get::<_, Option<String>>(13)?,
                    r.get::<_, Option<String>>(14)?,
                    r.get::<_, Option<String>>(15)?,
                    r.get::<_, Option<String>>(16)?,
                ))
            },
        )
        .optional()?;
    let Some((ma_da, ma_dv, mang, thbq, so_tam, tieu_de, bd, kt, so_to, trung_thua, tong_vb, so_trang, cdsd, ngon_ngu, ttvl, ghi_chu, cl_id)) = d else { return Ok(None) };
    let Some(ma_da) = ma_da.filter(|m| !m.trim().is_empty()) else { return Ok(None) };
    let Some(phong_id) = cl_phong_for(conn, &ma_da, who)? else { return Ok(None) };
    let mut row = Row::new();
    row.insert("phong_id".into(), json!(phong_id));
    row.insert("phong_ban_id".into(), json!(match ma_dv.as_deref().filter(|x| !x.trim().is_empty()) { Some(dv) => cl_phong_ban_for(conn, dv, who)?, None => None }));
    row.insert("mang_id".into(), json!(match mang { Some(m) => cl_mang_for(conn, m, who)?, None => None }));
    row.insert("thbq_id".into(), json!(match thbq { Some(t) => cl_thbq_for(conn, t, who)?, None => None }));
    row.insert("so_tam".into(), json!(so_tam.trim()));
    row.insert("tieu_de".into(), json!(tieu_de));
    row.insert("ngay_bat_dau".into(), json!(bd.and_then(|t| digitext_core::chinh_ly::normalize_date(&t).ok().flatten())));
    row.insert("ngay_ket_thuc".into(), json!(kt.and_then(|t| digitext_core::chinh_ly::normalize_date(&t).ok().flatten())));
    row.insert("so_to".into(), json!(so_to));
    row.insert("trung_thua".into(), json!(trung_thua != 0));
    row.insert("tong_so_vb".into(), json!(tong_vb));
    row.insert("so_luong_trang".into(), json!(so_trang));
    row.insert("che_do_su_dung".into(), json!(match cdsd.as_deref().map(str::trim) { Some("Hạn chế") | Some("han che") => Some("02".to_owned()), Some("Không hạn chế") | Some("Công khai") => Some("01".to_owned()), Some(x) if matches!(x, "01" | "02" | "03") => Some(x.to_owned()), _ => None }));
    row.insert("ngon_ngu".into(), json!(code_of(conn, "cl_ngon_ngu", ngon_ngu)));
    row.insert("tinh_trang_vat_ly".into(), json!(match ttvl.as_deref().map(str::trim) { Some(x) if matches!(x, "01" | "02" | "03") => Some(x.to_owned()), Some("Tốt") => Some("01".to_owned()), Some("Hỏng") => Some("03".to_owned()), Some(_) => Some("02".to_owned()), None => None }));
    row.insert("ghi_chu".into(), json!(ghi_chu));
    let id = cl_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
    store::write_local(conn, spec::spec("cl_ho_so").expect("spec"), &id, &row, who, None)?;
    conn.execute("UPDATE dossier SET cl_id = ?2 WHERE id = ?1", params![legacy_id_, id])?;
    Ok(Some(id))
}

/// Đẩy một văn bản cũ sang cl_van_ban (cần hồ sơ cha đã có cl_id — tự đẩy nếu chưa).
pub fn forward_van_ban(conn: &Connection, legacy_id_: i64, who: &store::Who) -> rusqlite::Result<Option<String>> {
    let v = conn
        .query_row(
            "SELECT dossier_id, thu_tu, loai_van_ban, so_ky_hieu, ngay_thang, co_quan_ban_hanh, trich_yeu, to_so, so_trang, trang_so, ngon_ngu, che_do_su_dung, do_tin_cay, but_tich, tinh_trang_vat_ly, tai_lieu_so, ghi_chu, cl_id FROM van_ban WHERE id = ?1",
            params![legacy_id_],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, Option<i64>>(7)?,
                    r.get::<_, Option<i64>>(8)?,
                    r.get::<_, Option<i64>>(9)?,
                    r.get::<_, Option<String>>(10)?,
                    r.get::<_, Option<String>>(11)?,
                    r.get::<_, Option<String>>(12)?,
                    r.get::<_, Option<String>>(13)?,
                    r.get::<_, Option<String>>(14)?,
                    r.get::<_, Option<String>>(15)?,
                    r.get::<_, Option<String>>(16)?,
                    r.get::<_, Option<String>>(17)?,
                ))
            },
        )
        .optional()?;
    let Some((dossier_id, thu_tu, loai, skh, ngay, cq, ty, to_so, so_trang, trang_so, nn, cdsd, dtc, bt, ttvl, tls, gc, cl_id)) = v else { return Ok(None) };
    let ho_so_id = match opt_str(conn, "SELECT cl_id FROM dossier WHERE id = ?1", params![dossier_id])? {
        Some(h) => h,
        None => match forward_dossier(conn, dossier_id, who)? {
            Some(h) => h,
            None => return Ok(None),
        },
    };
    let mut row = Row::new();
    row.insert("ho_so_id".into(), json!(ho_so_id));
    row.insert("so_thu_tu".into(), json!(thu_tu.max(1)));
    row.insert("loai_vb".into(), json!(code_of(conn, "cl_loai_van_ban", loai)));
    row.insert("so_ky_hieu".into(), json!(skh));
    row.insert("ngay_thang".into(), json!(ngay.and_then(|t| digitext_core::chinh_ly::normalize_date(&t).ok().flatten())));
    row.insert("co_quan_ban_hanh".into(), json!(cq));
    row.insert("trich_yeu".into(), json!(ty));
    row.insert("to_so".into(), json!(to_so));
    row.insert("so_trang".into(), json!(so_trang));
    row.insert("trang_so".into(), json!(trang_so));
    row.insert("ngon_ngu".into(), json!(code_of(conn, "cl_ngon_ngu", nn)));
    row.insert("che_do_su_dung".into(), json!(cdsd.filter(|x| matches!(x.trim(), "01" | "02" | "03"))));
    row.insert("do_tin_cay".into(), json!(dtc.filter(|x| matches!(x.trim(), "01" | "02" | "03"))));
    row.insert("but_tich".into(), json!(bt));
    row.insert("tinh_trang_vat_ly".into(), json!(ttvl.filter(|x| matches!(x.trim(), "01" | "02" | "03"))));
    row.insert("tai_lieu_so".into(), json!(tls));
    row.insert("ghi_chu".into(), json!(gc));
    let id = cl_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
    store::write_local(conn, spec::spec("cl_van_ban").expect("spec"), &id, &row, who, None)?;
    conn.execute("UPDATE van_ban SET cl_id = ?2 WHERE id = ?1", params![legacy_id_, id])?;
    Ok(Some(id))
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrateReport {
    pub phong: usize,
    pub thbq: usize,
    pub phong_ban: usize,
    pub mang: usize,
    pub ho_so: usize,
    pub van_ban: usize,
    pub kho: usize,
    pub gia: usize,
    pub errors: Vec<String>,
}

/// Số dòng cũ chưa gắn cl_id (xem trước wizard).
pub fn count_unmigrated(conn: &Connection) -> rusqlite::Result<MigrateReport> {
    let c = |sql: &str| conn.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap_or(0) as usize;
    Ok(MigrateReport {
        phong: c("SELECT count(*) FROM phong p WHERE NOT EXISTS (SELECT 1 FROM cl_phong c WHERE lower(trim(c.ma_du_an)) = lower(trim(p.ma_du_an)) AND c.deleted_at IS NULL)"),
        thbq: c("SELECT count(*) FROM thoi_han_bq WHERE cl_id IS NULL"),
        phong_ban: c("SELECT count(*) FROM phong_ban WHERE cl_id IS NULL"),
        mang: c("SELECT count(*) FROM mang_cong_viec WHERE cl_id IS NULL"),
        ho_so: c("SELECT count(*) FROM dossier WHERE cl_id IS NULL"),
        van_ban: c("SELECT count(*) FROM van_ban WHERE cl_id IS NULL"),
        kho: c("SELECT count(*) FROM kho WHERE cl_id IS NULL"),
        gia: c("SELECT count(*) FROM gia WHERE cl_id IS NULL"),
        errors: vec![],
    })
}

/// Đẩy mọi dòng cũ chưa có cl_id sang cl_* (idempotent; chạy lại chỉ xử lý phần còn thiếu).
pub fn push_unmigrated(conn: &Connection, who: &store::Who) -> rusqlite::Result<MigrateReport> {
    let mut rep = MigrateReport::default();
    let tx = conn.unchecked_transaction()?;
    let phongs: Vec<String> = {
        let mut st = tx.prepare("SELECT ma_du_an FROM phong")?;
        let v = st.query_map([], |r| r.get(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
        v
    };
    for ma in phongs {
        let before = opt_str(&tx, "SELECT id FROM cl_phong WHERE lower(trim(ma_du_an)) = lower(trim(?1)) AND deleted_at IS NULL LIMIT 1", params![ma])?;
        if before.is_none() && cl_phong_for(&tx, &ma, who)?.is_some() {
            rep.phong += 1;
        }
    }
    let thbqs: Vec<i64> = {
        let mut st = tx.prepare("SELECT thbqid FROM thoi_han_bq WHERE cl_id IS NULL")?;
        let v = st.query_map([], |r| r.get(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
        v
    };
    for t in thbqs {
        if cl_thbq_for(&tx, t, who)?.is_some() {
            rep.thbq += 1;
        }
    }
    let pbs: Vec<String> = {
        let mut st = tx.prepare("SELECT ma_dv FROM phong_ban WHERE cl_id IS NULL")?;
        let v = st.query_map([], |r| r.get(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
        v
    };
    for dv in pbs {
        match cl_phong_ban_for(&tx, &dv, who)? {
            Some(_) => rep.phong_ban += 1,
            None => rep.errors.push(format!("Phòng ban {dv}: không xác định được phông")),
        }
    }
    let mangs: Vec<i64> = {
        let mut st = tx.prepare("SELECT mang_cvid FROM mang_cong_viec WHERE cl_id IS NULL")?;
        let v = st.query_map([], |r| r.get(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
        v
    };
    for m in mangs {
        match cl_mang_for(&tx, m, who)? {
            Some(_) => rep.mang += 1,
            None => rep.errors.push(format!("Mảng #{m}: không xác định được phòng ban")),
        }
    }
    let ds: Vec<(i64, String)> = {
        let mut st = tx.prepare("SELECT id, so_tam FROM dossier WHERE cl_id IS NULL ORDER BY id")?;
        let v = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
        v
    };
    for (id, so_tam) in ds {
        match forward_dossier(&tx, id, who) {
            Ok(Some(_)) => rep.ho_so += 1,
            Ok(None) => rep.errors.push(format!("Hồ sơ số tạm {so_tam}: thiếu phông")),
            Err(e) => rep.errors.push(format!("Hồ sơ số tạm {so_tam}: {e}")),
        }
        if rep.errors.len() > 200 {
            break;
        }
    }
    let vbs: Vec<i64> = {
        let mut st = tx.prepare("SELECT id FROM van_ban WHERE cl_id IS NULL ORDER BY dossier_id, thu_tu")?;
        let v = st.query_map([], |r| r.get(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
        v
    };
    for id in vbs {
        match forward_van_ban(&tx, id, who) {
            Ok(Some(_)) => rep.van_ban += 1,
            Ok(None) => {}
            Err(e) => rep.errors.push(format!("Văn bản #{id}: {e}")),
        }
        if rep.errors.len() > 200 {
            break;
        }
    }
    tx.commit()?;
    Ok(rep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn open() -> (Connection, std::path::PathBuf) {
        let f = std::env::temp_dir().join(format!("digitext-legacy-{}-{}.db", std::process::id(), uuid::Uuid::now_v7()));
        let _ = std::fs::remove_file(&f);
        let c = crate::db::open(&f).unwrap();
        (c, f)
    }

    #[test]
    fn day_du_lieu_cu_len_cl_roi_chieu_nguoc() {
        let (c, f) = open();
        let who = store::Who { project_id: "p1", user_id: "u1" };
        crate::db::f_2o6(&c, "UBND", "Phông UBND", None, Some("01"), None).unwrap();
        c.execute("INSERT INTO phong_ban (ma_dv, ma_phong, ma_don_vi, ten_don_vi, so_tt) VALUES ('VP', 'UBND', 'VP', 'Văn phòng', 1)", []).unwrap();
        c.execute("INSERT INTO mang_cong_viec (ma_mang, ten_mang, ma_dv, so_tt) VALUES ('01', 'Tổng hợp', 'VP', 1)", []).unwrap();
        let mang: i64 = c.query_row("SELECT mang_cvid FROM mang_cong_viec", [], |r| r.get(0)).unwrap();
        c.execute(
            "INSERT INTO dossier (ma_da, ma_dv, mang_cvid, thbq, so_tam, tieu_de, ngay_ket_thuc, so_to, trung_thua) VALUES ('UBND', 'VP', ?1, 1, '12', 'Hồ sơ A', '2021-03-05', '9', 0)",
            params![mang],
        )
        .unwrap();
        let did: i64 = c.query_row("SELECT id FROM dossier", [], |r| r.get(0)).unwrap();
        c.execute("INSERT INTO van_ban (dossier_id, thu_tu, loai_van_ban, so_ky_hieu, trich_yeu, pdf_path) VALUES (?1, 1, 'Công văn', '12/CV', 'VB 1', 'C:/x.pdf')", params![did]).unwrap();
        let rep = push_unmigrated(&c, &who).unwrap();
        assert_eq!((rep.phong, rep.phong_ban, rep.mang, rep.ho_so, rep.van_ban), (1, 1, 1, 1, 1), "{rep:?}");
        assert!(rep.thbq >= 1);
        let cl_id: String = c.query_row("SELECT cl_id FROM dossier WHERE id = ?1", params![did], |r| r.get(0)).unwrap();
        let e = store::get(&c, "cl_ho_so", &cl_id).unwrap().unwrap();
        assert_eq!(e.row["so_tam"], json!("12"));
        assert_eq!(e.row["ngay_ket_thuc"], json!("05/03/2021"));
        assert_eq!(e.dirty, 2);
        let vb = store::get(&c, "cl_van_ban", &c.query_row::<String, _, _>("SELECT cl_id FROM van_ban", [], |r| r.get(0)).unwrap()).unwrap().unwrap();
        assert_eq!(vb.row["loai_vb"], json!("22"), "tên loại → mã TT05");
        assert_eq!(vb.row["so_ky_hieu"], json!("12/CV"));
        // chạy lại: không tạo thêm
        let rep2 = push_unmigrated(&c, &who).unwrap();
        assert_eq!((rep2.ho_so, rep2.van_ban), (0, 0));
        // sửa bên cl → chiếu về dossier
        let sp = spec::spec("cl_ho_so").unwrap();
        let row = store::write_local(&c, sp, &cl_id, &json!({"tieu_de": "Hồ sơ A2"}).as_object().unwrap().clone(), &who, None).unwrap();
        project_row(&c, "cl_ho_so", &row).unwrap();
        let td: String = c.query_row("SELECT tieu_de FROM dossier WHERE id = ?1", params![did], |r| r.get(0)).unwrap();
        assert_eq!(td, "Hồ sơ A2");
        // pdf_path giữ nguyên sau chiếu văn bản
        let vrow = store::get(&c, "cl_van_ban", &s(&vb.row, "id").unwrap()).unwrap().unwrap().row;
        project_row(&c, "cl_van_ban", &vrow).unwrap();
        let pdf: Option<String> = c.query_row("SELECT pdf_path FROM van_ban", [], |r| r.get(0)).unwrap();
        assert_eq!(pdf.as_deref(), Some("C:/x.pdf"));
        // xoá bên cl → prune bảng cũ
        store::delete_local(&c, sp, &cl_id, &who).unwrap();
        prune_legacy(&c).unwrap();
        let n: i64 = c.query_row("SELECT count(*) FROM dossier", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
        drop(c);
        let _ = std::fs::remove_file(&f);
    }
}
