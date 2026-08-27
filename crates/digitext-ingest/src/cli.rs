// Copyright (c) 2026 DigiText. All rights reserved.
use crate::{f_1i5, f_1zc, FsObjectStore, MinioObjectStore, Tde};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;
fn minio_from_env() -> Option<Tde> {
    let r_2xr = |r_2xq: &str| std::env::var(r_2xq).ok().filter(|r_2xs| !r_2xs.is_empty());
    Some(Tde {
        endpoint: r_2xr("DIGITEXT_MINIO_ENDPOINT")?,
        bucket: r_2xr("DIGITEXT_MINIO_BUCKET")?,
        access_key: r_2xr("DIGITEXT_MINIO_ACCESS_KEY")?,
        secret_key: r_2xr("DIGITEXT_MINIO_SECRET_KEY")?,
        region: r_2xr("DIGITEXT_MINIO_REGION").unwrap_or_else(|| "us-east-1".into()),
    })
}
fn f_wj(prog: &str, r_2xt: &[String]) -> Result<String, String> {
    r_2xt
        .first()
        .cloned()
        .ok_or_else(|| format!("dùng: {prog} <thư-mục-bó>"))
}
pub fn cli_main(prog: &str, r_2xu: &[String]) -> ExitCode {
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
    match rt.block_on(run(prog, r_2xu)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
async fn run(prog: &str, r_2xv: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let r_2xw = f_wj(prog, r_2xv)?;
    let r_2xy = std::env::var("DIGITEXT_DATABASE_URL")
        .map_err(|_| "cần DIGITEXT_DATABASE_URL")?;
    let storage_dir = std::env::var("DIGITEXT_STORAGE_DIR")
        .unwrap_or_else(|_| "/var/lib/digitext/objects".into());
    let key_prefix = std::env::var("DIGITEXT_STORAGE_PREFIX").unwrap_or_else(|_| "ingest".into());
    let r_2xz = PathBuf::from(&r_2xw);
    let r_2y1 = f_1zc(&tokio::fs::read(r_2xz.join("bundle.json")).await?)?;
    let mut r_2y0 = HashMap::new();
    for e in &r_2y1.files {
        r_2y0.insert(e.path.clone(), tokio::fs::read(r_2xz.join(&e.path)).await?);
    }
    let r_2y2 = digitext_db::connect(&r_2xy).await?;
    let r_2y3: uuid::Uuid = sqlx::query_scalar("SELECT id FROM projects WHERE code = $1")
        .bind(&r_2y1.project_id)
        .fetch_optional(&r_2y2)
        .await?
        .ok_or_else(|| {
            format!(
                "không thấy dự án mã '{}'",
                r_2y1.project_id
            )
        })?;
    let (ids, dest) = match minio_from_env() {
        Some(r_2xx) => {
            let bucket = r_2xx.bucket.clone();
            let store = MinioObjectStore::new(&r_2xx)?;
            let ids = f_1i5(&r_2y2, r_2y3, &store, &r_2y1, &r_2y0, &key_prefix).await?;
            (ids, format!("MinIO {bucket}/{key_prefix}/…"))
        }
        None => {
            let store = FsObjectStore::new(&storage_dir);
            let ids = f_1i5(&r_2y2, r_2y3, &store, &r_2y1, &r_2y0, &key_prefix).await?;
            (ids, format!("{storage_dir}/{key_prefix}/…"))
        }
    };
    println!(
        "✅ ingest {} tài liệu (dự án {}) → {dest}",
        ids.len(),
        r_2y1.project_id
    );
    Ok(())
}
