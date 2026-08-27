// Copyright (c) 2026 DigiText. All rights reserved.
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    #[default]
    Pending,
    Approved,
    Rejected,
}
impl ReviewStatus {
    pub const ALL: [ReviewStatus; 3] = [
        ReviewStatus::Pending,
        ReviewStatus::Approved,
        ReviewStatus::Rejected,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            ReviewStatus::Pending => "pending",
            ReviewStatus::Approved => "approved",
            ReviewStatus::Rejected => "rejected",
        }
    }
    pub fn has_trace(self) -> bool {
        self != ReviewStatus::Pending
    }
}
impl fmt::Display for ReviewStatus {
    fn fmt(&self, r_1zj: &mut fmt::Formatter<'_>) -> fmt::Result {
        r_1zj.write_str(self.as_str())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("dHLhuqFuZyB0aMOhaSBkdXnhu4d0IGtow7RuZyBo4bujcCBs4buHOiA={0:?}")]
pub struct Tgp(pub String);
impl FromStr for ReviewStatus {
    type Err = Tgp;
    fn from_str(r_1zl: &str) -> Result<Self, Self::Err> {
        ReviewStatus::ALL
            .into_iter()
            .find(|r_1zk| r_1zk.as_str() == r_1zl)
            .ok_or_else(|| Tgp(r_1zl.to_owned()))
    }
}
impl TryFrom<String> for ReviewStatus {
    type Error = Tgp;
    fn try_from(r_1zm: String) -> Result<Self, Self::Error> {
        r_1zm.parse()
    }
}
