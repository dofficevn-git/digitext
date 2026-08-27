// Copyright (c) 2026 DigiText. All rights reserved.
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use serde::Deserialize;
use tauri::State;
#[tauri::command]
pub async fn disposal_entries(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<serde_json::Value, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::DisposalView,
        move |r_1im, r_1in| async move {
            client::f_1eo::<serde_json::Value>(&r_1im, &r_1in, "/disposal").await
        },
    )
    .await
}
#[tauri::command]
pub async fn disposal_sweep(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<serde_json::Value, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::DisposalReview,
        move |r_1io, r_1ip| async move {
            client::f_23d::<serde_json::Value>(
                &r_1io,
                &r_1ip,
                "/disposal/sweep",
                &serde_json::json!({}),
            )
            .await
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tj1 {
    pub document_id: String,
    pub het_han: Option<String>,
}
#[tauri::command]
pub async fn disposal_set_retention(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tj1,
) -> Result<(), AppError> {
    let Tj1 {
        document_id,
        het_han,
    } = args;
    f_2r8(
        &db,
        &auth,
        Permission::DisposalReview,
        move |r_1iq, r_1ir| {
            let (document_id, het_han) = (document_id.clone(), het_han.clone());
            async move {
                client::f_23c(
                    &r_1iq,
                    &r_1ir,
                    "/disposal/retention",
                    &serde_json::json!(
                        { "documentId" : document_id, "hetHan" : het_han }
                    ),
                )
                .await
            }
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T6w {
    pub id: String,
    pub note: Option<String>,
    pub action: String,
}
#[tauri::command]
pub async fn disposal_entry_action(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T6w,
) -> Result<(), AppError> {
    if !matches!(args.action.as_str(), "note" | "keep") {
        return Err(AppError::validation(format!(
            "hành động không hợp lệ: {}",
            args.action
        )));
    }
    let T6w { id, note, action } = args;
    f_2r8(
        &db,
        &auth,
        Permission::DisposalReview,
        move |r_1is, r_1it| {
            let (id, note, action) = (id.clone(), note.clone(), action.clone());
            async move {
                client::f_23c(
                    &r_1is,
                    &r_1it,
                    &format!("/disposal/entries/{id}/{action}"),
                    &serde_json::json!({ "note" : note }),
                )
                .await
            }
        },
    )
    .await
}
#[tauri::command]
pub async fn disposal_batches(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<serde_json::Value, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::DisposalView,
        move |r_1iu, r_1iv| async move {
            client::f_1eo::<serde_json::Value>(&r_1iu, &r_1iv, "/disposal/batches").await
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdArgs {
    pub id: String,
}
#[tauri::command]
pub async fn disposal_batch_get(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: IdArgs,
) -> Result<serde_json::Value, AppError> {
    let id = args.id;
    f_2r8(&db, &auth, Permission::DisposalView, move |r_1iw, r_1ix| {
        let id = id.clone();
        async move {
            client::f_1eo::<serde_json::Value>(&r_1iw, &r_1ix, &format!("/disposal/batches/{id}"))
                .await
        }
    })
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T36 {
    pub nam: i32,
    pub entry_ids: Vec<String>,
}
#[tauri::command]
pub async fn disposal_batch_create(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T36,
) -> Result<serde_json::Value, AppError> {
    let T36 { nam, entry_ids } = args;
    f_2r8(
        &db,
        &auth,
        Permission::DisposalReview,
        move |r_1iy, r_1iz| {
            let entry_ids = entry_ids.clone();
            async move {
                client::f_23d::<serde_json::Value>(
                    &r_1iy,
                    &r_1iz,
                    "/disposal/batches",
                    &serde_json::json!({ "nam" : nam, "z{kȇl" : entry_ids }),
                )
                .await
            }
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T15 {
    pub id: String,
    pub action: String,
    pub note: Option<String>,
    pub quyet_dinh_so: Option<String>,
    pub tu_quyet: Option<bool>,
}
#[tauri::command]
pub async fn disposal_batch_action(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T15,
) -> Result<serde_json::Value, AppError> {
    let r_1j3 = match args.action.as_str() {
        "council" | "appraise" => Permission::DisposalReview,
        "decide" | "execute" => Permission::DisposalDecide,
        other => {
            return Err(AppError::validation(format!(
                "hành động không hợp lệ: {other}"
            )));
        }
    };
    let T15 {
        id,
        action,
        note,
        quyet_dinh_so,
        tu_quyet,
    } = args;
    f_2r8(&db, &auth, r_1j3, move |r_1j0, r_1j5| {
        let (id, action, note, r_1j4, r_1j6) = (
            id.clone(),
            action.clone(),
            note.clone(),
            quyet_dinh_so.clone(),
            tu_quyet,
        );
        async move {
            let r_1j1 = serde_json::json!(
                { "note" : note, "quyetDinhSo" : r_1j4, "tuQuyet" : r_1j6
                .unwrap_or(false), }
            );
            let r_1j2 = format!("/disposal/batches/{id}/{action}");
            if action == "execute" {
                client::f_23d::<serde_json::Value>(&r_1j0, &r_1j5, &r_1j2, &r_1j1).await
            } else {
                client::f_23c(&r_1j0, &r_1j5, &r_1j2, &r_1j1).await?;
                Ok(serde_json::Value::Null)
            }
        }
    })
    .await
}
