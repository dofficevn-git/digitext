// Copyright (c) 2026 DigiText. All rights reserved.
//! Ghi / đọc một dòng bất kỳ của 15 bảng cl_* qua `apply_op` (quyền theo bảng + thao tác).

use super::{ctx_parts, op_ctx, op_response, project_access, read_access, table_name};
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use digitext_db::chinh_ly::{self as cl, OpKind, Row, SyncOp};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cl/rows/{table}", post(insert))
        .route("/cl/rows/{table}/{id}", get(get_row).put(update).delete(delete))
        .route("/cl/rows/{table}/{id}/restore", post(restore))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectQ {
    project_id: Uuid,
    #[serde(default)]
    if_ver: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WriteBody {
    project_id: Uuid,
    #[serde(default)]
    id: Option<Uuid>,
    #[serde(default)]
    if_ver: Option<i64>,
    #[serde(default)]
    base: Option<Row>,
    #[serde(default)]
    new: Row,
}

async fn get_row(State(st): State<AppState>, ts: Ts, Path((table, id)): Path<(String, Uuid)>) -> Result<Json<Value>, ApiError> {
    let t = table_name(&table);
    cl::spec::spec(&t).ok_or_else(ApiError::not_found)?;
    let mut conn = st.pool.acquire().await?;
    let row = cl::rows::get(&mut conn, &t, id).await?.ok_or_else(ApiError::not_found)?;
    let project = cl::rows::uuid_of(&row, "project_id");
    match project {
        Some(p) => read_access(&st, &ts, p).await?,
        None => {
            if !(ts.has_perm(digitext_core::Permission::CatalogRead) || ts.has_perm(digitext_core::Permission::DossierRead)) {
                return Err(ApiError::forbidden());
            }
        }
    }
    Ok(Json(Value::Object(row)))
}

async fn insert(State(st): State<AppState>, ts: Ts, Path(table): Path<String>, Json(b): Json<WriteBody>) -> Result<Response, ApiError> {
    let t = table_name(&table);
    project_access(&st, &ts, b.project_id).await?;
    let parts = ctx_parts(&ts);
    let ctx = op_ctx(&ts, b.project_id, "online", &parts);
    let op = SyncOp { op_id: None, table: t, id: b.id.unwrap_or_else(Uuid::now_v7), op: OpKind::Upsert, base: None, new: b.new, if_ver: None, edited_at: None };
    let res = cl::apply_op(&st.pool, &ctx, &op).await?;
    Ok(op_response(res))
}

async fn update(State(st): State<AppState>, ts: Ts, Path((table, id)): Path<(String, Uuid)>, Json(b): Json<WriteBody>) -> Result<Response, ApiError> {
    let t = table_name(&table);
    project_access(&st, &ts, b.project_id).await?;
    let parts = ctx_parts(&ts);
    let ctx = op_ctx(&ts, b.project_id, "online", &parts);
    let op = SyncOp { op_id: None, table: t, id, op: OpKind::Upsert, base: b.base, new: b.new, if_ver: b.if_ver, edited_at: None };
    let res = cl::apply_op(&st.pool, &ctx, &op).await?;
    Ok(op_response(res))
}

async fn delete(State(st): State<AppState>, ts: Ts, Path((table, id)): Path<(String, Uuid)>, Query(q): Query<ProjectQ>) -> Result<Response, ApiError> {
    let t = table_name(&table);
    project_access(&st, &ts, q.project_id).await?;
    let parts = ctx_parts(&ts);
    let ctx = op_ctx(&ts, q.project_id, "online", &parts);
    let op = SyncOp { op_id: None, table: t.clone(), id, op: OpKind::Delete, base: None, new: Row::new(), if_ver: q.if_ver, edited_at: None };
    let res = cl::apply_op(&st.pool, &ctx, &op).await?;
    if res.is_ok() {
        digitext_db::audit::append(&st.pool, super::audit(&ts, "cl.delete", id.to_string(), serde_json::json!({"table": t}))).await?;
    }
    Ok(op_response(res))
}

async fn restore(State(st): State<AppState>, ts: Ts, Path((table, id)): Path<(String, Uuid)>, Query(q): Query<ProjectQ>) -> Result<Json<Value>, ApiError> {
    let t = table_name(&table);
    let sp = cl::spec::spec(&t).ok_or_else(ApiError::not_found)?;
    project_access(&st, &ts, q.project_id).await?;
    ts.require_perm(sp.perm_for(cl::spec::WriteOp::Delete))?;
    let mut tx = st.pool.begin().await?;
    let row = cl::rows::get_for_update(&mut tx, &t, id).await?.ok_or_else(ApiError::not_found)?;
    if cl::rows::uuid_of(&row, "project_id") != Some(q.project_id) {
        return Err(ApiError::forbidden());
    }
    if row.get("deleted_at").map_or(true, Value::is_null) {
        return Ok(Json(Value::Object(row)));
    }
    let n = cl::rows::restore(&mut tx, sp, id, Some(ts.user.id)).await.map_err(|e| {
        if digitext_db::f_1jn(&e) {
            ApiError::conflict("Không khôi phục được: số tạm / số thứ tự đã bị bản ghi khác dùng")
        } else {
            e.into()
        }
    })?;
    let after = cl::rows::get(&mut tx, &t, id).await?.unwrap_or_default();
    if sp.history {
        let entity = t.trim_start_matches("cl_");
        let (ho_so_id, phong_id) = if t == "cl_van_ban" {
            (cl::rows::uuid_of(&after, "ho_so_id").unwrap_or(id), None)
        } else {
            (id, cl::rows::uuid_of(&after, "phong_id"))
        };
        cl::lich_su::insert(
            &mut tx,
            cl::lich_su::NewEntry {
                project_id: q.project_id,
                entity,
                entity_id: id,
                ho_so_id,
                phong_id,
                action: "restore",
                actor_id: Some(ts.user.id),
                actor_name: Some(&ts.user.display_name),
                at_client: None,
                source: "online",
                title: cl::rows::str_of(&after, "tieu_de").or(cl::rows::str_of(&after, "trich_yeu")),
                diffs: vec![],
                n_new: 0,
                n_edited: 0,
            },
        )
        .await?;
    }
    tx.commit().await?;
    digitext_db::audit::append(&st.pool, super::audit(&ts, "cl.restore", id.to_string(), serde_json::json!({"table": t, "rows": n}))).await?;
    Ok(Json(Value::Object(after)))
}
