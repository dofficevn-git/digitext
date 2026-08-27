// Copyright (c) 2026 DigiText. All rights reserved.
use crate::metrics::Metrics;
use crate::readiness::{Readiness, Tol};
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
pub struct AppState<P: Tol> {
    pub metrics: Arc<Metrics>,
    pub readiness: Arc<Readiness<P>>,
}
impl<P: Tol> Clone for AppState<P> {
    fn clone(&self) -> Self {
        AppState {
            metrics: Arc::clone(&self.metrics),
            readiness: Arc::clone(&self.readiness),
        }
    }
}
pub fn router<P: Tol + 'static>(state: AppState<P>) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz::<P>))
        .with_state(state)
}
pub async fn serve<P: Tol + 'static>(r_3f3: SocketAddr, state: AppState<P>) -> std::io::Result<()> {
    let r_3f4 = tokio::net::TcpListener::bind(r_3f3).await?;
    axum::serve(r_3f4, router(state)).await
}
async fn healthz() -> &'static str {
    "ok"
}
async fn readyz<P: Tol + 'static>(State(r_3f7): State<AppState<P>>) -> Response {
    let r_3f6 = r_3f7.readiness.report().await;
    let r_3f5 = if r_3f6.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        r_3f5,
        [(header::CONTENT_TYPE, "application/json")],
        r_3f6.to_json(),
    )
        .into_response()
}
