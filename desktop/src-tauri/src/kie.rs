// Copyright (c) 2026 DigiText. All rights reserved.
//! Kho ví dụ dạy máy bóc trường — lệnh desktop gọi thẳng API /kie/examples (5/10/2026).
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use tauri::State;

type J = serde_json::Value;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KieListArgs {
    project_id: Option<String>,
    scope: Option<String>,
    active: Option<bool>,
    q: Option<String>,
    order: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

fn list_path(a: &KieListArgs) -> Result<String, AppError> {
    let mut u = reqwest::Url::parse("http://x/kie/examples").expect("URL tĩnh");
    {
        let mut qp = u.query_pairs_mut();
        if let Some(p) = a.project_id.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
            if !p.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
                return Err(AppError::validation("mã dự án không hợp lệ"));
            }
            qp.append_pair("projectId", p);
        }
        for (k, v) in [("scope", &a.scope), ("q", &a.q), ("order", &a.order)] {
            if let Some(v) = v.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
                qp.append_pair(k, v);
            }
        }
        if let Some(v) = a.active {
            qp.append_pair("active", if v { "true" } else { "false" });
        }
        if let Some(v) = a.limit {
            qp.append_pair("limit", &v.to_string());
        }
        if let Some(v) = a.offset {
            qp.append_pair("offset", &v.to_string());
        }
    }
    Ok(match u.query() {
        Some(q) if !q.is_empty() => format!("{}?{q}", u.path()),
        _ => u.path().to_owned(),
    })
}

#[tauri::command]
pub async fn kie_examples_list(db: State<'_, Db>, auth: State<'_, AuthState>, args: KieListArgs) -> Result<J, AppError> {
    let path = list_path(&args)?;
    f_2r8(&db, &auth, Permission::DocumentReview, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<J>(&base, &token, &path).await }
    })
    .await
}

#[tauri::command]
pub async fn kie_example_set_active(db: State<'_, Db>, auth: State<'_, AuthState>, id: String, active: bool) -> Result<J, AppError> {
    let id = id.trim().to_owned();
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        return Err(AppError::validation("mã ví dụ không hợp lệ"));
    }
    let path = format!("/kie/examples/{id}");
    let body = serde_json::json!({ "active": active });
    f_2r8(&db, &auth, Permission::DocumentReview, move |base, token| {
        let (path, body) = (path.clone(), body.clone());
        async move { client::f_20e::<J>(&base, &token, &path, &body).await }
    })
    .await
}
