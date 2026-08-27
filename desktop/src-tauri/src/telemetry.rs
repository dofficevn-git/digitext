// Copyright (c) 2026 DigiText. All rights reserved.
use crate::config;
use std::path::Path;
use std::sync::OnceLock;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};
static C_8I: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();
pub fn init(r_32x: &Path) {
    let r_32v = EnvFilter::try_from_env(config::C_AK).unwrap_or_else(|_| EnvFilter::new("info"));
    let r_32u = match std::fs::create_dir_all(r_32x) {
        Ok(_) => {
            let r_32t = tracing_appender::rolling::daily(r_32x, "digitext.log");
            let (r_32y, r_32w) = tracing_appender::non_blocking(r_32t);
            let _ = C_8I.set(r_32w);
            Some(
                fmt::layer()
                    .with_ansi(false)
                    .with_target(false)
                    .with_writer(r_32y),
            )
        }
        Err(_) => None,
    };
    let _ = tracing_subscriber::registry()
        .with(r_32v)
        .with(fmt::layer().with_target(false))
        .with(r_32u)
        .try_init();
}
