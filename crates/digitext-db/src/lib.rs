// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
pub mod access;
pub mod audit;
pub mod changes;
pub mod devices;
pub mod disposal;
pub mod documents;
pub mod engine_versions;
pub mod info;
pub mod notifications;
pub mod ocr_fields;
pub mod ocr_jobs;
pub mod ocr_results;
pub mod ops;
pub mod packages;
pub mod permissions;
pub mod pool;
pub mod projects;
pub mod records;
pub mod review_queue;
pub mod sign_sessions;
pub mod stats;
pub mod submissions;
pub mod tokens;
pub mod trees;
pub mod upload_batches;
pub mod users;
pub mod workdays;
pub use pool::{connect, f_11d};
pub fn f_1jn(r_25w: &sqlx::Error) -> bool {
    matches!(r_25w, sqlx::Error::Database(db) if db.code().as_deref() == Some("23505"))
}
