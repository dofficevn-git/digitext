// Copyright (c) 2026 DigiText. All rights reserved.
use serde::Serialize;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Health {
    Ok,
    Degraded,
    Down,
}
#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub name: &'static str,
    pub critical: bool,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
impl Check {
    pub fn ok(name: &'static str, critical: bool) -> Self {
        Check {
            name,
            critical,
            ok: true,
            detail: None,
        }
    }
    pub fn fail(name: &'static str, critical: bool, detail: impl Into<String>) -> Self {
        Check {
            name,
            critical,
            ok: false,
            detail: Some(detail.into()),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub status: Health,
    pub checks: Vec<Check>,
}
impl HealthReport {
    pub fn aggregate(checks: Vec<Check>) -> Self {
        let r_39o = checks.iter().any(|c| c.critical && !c.ok);
        let r_39p = checks.iter().any(|c| !c.ok);
        let status = if r_39o {
            Health::Down
        } else if r_39p {
            Health::Degraded
        } else {
            Health::Ok
        };
        HealthReport { status, checks }
    }
    pub fn is_ok(&self) -> bool {
        self.status == Health::Ok
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_owned())
    }
}
