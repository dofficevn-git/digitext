// Copyright (c) 2026 DigiText. All rights reserved.
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::{self, Db, Tks};
use crate::error::AppError;
use digitext_core::Permission;
use serde::{Deserialize, Serialize};
use tauri::State;
fn f_1p6(r_2r9: &State<'_, Db>, user_id: &str) -> Result<Tks, AppError> {
    let r_2ra = r_2r9.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(db::f_2oe(&r_2ra, user_id)?)
}
#[tauri::command]
pub fn stats_local(auth: State<'_, AuthState>, db: State<'_, Db>) -> Result<Tks, AppError> {
    let r_2rb = auth.require_perm(Permission::StatsView)?;
    f_1p6(&db, &r_2rb.user_id)
}
#[tauri::command]
pub async fn stats_push(db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<Tks, AppError> {
    let r_2rd = auth.require_perm(Permission::StatsView)?;
    let stats = f_1p6(&db, &r_2rd.user_id)?;
    let body = serde_json::json!(
        { "pagesScanned" : stats.pages_scanned, "hoSoCount" : stats.ho_so_count,
        "pdfCount" : stats.pdf_count, "pdfBytes" : stats.pdf_bytes, }
    );
    f_2r8(&db, &auth, Permission::StatsView, move |r_2rc, r_2re| {
        let body = body.clone();
        async move { client::f_23c(&r_2rc, &r_2re, "/stats", &body).await }
    })
    .await?;
    Ok(stats)
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatRow {
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub project_id: Option<String>,
    pub pages_scanned: i64,
    pub ho_so_count: i64,
    pub pdf_count: i64,
    pub pdf_bytes: i64,
    pub files_uploaded: i64,
    pub bytes_uploaded: i64,
    pub reported_at: Option<String>,
}
#[tauri::command]
pub async fn stats_summary(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Vec<StatRow>, AppError> {
    f_2r8(
            &db,
            &auth,
            Permission::StatsViewProject,
            move |r_2rf, r_2rg| async move {
                client::f_1eo::<Vec<StatRow>>(&r_2rf, &r_2rg, "/stats").await
            },
        )
        .await
}
