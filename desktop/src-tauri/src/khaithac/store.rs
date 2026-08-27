// Copyright (c) 2026 DigiText. All rights reserved.
use super::types::{KhaiThacFilter, T8l, T8m, T8o, Ta5, Ta6, Ta7, Ta8, Ta9};
use rusqlite::{params, Connection};
pub const C_JK: &str = r#"
CREATE TABLE IF NOT EXISTS kho (
    ma_kho  INTEGER PRIMARY KEY,
    ten_kho TEXT NOT NULL,
    so_tt   INTEGER
);

CREATE TABLE IF NOT EXISTS gia (
    ma_gia  INTEGER PRIMARY KEY,
    ten_gia TEXT NOT NULL,
    ma_kho  INTEGER NOT NULL REFERENCES kho(ma_kho) ON DELETE CASCADE,
    so_tt   INTEGER
);
CREATE INDEX IF NOT EXISTS ix_gia_kho ON gia (ma_kho);
"#;
pub const C_JL: &str = "ALTER TABLE dossier ADD COLUMN ma_gia INTEGER;";
pub const C_JM: &[(&str, &str, &str)] = &[
    ("kho", "so_do_kho", "TEXT"),
    ("kho", "ghi_chu", "TEXT"),
    ("gia", "ma_dv", "TEXT"),
    ("gia", "tu_hop", "INTEGER"),
    ("gia", "den_hop", "INTEGER"),
    ("gia", "ghi_chu", "TEXT"),
];
pub const C_JN: &[(&str, &str, &str)] = &[("gia", "ma_da", "TEXT")];
const HET_EXPR: &str = "(t.trang_thai = 2 AND d.ngay_ket_thuc IS NOT NULL \
     AND date(d.ngay_ket_thuc, '+' || t.gia_tri || ' years') < date('now'))";
const VV_EXPR: &str = "(t.trang_thai = 1)";
fn f_2he() -> String {
    format!("CASE WHEN {VV_EXPR} THEN 'VV' WHEN {HET_EXPR} THEN 'HET' ELSE 'GL' END")
}
const FROM_JOIN: &str = "FROM dossier d \
     LEFT JOIN phong p ON p.ma_du_an = d.ma_da \
     LEFT JOIN thoi_han_bq t ON t.thbqid = d.thbq \
     LEFT JOIN gia g ON g.ma_gia = d.ma_gia \
     LEFT JOIN kho k ON k.ma_kho = g.ma_kho";
