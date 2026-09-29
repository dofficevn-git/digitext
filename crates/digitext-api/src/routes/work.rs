// Copyright (c) 2026 DigiText. All rights reserved.
//! Gói việc & cộng tác viên xác nhận trường (kế hoạch 29/9/2026, từ doc_extract_pla).
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use digitext_core::Permission;
use digitext_db::work_packages::{self as wp, Outcome};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/work/packages", get(list_packages).post(create_package))
        .route("/work/packages/{id}", get(get_package).patch(patch_package))
        .route("/work/packages/{id}/members", put(put_members))
        .route("/work/packages/{id}/next", post(next_item))
        .route("/work/packages/{id}/review", post(review))
        .route("/work/packages/{id}/close", post(close_package))
        .route("/work/my", get(my_packages))
        .route("/work/items/rejected", get(rejected_items))
        .route("/work/items/{id}", get(get_item))
        .route("/work/items/{id}/claim", post(claim_item))
        .route("/work/items/{id}/navigate", post(navigate))
        .route("/work/items/{id}/release", post(release))
        .route("/work/items/{id}/done", post(done))
        .route("/work/items/{id}/check", post(check))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MemberDto {
    user_id: Uuid,
    username: String,
    display_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageDto {
    id: Uuid,
    project_id: Uuid,
    name: String,
    kind: String,
    mode: String,
    status: String,
    due_at: Option<NaiveDate>,
    note: Option<String>,
    source_package_id: Option<Uuid>,
    sample_pct: Option<i32>,
    created_at: DateTime<Utc>,
    n_total: i64,
    n_pending: i64,
    n_done: i64,
    n_checked: i64,
    n_rejected: i64,
    n_approved: i64,
    members: Vec<MemberDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemDto {
    id: Uuid,
    package_id: Uuid,
    document_id: Uuid,
    ord: i32,
    status: String,
    locked_by: Option<Uuid>,
    lease_until: Option<DateTime<Utc>>,
    done_by: Option<Uuid>,
    done_by_name: Option<String>,
    done_at: Option<DateTime<Utc>>,
    checked_by_name: Option<String>,
    check_note: Option<String>,
    review_note: Option<String>,
    filename: String,
    rel_path: Option<String>,
    ho_so_ref: Option<String>,
    page_count: Option<i32>,
}
impl From<wp::Item> for ItemDto {
    fn from(i: wp::Item) -> Self {
        ItemDto {
            id: i.id,
            package_id: i.package_id,
            document_id: i.document_id,
            ord: i.ord,
            status: i.status,
            locked_by: i.locked_by,
            lease_until: i.lease_until,
            done_by: i.done_by,
            done_by_name: i.done_by_name,
            done_at: i.done_at,
            checked_by_name: i.checked_by_name,
            check_note: i.check_note,
            review_note: i.review_note,
            filename: i.filename,
            rel_path: i.rel_path,
            ho_so_ref: i.ho_so_ref,
            page_count: i.page_count,
        }
    }
}

async fn with_members(pool: &sqlx::PgPool, pkgs: Vec<wp::Package>) -> Result<Vec<PackageDto>, ApiError> {
    let ids: Vec<Uuid> = pkgs.iter().map(|p| p.id).collect();
    let mems = wp::members(pool, &ids).await?;
    Ok(pkgs
        .into_iter()
        .map(|p| PackageDto {
            members: mems
                .iter()
                .filter(|m| m.package_id == p.id)
                .map(|m| MemberDto { user_id: m.user_id, username: m.username.clone(), display_name: m.display_name.clone() })
                .collect(),
            id: p.id,
            project_id: p.project_id,
            name: p.name,
            kind: p.kind,
            mode: p.mode,
            status: p.status,
            due_at: p.due_at,
            note: p.note,
            source_package_id: p.source_package_id,
            sample_pct: p.sample_pct,
            created_at: p.created_at,
            n_total: p.n_total,
            n_pending: p.n_pending,
            n_done: p.n_done,
            n_checked: p.n_checked,
            n_rejected: p.n_rejected,
            n_approved: p.n_approved,
        })
        .collect())
}

async fn manage_scope(st: &AppState, ts: &Ts, project: Uuid) -> Result<(), ApiError> {
    ts.require_perm(Permission::WorkManage)?;
    if !f_29a(&st.pool, &ts.user).await?.allows_project(project) {
        return Err(ApiError::forbidden());
    }
    Ok(())
}

async fn managed_package(st: &AppState, ts: &Ts, id: Uuid) -> Result<wp::Package, ApiError> {
    let p = wp::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    manage_scope(st, ts, p.project_id).await?;
    Ok(p)
}

/// Gói mà CTV (người gọi) được làm: thành viên + chưa đóng.
async fn member_package(st: &AppState, ts: &Ts, id: Uuid) -> Result<wp::Package, ApiError> {
    ts.require_perm(Permission::WorkExecute)?;
    let p = wp::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    if p.status == "closed" || !wp::is_member(&st.pool, id, ts.user.id).await? {
        return Err(ApiError::forbidden());
    }
    Ok(p)
}

fn audit(ts: &Ts, action: &'static str, target: String, detail: serde_json::Value) -> digitext_db::audit::Tp<'static> {
    digitext_db::audit::Tp {
        actor_id: Some(ts.user.id),
        device_id: ts.claims.device_id,
        action,
        target: Some(target),
        detail: Some(detail),
        ip: ts.ip(),
    }
}

// ---------------------------------------------------------------- quản trị

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    #[serde(default)]
    project_id: Option<Uuid>,
}

async fn list_packages(State(st): State<AppState>, ts: Ts, Query(q): Query<ListQuery>) -> Result<Json<Vec<PackageDto>>, ApiError> {
    ts.require_perm(Permission::WorkManage)?;
    let scope = f_29a(&st.pool, &ts.user).await?;
    let projects: Option<Vec<Uuid>> = match (&scope, q.project_id) {
        (s, Some(pid)) => {
            if !s.allows_project(pid) {
                return Err(ApiError::forbidden());
            }
            Some(vec![pid])
        }
        (Scope::All, None) => None,
        (Scope::Projects(p), None) => Some(p.clone()),
    };
    let pkgs = wp::list_for_projects(&st.pool, projects.as_deref()).await?;
    Ok(Json(with_members(&st.pool, pkgs).await?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateReq {
    project_id: Uuid,
    #[serde(default)]
    name: String,
    /// xac_nhan | kiem_tra
    kind: String,
    /// hoso | ngau_nhien
    #[serde(default = "default_mode")]
    mode: String,
    /// Tiền tố rel_path (phông / hộp / hồ sơ) giới hạn phạm vi; rỗng = cả dự án.
    #[serde(default)]
    prefixes: Vec<String>,
    /// Cỡ mỗi gói (số tài liệu).
    #[serde(default = "default_size")]
    size: usize,
    /// Số gói tạo liên tiếp (chia đều).
    #[serde(default = "default_count")]
    count: usize,
    #[serde(default)]
    member_ids: Vec<Uuid>,
    #[serde(default)]
    due_at: Option<NaiveDate>,
    #[serde(default)]
    note: Option<String>,
    /// Gói kiem_tra: soát gói xac_nhan nào, lấy mẫu bao nhiêu %.
    #[serde(default)]
    source_package_id: Option<Uuid>,
    #[serde(default)]
    sample_pct: Option<i32>,
    /// Chỉ xem trước số tài liệu / số gói, không tạo.
    #[serde(default)]
    dry_run: bool,
}
fn default_mode() -> String {
    "hoso".into()
}
fn default_size() -> usize {
    50
}
fn default_count() -> usize {
    1
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateResp {
    /// Tài liệu đủ điều kiện trong phạm vi (đã OCR xong, chưa giao).
    available: usize,
    /// Cỡ từng gói sẽ tạo / đã tạo.
    sizes: Vec<usize>,
    /// Trong số được giao, bao nhiêu tài liệu CHƯA bóc trường (tạo gói sẽ tự xếp hàng bóc).
    to_extract: usize,
    package_ids: Vec<Uuid>,
}

async fn create_package(State(st): State<AppState>, ts: Ts, Json(r): Json<CreateReq>) -> Result<Json<CreateResp>, ApiError> {
    manage_scope(&st, &ts, r.project_id).await?;
    if !matches!(r.kind.as_str(), "xac_nhan" | "kiem_tra") {
        return Err(ApiError::bad_request("loại gói phải là xac_nhan hoặc kiem_tra"));
    }
    if !matches!(r.mode.as_str(), "hoso" | "ngau_nhien") {
        return Err(ApiError::bad_request("cách chia phải là hoso hoặc ngau_nhien"));
    }
    let size = r.size.clamp(1, 5000);
    let count = r.count.clamp(1, 200);
    // Thành viên phải thuộc dự án (hoặc là quản trị của dự án).
    for m in &r.member_ids {
        if !wp::is_project_user(&st.pool, r.project_id, *m).await? {
            return Err(ApiError::bad_request(format!("người {m} không thuộc dự án — thêm vào dự án trước")));
        }
    }
    let seed = Utc::now().timestamp_nanos_opt().unwrap_or(1) as u64;
    let mut no_fields: std::collections::HashSet<Uuid> = Default::default();
    let groups: Vec<Vec<Uuid>> = if r.kind == "kiem_tra" {
        let src = r.source_package_id.ok_or_else(|| ApiError::bad_request("gói kiểm tra cần chọn gói xác nhận nguồn"))?;
        let src_pkg = managed_package(&st, &ts, src).await?;
        if src_pkg.kind != "xac_nhan" || src_pkg.project_id != r.project_id {
            return Err(ApiError::bad_request("gói nguồn phải là gói xác nhận cùng dự án"));
        }
        let pct = r.sample_pct.unwrap_or(20).clamp(1, 100);
        let cands = wp::done_candidates(&st.pool, src).await?;
        let n = ((cands.len() * pct as usize) + 99) / 100;
        let mut g = wp::split_random(&cands, n.max(1), 1, seed);
        g.truncate(1);
        g
    } else {
        let cands = wp::candidates(&st.pool, r.project_id, &r.prefixes).await?;
        no_fields = cands.iter().filter(|c| !c.has_fields).map(|c| c.id).collect();
        if r.mode == "hoso" {
            wp::split_hoso(&cands, size, count)
        } else {
            wp::split_random(&cands, size, count, seed)
        }
    };
    let available = if r.kind == "kiem_tra" {
        wp::done_candidates(&st.pool, r.source_package_id.unwrap_or_default()).await?.len()
    } else {
        wp::candidates(&st.pool, r.project_id, &r.prefixes).await?.len()
    };
    let sizes: Vec<usize> = groups.iter().map(Vec::len).collect();
    let to_extract = groups.iter().flatten().filter(|d| no_fields.contains(d)).count();
    if r.dry_run {
        return Ok(Json(CreateResp { available, sizes, to_extract, package_ids: Vec::new() }));
    }
    if groups.is_empty() {
        return Err(ApiError::bad_request("không có tài liệu nào đủ điều kiện (đã OCR xong, chưa giao) trong phạm vi"));
    }
    // Tài liệu chưa bóc trường → xếp hàng bóc ngay để CTV mở ra là có sẵn trường máy điền.
    for d in groups.iter().flatten().filter(|d| no_fields.contains(d)) {
        digitext_db::ocr_jobs::f_19p(&st.pool, *d).await?;
    }
    let base = if r.name.trim().is_empty() {
        format!("Gói {}", Utc::now().format("%d/%m %H:%M"))
    } else {
        r.name.trim().to_owned()
    };
    let mut ids = Vec::new();
    for (i, docs) in groups.iter().enumerate() {
        let name = if groups.len() > 1 { format!("{base} #{}", i + 1) } else { base.clone() };
        let id = wp::create(
            &st.pool,
            &wp::NewPackage {
                project_id: r.project_id,
                name: &name,
                kind: &r.kind,
                mode: &r.mode,
                due_at: r.due_at,
                note: r.note.as_deref(),
                source_package_id: r.source_package_id,
                sample_pct: if r.kind == "kiem_tra" { Some(r.sample_pct.unwrap_or(20).clamp(1, 100)) } else { None },
                members: &r.member_ids,
                created_by: ts.user.id,
            },
            docs,
        )
        .await?;
        ids.push(id);
    }
    digitext_db::audit::append(
        &st.pool,
        audit(&ts, "work.package_create", format!("project:{}", r.project_id), serde_json::json!({ "packages": ids, "sizes": sizes, "kind": r.kind, "mode": r.mode })),
    )
    .await?;
    Ok(Json(CreateResp { available, sizes, to_extract, package_ids: ids }))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageDetail {
    #[serde(flatten)]
    package: PackageDto,
    items: Vec<ItemDto>,
}

async fn get_package(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<PackageDetail>, ApiError> {
    // Quản trị xem mọi gói trong phạm vi; CTV xem gói mình là thành viên.
    let p = if ts.has_perm(Permission::WorkManage) {
        managed_package(&st, &ts, id).await?
    } else {
        member_package(&st, &ts, id).await?
    };
    let package = with_members(&st.pool, vec![p]).await?.pop().ok_or_else(ApiError::not_found)?;
    let items = wp::items(&st.pool, id).await?.into_iter().map(ItemDto::from).collect();
    Ok(Json(PackageDetail { package, items }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PatchReq {
    #[serde(default)]
    name: Option<String>,
    /// Gửi khoá `dueAt` (kể cả null) mới đổi hạn.
    #[serde(default, deserialize_with = "deserialize_some")]
    due_at: Option<Option<NaiveDate>>,
    #[serde(default)]
    note: Option<String>,
}
fn deserialize_some<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Option<NaiveDate>>, D::Error> {
    Ok(Some(Option::<NaiveDate>::deserialize(d)?))
}

async fn patch_package(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(r): Json<PatchReq>) -> Result<Json<PackageDto>, ApiError> {
    managed_package(&st, &ts, id).await?;
    wp::update_meta(&st.pool, id, r.name.as_deref().map(str::trim).filter(|s| !s.is_empty()), r.due_at, r.note.as_deref()).await?;
    let p = wp::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    Ok(Json(with_members(&st.pool, vec![p]).await?.pop().ok_or_else(ApiError::not_found)?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MembersReq {
    user_ids: Vec<Uuid>,
}

async fn put_members(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(r): Json<MembersReq>) -> Result<Json<PackageDto>, ApiError> {
    let p = managed_package(&st, &ts, id).await?;
    wp::set_members(&st.pool, id, &r.user_ids, ts.user.id).await?;
    digitext_db::audit::append(&st.pool, audit(&ts, "work.members_set", format!("work_package:{id}"), serde_json::json!({ "userIds": r.user_ids }))).await?;
    let p = wp::get(&st.pool, p.id).await?.ok_or_else(ApiError::not_found)?;
    Ok(Json(with_members(&st.pool, vec![p]).await?.pop().ok_or_else(ApiError::not_found)?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReviewReq {
    /// Không gửi = tất cả mục đủ điều kiện trong gói.
    #[serde(default)]
    item_ids: Option<Vec<Uuid>>,
    /// approve | reject
    decision: String,
    #[serde(default)]
    note: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewResp {
    changed: usize,
    skipped: usize,
}

async fn review(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(r): Json<ReviewReq>) -> Result<Json<ReviewResp>, ApiError> {
    managed_package(&st, &ts, id).await?;
    let approve = match r.decision.as_str() {
        "approve" => true,
        "reject" => false,
        _ => return Err(ApiError::bad_request("decision phải là approve hoặc reject")),
    };
    let note = r.note.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if !approve && note.is_none() {
        return Err(ApiError::bad_request("từ chối phải ghi lý do để CTV sửa"));
    }
    let (changed, skipped, docs) = wp::review(&st.pool, id, r.item_ids.as_deref(), approve, note, ts.user.id).await?;
    if approve {
        // Duyệt = tài liệu "approved" (như /documents/{id}/review) + trang 1 thành ví dụ dạy máy.
        for d in &docs {
            digitext_db::documents::f_2ef(&st.pool, *d, digitext_core::ReviewStatus::Approved, Some(ts.user.id), None).await?;
            if let Err(e) = digitext_db::kie_examples::upsert_from_document(&st.pool, *d, 1, ts.user.id).await {
                tracing::warn!(error = %e, document = %d, "không lưu được ví dụ bóc trường sau duyệt");
            }
        }
    }
    digitext_db::audit::append(
        &st.pool,
        audit(&ts, "work.review", format!("work_package:{id}"), serde_json::json!({ "decision": r.decision, "changed": changed, "skipped": skipped, "note": note })),
    )
    .await?;
    Ok(Json(ReviewResp { changed, skipped }))
}

async fn close_package(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<PackageDto>, ApiError> {
    managed_package(&st, &ts, id).await?;
    wp::close(&st.pool, id, ts.user.id).await?;
    digitext_db::audit::append(&st.pool, audit(&ts, "work.package_close", format!("work_package:{id}"), serde_json::json!({}))).await?;
    let p = wp::get(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    Ok(Json(with_members(&st.pool, vec![p]).await?.pop().ok_or_else(ApiError::not_found)?))
}

// ---------------------------------------------------------------- cộng tác viên

async fn my_packages(State(st): State<AppState>, ts: Ts) -> Result<Json<Vec<PackageDto>>, ApiError> {
    ts.require_perm(Permission::WorkExecute)?;
    let pkgs = wp::list_for_member(&st.pool, ts.user.id).await?;
    Ok(Json(with_members(&st.pool, pkgs).await?))
}

async fn rejected_items(State(st): State<AppState>, ts: Ts) -> Result<Json<Vec<ItemDto>>, ApiError> {
    ts.require_perm(Permission::WorkExecute)?;
    Ok(Json(wp::rejected_for(&st.pool, ts.user.id).await?.into_iter().map(ItemDto::from).collect()))
}

async fn next_item(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Option<ItemDto>>, ApiError> {
    let p = member_package(&st, &ts, id).await?;
    Ok(Json(wp::claim_next(&st.pool, id, &p.kind, ts.user.id).await?.map(ItemDto::from)))
}

async fn item_for_member(st: &AppState, ts: &Ts, id: Uuid) -> Result<(wp::Item, wp::Package), ApiError> {
    ts.require_perm(Permission::WorkExecute)?;
    let it = wp::item(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
    let p = member_package(st, ts, it.package_id).await?;
    Ok((it, p))
}

async fn get_item(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<ItemDto>, ApiError> {
    let it = if ts.has_perm(Permission::WorkManage) {
        let it = wp::item(&st.pool, id).await?.ok_or_else(ApiError::not_found)?;
        managed_package(&st, &ts, it.package_id).await?;
        it
    } else {
        item_for_member(&st, &ts, id).await?.0
    };
    Ok(Json(ItemDto::from(it)))
}

async fn claim_item(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<Option<ItemDto>>, ApiError> {
    let (_, p) = item_for_member(&st, &ts, id).await?;
    Ok(Json(wp::claim_item(&st.pool, id, &p.kind, ts.user.id).await?.map(ItemDto::from)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NavReq {
    /// prev | next
    direction: String,
}

async fn navigate(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(r): Json<NavReq>) -> Result<Json<Option<ItemDto>>, ApiError> {
    let (_, p) = item_for_member(&st, &ts, id).await?;
    let dir = match r.direction.as_str() {
        "prev" => -1,
        "next" => 1,
        _ => return Err(ApiError::bad_request("direction phải là prev hoặc next")),
    };
    Ok(Json(wp::navigate(&st.pool, id, &p.kind, ts.user.id, dir).await?.map(ItemDto::from)))
}

async fn release(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, ApiError> {
    item_for_member(&st, &ts, id).await?;
    wp::release(&st.pool, id, ts.user.id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn done(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>) -> Result<Json<ItemDto>, ApiError> {
    let (_, p) = item_for_member(&st, &ts, id).await?;
    if p.kind != "xac_nhan" {
        return Err(ApiError::bad_request("gói kiểm tra dùng nút Đạt / Có lỗi"));
    }
    match wp::mark_done(&st.pool, id, ts.user.id).await? {
        Outcome::Ok => {}
        Outcome::Conflict => {
            return Err(ApiError::conflict("mục này đã được xác nhận, hoặc khoá của bạn đã hết hạn — bấm Tiếp để nhận lại"))
        }
    }
    digitext_db::audit::append(&st.pool, audit(&ts, "work.item_done", format!("work_item:{id}"), serde_json::json!({}))).await?;
    Ok(Json(ItemDto::from(wp::item(&st.pool, id).await?.ok_or_else(ApiError::not_found)?)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckReq {
    ok: bool,
    #[serde(default)]
    note: Option<String>,
}

async fn check(State(st): State<AppState>, ts: Ts, Path(id): Path<Uuid>, Json(r): Json<CheckReq>) -> Result<Json<ItemDto>, ApiError> {
    let (_, p) = item_for_member(&st, &ts, id).await?;
    if p.kind != "kiem_tra" {
        return Err(ApiError::bad_request("chỉ gói kiểm tra mới chấm Đạt / Có lỗi"));
    }
    let note = r.note.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if !r.ok && note.is_none() {
        return Err(ApiError::bad_request("chấm Có lỗi phải ghi lỗi gì"));
    }
    match wp::mark_checked(&st.pool, id, ts.user.id, r.ok, note).await? {
        Outcome::Ok => {}
        Outcome::Conflict => return Err(ApiError::conflict("mục này đã được chấm, hoặc khoá của bạn đã hết hạn")),
    }
    digitext_db::audit::append(&st.pool, audit(&ts, "work.item_check", format!("work_item:{id}"), serde_json::json!({ "ok": r.ok, "note": note }))).await?;
    Ok(Json(ItemDto::from(wp::item(&st.pool, id).await?.ok_or_else(ApiError::not_found)?)))
}
