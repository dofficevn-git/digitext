// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use crate::routes::users::UserDto;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::{Permission, Role};
use digitext_db::projects::{self, Project};
use digitext_db::{audit, trees, users};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/projects", get(list_projects).post(f_12c))
        .route("/projects/{id}", axum::routing::patch(f_20g))
        .route("/projects/{id}/admins", put(f_2ed).post(f_qt))
        .route(
            "/projects/{id}/admins/{admin_id}",
            axum::routing::delete(f_276),
        )
        .route("/projects/{id}/members", get(list_members).post(add_member))
        .route("/projects/{id}/tree", put(f_24r).get(f_24s))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDto {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub admin_ids: Vec<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
impl ProjectDto {
    fn new(r_nt: Project, admin_ids: Vec<Uuid>) -> Self {
        ProjectDto {
            id: r_nt.id,
            code: r_nt.code,
            name: r_nt.name,
            admin_ids,
            is_active: r_nt.is_active,
            created_at: r_nt.created_at,
        }
    }
}
async fn f_17s(r_nz: &AppState, r_nw: Vec<Project>) -> Result<Vec<ProjectDto>, ApiError> {
    let r_nv: Vec<Uuid> = r_nw.iter().map(|p| p.id).collect();
    let r_nx = projects::f_rd(&r_nz.pool, &r_nv).await?;
    let mut r_nu: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for (r_ny, admin_id) in r_nx {
        r_nu.entry(r_ny).or_default().push(admin_id);
    }
    Ok(r_nw
        .into_iter()
        .map(|p| {
            let admins = r_nu.remove(&p.id).unwrap_or_default();
            ProjectDto::new(p, admins)
        })
        .collect())
}
async fn f_17r(r_o1: &AppState, r_o0: Project) -> Result<ProjectDto, ApiError> {
    let admins = projects::f_re(&r_o1.pool, r_o0.id).await?;
    Ok(ProjectDto::new(r_o0, admins))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3e {
    pub code: String,
    pub name: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tgs {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tlb {
    pub admin_ids: Vec<Uuid>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Te {
    pub admin_id: Uuid,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Th {
    pub user_id: Uuid,
}
async fn list_projects(
    State(r_o4): State<AppState>,
    r_o2: Ts,
) -> Result<Json<Vec<ProjectDto>>, ApiError> {
    let r_o3 = match f_29a(&r_o4.pool, &r_o2.user).await? {
        Scope::All => projects::list(&r_o4.pool).await?,
        Scope::Projects(_) => projects::f_1ng(&r_o4.pool, r_o2.user.id).await?,
    };
    Ok(Json(f_17s(&r_o4, r_o3).await?))
}
async fn f_12c(
    State(r_ob): State<AppState>,
    r_o5: Ts,
    Json(r_oa): Json<T3e>,
) -> Result<(StatusCode, Json<ProjectDto>), ApiError> {
    r_o5.require_perm(Permission::ProjectCreate)?;
    if r_oa.code.trim().is_empty() || r_oa.name.trim().is_empty() {
        return Err(ApiError::bad_request(
            "code và name không được rỗng",
        ));
    }
    let r_o9 = if r_o5.user.role == Role::Admin {
        Some(r_o5.user.id)
    } else {
        None
    };
    let r_o6 = projects::f_12a(&r_ob.pool, r_oa.code.trim(), r_oa.name.trim(), r_o9)
        .await
        .map_err(|r_o8| {
            if digitext_db::f_1jn(&r_o8) {
                ApiError::conflict("mã dự án đã tồn tại")
            } else {
                r_o8.into()
            }
        })?;
    if r_o5.user.role == Role::Admin && r_o5.user.project_id.is_none() {
        users::f_2ec(&r_ob.pool, r_o5.user.id, Some(r_o6.id)).await?;
    }
    audit::append(
        &r_ob.pool,
        audit::Tp {
            actor_id: Some(r_o5.user.id),
            device_id: r_o5.claims.device_id,
            action: "project.create",
            target: Some(format!("project:{}", r_o6.id)),
            detail: Some(serde_json::json!({ "code" : r_o6.code })),
            ip: r_o5.ip(),
        },
    )
    .await?;
    let r_o7 = f_17r(&r_ob, r_o6).await?;
    Ok((StatusCode::CREATED, Json(r_o7)))
}
async fn f_20g(
    State(r_oe): State<AppState>,
    r_oc: Ts,
    Path(id): Path<Uuid>,
    Json(r_od): Json<Tgs>,
) -> Result<Json<ProjectDto>, ApiError> {
    r_oc.require_perm(Permission::ProjectUpdate)?;
    if !f_29a(&r_oe.pool, &r_oc.user).await?.allows_project(id) {
        return Err(ApiError::forbidden());
    }
    let r_of = projects::update(&r_oe.pool, id, r_od.name.as_deref(), r_od.is_active)
        .await?
        .ok_or_else(ApiError::not_found)?;
    audit::append(
        &r_oe.pool,
        audit::Tp {
            actor_id: Some(r_oc.user.id),
            device_id: r_oc.claims.device_id,
            action: "project.update",
            target: Some(format!("project:{id}")),
            detail: None,
            ip: r_oc.ip(),
        },
    )
    .await?;
    Ok(Json(f_17r(&r_oe, r_of).await?))
}
async fn f_28k(r_oi: &AppState, r_og: &Ts, id: Uuid) -> Result<Scope, ApiError> {
    r_og.require_perm(Permission::ProjectAssignAdmin)?;
    let r_oh = f_29a(&r_oi.pool, &r_og.user).await?;
    if !r_oh.allows_project(id) {
        return Err(ApiError::forbidden());
    }
    Ok(r_oh)
}
async fn f_28e(r_oj: &AppState, admin_id: Uuid, project_id: Uuid) -> Result<(), ApiError> {
    let r_ok = users::f_1eg(&r_oj.pool, admin_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if r_ok.role != Role::Admin {
        return Err(ApiError::bad_request(
            "người được gán phải có vai trò quản trị dự án",
        ));
    }
    // Quản trị dự án chỉ phụ trách MỘT dự án (15/10/2026).
    if let Some(khac) = projects::f_1ng(&r_oj.pool, admin_id).await?.into_iter().find(|p| p.id != project_id) {
        return Err(ApiError::conflict(format!(
            "quản trị dự án chỉ phụ trách một dự án — {} đang phụ trách {}, gỡ ở đó trước",
            r_ok.username, khac.code
        )));
    }
    Ok(())
}
async fn f_2eb(r_om: &AppState, admin_id: Uuid, r_ol: Uuid) -> Result<(), ApiError> {
    if let Some(admin) = users::f_1eg(&r_om.pool, admin_id).await? {
        if admin.project_id.is_none() {
            users::f_2ec(&r_om.pool, admin_id, Some(r_ol)).await?;
        }
    }
    Ok(())
}
async fn f_2ed(
    State(r_oq): State<AppState>,
    r_on: Ts,
    Path(id): Path<Uuid>,
    Json(r_oo): Json<Tlb>,
) -> Result<Json<ProjectDto>, ApiError> {
    let r_op = f_28k(&r_oq, &r_on, id).await?;
    if !matches!(r_op, Scope::All) {
        return Err(ApiError::forbidden());
    }
    let project = projects::get(&r_oq.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let mut admin_ids = r_oo.admin_ids.clone();
    admin_ids.sort();
    admin_ids.dedup();
    for admin_id in &admin_ids {
        f_28e(&r_oq, *admin_id, id).await?;
    }
    projects::f_2dy(&r_oq.pool, id, &admin_ids, Some(r_on.user.id)).await?;
    for admin_id in &admin_ids {
        f_2eb(&r_oq, *admin_id, id).await?;
    }
    audit::append(
        &r_oq.pool,
        audit::Tp {
            actor_id: Some(r_on.user.id),
            device_id: r_on.claims.device_id,
            action: "project.admins_set",
            target: Some(format!("project:{id}")),
            detail: Some(serde_json::json!({ "adminIds" : admin_ids })),
            ip: r_on.ip(),
        },
    )
    .await?;
    Ok(Json(f_17r(&r_oq, project).await?))
}
async fn f_qt(
    State(r_ot): State<AppState>,
    r_or: Ts,
    Path(id): Path<Uuid>,
    Json(r_os): Json<Te>,
) -> Result<Json<ProjectDto>, ApiError> {
    f_28k(&r_ot, &r_or, id).await?;
    let project = projects::get(&r_ot.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    f_28e(&r_ot, r_os.admin_id, id).await?;
    projects::f_qm(&r_ot.pool, id, r_os.admin_id, Some(r_or.user.id)).await?;
    f_2eb(&r_ot, r_os.admin_id, id).await?;
    audit::append(
        &r_ot.pool,
        audit::Tp {
            actor_id: Some(r_or.user.id),
            device_id: r_or.claims.device_id,
            action: "project.admin_added",
            target: Some(format!("project:{id}")),
            detail: Some(serde_json::json!({ "adminId" : r_os.admin_id })),
            ip: r_or.ip(),
        },
    )
    .await?;
    Ok(Json(f_17r(&r_ot, project).await?))
}
async fn f_276(
    State(r_oy): State<AppState>,
    r_ou: Ts,
    Path((id, admin_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ProjectDto>, ApiError> {
    let r_ox = f_28k(&r_oy, &r_ou, id).await?;
    let project = projects::get(&r_oy.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let r_ow = matches!(r_ox, Scope::All);
    if !r_ow && admin_id != r_ou.user.id {
        return Err(ApiError::forbidden());
    }
    if !r_ow {
        let r_ov = projects::f_re(&r_oy.pool, id).await?;
        if r_ov.len() <= 1 {
            return Err(
                ApiError::bad_request(
                    "bạn là admin duy nhất của dự án — nhờ super_admin gán người thay trước khi rút",
                ),
            );
        }
    }
    if !projects::f_275(&r_oy.pool, id, admin_id).await? {
        return Err(ApiError::not_found());
    }
    audit::append(
        &r_oy.pool,
        audit::Tp {
            actor_id: Some(r_ou.user.id),
            device_id: r_ou.claims.device_id,
            action: "project.admin_removed",
            target: Some(format!("project:{id}")),
            detail: Some(serde_json::json!({ "adminId" : admin_id })),
            ip: r_ou.ip(),
        },
    )
    .await?;
    Ok(Json(f_17r(&r_oy, project).await?))
}
async fn list_members(
    State(r_p0): State<AppState>,
    r_oz: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<UserDto>>, ApiError> {
    r_oz.require_perm(Permission::ProjectView)?;
    if !f_29a(&r_p0.pool, &r_oz.user).await?.allows_project(id) {
        return Err(ApiError::forbidden());
    }
    let members = projects::list_members(&r_p0.pool, id).await?;
    Ok(Json(members.into_iter().map(UserDto::from).collect()))
}
async fn add_member(
    State(r_p3): State<AppState>,
    r_p1: Ts,
    Path(id): Path<Uuid>,
    Json(r_p2): Json<Th>,
) -> Result<StatusCode, ApiError> {
    r_p1.require_perm(Permission::ProjectUpdate)?;
    if !f_29a(&r_p3.pool, &r_p1.user).await?.allows_project(id) {
        return Err(ApiError::forbidden());
    }
    users::f_1eg(&r_p3.pool, r_p2.user_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    projects::add_member(&r_p3.pool, id, r_p2.user_id, Some(r_p1.user.id)).await?;
    audit::append(
        &r_p3.pool,
        audit::Tp {
            actor_id: Some(r_p1.user.id),
            device_id: r_p1.claims.device_id,
            action: "project.member_add",
            target: Some(format!("project:{id}")),
            detail: Some(serde_json::json!({ "userId" : r_p2.user_id })),
            ip: r_p1.ip(),
        },
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tog {
    pub version: i32,
    pub published_by: Option<Uuid>,
    pub published_at: DateTime<Utc>,
    pub payload: Value,
}
async fn f_24r(
    State(r_p5): State<AppState>,
    r_p4: Ts,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    r_p4.require_perm(Permission::TreeAssign)?;
    if !f_29a(&r_p5.pool, &r_p4.user).await?.allows_project(id) {
        return Err(ApiError::forbidden());
    }
    projects::get(&r_p5.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let version = trees::upsert(&r_p5.pool, id, &payload, Some(r_p4.user.id)).await?;
    audit::append(
        &r_p5.pool,
        audit::Tp {
            actor_id: Some(r_p4.user.id),
            device_id: r_p4.claims.device_id,
            action: "project.tree_publish",
            target: Some(format!("project:{id}")),
            detail: Some(serde_json::json!({ "version" : version })),
            ip: r_p4.ip(),
        },
    )
    .await?;
    Ok(Json(serde_json::json!({ "version" : version })))
}
async fn f_24s(
    State(r_p9): State<AppState>,
    r_p6: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<Tog>, ApiError> {
    let r_p8 = f_29a(&r_p9.pool, &r_p6.user).await?;
    let r_p7 = r_p6.user.project_id == Some(id);
    if !r_p7 && !r_p8.allows_project(id) {
        return Err(ApiError::forbidden());
    }
    let tree = trees::get(&r_p9.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(Tog {
        version: tree.version,
        published_by: tree.published_by,
        published_at: tree.published_at,
        payload: tree.payload,
    }))
}
