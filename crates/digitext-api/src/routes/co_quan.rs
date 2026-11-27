// Copyright (c) 2026 DigiText. All rights reserved.
//! Cấu hình CƠ QUAN LƯU TRỮ của một dự án (0039, 27/11/2026 — đợt 5, Digifile B01; Điều 4 + Phụ lục I TT 05/2025/TT-BNV).
//! `GET /projects/{id}/co-quan` (người thuộc dự án) trả cấu hình đã khai + giá trị hiệu dụng; `PUT` (quyền `project.update` trong
//! phạm vi dự án) ghi đè. Giá trị dùng ở: hạn nộp lưu / mốc nhắc / quyền sử dụng (nộp lưu), khối ký báo cáo TT05, trang thể thức
//! chuyển đổi PL VI, tên cơ quan trên tiêu đề in.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Ts};
use crate::routes::chinh_ly::project_access;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use digitext_core::Permission;
use digitext_db::{audit, co_quan};
use serde::Serialize;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/projects/{id}/co-quan", get(xem).put(ghi))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoQuanDto {
    pub project_id: Uuid,
    /// Dòng đã khai (null = chưa khai, đang dùng mặc định).
    pub cau_hinh: Option<co_quan::CoQuanRow>,
    pub hieu_dung: co_quan::HieuDung,
}

async fn dto(st: &AppState, project: Uuid) -> Result<CoQuanDto, ApiError> {
    Ok(CoQuanDto { project_id: project, cau_hinh: co_quan::get(&st.pool, project).await?, hieu_dung: co_quan::hieu_dung(&st.pool, project).await? })
}

async fn xem(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<CoQuanDto>, ApiError> {
    project_access(&st, &ts, id).await?;
    if digitext_db::projects::get(&st.pool, id).await?.is_none() {
        return Err(ApiError::not_found());
    }
    Ok(Json(dto(&st, id).await?))
}

async fn ghi(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(b): Json<co_quan::Moi>) -> Result<Json<CoQuanDto>, ApiError> {
    ts.require_perm(Permission::ProjectUpdate)?;
    if !f_29a(&st.pool, &ts.user).await?.allows_project(id) {
        return Err(ApiError::forbidden());
    }
    if digitext_db::projects::get(&st.pool, id).await?.is_none() {
        return Err(ApiError::not_found());
    }
    let row = co_quan::dat(&st.pool, id, &b, ts.user.id).await?.map_err(ApiError::bad_request)?;
    audit::append(
        &st.pool,
        audit::Tp {
            actor_id: Some(ts.user.id),
            device_id: ts.claims.device_id,
            action: "project.co_quan",
            target: Some(format!("project:{id}")),
            detail: Some(serde_json::json!({ "maCoQuan": row.ma_co_quan, "loaiHinh": row.loai_hinh, "hanNopNgay": row.han_nop_ngay })),
            ip: ts.ip(),
        },
    )
    .await?;
    Ok(Json(dto(&st, id).await?))
}
