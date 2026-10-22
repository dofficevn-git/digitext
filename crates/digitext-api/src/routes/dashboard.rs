// Copyright (c) 2026 DigiText. All rights reserved.
//! Dashboard 3 góc nhìn (22/10/2026): lãnh đạo (mọi dự án), quản lý dự án (một dự án), nhân viên (số của mình +
//! trung bình nhóm ẩn danh). Số theo ngày đọc từ `dashboard_daily` (migration 0021, API tự tổng hợp — xem cli.rs);
//! số hiện tại đọc trực tiếp theo dự án. Toán sức khoẻ / dự kiến ở digitext_core::dashboard — desktop không tự tính.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use digitext_core::dashboard::{self as calc, Health, HealthInput, KpiThresholds, RagThresholds, Rates, Rating};
use digitext_core::{Perm, Permission};
use digitext_db::dashboard::{self as db, KpiSettingsRow, Metrics};
use digitext_db::progress;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dashboard/overview", get(overview))
        .route("/dashboard/project", get(project))
        .route("/dashboard/staff", get(staff))
        .route("/dashboard/me", get(me))
        .route("/dashboard/kpi-settings", get(kpi_get).put(kpi_put))
        .route("/dashboard/rollup", post(rollup))
}

// ---------------------------------------------------------------------------------------------------------------
// Cache 60 s theo (route, phạm vi, tham số) — như STATS_CACHE của documents.rs; job tổng hợp xoá khi có số mới.
// ---------------------------------------------------------------------------------------------------------------
type CacheMap = HashMap<String, (std::time::Instant, std::sync::Arc<dyn std::any::Any + Send + Sync>)>;
static DASH_CACHE: std::sync::OnceLock<std::sync::Mutex<CacheMap>> = std::sync::OnceLock::new();
const DASH_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(60);
fn cache_get<T: Clone + Send + Sync + 'static>(key: &str) -> Option<T> {
    let c = DASH_CACHE.get_or_init(Default::default).lock().ok()?;
    let (at, v) = c.get(key)?;
    (at.elapsed() < DASH_CACHE_TTL).then(|| v.downcast_ref::<T>().cloned()).flatten()
}
fn cache_put<T: Clone + Send + Sync + 'static>(key: String, v: &T) {
    if let Ok(mut c) = DASH_CACHE.get_or_init(Default::default).lock() {
        c.retain(|_, (at, _)| at.elapsed() < DASH_CACHE_TTL * 10);
        c.insert(key, (std::time::Instant::now(), std::sync::Arc::new(v.clone())));
    }
}
/// Xoá toàn bộ cache (job tổng hợp gọi sau mỗi lần có dòng mới).
pub fn cache_clear() {
    if let Ok(mut c) = DASH_CACHE.get_or_init(Default::default).lock() {
        c.clear();
    }
}
fn scope_key(p: &Option<Vec<Uuid>>) -> String {
    match p {
        None => "*".into(),
        Some(ids) => {
            let mut v: Vec<String> = ids.iter().map(|u| u.to_string()).collect();
            v.sort();
            v.join(",")
        }
    }
}

fn clamp_days(d: Option<i64>, default: i64) -> i64 {
    d.unwrap_or(default).clamp(1, 365)
}

/// Phạm vi dự án theo quyền (StatsViewProject) và tham số `projectId`.
async fn scoped_projects(st: &AppState, ts: &Ts, project_id: Option<Uuid>) -> Result<Option<Vec<Uuid>>, ApiError> {
    ts.require_perm(Permission::StatsViewProject)?;
    let scope = f_29a(&st.pool, &ts.user).await?;
    Ok(match (&scope, project_id) {
        (s, Some(pid)) => {
            if !s.allows_project(pid) {
                return Err(ApiError::forbidden());
            }
            Some(vec![pid])
        }
        (Scope::All, None) => None,
        (Scope::Projects(p), None) => Some(p.clone()),
    })
}

fn rates_of(rows: &[&Metrics]) -> Rates {
    let n = rows.len().max(1) as f64;
    let sum = rows.iter().fold(Metrics::default(), |mut a, m| {
        a.add(m);
        a
    });
    Rates {
        pages_scanned: sum.pages_scanned as f64 / n,
        docs_pdfa: sum.docs_pdfa as f64 / n,
        docs_signed: sum.docs_signed as f64 / n,
        docs_approved: sum.docs_reviewed_approved as f64 / n,
    }
}

