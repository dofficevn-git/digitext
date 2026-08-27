// Copyright (c) 2026 DigiText. All rights reserved.
use super::types::{Tb6, Tb7, Tcz, Td0, Te3, Te4, Th7, Th8, ThbqIn, To1};
use rusqlite::{params, Connection};
pub const C_JQ: &str = r#"
CREATE TABLE IF NOT EXISTS phong_ban (
    ma_dv      TEXT PRIMARY KEY,
    ma_phong   TEXT NOT NULL REFERENCES phong(ma_du_an) ON DELETE CASCADE,
    ma_don_vi  TEXT,
    ten_don_vi TEXT NOT NULL,
    so_tt      INTEGER
);
CREATE INDEX IF NOT EXISTS ix_phong_ban_phong ON phong_ban (ma_phong);

CREATE TABLE IF NOT EXISTS mang_cong_viec (
    mang_cvid INTEGER PRIMARY KEY,
    ma_mang   TEXT,
    ten_mang  TEXT NOT NULL,
    ma_dv     TEXT NOT NULL REFERENCES phong_ban(ma_dv) ON DELETE CASCADE,
    so_tt     INTEGER
);
CREATE INDEX IF NOT EXISTS ix_mang_pb ON mang_cong_viec (ma_dv);

CREATE TABLE IF NOT EXISTS thoi_han_bq (
    thbqid     INTEGER PRIMARY KEY,
    ten_thbq   TEXT NOT NULL,
    trang_thai INTEGER NOT NULL DEFAULT 2,
    gia_tri    INTEGER NOT NULL DEFAULT 0,
    sort       INTEGER NOT NULL DEFAULT 0
);

INSERT OR IGNORE INTO thoi_han_bq (thbqid, ten_thbq, trang_thai, gia_tri, sort) VALUES
    (1, 'Vĩnh viễn',            1,  0, 1),
    (2, '5 năm',                2,  5, 2),
    (3, '10 năm',               2, 10, 3),
    (4, '20 năm',               2, 20, 4),
    (5, '50 năm',               2, 50, 5),
    (6, '70 năm',               2, 70, 6),
    (7, 'Giữ lại để xét hủy',   3,  0, 7);
