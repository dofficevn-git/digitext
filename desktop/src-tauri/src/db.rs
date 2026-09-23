// Copyright (c) 2026 DigiText. All rights reserved.
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;
pub struct Db(pub Mutex<Connection>);
const C_P1: i64 = 19;
pub(crate) const C_JI: &str = r#"
CREATE TABLE IF NOT EXISTS phong (
    ma_du_an    TEXT PRIMARY KEY,
    ten_du_an   TEXT NOT NULL,
    ma_khoi     TEXT,
    phong_so    TEXT,
    muc_luc_so  TEXT
);

CREATE TABLE IF NOT EXISTS ho_so (
    id           INTEGER PRIMARY KEY,
    phong_id     TEXT REFERENCES phong(ma_du_an) ON DELETE SET NULL,
    hop_so       TEXT NOT NULL,
    ho_so_so     TEXT NOT NULL,
    tieu_de      TEXT NOT NULL DEFAULT '',
    thoi_gian    TEXT,
    so_to        TEXT,
    thbq         TEXT,
    ghi_chu      TEXT,
    nhom         TEXT,
    nam          TEXT,
    ngay_bat_dau TEXT,
    ngay_ket_thuc TEXT,
    folder_path  TEXT,
    pdf_path     TEXT,
    pdf_size     INTEGER,
    page_count   INTEGER,
    status       TEXT NOT NULL DEFAULT 'pending',
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_ho_so_key ON ho_so (phong_id, hop_so, ho_so_so);

CREATE TABLE IF NOT EXISTS trang (
    id          INTEGER PRIMARY KEY,
    ho_so_id    INTEGER NOT NULL REFERENCES ho_so(id) ON DELETE CASCADE,
    page_no     INTEGER NOT NULL,
    image_path  TEXT NOT NULL,
    width_px    INTEGER,
    height_px   INTEGER,
    dpi         INTEGER,
    paper       TEXT
);
CREATE INDEX IF NOT EXISTS ix_trang_hoso ON trang (ho_so_id);
"#;
const C_JO: &str = r#"
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT
);
"#;
const C_JJ: &str = r#"
CREATE TABLE IF NOT EXISTS collected_docs (
    id              INTEGER PRIMARY KEY,
    source_user_id  TEXT NOT NULL,          -- người quét gốc (onBehalfOf khi đẩy server)
    source_username TEXT,
    project_id      TEXT NOT NULL,          -- projectId phía server
    ho_so_ref       TEXT,
    filename        TEXT NOT NULL,
    page_count      INTEGER,
    size_bytes      INTEGER NOT NULL,
    blake3          TEXT NOT NULL,
    storage_path    TEXT NOT NULL,          -- file trong kho máy admin
    collected_at    TEXT NOT NULL DEFAULT (datetime('now')),
    forwarded_at    TEXT,                   -- NULL = chưa đẩy lên server
    forward_error   TEXT,                   -- lỗi lần đẩy gần nhất (để thử lại)
    forward_tries   INTEGER NOT NULL DEFAULT 0
);
-- Khử trùng: cùng (dự án, hash) chỉ gom 1 lần dù kéo lại nhiều lần.
CREATE UNIQUE INDEX IF NOT EXISTS ux_collected_hash ON collected_docs (project_id, blake3);
"#;
pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let mut r_1cw = Connection::open(path)?;
    r_1cw.busy_timeout(Duration::from_secs(5))?;
    r_1cw.pragma_update(None, "journal_mode", "WAL")?;
    r_1cw.pragma_update(None, "synchronous", "NORMAL")?;
    r_1cw.pragma_update(None, "foreign_keys", "ON")?;
    migrate(&mut r_1cw)?;
    Ok(r_1cw)
}
fn f_10n(r_1cy: &Connection, table: &str, r_1cx: &str) -> rusqlite::Result<bool> {
    let mut r_1d0 = r_1cy.prepare(&format!("PRAGMA table_info(\"{table}\")"))?;
    let mut r_1cz = r_1d0.query([])?;
    while let Some(r) = r_1cz.next()? {
        if r.get::<_, String>(1)? == r_1cx {
            return Ok(true);
        }
    }
    Ok(false)
}
fn migrate(r_1d1: &mut Connection) -> rusqlite::Result<()> {
    let r_1d2: i64 = r_1d1.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if r_1d2 >= C_P1 {
        return Ok(());
    }
    let r_1d3 = r_1d1.transaction()?;
    if r_1d2 < 1 {
        r_1d3.execute_batch(C_JI)?;
    }
    if r_1d2 < 2 {
        r_1d3.execute_batch(C_JO)?;
    }
    if r_1d2 < 3 {
        r_1d3.execute_batch(crate::dossier::store::C_JP)?;
    }
    if r_1d2 < 4 {
        r_1d3.execute_batch(crate::catalog::store::C_JQ)?;
    }
    if r_1d2 < 5 && !f_10n(&r_1d3, "dossier", "thu_tu_sap_xep")? {
        r_1d3.execute_batch(crate::dossier::store::C_JR)?;
    }
    if r_1d2 < 6 {
        r_1d3.execute_batch(crate::dossier::history::C_JS)?;
    }
    if r_1d2 < 7 {
        r_1d3.execute_batch(crate::catalog::store::C_JT)?;
    }
    if r_1d2 < 8 {
        r_1d3.execute_batch(crate::auth::store::C_JU)?;
        for (table, col) in crate::auth::store::C_6 {
            if !f_10n(&r_1d3, table, col)? {
                r_1d3.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {col} TEXT;"))?;
            }
        }
    }
    if r_1d2 < 9 && !f_10n(&r_1d3, "ho_so", "uploaded_at")? {
        r_1d3.execute_batch("ALTER TABLE ho_so ADD COLUMN uploaded_at TEXT;")?;
    }
    if r_1d2 < 10 {
        if !f_10n(&r_1d3, "ho_so", "pdf_blake3")? {
            r_1d3.execute_batch("ALTER TABLE ho_so ADD COLUMN pdf_blake3 TEXT;")?;
        }
        r_1d3.execute_batch(C_JJ)?;
    }
    if r_1d2 < 11 {
        r_1d3.execute_batch(crate::khaithac::store::C_JK)?;
        if !f_10n(&r_1d3, "dossier", "ma_gia")? {
            r_1d3.execute_batch(crate::khaithac::store::C_JL)?;
        }
    }
    if r_1d2 < 12 {
        if !f_10n(&r_1d3, "ho_so", "ngay_bat_dau")? {
            r_1d3.execute_batch("ALTER TABLE ho_so ADD COLUMN ngay_bat_dau TEXT;")?;
        }
        if !f_10n(&r_1d3, "ho_so", "ngay_ket_thuc")? {
            r_1d3.execute_batch("ALTER TABLE ho_so ADD COLUMN ngay_ket_thuc TEXT;")?;
        }
    }
    if r_1d2 < 13 {
        r_1d3.execute_batch(
            "CREATE TABLE IF NOT EXISTS pdf_hash_cache (
                path       TEXT PRIMARY KEY,
                size       INTEGER NOT NULL,
                mtime      INTEGER NOT NULL,
                blake3     TEXT NOT NULL,
                page_count INTEGER
             );
             CREATE INDEX IF NOT EXISTS ix_pdf_hash_cache_blake3 ON pdf_hash_cache (blake3);",
        )?;
    }
    if r_1d2 < 14 {
        for (table, col, ty) in crate::khaithac::store::C_JM {
            if !f_10n(&r_1d3, table, col)? {
                r_1d3.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {col} {ty};"))?;
            }
        }
    }
    if r_1d2 < 15 {
        for (table, col, ty) in crate::khaithac::store::C_JN {
            if !f_10n(&r_1d3, table, col)? {
                r_1d3.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {col} {ty};"))?;
            }
        }
    }
    if r_1d2 < 16 && !f_10n(&r_1d3, "collected_docs", "forward_fatal")? {
        r_1d3.execute_batch(
            "ALTER TABLE collected_docs ADD COLUMN forward_fatal INTEGER NOT NULL DEFAULT 0;",
        )?;
    }
    if r_1d2 < 17 && !f_10n(&r_1d3, "trang", "pdf_path")? {
        r_1d3.execute_batch("ALTER TABLE trang ADD COLUMN pdf_path TEXT;")?;
        r_1d3
            .execute_batch(
                "UPDATE trang SET pdf_path = (SELECT h.pdf_path FROM ho_so h WHERE h.id = trang.ho_so_id) \
             WHERE pdf_path IS NULL;",
            )?;
        r_1d3.execute_batch(
            "CREATE INDEX IF NOT EXISTS ix_trang_hoso_pdf ON trang (ho_so_id, pdf_path);",
        )?;
    }
    if r_1d2 < 18 {
        if !f_10n(&r_1d3, "collected_docs", "phong_ma")? {
            r_1d3.execute_batch("ALTER TABLE collected_docs ADD COLUMN phong_ma TEXT;")?;
        }
        r_1d3.execute_batch(
            "CREATE INDEX IF NOT EXISTS ix_collected_hoso ON collected_docs (phong_ma, ho_so_ref);",
        )?;
    }
    if r_1d2 < 19 {
        r_1d3.execute_batch(
            "CREATE TABLE IF NOT EXISTS lan_peer_pubkey (
                 device_id  TEXT PRIMARY KEY,
                 pubkey     TEXT NOT NULL,
                 first_seen TEXT NOT NULL DEFAULT (datetime('now')),
                 last_addr  TEXT
             );",
        )?;
    }
    r_1d3.pragma_update(None, "user_version", C_P1)?;
    r_1d3.commit()?;
    Ok(())
}
pub fn f_1ew(r_1d5: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    r_1d5
        .query_row(
            "SELECT value FROM settings WHERE key=?1",
            params![key],
            |r| r.get(0),
        )
        .optional()
}
pub fn f_2ei(r_1d6: &Connection, key: &str, value: Option<&str>) -> rusqlite::Result<()> {
    match value {
        Some(r_1d7) => r_1d6.execute(
            "INSERT INTO settings(key, value) VALUES(?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, r_1d7],
        )?,
        None => r_1d6.execute("DELETE FROM settings WHERE key=?1", params![key])?,
    };
    Ok(())
}
pub fn f_1et(r_1d8: &Connection, device_id: &str) -> rusqlite::Result<Option<String>> {
    r_1d8
        .query_row(
            "SELECT pubkey FROM lan_peer_pubkey WHERE device_id=?1",
            params![device_id],
            |r| r.get(0),
        )
        .optional()
}
pub fn f_22x(
    r_1da: &Connection,
    device_id: &str,
    pubkey: &str,
    r_1d9: Option<&str>,
) -> rusqlite::Result<String> {
    r_1da.execute(
        "INSERT INTO lan_peer_pubkey (device_id, pubkey, last_addr) VALUES (?1, ?2, ?3)
         ON CONFLICT(device_id) DO UPDATE SET last_addr = excluded.last_addr",
        params![device_id, pubkey, r_1d9],
    )?;
    r_1da.query_row(
        "SELECT pubkey FROM lan_peer_pubkey WHERE device_id=?1",
        params![device_id],
        |r| r.get(0),
    )
}
#[derive(Debug, Clone)]
pub struct T96 {
    pub phong_id: Option<String>,
    pub hop_so: String,
    pub ho_so_so: String,
    pub tieu_de: String,
    pub thoi_gian: Option<String>,
    pub so_to: Option<String>,
    pub thbq: Option<String>,
    pub ghi_chu: Option<String>,
    pub nhom: Option<String>,
    pub nam: Option<String>,
    pub ngay_bat_dau: Option<String>,
    pub ngay_ket_thuc: Option<String>,
}
#[derive(Debug, Clone)]
pub struct Tod {
    pub page_no: u32,
    pub image_path: String,
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: u32,
    pub paper: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T98 {
    pub id: i64,
    pub phong_id: Option<String>,
    pub hop_so: String,
    pub ho_so_so: String,
    pub tieu_de: String,
    pub thoi_gian: Option<String>,
    pub so_to: Option<String>,
    pub thbq: Option<String>,
    pub ghi_chu: Option<String>,
    pub nhom: Option<String>,
    pub nam: Option<String>,
    pub folder_path: Option<String>,
    pub pdf_path: Option<String>,
    pub pdf_size: Option<i64>,
    pub page_count: Option<i64>,
    pub status: String,
    pub ngay_bat_dau: Option<String>,
    pub ngay_ket_thuc: Option<String>,
}
pub fn f_2o6(
    r_1db: &Connection,
    r_1dd: &str,
    r_1df: &str,
    r_1dc: Option<&str>,
    phong_so: Option<&str>,
    r_1de: Option<&str>,
) -> rusqlite::Result<()> {
    r_1db.execute(
        "INSERT INTO phong (ma_du_an, ten_du_an, ma_khoi, phong_so, muc_luc_so)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(ma_du_an) DO UPDATE SET
            ten_du_an  = excluded.ten_du_an,
            ma_khoi    = COALESCE(excluded.ma_khoi, ma_khoi),
            phong_so   = COALESCE(excluded.phong_so, phong_so),
            muc_luc_so = COALESCE(excluded.muc_luc_so, muc_luc_so)",
        params![r_1dd, r_1df, r_1dc, phong_so, r_1de],
    )?;
    Ok(())
}
pub fn f_145(r_1dg: &Connection, r_1dh: &str) -> rusqlite::Result<bool> {
    let r_1di = r_1dg.execute("DELETE FROM phong WHERE ma_du_an = ?1", params![r_1dh])?;
    Ok(r_1di > 0)
}
pub fn f_2nz(r_1dj: &Connection, h: &T96) -> rusqlite::Result<i64> {
    let r_1dk: Option<i64> = r_1dj
        .query_row(
            "SELECT id FROM ho_so WHERE phong_id IS ?1 AND hop_so = ?2 AND ho_so_so = ?3 LIMIT 1",
            params![h.phong_id, h.hop_so, h.ho_so_so],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(id) = r_1dk {
        r_1dj.execute(
            "UPDATE ho_so SET tieu_de=?2, thoi_gian=?3, so_to=?4, thbq=?5, ghi_chu=?6, nhom=?7,
                nam=?8, ngay_bat_dau=?9, ngay_ket_thuc=?10, updated_at=datetime('now') WHERE id=?1",
            params![
                id,
                h.tieu_de,
                h.thoi_gian,
                h.so_to,
                h.thbq,
                h.ghi_chu,
                h.nhom,
                h.nam,
                h.ngay_bat_dau,
                h.ngay_ket_thuc,
            ],
        )?;
        return Ok(id);
    }
    r_1dj.execute(
        "INSERT INTO ho_so
            (phong_id, hop_so, ho_so_so, tieu_de, thoi_gian, so_to, thbq, ghi_chu, nhom, nam,
             ngay_bat_dau, ngay_ket_thuc, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,datetime('now'))",
        params![
            h.phong_id,
            h.hop_so,
            h.ho_so_so,
            h.tieu_de,
            h.thoi_gian,
            h.so_to,
            h.thbq,
            h.ghi_chu,
            h.nhom,
            h.nam,
            h.ngay_bat_dau,
            h.ngay_ket_thuc,
        ],
    )?;
    Ok(r_1dj.last_insert_rowid())
}
pub fn f_1hb(
    r_1dl: &Connection,
    phong_id: Option<&str>,
    hop_so: &str,
    ho_so_so: &str,
) -> rusqlite::Result<bool> {
    let r_1dm: i64 = r_1dl.query_row(
        "SELECT count(*) FROM ho_so \
         WHERE phong_id IS ?1 AND hop_so = ?2 AND ho_so_so = ?3 AND finalized_by IS NOT NULL",
        params![phong_id, hop_so, ho_so_so],
        |r| r.get(0),
    )?;
    Ok(r_1dm > 0)
}
pub fn f_2nf(r_1dn: &Connection, id: i64, h: &T96) -> rusqlite::Result<bool> {
    let r_1do = r_1dn.execute(
        "UPDATE ho_so SET hop_so=?2, ho_so_so=?3, tieu_de=?4, thoi_gian=?5, so_to=?6, thbq=?7,
            ghi_chu=?8, nhom=?9, nam=?10, ngay_bat_dau=?11, ngay_ket_thuc=?12,
            updated_at=datetime('now')
         WHERE id=?1",
        params![
            id,
            h.hop_so,
            h.ho_so_so,
            h.tieu_de,
            h.thoi_gian,
            h.so_to,
            h.thbq,
            h.ghi_chu,
            h.nhom,
            h.nam,
            h.ngay_bat_dau,
            h.ngay_ket_thuc,
        ],
    )?;
    Ok(r_1do > 0)
}
pub fn f_27w(
    r_1dp: &Connection,
    ho_so_id: i64,
    pdf_path: &str,
    r_1dq: &[Tod],
) -> rusqlite::Result<()> {
    r_1dp.execute(
        "DELETE FROM trang WHERE ho_so_id = ?1 AND pdf_path IS ?2",
        params![ho_so_id, pdf_path],
    )?;
    let mut r_1dr = r_1dp
        .prepare(
            "INSERT INTO trang (ho_so_id, page_no, image_path, width_px, height_px, dpi, paper, pdf_path)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        )?;
    for p in r_1dq {
        r_1dr.execute(params![
            ho_so_id,
            p.page_no,
            p.image_path,
            p.width_px,
            p.height_px,
            p.dpi,
            p.paper,
            pdf_path
        ])?;
    }
    Ok(())
}
pub fn f_2e5(
    r_1ds: &Connection,
    id: i64,
    folder_path: &str,
    pdf_path: &str,
    pdf_size: i64,
    page_count: i64,
) -> rusqlite::Result<()> {
    r_1ds.execute(
        "UPDATE ho_so SET folder_path=?2, pdf_path=?3, pdf_size=?4, page_count=?5,
            status='done', updated_at=datetime('now') WHERE id=?1",
        params![id, folder_path, pdf_path, pdf_size, page_count],
    )?;
    Ok(())
}
pub fn f_2e4(r_1dt: &Connection, id: i64, folder_path: &str) -> rusqlite::Result<()> {
    r_1dt.execute(
        "UPDATE ho_so SET folder_path=?2, updated_at=datetime('now') WHERE id=?1",
        params![id, folder_path],
    )?;
    Ok(())
}
fn f_1n2(r_1du: &str) -> String {
    let mut out = String::with_capacity(r_1du.len() + 8);
    for c in r_1du.chars() {
        if matches!(c, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
fn f_yp(r_c: &str) -> (String, String) {
    let esc = f_1n2(r_c);
    (format!("{esc}/%"), format!("{esc}\\\\%"))
}
pub fn f_148(r_1dv: &Connection, r_1dw: &str) -> rusqlite::Result<()> {
    let (r_1dy, r_1dx) = f_yp(r_1dw);
    r_1dv.execute(
        "DELETE FROM ho_so WHERE folder_path = ?1 \
            OR folder_path LIKE ?2 ESCAPE '\\' OR folder_path LIKE ?3 ESCAPE '\\'",
        params![r_1dw, r_1dy, r_1dx],
    )?;
    Ok(())
}
pub fn f_102(r_1dz: &Connection, pdf_path: &str) -> rusqlite::Result<()> {
    r_1dz
        .execute(
            "UPDATE ho_so SET pdf_path=NULL, pdf_size=NULL, page_count=NULL, status='pending', updated_at=datetime('now')
         WHERE pdf_path = ?1",
            params![pdf_path],
        )?;
    Ok(())
}
pub fn f_27b(r_1e0: &Connection, r_1e2: &str, r_1e1: &str) -> rusqlite::Result<()> {
    let (r_1e4, r_1e3) = f_yp(r_1e2);
    let r_1e5 = r_1e0.unchecked_transaction()?;
    r_1e5
        .execute(
            "UPDATE ho_so SET folder_path = ?2 || substr(folder_path, length(?1)+1) \
         WHERE folder_path = ?1 OR folder_path LIKE ?3 ESCAPE '\\' OR folder_path LIKE ?4 ESCAPE '\\'",
            params![r_1e2, r_1e1, r_1e4, r_1e3],
        )?;
    r_1e5.execute(
        "UPDATE ho_so SET pdf_path = ?2 || substr(pdf_path, length(?1)+1) \
         WHERE pdf_path = ?1 OR pdf_path LIKE ?3 ESCAPE '\\' OR pdf_path LIKE ?4 ESCAPE '\\'",
        params![r_1e2, r_1e1, r_1e4, r_1e3],
    )?;
    r_1e5.execute(
        "UPDATE trang SET image_path = ?2 || substr(image_path, length(?1)+1) \
         WHERE image_path = ?1 OR image_path LIKE ?3 ESCAPE '\\' OR image_path LIKE ?4 ESCAPE '\\'",
        params![r_1e2, r_1e1, r_1e4, r_1e3],
    )?;
    r_1e5.execute(
        "UPDATE trang SET pdf_path = ?2 || substr(pdf_path, length(?1)+1) \
         WHERE pdf_path = ?1 OR pdf_path LIKE ?3 ESCAPE '\\' OR pdf_path LIKE ?4 ESCAPE '\\'",
        params![r_1e2, r_1e1, r_1e4, r_1e3],
    )?;
    r_1e5.execute(
        "UPDATE pdf_hash_cache SET path = ?2 || substr(path, length(?1)+1) \
         WHERE path = ?1 OR path LIKE ?3 ESCAPE '\\' OR path LIKE ?4 ESCAPE '\\'",
        params![r_1e2, r_1e1, r_1e4, r_1e3],
    )?;
    r_1e5.commit()
}
pub fn list_ho_so(r_1e6: &Connection, phong_id: Option<&str>) -> rusqlite::Result<Vec<T98>> {
    let mut r_1e8 = r_1e6.prepare(
        "SELECT id, phong_id, hop_so, ho_so_so, tieu_de, thoi_gian, so_to, thbq, ghi_chu,
                nhom, nam, folder_path, pdf_path, pdf_size, page_count, status,
                ngay_bat_dau, ngay_ket_thuc
         FROM ho_so
         WHERE (?1 IS NULL OR phong_id = ?1)
         ORDER BY created_at DESC, id DESC",
    )?;
    let r_1e7 = r_1e8
        .query_map(params![phong_id], |r| {
            Ok(T98 {
                id: r.get(0)?,
                phong_id: r.get(1)?,
                hop_so: r.get(2)?,
                ho_so_so: r.get(3)?,
                tieu_de: r.get(4)?,
                thoi_gian: r.get(5)?,
                so_to: r.get(6)?,
                thbq: r.get(7)?,
                ghi_chu: r.get(8)?,
                nhom: r.get(9)?,
                nam: r.get(10)?,
                folder_path: r.get(11)?,
                pdf_path: r.get(12)?,
                pdf_size: r.get(13)?,
                page_count: r.get(14)?,
                status: r.get(15)?,
                ngay_bat_dau: r.get(16)?,
                ngay_ket_thuc: r.get(17)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1e7)
}
pub fn f_13x(r_1e9: &Connection, id: i64) -> rusqlite::Result<bool> {
    let r_1ea = r_1e9.execute("DELETE FROM ho_so WHERE id = ?1", params![id])?;
    Ok(r_1ea > 0)
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Th9 {
    pub ma_du_an: String,
    pub ten_du_an: String,
    pub ma_khoi: Option<String>,
    pub phong_so: Option<String>,
    pub muc_luc_so: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T93 {
    pub id: i64,
    pub ma_du_an: String,
    pub ten_du_an: String,
    pub hop_so: String,
    pub ho_so_so: String,
    pub tieu_de: String,
    pub phong_so: Option<String>,
    pub muc_luc_so: Option<String>,
    pub ngay_bat_dau: Option<String>,
    pub ngay_ket_thuc: Option<String>,
    pub nam: Option<String>,
    pub thoi_gian: Option<String>,
}
pub fn f_1cx(r_1eb: &Connection, pdf_path: &str) -> rusqlite::Result<Option<i64>> {
    r_1eb
        .query_row(
            "SELECT id FROM ho_so WHERE pdf_path = ?1 LIMIT 1",
            params![pdf_path],
            |r| r.get(0),
        )
        .optional()
}
pub fn f_1cw(r_1ec: &Connection, r_1ed: &str) -> rusqlite::Result<Option<T93>> {
    r_1ec
        .query_row(
            "SELECT h.id, p.ma_du_an, p.ten_du_an, h.hop_so, h.ho_so_so, h.tieu_de,
                p.phong_so, p.muc_luc_so, h.ngay_bat_dau, h.ngay_ket_thuc,
                h.nam, h.thoi_gian
         FROM ho_so h JOIN phong p ON p.ma_du_an = h.phong_id
         WHERE h.folder_path = ?1 LIMIT 1",
            params![r_1ed],
            |r| {
                Ok(T93 {
                    id: r.get(0)?,
                    ma_du_an: r.get(1)?,
                    ten_du_an: r.get(2)?,
                    hop_so: r.get(3)?,
                    ho_so_so: r.get(4)?,
                    tieu_de: r.get(5)?,
                    phong_so: r.get(6)?,
                    muc_luc_so: r.get(7)?,
                    ngay_bat_dau: r.get(8)?,
                    ngay_ket_thuc: r.get(9)?,
                    nam: r.get(10)?,
                    thoi_gian: r.get(11)?,
                })
            },
        )
        .optional()
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T94 {
    pub id: i64,
    pub ma_du_an: String,
    pub ten_du_an: String,
    pub ma_khoi: Option<String>,
    pub phong_so: Option<String>,
    pub muc_luc_so: Option<String>,
    pub hop_so: String,
    pub ho_so_so: String,
    pub tieu_de: String,
    pub thoi_gian: Option<String>,
    pub thbq: Option<String>,
    pub nhom: Option<String>,
    pub nam: Option<String>,
    pub folder_path: Option<String>,
    pub pdf_path: Option<String>,
    pub status: String,
    pub ngay_bat_dau: Option<String>,
    pub ngay_ket_thuc: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Toe {
    pub page_no: u32,
    pub image_path: String,
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
    pub dpi: Option<u32>,
    pub paper: Option<String>,
}
pub fn f_1em(r_1ee: &Connection, id: i64) -> rusqlite::Result<Option<T94>> {
    r_1ee
        .query_row(
            "SELECT h.id, h.phong_id, p.ten_du_an, p.ma_khoi, p.phong_so, p.muc_luc_so,
                h.hop_so, h.ho_so_so, h.tieu_de, h.thbq, h.nhom, h.nam,
                h.folder_path, h.pdf_path, h.status,
                h.ngay_bat_dau, h.ngay_ket_thuc, h.thoi_gian
         FROM ho_so h LEFT JOIN phong p ON p.ma_du_an = h.phong_id
         WHERE h.id = ?1",
            params![id],
            |r| {
                Ok(T94 {
                    id: r.get(0)?,
                    ma_du_an: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    ten_du_an: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    ma_khoi: r.get(3)?,
                    phong_so: r.get(4)?,
                    muc_luc_so: r.get(5)?,
                    hop_so: r.get(6)?,
                    ho_so_so: r.get(7)?,
                    tieu_de: r.get(8)?,
                    thbq: r.get(9)?,
                    nhom: r.get(10)?,
                    nam: r.get(11)?,
                    folder_path: r.get(12)?,
                    pdf_path: r.get(13)?,
                    status: r.get(14)?,
                    ngay_bat_dau: r.get(15)?,
                    ngay_ket_thuc: r.get(16)?,
                    thoi_gian: r.get(17)?,
                })
            },
        )
        .optional()
}
pub fn f_1oi(r_1ef: &Connection, ho_so_id: i64) -> rusqlite::Result<Vec<Toe>> {
    f_1ok(r_1ef, ho_so_id, None)
}
pub fn f_1oj(r_1eg: &Connection, ho_so_id: i64, pdf_path: &str) -> rusqlite::Result<Vec<Toe>> {
    f_1ok(r_1eg, ho_so_id, Some(pdf_path))
}
fn f_1ok(r_1eh: &Connection, ho_so_id: i64, pdf_path: Option<&str>) -> rusqlite::Result<Vec<Toe>> {
    let r_1ej = match pdf_path {
        Some(_) => {
            "SELECT page_no, image_path, width_px, height_px, dpi, paper
             FROM trang WHERE ho_so_id = ?1 AND pdf_path IS ?2 ORDER BY page_no"
        }
        None => {
            "SELECT page_no, image_path, width_px, height_px, dpi, paper
             FROM trang WHERE ho_so_id = ?1 AND (?2 IS NULL) ORDER BY page_no"
        }
    };
    let mut r_1ek = r_1eh.prepare(r_1ej)?;
    let r_1ei = r_1ek
        .query_map(params![ho_so_id, pdf_path], |r| {
            Ok(Toe {
                page_no: r.get(0)?,
                image_path: r.get(1)?,
                width_px: r.get(2)?,
                height_px: r.get(3)?,
                dpi: r.get(4)?,
                paper: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1ei)
}
pub fn f_2h4(r_1el: &Connection, id: i64, user_id: &str) -> rusqlite::Result<()> {
    r_1el.execute(
        "UPDATE ho_so SET created_by = COALESCE(created_by, ?1) WHERE id = ?2",
        params![user_id, id],
    )?;
    Ok(())
}
pub fn f_2h5(r_1em: &Connection, id: i64, user_id: &str) -> rusqlite::Result<()> {
    r_1em.execute(
        "UPDATE ho_so SET finalized_by = ?1 WHERE id = ?2",
        params![user_id, id],
    )?;
    Ok(())
}
pub fn f_2h6(r_1en: &Connection, ho_so_id: i64, user_id: &str) -> rusqlite::Result<()> {
    r_1en.execute(
        "UPDATE trang SET scanned_by = ?1 WHERE ho_so_id = ?2",
        params![user_id, ho_so_id],
    )?;
    Ok(())
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tks {
    pub pages_scanned: i64,
    pub ho_so_count: i64,
    pub pdf_count: i64,
    pub pdf_bytes: i64,
}
pub fn f_2oe(r_1eo: &Connection, user_id: &str) -> rusqlite::Result<Tks> {
    let pages_scanned: i64 = r_1eo.query_row(
        "SELECT COUNT(*) FROM trang WHERE scanned_by = ?1",
        params![user_id],
        |r| r.get(0),
    )?;
    let (ho_so_count, pdf_count, pdf_bytes) = r_1eo.query_row(
        "SELECT COUNT(*), COUNT(pdf_path), COALESCE(SUM(pdf_size), 0) \
         FROM ho_so WHERE finalized_by = ?1",
        params![user_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    Ok(Tks {
        pages_scanned,
        ho_so_count,
        pdf_count,
        pdf_bytes,
    })
}
#[derive(Debug, Clone)]
pub struct Tln {
    pub folder_path: Option<String>,
    pub pdf_path: String,
    pub hop_so: String,
    pub ho_so_so: String,
    pub source_user_id: String,
    pub source_username: Option<String>,
    pub source_project_id: Option<String>,
    pub phong_ma: Option<String>,
    pub phong_ten: Option<String>,
    pub phong_so: Option<String>,
    pub muc_luc_so: Option<String>,
    pub tieu_de: String,
    pub thoi_gian: Option<String>,
    pub nam: Option<String>,
    pub so_to: Option<String>,
}
pub fn f_2er(r_1ep: &Connection) -> rusqlite::Result<Vec<Tln>> {
    let mut r_1er = r_1ep.prepare(
        "SELECT h.folder_path, h.pdf_path, h.hop_so, h.ho_so_so, \
                h.finalized_by, a.username, a.project_id, \
                h.phong_id, p.ten_du_an, p.phong_so, p.muc_luc_so, \
                h.tieu_de, h.thoi_gian, h.nam, h.so_to \
         FROM ho_so h \
         LEFT JOIN auth_account a ON a.user_id = h.finalized_by \
         LEFT JOIN phong p ON p.ma_du_an = h.phong_id \
         WHERE h.pdf_path IS NOT NULL AND h.finalized_by IS NOT NULL \
         ORDER BY h.id",
    )?;
    let r_1eq = r_1er
        .query_map([], |r| {
            Ok(Tln {
                folder_path: r.get(0)?,
                pdf_path: r.get(1)?,
                hop_so: r.get(2)?,
                ho_so_so: r.get(3)?,
                source_user_id: r.get(4)?,
                source_username: r.get(5)?,
                source_project_id: r.get(6)?,
                phong_ma: r.get(7)?,
                phong_ten: r.get(8)?,
                phong_so: r.get(9)?,
                muc_luc_so: r.get(10)?,
                tieu_de: r.get(11)?,
                thoi_gian: r.get(12)?,
                nam: r.get(13)?,
                so_to: r.get(14)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1eq)
}
pub fn f_20u(
    r_1es: &Connection,
    path: &str,
    size: i64,
    mtime: i64,
) -> rusqlite::Result<Option<(String, Option<i64>)>> {
    r_1es
        .query_row(
            "SELECT blake3, page_count FROM pdf_hash_cache WHERE path = ?1 AND size = ?2 AND mtime = ?3",
            params![path, size, mtime],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
}
pub fn f_20v(
    r_1et: &Connection,
    path: &str,
    size: i64,
    mtime: i64,
    blake3: &str,
    page_count: Option<i64>,
) -> rusqlite::Result<()> {
    r_1et.execute(
        "INSERT INTO pdf_hash_cache (path, size, mtime, blake3, page_count) \
         VALUES (?1, ?2, ?3, ?4, ?5) \
         ON CONFLICT(path) DO UPDATE SET size = ?2, mtime = ?3, blake3 = ?4, page_count = ?5",
        params![path, size, mtime, blake3, page_count],
    )?;
    Ok(())
}
pub fn f_218(r_1eu: &Connection, blake3: &str) -> rusqlite::Result<Option<String>> {
    r_1eu
        .query_row(
            "SELECT path FROM pdf_hash_cache WHERE blake3 = ?1 ORDER BY rowid DESC LIMIT 1",
            params![blake3],
            |r| r.get(0),
        )
        .optional()
}
pub fn f_24i(r_1ev: &Connection) -> rusqlite::Result<usize> {
    let r_1ew: Vec<String> = {
        let mut r_1ez = r_1ev.prepare("SELECT path FROM pdf_hash_cache")?;
        let r_1ey = r_1ez.query_map([], |r| r.get::<_, String>(0))?;
        r_1ey.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let mut r_1ex = 0usize;
    for p in r_1ew {
        if !std::path::Path::new(&p).is_file() {
            r_1ex += r_1ev.execute("DELETE FROM pdf_hash_cache WHERE path = ?1", params![p])?;
        }
    }
    Ok(r_1ex)
}
#[derive(Debug, Clone)]
pub struct Tds<'a> {
    pub source_user_id: &'a str,
    pub source_username: Option<&'a str>,
    pub project_id: &'a str,
    pub phong_ma: Option<&'a str>,
    pub ho_so_ref: Option<&'a str>,
    pub filename: &'a str,
    pub page_count: Option<i64>,
    pub size_bytes: i64,
    pub blake3: &'a str,
    pub storage_path: &'a str,
}
pub fn f_10g(r_1f0: &Connection, project_id: &str, blake3: &str) -> rusqlite::Result<bool> {
    let r_1f1: i64 = r_1f0.query_row(
        "SELECT COUNT(*) FROM collected_docs WHERE project_id = ?1 AND blake3 = ?2",
        params![project_id, blake3],
        |r| r.get(0),
    )?;
    Ok(r_1f1 > 0)
}
pub fn f_10i(r_1f2: &Connection) -> rusqlite::Result<HashSet<String>> {
    let mut r_1f4 = r_1f2.prepare("SELECT storage_path FROM collected_docs")?;
    let r_1f3 = r_1f4.query_map([], |r| r.get::<_, String>(0))?;
    r_1f3.collect()
}
pub fn f_1id(r_1f5: &Connection, c: Tds) -> rusqlite::Result<bool> {
    let r_1f6 = r_1f5.execute(
        "INSERT OR IGNORE INTO collected_docs \
            (source_user_id, source_username, project_id, phong_ma, ho_so_ref, filename, \
             page_count, size_bytes, blake3, storage_path) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            c.source_user_id,
            c.source_username,
            c.project_id,
            c.phong_ma,
            c.ho_so_ref,
            c.filename,
            c.page_count,
            c.size_bytes,
            c.blake3,
            c.storage_path,
        ],
    )?;
    Ok(r_1f6 > 0)
}
#[derive(Debug, Clone)]
pub struct T2w {
    pub id: i64,
    pub source_user_id: String,
    pub source_username: Option<String>,
    pub project_id: String,
    pub phong_ma: Option<String>,
    pub ho_so_ref: Option<String>,
    pub filename: String,
    pub page_count: Option<i64>,
    pub size_bytes: i64,
    pub blake3: String,
    pub storage_path: String,
    pub forward_tries: i64,
    pub forwarded_at: Option<String>,
    pub forward_fatal: bool,
    pub collected_at: String,
    pub forward_error: Option<String>,
}
fn f_1qr(r: &rusqlite::Row) -> rusqlite::Result<T2w> {
    Ok(T2w {
        id: r.get(0)?,
        source_user_id: r.get(1)?,
        source_username: r.get(2)?,
        project_id: r.get(3)?,
        phong_ma: r.get(4)?,
        ho_so_ref: r.get(5)?,
        filename: r.get(6)?,
        page_count: r.get(7)?,
        size_bytes: r.get(8)?,
        blake3: r.get(9)?,
        storage_path: r.get(10)?,
        forward_tries: r.get(11)?,
        forwarded_at: r.get(12)?,
        forward_fatal: r.get::<_, i64>(13)? != 0,
        collected_at: r.get(14)?,
        forward_error: r.get(15)?,
    })
}
const COLLECTED_COLS: &str = "id, source_user_id, source_username, project_id, phong_ma, \
     ho_so_ref, filename, page_count, size_bytes, blake3, storage_path, forward_tries, \
     forwarded_at, forward_fatal, collected_at, forward_error";
pub fn f_10h(r_1f7: &Connection, storage_path: &str) -> rusqlite::Result<bool> {
    let r_1f8: i64 = r_1f7.query_row(
        "SELECT COUNT(*) FROM collected_docs \
         WHERE storage_path = ?1 AND forwarded_at IS NULL AND forward_fatal = 0",
        params![storage_path],
        |r| r.get(0),
    )?;
    Ok(r_1f8 > 0)
}
pub fn f_29p(r_1f9: &Connection, id: i64, blake3: &str, size_bytes: i64) -> rusqlite::Result<bool> {
    match r_1f9.execute(
        "UPDATE collected_docs SET blake3 = ?2, size_bytes = ?3 WHERE id = ?1",
        params![id, blake3, size_bytes],
    ) {
        Ok(_) => Ok(true),
        Err(rusqlite::Error::SqliteFailure(e, _))
            if e.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            Ok(false)
        }
        Err(e) => Err(e),
    }
}
pub fn f_13u(r_1fa: &Connection, id: i64) -> rusqlite::Result<()> {
    r_1fa.execute("DELETE FROM collected_docs WHERE id = ?1", params![id])?;
    Ok(())
}
pub fn f_10k(r_1fb: &Connection) -> rusqlite::Result<Vec<T2w>> {
    let mut r_1fd = r_1fb.prepare(&format!(
        "SELECT {COLLECTED_COLS} FROM collected_docs \
         WHERE forwarded_at IS NULL AND forward_fatal = 0 ORDER BY id"
    ))?;
    let r_1fc = r_1fd
        .query_map([], f_1qr)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1fc)
}
pub fn f_1r8(r_1fe: &Connection, id: i64) -> rusqlite::Result<()> {
    r_1fe
        .execute(
            "UPDATE collected_docs SET forwarded_at = datetime('now'), forward_error = NULL WHERE id = ?1",
            params![id],
        )?;
    Ok(())
}
pub fn f_10m(r_1ff: &Connection, blake3: &str) -> rusqlite::Result<Option<String>> {
    r_1ff
        .query_row(
            "SELECT source_user_id FROM collected_docs WHERE blake3 = ?1 LIMIT 1",
            params![blake3],
            |r| r.get(0),
        )
        .optional()
}
pub fn f_1r6(r_1fg: &Connection, id: i64, r_1fh: &str) -> rusqlite::Result<()> {
    r_1fg
        .execute(
            "UPDATE collected_docs SET forward_error = ?2, forward_tries = forward_tries + 1 WHERE id = ?1",
            params![id, r_1fh],
        )?;
    Ok(())
}
pub fn f_1r7(r_1fi: &Connection, id: i64, r_1fj: &str) -> rusqlite::Result<()> {
    r_1fi.execute(
        "UPDATE collected_docs \
         SET forward_error = ?2, forward_tries = forward_tries + 1, forward_fatal = 1 \
         WHERE id = ?1",
        params![id, r_1fj],
    )?;
    Ok(())
}
pub fn f_10l(r_1fk: &Connection, r_1fl: &[i64]) -> rusqlite::Result<Vec<T2w>> {
    if r_1fl.is_empty() {
        return Ok(Vec::new());
    }
    let holes = vec!["?"; r_1fl.len()].join(",");
    let mut r_1fn = r_1fk.prepare(&format!(
        "SELECT {COLLECTED_COLS} FROM collected_docs \
         WHERE forwarded_at IS NULL AND id IN ({holes}) ORDER BY id"
    ))?;
    let r_1fm = r_1fn
        .query_map(rusqlite::params_from_iter(r_1fl), f_1qr)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1fm)
}
pub fn f_101(r_1fo: &Connection, id: i64) -> rusqlite::Result<usize> {
    r_1fo.execute(
        "UPDATE collected_docs SET forward_fatal = 0 WHERE id = ?1 AND forward_fatal = 1",
        params![id],
    )
}
pub fn f_100(r_1fp: &Connection) -> rusqlite::Result<usize> {
    r_1fp.execute("DELETE FROM collected_docs WHERE forward_fatal = 1", [])
}
/// Xoá DẤU LỖI (không xoá bản): mọi bản chưa lên được máy chủ về trạng thái chưa thử —
/// lần "② Đẩy lên" tiếp theo gửi lại từ đầu, kể cả bản đã bị từ chối vĩnh viễn.
pub fn clear_forward_errors(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE collected_docs SET forward_error = NULL, forward_tries = 0, forward_fatal = 0 \
         WHERE forwarded_at IS NULL AND (forward_tries > 0 OR forward_fatal = 1 OR forward_error IS NOT NULL)",
        [],
    )
}
pub fn f_10e(r_1fq: &Connection) -> rusqlite::Result<(i64, i64, i64, i64)> {
    r_1fq.query_row(
        "SELECT COUNT(*), \
                COUNT(*) FILTER (WHERE forwarded_at IS NULL AND forward_fatal = 0), \
                COUNT(*) FILTER (WHERE forwarded_at IS NOT NULL), \
                COUNT(*) FILTER (WHERE forward_fatal = 1) \
         FROM collected_docs",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )
}
pub fn f_1na(r_1fr: &Connection, r_1fs: i64) -> rusqlite::Result<Vec<T2w>> {
    let mut r_1fu = r_1fr.prepare(&format!(
        "SELECT {COLLECTED_COLS} FROM collected_docs ORDER BY id DESC LIMIT ?1"
    ))?;
    let r_1ft = r_1fu
        .query_map(params![r_1fs], f_1qr)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1ft)
}
pub fn list_phong(r_1fv: &Connection) -> rusqlite::Result<Vec<Th9>> {
    let mut r_1fx = r_1fv.prepare(
        "SELECT ma_du_an, ten_du_an, ma_khoi, phong_so, muc_luc_so FROM phong ORDER BY ten_du_an",
    )?;
    let r_1fw = r_1fx
        .query_map([], |r| {
            Ok(Th9 {
                ma_du_an: r.get(0)?,
                ten_du_an: r.get(1)?,
                ma_khoi: r.get(2)?,
                phong_so: r.get(3)?,
                muc_luc_so: r.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1fw)
}
