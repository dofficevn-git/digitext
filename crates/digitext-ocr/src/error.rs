// Copyright (c) 2026 DigiText. All rights reserved.
use std::time::Duration;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Transient,
    Permanent,
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WorkerError {
    #[error("Triton không sẵn sàng: {0}")]
    TritonUnavailable(String),
    #[error("GPU hết bộ nhớ (OOM): {0}")]
    GpuOom(String),
    #[error("cơ sở dữ liệu không sẵn sàng: {0}")]
    DbUnavailable(String),
    #[error("hết thời gian chờ: {0}")]
    Timeout(String),
    #[error("kho đối tượng không sẵn sàng: {0}")]
    StorageUnavailable(String),
    #[error("dữ liệu đầu vào hỏng: {0}")]
    CorruptInput(String),
    #[error("định dạng không hỗ trợ: {0}")]
    UnsupportedFormat(String),
    #[error("cấu hình sai: {0}")]
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
#[cfg(test)]
mod tests {
    use super::*;
    fn policy() -> RetryPolicy {
        RetryPolicy::new(5, Duration::from_millis(200), Duration::from_secs(30))
    }
    #[test]
    fn backoff_doubles_then_caps() {
        let p = policy();
        assert_eq!(p.backoff(0), Duration::from_millis(200));
        assert_eq!(p.backoff(1), Duration::from_millis(400));
        assert_eq!(p.backoff(3), Duration::from_millis(1600));
        assert_eq!(p.backoff(10), Duration::from_secs(30));
        assert_eq!(p.backoff(63), Duration::from_secs(30));
        assert_eq!(p.backoff(200), Duration::from_secs(30));
    }
    #[test]
    fn decide_gives_up_on_permanent_or_exhausted() {
        let p = policy();
        let transient = WorkerError::Timeout("x".into());
        let permanent = WorkerError::CorruptInput("x".into());
        assert_eq!(p.decide(&permanent, 0), RetryDecision::GiveUp);
        assert_eq!(p.decide(&transient, 5), RetryDecision::GiveUp);
        assert_eq!(
            p.decide(&transient, 2),
            RetryDecision::Retry {
                after: Duration::from_millis(800)
            }
        );
    }
    #[test]
    fn severity_partition() {
        assert!(WorkerError::TritonUnavailable("x".into()).is_transient());
        assert!(WorkerError::GpuOom("x".into()).is_oom());
        assert!(WorkerError::ConfigInvalid("x".into()).is_permanent());
    }
}
