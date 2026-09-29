// Copyright (c) 2026 DigiText. All rights reserved.
use crate::jwt::JwtKeys;
use crate::limiter::LoginLimiter;
use axum::routing::get;
use axum::Router;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;
use tower_http::compression::Predicate as _;
use tower_http::trace::TraceLayer;
pub type Tpc = Arc<Mutex<HashMap<String, Arc<AsyncMutex<()>>>>>;
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub keys: Arc<JwtKeys>,
    pub limiter: Arc<LoginLimiter>,
    pub data_dir: Arc<std::path::PathBuf>,
    /// Thư mục phát bộ cài tự cập nhật (DIGITEXT_UPDATES_DIR) — xem routes::updates.
    pub updates_dir: Arc<std::path::PathBuf>,
    pub upload_locks: Tpc,
    pub behind_proxy: bool,
    pub store: Option<Arc<dyn digitext_storage::ObjectStore>>,
    pub source_store: Option<Arc<dyn digitext_storage::ObjectStore>>,
    pub tsa_url: Option<Arc<str>>,
    pub trust_roots: Option<Arc<digitext_sign_verify::chain::TrustStore>>,
    pub tsa_trust_roots: Option<Arc<digitext_sign_verify::chain::TrustStore>>,
    pub pades_lt: bool,
    pub pades_lta: bool,
    pub sign_visible: bool,
    pub sign_stamp: bool,
    pub change_prune: Arc<Mutex<Option<std::time::Instant>>>,
}
impl AppState {
    pub fn new(pool: PgPool, keys: Arc<JwtKeys>, data_dir: impl Into<std::path::PathBuf>) -> Self {
        AppState {
            pool,
            keys,
            limiter: Arc::new(LoginLimiter::default()),
            data_dir: Arc::new(data_dir.into()),
            updates_dir: Arc::new(std::path::PathBuf::from("updates")),
            upload_locks: Arc::new(Mutex::new(HashMap::new())),
            behind_proxy: false,
            store: None,
            source_store: None,
            tsa_url: None,
            trust_roots: None,
            tsa_trust_roots: None,
            pades_lt: false,
            pades_lta: false,
            sign_visible: true,
            sign_stamp: false,
            change_prune: Arc::new(Mutex::new(None)),
        }
    }
    pub fn with_updates_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.updates_dir = Arc::new(dir.into());
        self
    }
    pub fn with_behind_proxy(mut self, r_0: bool) -> Self {
        self.behind_proxy = r_0;
        self
    }
    pub fn with_store(mut self, store: Arc<dyn digitext_storage::ObjectStore>) -> Self {
        self.store = Some(store);
        self
    }
    pub fn with_source_store(mut self, store: Arc<dyn digitext_storage::ObjectStore>) -> Self {
        self.source_store = Some(store);
        self
    }
    pub fn with_tsa_url(mut self, r_1: impl Into<Arc<str>>) -> Self {
        self.tsa_url = Some(r_1.into());
        self
    }
    pub fn with_trust_roots(mut self, r_2: digitext_sign_verify::chain::TrustStore) -> Self {
        self.trust_roots = Some(Arc::new(r_2));
        self
    }
    pub fn with_tsa_trust_roots(mut self, r_3: digitext_sign_verify::chain::TrustStore) -> Self {
        self.tsa_trust_roots = Some(Arc::new(r_3));
        self
    }
    pub fn with_sign_appearance(mut self, r_5: bool, r_4: bool) -> Self {
        self.sign_visible = r_5;
        self.sign_stamp = r_4;
        self
    }
    pub fn with_pades_lt(mut self, r_6: bool) -> Self {
        self.pades_lt = r_6;
        self
    }
    pub fn with_pades_lta(mut self, r_7: bool) -> Self {
        self.pades_lta = r_7;
        self
    }
    pub fn upload_lock(&self, project_id: uuid::Uuid, blake3: &str) -> Arc<AsyncMutex<()>> {
        let r_8 = format!("{project_id}:{blake3}");
        let mut r_9 = self.upload_locks.lock().expect("upload_locks mutex");
        r_9.retain(|_, r_a| Arc::strong_count(r_a) > 1);
        r_9.entry(r_8).or_default().clone()
    }
}
pub fn router(r_b: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .merge(crate::routes::updates::router())
        .nest("/api/v1", f_rv())
        // Nén gzip JSON (danh sách 30.000 tài liệu ≈ 12 MB → ~1 MB) khi client xin
        // (Accept-Encoding). Không nén PDF/zip/bộ cài: đã nén sẵn, chỉ tốn CPU.
        .layer(
            tower_http::compression::CompressionLayer::new().gzip(true).compress_when(
                tower_http::compression::predicate::DefaultPredicate::new()
                    .and(tower_http::compression::predicate::NotForContentType::const_new("application/pdf"))
                    .and(tower_http::compression::predicate::NotForContentType::const_new("application/zip"))
                    .and(tower_http::compression::predicate::NotForContentType::const_new("application/octet-stream")),
            ),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(r_b)
}
fn f_rv() -> Router<AppState> {
    crate::routes::auth::router()
        .merge(crate::routes::users::router())
        .merge(crate::routes::projects::router())
        .merge(crate::routes::devices::router())
        .merge(crate::routes::stats::router())
        .merge(crate::routes::progress::router())
        .merge(crate::routes::work::router())
        .merge(crate::routes::documents::router())
        .merge(crate::routes::ocr_fields::router())
        .merge(crate::routes::packages::router())
        .merge(crate::routes::submissions::router())
        .merge(crate::routes::notifications::router())
        .merge(crate::routes::records::router())
        .merge(crate::routes::sign::router())
        .merge(crate::routes::access::router())
        .merge(crate::routes::disposal::router())
        .merge(crate::routes::info::router())
        .merge(crate::routes::ops::router())
        .merge(crate::routes::upload_batches::router())
        .merge(crate::routes::changes::router())
}
