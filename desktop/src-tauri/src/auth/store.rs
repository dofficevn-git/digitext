// Copyright (c) 2026 DigiText. All rights reserved.
use crate::db;
use chrono::{DateTime, Utc};
use digitext_core::Role;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
pub const C_JU: &str = r#"
CREATE TABLE IF NOT EXISTS auth_account (
    user_id        TEXT PRIMARY KEY,
    username       TEXT NOT NULL UNIQUE,
    display_name   TEXT NOT NULL,
    role           TEXT NOT NULL,
    project_id     TEXT,
    project_code   TEXT,
    project_name   TEXT,
    salt           BLOB NOT NULL,
    verifier       TEXT NOT NULL,
    secrets        BLOB NOT NULL,
    lease_jwt      TEXT NOT NULL,
    offline_expiry TEXT NOT NULL,
    last_online_at TEXT NOT NULL,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT
);

CREATE TABLE IF NOT EXISTS auth_session (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    user_id    TEXT NOT NULL REFERENCES auth_account(user_id) ON DELETE CASCADE,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    locked     INTEGER NOT NULL DEFAULT 1
);
"#;
pub const C_6: [(&str, &str); 7] = [
    ("trang", "scanned_by"),
    ("ho_so", "created_by"),
    ("ho_so", "finalized_by"),
    ("dossier", "created_by"),
    ("dossier", "updated_by"),
    ("van_ban", "created_by"),
    ("ho_so_history", "nguoi_id"),
];
pub const C_K0: &str = "server_url";
pub const C_JX: &str = "device_id";
pub const C_JZ: &str = "server_pubkey_pem";
pub const C_JY: &str = "max_seen_time";
#[derive(Debug, Clone)]
pub struct Tb {
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub project_id: Option<String>,
    pub project_code: Option<String>,
    pub project_name: Option<String>,
    pub salt: Vec<u8>,
    pub verifier: String,
    pub secrets: Vec<u8>,
    pub lease_jwt: String,
    pub offline_expiry: String,
    pub last_online_at: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tc {
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub project_name: Option<String>,
    pub offline_expiry: String,
    pub last_online_at: String,
}
pub fn f_2nx(r_lm: &Connection, r_ll: &Tb) -> rusqlite::Result<()> {
    r_lm.execute(
        "INSERT INTO auth_account
            (user_id, username, display_name, role, project_id, project_code, project_name,
             salt, verifier, secrets, lease_jwt, offline_expiry, last_online_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13, datetime('now'))
         ON CONFLICT(user_id) DO UPDATE SET
            username = excluded.username,
            display_name = excluded.display_name,
            role = excluded.role,
            project_id = excluded.project_id,
            project_code = excluded.project_code,
            project_name = excluded.project_name,
            salt = excluded.salt,
            verifier = excluded.verifier,
            secrets = excluded.secrets,
            lease_jwt = excluded.lease_jwt,
            offline_expiry = excluded.offline_expiry,
            last_online_at = excluded.last_online_at,
            updated_at = datetime('now')",
        params![
            r_ll.user_id,
            r_ll.username,
            r_ll.display_name,
            r_ll.role.as_str(),
            r_ll.project_id,
            r_ll.project_code,
            r_ll.project_name,
            r_ll.salt,
            r_ll.verifier,
            r_ll.secrets,
            r_ll.lease_jwt,
            r_ll.offline_expiry,
            r_ll.last_online_at,
        ],
    )?;
    Ok(())
}
fn f_2ak(r_ln: &rusqlite::Row<'_>) -> rusqlite::Result<Tb> {
    let role_str: String = r_ln.get(3)?;
    Ok(Tb {
        user_id: r_ln.get(0)?,
        username: r_ln.get(1)?,
        display_name: r_ln.get(2)?,
        role: role_str.parse().map_err(|_| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                format!("cm9sZSBs4bqhOiA={role_str}").into(),
            )
        })?,
        project_id: r_ln.get(4)?,
        project_code: r_ln.get(5)?,
        project_name: r_ln.get(6)?,
        salt: r_ln.get(7)?,
        verifier: r_ln.get(8)?,
        secrets: r_ln.get(9)?,
        lease_jwt: r_ln.get(10)?,
        offline_expiry: r_ln.get(11)?,
        last_online_at: r_ln.get(12)?,
    })
}
const ACCOUNT_COLS: &str = "user_id, username, display_name, role, project_id, project_code, \
     project_name, salt, verifier, secrets, lease_jwt, offline_expiry, last_online_at";
