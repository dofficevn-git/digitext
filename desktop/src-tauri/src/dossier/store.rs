// Copyright (c) 2026 DigiText. All rights reserved.
use super::types::{DossierFilter, T9r, Tkl, Tkp, Tmf, Tpq, VanBanIn, T54, T55};
use rusqlite::{params, Connection, OptionalExtension};
pub const C_CC: i64 = 200;
pub const C_JP: &str = r#"
CREATE TABLE IF NOT EXISTS dossier (
    id                INTEGER PRIMARY KEY,
    ma_da             TEXT REFERENCES phong(ma_du_an) ON DELETE SET NULL,
    ma_dv             TEXT,
    mang_cvid         INTEGER,
    thbq              INTEGER,
    so_tam            TEXT NOT NULL,
    ma_ho_so          TEXT,
    tieu_de           TEXT NOT NULL DEFAULT '',
    ngay_bat_dau      TEXT,
    ngay_ket_thuc     TEXT,
    so_to             TEXT,
    trung_thua        INTEGER NOT NULL DEFAULT 0,
    tong_so_van_ban   INTEGER,
    so_luong_trang    INTEGER,
    che_do_su_dung    TEXT,
    ngon_ngu          TEXT,
    tinh_trang_vat_ly TEXT,
    ghi_chu           TEXT,
    hop_so            TEXT,
    bo_so             TEXT,
    tap_so            TEXT,
    trang_thai        TEXT NOT NULL DEFAULT 'temp',
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_dossier_sotam ON dossier (ma_da, COALESCE(thbq, 0), so_tam);
CREATE INDEX IF NOT EXISTS ix_dossier_phong ON dossier (ma_da);

CREATE TABLE IF NOT EXISTS van_ban (
    id                INTEGER PRIMARY KEY,
    dossier_id        INTEGER NOT NULL REFERENCES dossier(id) ON DELETE CASCADE,
    thu_tu            INTEGER NOT NULL DEFAULT 0,
    loai_van_ban      TEXT,
    so_ky_hieu        TEXT,
    ngay_thang        TEXT,
    co_quan_ban_hanh  TEXT,
    trich_yeu         TEXT,
    to_so             INTEGER,
    so_trang          INTEGER,
    trang_so          INTEGER,
    ngon_ngu          TEXT,
    che_do_su_dung    TEXT,
    do_tin_cay        TEXT,
    but_tich          TEXT,
    tinh_trang_vat_ly TEXT,
    tai_lieu_so       TEXT,
    ghi_chu           TEXT,
    pdf_path          TEXT,
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT
);
CREATE INDEX IF NOT EXISTS ix_van_ban_dossier ON van_ban (dossier_id);
"#;
pub const C_JR: &str = "ALTER TABLE dossier ADD COLUMN thu_tu_sap_xep INTEGER;";
fn f_2c4(d_alias: &str, h_alias: &str) -> String {
    format!(
        "(
           ({h_alias}.phong_id IS {d_alias}.ma_da
            AND TRIM(COALESCE({h_alias}.ho_so_so, '')) = TRIM(COALESCE({d_alias}.so_tam, '')))
           OR
           ({d_alias}.ma_da IS NULL
            AND TRIM(COALESCE({h_alias}.ho_so_so, '')) = TRIM(COALESCE({d_alias}.so_tam, ''))
            AND TRIM(COALESCE({h_alias}.tieu_de, '')) = TRIM(COALESCE({d_alias}.tieu_de, '')))
         )"
    )
}
fn f_2c5(d_alias: &str, h_alias: &str) -> String {
    format!(
        "
            CASE
              WHEN NULLIF(TRIM(COALESCE({d_alias}.hop_so, '')), '') IS NOT NULL
                   AND TRIM(COALESCE({h_alias}.hop_so, '')) = TRIM(COALESCE({d_alias}.hop_so, '')) THEN 4
              WHEN NULLIF(TRIM(COALESCE({d_alias}.hop_so, '')), '') IS NULL
                   AND NULLIF(TRIM(COALESCE({h_alias}.hop_so, '')), '') IS NULL THEN 3
              ELSE 0
            END DESC,
            CASE
              WHEN NULLIF(TRIM(COALESCE({d_alias}.tieu_de, '')), '') IS NOT NULL
                   AND LOWER(TRIM(COALESCE({h_alias}.tieu_de, ''))) = LOWER(TRIM(COALESCE({d_alias}.tieu_de, ''))) THEN 2
              WHEN NULLIF(TRIM(COALESCE({d_alias}.tieu_de, '')), '') IS NULL
                   AND NULLIF(TRIM(COALESCE({h_alias}.tieu_de, '')), '') IS NULL THEN 1
              ELSE 0
            END DESC,
            COALESCE({h_alias}.page_count, 0) DESC,
            COALESCE({h_alias}.updated_at, {h_alias}.created_at, '') DESC,
            {h_alias}.id DESC"
    )
}
fn f_16z() -> String {
    let on = f_2c4("d", "h");
    let order = f_2c5("d", "h");
    format!(
        "LEFT JOIN (
            SELECT dossier_id, folder_path, pdf_path
            FROM (
                SELECT d.id AS dossier_id,
                       h.folder_path AS folder_path,
                       h.pdf_path AS pdf_path,
                       ROW_NUMBER() OVER (PARTITION BY d.id ORDER BY {order}) AS rn
                FROM dossier d
                LEFT JOIN ho_so h ON {on}
            ) ranked
            WHERE rn = 1
         ) sm ON sm.dossier_id = d.id"
    )
}
const DOSSIER_COLS: &str = "d.id, d.ma_da, p.ten_du_an, d.ma_dv, d.mang_cvid, d.thbq, d.so_tam,
        d.ma_ho_so, d.tieu_de, d.ngay_bat_dau, d.ngay_ket_thuc, d.so_to, d.trung_thua,
        d.tong_so_van_ban, d.so_luong_trang, d.che_do_su_dung, d.ngon_ngu, d.tinh_trang_vat_ly,
        d.ghi_chu, d.hop_so, d.bo_so, d.tap_so, d.trang_thai,
        (SELECT COUNT(*) FROM van_ban v WHERE v.dossier_id = d.id) AS so_van_ban,
        sm.folder_path AS scan_folder_path,
        sm.pdf_path AS scan_pdf_path,
        (SELECT COUNT(*) FROM van_ban v WHERE v.dossier_id = d.id
                AND v.pdf_path IS NOT NULL AND TRIM(v.pdf_path) <> '') AS so_file_scan,
        d.created_at, d.updated_at, d.thu_tu_sap_xep";
