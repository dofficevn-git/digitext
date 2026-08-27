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
            .ok_or_else(|| AppError::new("unauthorized", "Chưa đăng nhập."))?;
        if r_ke.locked {
            return Err(
                AppError::new(
                    "locked",
                    "Màn hình đang khoá — mở khoá bằng mật khẩu để tiếp tục.",
                ),
            );
        }
        let r_kc = DateTime::parse_from_rfc3339(&r_ke.offline_expiry)
            .map_err(|_| {
                AppError::new(
                    "auth_cache_corrupt",
                    "Hạn offline trong phiên không hợp lệ.",
                )
            })?
            .with_timezone(&Utc);
        if r_kd >= r_kc {
            return Err(
                AppError::new(
                    "offline_expired",
                    "Quá 30 ngày chưa xác thực online — kết nối mạng và đăng nhập lại.",
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
        "Tài khoản của bạn không đủ quyền thực hiện thao tác này.",
    )
}
