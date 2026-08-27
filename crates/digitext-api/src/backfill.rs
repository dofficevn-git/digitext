// Copyright (c) 2026 DigiText. All rights reserved.
use digitext_storage::ObjectStore;
use sqlx::{PgPool, Row};
use std::path::{Path, PathBuf};
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BackfillReport {
    pub total: u64,
    pub uploaded: u64,
    pub skipped: u64,
    pub missing: u64,
    pub errors: u64,
}
pub fn f_28w(r_c: &Path, storage_path: &str) -> Option<PathBuf> {
    let r_e = r_c.join(storage_path);
    if r_e.is_file() {
        return Some(r_e);
    }
    let r_d = PathBuf::from(storage_path);
    if r_d.is_file() {
        return Some(r_d);
    }
    None
}
pub async fn backfill_source_pdfs(
    r_j: &PgPool,
    r_o: &dyn ObjectStore,
    r_g: &Path,
    r_n: bool,
) -> anyhow::Result<BackfillReport> {
    let r_m = sqlx::query("SELECT id, storage_path FROM documents ORDER BY uploaded_at")
        .fetch_all(r_j)
        .await?;
    let mut r_l = BackfillReport::default();
    for r_k in r_m {
        r_l.total += 1;
        let id: uuid::Uuid = r_k.get("id");
        let r_h: String = r_k.get("storage_path");
        if r_h.trim().is_empty() {
            r_l.missing += 1;
            continue;
        }
        if r_n {
            match r_o.get(&r_h).await {
                Ok(Some(_)) => {
                    r_l.skipped += 1;
                    continue;
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(
                        document = % id, r_h = % r_h, error = % e,
                        "kiểm object trên kho lỗi — vẫn thử PUT"
                    );
                }
            }
        }
        let Some(r_i) = f_28w(r_g, &r_h) else {
            r_l.missing += 1;
            tracing::warn!(
                document = % id, r_h = % r_h,
                "không thấy file PDF local — bỏ qua"
            );
            continue;
        };
        match tokio::fs::read(&r_i).await {
            Ok(r_f) => match r_o.put(&r_h, &r_f).await {
                Ok(()) => {
                    r_l.uploaded += 1;
                    tracing::info!(
                        document = % id, r_h = % r_h, r_f = r_f.len(),
                        "backfill PUT PDF nguồn"
                    );
                }
                Err(e) => {
                    r_l.errors += 1;
                    tracing::error!(
                        document = % id, r_h = % r_h, error = % e,
                        "backfill PUT lỗi"
                    );
                }
            },
            Err(e) => {
                r_l.errors += 1;
                tracing::error!(
                    document = % id, r_h = % r_h, error = % e,
                    "backfill đọc file lỗi"
                );
            }
        }
    }
    Ok(r_l)
}
