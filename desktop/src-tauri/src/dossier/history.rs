// Copyright (c) 2026 DigiText. All rights reserved.
use rusqlite::{params, Connection};
use serde::Serialize;
pub const C_JS: &str = r#"
CREATE TABLE IF NOT EXISTS ho_so_history (
    id          INTEGER PRIMARY KEY,
    entity      TEXT NOT NULL,
    entity_id   INTEGER NOT NULL,
    dossier_id  INTEGER,
    ma_da       TEXT,
    action      TEXT NOT NULL,
    summary     TEXT,
    content     TEXT,
    nguoi       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS ix_history_entity ON ho_so_history (entity, created_at);
CREATE INDEX IF NOT EXISTS ix_history_phong ON ho_so_history (ma_da);
"#;
#[derive(Debug, Clone)]
pub struct Who {
    pub user_id: String,
    pub username: String,
}
impl From<&crate::auth::session::Tr> for Who {
    fn from(r_1oc: &crate::auth::session::Tr) -> Self {
        let (user_id, username) = r_1oc.who();
        Who { user_id, username }
    }
}
#[allow(clippy::too_many_arguments)]
pub fn record(
    r_1od: &Connection,
    entity: &str,
    entity_id: i64,
    dossier_id: Option<i64>,
    ma_da: Option<&str>,
    action: &str,
    summary: &str,
    content: &str,
    who: &Who,
) -> rusqlite::Result<()> {
    r_1od.execute(
        "INSERT INTO ho_so_history
            (entity, entity_id, dossier_id, ma_da, action, summary, content, nguoi, nguoi_id)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            entity,
            entity_id,
            dossier_id,
            ma_da,
            action,
            summary,
            content,
            who.username,
            who.user_id
        ],
    )?;
    Ok(())
}
#[derive(Debug, Default)]
pub struct HistoryFilter<'a> {
    pub entity: Option<&'a str>,
    pub ma_da: Option<&'a str>,
    pub tu_ngay: Option<&'a str>,
    pub den_ngay: Option<&'a str>,
    pub search: Option<&'a str>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T92 {
    pub id: i64,
    pub entity: String,
    pub entity_id: i64,
    pub dossier_id: Option<i64>,
    pub ma_da: Option<String>,
    pub action: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub nguoi: Option<String>,
    pub created_at: String,
}
pub fn list(r_1oe: &Connection, r_1of: &HistoryFilter) -> rusqlite::Result<Vec<T92>> {
    let mut r_1oi = r_1oe
        .prepare(
            "SELECT id, entity, entity_id, dossier_id, ma_da, action, summary, content, nguoi, created_at
         FROM ho_so_history
         WHERE (?1 IS NULL OR entity = ?1)
           AND (?2 IS NULL OR ma_da = ?2)
           AND (?3 IS NULL OR date(created_at) >= date(?3))
           AND (?4 IS NULL OR date(created_at) <= date(?4))
           AND (?5 IS NULL OR summary LIKE '%'||?5||'%' OR content LIKE '%'||?5||'%')
         ORDER BY created_at DESC, id DESC
         LIMIT 2000",
        )?;
    let r_1oh = r_1oi
        .query_map(
            params![
                r_1of.entity,
                r_1of.ma_da,
                r_1of.tu_ngay,
                r_1of.den_ngay,
                r_1of.search
            ],
            |r_1og| {
                Ok(T92 {
                    id: r_1og.get(0)?,
                    entity: r_1og.get(1)?,
                    entity_id: r_1og.get(2)?,
                    dossier_id: r_1og.get(3)?,
                    ma_da: r_1og.get(4)?,
                    action: r_1og.get(5)?,
                    summary: r_1og.get(6)?,
                    content: r_1og.get(7)?,
                    nguoi: r_1og.get(8)?,
                    created_at: r_1og.get(9)?,
                })
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(r_1oh)
}
