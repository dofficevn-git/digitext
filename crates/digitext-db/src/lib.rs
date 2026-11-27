// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
pub mod access;
pub mod audit;
pub mod bao_cao;
pub mod bao_quan;
pub mod changes;
pub mod chinh_ly;
pub mod chuyen_doi;
pub mod co_quan;
pub mod co_quan_nop;
pub mod danh_muc_ho_so;
pub mod dashboard;
pub mod devices;
pub mod disposal;
pub mod documents;
pub mod engine_versions;
pub mod info;
pub mod kie_examples;
pub mod ma_tt05;
pub mod nop_luu;
pub mod notifications;
pub mod ocr_fields;
pub mod ocr_jobs;
pub mod ocr_results;
pub mod ops;
pub mod packages;
pub mod permissions;
pub mod phong_lich_su;
pub mod pool;
pub mod progress;
pub mod project_purge;
pub mod projects;
pub mod records;
pub mod review_queue;
pub mod roles;
pub mod sign_sessions;
pub mod stats;
pub mod submissions;
pub mod tokens;
pub mod trees;
pub mod upload_batches;
pub mod user_phong;
pub mod users;
pub mod webhooks;
pub mod work_packages;
pub mod workdays;
pub use pool::{connect, f_11d};
pub fn f_1jn(r_25w: &sqlx::Error) -> bool {
    matches!(r_25w, sqlx::Error::Database(db) if db.code().as_deref() == Some("23505"))
}
