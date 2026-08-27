// Copyright (c) 2026 DigiText. All rights reserved.
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use serde::{Deserialize, Serialize};
use tauri::State;
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tm5 {
    pub id: String,
    pub project_id: String,
    pub ma_yeu_cau: String,
    pub org_name: String,
    pub org_code: String,
    pub so_lan_nop: i32,
    pub fond_code: String,
    pub cach_thuc: String,
    pub status: String,
    pub decision_note: Option<String>,
    pub reconcile_note: Option<String>,
    pub approval_doc: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
#[tauri::command]
pub async fn submissions_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Vec<Tm5>, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::SubmissionView,
        move |r_31f, r_31g| async move {
            client::f_1eo::<Vec<Tm5>>(&r_31f, &r_31g, "/submissions").await
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
pub async fn submission_get(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: IdArgs,
) -> Result<serde_json::Value, AppError> {
    let id = args.id;
    f_2r8(
        &db,
        &auth,
        Permission::SubmissionView,
        move |r_31h, r_31i| {
            let id = id.clone();
            async move {
                client::f_1eo::<serde_json::Value>(&r_31h, &r_31i, &format!("/submissions/{id}"))
                    .await
            }
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tik {
    pub body: serde_json::Value,
}
#[tauri::command]
pub async fn submission_register(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tik,
) -> Result<serde_json::Value, AppError> {
    let body = args.body;
    f_2r8(
        &db,
        &auth,
        Permission::SubmissionCreate,
        move |r_31j, r_31k| {
            let body = body.clone();
            async move {
                client::f_23d::<serde_json::Value>(&r_31j, &r_31k, "/submissions", &body).await
            }
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionArgs {
    pub id: String,
    pub action: String,
    pub note: Option<String>,
}
#[tauri::command]
pub async fn submission_action(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: ActionArgs,
) -> Result<serde_json::Value, AppError> {
    let r_31m = match args.action.as_str() {
        "confirm" | "refuse" | "receive" => Permission::SubmissionReview,
        "reconcile" => Permission::SubmissionView,
        "complete" => Permission::SubmissionApprove,
        other => {
            return Err(AppError::validation(format!(
                "aMOgbmggxJHhu5luZyBraMO0bmcgaOG7o3AgbOG7hzog{other}"
            )));
        }
    };
    let (id, action, note) = (args.id, args.action, args.note);
    f_2r8(&db, &auth, r_31m, move |r_31l, r_31n| {
        let (id, action, note) = (id.clone(), action.clone(), note.clone());
        async move {
            client::f_23d::<serde_json::Value>(
                &r_31l,
                &r_31n,
                &format!("/submissions/{id}/{action}"),
                &serde_json::json!({ "note" : note }),
            )
            .await
        }
    })
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tj8 {
    pub id: String,
    pub item_id: String,
    pub result: String,
    pub note: Option<String>,
}
#[tauri::command]
pub async fn submission_review_item(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tj8,
) -> Result<(), AppError> {
    let Tj8 {
        id,
        item_id,
        result,
        note,
    } = args;
    f_2r8(
        &db,
        &auth,
        Permission::SubmissionReview,
        move |r_31o, r_31p| {
            let (id, item_id, result, note) =
                (id.clone(), item_id.clone(), result.clone(), note.clone());
            async move {
                client::f_23c(
                    &r_31o,
                    &r_31p,
                    &format!("/submissions/{id}/items/{item_id}/review"),
                    &serde_json::json!({ "result" : result, "note" : note }),
                )
                .await
            }
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tj {
    pub id: String,
    pub quyet_dinh_so: String,
    pub note: Option<String>,
}
#[tauri::command]
pub async fn submission_approve(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tj,
) -> Result<serde_json::Value, AppError> {
    let Tj {
        id,
        quyet_dinh_so,
        note,
    } = args;
    f_2r8(
        &db,
        &auth,
        Permission::SubmissionApprove,
        move |r_31q, r_31s| {
            let (id, r_31r, note) = (id.clone(), quyet_dinh_so.clone(), note.clone());
            async move {
                client::f_23d::<serde_json::Value>(
                    &r_31q,
                    &r_31s,
                    &format!("/submissions/{id}/approve"),
                    &serde_json::json!({ "quyetDinhSo" : r_31r, "note" : note }),
                )
                .await
            }
        },
    )
    .await
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tea {
    pub id: String,
    pub submission_id: Option<String>,
    pub kind: String,
    pub message: String,
    pub read_at: Option<String>,
    pub created_at: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Te9 {
    pub unread: i64,
    pub items: Vec<Tea>,
}
#[tauri::command]
pub async fn notifications_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Te9, AppError> {
    f_2r8(
            &db,
            &auth,
            Permission::SubmissionView,
            move |r_31t, r_31u| async move {
                client::f_1eo::<Te9>(&r_31t, &r_31u, "/notifications").await
            },
        )
        .await
}
#[tauri::command]
pub async fn notification_read(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: IdArgs,
) -> Result<(), AppError> {
    let id = args.id;
    f_2r8(
        &db,
        &auth,
        Permission::SubmissionView,
        move |r_31v, r_31w| {
            let id = id.clone();
            async move {
                client::f_23c(
                    &r_31v,
                    &r_31w,
                    &format!("/notifications/{id}/read"),
                    &serde_json::json!({}),
                )
                .await
            }
        },
    )
    .await
}
#[tauri::command]
pub async fn notifications_read_all(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<(), AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::SubmissionView,
        move |r_31x, r_31y| async move {
            client::f_23c(
                &r_31x,
                &r_31y,
                "/notifications/read-all",
                &serde_json::json!({}),
            )
            .await
        },
    )
    .await
}
#[tauri::command]
pub async fn submission_returned(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: IdArgs,
) -> Result<serde_json::Value, AppError> {
    let id = args.id;
    f_2r8(
        &db,
        &auth,
        Permission::SubmissionView,
        move |r_31z, r_320| {
            let id = id.clone();
            async move {
                client::f_1eo::<serde_json::Value>(
                    &r_31z,
                    &r_320,
                    &format!("/submissions/{id}/returned"),
                )
                .await
            }
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportArgs {
    pub id: String,
    pub ghi_chu: Option<String>,
    pub loai: Option<String>,
}
#[tauri::command]
pub async fn submission_report(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: ReportArgs,
) -> Result<serde_json::Value, AppError> {
    let ReportArgs { id, ghi_chu, loai } = args;
    f_2r8(
        &db,
        &auth,
        Permission::SubmissionReview,
        move |r_321, r_322| {
            let (id, ghi_chu, loai) = (id.clone(), ghi_chu.clone(), loai.clone());
            async move {
                client::f_23d::<serde_json::Value>(
                    &r_321,
                    &r_322,
                    &format!("/submissions/{id}/report"),
                    &serde_json::json!({ "ghiChu" : ghi_chu, "loai" : loai }),
                )
                .await
            }
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T52 {
    pub id: String,
    pub file_name: String,
}
#[tauri::command]
pub async fn submission_dossier(
    app: tauri::AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T52,
) -> Result<String, AppError> {
    let id = args.id.clone();
    let r_324 = f_2r8(
            &db,
            &auth,
            Permission::SubmissionView,
            move |r_323, r_32b| {
                let id = id.clone();
                async move {
                    client::f_1ei(&r_323, &r_32b, &format!("/submissions/{id}/dossier"))
                        .await
                }
            },
        )
        .await?;
    let r_328 = crate::archive::paths::f_29d(&app, crate::archive::paths::f_112(&db).as_deref())?;
    let r_325 = r_328.join("_thu_nop");
    let file_name = args.file_name.clone();
    let r_327 = tauri::async_runtime::spawn_blocking(move || -> std::io::Result<String> {
        std::fs::create_dir_all(&r_325)?;
        let r_32a = std::path::PathBuf::from(&file_name)
            .file_name()
            .map(|r_329| r_329.to_string_lossy().into_owned())
            .unwrap_or_else(|| "ho-so-qua-trinh.zip".into());
        let r_326 = r_325.join(r_32a);
        std::fs::write(&r_326, &r_324)?;
        Ok(r_326.to_string_lossy().into_owned())
    })
    .await
    .map_err(|e| AppError::internal(format!("bHXhu5NuZyBsxrB1IGZpbGUgbOG7l2k6IA=={e}")))?
    .map_err(|e| AppError::internal(format!("bMawdSBmaWxlIGzhu5dpOiA={e}")))?;
    tracing::info!(
        submission = % args.id,
        "c3VibWlzc2lvbl9kb3NzaWVyOiDEkcOjIGzGsHUgaOG7kyBzxqEgcXXDoSB0csOsbmg="
    );
    Ok(r_327)
}
