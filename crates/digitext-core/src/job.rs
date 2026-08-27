// Copyright (c) 2026 DigiText. All rights reserved.
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    Ocr,
    ReOcr,
    Audit,
    MakePdf2,
    SignWait,
    ExtractFields,
}
impl JobKind {
    pub const ALL: [JobKind; 6] = [
        JobKind::Ocr,
        JobKind::ReOcr,
        JobKind::Audit,
        JobKind::MakePdf2,
        JobKind::SignWait,
        JobKind::ExtractFields,
    ];
    pub const WORKER_KINDS: [JobKind; 4] = [
        JobKind::Ocr,
        JobKind::ReOcr,
        JobKind::Audit,
        JobKind::MakePdf2,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            JobKind::Ocr => "ocr",
            JobKind::ReOcr => "re_ocr",
            JobKind::Audit => "audit",
            JobKind::MakePdf2 => "make_pdf2",
            JobKind::SignWait => "sign_wait",
            JobKind::ExtractFields => "extract_fields",
        }
    }
}
impl fmt::Display for JobKind {
    fn fmt(&self, r_1yp: &mut fmt::Formatter<'_>) -> fmt::Result {
        r_1yp.write_str(self.as_str())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("loại job không hợp lệ: {0:?}")]
pub struct Tgm(pub String);
impl FromStr for JobKind {
    type Err = Tgm;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JobKind::ALL
            .into_iter()
            .find(|k| k.as_str() == s)
            .ok_or_else(|| Tgm(s.to_owned()))
    }
}
impl TryFrom<String> for JobKind {
    type Error = Tgm;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Done,
    FailedTransient,
    PageFailed,
    Cancelled,
}
impl JobStatus {
    pub const ALL: [JobStatus; 6] = [
        JobStatus::Queued,
        JobStatus::Running,
        JobStatus::Done,
        JobStatus::FailedTransient,
        JobStatus::PageFailed,
        JobStatus::Cancelled,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            JobStatus::Queued => "queued",
            JobStatus::Running => "running",
            JobStatus::Done => "done",
            JobStatus::FailedTransient => "failed_transient",
            JobStatus::PageFailed => "page_failed",
            JobStatus::Cancelled => "cancelled",
        }
    }
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            JobStatus::Done | JobStatus::PageFailed | JobStatus::Cancelled
        )
    }
    pub fn can_transition_to(self, r_1yq: JobStatus) -> bool {
        use JobStatus::*;
        matches!(
            (self, r_1yq),
            (Queued, Running)
                | (Queued, Cancelled)
                | (Running, Done)
                | (Running, FailedTransient)
                | (Running, PageFailed)
                | (Running, Cancelled)
                | (FailedTransient, Queued)
                | (FailedTransient, Cancelled)
        )
    }
}
impl fmt::Display for JobStatus {
    fn fmt(&self, r_1yr: &mut fmt::Formatter<'_>) -> fmt::Result {
        r_1yr.write_str(self.as_str())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("trạng thái job không hợp lệ: {0:?}")]
pub struct Tgn(pub String);
impl FromStr for JobStatus {
    type Err = Tgn;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JobStatus::ALL
            .into_iter()
            .find(|r_1ys| r_1ys.as_str() == s)
            .ok_or_else(|| Tgn(s.to_owned()))
    }
}
impl TryFrom<String> for JobStatus {
    type Error = Tgn;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}
