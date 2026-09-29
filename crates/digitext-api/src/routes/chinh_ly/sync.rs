// Copyright (c) 2026 DigiText. All rights reserved.
//! Đồng bộ offline: pull theo `ver`, push lô op (gộp 3 chiều trên máy chủ), dọn tombstone 90 ngày.

use super::{ctx_parts, op_ctx, project_access, read_access};
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use digitext_db::chinh_ly::{self as cl, SyncOp};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/cl/sync", get(pull)).route("/cl/sync/push", post(push))
}

pub const PRUNE_DAYS: i64 = 90;
const PRUNE_EVERY: Duration = Duration::from_secs(3600);
pub const PUSH_MAX: usize = 500;

fn should_prune(st: &AppState) -> bool {
    let mut g = st.cl_prune.lock().unwrap_or_else(|e| e.into_inner());
    let now = Instant::now();
    match *g {
        Some(t) if now.duration_since(t) < PRUNE_EVERY => false,
        _ => {
            *g = Some(now);
            true
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullQ {
    project_id: Uuid,
    #[serde(default)]
    since: i64,
    #[serde(default = "default_limit")]
    limit: i64,
}
fn default_limit() -> i64 {
    2000
}

async fn pull(State(st): State<AppState>, ts: Ts, Query(q): Query<PullQ>) -> Result<Json<cl::sync::Pull>, ApiError> {
    read_access(&st, &ts, q.project_id).await?;
    if should_prune(&st) {
        if let Err(e) = cl::sync::prune(&st.pool, PRUNE_DAYS).await {
            tracing::warn!(error = %e, "không dọn được tombstone chỉnh lý");
        }
    }
    Ok(Json(cl::sync::pull(&st.pool, q.project_id, q.since.max(0), q.limit).await?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PushReq {
    project_id: Uuid,
    #[serde(default)]
    device_id: Option<Uuid>,
    ops: Vec<SyncOp>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PushResp {
    results: Vec<Value>,
    next: i64,
}

async fn push(State(st): State<AppState>, ts: Ts, Json(mut b): Json<PushReq>) -> Result<Json<PushResp>, ApiError> {
    project_access(&st, &ts, b.project_id).await?;
    if b.ops.len() > PUSH_MAX {
        return Err(ApiError::bad_request(format!("Tối đa {PUSH_MAX} thay đổi mỗi lần đẩy")));
    }
    cl::sync::order_ops(&mut b.ops);
    let parts = ctx_parts(&ts);
    let ctx = op_ctx(&ts, b.project_id, "sync", &parts);
    let mut results = Vec::with_capacity(b.ops.len());
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for op in &b.ops {
        let r = cl::apply_op(&st.pool, &ctx, op).await?;
        let mut v = serde_json::to_value(&r).unwrap_or(Value::Null);
        if let Some(o) = v.as_object_mut() {
            o.insert("opId".into(), json!(op.op_id));
            o.insert("table".into(), json!(op.table));
            o.insert("id".into(), json!(op.id));
            let status = o.get("status").and_then(Value::as_str).unwrap_or("?").to_owned();
            *counts.entry(status).or_default() += 1;
        }
        results.push(v);
    }
    let next: i64 = sqlx::query_scalar("SELECT coalesce(pg_sequence_last_value('cl_ver_seq'), 0)::bigint").fetch_one(&st.pool).await?;
    if !b.ops.is_empty() {
        digitext_db::audit::append(&st.pool, super::audit(&ts, "cl.sync_push", b.project_id.to_string(), json!({"ops": b.ops.len(), "counts": counts, "device": b.device_id}))).await?;
    }
    Ok(Json(PushResp { results, next }))
}
