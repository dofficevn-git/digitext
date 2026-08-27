// Copyright (c) 2026 DigiText. All rights reserved.
use std::time::Duration;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Transient,
    Permanent,
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WorkerError {
    #[error("VHJpdG9uIGtow7RuZyBz4bq1biBzw6BuZzog{0}")]
    TritonUnavailable(String),
    #[error("R1BVIGjhur90IGLhu5kgbmjhu5sgKE9PTSk6IA=={0}")]
    GpuOom(String),
    #[error("Y8ahIHPhu58gZOG7ryBsaeG7h3Uga2jDtG5nIHPhurVuIHPDoG5nOiA={0}")]
    DbUnavailable(String),
    #[error("aOG6v3QgdGjhu51pIGdpYW4gY2jhu506IA=={0}")]
    Timeout(String),
    #[error("a2hvIMSR4buRaSB0xrDhu6NuZyBraMO0bmcgc+G6tW4gc8Ogbmc6IA=={0}")]
    StorageUnavailable(String),
    #[error("ZOG7ryBsaeG7h3UgxJHhuqd1IHbDoG8gaOG7j25nOiA={0}")]
    CorruptInput(String),
    #[error("xJHhu4tuaCBk4bqhbmcga2jDtG5nIGjhu5cgdHLhu6M6IA=={0}")]
    UnsupportedFormat(String),
    #[error("Y+G6pXUgaMOsbmggc2FpOiA={0}")]
    ConfigInvalid(String),
}
impl WorkerError {
    pub fn severity(&self) -> Severity {
        use WorkerError::*;
        match self {
            TritonUnavailable(_)
            | GpuOom(_)
            | DbUnavailable(_)
            | Timeout(_)
            | StorageUnavailable(_) => Severity::Transient,
            CorruptInput(_) | UnsupportedFormat(_) | ConfigInvalid(_) => Severity::Permanent,
        }
    }
    pub fn is_transient(&self) -> bool {
        self.severity() == Severity::Transient
    }
    pub fn is_permanent(&self) -> bool {
        self.severity() == Severity::Permanent
    }
    pub fn is_oom(&self) -> bool {
        matches!(self, WorkerError::GpuOom(_))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDecision {
    Retry { after: Duration },
    GiveUp,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}
impl RetryPolicy {
    pub fn new(max_retries: u32, base_delay: Duration, max_delay: Duration) -> Self {
        RetryPolicy {
            max_retries,
            base_delay,
            max_delay,
        }
    }
    pub fn backoff(&self, attempt: u32) -> Duration {
        let r_35a = self.base_delay.as_millis() as u64;
        let r_35d = self.max_delay.as_millis() as u64;
        let r_35c = 1u64.checked_shl(attempt).unwrap_or(u64::MAX);
        let r_35b = r_35a.saturating_mul(r_35c).min(r_35d);
        Duration::from_millis(r_35b)
    }
    pub fn decide(&self, r_35e: &WorkerError, attempt: u32) -> RetryDecision {
        if r_35e.is_permanent() || attempt >= self.max_retries {
            RetryDecision::GiveUp
        } else {
            RetryDecision::Retry {
                after: self.backoff(attempt),
            }
        }
    }
}