pub(crate) fn f_299(r_1ot: Option<&str>, pdf_path: Option<&str>) -> Option<std::path::PathBuf> {
    r_1ot
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| {
            pdf_path
                .filter(|s| !s.trim().is_empty())
                .and_then(|p| std::path::Path::new(p).parent())
                .map(|d| d.to_path_buf())
        })
}
pub(crate) fn f_219(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let Ok(r_1ov) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut r_1oz: Vec<std::path::PathBuf> = r_1ov
        .flatten()
        .filter_map(|r_1ou| {
            let r_1p0 = r_1ou.path();
            let r_1oy = r_1p0
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|r_1ow| r_1ow.eq_ignore_ascii_case("pdf"));
            let r_1ox = r_1ou.file_type().map(|t| t.is_file()).unwrap_or(true);
            (r_1oy && r_1ox).then_some(r_1p0)
        })
        .collect();
    r_1oz.sort();
    r_1oz
}
fn f_124(r_1p1: Option<&str>, pdf_path: Option<&str>) -> i64 {
    match f_299(r_1p1, pdf_path) {
        Some(dir) => f_219(&dir).len() as i64,
        None => 0,
    }
}
pub(crate) fn f_122(r_1p2: &std::path::Path) -> i64 {
    lopdf::Document::load(r_1p2)
        .map(|d| d.get_pages().len() as i64)
        .unwrap_or(0)
}
fn f_20p(pdf_path: Option<&str>) -> Option<String> {
    pdf_path
        .filter(|s| !s.trim().is_empty())
        .and_then(|p| std::path::Path::new(p).file_name())
        .map(|r_1p3| r_1p3.to_string_lossy().to_lowercase())
}
pub fn f_1oa(dir: &std::path::Path, van_ban: &[Tpq]) -> Vec<Tkp> {
    let mut r_1p4: std::collections::HashMap<String, &Tpq> = std::collections::HashMap::new();
    for v in van_ban {
        if let Some(r_1p6) = f_20p(v.pdf_path.as_deref()) {
            r_1p4.entry(r_1p6).or_insert(v);
        }
    }
    f_219(dir)
        .into_iter()
        .map(|p| {
            let r_1p5 = p
                .file_name()
                .map(|r_1p7| r_1p7.to_string_lossy().into_owned())
                .unwrap_or_default();
            let vb = r_1p4.get(&r_1p5.to_lowercase());
            Tkp {
                page_count: f_122(&p),
                file_path: p.to_string_lossy().into_owned(),
                loai_van_ban: vb.and_then(|v| v.loai_van_ban.clone()),
                so_ky_hieu: vb.and_then(|v| v.so_ky_hieu.clone()),
                ngay_thang: vb.and_then(|v| v.ngay_thang.clone()),
                tac_gia: vb.and_then(|v| v.co_quan_ban_hanh.clone()),
                trich_yeu: vb.and_then(|v| v.trich_yeu.clone()),
                tinh_trang_vat_ly: vb.and_then(|v| v.tinh_trang_vat_ly.clone()),
                file_name: r_1p5,
            }
        })
        .collect()
}
pub fn f_1rn(r_1p8: &Connection, dossier_id: i64) -> rusqlite::Result<Option<String>> {
    let on = f_2c4("d", "h");
    let order = f_2c5("d", "h");
    let r_1pc = format!(
        "SELECT folder_path, pdf_path FROM (
            SELECT h.folder_path AS folder_path, h.pdf_path AS pdf_path,
                   ROW_NUMBER() OVER (PARTITION BY d.id ORDER BY {order}) AS rn
            FROM dossier d LEFT JOIN ho_so h ON {on}
            WHERE d.id = ?1
         ) ranked WHERE rn = 1"
    );
    let r_1pa = r_1p8
        .query_row(&r_1pc, params![dossier_id], |r_1pb| {
            let r_1p9: Option<String> = r_1pb.get(0)?;
            let pdf: Option<String> = r_1pb.get(1)?;
            Ok((r_1p9, pdf))
        })
        .optional()?;
    Ok(r_1pa.and_then(|(f, p)| {
        f_299(f.as_deref(), p.as_deref()).map(|d| d.to_string_lossy().into_owned())
    }))
}
fn f_1qs(r_1pd: &rusqlite::Row) -> rusqlite::Result<T55> {
    let r_1pe: Option<String> = r_1pd.get(24)?;
    let r_1pf: Option<String> = r_1pd.get(25)?;
    let r_1pg = f_124(r_1pe.as_deref(), r_1pf.as_deref());
    Ok(T55 {
        id: r_1pd.get(0)?,
        ma_da: r_1pd.get(1)?,
        ten_da: r_1pd.get(2)?,
        ma_dv: r_1pd.get(3)?,
        mang_cvid: r_1pd.get(4)?,
        thbq: r_1pd.get(5)?,
        so_tam: r_1pd.get(6)?,
        ma_ho_so: r_1pd.get(7)?,
        tieu_de: r_1pd.get(8)?,
        ngay_bat_dau: r_1pd.get(9)?,
        ngay_ket_thuc: r_1pd.get(10)?,
        so_to: r_1pd.get(11)?,
        trung_thua: r_1pd.get(12)?,
        tong_so_van_ban: r_1pd.get(13)?,
        so_luong_trang: r_1pd.get(14)?,
        che_do_su_dung: r_1pd.get(15)?,
        ngon_ngu: r_1pd.get(16)?,
        tinh_trang_vat_ly: r_1pd.get(17)?,
        ghi_chu: r_1pd.get(18)?,
        hop_so: r_1pd.get(19)?,
        bo_so: r_1pd.get(20)?,
        tap_so: r_1pd.get(21)?,
        trang_thai: r_1pd.get(22)?,
        so_van_ban: r_1pd.get(23)?,
        so_tai_lieu_scan: r_1pg,
        scan_pdf_path: r_1pf,
        so_file_scan: r_1pd.get(26)?,
        created_at: r_1pd.get(27)?,
        updated_at: r_1pd.get(28)?,
        thu_tu_sap_xep: r_1pd.get(29)?,
    })
}
pub fn f_2gf(
    r_1ph: &Connection,
    ma_da: Option<&str>,
    so_tam: &str,
    r_1pi: Option<i64>,
) -> rusqlite::Result<bool> {
    r_1ph
        .query_row(
            "SELECT 1 FROM dossier
         WHERE ma_da IS ?1 AND so_tam = ?2 AND (?3 IS NULL OR id <> ?3) LIMIT 1",
            params![ma_da, so_tam, r_1pi],
            |_| Ok(()),
        )
        .optional()
        .map(|r_1pj| r_1pj.is_some())
}
pub fn f_128(r_1pk: &Connection, d: &T54) -> rusqlite::Result<i64> {
    r_1pk
        .execute(
            "INSERT INTO dossier
            (ma_da, ma_dv, mang_cvid, thbq, so_tam, ma_ho_so, tieu_de, ngay_bat_dau, ngay_ket_thuc,
             so_to, trung_thua, tong_so_van_ban, so_luong_trang, che_do_su_dung, ngon_ngu,
             tinh_trang_vat_ly, ghi_chu, hop_so, bo_so, tap_so, trang_thai, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21, datetime('now'))",
            params![
                d.ma_da, d.ma_dv, d.mang_cvid, d.thbq, d.so_tam, d.ma_ho_so, d.tieu_de, d
                .ngay_bat_dau, d.ngay_ket_thuc, d.so_to, d.trung_thua, d.tong_so_van_ban,
                d.so_luong_trang, d.che_do_su_dung, d.ngon_ngu, d.tinh_trang_vat_ly, d
                .ghi_chu, d.hop_so, d.bo_so, d.tap_so, d.trang_thai_or_default(),
            ],
        )?;
    Ok(r_1pk.last_insert_rowid())
}
pub fn f_2if(r_1pl: &Connection, ma_da: Option<&str>, r_1pp: &str) -> rusqlite::Result<Tmf> {
    let r_1po: i64 = r_1pl.query_row(
        "SELECT COUNT(*) FROM (
            SELECT h.phong_id
            FROM ho_so h
            WHERE (?1 IS NULL OR h.phong_id = ?1)
              AND h.phong_id IS NOT NULL
              AND TRIM(COALESCE(h.ho_so_so, '')) <> ''
              AND EXISTS (
                 SELECT 1 FROM dossier d
                 WHERE d.ma_da = h.phong_id AND TRIM(d.so_tam) = TRIM(h.ho_so_so)
              )
            GROUP BY h.phong_id, TRIM(h.ho_so_so)
         )",
        params![ma_da],
        |r_1pn| r_1pn.get(0),
    )?;
    let r_1pm = r_1pl.execute(
        "INSERT INTO dossier
            (ma_da, so_tam, ma_ho_so, tieu_de, hop_so, so_to, ngay_bat_dau, ngay_ket_thuc,
             ghi_chu, thbq, so_luong_trang, trang_thai, created_by)
         SELECT s.phong_id, s.hs, s.ma_ho_so, s.tieu_de, s.hop_so, s.so_to, s.nbd, s.nkt, s.ghi_chu,
                (SELECT t.thbqid FROM thoi_han_bq t
                 WHERE LOWER(TRIM(t.ten_thbq)) = LOWER(s.thbq_name) LIMIT 1),
                s.page_count, 'temp', COALESCE(s.created_by, ?2)
         FROM (
            SELECT h.phong_id AS phong_id, TRIM(h.ho_so_so) AS hs,
                   MAX(h.ho_so_so) AS ma_ho_so, COALESCE(MAX(h.tieu_de), '') AS tieu_de,
                   MAX(h.hop_so) AS hop_so, MAX(h.so_to) AS so_to,
                   MAX(h.ngay_bat_dau) AS nbd, MAX(h.ngay_ket_thuc) AS nkt,
                   MAX(h.ghi_chu) AS ghi_chu, TRIM(COALESCE(MAX(h.thbq), '')) AS thbq_name,
                   MAX(h.page_count) AS page_count,
                   COALESCE(MAX(h.created_by), MAX(h.finalized_by)) AS created_by
            FROM ho_so h
            WHERE (?1 IS NULL OR h.phong_id = ?1)
              AND h.phong_id IS NOT NULL
              AND TRIM(COALESCE(h.ho_so_so, '')) <> ''
              AND NOT EXISTS (
                 SELECT 1 FROM dossier d
                 WHERE d.ma_da = h.phong_id AND TRIM(d.so_tam) = TRIM(h.ho_so_so)
              )
            GROUP BY h.phong_id, TRIM(h.ho_so_so)
         ) s",
        params![ma_da, r_1pp],
    )?;
    Ok(Tmf {
        created: r_1pm,
        skipped_existing: r_1po as usize,
    })
}
pub fn f_2ne(r_1pq: &Connection, id: i64, d: &T54) -> rusqlite::Result<bool> {
    let r_1pr = r_1pq.execute(
        "UPDATE dossier SET
            ma_da=?2, ma_dv=?3, mang_cvid=?4, thbq=?5, so_tam=?6, ma_ho_so=?7, tieu_de=?8,
            ngay_bat_dau=?9, ngay_ket_thuc=?10, so_to=?11, trung_thua=?12, tong_so_van_ban=?13,
            so_luong_trang=?14, che_do_su_dung=?15, ngon_ngu=?16, tinh_trang_vat_ly=?17,
            ghi_chu=?18, hop_so=?19, bo_so=?20, tap_so=?21, trang_thai=?22,
            updated_at=datetime('now')
         WHERE id=?1",
        params![
            id,
            d.ma_da,
            d.ma_dv,
            d.mang_cvid,
            d.thbq,
            d.so_tam,
            d.ma_ho_so,
            d.tieu_de,
            d.ngay_bat_dau,
            d.ngay_ket_thuc,
            d.so_to,
            d.trung_thua,
            d.tong_so_van_ban,
            d.so_luong_trang,
            d.che_do_su_dung,
            d.ngon_ngu,
            d.tinh_trang_vat_ly,
            d.ghi_chu,
            d.hop_so,
            d.bo_so,
            d.tap_so,
            d.trang_thai_or_default(),
        ],
    )?;
    Ok(r_1pr > 0)
}
pub fn f_13v(r_1ps: &Connection, id: i64) -> rusqlite::Result<bool> {
    let r_1pt = r_1ps.execute("DELETE FROM dossier WHERE id=?1", params![id])?;
    Ok(r_1pt > 0)
}
pub fn f_1nc(r_1pu: &Connection, f: &DossierFilter) -> rusqlite::Result<Vec<T55>> {
    let scan_joins = f_16z();
    let r_1pw = format!(
        "SELECT {DOSSIER_COLS}
         FROM dossier d LEFT JOIN phong p ON p.ma_du_an = d.ma_da
         {scan_joins}
         WHERE (?1 IS NULL OR d.ma_da = ?1)
           AND (?2 IS NULL OR d.ma_dv = ?2)
           AND (?3 IS NULL OR d.mang_cvid = ?3)
           AND (?4 IS NULL OR d.thbq = ?4)
           AND (?5 IS NULL OR d.tieu_de LIKE '%'||?5||'%'
                           OR d.so_tam LIKE '%'||?5||'%'
                           OR IFNULL(d.ma_ho_so,'') LIKE '%'||?5||'%')
         ORDER BY d.created_at DESC, d.id DESC
         LIMIT 5000"
    );
    let mut r_1px = r_1pu.prepare(&r_1pw)?;
    let r_1pv = r_1px
        .query_map(
            params![
                f.ma_da_opt(),
                f.ma_dv_opt(),
                f.mang_cvid,
                f.thbq,
                f.search_opt()
            ],
            f_1qs,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1pv)
}
pub fn f_1ek(r_1py: &Connection, id: i64) -> rusqlite::Result<Option<T55>> {
    let scan_joins = f_16z();
    let r_1pz = format!(
        "SELECT {DOSSIER_COLS}
         FROM dossier d LEFT JOIN phong p ON p.ma_du_an = d.ma_da
         {scan_joins}
         WHERE d.id = ?1"
    );
    r_1py.query_row(&r_1pz, params![id], f_1qs).optional()
}
const INCOMPLETE_WHERE: &str = "(v.pdf_path IS NULL OR TRIM(v.pdf_path) = '')
       AND (?1 IS NULL OR d.ma_da = ?1)
       AND (?2 IS NULL OR d.ma_dv = ?2)
       AND (?3 IS NULL OR d.mang_cvid = ?3)
       AND (?4 IS NULL OR d.thbq = ?4)
       AND (?5 IS NULL OR d.tieu_de LIKE '%'||?5||'%'
                       OR d.so_tam LIKE '%'||?5||'%'
                       OR IFNULL(d.ma_ho_so,'') LIKE '%'||?5||'%')";