"#;
pub const C_JT: &str = r#"
CREATE TABLE IF NOT EXISTS ngon_ngu (
    ma_ngon_ngu  TEXT PRIMARY KEY,
    ten_ngon_ngu TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS loai_van_ban (
    ma_loai  TEXT PRIMARY KEY,
    ten_loai TEXT NOT NULL
);

INSERT OR IGNORE INTO ngon_ngu (ma_ngon_ngu, ten_ngon_ngu) VALUES
    ('01', 'Tiếng Việt'),
    ('02', 'English');

INSERT OR IGNORE INTO loai_van_ban (ma_loai, ten_loai) VALUES
    ('01', 'Nghị quyết'),
    ('02', 'Quyết định'),
    ('03', 'Chỉ thị'),
    ('04', 'Công văn');
"#;
pub fn f_2o7(r_p2: &Connection, pb: &Th7) -> rusqlite::Result<()> {
    r_p2.execute(
        "INSERT INTO phong_ban (ma_dv, ma_phong, ma_don_vi, ten_don_vi, so_tt)
         VALUES (?1,?2,?3,?4,?5)
         ON CONFLICT(ma_dv) DO UPDATE SET
            ma_phong   = excluded.ma_phong,
            ma_don_vi  = excluded.ma_don_vi,
            ten_don_vi = excluded.ten_don_vi,
            so_tt      = excluded.so_tt",
        params![pb.ma_dv, pb.ma_phong, pb.ma_don_vi, pb.ten_don_vi, pb.so_tt],
    )?;
    Ok(())
}
pub fn f_146(r_p3: &Connection, ma_dv: &str) -> rusqlite::Result<bool> {
    let r_p4 = r_p3.execute("DELETE FROM phong_ban WHERE ma_dv = ?1", params![ma_dv])?;
    Ok(r_p4 > 0)
}
fn f_1o7(r_p5: &Connection) -> rusqlite::Result<Vec<Th8>> {
    let mut r_p8 = r_p5.prepare(
        "SELECT ma_dv, ma_phong, ma_don_vi, ten_don_vi, so_tt
         FROM phong_ban ORDER BY ma_phong, so_tt, ma_dv",
    )?;
    let r_p7 = r_p8
        .query_map([], |r_p6| {
            Ok(Th8 {
                ma_dv: r_p6.get(0)?,
                ma_phong: r_p6.get(1)?,
                ma_don_vi: r_p6.get(2)?,
                ten_don_vi: r_p6.get(3)?,
                so_tt: r_p6.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_p7)
}
pub fn f_2o4(r_p9: &Connection, r_pa: &Tcz) -> rusqlite::Result<()> {
    r_p9.execute(
        "INSERT INTO mang_cong_viec (mang_cvid, ma_mang, ten_mang, ma_dv, so_tt)
         VALUES (?1,?2,?3,?4,?5)
         ON CONFLICT(mang_cvid) DO UPDATE SET
            ma_mang  = excluded.ma_mang,
            ten_mang = excluded.ten_mang,
            ma_dv    = excluded.ma_dv,
            so_tt    = excluded.so_tt",
        params![
            r_pa.mang_cvid,
            r_pa.ma_mang,
            r_pa.ten_mang,
            r_pa.ma_dv,
            r_pa.so_tt
        ],
    )?;
    Ok(())
}
pub fn f_143(r_pb: &Connection, mang_cvid: i64) -> rusqlite::Result<bool> {
    let r_pc = r_pb.execute(
        "DELETE FROM mang_cong_viec WHERE mang_cvid = ?1",
        params![mang_cvid],
    )?;
    Ok(r_pc > 0)
}
fn f_1nw(r_pd: &Connection) -> rusqlite::Result<Vec<Td0>> {
    let mut r_pg = r_pd.prepare(
        "SELECT mang_cvid, ma_mang, ten_mang, ma_dv, so_tt
         FROM mang_cong_viec ORDER BY ma_dv, so_tt, mang_cvid",
    )?;
    let r_pf = r_pg
        .query_map([], |r_pe| {
            Ok(Td0 {
                mang_cvid: r_pe.get(0)?,
                ma_mang: r_pe.get(1)?,
                ten_mang: r_pe.get(2)?,
                ma_dv: r_pe.get(3)?,
                so_tt: r_pe.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_pf)
}
pub fn f_2o8(r_ph: &Connection, r_pi: &ThbqIn) -> rusqlite::Result<()> {
    r_ph.execute(
        "INSERT INTO thoi_han_bq (thbqid, ten_thbq, trang_thai, gia_tri, sort)
         VALUES (?1,?2,?3,?4,?5)
         ON CONFLICT(thbqid) DO UPDATE SET
            ten_thbq   = excluded.ten_thbq,
            trang_thai = excluded.trang_thai,
            gia_tri    = excluded.gia_tri,
            sort       = excluded.sort",
        params![
            r_pi.thbqid,
            r_pi.ten_thbq,
            r_pi.trang_thai,
            r_pi.gia_tri,
            r_pi.sort
        ],
    )?;
    Ok(())
}
pub fn f_147(r_pj: &Connection, thbqid: i64) -> rusqlite::Result<bool> {
    let r_pk = r_pj.execute("DELETE FROM thoi_han_bq WHERE thbqid = ?1", params![thbqid])?;
    Ok(r_pk > 0)
}
fn f_1og(r_pl: &Connection) -> rusqlite::Result<Vec<To1>> {
    let mut r_po = r_pl.prepare(
        "SELECT thbqid, ten_thbq, trang_thai, gia_tri, sort FROM thoi_han_bq ORDER BY sort, thbqid",
    )?;
    let r_pn = r_po
        .query_map([], |r_pm| {
            Ok(To1 {
                thbqid: r_pm.get(0)?,
                ten_thbq: r_pm.get(1)?,
                trang_thai: r_pm.get(2)?,
                gia_tri: r_pm.get(3)?,
                sort: r_pm.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_pn)
}
pub fn f_2o5(r_pp: &Connection, r_pq: &Te3) -> rusqlite::Result<()> {
    r_pp.execute(
        "INSERT INTO ngon_ngu (ma_ngon_ngu, ten_ngon_ngu) VALUES (?1,?2)
         ON CONFLICT(ma_ngon_ngu) DO UPDATE SET ten_ngon_ngu = excluded.ten_ngon_ngu",
        params![r_pq.ma_ngon_ngu, r_pq.ten_ngon_ngu],
    )?;
    Ok(())
}
pub fn f_144(r_pr: &Connection, r_ps: &str) -> rusqlite::Result<bool> {
    let r_pt = r_pr.execute("DELETE FROM ngon_ngu WHERE ma_ngon_ngu = ?1", params![r_ps])?;
    Ok(r_pt > 0)
}
fn f_1o2(r_pu: &Connection) -> rusqlite::Result<Vec<Te4>> {
    let mut r_px =
        r_pu.prepare("SELECT ma_ngon_ngu, ten_ngon_ngu FROM ngon_ngu ORDER BY ma_ngon_ngu")?;
    let r_pw = r_px
        .query_map([], |r_pv| {
            Ok(Te4 {
                ma_ngon_ngu: r_pv.get(0)?,
                ten_ngon_ngu: r_pv.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_pw)
}
pub fn f_2o3(r_py: &Connection, r_pz: &Tb6) -> rusqlite::Result<()> {
    r_py.execute(
        "INSERT INTO loai_van_ban (ma_loai, ten_loai) VALUES (?1,?2)
         ON CONFLICT(ma_loai) DO UPDATE SET ten_loai = excluded.ten_loai",
        params![r_pz.ma_loai, r_pz.ten_loai],
    )?;
    Ok(())
}
pub fn f_142(r_q0: &Connection, r_q1: &str) -> rusqlite::Result<bool> {
    let r_q2 = r_q0.execute("DELETE FROM loai_van_ban WHERE ma_loai = ?1", params![r_q1])?;
    Ok(r_q2 > 0)
}
fn f_1nv(r_q3: &Connection) -> rusqlite::Result<Vec<Tb7>> {
    let mut r_q6 = r_q3.prepare("SELECT ma_loai, ten_loai FROM loai_van_ban ORDER BY ma_loai")?;
    let r_q5 = r_q6
        .query_map([], |r_q4| {
            Ok(Tb7 {
                ma_loai: r_q4.get(0)?,
                ten_loai: r_q4.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_q5)
}
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T2l {
    pub phong_bans: Vec<Th8>,
    pub mangs: Vec<Td0>,
    pub thbqs: Vec<To1>,
    pub ngon_ngus: Vec<Te4>,
    pub loai_van_bans: Vec<Tb7>,
}
pub fn snapshot(r_q7: &Connection) -> rusqlite::Result<T2l> {
    Ok(T2l {
        phong_bans: f_1o7(r_q7)?,
        mangs: f_1nw(r_q7)?,
        thbqs: f_1og(r_q7)?,
        ngon_ngus: f_1o2(r_q7)?,
        loai_van_bans: f_1nv(r_q7)?,
    })
}
