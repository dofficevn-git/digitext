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
pub struct Tfc {
    pub page_no: i32,
    pub engine_version_id: String,
    pub text_source: String,
    pub field_key: String,
    pub value: String,
    #[serde(default)]
    pub bbox: Option<serde_json::Value>,
    #[serde(default)]
    pub confidence: Option<f32>,
    pub verified: bool,
    pub updated_at: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T4z {
    pub document_id: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T33 {
    pub document_id: String,
    pub page_no: i32,
    pub field_key: String,
    pub value: String,
    #[serde(default)]
    pub engine_version_id: Option<String>,
    #[serde(default)]
    pub bbox: Option<serde_json::Value>,
    #[serde(default)]
    pub verified: Option<bool>,
}
#[tauri::command]
pub async fn ocr_fields_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T4z,
) -> Result<Vec<Tfc>, AppError> {
    let id = args.document_id.trim().to_string();
    if id.is_empty() {
        return Err(AppError::validation("VGhp4bq/dSBtw6MgdMOgaSBsaeG7h3Uu"));
    }
    f_2r8(&db, &auth, Permission::DocumentList, move |r_29n, r_29o| {
        let id = id.clone();
        async move {
            client::f_1eo::<Vec<Tfc>>(&r_29n, &r_29o, &format!("/documents/{id}/fields")).await
        }
    })
    .await
}
#[tauri::command]
pub async fn ocr_field_correct(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T33,
) -> Result<Tfc, AppError> {
    let id = args.document_id.trim().to_string();
    if id.is_empty() {
        return Err(AppError::validation("VGhp4bq/dSBtw6MgdMOgaSBsaeG7h3Uu"));
    }
    if args.field_key.trim().is_empty() {
        return Err(AppError::validation("VGhp4bq/dSBraG/DoSB0csaw4budbmcu"));
    }
    let mut body = serde_json::json!(
        { "pageNo" : args.page_no, "fieldKey" : args.field_key.trim(), "value" : args
        .value, }
    );
    if let Some(r_29r) = args
        .engine_version_id
        .as_deref()
        .map(str::trim)
        .filter(|r_29s| !r_29s.is_empty())
    {
        body["engineVersionId"] = serde_json::Value::String(r_29r.to_string());
    }
    if let Some(r_29p) = args.bbox.clone() {
        body["bbox"] = r_29p;
    }
    if let Some(r_29u) = args.verified {
        body["verified"] = serde_json::Value::Bool(r_29u);
    }
    f_2r8(&db, &auth, Permission::DocumentList, move |r_29q, r_29t| {
        let (id, body) = (id.clone(), body.clone());
        async move {
            client::f_20e::<Tfc>(&r_29q, &r_29t, &format!("/documents/{id}/fields"), &body).await
        }
    })
    .await
}
