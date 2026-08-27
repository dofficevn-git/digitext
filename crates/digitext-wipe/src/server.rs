// Copyright (c) 2026 DigiText. All rights reserved.
use crate::{Depth, PlanItem, Report, Scope, Thd};
use digitext_storage::ObjectStore;
use sqlx::PgPool;
use std::path::{Path, PathBuf};
pub const C_10: &[&str] = &[
    "ocr_fields",
    "ocr_results",
    "ocr_jobs",
    "review_queue",
    "pages",
    "sign_sessions",
    "submission_items",
    "submissions",
    "packages",
    "records",
    "notifications",
    "access_request_items",
    "access_requests",
    "info_requests",
    "integrity_checks",
    "backup_runs",
    "disposal_log",
    "disposal_entries",
    "disposal_batches",
    "documents",
    "upload_batches",
    "audit_log",
    "user_stats",
];
pub const C_9I: &[&str] = &[
    "project_trees",
    "access_fee_rates",
    "engine_versions",
    "user_permission_overrides",
    "project_admins",
    "project_members",
    "refresh_tokens",
    "devices",
    "users",
    "projects",
];
pub const NEVER_TRUNCATE: &str = "_sqlx_migrations";
pub fn f_2iv(r_5ro: Depth) -> Vec<&'static str> {
    let mut r_5rp = C_10.to_vec();
    if r_5ro == Depth::All {
        r_5rp.extend_from_slice(C_9I);
    }
    r_5rp
}
pub fn f_2mg(r_5rq: Depth) -> String {
    format!(
        "TRUNCATE TABLE {} RESTART IDENTITY CASCADE",
        f_2iv(r_5rq).join(", ")
    )
}
pub fn f_139(dsn: &str) -> String {
    let r_5rr = dsn.split("://").nth(1).unwrap_or(dsn);
    let r_5rt = r_5rr.split_once('/').map(|(_, p)| p).unwrap_or("");
    let r_5rs = r_5rt.split(['?', '#']).next().unwrap_or("");
    if r_5rs.is_empty() {
        "(khong-ro)".to_owned()
    } else {
        r_5rs.to_owned()
    }
}
pub fn f_26f(dsn: &str) -> String {
    let Some((scheme, r_5rv)) = dsn.split_once("://") else {
        return dsn.to_owned();
    };
    let Some((r_5ru, host)) = r_5rv.split_once('@') else {
        return dsn.to_owned();
    };
    let user = r_5ru.split_once(':').map(|(u, _)| u).unwrap_or(r_5ru);
    format!("{scheme}://{user}:***@{host}")
}
pub struct Tdp<'a> {
    pub label: String,
    pub store: &'a dyn ObjectStore,
}
pub struct Tl5<'a> {
    pub dsn: String,
    pub pool: &'a PgPool,
    pub stores: Vec<Tdp<'a>>,
    pub data_dir: Option<PathBuf>,
}
async fn f_123(pool: &PgPool, table: &str) -> u64 {
    sqlx::query_scalar::<_, i64>(&format!("SELECT count(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .map(|r_5rw| r_5rw.max(0) as u64)
        .unwrap_or(0)
}
fn measure(r_5rz: &Path) -> (u64, u64) {
    let (mut r_5s1, mut r_5rx) = (0u64, 0u64);
    let mut r_5s3 = vec![r_5rz.to_path_buf()];
    while let Some(r_5ry) = r_5s3.pop() {
        let Ok(r_5s2) = std::fs::read_dir(&r_5ry) else {
            continue;
        };
        for e in r_5s2.flatten() {
            match e.file_type() {
                Ok(t) if t.is_dir() => r_5s3.push(e.path()),
                Ok(_) => {
                    r_5s1 += 1;
                    r_5rx += e.metadata().map(|r_5s0| r_5s0.len()).unwrap_or(0);
                }
                Err(_) => {}
            }
        }
    }
    (r_5s1, r_5rx)
}
pub async fn build_plan(t: &Tl5<'_>, r_5s5: Depth) -> Thd {
    let mut r_5s7 = Vec::new();
    let mut r_5sd = vec![
        "S2jDtG5nIGhvw6BuIHTDoWMgxJHGsOG7o2Mg4oCUIGNo4buJIGNo4bqheSBraGkgQ0jhuq5DIENI4bquTiDEkcOjIHNhbyBsxrB1LCBob+G6t2MgxJHDonkgbMOgIG3DoXkgdGjhu60u"
        .into(),
        "TsOqbiBk4burbmcgZGlnaXRleHQtYXBpIC8gZGlnaXRleHQtb2NyIHRyxrDhu5tjIGtoaSB4b8OhOiB0aeG6v24gdHLDrG5oIMSRYW5nIGNo4bqheSBjw7MgdGjhu4MgZ2hpIGzhuqFpIGThu68gbGnhu4d1IG3hu5tpIG5nYXkgc2F1IGtoaSB0cnVuY2F0ZS4="
        .into(),
    ];
    let mut r_5sb = 0u64;
    for r_5sc in f_2iv(r_5s5) {
        r_5sb += f_123(t.pool, r_5sc).await;
    }
    r_5s7.push(PlanItem::new(
        format!(
            "RMOybmcgdHJvbmcg{}IGLhuqNuZyAoUG9zdGdyZXMp",
            f_2iv(r_5s5).len()
        ),
        f_26f(&t.dsn),
        r_5sb,
    ));
    if t.stores.is_empty() {
        r_5sd
            .push(
                "Q2jGsGEgY+G6pXUgaMOsbmgga2hvIG9iamVjdCBuw6BvIChESUdJVEVYVF9NSU5JT18qIC8gRElHSVRFWFRfUERGX1NUT1JFX0RJUikg4oCUIFBERiB2w6Ag4bqjbmggdHJhbmcgdHLDqm4ga2hvIHPhur0gS0jDlE5HIGLhu4sgeG/DoQ=="
                    .into(),
            );
    }
    for r_5sa in &t.stores {
        match r_5sa.store.list_prefix("").await {
            Ok(r_5s8) => {
                r_5s7
                    .push(
                        PlanItem::new(
                            "T2JqZWN0IHRyb25nIGtobyAoUERGL+G6o25oL2fDs2kp",
                            r_5sa.label.clone(),
                            r_5s8.len() as u64,
                        ),
                    )
            }
            Err(e) => {
                r_5sd
                    .push(
                        format!(
                            "S0jDlE5HIGxp4buHdCBrw6ogxJHGsOG7o2Mga2hvIG9iamVjdCAn{}' ({e}KSDigJQgcGjhuqduIG7DoHkgc+G6vSBLSMOUTkcgYuG7iyB4b8OhLCBwaOG6o2kgZOG7jW4gdGF5",
                            r_5sa.label
                        ),
                    )
            }
        }
    }
    if let Some(r_5s6) = &t.data_dir {
        let (r_5s9, r_5s4) = measure(r_5s6);
        r_5s7.push(
            PlanItem::new(
                "RmlsZSB0csOqbiDEkcSpYSBBUEkgKFBERiBuZ3Xhu5NuLCAucGFydCk=",
                r_5s6.display().to_string(),
                r_5s9,
            )
            .with_bytes(r_5s4),
        );
    }
    if r_5s5 == Depth::All {
        r_5sd
            .push(
                "TeG7qWMgVE/DgE4gQuG7mCB4b8OhIGPhuqMgdMOgaSBraG/huqNuICsgZOG7sSDDoW46IHNhdSDEkcOzIHBo4bqjaSBjaOG6oXkgYGRpZ2l0ZXh0IGFwaSBib290c3RyYXAtc3VwZXJhZG1pbiAtLXVzZXJuYW1lIDx0w6puPmAgbeG7m2kgxJHEg25nIG5o4bqtcCBs4bqhaSDEkcaw4bujYy4="
                    .into(),
            );
    }
    r_5sd
        .push(
            format!(
                "R0nhu64gTOG6oEk6IGLhuqNuZyA={NEVER_TRUNCATE}IChs4buLY2ggc+G7rSBtaWdyYXRpb24pLCBraG/DoSBKV1QgdsOgIGPDoWMgYnVuZGxlIFBFTSB0cnVzdCBzdG9yZSDigJQgxJHDsyBsw6AgY+G6pXUgaMOsbmgsIGtow7RuZyBwaOG6o2kgZOG7ryBsaeG7h3Uu"
            ),
        );
    Thd {
        scope: Scope::Server,
        depth: r_5s5,
        target_id: f_139(&t.dsn),
        items: r_5s7,
        warnings: r_5sd,
    }
}
pub async fn execute(t: &Tl5<'_>, r_5se: Depth) -> Report {
    let mut r_5si = Report::default();
    let sql = f_2mg(r_5se);
    match sqlx::query(&sql).execute(t.pool).await {
        Ok(_) => r_5si.removed += f_2iv(r_5se).len() as u64,
        Err(e) => r_5si
            .errors
            .push(format!("VFJVTkNBVEUgdGjhuqV0IGLhuqFpOiA={e}")),
    }
    for r_5sh in &t.stores {
        match r_5sh.store.list_prefix("").await {
            Ok(r_5sg) => {
                for k in r_5sg {
                    match r_5sh.store.delete(&k).await {
                        Ok(()) => r_5si.removed += 1,
                        Err(e) => r_5si
                            .errors
                            .push(format!("eG/DoSBvYmplY3Qg{}/{k}: {e}", r_5sh.label)),
                    }
                }
            }
            Err(e) => r_5si.errors.push(format!(
                "bGnhu4d0IGvDqiBraG8gb2JqZWN0ICc={}': {e}",
                r_5sh.label
            )),
        }
    }
    if let Some(r_5sf) = &t.data_dir {
        remove_contents(r_5sf, &mut r_5si);
    }
    r_5si
}
fn remove_contents(r_5sj: &Path, r_5so: &mut Report) {
    let Ok(r_5sn) = std::fs::read_dir(r_5sj) else {
        return;
    };
    for e in r_5sn.flatten() {
        let p = e.path();
        let r_5sk = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let (r_5sl, _) = if r_5sk { measure(&p) } else { (1, 0) };
        let r_5sm = if r_5sk {
            std::fs::remove_dir_all(&p)
        } else {
            std::fs::remove_file(&p)
        };
        match r_5sm {
            Ok(()) => r_5so.removed += r_5sl,
            Err(e) => r_5so.errors.push(format!("eG/DoSA={}: {e}", p.display())),
        }
    }
}
