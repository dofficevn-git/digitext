// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
const C_G3: Duration = Duration::from_secs(60);
pub fn router() -> Router<AppState> {
    Router::new().route("/changes", get(changes))
}
#[derive(Debug, Deserialize)]
pub struct T2p {
    pub since: Option<i64>,
}
#[derive(Debug, Serialize)]
pub struct T2o {
    pub seq: i64,
    pub topics: Vec<String>,
    pub full: bool,
}
fn f_24h(r_bc: &AppState) -> bool {
    let mut r_b8 = r_bc
        .change_prune
        .lock()
        .unwrap_or_else(|r_ba| r_ba.into_inner());
    let r_b9 = Instant::now();
    match *r_b8 {
        Some(r_bb) if r_b9.duration_since(r_bb) < C_G3 => false,
        _ => {
            *r_b8 = Some(r_b9);
            true
        }
    }
}
async fn changes(
    State(r_bg): State<AppState>,
    _auth: Ts,
    Query(r_bf): Query<T2p>,
) -> Result<Json<T2o>, ApiError> {
    if f_24h(&r_bg) {
        if let Err(r_be) = digitext_db::changes::prune(&r_bg.pool, digitext_db::changes::C_I8).await
        {
            tracing::warn!(
                error = % r_be, "a2jDtG5nIGThu41uIMSRxrDhu6NjIGNoYW5nZV9sb2c="
            );
        }
    }
    let r_bd =
        digitext_db::changes::since(&r_bg.pool, r_bf.since, digitext_db::changes::C_F6).await?;
    Ok(Json(T2o {
        seq: r_bd.seq,
        topics: r_bd.topics,
        full: r_bd.full,
    }))
}
