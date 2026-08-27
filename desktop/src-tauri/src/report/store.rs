// Copyright (c) 2026 DigiText. All rights reserved.
use super::types::{T2r, Te5, Th4, Tlt};
use rusqlite::{params, Connection};
const RETENTION_COUNTS: &str = "\
    COUNT(DISTINCT CASE WHEN t.trang_thai = 1 THEN NULLIF(TRIM(d.hop_so), '') END) AS hop_vv, \
    COUNT(CASE WHEN t.trang_thai = 1 THEN 1 END) AS hs_vv, \
    COUNT(DISTINCT CASE WHEN t.trang_thai = 2 THEN NULLIF(TRIM(d.hop_so), '') END) AS hop_cth, \
    COUNT(CASE WHEN t.trang_thai = 2 THEN 1 END) AS hs_cth, \
    COUNT(DISTINCT CASE WHEN t.trang_thai = 3 THEN NULLIF(TRIM(d.bo_so), '') END) AS bo_loai, \
    COUNT(DISTINCT CASE WHEN t.trang_thai = 3 THEN NULLIF(TRIM(d.tap_so), '') END) AS tap_loai";
fn f_25h(a0: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64, r_2au: i64) -> f64 {
    16.0 * a0 as f64
        + 8.0 * a1 as f64
        + 4.0 * a2 as f64
        + 2.0 * a3 as f64
        + a4 as f64
        + 0.5 * a5 as f64
        + r_2au as f64
}
pub fn f_2g2(r_2av: &Connection) -> rusqlite::Result<Vec<Tlt>> {
    let r_2az = "\
        SELECT h.phong_id, p.ten_du_an, \
               COUNT(DISTINCT h.id) AS so_ho_so, \
               COUNT(DISTINCT CASE WHEN h.pdf_path IS NOT NULL THEN h.id END) AS so_pdf, \
               SUM(CASE WHEN tr.paper = 'A0' THEN 1 ELSE 0 END) AS a0, \
               SUM(CASE WHEN tr.paper = 'A1' THEN 1 ELSE 0 END) AS a1, \
               SUM(CASE WHEN tr.paper = 'A2' THEN 1 ELSE 0 END) AS a2, \
               SUM(CASE WHEN tr.paper = 'A3' THEN 1 ELSE 0 END) AS a3, \
               SUM(CASE WHEN tr.paper = 'A4' THEN 1 ELSE 0 END) AS a4, \
               SUM(CASE WHEN tr.paper = 'A5' THEN 1 ELSE 0 END) AS a5, \
               COUNT(tr.id) AS tong_trang \
        FROM ho_so h \
        LEFT JOIN phong p ON p.ma_du_an = h.phong_id \
        LEFT JOIN trang tr ON tr.ho_so_id = h.id \
        GROUP BY h.phong_id \
        ORDER BY (p.ten_du_an IS NULL), p.ten_du_an, h.phong_id";
    let mut r_2b0 = r_2av.prepare(r_2az)?;
    let r_2ay = r_2b0
        .query_map([], |r_2ax| {
            let a0: i64 = r_2ax.get(4)?;
            let a1: i64 = r_2ax.get(5)?;
            let a2: i64 = r_2ax.get(6)?;
            let a3: i64 = r_2ax.get(7)?;
            let a4: i64 = r_2ax.get(8)?;
            let a5: i64 = r_2ax.get(9)?;
            let tong_trang: i64 = r_2ax.get(10)?;
            let r_2aw = (tong_trang - (a0 + a1 + a2 + a3 + a4 + a5)).max(0);
            Ok(Tlt {
                ma_du_an: r_2ax.get(0)?,
                ten_du_an: r_2ax.get(1)?,
                so_ho_so: r_2ax.get(2)?,
                so_pdf: r_2ax.get(3)?,
                a0,
                a1,
                a2,
                a3,
                a4,
                a5,
                khac: r_2aw,
                tong_trang,
                quy_doi: f_25h(a0, a1, a2, a3, a4, a5, r_2aw),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_2ay)
}
pub fn f_yq(r_2b1: &Connection, ma_da: Option<&str>) -> rusqlite::Result<Vec<T2r>> {
    let r_2b4 = format!(
        "SELECT d.ma_dv, pb.ten_don_vi, {RETENTION_COUNTS} \
         FROM dossier d \
         LEFT JOIN phong_ban pb ON pb.ma_dv = d.ma_dv \
         LEFT JOIN thoi_han_bq t ON t.thbqid = d.thbq \
         WHERE (?1 IS NULL OR d.ma_da = ?1) \
         GROUP BY d.ma_dv \
         ORDER BY (pb.ten_don_vi IS NULL), pb.ten_don_vi, d.ma_dv"
    );
    let mut r_2b5 = r_2b1.prepare(&r_2b4)?;
    let r_2b3 = r_2b5
        .query_map(params![ma_da], |r_2b2| {
            Ok(T2r {
                ma_dv: r_2b2.get(0)?,
                ten_don_vi: r_2b2.get(1)?,
                hop_vinh_vien: r_2b2.get(2)?,
                ho_so_vinh_vien: r_2b2.get(3)?,
                hop_co_thoi_han: r_2b2.get(4)?,
                ho_so_co_thoi_han: r_2b2.get(5)?,
                bo_loai: r_2b2.get(6)?,
                tap_loai: r_2b2.get(7)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_2b3)
}
pub fn f_22b(
    r_2b6: &Connection,
    r_2bc: Option<&str>,
    r_2b7: Option<&str>,
) -> rusqlite::Result<Vec<Th4>> {
    let r_2ba = format!(
        "SELECT d.ma_da, p.ten_du_an, MIN(d.created_at) AS khoi_tao, {RETENTION_COUNTS} \
         FROM dossier d \
         LEFT JOIN phong p ON p.ma_du_an = d.ma_da \
         LEFT JOIN thoi_han_bq t ON t.thbqid = d.thbq \
         WHERE (?1 IS NULL OR date(d.created_at) >= date(?1)) \
           AND (?2 IS NULL OR date(d.created_at) <= date(?2)) \
         GROUP BY d.ma_da \
         ORDER BY (p.ten_du_an IS NULL), p.ten_du_an, d.ma_da"
    );
    let mut r_2bb = r_2b6.prepare(&r_2ba)?;
    let r_2b9 = r_2bb
        .query_map(params![r_2bc, r_2b7], |r_2b8| {
            Ok(Th4 {
                ma_du_an: r_2b8.get(0)?,
                ten_du_an: r_2b8.get(1)?,
                thoi_gian_khoi_tao: r_2b8.get(2)?,
                hop_vinh_vien: r_2b8.get(3)?,
                ho_so_vinh_vien: r_2b8.get(4)?,
                hop_co_thoi_han: r_2b8.get(5)?,
                ho_so_co_thoi_han: r_2b8.get(6)?,
                bo_loai: r_2b8.get(7)?,
                tap_loai: r_2b8.get(8)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_2b9)
}
pub fn f_1ul(r_2bd: &Connection) -> rusqlite::Result<Vec<Te5>> {
    let r_2bg = "\
        WITH users(uid) AS ( \
            SELECT created_by FROM dossier WHERE created_by IS NOT NULL \
            UNION SELECT created_by FROM van_ban WHERE created_by IS NOT NULL \
            UNION SELECT scanned_by FROM trang WHERE scanned_by IS NOT NULL \
            UNION SELECT finalized_by FROM ho_so WHERE finalized_by IS NOT NULL \
            UNION SELECT nguoi_id FROM ho_so_history WHERE nguoi_id IS NOT NULL \
        ) \
        SELECT u.uid, \
               COALESCE(NULLIF(TRIM(a.display_name), ''), a.username, u.uid) AS ten, \
               (SELECT COUNT(*) FROM dossier d WHERE d.created_by = u.uid) AS so_ho_so, \
               (SELECT COUNT(*) FROM van_ban v WHERE v.created_by = u.uid) AS so_van_ban, \
               (SELECT COUNT(*) FROM trang tr WHERE tr.scanned_by = u.uid) AS so_trang, \
               (SELECT COUNT(*) FROM ho_so_history h WHERE h.nguoi_id = u.uid AND h.action = 'create') AS them, \
               (SELECT COUNT(*) FROM ho_so_history h WHERE h.nguoi_id = u.uid AND h.action = 'update') AS sua \
        FROM users u LEFT JOIN auth_account a ON a.user_id = u.uid \
        ORDER BY ten";
    let mut r_2bh = r_2bd.prepare(r_2bg)?;
    let r_2bf = r_2bh
        .query_map([], |r_2be| {
            Ok(Te5 {
                user_id: r_2be.get(0)?,
                ten: r_2be.get(1)?,
                so_ho_so: r_2be.get(2)?,
                so_van_ban: r_2be.get(3)?,
                so_trang_quet: r_2be.get(4)?,
                thao_tac_them: r_2be.get(5)?,
                thao_tac_sua: r_2be.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_2bf)
}
