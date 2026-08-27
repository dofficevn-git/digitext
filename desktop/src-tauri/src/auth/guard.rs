// Copyright (c) 2026 DigiText. All rights reserved.
use super::session::Tr;
use super::AuthState;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use digitext_core::{Perm, Permission};
impl AuthState {
    pub fn require(&self, perm: Perm) -> Result<Tr, AppError> {
        self.require_at(perm, Utc::now())
    }
    pub fn require_at(&self, perm: Perm, r_ka: DateTime<Utc>) -> Result<Tr, AppError> {
        let r_kb = self.active_session_at(r_ka)?;
        if !r_kb.role.allows(perm) {
            return Err(forbidden());
        }
        Ok(r_kb)
    }
    fn active_session_at(&self, r_kd: DateTime<Utc>) -> Result<Tr, AppError> {
        let r_ke = self
            .snapshot()
            .ok_or_else(|| AppError::new("unauthorized", "Q2jGsGEgxJHEg25nIG5o4bqtcC4="))?;
        if r_ke.locked {
            return Err(
                AppError::new(
                    "locked",
                    "TcOgbiBow6xuaCDEkWFuZyBraG/DoSDigJQgbeG7nyBraG/DoSBi4bqxbmcgbeG6rXQga2jhuql1IMSR4buDIHRp4bq/cCB04bulYy4=",
                ),
            );
        }
        let r_kc = DateTime::parse_from_rfc3339(&r_ke.offline_expiry)
            .map_err(|_| {
                AppError::new(
                    "auth_cache_corrupt",
                    "SOG6oW4gb2ZmbGluZSB0cm9uZyBwaGnDqm4ga2jDtG5nIGjhu6NwIGzhu4cu",
                )
            })?
            .with_timezone(&Utc);
        if r_kd >= r_kc {
            return Err(
                AppError::new(
                    "offline_expired",
                    "UXXDoSAzMCBuZ8OgeSBjaMawYSB4w6FjIHRo4buxYyBvbmxpbmUg4oCUIGvhur90IG7hu5FpIG3huqFuZyB2w6AgxJHEg25nIG5o4bqtcCBs4bqhaS4=",
                ),
            );
        }
        Ok(r_ke)
    }
    pub fn require_perm(&self, perm: Permission) -> Result<Tr, AppError> {
        self.require_perm_at(perm, Utc::now())
    }
    pub fn require_perm_at(&self, perm: Permission, r_kf: DateTime<Utc>) -> Result<Tr, AppError> {
        let r_kg = self.active_session_at(r_kf)?;
        if !r_kg.permissions.contains(&perm) {
            return Err(forbidden());
        }
        Ok(r_kg)
    }
    pub fn require_perm_str(&self, perm: Permission) -> Result<Tr, String> {
        self.require_perm(perm).map_err(|r_kh| r_kh.to_string())
    }
}
fn forbidden() -> AppError {
    AppError::new(
        "forbidden",
        "VMOgaSBraG/huqNuIGPhu6dhIGLhuqFuIGtow7RuZyDEkeG7pyBxdXnhu4FuIHRo4buxYyBoaeG7h24gdGhhbyB0w6FjIG7DoHku",
    )
}
