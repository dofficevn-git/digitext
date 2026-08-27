// Copyright (c) 2026 DigiText. All rights reserved.
use crate::archive::commands::apply_folder_tree;
use crate::archive::types::{PhongIn, T97};
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::{self, Db};
use crate::error::AppError;
use digitext_core::Permission;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Toh {
    pub phong: PhongIn,
    pub ho_sos: Vec<T97>,
    #[serde(default)]
    pub strip_accents: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_code: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thv {
    pub project_id: String,
    pub phong: PhongIn,
    pub ho_sos: Vec<T97>,
    #[serde(default)]
    pub strip_accents: bool,
}
fn f_s1(project_id: &str) -> String {
    format!("applied_tree_version:{project_id}")
}
#[tauri::command]
pub async fn tree_publish(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Thv,
) -> Result<i32, AppError> {
    if args.project_id.trim().is_empty() {
        return Err(AppError::validation(
            "VGhp4bq/dSBk4buxIMOhbiDEkcOtY2ggxJHhu4MgcGjDoXQgY8OieSB0aMawIG3hu6VjLg==",
        ));
    }
    if args.phong.ma_du_an.trim().is_empty() || args.phong.ten_du_an.trim().is_empty() {
        return Err(AppError::validation(
            "VGhp4bq/dSB0aMO0bmcgdGluIGPGoSBxdWFuIChQaMO0bmcpLg==",
        ));
    }
    let site_code = {
        let r_32e = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        db::f_1ew(&r_32e, crate::data_share::C_KA)?.filter(|r_32d| !r_32d.trim().is_empty())
    };
    let payload = serde_json::to_value(Toh {
        phong: args.phong,
        ho_sos: args.ho_sos,
        strip_accents: args.strip_accents,
        site_code,
    })
    .map_err(|e| {
        AppError::internal(format!(
            "xJHDs25nIGfDs2kgY8OieSB0aMawIG3hu6VjIGzhu5dpOiA={e}"
        ))
    })?;
    let project_id = args.project_id;
    #[derive(Deserialize)]
    struct Thw {
        version: i32,
    }
    let r_32f = f_2r8(&db, &auth, Permission::TreeAssign, move |r_32c, r_32g| {
        let (project_id, payload) = (project_id.clone(), payload.clone());
        async move {
            client::f_24x::<Thw>(
                &r_32c,
                &r_32g,
                &format!("/projects/{project_id}/tree"),
                &payload,
            )
            .await
        }
    })
    .await?;
    Ok(r_32f.version)
}
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Thy {
    pub applied: bool,
    pub version: i32,
    pub created: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offered_site_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_device_id: Option<String>,
}
#[derive(Debug, Deserialize)]
struct Toi {
    version: i32,
    payload: Toh,
}
#[tauri::command]
pub async fn tree_pull(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Thy, AppError> {
    let r_32l = auth.require_perm(Permission::TreeView)?;
    let project_id = r_32l
        .project_id
        .clone()
        .ok_or_else(|| {
            AppError::new(
                "no_project",
                "VMOgaSBraG/huqNuIGNoxrBhIGfhuq9uIGThu7Egw6FuIG7DoG8g4oCUIGxpw6puIGjhu4cgcXXhuqNuIHRy4buLLg==",
            )
        })?;
    let pid = project_id.clone();
    let tree = f_2r8(&db, &auth, Permission::TreeView, move |r_32i, r_32m| {
        let pid = pid.clone();
        async move { client::f_1eo::<Toi>(&r_32i, &r_32m, &format!("/projects/{pid}/tree")).await }
    })
    .await
    .map_err(|e| {
        if e.code() == "not_found" {
            AppError::new(
                "no_tree",
                "UXXhuqNuIHRy4buLIGNoxrBhIHBow6F0IGPDonkgdGjGsCBt4bulYyBjaG8gZOG7sSDDoW4gbsOgeS4=",
            )
        } else {
            e
        }
    })?;
    crate::data_share::f_265(
        &app,
        tree.payload.site_code.as_deref(),
        crate::data_share::SiteCodeSource::Server,
    );
    let r_32h: i32 = {
        let guard = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        db::f_1ew(&guard, &f_s1(&project_id))?
            .and_then(|r_32k| r_32k.parse().ok())
            .unwrap_or(0)
    };
    if tree.version <= r_32h {
        return Ok(Thy {
            applied: false,
            version: r_32h,
            created: 0,
            ..Default::default()
        });
    }
    let r_32j = apply_folder_tree(
        &app,
        &db,
        tree.payload.phong,
        tree.payload.ho_sos,
        tree.payload.strip_accents,
        None,
        &r_32l.user_id,
    )
    .await?;
    {
        let guard = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        db::f_2ei(&guard, &f_s1(&project_id), Some(&tree.version.to_string()))?;
    }
    tracing::info!(
        project = % project_id, version = tree.version, created = r_32j.created,
        "dHJlZV9wdWxsOiDEkcOjIMOhcCBjw6J5IHRoxrAgbeG7pWMgbeG7m2k="
    );
    Ok(Thy {
        applied: true,
        version: tree.version,
        created: r_32j.created,
        ..Default::default()
    })
}
