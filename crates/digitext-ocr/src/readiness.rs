// Copyright (c) 2026 DigiText. All rights reserved.
use crate::health::{Check, HealthReport};
use std::future::Future;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toj {
    pub server_ready: bool,
    pub models: Vec<(String, bool)>,
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("không dò được Triton: {0}")]
pub struct Thj(pub String);
pub trait Tol: Send + Sync {
    fn probe(&self, required_models: &[String]) -> impl Future<Output = Result<Toj, Thj>> + Send;
}
#[derive(Debug, Clone)]
pub struct Readiness<P: Tol> {
    probe: P,
    required_models: Vec<String>,
}
impl<P: Tol> Readiness<P> {
    pub fn new(probe: P, required_models: Vec<String>) -> Self {
        Readiness {
            probe,
            required_models,
        }
    }
    pub fn default_models() -> Vec<String> {
        [
            "ocr_pipeline",
            "det_dbnet",
            "cls_angle",
            "rec_svtr_vi",
            "rec_vietocr",
        ]
        .iter()
        .map(|r_3d3| (*r_3d3).to_owned())
        .collect()
    }
    pub async fn report(&self) -> HealthReport {
        let mut r_3d4 = vec![Check::ok("config", true)];
        match self.probe.probe(&self.required_models).await {
            Ok(r_3d6) => {
                r_3d4.push(if r_3d6.server_ready {
                    Check::ok("triton_server", true)
                } else {
                    Check::fail("triton_server", true, "server chưa ready")
                });
                let r_3d8: Vec<&str> = r_3d6
                    .models
                    .iter()
                    .filter(|(_, ready)| !ready)
                    .map(|(r_3d7, _)| r_3d7.as_str())
                    .collect();
                r_3d4.push(if r_3d8.is_empty() {
                    Check::ok("triton_models", true)
                } else {
                    Check::fail(
                        "triton_models",
                        true,
                        format!("model chưa ready: {}", r_3d8.join(", ")),
                    )
                });
            }
            Err(r_3d5) => r_3d4.push(Check::fail("triton_server", true, r_3d5.to_string())),
        }
        HealthReport::aggregate(r_3d4)
    }
}