/// Tốc độ 7 ngày gần nhất (chia đều cho 7 kể cả ngày không làm) từ các dòng ngày của một dự án.
fn rate7d(days: &BTreeMap<NaiveDate, Metrics>, today: NaiveDate) -> Rates {
    let from = calc::days_back(today, 7);
    let rows: Vec<&Metrics> = days.range(from..=today).map(|(_, m)| m).collect();
    let r = rates_of(&rows);
    let k = rows.len().max(1) as f64 / 7.0;
    Rates { pages_scanned: r.pages_scanned * k, docs_pdfa: r.docs_pdfa * k, docs_signed: r.docs_signed * k, docs_approved: r.docs_approved * k }
}

fn thresholds_of(s: &KpiSettingsRow) -> (RagThresholds, KpiThresholds) {
    (
        RagThresholds { amber_lech: s.rag_amber_lech, red_lech: s.rag_red_lech },
        KpiThresholds { excellent_pct: s.excellent_pct, pass_pct: s.pass_pct, excellent_err_pct: s.excellent_err_pct, pass_err_pct: s.pass_err_pct },
    )
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ContractDto {
    signed_on: Option<NaiveDate>,
    end_on: Option<NaiveDate>,
    deadline_on: Option<NaiveDate>,
    contract_pages: Option<i64>,
    contract_files: Option<i64>,
    manager_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CountsDto {
    files: i64,
    pages: i64,
    pdfa_files: i64,
    signed_files: i64,
    approved_files: i64,
    rejected_files: i64,
    pending_review_files: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DayMetrics {
    day: NaiveDate,
    #[serde(flatten)]
    m: Metrics,
}

/// Sức khoẻ một dự án từ dòng tiến độ + lịch sử ngày của nó.
fn health_of(r: &progress::ProgressRow, days: &BTreeMap<NaiveDate, Metrics>, today: NaiveDate, rag: &RagThresholds) -> Health {
    calc::health(
        &HealthInput {
            today,
            signed_on: r.signed_on,
            end_on: r.end_on,
            deadline_on: r.deadline_on,
            contract_pages: r.contract_pages,
            contract_files: r.contract_files,
            pages: r.pages,
            files: r.files,
            pdfa_files: r.pdfa_files,
            signed_files: r.signed_files,
            approved_files: r.approved_files,
            rate7d: rate7d(days, today),
        },
        rag,
    )
}

// ---------------------------------------------------------------------------------------------------------------
// GET /dashboard/overview — lãnh đạo / quản lý nhiều dự án
// ---------------------------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OverviewQuery {
    days: Option<i64>,
    project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OverviewProject {
    project_id: Uuid,
    code: String,
    name: String,
    contract: ContractDto,
    counts: CountsDto,
    health: Health,
    trend: Vec<DayMetrics>,
    issues: db::ProjectIssues,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Risk {
    project_id: Option<Uuid>,
    project_code: Option<String>,
    kind: &'static str,
    label: String,
    value: i64,
    severity: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OverviewTotals {
    projects: i64,
    late: i64,
    at_risk: i64,
    pages7d: i64,
    pages7d_prev: i64,
    submitted_files: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Overview {
    generated_at: DateTime<Utc>,
    today: NaiveDate,
    days: i64,
    totals: OverviewTotals,
    projects: Vec<OverviewProject>,
    throughput: Vec<DayMetrics>,
    risks: Vec<Risk>,
}

async fn overview(State(st): State<AppState>, ts: Ts, Query(q): Query<OverviewQuery>) -> Result<Json<Overview>, ApiError> {
    let projects = scoped_projects(&st, &ts, q.project_id).await?;
    let days = clamp_days(q.days, 30);
    let key = format!("overview|{}|{days}", scope_key(&projects));
    if let Some(v) = cache_get::<Overview>(&key) {
        return Ok(Json(v));
    }
    let today = calc::vn_today();
    // Lịch sử kéo dài thêm 14 ngày để tính "so tuần trước" khi days nhỏ.
    let from = calc::days_back(today, days.max(14));
    let rows = progress::project_progress(&st.pool, projects.as_deref(), None, None).await?;
    let ids: Vec<Uuid> = rows.iter().map(|r| r.project_id).collect();
    let daily = db::project_daily(&st.pool, Some(&ids), from, today).await?;
    let settings: HashMap<Uuid, KpiSettingsRow> = db::kpi_settings_many(&st.pool, &ids).await?.into_iter().map(|s| (s.project_id, s)).collect();
    let issues: HashMap<Uuid, db::ProjectIssues> = db::issues_by_project(&st.pool, Some(&ids)).await?.into_iter().map(|i| (i.project_id, i)).collect();

    let mut by_project: HashMap<Uuid, BTreeMap<NaiveDate, Metrics>> = HashMap::new();
    let mut total_by_day: BTreeMap<NaiveDate, Metrics> = BTreeMap::new();
    for d in &daily {
        by_project.entry(d.project_id).or_default().insert(d.day, d.m);
        total_by_day.entry(d.day).or_default().add(&d.m);
    }
    let trend_from = calc::days_back(today, days);
    let mut out: Vec<OverviewProject> = Vec::with_capacity(rows.len());
    let mut risks: Vec<Risk> = Vec::new();
    for r in &rows {
        let dmap = by_project.remove(&r.project_id).unwrap_or_default();
        let s = settings.get(&r.project_id).cloned().unwrap_or_else(|| KpiSettingsRow::default_for(r.project_id));
        let (rag_t, _) = thresholds_of(&s);
        let health = health_of(r, &dmap, today, &rag_t);
        let iss = issues.get(&r.project_id).cloned().unwrap_or_default();
        if health.rag == calc::Rag::Red {
            risks.push(Risk { project_id: Some(r.project_id), project_code: Some(r.code.clone()), kind: "late", label: format!("{} chậm tiến độ", r.code), value: health.lech.map(|l| l.round() as i64).unwrap_or(0), severity: "err" });
        }
        if iss.ocr_failed_pages > 0 {
            risks.push(Risk { project_id: Some(r.project_id), project_code: Some(r.code.clone()), kind: "ocr_failed", label: format!("{} có trang OCR lỗi", r.code), value: iss.ocr_failed_pages, severity: "warn" });
        }
        if iss.overdue_packages > 0 {
            risks.push(Risk { project_id: Some(r.project_id), project_code: Some(r.code.clone()), kind: "overdue_work", label: format!("{} có gói việc quá hạn", r.code), value: iss.overdue_packages, severity: "err" });
        }
        out.push(OverviewProject {
            project_id: r.project_id,
            code: r.code.clone(),
            name: r.name.clone(),
            contract: ContractDto { signed_on: r.signed_on, end_on: r.end_on, deadline_on: r.deadline_on, contract_pages: r.contract_pages, contract_files: r.contract_files, manager_name: r.manager_name.clone() },
            counts: CountsDto {
                files: r.files,
                pages: r.pages,
                pdfa_files: r.pdfa_files,
                signed_files: r.signed_files,
                approved_files: r.approved_files,
                rejected_files: r.rejected_files,
                pending_review_files: (r.ocr_files - r.approved_files - r.rejected_files).max(0),
            },
            health,
            trend: dmap.range(trend_from..=today).map(|(d, m)| DayMetrics { day: *d, m: *m }).collect(),
            issues: iss,
        });
    }
    // Đỏ trước, rồi vàng, xám, xanh; cùng màu thì lệch âm nhiều hơn lên trước.
    let rank = |h: &Health| match h.rag {
        calc::Rag::Red => 0,
        calc::Rag::Amber => 1,
        calc::Rag::Grey => 2,
        calc::Rag::Green => 3,
    };
    out.sort_by(|a, b| rank(&a.health).cmp(&rank(&b.health)).then(a.health.lech.unwrap_or(0.0).partial_cmp(&b.health.lech.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal)));
    risks.sort_by(|a, b| (a.severity == "warn").cmp(&(b.severity == "warn")).then(b.value.cmp(&a.value)));
    risks.truncate(8);

    let sum_pages = |from: NaiveDate, to: NaiveDate| total_by_day.range(from..=to).map(|(_, m)| m.pages_scanned).sum::<i64>();
    let w0 = calc::days_back(today, 7);
    let w1 = calc::days_back(today, 14);
    let totals = OverviewTotals {
        projects: out.len() as i64,
        late: out.iter().filter(|p| p.health.rag == calc::Rag::Red).count() as i64,
        at_risk: out.iter().filter(|p| p.health.rag == calc::Rag::Amber).count() as i64,
        pages7d: sum_pages(w0, today),
        pages7d_prev: sum_pages(w1, w0 - chrono::Duration::days(1)),
        submitted_files: out.iter().map(|p| p.counts.signed_files).sum(),
    };
    let v = Overview {
        generated_at: Utc::now(),
        today,
        days,
        totals,
        projects: out,
        throughput: total_by_day.range(trend_from..=today).map(|(d, m)| DayMetrics { day: *d, m: *m }).collect(),
        risks,
    };
    cache_put(key, &v);
    Ok(Json(v))
}

// ---------------------------------------------------------------------------------------------------------------
// GET /dashboard/project — một dự án: dây chuyền, chuỗi ngày, tồn đọng, SLA gói việc, nhân viên so định mức
// ---------------------------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectQuery {
    project_id: Uuid,
    days: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BacklogStage {
    stage: &'static str,
    label: &'static str,
    /// 4 nhóm tuổi: 0–3 · 4–7 · 8–14 · >14 ngày.
    buckets: [i64; 4],
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct KpiLine {
    metric: String,
    quota_per_day: Option<f64>,
    me_avg_per_active_day: Option<f64>,
    attainment_pct: Option<f64>,
    error_pct: f64,
    rating: Rating,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StaffDto {
    user_id: Uuid,
    username: String,
    display_name: String,
    totals: Metrics,
    active_days: i64,
    today: Metrics,
    series: Vec<DayMetrics>,
    kpi: Vec<KpiLine>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct KpiSettingsDto {
    project_id: Uuid,
    quotas: serde_json::Value,
    excellent_pct: f64,
    pass_pct: f64,
    excellent_err_pct: f64,
    pass_err_pct: f64,
    rag_amber_lech: f64,
    rag_red_lech: f64,
    review_backlog_warn_days: i32,
    work_item_sla_hours: i32,
    group_min_users: i32,
    updated_at: Option<DateTime<Utc>>,
    updated_by: Option<Uuid>,
}
impl From<KpiSettingsRow> for KpiSettingsDto {
    fn from(s: KpiSettingsRow) -> Self {
        KpiSettingsDto {
            project_id: s.project_id,
            quotas: s.quotas,
            excellent_pct: s.excellent_pct,
            pass_pct: s.pass_pct,
            excellent_err_pct: s.excellent_err_pct,
            pass_err_pct: s.pass_err_pct,
            rag_amber_lech: s.rag_amber_lech,
            rag_red_lech: s.rag_red_lech,
            review_backlog_warn_days: s.review_backlog_warn_days,
            work_item_sla_hours: s.work_item_sla_hours,
            group_min_users: s.group_min_users,
            updated_at: s.updated_at,
            updated_by: s.updated_by,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectDash {
    project_id: Uuid,
    code: String,
    name: String,
    today: NaiveDate,
    days: i64,
    contract: ContractDto,
    counts: CountsDto,
    health: Health,
    pipeline: db::PipelineCounts,
    series: Vec<DayMetrics>,
    backlog: Vec<BacklogStage>,
    work_sla: db::WorkSla,
    overdue_packages: Vec<db::OverduePackage>,
    staff: Vec<StaffDto>,
    kpi: KpiSettingsDto,
}

/// Tỉ lệ lỗi của một người: mục bị trả + kiểm lỗi trên mục đã làm; tài liệu bị từ chối trên tài liệu đã đưa lên.
fn error_pct_of(t: &Metrics) -> f64 {
    let den = t.work_items_done + t.docs_uploaded;
    if den == 0 {
        return 0.0;
    }
    let num = t.work_items_rejected + t.work_items_checked_err + t.docs_reviewed_rejected.min(t.docs_uploaded);
    ((num as f64) / (den as f64) * 1000.0).round() / 10.0
}

/// Gom dòng người × ngày thành bảng nhân viên (tổng, ngày hoạt động, hôm nay, chuỗi, KPI theo định mức).
fn build_staff(rows: &[db::UserDay], names: &[db::UserName], settings: &KpiSettingsRow, today: NaiveDate, series_from: NaiveDate) -> Vec<StaffDto> {
    let (_, kpi_t) = thresholds_of(settings);
    let name_of: HashMap<Uuid, &db::UserName> = names.iter().map(|n| (n.id, n)).collect();
    let mut by_user: BTreeMap<Uuid, Vec<&db::UserDay>> = BTreeMap::new();
    for r in rows {
        by_user.entry(r.user_id).or_default().push(r);
    }
    let mut out: Vec<StaffDto> = by_user
        .into_iter()
        .map(|(uid, days)| {
            let mut totals = Metrics::default();
            let mut today_m = Metrics::default();
            for d in &days {
                totals.add(&d.m);
                if d.day == today {
                    today_m = d.m;
                }
            }
            let active_days = days.len() as i64;
            let err = error_pct_of(&totals);
            let kpi = calc::QUOTA_METRICS
                .iter()
                .filter_map(|m| {
                    let quota = settings.quota(m);
                    let done = totals.get(m);
                    if quota.is_none() && done == 0 {
                        return None;
                    }
                    let avg = (active_days > 0).then(|| (done as f64 / active_days as f64 * 10.0).round() / 10.0);
                    let att = calc::attainment_pct(done as f64, active_days, quota);
                    Some(KpiLine { metric: (*m).to_string(), quota_per_day: quota, me_avg_per_active_day: avg, attainment_pct: att, error_pct: err, rating: calc::rating(att, err, &kpi_t) })
                })
                .collect();
            let n = name_of.get(&uid);
            StaffDto {
                user_id: uid,
                username: n.map(|x| x.username.clone()).unwrap_or_default(),
                display_name: n.map(|x| x.display_name.clone()).unwrap_or_else(|| "Tài khoản đã xoá".into()),
                totals,
                active_days,
                today: today_m,
                series: days.iter().filter(|d| d.day >= series_from).map(|d| DayMetrics { day: d.day, m: d.m }).collect(),
                kpi,
            }
        })
        .collect();
    out.sort_by(|a, b| b.today.pages_scanned.cmp(&a.today.pages_scanned).then_with(|| b.totals.pages_scanned.cmp(&a.totals.pages_scanned)).then_with(|| a.display_name.cmp(&b.display_name)));
    out
}

fn backlog_stages(rows: &[db::BacklogRow]) -> Vec<BacklogStage> {
    let defs: [(&'static str, &'static str); 5] = [("ocr", "Chờ OCR"), ("review", "Chờ duyệt"), ("sign", "Chờ ký"), ("package", "Chờ đóng gói"), ("submit", "Chờ nộp")];
    defs.iter()
        .map(|(stage, label)| {
            let mut b = [0i64; 4];
            for r in rows.iter().filter(|r| r.stage == *stage) {
                if (0..4).contains(&r.bucket) {
                    b[r.bucket as usize] = r.docs;
                }
            }
            BacklogStage { stage, label, buckets: b }
        })
        .collect()
}

async fn project(State(st): State<AppState>, ts: Ts, Query(q): Query<ProjectQuery>) -> Result<Json<ProjectDash>, ApiError> {
    scoped_projects(&st, &ts, Some(q.project_id)).await?;
    let days = clamp_days(q.days, 30);
    let key = format!("project|{}|{days}", q.project_id);
    if let Some(v) = cache_get::<ProjectDash>(&key) {
        return Ok(Json(v));
    }
    let today = calc::vn_today();
    let from = calc::days_back(today, days.max(14));
    let series_from = calc::days_back(today, days);
    let ids = [q.project_id];
    let row = progress::project_progress(&st.pool, Some(&ids), None, None).await?.into_iter().next().ok_or_else(ApiError::not_found)?;
    let settings = db::get_kpi_settings(&st.pool, q.project_id).await?;
    let (rag_t, _) = thresholds_of(&settings);
    let daily = db::project_daily(&st.pool, Some(&ids), from, today).await?;
    let dmap: BTreeMap<NaiveDate, Metrics> = daily.iter().map(|d| (d.day, d.m)).collect();
    let health = health_of(&row, &dmap, today, &rag_t);
    let pipeline = db::pipeline_counts(&st.pool, q.project_id).await?;
    let backlog = backlog_stages(&db::backlog_buckets(&st.pool, q.project_id).await?);
    let (work_sla, overdue_packages) = db::work_sla(&st.pool, q.project_id, settings.work_item_sla_hours).await?;
    let user_rows = db::user_daily(&st.pool, q.project_id, None, from, today).await?;
    let uids: Vec<Uuid> = {
        let mut v: Vec<Uuid> = user_rows.iter().map(|r| r.user_id).collect();
        v.sort();
        v.dedup();
        v
    };
    let names = db::user_names(&st.pool, &uids).await?;
    let staff = build_staff(&user_rows, &names, &settings, today, series_from);
    let v = ProjectDash {
        project_id: row.project_id,
        code: row.code.clone(),
        name: row.name.clone(),
        today,
        days,
        contract: ContractDto { signed_on: row.signed_on, end_on: row.end_on, deadline_on: row.deadline_on, contract_pages: row.contract_pages, contract_files: row.contract_files, manager_name: row.manager_name.clone() },
        counts: CountsDto {
            files: row.files,
            pages: row.pages,
            pdfa_files: row.pdfa_files,
            signed_files: row.signed_files,
            approved_files: row.approved_files,
            rejected_files: row.rejected_files,
            pending_review_files: (row.ocr_files - row.approved_files - row.rejected_files).max(0),
        },
        health,
        pipeline,
        series: dmap.range(series_from..=today).map(|(d, m)| DayMetrics { day: *d, m: *m }).collect(),
        backlog,
        work_sla,
        overdue_packages,
        staff,
        kpi: settings.into(),
    };
    cache_put(key, &v);
    Ok(Json(v))
}

// ---------------------------------------------------------------------------------------------------------------
// GET /dashboard/staff — bảng nhân viên của một dự án theo khoảng ngày (quản lý)
// ---------------------------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StaffQuery {
    project_id: Uuid,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StaffBoard {
    project_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
    users: Vec<StaffDto>,
    kpi: KpiSettingsDto,
}

async fn staff(State(st): State<AppState>, ts: Ts, Query(q): Query<StaffQuery>) -> Result<Json<StaffBoard>, ApiError> {
    scoped_projects(&st, &ts, Some(q.project_id)).await?;
    let today = calc::vn_today();
    let to = q.to.unwrap_or(today).min(today);
    let from = q.from.unwrap_or_else(|| calc::days_back(to, 30));
    if from > to {
        return Err(ApiError::bad_request("từ ngày phải trước đến ngày"));
    }
    if (to - from).num_days() > 400 {
        return Err(ApiError::bad_request("khoảng ngày tối đa 400 ngày"));
    }
    let settings = db::get_kpi_settings(&st.pool, q.project_id).await?;
    let rows = db::user_daily(&st.pool, q.project_id, None, from, to).await?;
    let uids: Vec<Uuid> = {
        let mut v: Vec<Uuid> = rows.iter().map(|r| r.user_id).collect();
        v.sort();
        v.dedup();
        v
    };
    let names = db::user_names(&st.pool, &uids).await?;
    let users = build_staff(&rows, &names, &settings, today, from);
    Ok(Json(StaffBoard { project_id: q.project_id, from, to, users, kpi: settings.into() }))
}

// ---------------------------------------------------------------------------------------------------------------
// GET /dashboard/me — nhân viên: chỉ số của MÌNH + trung bình nhóm ẩn danh (ẩn khi nhóm < group_min_users)
// ---------------------------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MeQuery {
    project_id: Option<Uuid>,
    days: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MeDay {
    day: NaiveDate,
    me: Metrics,
    /// Trung bình mỗi người có hoạt động trong ngày; None khi chưa đủ người.
    group_avg: Option<MetricsF>,
    active_users: i64,
}

/// Bản số thực của Metrics (trung bình nhóm).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct MetricsF {
    pages_scanned: f64,
    docs_uploaded: f64,
    fields_edited: f64,
    fields_verified: f64,
    work_items_done: f64,
    docs_reviewed_approved: f64,
    docs_signed: f64,
    cl_ho_so_new: f64,
    cl_van_ban_new: f64,
    cl_fields_entered: f64,
}
impl MetricsF {
    fn avg(sum: &Metrics, n: i64) -> Self {
        let n = n.max(1) as f64;
        let f = |v: i64| ((v as f64) / n * 10.0).round() / 10.0;
        MetricsF {
            pages_scanned: f(sum.pages_scanned),
            docs_uploaded: f(sum.docs_uploaded),
            fields_edited: f(sum.fields_edited),
            fields_verified: f(sum.fields_verified),
            work_items_done: f(sum.work_items_done),
            docs_reviewed_approved: f(sum.docs_reviewed_approved),
            docs_signed: f(sum.docs_signed),
            cl_ho_so_new: f(sum.cl_ho_so_new),
            cl_van_ban_new: f(sum.cl_van_ban_new),
            cl_fields_entered: f(sum.cl_fields_entered),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MeDash {
    user_id: Uuid,
    project_id: Uuid,
    project_code: String,
    project_name: String,
    today: NaiveDate,
    days: i64,
    series: Vec<MeDay>,
    totals: Metrics,
    active_days: i64,
    /// Trung bình MỖI NGƯỜI MỖI NGÀY HOẠT ĐỘNG của cả nhóm trong kỳ; None khi nhóm chưa đủ người.
    group_avg: Option<MetricsF>,
    group_users: i64,
    quotas: serde_json::Value,
    kpi: Vec<KpiLine>,
}

async fn me(State(st): State<AppState>, ts: Ts, Query(q): Query<MeQuery>) -> Result<Json<MeDash>, ApiError> {
    ts.require_perm(Permission::StatsView)?;
    let me = ts.user.id;
    let project = match q.project_id.or(ts.user.project_id) {
        Some(p) => p,
        None => db::first_project_of(&st.pool, me).await?.ok_or_else(|| ApiError::bad_request("tài khoản chưa thuộc dự án nào"))?,
    };
    if !db::is_member(&st.pool, project, me).await? {
        return Err(ApiError::forbidden());
    }
    let days = clamp_days(q.days, 30);
    let today = calc::vn_today();
    let from = calc::days_back(today, days);
    let proj = digitext_db::projects::get(&st.pool, project).await?.ok_or_else(ApiError::not_found)?;
    let settings = db::get_kpi_settings(&st.pool, project).await?;
    let (_, kpi_t) = thresholds_of(&settings);
    let k = settings.group_min_users.max(2) as i64;
    let mine: BTreeMap<NaiveDate, Metrics> = db::user_daily(&st.pool, project, Some(me), from, today).await?.into_iter().map(|r| (r.day, r.m)).collect();
    let group = db::group_daily(&st.pool, project, from, today).await?;
    let mut series: Vec<MeDay> = Vec::new();
    let mut d = from;
    let mut gsum = Metrics::default();
    let mut g_user_days: i64 = 0;
    let mut g_users: std::collections::BTreeSet<NaiveDate> = Default::default();
    let mut max_active = 0i64;
    while d <= today {
        let g = group.iter().find(|g| g.day == d);
        let active = g.map(|g| g.active_users).unwrap_or(0);
        max_active = max_active.max(active);
        if let Some(g) = g {
            gsum.add(&g.sum);
            g_user_days += g.active_users;
            g_users.insert(d);
        }
        series.push(MeDay {
            day: d,
            me: mine.get(&d).copied().unwrap_or_default(),
            group_avg: g.filter(|g| g.active_users >= k).map(|g| MetricsF::avg(&g.sum, g.active_users)),
            active_users: active,
        });
        d = d.succ_opt().unwrap_or(today);
        if d == today && series.len() > 400 {
            break;
        }
    }
    let mut totals = Metrics::default();
    for m in mine.values() {
        totals.add(m);
    }
    let active_days = mine.len() as i64;
    let err = error_pct_of(&totals);
    let kpi = calc::QUOTA_METRICS
        .iter()
        .filter_map(|m| {
            let quota = settings.quota(m);
            let done = totals.get(m);
            if quota.is_none() && done == 0 {
                return None;
            }
            let avg = (active_days > 0).then(|| (done as f64 / active_days as f64 * 10.0).round() / 10.0);
            let att = calc::attainment_pct(done as f64, active_days, quota);
            Some(KpiLine { metric: (*m).to_string(), quota_per_day: quota, me_avg_per_active_day: avg, attainment_pct: att, error_pct: err, rating: calc::rating(att, err, &kpi_t) })
        })
        .collect();
    let v = MeDash {
        user_id: me,
        project_id: project,
        project_code: proj.code,
        project_name: proj.name,
        today,
        days,
        series,
        totals,
        active_days,
        group_avg: (max_active >= k && g_user_days > 0).then(|| MetricsF::avg(&gsum, g_user_days)),
        group_users: max_active,
        quotas: settings.quotas.clone(),
        kpi,
    };
    Ok(Json(v))
}

// ---------------------------------------------------------------------------------------------------------------
// Cài đặt KPI: GET (StatsViewProject) / PUT (ProjectUpdate)
// ---------------------------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KpiQuery {
    project_id: Uuid,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KpiPut {
    #[serde(default)]
    quotas: serde_json::Value,
    excellent_pct: f64,
    pass_pct: f64,
    excellent_err_pct: f64,
    pass_err_pct: f64,
    rag_amber_lech: f64,
    rag_red_lech: f64,
    review_backlog_warn_days: i32,
    work_item_sla_hours: i32,
    group_min_users: i32,
}

async fn kpi_get(State(st): State<AppState>, ts: Ts, Query(q): Query<KpiQuery>) -> Result<Json<KpiSettingsDto>, ApiError> {
    scoped_projects(&st, &ts, Some(q.project_id)).await?;
    Ok(Json(db::get_kpi_settings(&st.pool, q.project_id).await?.into()))
}

async fn kpi_put(State(st): State<AppState>, ts: Ts, Query(q): Query<KpiQuery>, Json(b): Json<KpiPut>) -> Result<Json<KpiSettingsDto>, ApiError> {
    ts.require_perm(Permission::ProjectUpdate)?;
    if !f_29a(&st.pool, &ts.user).await?.allows_project(q.project_id) {
        return Err(ApiError::forbidden());
    }
    let quotas = match &b.quotas {
        serde_json::Value::Null => serde_json::json!({}),
        serde_json::Value::Object(m) => {
            let mut out = serde_json::Map::new();
            for (k, v) in m {
                if !calc::is_quota_metric(k) {
                    return Err(ApiError::bad_request(format!("chỉ số định mức không hợp lệ: {k}")));
                }
                match v.as_f64() {
                    Some(x) if x > 0.0 && x.is_finite() => {
                        out.insert(k.clone(), serde_json::json!(x));
                    }
                    Some(x) if x == 0.0 => {}
                    None if v.is_null() => {}
                    _ => return Err(ApiError::bad_request(format!("định mức {k} phải là số không âm"))),
                }
            }
            serde_json::Value::Object(out)
        }
        _ => return Err(ApiError::bad_request("quotas phải là đối tượng {chỉ số: định mức/ngày}")),
    };
    let pct_ok = |v: f64| (0.0..=1000.0).contains(&v) && v.is_finite();
    if ![b.excellent_pct, b.pass_pct, b.excellent_err_pct, b.pass_err_pct].iter().all(|v| pct_ok(*v)) || b.pass_pct > b.excellent_pct || b.excellent_err_pct > b.pass_err_pct {
        return Err(ApiError::bad_request("ngưỡng KPI: Đạt ≤ Xuất sắc về hoàn thành và Xuất sắc ≤ Đạt về lỗi, trong 0–1000 %"));
    }
    if !(b.rag_red_lech < b.rag_amber_lech && b.rag_amber_lech <= 0.0 && b.rag_red_lech >= -100.0) {
        return Err(ApiError::bad_request("ngưỡng tiến độ: đỏ < vàng ≤ 0 (điểm lệch)"));
    }
    if !(1..=365).contains(&b.review_backlog_warn_days) || !(1..=24 * 90).contains(&b.work_item_sla_hours) || !(2..=100).contains(&b.group_min_users) {
        return Err(ApiError::bad_request("ngày cảnh báo tồn đọng 1–365, SLA 1–2160 giờ, số người tối thiểu 2–100"));
    }
    let row = KpiSettingsRow {
        project_id: q.project_id,
        quotas,
        excellent_pct: b.excellent_pct,
        pass_pct: b.pass_pct,
        excellent_err_pct: b.excellent_err_pct,
        pass_err_pct: b.pass_err_pct,
        rag_amber_lech: b.rag_amber_lech,
        rag_red_lech: b.rag_red_lech,
        review_backlog_warn_days: b.review_backlog_warn_days,
        work_item_sla_hours: b.work_item_sla_hours,
        group_min_users: b.group_min_users,
        updated_by: Some(ts.user.id),
        updated_at: None,
    };
    let saved = db::upsert_kpi_settings(&st.pool, &row, ts.user.id).await?;
    digitext_db::audit::append(
        &st.pool,
        digitext_db::audit::Tp {
            actor_id: Some(ts.user.id),
            device_id: ts.claims.device_id,
            action: "dashboard.kpi_update",
            target: Some(format!("project:{}", q.project_id)),
            detail: Some(serde_json::json!({ "quotas": saved.quotas, "excellentPct": saved.excellent_pct, "passPct": saved.pass_pct })),
            ip: ts.ip(),
        },
    )
    .await?;
    cache_clear();
    let _ = digitext_db::changes::note(&st.pool, "dashboard").await;
    Ok(Json(saved.into()))
}

// ---------------------------------------------------------------------------------------------------------------
// POST /dashboard/rollup?from&to — super_admin: tổng hợp lại một khoảng (sửa chữa / backfill)
// ---------------------------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RollupQuery {
    from: NaiveDate,
    to: NaiveDate,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RollupResp {
    from: NaiveDate,
    to: NaiveDate,
    rows: i64,
}

async fn rollup(State(st): State<AppState>, ts: Ts, Query(q): Query<RollupQuery>) -> Result<Json<RollupResp>, ApiError> {
    ts.require(Perm::Super)?;
    if q.to < q.from || (q.to - q.from).num_days() > 400 {
        return Err(ApiError::bad_request("khoảng tổng hợp tối đa 400 ngày và từ ≤ đến"));
    }
    let rows = db::rollup(&st.pool, q.from, q.to).await?;
    cache_clear();
    let _ = digitext_db::changes::note(&st.pool, "dashboard").await;
    Ok(Json(RollupResp { from: q.from, to: q.to, rows }))
}

/// Vòng tổng hợp định kỳ — cli.rs gọi khi khởi động API. Mỗi 2 phút: hôm qua + hôm nay (bù ngày thiếu khi máy tắt);
/// 00:15 giờ Việt Nam: 35 ngày gần nhất (bắt các mốc ghi đè như duyệt lại, sửa trường).
pub fn spawn_rollup(pool: sqlx::PgPool) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(120));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut last_nightly: Option<NaiveDate> = None;
        let mut first = true;
        loop {
            tick.tick().await;
            let today = calc::vn_today();
            let yesterday = today.pred_opt().unwrap_or(today);
            let mut from = yesterday;
            if first {
                first = false;
                match db::max_day(&pool).await {
                    Ok(Some(d)) if d < yesterday => from = d,
                    Ok(None) => from = calc::days_back(today, 400),
                    _ => {}
                }
            }
            let now_vn = Utc::now().with_timezone(&chrono::FixedOffset::east_opt(calc::VN_OFFSET_SECS).expect("+07"));
            let nightly_due = last_nightly != Some(today) && chrono::Timelike::hour(&now_vn) == 0 && chrono::Timelike::minute(&now_vn) >= 15;
            if nightly_due {
                from = from.min(calc::days_back(today, 35));
            }
            match db::rollup(&pool, from, today).await {
                Ok(n) => {
                    if nightly_due {
                        last_nightly = Some(today);
                    }
                    if n > 0 {
                        cache_clear();
                        let _ = digitext_db::changes::note(&pool, "dashboard").await;
                    }
                    tracing::debug!(from = %from, to = %today, rows = n, "dashboard: tổng hợp theo ngày");
                }
                Err(e) => tracing::warn!(error = %e, "dashboard: tổng hợp theo ngày lỗi (sẽ thử lại)"),
            }
        }
    });
}
