// Copyright (c) 2026 DigiText. All rights reserved.
use digitext_core::{Permission, Role};
use serde::Serialize;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionMode {
    Online,
    Offline,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tr {
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub project_id: Option<String>,
    pub project_code: Option<String>,
    pub project_name: Option<String>,
    pub mode: SessionMode,
    pub offline_expiry: String,
    pub locked: bool,
    pub permissions: Vec<Permission>,
}
impl Tr {
    pub fn who(&self) -> (String, String) {
        (self.user_id.clone(), self.username.clone())
    }
}
