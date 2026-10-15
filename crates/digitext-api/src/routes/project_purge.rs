//! "Xoá dữ liệu dự án" (15/10/2026, docs/KE-HOACH-CHUYEN-KHO-MINIO.md mục 6) — chỉ super_admin.
//! POST /projects/{id}/purge  { dryRun (mặc định true), confirmCode, reason, allowUndelivered }
//! GET  /projects/{id}/purge  → tiến độ lần xoá đang chạy / vừa xong (giữ trong RAM API).
//! Xoá theo lô 500 tài liệu: tệp trong kho TRƯỚC, dòng CSDL SAU → đứt giữa chừng chạy lại được.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use digitext_core::Role;
use digitext_db::project_purge::{self, PurgeCounts};
use digitext_db::{audit, projects};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/projects/{id}/purge", post(purge).get(status))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PurgeReq {
    #[serde(default = "mac_dinh_chay_thu")]
    dry_run: bool,
    #[serde(default)]
    confirm_code: Option<String>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    allow_undelivered: bool,
}
fn mac_dinh_chay_thu() -> bool {
    true
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreviewResp {
    project_code: String,
    project_name: String,
    counts: PurgeCounts,
    /// Lý do chưa cho xoá (rỗng = được xoá).
    blockers: Vec<String>,
    /// Dự án chưa có gói bàn giao — muốn xoá phải đánh dấu allowUndelivered.
    undelivered: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgeStatus {
    /// "running" | "done" | "failed"
    state: String,
    docs_total: i64,
    docs_done: i64,
    objects_deleted: u64,
    error: Option<String>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

static TIEN_DO: LazyLock<Mutex<HashMap<Uuid, PurgeStatus>>> = LazyLock::new(Default::default);

fn cap_nhat(id: Uuid, f: impl FnOnce(&mut PurgeStatus)) {
    if let Ok(mut m) = TIEN_DO.lock() {
        f(m.entry(id).or_default());
    }
}

fn chi_super_admin(ts: &Ts) -> Result<(), ApiError> {
    if ts.user.role == Role::SuperAdmin {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}

async fn status(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Option<PurgeStatus>>, ApiError> {
    let _ = &st;
    chi_super_admin(&ts)?;
    Ok(Json(TIEN_DO.lock().ok().and_then(|m| m.get(&id).cloned())))
}

async fn purge(
    State(st): State<AppState>,
    ts: Ts,
    Path(id): Path<Uuid>,
    Json(req): Json<PurgeReq>,
) -> Result<Json<serde_json::Value>, ApiError> {
    chi_super_admin(&ts)?;
    let project = projects::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    let counts = project_purge::counts(&st.pool, id).await?;
    let mut blockers = Vec::new();
    if counts.active_jobs > 0 {
        blockers.push(format!("còn {} việc OCR / PDF/A / đóng gói đang chờ hoặc đang chạy", counts.active_jobs));
    }
    if counts.open_sign_sessions > 0 {
        blockers.push(format!("còn {} phiên ký số đang mở", counts.open_sign_sessions));
    }
    let dang_chay = TIEN_DO.lock().ok().and_then(|m| m.get(&id).map(|s| s.state == "running")).unwrap_or(false);
    if dang_chay {
        blockers.push("đang xoá dở dự án này".into());
    }
    let undelivered = counts.packages == 0 && counts.documents > 0;
    if req.dry_run {
        return Ok(Json(serde_json::to_value(PreviewResp {
            project_code: project.code,
            project_name: project.name,
            counts,
            blockers,
            undelivered,
        })
        .map_err(|_| ApiError::internal())?));
    }
    if !blockers.is_empty() {
        return Err(ApiError::conflict(format!("chưa xoá được: {}", blockers.join("; "))));
    }
    let ma = req.confirm_code.as_deref().map(str::trim).unwrap_or_default();
    if !ma.eq_ignore_ascii_case(project.code.trim()) {
        return Err(ApiError::bad_request("mã dự án gõ lại không khớp"));
    }
    let reason = req.reason.as_deref().map(str::trim).unwrap_or_default().to_string();
    if reason.chars().count() < 5 {
        return Err(ApiError::bad_request("nhập lý do xoá (ít nhất 5 ký tự)"));
    }
    if undelivered && !req.allow_undelivered {
        return Err(ApiError::conflict(
            "dự án chưa có gói bàn giao nào — đánh dấu \"xoá dù chưa bàn giao\" nếu chắc chắn",
        ));
    }
    let store = st
        .store
        .clone()
        .ok_or_else(|| ApiError::service_unavailable("kho object chưa cấu hình"))?;
    audit::append(
        &st.pool,
        audit::Tp {
            actor_id: Some(ts.user.id),
            device_id: ts.claims.device_id,
            action: "project.purge_started",
            target: Some(format!("project:{id}")),
            detail: Some(serde_json::json!({
                "code": project.code, "name": project.name, "reason": reason,
                "allowUndelivered": req.allow_undelivered, "counts": counts,
            })),
            ip: ts.ip(),
        },
    )
    .await?;
    cap_nhat(id, |s| {
        *s = PurgeStatus {
            state: "running".into(),
            docs_total: counts.documents,
            started_at: Some(chrono::Utc::now()),
            ..Default::default()
        }
    });
    let pool = st.pool.clone();
    let (actor, device, ip) = (ts.user.id, ts.claims.device_id, ts.ip());
    tokio::spawn(async move {
        let kq = chay(&pool, &*store, id).await;
        let (state, error) = match &kq {
            Ok(()) => ("done", None),
            Err(e) => ("failed", Some(e.clone())),
        };
        cap_nhat(id, |s| {
            s.state = state.into();
            s.error = error.clone();
            s.finished_at = Some(chrono::Utc::now());
        });
        let s = TIEN_DO.lock().ok().and_then(|m| m.get(&id).cloned()).unwrap_or_default();
        if let Err(e) = audit::append(
            &pool,
            audit::Tp {
                actor_id: Some(actor),
                device_id: device,
                action: if kq.is_ok() { "project.purge_done" } else { "project.purge_failed" },
                target: Some(format!("project:{id}")),
                detail: Some(serde_json::json!({
                    "docsDone": s.docs_done, "objectsDeleted": s.objects_deleted, "error": error,
                })),
                ip,
            },
        )
        .await
        {
            tracing::error!(project = %id, error = %e, "không ghi được nhật ký xoá dự án");
        }
        tracing::warn!(project = %id, state, docs = s.docs_done, objects = s.objects_deleted, ?error, "xoá dữ liệu dự án");
    });
    Ok(Json(serde_json::json!({ "started": true })))
}

const LO: i64 = 500;

async fn xoa_khoa(store: &dyn digitext_storage::ObjectStore, key: &str) -> Result<(), String> {
    let mut lan = 0u64;
    loop {
        match store.delete(key).await {
            Ok(()) => return Ok(()),
            Err(e) if lan < 3 => {
                lan += 1;
                tracing::warn!(key, error = %e, lan, "xoá tệp kho lỗi — thử lại");
                tokio::time::sleep(std::time::Duration::from_secs(2 * lan)).await;
            }
            Err(e) => return Err(format!("xoá {key} lỗi: {e}")),
        }
    }
}

async fn chay(pool: &sqlx::PgPool, store: &dyn digitext_storage::ObjectStore, id: Uuid) -> Result<(), String> {
    let db = |e: sqlx::Error| format!("CSDL: {e}");
    loop {
        let ids = project_purge::document_batch(pool, id, LO).await.map_err(db)?;
        if ids.is_empty() {
            break;
        }
        let keys = project_purge::object_keys_of(pool, &ids).await.map_err(db)?;
        for k in &keys {
            xoa_khoa(store, k).await?;
        }
        let n = project_purge::delete_documents(pool, &ids).await.map_err(db)?;
        cap_nhat(id, |s| {
            s.docs_done += n as i64;
            s.objects_deleted += keys.len() as u64;
        });
    }
    for pkg in project_purge::package_keys(pool, id).await.map_err(db)? {
        xoa_khoa(store, &digitext_storage::f_1x5(id, pkg)).await?;
        cap_nhat(id, |s| s.objects_deleted += 1);
    }
    // Vét tệp sót theo tiền tố của dự án (bản gốc / PDF/A / gói không còn dòng CSDL trỏ tới).
    for prefix in [format!("{id}/"), format!("{id}-warm/"), format!("{id}-packages/")] {
        let sot = store.list_prefix(&prefix).await.map_err(|e| format!("liệt kê {prefix}: {e}"))?;
        for k in &sot {
            xoa_khoa(store, k).await?;
        }
        cap_nhat(id, |s| s.objects_deleted += sot.len() as u64);
    }
    project_purge::delete_rest(pool, id).await.map_err(db)
}
