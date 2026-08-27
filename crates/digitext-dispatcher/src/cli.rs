// Copyright (c) 2026 DigiText. All rights reserved.
use crate::reprioritize;
use sqlx::postgres::PgListener;
use std::process::ExitCode;
use std::time::Duration;
fn interval_ms(r_2w6: Option<String>) -> u64 {
    r_2w6.and_then(|r_2w7| r_2w7.parse().ok()).unwrap_or(1000)
}
pub fn cli_main() -> ExitCode {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("không tạo được tokio runtime: {e}");
            return ExitCode::FAILURE;
        }
    };
    match rt.block_on(run()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let r_2w8 = std::env::var("DIGITEXT_DATABASE_URL")
        .map_err(|_| "cần DIGITEXT_DATABASE_URL")?;
    let interval_ms = interval_ms(std::env::var("DIGITEXT_DISPATCH_INTERVAL_MS").ok());
    let r_2wb = digitext_db::connect(&r_2w8).await?;
    let mut r_2wa = PgListener::connect(&r_2w8).await?;
    r_2wa.listen("ocr_jobs").await?;
    let mut r_2wc = tokio::time::interval(Duration::from_millis(interval_ms.max(50)));
    tracing::info!("dispatcher fair-share: nhịp {interval_ms}ms + LISTEN ocr_jobs");
    loop {
        tokio::select! {
            _ = r_2wc.tick() => {} r = r_2wa.recv() => { r ?; }
        }
        match reprioritize(&r_2wb).await {
            Ok(r_2w9) if r_2w9 > 0 => {
                tracing::debug!(
                    f_17t = r_2w9,
                    "đã cập nhật priority fair-share"
                )
            }
            Ok(_) => {}
            Err(e) => tracing::error!("reprioritize lỗi: {e}"),
        }
    }
}