pub fn f_1no(r_1q0: &Connection, f: &DossierFilter) -> rusqlite::Result<Vec<T9r>> {
    let r_1q3 = format!(
        "SELECT v.id, v.dossier_id, d.so_tam, v.thu_tu, v.so_ky_hieu, v.trich_yeu, v.ngay_thang
         FROM van_ban v JOIN dossier d ON d.id = v.dossier_id
         WHERE {INCOMPLETE_WHERE}
         ORDER BY d.so_tam, v.thu_tu, v.id
         LIMIT 5000"
    );
    let mut r_1q4 = r_1q0.prepare(&r_1q3)?;
    let r_1q2 = r_1q4
        .query_map(
            params![
                f.ma_da_opt(),
                f.ma_dv_opt(),
                f.mang_cvid,
                f.thbq,
                f.search_opt()
            ],
            |r_1q1| {
                Ok(T9r {
                    van_ban_id: r_1q1.get(0)?,
                    dossier_id: r_1q1.get(1)?,
                    ho_so: r_1q1.get(2)?,
                    thu_tu: r_1q1.get(3)?,
                    so_ky_hieu: r_1q1.get(4)?,
                    trich_yeu: r_1q1.get(5)?,
                    ngay_thang: r_1q1.get(6)?,
                })
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1q2)
}
pub fn f_13y(r_1q5: &Connection, f: &DossierFilter) -> rusqlite::Result<usize> {
    let r_1q7 = format!(
        "DELETE FROM van_ban WHERE id IN (
            SELECT v.id FROM van_ban v JOIN dossier d ON d.id = v.dossier_id
            WHERE {INCOMPLETE_WHERE})"
    );
    let r_1q6 = r_1q5.execute(
        &r_1q7,
        params![
            f.ma_da_opt(),
            f.ma_dv_opt(),
            f.mang_cvid,
            f.thbq,
            f.search_opt()
        ],
    )?;
    Ok(r_1q6)
}
pub fn f_2c2(r_1q8: &Connection, r_1q9: &[i64], r_1qa: i64) -> rusqlite::Result<Vec<Tkl>> {
    if r_1q9.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = r_1q9.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let on = f_2c4("d", "h");
    let order = f_2c5("d", "h");
    let r_1qf = format!(
        "WITH matched AS (
            SELECT dossier_id, hop_so, so_tam, ho_so_id
            FROM (
                SELECT d.id AS dossier_id,
                       d.hop_so,
                       d.so_tam,
                       h.id AS ho_so_id,
                       ROW_NUMBER() OVER (PARTITION BY d.id ORDER BY {order}) AS rn
                FROM dossier d
                LEFT JOIN ho_so h ON {on}
                WHERE d.id IN ({placeholders})
            ) ranked
            WHERE rn = 1
         )
         SELECT t.image_path, IFNULL(t.dpi, 0)
         FROM matched m
         JOIN trang t ON t.ho_so_id = m.ho_so_id
         ORDER BY m.hop_so, m.so_tam, t.page_no, t.id"
    );
    let mut r_1qg = r_1q8.prepare(&r_1qf)?;
    let params = rusqlite::params_from_iter(r_1q9.iter());
    let r_1qe = r_1qg
        .query_map(params, |r_1qd| {
            let r_1qc: String = r_1qd.get(0)?;
            let dpi: i64 = r_1qd.get(1)?;
            let r_1qb = r_1qc
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or(&r_1qc)
                .to_string();
            Ok(Tkl {
                file_name: r_1qb,
                file_path: r_1qc,
                dpi,
                is_pass: dpi >= r_1qa,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1qe)
}
fn f_2il(r_1qh: &str) -> &'static str {
    match r_1qh {
        "thbqDesc" => "ORDER BY (t.sort IS NULL), t.sort DESC, d.ngay_bat_dau, d.id",
        "timeAsc" => "ORDER BY (d.ngay_bat_dau IS NULL), d.ngay_bat_dau, d.id",
        "timeDesc" => "ORDER BY (d.ngay_ket_thuc IS NULL), d.ngay_ket_thuc DESC, d.id",
        _ => "ORDER BY (t.sort IS NULL), t.sort, d.ngay_bat_dau, d.id",
    }
}
pub fn f_2ij(
    r_1qi: &Connection,
    ma_da: Option<&str>,
    r_1qn: &str,
    r_1ql: i64,
) -> rusqlite::Result<usize> {
    let r_1qo = format!(
        "SELECT d.id FROM dossier d LEFT JOIN thoi_han_bq t ON t.thbqid = d.thbq
         WHERE (?1 IS NULL OR d.ma_da = ?1) {}",
        f_2il(r_1qn)
    );
    let r_1qk: Vec<i64> = r_1qi
        .prepare(&r_1qo)?
        .query_map(params![ma_da], |r_1qm| r_1qm.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (i, id) in r_1qk.iter().enumerate() {
        let order = i as i64 + 1;
        if r_1ql > 0 {
            let r_1qj = (order - 1) / r_1ql + 1;
            r_1qi
                .execute(
                    "UPDATE dossier SET thu_tu_sap_xep=?2, hop_so=?3, updated_at=datetime('now') WHERE id=?1",
                    params![id, order, r_1qj.to_string()],
                )?;
        } else {
            r_1qi.execute(
                "UPDATE dossier SET thu_tu_sap_xep=?2, updated_at=datetime('now') WHERE id=?1",
                params![id, order],
            )?;
        }
    }
    Ok(r_1qk.len())
}
pub fn f_2e0(
    r_1qp: &Connection,
    id: i64,
    hop_so: Option<&str>,
    bo_so: Option<&str>,
    tap_so: Option<&str>,
) -> rusqlite::Result<bool> {
    let r_1qq = r_1qp.execute(
        "UPDATE dossier SET hop_so=?2, bo_so=?3, tap_so=?4, updated_at=datetime('now') WHERE id=?1",
        params![id, hop_so, bo_so, tap_so],
    )?;
    Ok(r_1qq > 0)
}
pub fn f_wd(r_1qr: &Connection, ma_da: Option<&str>, r_1qv: &[T54]) -> rusqlite::Result<usize> {
    let mut r_1qt = 0usize;
    for d in r_1qv {
        let so_tam = d.so_tam.trim();
        if so_tam.is_empty() {
            continue;
        }
        let r_1qs: Option<i64> = r_1qr
            .query_row(
                "SELECT id FROM dossier WHERE ma_da IS ?1 AND so_tam = ?2 LIMIT 1",
                params![ma_da, so_tam],
                |r_1qu| r_1qu.get(0),
            )
            .optional()?;
        match r_1qs {
            Some(id) => {
                r_1qr.execute(
                    "UPDATE dossier SET
                        ma_dv=?2, mang_cvid=?3, thbq=?4, ma_ho_so=?5, tieu_de=?6, ngay_bat_dau=?7,
                        ngay_ket_thuc=?8, so_to=?9, trung_thua=?10, tong_so_van_ban=?11,
                        so_luong_trang=?12, che_do_su_dung=?13, ngon_ngu=?14, tinh_trang_vat_ly=?15,
                        ghi_chu=?16, hop_so=?17, bo_so=?18, tap_so=?19, trang_thai=?20,
                        updated_at=datetime('now')
                     WHERE id=?1",
                    params![
                        id,
                        d.ma_dv,
                        d.mang_cvid,
                        d.thbq,
                        d.ma_ho_so,
                        d.tieu_de,
                        d.ngay_bat_dau,
                        d.ngay_ket_thuc,
                        d.so_to,
                        d.trung_thua,
                        d.tong_so_van_ban,
                        d.so_luong_trang,
                        d.che_do_su_dung,
                        d.ngon_ngu,
                        d.tinh_trang_vat_ly,
                        d.ghi_chu,
                        d.hop_so,
                        d.bo_so,
                        d.tap_so,
                        d.trang_thai_or_default(),
                    ],
                )?;
            }
            None => {
                r_1qr
                    .execute(
                        "INSERT INTO dossier
                        (ma_da, ma_dv, mang_cvid, thbq, so_tam, ma_ho_so, tieu_de, ngay_bat_dau,
                         ngay_ket_thuc, so_to, trung_thua, tong_so_van_ban, so_luong_trang,
                         che_do_su_dung, ngon_ngu, tinh_trang_vat_ly, ghi_chu, hop_so, bo_so, tap_so,
                         trang_thai, updated_at)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21, datetime('now'))",
                        params![
                            ma_da, d.ma_dv, d.mang_cvid, d.thbq, so_tam, d.ma_ho_so, d
                            .tieu_de, d.ngay_bat_dau, d.ngay_ket_thuc, d.so_to, d
                            .trung_thua, d.tong_so_van_ban, d.so_luong_trang, d
                            .che_do_su_dung, d.ngon_ngu, d.tinh_trang_vat_ly, d.ghi_chu,
                            d.hop_so, d.bo_so, d.tap_so, d.trang_thai_or_default(),
                        ],
                    )?;
            }
        }
        r_1qt += 1;
    }
    Ok(r_1qt)
}
fn f_1r2(r_1qw: &rusqlite::Row) -> rusqlite::Result<Tpq> {
    Ok(Tpq {
        id: r_1qw.get(0)?,
        dossier_id: r_1qw.get(1)?,
        thu_tu: r_1qw.get(2)?,
        loai_van_ban: r_1qw.get(3)?,
        so_ky_hieu: r_1qw.get(4)?,
        ngay_thang: r_1qw.get(5)?,
        co_quan_ban_hanh: r_1qw.get(6)?,
        trich_yeu: r_1qw.get(7)?,
        to_so: r_1qw.get(8)?,
        so_trang: r_1qw.get(9)?,
        trang_so: r_1qw.get(10)?,
        ngon_ngu: r_1qw.get(11)?,
        che_do_su_dung: r_1qw.get(12)?,
        do_tin_cay: r_1qw.get(13)?,
        but_tich: r_1qw.get(14)?,
        tinh_trang_vat_ly: r_1qw.get(15)?,
        tai_lieu_so: r_1qw.get(16)?,
        ghi_chu: r_1qw.get(17)?,
        pdf_path: r_1qw.get(18)?,
        created_at: r_1qw.get(19)?,
        updated_at: r_1qw.get(20)?,
    })
}
const VAN_BAN_COLS: &str = "id, dossier_id, thu_tu, loai_van_ban, so_ky_hieu, ngay_thang,
        co_quan_ban_hanh, trich_yeu, to_so, so_trang, trang_so, ngon_ngu, che_do_su_dung,
        do_tin_cay, but_tich, tinh_trang_vat_ly, tai_lieu_so, ghi_chu, pdf_path,
        created_at, updated_at";
pub fn f_16x(r_1qx: &Connection, id: i64) -> rusqlite::Result<bool> {
    r_1qx
        .query_row("SELECT 1 FROM dossier WHERE id=?1", params![id], |_| Ok(()))
        .optional()
        .map(|r_1qy| r_1qy.is_some())
}
pub fn f_12h(r_1qz: &Connection, dossier_id: i64, v: &VanBanIn) -> rusqlite::Result<i64> {
    let thu_tu = if v.thu_tu > 0 {
        v.thu_tu
    } else {
        r_1qz.query_row(
            "SELECT COALESCE(MAX(thu_tu), 0) + 1 FROM van_ban WHERE dossier_id=?1",
            params![dossier_id],
            |r_1r0| r_1r0.get(0),
        )?
    };
    r_1qz.execute(
        "INSERT INTO van_ban
            (dossier_id, thu_tu, loai_van_ban, so_ky_hieu, ngay_thang, co_quan_ban_hanh, trich_yeu,
             to_so, so_trang, trang_so, ngon_ngu, che_do_su_dung, do_tin_cay, but_tich,
             tinh_trang_vat_ly, tai_lieu_so, ghi_chu, pdf_path, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18, datetime('now'))",
        params![
            dossier_id,
            thu_tu,
            v.loai_van_ban,
            v.so_ky_hieu,
            v.ngay_thang,
            v.co_quan_ban_hanh,
            v.trich_yeu,
            v.to_so,
            v.so_trang,
            v.trang_so,
            v.ngon_ngu,
            v.che_do_su_dung,
            v.do_tin_cay,
            v.but_tich,
            v.tinh_trang_vat_ly,
            v.tai_lieu_so,
            v.ghi_chu,
            v.pdf_path,
        ],
    )?;
    Ok(r_1qz.last_insert_rowid())
}
pub fn f_2ni(r_1r1: &Connection, id: i64, v: &VanBanIn) -> rusqlite::Result<bool> {
    let r_1r2 = r_1r1.execute(
        "UPDATE van_ban SET
            thu_tu=?2, loai_van_ban=?3, so_ky_hieu=?4, ngay_thang=?5, co_quan_ban_hanh=?6,
            trich_yeu=?7, to_so=?8, so_trang=?9, trang_so=?10, ngon_ngu=?11, che_do_su_dung=?12,
            do_tin_cay=?13, but_tich=?14, tinh_trang_vat_ly=?15, tai_lieu_so=?16, ghi_chu=?17,
            pdf_path=?18, updated_at=datetime('now')
         WHERE id=?1",
        params![
            id,
            v.thu_tu,
            v.loai_van_ban,
            v.so_ky_hieu,
            v.ngay_thang,
            v.co_quan_ban_hanh,
            v.trich_yeu,
            v.to_so,
            v.so_trang,
            v.trang_so,
            v.ngon_ngu,
            v.che_do_su_dung,
            v.do_tin_cay,
            v.but_tich,
            v.tinh_trang_vat_ly,
            v.tai_lieu_so,
            v.ghi_chu,
            v.pdf_path,
        ],
    )?;
    Ok(r_1r2 > 0)
}
pub fn f_149(r_1r3: &Connection, id: i64) -> rusqlite::Result<bool> {
    let r_1r4 = r_1r3.execute("DELETE FROM van_ban WHERE id=?1", params![id])?;
    Ok(r_1r4 > 0)
}
pub fn f_1op(r_1r5: &Connection, dossier_id: i64) -> rusqlite::Result<Vec<Tpq>> {
    let r_1r7 =
        format!("SELECT {VAN_BAN_COLS} FROM van_ban WHERE dossier_id=?1 ORDER BY thu_tu, id");
    let mut r_1r8 = r_1r5.prepare(&r_1r7)?;
    let r_1r6 = r_1r8
        .query_map(params![dossier_id], f_1r2)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1r6)
}
pub fn f_16y(r_1r9: &Connection, id: i64) -> rusqlite::Result<Option<String>> {
    r_1r9
        .query_row(
            "SELECT ma_da FROM dossier WHERE id=?1",
            params![id],
            |r_1rb| r_1rb.get::<_, Option<String>>(0),
        )
        .optional()
        .map(|r_1ra| r_1ra.flatten())
}
pub fn f_2ph(r_1rc: &Connection, r_1re: i64) -> rusqlite::Result<Option<i64>> {
    r_1rc
        .query_row(
            "SELECT dossier_id FROM van_ban WHERE id=?1",
            params![r_1re],
            |r_1rd| r_1rd.get(0),
        )
        .optional()
}
pub fn f_2h3(
    r_1rf: &Connection,
    id: i64,
    created_by: Option<&str>,
    updated_by: Option<&str>,
) -> rusqlite::Result<()> {
    if let Some(u) = created_by {
        r_1rf.execute(
            "UPDATE dossier SET created_by = COALESCE(created_by, ?1) WHERE id = ?2",
            params![u, id],
        )?;
    }
    if let Some(u) = updated_by {
        r_1rf.execute(
            "UPDATE dossier SET updated_by = ?1 WHERE id = ?2",
            params![u, id],
        )?;
    }
    Ok(())
}
pub fn f_2h8(r_1rg: &Connection, id: i64, created_by: &str) -> rusqlite::Result<()> {
    r_1rg.execute(
        "UPDATE van_ban SET created_by = COALESCE(created_by, ?1) WHERE id = ?2",
        params![created_by, id],
    )?;
    Ok(())
}
pub fn f_wa(r_1rh: &Connection, r_1ri: &[(i64, VanBanIn)]) -> rusqlite::Result<usize> {
    let mut r_1rj = 0usize;
    for (dossier_id, v) in r_1ri {
        if !f_16x(r_1rh, *dossier_id)? {
            continue;
        }
        f_12h(r_1rh, *dossier_id, v)?;
        r_1rj += 1;
    }
    Ok(r_1rj)
}