pub fn f_1ea(r_lo: &Connection, username: &str) -> rusqlite::Result<Option<Tb>> {
    r_lo.query_row(
        &format!("SELECT {ACCOUNT_COLS} FROM auth_account WHERE username = ?1"),
        params![username],
        f_2ak,
    )
    .optional()
}
pub fn f_1eb(r_lp: &Connection, user_id: &str) -> rusqlite::Result<Option<Tb>> {
    r_lp.query_row(
        &format!("SELECT {ACCOUNT_COLS} FROM auth_account WHERE user_id = ?1"),
        params![user_id],
        f_2ak,
    )
    .optional()
}
pub fn f_1n4(r_lq: &Connection) -> rusqlite::Result<Vec<Tc>> {
    let mut r_lt = r_lq.prepare(
        "SELECT user_id, username, display_name, role, project_name, offline_expiry, last_online_at
         FROM auth_account ORDER BY last_online_at DESC, username",
    )?;
    let r_ls = r_lt
        .query_map([], |r_lr| {
            let role_str: String = r_lr.get(3)?;
            let Ok(role) = role_str.parse::<Role>() else {
                return Ok(None);
            };
            Ok(Some(Tc {
                user_id: r_lr.get(0)?,
                username: r_lr.get(1)?,
                display_name: r_lr.get(2)?,
                role,
                project_name: r_lr.get(4)?,
                offline_expiry: r_lr.get(5)?,
                last_online_at: r_lr.get(6)?,
            }))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(r_ls)
}
pub fn f_13t(r_lu: &Connection, user_id: &str) -> rusqlite::Result<bool> {
    let r_lv = r_lu.execute(
        "DELETE FROM auth_account WHERE user_id = ?1",
        params![user_id],
    )?;
    Ok(r_lv > 0)
}
pub fn f_2dx(r_lw: &Connection, user_id: &str, locked: bool) -> rusqlite::Result<()> {
    r_lw.execute(
        "INSERT INTO auth_session (id, user_id, started_at, locked)
         VALUES (1, ?1, datetime('now'), ?2)
         ON CONFLICT(id) DO UPDATE SET
            user_id = excluded.user_id,
            started_at = excluded.started_at,
            locked = excluded.locked",
        params![user_id, locked as i64],
    )?;
    Ok(())
}
pub fn f_1ec(r_lx: &Connection) -> rusqlite::Result<Option<(String, bool)>> {
    r_lx.query_row(
        "SELECT user_id, locked FROM auth_session WHERE id = 1",
        [],
        |r_ly| Ok((r_ly.get::<_, String>(0)?, r_ly.get::<_, i64>(1)? != 0)),
    )
    .optional()
}
pub fn f_2eh(r_lz: &Connection, locked: bool) -> rusqlite::Result<()> {
    r_lz.execute(
        "UPDATE auth_session SET locked = ?1 WHERE id = 1",
        params![locked as i64],
    )?;
    Ok(())
}
pub fn f_zz(r_m0: &Connection) -> rusqlite::Result<()> {
    r_m0.execute("DELETE FROM auth_session WHERE id = 1", [])?;
    Ok(())
}
pub fn max_seen_time(r_m1: &Connection) -> rusqlite::Result<Option<DateTime<Utc>>> {
    Ok(db::f_1ew(r_m1, C_JY)?
        .and_then(|r_m2| DateTime::parse_from_rfc3339(&r_m2).ok())
        .map(|r_m3| r_m3.with_timezone(&Utc)))
}
const C_C1: i64 = 90;
pub fn f_rg(r_m4: &Connection, now: DateTime<Utc>) -> rusqlite::Result<()> {
    match max_seen_time(r_m4)? {
        None => {
            db::f_2ei(r_m4, C_JY, Some(&now.to_rfc3339()))?;
        }
        Some(r_m5) if now > r_m5 && now <= r_m5 + chrono::Duration::days(C_C1) => {
            db::f_2ei(r_m4, C_JY, Some(&now.to_rfc3339()))?;
        }
        Some(_) => {}
    }
    Ok(())
}
