// Copyright (c) 2026 DigiText. All rights reserved.
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use base64::Engine as _;
use digitext_core::Permission;
use serde::Deserialize;
use tauri::State;
#[tauri::command]
pub async fn access_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<serde_json::Value, AppError> {
    f_2r8(
            &db,
            &auth,
            Permission::AccessView,
            move |r_0, r_1| async move {
                client::f_1eo::<serde_json::Value>(&r_0, &r_1, "/access").await
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
pub async fn access_get(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: IdArgs,
) -> Result<serde_json::Value, AppError> {
    let id = args.id;
    f_2r8(
            &db,
            &auth,
            Permission::AccessView,
            move |r_2, r_3| {
                let id = id.clone();
                async move {
                    client::f_1eo::<
                        serde_json::Value,
                    >(&r_2, &r_3, &format!("/access/{id}"))
                        .await
                }
            },
        )
        .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyArgs {
    pub body: serde_json::Value,
}
#[tauri::command]
pub async fn access_create(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: BodyArgs,
) -> Result<serde_json::Value, AppError> {
    let body = args.body;
    f_2r8(&db, &auth, Permission::AccessRequest, move |r_4, r_5| {
        let body = body.clone();
        async move { client::f_23d::<serde_json::Value>(&r_4, &r_5, "/access", &body).await }
    })
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewArgs {
    pub id: String,
    pub item_id: String,
    pub verdict: String,
    pub ly_do: Option<String>,
    pub dieu_kien: bool,
}
#[tauri::command]
pub async fn access_review_item(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: ReviewArgs,
) -> Result<(), AppError> {
    let ReviewArgs {
        id,
        item_id,
        verdict,
        ly_do,
        dieu_kien,
    } = args;
    f_2r8(&db, &auth, Permission::AccessReview, move |r_6, r_7| {
        let (id, item_id, verdict, ly_do) =
            (id.clone(), item_id.clone(), verdict.clone(), ly_do.clone());
        async move {
            client::f_23c(
                &r_6,
                &r_7,
                &format!("/access/{id}/items/{item_id}/review"),
                &serde_json::json!(
                    { "verdict" : verdict, "lyDo" : ly_do, "dieuKien" :
                    dieu_kien }
                ),
            )
            .await
        }
    })
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
pub async fn access_action(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: ActionArgs,
) -> Result<serde_json::Value, AppError> {
    if !matches!(args.action.as_str(), "approve" | "refuse" | "pay" | "dip") {
        return Err(AppError::validation(format!(
            "hành động không hợp lệ: {}",
            args.action
        )));
    }
    let ActionArgs { id, action, note } = args;
    f_2r8(&db, &auth, Permission::AccessReview, move |r_8, r_9| {
        let (id, action, note) = (id.clone(), action.clone(), note.clone());
        async move {
            client::f_23d::<serde_json::Value>(
                &r_8,
                &r_9,
                &format!("/access/{id}/{action}"),
                &serde_json::json!({ "note" : note }),
            )
            .await
        }
    })
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ta1 {
    pub id: String,
    pub item_id: String,
}
#[tauri::command]
pub async fn access_read(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Ta1,
) -> Result<String, AppError> {
    let Ta1 { id, item_id } = args;
    let r_b = f_2r8(
            &db,
            &auth,
            Permission::AccessView,
            move |r_a, r_c| {
                let (id, item_id) = (id.clone(), item_id.clone());
                async move {
                    client::f_1ei(
                            &r_a,
                            &r_c,
                            &format!("/access/{id}/items/{item_id}/read"),
                        )
                        .await
                }
            },
        )
        .await?;
    Ok(base64::engine::general_purpose::STANDARD.encode(r_b))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadArgs {
    pub id: String,
    pub item_id: String,
    pub file_name: String,
}
#[tauri::command]
pub async fn access_download(
    app: tauri::AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: DownloadArgs,
) -> Result<String, AppError> {
    let (id, item_id) = (args.id.clone(), args.item_id.clone());
    let r_e = f_2r8(&db, &auth, Permission::AccessView, move |r_d, r_l| {
        let (id, item_id) = (id.clone(), item_id.clone());
        async move {
            client::f_1ei(
                &r_d,
                &r_l,
                &format!("/access/{id}/items/{item_id}/download"),
            )
            .await
        }
    })
    .await?;
    let r_i = crate::archive::paths::f_29d(&app, crate::archive::paths::f_112(&db).as_deref())?;
    let r_f = r_i.join("_khai_thac");
    let file_name = args.file_name.clone();
    let r_h = tauri::async_runtime::spawn_blocking(move || -> std::io::Result<String> {
        std::fs::create_dir_all(&r_f)?;
        let r_k = std::path::PathBuf::from(&file_name)
            .file_name()
            .map(|r_j| r_j.to_string_lossy().into_owned())
            .unwrap_or_else(|| "ban-sao.pdf".into());
        let r_g = r_f.join(r_k);
        std::fs::write(&r_g, &r_e)?;
        Ok(r_g.to_string_lossy().into_owned())
    })
    .await
    .map_err(|e| AppError::internal(format!("luồng lưu file lỗi: {e}")))?
    .map_err(|e| AppError::internal(format!("lưu file lỗi: {e}")))?;
    tracing::info!(
        request = % args.id, "access_download: đã lưu bản sao"
    );
    Ok(r_h)
}
#[tauri::command]
pub async fn access_fees_get(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<serde_json::Value, AppError> {
    f_2r8(
        &db,
        &auth,
        Permission::AccessView,
        move |r_m, r_n| async move {
            client::f_1eo::<serde_json::Value>(&r_m, &r_n, "/access/fees").await
        },
    )
    .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T7w {
    pub phi_doc_tai_lieu: i64,
    pub phi_sao_trang: i64,
}
#[tauri::command]
pub async fn access_fees_set(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T7w,
) -> Result<serde_json::Value, AppError> {
    let T7w {
        phi_doc_tai_lieu,
        phi_sao_trang,
    } = args;
    f_2r8(
        &db,
        &auth,
        Permission::AccessReview,
        move |r_o, r_p| async move {
            client::f_24x::<serde_json::Value>(
                &r_o,
                &r_p,
                "/access/fees",
                &serde_json::json!(
                    { "phiDocTaiLieu" : phi_doc_tai_lieu, "phiSaoTrang" :
                    phi_sao_trang, }
                ),
            )
            .await
        },
    )
    .await
}