fn f_2qw(r_1xu: &KhaiThacFilter) -> String {
    let mut c = String::from(
        "WHERE (?1 IS NULL OR d.ma_da = ?1) \
           AND (?2 IS NULL OR d.ma_dv = ?2) \
           AND (?3 IS NULL OR d.tieu_de LIKE '%'||?3||'%' \
                           OR d.so_tam LIKE '%'||?3||'%' \
                           OR IFNULL(d.ma_ho_so,'') LIKE '%'||?3||'%')",
    );
    if let Some(m) = r_1xu.mang_cvid {
        c.push_str(&format!(" AND d.mang_cvid = {m}"));
    }
    if let Some(k) = r_1xu.ma_kho {
        c.push_str(&format!(" AND g.ma_kho = {k}"));
    }
    if let Some(g) = r_1xu.ma_gia {
        c.push_str(&format!(" AND d.ma_gia = {g}"));
    }
    let r_1xw = r_1xu.thbq_ids();
    if !r_1xw.is_empty() {
        let list = r_1xw
            .iter()
            .map(|r_1xx| r_1xx.to_string())
            .collect::<Vec<_>>()
            .join(",");
        c.push_str(&format!(" AND d.thbq IN ({list})"));
    }
    if r_1xu.trung_thua {
        c.push_str(" AND d.trung_thua = 1");
    }
    let mut r_1xv: Vec<String> = Vec::new();
    if r_1xu.vinh_vien {
        r_1xv.push(VV_EXPR.to_string());
    }
    if r_1xu.het_thbq {
        r_1xv.push(HET_EXPR.to_string());
    }
    if r_1xu.giu_lai {
        r_1xv.push(format!("(NOT {VV_EXPR} AND NOT {HET_EXPR})"));
    }
    if !r_1xv.is_empty() {
        c.push_str(&format!(" AND ({})", r_1xv.join(" OR ")));
    }
    c
}
fn f_1qy(r_1xy: &rusqlite::Row) -> rusqlite::Result<Ta5> {
    Ok(Ta5 {
        ho_so_id: r_1xy.get(0)?,
        ma_da: r_1xy.get(1)?,
        ten_da: r_1xy.get(2)?,
        ma_dv: r_1xy.get(3)?,
        mang_cvid: r_1xy.get(4)?,
        ma_ho_so: r_1xy.get(5)?,
        hop_tai_lieu_so: r_1xy.get(6)?,
        so_bo: r_1xy.get(7)?,
        so_tap: r_1xy.get(8)?,
        hososo: r_1xy.get(9)?,
        tieu_de_hs: r_1xy.get(10)?,
        thbq: r_1xy.get(11)?,
        thoi_han_bao_quan_str: r_1xy.get(12)?,
        so_to: r_1xy.get(13)?,
        thoi_gian_bat_dau: r_1xy.get(14)?,
        thoi_gian_ket_thuc: r_1xy.get(15)?,
        trung_thua: r_1xy.get(16)?,
        so_luong_trang: r_1xy.get(17)?,
        ghi_chu: r_1xy.get(18)?,
        so_van_ban: r_1xy.get(19)?,
        ten_kho: r_1xy.get(20)?,
        ten_gia: r_1xy.get(21)?,
        status: r_1xy.get(22)?,
    })
}
fn f_2d7() -> String {
    format!(
        "d.id, d.ma_da, p.ten_du_an, d.ma_dv, d.mang_cvid, d.ma_ho_so, d.hop_so, d.bo_so, \
         d.tap_so, d.so_tam, d.tieu_de, d.thbq, t.ten_thbq, d.so_to, d.ngay_bat_dau, \
         d.ngay_ket_thuc, d.trung_thua, d.so_luong_trang, d.ghi_chu, \
         (SELECT COUNT(*) FROM van_ban v WHERE v.dossier_id = d.id) AS so_van_ban, \
         k.ten_kho, g.ten_gia, {} AS status",
        f_2he()
    )
}
const ORDER_BY: &str = "ORDER BY (d.thu_tu_sap_xep IS NULL), d.thu_tu_sap_xep, d.hop_so, d.id";
pub fn count(r_1xz: &Connection, r_1y0: &KhaiThacFilter) -> rusqlite::Result<i64> {
    let r_1y2 = format!("SELECT COUNT(*) {FROM_JOIN} {}", f_2qw(r_1y0));
    r_1xz.query_row(
        &r_1y2,
        params![r_1y0.ma_da_opt(), r_1y0.ma_dv_opt(), r_1y0.search_opt()],
        |r_1y1| r_1y1.get(0),
    )
}
pub fn search(r_1y3: &Connection, r_1y4: &KhaiThacFilter) -> rusqlite::Result<Vec<Ta5>> {
    let r_1y5 = r_1y4.page.max(1);
    let size = r_1y4.page_size.clamp(1, 500);
    let offset = (r_1y5 - 1) * size;
    let r_1y7 = format!(
        "SELECT {} {FROM_JOIN} {} {ORDER_BY} LIMIT {size} OFFSET {offset}",
        f_2d7(),
        f_2qw(r_1y4)
    );
    let mut r_1y8 = r_1y3.prepare(&r_1y7)?;
    let r_1y6 = r_1y8
        .query_map(
            params![r_1y4.ma_da_opt(), r_1y4.ma_dv_opt(), r_1y4.search_opt()],
            f_1qy,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1y6)
}
pub fn f_2cr(r_1y9: &Connection, r_1ya: &KhaiThacFilter) -> rusqlite::Result<Vec<Ta5>> {
    let r_1yc = format!(
        "SELECT {} {FROM_JOIN} {} {ORDER_BY} LIMIT 100000",
        f_2d7(),
        f_2qw(r_1ya)
    );
    let mut r_1yd = r_1y9.prepare(&r_1yc)?;
    let r_1yb = r_1yd
        .query_map(
            params![r_1ya.ma_da_opt(), r_1ya.ma_dv_opt(), r_1ya.search_opt()],
            f_1qy,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1yb)
}
pub fn f_2pe(r_1ye: &Connection, r_1yf: &KhaiThacFilter) -> rusqlite::Result<Vec<Ta6>> {
    let r_1yi = format!(
        "SELECT v.dossier_id, d.so_tam, d.hop_so, v.thu_tu, v.loai_van_ban, v.so_ky_hieu, \
                v.ngay_thang, v.co_quan_ban_hanh, v.trich_yeu, v.but_tich, v.to_so, \
                v.so_trang, v.trang_so, v.ghi_chu \
         FROM van_ban v JOIN dossier d ON d.id = v.dossier_id \
         LEFT JOIN phong p ON p.ma_du_an = d.ma_da \
         LEFT JOIN thoi_han_bq t ON t.thbqid = d.thbq \
         LEFT JOIN gia g ON g.ma_gia = d.ma_gia \
         {} \
         ORDER BY d.hop_so, d.so_tam, v.thu_tu, v.id LIMIT 100000",
        f_2qw(r_1yf)
    );
    let mut r_1yj = r_1ye.prepare(&r_1yi)?;
    let r_1yh = r_1yj
        .query_map(
            params![r_1yf.ma_da_opt(), r_1yf.ma_dv_opt(), r_1yf.search_opt()],
            |r_1yg| {
                Ok(Ta6 {
                    dossier_id: r_1yg.get(0)?,
                    ho_so: r_1yg.get(1)?,
                    hop_so: r_1yg.get(2)?,
                    thu_tu: r_1yg.get(3)?,
                    loai_van_ban: r_1yg.get(4)?,
                    so_ky_hieu: r_1yg.get(5)?,
                    ngay_thang: r_1yg.get(6)?,
                    co_quan_ban_hanh: r_1yg.get(7)?,
                    trich_yeu: r_1yg.get(8)?,
                    but_tich: r_1yg.get(9)?,
                    to_so: r_1yg.get(10)?,
                    so_trang: r_1yg.get(11)?,
                    trang_so: r_1yg.get(12)?,
                    ghi_chu: r_1yg.get(13)?,
                })
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1yh)
}
pub fn f_1nq(r_1yk: &Connection) -> rusqlite::Result<Vec<Ta9>> {
    let mut r_1yn =
        r_1yk.prepare("SELECT ma_kho, ten_kho FROM kho ORDER BY (so_tt IS NULL), so_tt, ma_kho")?;
    let r_1ym = r_1yn
        .query_map([], |r_1yl| {
            Ok(Ta9 {
                ma_kho: r_1yl.get(0)?,
                ten_kho: r_1yl.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1ym)
}
pub fn f_1nl(r_1yo: &Connection, ma_kho: Option<i64>) -> rusqlite::Result<Vec<T8o>> {
    let mut r_1yr = r_1yo.prepare(
        "SELECT ma_gia, ten_gia, ma_kho FROM gia \
         WHERE (?1 IS NULL OR ma_kho = ?1) ORDER BY (so_tt IS NULL), so_tt, ma_gia",
    )?;
    let r_1yq = r_1yr
        .query_map(params![ma_kho], |r_1yp| {
            Ok(T8o {
                ma_gia: r_1yp.get(0)?,
                ten_gia: r_1yp.get(1)?,
                ma_kho: r_1yp.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1yq)
}
pub fn f_1nr(r_1ys: &Connection) -> rusqlite::Result<Vec<Ta7>> {
    let mut r_1yv = r_1ys.prepare(
        "SELECT k.ma_kho, k.ten_kho, k.so_do_kho, k.ghi_chu, k.so_tt, \
                (SELECT COUNT(*) FROM gia g WHERE g.ma_kho = k.ma_kho) AS so_gia \
         FROM kho k ORDER BY (k.so_tt IS NULL), k.so_tt, k.ma_kho",
    )?;
    let r_1yu = r_1yv
        .query_map([], |r_1yt| {
            Ok(Ta7 {
                ma_kho: r_1yt.get(0)?,
                ten_kho: r_1yt.get(1)?,
                so_do_kho: r_1yt.get(2)?,
                ghi_chu: r_1yt.get(3)?,
                so_tt: r_1yt.get(4)?,
                so_gia: r_1yt.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1yu)
}
pub fn f_2o2(r_1yw: &Connection, k: &Ta8) -> rusqlite::Result<i64> {
    if let Some(id) = k.ma_kho {
        r_1yw.execute(
            "UPDATE kho SET ten_kho = ?2, so_do_kho = ?3, ghi_chu = ?4, so_tt = ?5 \
             WHERE ma_kho = ?1",
            params![id, k.ten_kho, k.so_do_kho, k.ghi_chu, k.so_tt],
        )?;
        Ok(id)
    } else {
        r_1yw.execute(
            "INSERT INTO kho (ten_kho, so_do_kho, ghi_chu, so_tt) VALUES (?1, ?2, ?3, ?4)",
            params![k.ten_kho, k.so_do_kho, k.ghi_chu, k.so_tt],
        )?;
        Ok(r_1yw.last_insert_rowid())
    }
}
pub fn f_140(r_1yx: &Connection, ma_kho: i64) -> rusqlite::Result<bool> {
    r_1yx.execute(
        "UPDATE dossier SET ma_gia = NULL \
         WHERE ma_gia IN (SELECT ma_gia FROM gia WHERE ma_kho = ?1)",
        params![ma_kho],
    )?;
    let r_1yy = r_1yx.execute("DELETE FROM kho WHERE ma_kho = ?1", params![ma_kho])?;
    Ok(r_1yy > 0)
}
pub fn f_1nm(r_1yz: &Connection, ma_kho: Option<i64>) -> rusqlite::Result<Vec<T8l>> {
    let mut r_1z2 = r_1yz.prepare(
        "SELECT g.ma_gia, g.ten_gia, g.ma_kho, k.ten_kho, g.ma_da, p.ten_du_an, \
                g.ma_dv, pb.ten_don_vi, g.tu_hop, g.den_hop, g.ghi_chu, g.so_tt, \
                (SELECT COUNT(*) FROM dossier d WHERE d.ma_gia = g.ma_gia) AS so_ho_so \
         FROM gia g \
         LEFT JOIN kho k ON k.ma_kho = g.ma_kho \
         LEFT JOIN phong p ON p.ma_du_an = g.ma_da \
         LEFT JOIN phong_ban pb ON pb.ma_dv = g.ma_dv \
         WHERE (?1 IS NULL OR g.ma_kho = ?1) \
         ORDER BY (g.so_tt IS NULL), g.so_tt, g.ma_gia",
    )?;
    let r_1z1 = r_1z2
        .query_map(params![ma_kho], |r_1z0| {
            Ok(T8l {
                ma_gia: r_1z0.get(0)?,
                ten_gia: r_1z0.get(1)?,
                ma_kho: r_1z0.get(2)?,
                ten_kho: r_1z0.get(3)?,
                ma_da: r_1z0.get(4)?,
                ten_du_an: r_1z0.get(5)?,
                ma_dv: r_1z0.get(6)?,
                ten_don_vi: r_1z0.get(7)?,
                tu_hop: r_1z0.get(8)?,
                den_hop: r_1z0.get(9)?,
                ghi_chu: r_1z0.get(10)?,
                so_tt: r_1z0.get(11)?,
                so_ho_so: r_1z0.get(12)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1z1)
}
pub fn f_2ny(r_1z3: &Connection, g: &T8m) -> rusqlite::Result<i64> {
    if let Some(id) = g.ma_gia {
        r_1z3.execute(
            "UPDATE gia SET ten_gia = ?2, ma_kho = ?3, ma_da = ?4, ma_dv = ?5, tu_hop = ?6, \
                den_hop = ?7, ghi_chu = ?8, so_tt = ?9 WHERE ma_gia = ?1",
            params![
                id, g.ten_gia, g.ma_kho, g.ma_da, g.ma_dv, g.tu_hop, g.den_hop, g.ghi_chu, g.so_tt
            ],
        )?;
        Ok(id)
    } else {
        r_1z3.execute(
            "INSERT INTO gia (ten_gia, ma_kho, ma_da, ma_dv, tu_hop, den_hop, ghi_chu, so_tt) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![g.ten_gia, g.ma_kho, g.ma_da, g.ma_dv, g.tu_hop, g.den_hop, g.ghi_chu, g.so_tt],
        )?;
        Ok(r_1z3.last_insert_rowid())
    }
}
pub fn f_13w(r_1z4: &Connection, ma_gia: i64) -> rusqlite::Result<bool> {
    r_1z4.execute(
        "UPDATE dossier SET ma_gia = NULL WHERE ma_gia = ?1",
        params![ma_gia],
    )?;
    let r_1z5 = r_1z4.execute("DELETE FROM gia WHERE ma_gia = ?1", params![ma_gia])?;
    Ok(r_1z5 > 0)
}
pub fn f_sh(r_1z6: &Connection, ma_gia: Option<i64>, r_1z7: &[i64]) -> rusqlite::Result<usize> {
    let mut r_1z9 = r_1z6.prepare("UPDATE dossier SET ma_gia = ?1 WHERE id = ?2")?;
    let mut r_1z8 = 0;
    for id in r_1z7 {
        r_1z8 += r_1z9.execute(params![ma_gia, id])?;
    }
    Ok(r_1z8)
}
pub fn f_si(r_1za: &Connection, ma_gia: i64) -> rusqlite::Result<usize> {
    let r_1zb = r_1za.execute(
        "UPDATE dossier SET ma_gia = ?1 WHERE id IN ( \
            SELECT d.id FROM dossier d JOIN gia g ON g.ma_gia = ?1 \
            WHERE g.tu_hop IS NOT NULL AND g.den_hop IS NOT NULL \
              AND CAST(NULLIF(TRIM(d.hop_so), '') AS INTEGER) BETWEEN g.tu_hop AND g.den_hop \
              AND (g.ma_da IS NULL OR d.ma_da = g.ma_da) \
              AND (g.ma_dv IS NULL OR d.ma_dv = g.ma_dv) \
         )",
        params![ma_gia],
    )?;
    Ok(r_1zb)
}
