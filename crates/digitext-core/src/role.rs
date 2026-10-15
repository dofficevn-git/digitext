// Copyright (c) 2026 DigiText. All rights reserved.
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    SuperAdmin,
    /// Quản lý (15/10/2026): xem và điều hành MỌI dự án như quản trị, nhưng không được xoá.
    Manager,
    Admin,
    User,
    Customer,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Perm {
    Read,
    Operate,
    Admin,
    Super,
}
impl Role {
    pub const ALL: [Role; 5] = [Role::SuperAdmin, Role::Manager, Role::Admin, Role::User, Role::Customer];
    pub fn max_perm(self) -> Perm {
        match self {
            Role::SuperAdmin => Perm::Super,
            Role::Manager => Perm::Admin,
            Role::Admin => Perm::Admin,
            Role::User => Perm::Operate,
            Role::Customer => Perm::Read,
        }
    }
    pub fn allows(self, perm: Perm) -> bool {
        self.max_perm() >= perm
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Role::SuperAdmin => "super_admin",
            Role::Manager => "manager",
            Role::Admin => "admin",
            Role::User => "user",
            Role::Customer => "customer",
        }
    }
}
impl fmt::Display for Role {
    fn fmt(&self, r_1zq: &mut fmt::Formatter<'_>) -> fmt::Result {
        r_1zq.write_str(self.as_str())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("vai trò không hợp lệ: {0:?}")]
pub struct Tgq(pub String);
impl FromStr for Role {
    type Err = Tgq;
    fn from_str(r_1zs: &str) -> Result<Self, Self::Err> {
        match r_1zs {
            "super_admin" => Ok(Role::SuperAdmin),
            "manager" => Ok(Role::Manager),
            "admin" => Ok(Role::Admin),
            "user" => Ok(Role::User),
            "customer" => Ok(Role::Customer),
            r_1zr => Err(Tgq(r_1zr.to_owned())),
        }
    }
}
impl TryFrom<String> for Role {
    type Error = Tgq;
    fn try_from(r_1zt: String) -> Result<Self, Self::Error> {
        r_1zt.parse()
    }
}
