// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::jwt::{Purpose, T2s};
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::StatusCode;
pub struct Ts {
    pub user: digitext_db::users::User,
    pub claims: T2s,
    pub overrides: digitext_core::PermissionOverrides,
    pub client_ip: Option<std::net::IpAddr>,
}
impl Ts {
    pub fn require(&self, r_3y: digitext_core::Perm) -> Result<(), ApiError> {
        if self.user.role.allows(r_3y) {
            Ok(())
        } else {
            Err(ApiError::forbidden())
        }
    }
    pub fn has_perm(&self, r_3z: digitext_core::Permission) -> bool {
        digitext_core::effective(self.user.role, &self.overrides, r_3z)
    }
    pub fn require_perm(&self, r_40: digitext_core::Permission) -> Result<(), ApiError> {
        if self.has_perm(r_40) {
            Ok(())
        } else {
            Err(ApiError::forbidden())
        }
    }
    pub fn ip(&self) -> Option<String> {
        self.client_ip.map(|r_41| r_41.to_string())
    }
    pub fn can_touch_project(&self, scope: &Scope, pid: uuid::Uuid) -> bool {
        f_249(self.user.role, self.user.project_id, scope, pid)
    }
    pub fn list_projects(&self, scope: Scope) -> Option<Vec<uuid::Uuid>> {
        f_247(self.user.role, self.user.project_id, scope)
    }
}
fn f_23t(role: digitext_core::Role) -> bool {
    !matches!(
        role,
        digitext_core::Role::Admin | digitext_core::Role::SuperAdmin
    )
}
pub(crate) fn f_249(
    role: digitext_core::Role,
    r_42: Option<uuid::Uuid>,
    scope: &Scope,
    pid: uuid::Uuid,
) -> bool {
    if scope.allows_project(pid) {
        return true;
    }
    f_23t(role) && r_42 == Some(pid)
}
pub(crate) fn f_247(
    role: digitext_core::Role,
    r_44: Option<uuid::Uuid>,
    scope: Scope,
) -> Option<Vec<uuid::Uuid>> {
    match scope {
        Scope::All => None,
        Scope::Projects(mut r_43) => {
            if let Some(own) = r_44.filter(|own| f_23t(role) && !r_43.contains(own)) {
                r_43.push(own);
            }
            Some(r_43)
        }
    }
}
#[derive(Debug, Clone)]
pub enum Scope {
    All,
    Projects(Vec<uuid::Uuid>),
}
impl Scope {
    pub fn allows_project(&self, pid: uuid::Uuid) -> bool {
        match self {
            Scope::All => true,
            Scope::Projects(r_45) => r_45.contains(&pid),
        }
    }
    pub fn allows_opt(&self, pid: Option<uuid::Uuid>) -> bool {
        match (self, pid) {
            (Scope::All, _) => true,
            (Scope::Projects(r_46), Some(r_47)) => r_46.contains(&r_47),
            (Scope::Projects(_), None) => false,
        }
    }
}
pub async fn f_29a(
    r_4a: &sqlx::PgPool,
    user: &digitext_db::users::User,
) -> Result<Scope, ApiError> {
    match user.role {
        digitext_core::Role::SuperAdmin => Ok(Scope::All),
        digitext_core::Role::Admin => {
            let r_48 = digitext_db::projects::f_1ng(r_4a, user.id)
                .await?
                .into_iter()
                .map(|r_49| r_49.id)
                .collect();
            Ok(Scope::Projects(r_48))
        }
        _ => Ok(Scope::Projects(Vec::new())),
    }
}
impl FromRequestParts<AppState> for Ts {
    type Rejection = ApiError;
    async fn from_request_parts(
        r_4e: &mut Parts,
        r_4g: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let r_4h = r_4e
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(ApiError::unauthorized)?;
        let claims = r_4g
            .keys
            .decode(r_4h)
            .map_err(|_| ApiError::unauthorized())?;
        if claims.purpose != Purpose::Access {
            return Err(ApiError::unauthorized());
        }
        let user = digitext_db::users::f_1eg(&r_4g.pool, claims.sub)
            .await?
            .ok_or_else(|| ApiError::revoked(StatusCode::UNAUTHORIZED))?;
        if !user.is_active || user.token_version != claims.tv {
            return Err(ApiError::revoked(StatusCode::UNAUTHORIZED));
        }
        if let Some(r_4d) = claims.device_id {
            if let Some(r_4c) = digitext_db::devices::get(&r_4g.pool, r_4d).await? {
                if r_4c.is_revoked() {
                    return Err(ApiError::device_revoked());
                }
            }
        }
        let overrides = digitext_db::permissions::f_1o5(&r_4g.pool, user.id).await?;
        let r_4f = r_4e
            .extensions
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|r_4b| r_4b.0.ip());
        let client_ip = crate::client_ip::client_ip(&r_4e.headers, r_4f, r_4g.behind_proxy);
        Ok(Ts {
            user,
            claims,
            overrides,
            client_ip,
        })
    }
}
