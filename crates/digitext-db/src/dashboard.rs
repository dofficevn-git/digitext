// Copyright (c) 2026 DigiText. All rights reserved.
//! Dashboard 3 góc nhìn (migration 0021): đọc bảng tổng hợp theo ngày `dashboard_daily`, số đếm dây chuyền theo dự
//! án, tồn đọng theo tuổi, SLA gói việc, cài đặt KPI. Mọi "ngày" là ngày Việt Nam (xem `dashboard_rollup`).
use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// user_id của dòng TỔNG DỰ ÁN trong dashboard_daily.
pub const PROJECT_ROW: Uuid = Uuid::nil();

/// Các cột đo của dashboard_daily — đúng thứ tự khai báo trong migration 0021.
pub const METRIC_COLS: [&str; 20] = [
    "pages_scanned",
    "docs_uploaded",
    "pages_ocr_done",
    "docs_pdfa",
    "docs_signed",
    "docs_reviewed_approved",
    "docs_reviewed_rejected",
    "fields_edited",
    "fields_verified",
    "work_items_done",
    "work_items_checked_ok",
    "work_items_checked_err",
    "work_items_rejected",
    "cl_ho_so_new",
    "cl_ho_so_edited",
    "cl_van_ban_new",
    "cl_fields_entered",
    "packages_built",
    "submissions_created",
    "submissions_completed",
];

/// Một bộ số đo (một ngày × một dự án × một người, hoặc tổng của nhiều dòng).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub pages_scanned: i64,
    pub docs_uploaded: i64,
    pub pages_ocr_done: i64,
    pub docs_pdfa: i64,
    pub docs_signed: i64,
    pub docs_reviewed_approved: i64,
    pub docs_reviewed_rejected: i64,
    pub fields_edited: i64,
    pub fields_verified: i64,
    pub work_items_done: i64,
    pub work_items_checked_ok: i64,
    pub work_items_checked_err: i64,
    pub work_items_rejected: i64,
    pub cl_ho_so_new: i64,
    pub cl_ho_so_edited: i64,
    pub cl_van_ban_new: i64,
    pub cl_fields_entered: i64,
    pub packages_built: i64,
    pub submissions_created: i64,
    pub submissions_completed: i64,
}
impl Metrics {
    pub fn get(&self, metric: &str) -> i64 {
        match metric {
            "pages_scanned" => self.pages_scanned,
            "docs_uploaded" => self.docs_uploaded,
            "pages_ocr_done" => self.pages_ocr_done,
            "docs_pdfa" => self.docs_pdfa,
            "docs_signed" => self.docs_signed,
            "docs_reviewed_approved" => self.docs_reviewed_approved,
            "docs_reviewed_rejected" => self.docs_reviewed_rejected,
            "fields_edited" => self.fields_edited,
            "fields_verified" => self.fields_verified,
            "work_items_done" => self.work_items_done,
            "work_items_checked_ok" => self.work_items_checked_ok,
            "work_items_checked_err" => self.work_items_checked_err,
            "work_items_rejected" => self.work_items_rejected,
            "cl_ho_so_new" => self.cl_ho_so_new,
            "cl_ho_so_edited" => self.cl_ho_so_edited,
            "cl_van_ban_new" => self.cl_van_ban_new,
            "cl_fields_entered" => self.cl_fields_entered,
            "packages_built" => self.packages_built,
            "submissions_created" => self.submissions_created,
            "submissions_completed" => self.submissions_completed,
            _ => 0,
        }
    }
    pub fn add(&mut self, o: &Metrics) {
        self.pages_scanned += o.pages_scanned;
        self.docs_uploaded += o.docs_uploaded;
        self.pages_ocr_done += o.pages_ocr_done;
        self.docs_pdfa += o.docs_pdfa;
        self.docs_signed += o.docs_signed;
        self.docs_reviewed_approved += o.docs_reviewed_approved;
        self.docs_reviewed_rejected += o.docs_reviewed_rejected;
        self.fields_edited += o.fields_edited;
        self.fields_verified += o.fields_verified;
        self.work_items_done += o.work_items_done;
        self.work_items_checked_ok += o.work_items_checked_ok;
        self.work_items_checked_err += o.work_items_checked_err;
        self.work_items_rejected += o.work_items_rejected;
        self.cl_ho_so_new += o.cl_ho_so_new;
        self.cl_ho_so_edited += o.cl_ho_so_edited;
        self.cl_van_ban_new += o.cl_van_ban_new;
        self.cl_fields_entered += o.cl_fields_entered;
        self.packages_built += o.packages_built;
        self.submissions_created += o.submissions_created;
        self.submissions_completed += o.submissions_completed;
    }
    pub fn is_zero(&self) -> bool {
        METRIC_COLS.iter().all(|m| self.get(m) == 0)
    }
}

fn cols(prefix: &str) -> String {
    METRIC_COLS.iter().map(|c| format!("{prefix}{c}")).collect::<Vec<_>>().join(", ")
}
fn sum_cols() -> String {
    METRIC_COLS.iter().map(|c| format!("COALESCE(sum({c}), 0)::bigint AS {c}")).collect::<Vec<_>>().join(", ")
}

/// Tổng hợp lại các ngày trong khoảng (cả hai đầu) — gọi hàm SQL `dashboard_rollup`. Trả số dòng đã chạm.
pub async fn rollup(pool: &PgPool, from: NaiveDate, to: NaiveDate) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>("SELECT dashboard_rollup($1, $2)")
        .bind(from)
        .bind(to)
        .fetch_one(pool)
        .await
}

/// Ngày mới nhất đã có trong bảng tổng hợp (None = chưa backfill).
pub async fn max_day(pool: &PgPool) -> sqlx::Result<Option<NaiveDate>> {
    sqlx::query_scalar::<_, Option<NaiveDate>>("SELECT max(day) FROM dashboard_daily")
        .fetch_one(pool)
        .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProjectDay {
    pub project_id: Uuid,
    pub day: NaiveDate,
    #[sqlx(flatten)]
    pub m: Metrics,
}

/// Dòng tổng dự án theo ngày cho một hoặc nhiều dự án (None = mọi dự án).
pub async fn project_daily(pool: &PgPool, projects: Option<&[Uuid]>, from: NaiveDate, to: NaiveDate) -> sqlx::Result<Vec<ProjectDay>> {
    let projects: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    sqlx::query_as::<_, ProjectDay>(&format!(
        "SELECT project_id, day, {} FROM dashboard_daily \
         WHERE user_id = $1 AND day BETWEEN $2 AND $3 AND ($4::uuid[] IS NULL OR project_id = ANY($4)) \
         ORDER BY project_id, day",
        cols("")
    ))
    .bind(PROJECT_ROW)
    .bind(from)
    .bind(to)
    .bind(projects)
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserDay {
    pub user_id: Uuid,
    pub day: NaiveDate,
    #[sqlx(flatten)]
    pub m: Metrics,
}

/// Dòng theo người × ngày của một dự án (mọi người, hoặc một người khi `user` có giá trị).
pub async fn user_daily(pool: &PgPool, project: Uuid, user: Option<Uuid>, from: NaiveDate, to: NaiveDate) -> sqlx::Result<Vec<UserDay>> {
    sqlx::query_as::<_, UserDay>(&format!(
        "SELECT user_id, day, {} FROM dashboard_daily \
         WHERE project_id = $1 AND user_id <> $2 AND ($3::uuid IS NULL OR user_id = $3) AND day BETWEEN $4 AND $5 \
         ORDER BY user_id, day",
        cols("")
    ))
    .bind(project)
    .bind(PROJECT_ROW)
    .bind(user)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GroupDay {
    pub day: NaiveDate,
    /// Số người có hoạt động trong ngày (dòng người chỉ tồn tại khi khác 0).
    pub active_users: i64,
    #[sqlx(flatten)]
    pub sum: Metrics,
}

/// Tổng theo ngày của MỌI người trong dự án + số người hoạt động — để tính trung bình nhóm ẩn danh.
pub async fn group_daily(pool: &PgPool, project: Uuid, from: NaiveDate, to: NaiveDate) -> sqlx::Result<Vec<GroupDay>> {
    sqlx::query_as::<_, GroupDay>(&format!(
        "SELECT day, count(*)::bigint AS active_users, {} FROM dashboard_daily \
         WHERE project_id = $1 AND user_id <> $2 AND day BETWEEN $3 AND $4 GROUP BY day ORDER BY day",
        sum_cols()
    ))
    .bind(project)
    .bind(PROJECT_ROW)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserName {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
}
pub async fn user_names(pool: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<UserName>> {
    sqlx::query_as::<_, UserName>("SELECT id, username, display_name FROM users WHERE id = ANY($1)")
        .bind(ids)
        .fetch_all(pool)
        .await
}

/// Dự án đầu tiên mà người dùng thuộc về (tài khoản gắn dự án hoặc project_members).
pub async fn first_project_of(pool: &PgPool, user: Uuid) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT p.id FROM projects p WHERE p.is_active AND ( \
            EXISTS (SELECT 1 FROM users u WHERE u.id = $1 AND u.project_id = p.id) \
            OR EXISTS (SELECT 1 FROM project_members m WHERE m.user_id = $1 AND m.project_id = p.id) \
            OR EXISTS (SELECT 1 FROM project_admins a WHERE a.admin_id = $1 AND a.project_id = p.id)) \
         ORDER BY p.code LIMIT 1",
    )
    .bind(user)
    .fetch_optional(pool)
    .await
}

/// Người dùng có thuộc dự án không (nhân viên, thành viên gói việc, hoặc quản trị dự án).
pub async fn is_member(pool: &PgPool, project: Uuid, user: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM users u WHERE u.id = $2 AND u.is_active AND u.project_id = $1) \
             OR EXISTS (SELECT 1 FROM project_members m WHERE m.project_id = $1 AND m.user_id = $2) \
             OR EXISTS (SELECT 1 FROM project_admins a WHERE a.project_id = $1 AND a.admin_id = $2)",
    )
    .bind(project)
    .bind(user)
    .fetch_one(pool)
    .await
}

// ---------------------------------------------------------------------------------------------------------------
// Dây chuyền (số đếm hiện tại theo dự án)
// ---------------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, Default, sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineCounts {
    pub cl_ho_so: i64,
    pub cl_van_ban: i64,
    pub scan_pages_reported: i64,
    pub files: i64,
    pub pages: i64,
    pub ocr_pending_docs: i64,
    pub ocr_pending_pages: i64,
    pub ocr_done_docs: i64,
    pub ocr_failed_pages: i64,
    pub pdfa_files: i64,
    pub to_sign: i64,
    pub signed_files: i64,
    pub sign_failed: i64,
    pub approved_files: i64,
    pub rejected_files: i64,
    pub pending_review_files: i64,
    pub packages_built: i64,
    pub docs_packaged: i64,
    pub sub_registered: i64,
    pub sub_received: i64,
    pub sub_completed: i64,
}

pub async fn pipeline_counts(pool: &PgPool, project: Uuid) -> sqlx::Result<PipelineCounts> {
    sqlx::query_as::<_, PipelineCounts>(
        "WITH d AS (SELECT status, review_status, page_count, pdfa_key, id FROM documents WHERE project_id = $1 AND status <> 'disposed') \
         SELECT \
           (SELECT count(*) FROM cl_ho_so WHERE project_id = $1 AND deleted_at IS NULL)::bigint AS cl_ho_so, \
           (SELECT count(*) FROM cl_van_ban WHERE project_id = $1 AND deleted_at IS NULL)::bigint AS cl_van_ban, \
           (SELECT COALESCE(sum(pages_scanned), 0) FROM user_stats WHERE project_id = $1)::bigint AS scan_pages_reported, \
           (SELECT count(*) FROM d)::bigint AS files, \
           (SELECT COALESCE(sum(page_count), 0) FROM d)::bigint AS pages, \
           (SELECT count(*) FROM d WHERE status = 'ocr_pending')::bigint AS ocr_pending_docs, \
           (SELECT COALESCE(sum(page_count), 0) FROM d WHERE status = 'ocr_pending')::bigint AS ocr_pending_pages, \
           (SELECT count(*) FROM d WHERE status NOT IN ('uploaded', 'ocr_pending'))::bigint AS ocr_done_docs, \
           (SELECT count(*) FROM ocr_jobs j JOIN d ON d.id = j.document_id \
              WHERE j.kind = 'ocr' AND j.status IN ('page_failed', 'failed_transient'))::bigint AS ocr_failed_pages, \
           (SELECT count(*) FROM d WHERE pdfa_key IS NOT NULL)::bigint AS pdfa_files, \
           (SELECT count(*) FROM d WHERE status = 'to_sign')::bigint AS to_sign, \
           (SELECT count(*) FROM d WHERE status IN ('signed', 'archived'))::bigint AS signed_files, \
           (SELECT count(*) FROM sign_sessions s JOIN d ON d.id = s.document_id WHERE s.status = 'rejected')::bigint AS sign_failed, \
           (SELECT count(*) FROM d WHERE review_status = 'approved')::bigint AS approved_files, \
           (SELECT count(*) FROM d WHERE review_status = 'rejected')::bigint AS rejected_files, \
           (SELECT count(*) FROM d WHERE review_status = 'pending' AND status NOT IN ('uploaded', 'ocr_pending'))::bigint AS pending_review_files, \
           (SELECT count(*) FROM packages WHERE project_id = $1 AND status = 'built')::bigint AS packages_built, \
           (SELECT COALESCE(sum(document_count), 0) FROM packages WHERE project_id = $1 AND status = 'built')::bigint AS docs_packaged, \
           (SELECT count(*) FROM submissions WHERE project_id = $1 AND status IN ('registered', 'awaiting_reception'))::bigint AS sub_registered, \
           (SELECT count(*) FROM submissions WHERE project_id = $1 AND status = 'received')::bigint AS sub_received, \
           (SELECT count(*) FROM submissions WHERE project_id = $1 AND status = 'completed')::bigint AS sub_completed",
    )
    .bind(project)
    .fetch_one(pool)
    .await
}

// ---------------------------------------------------------------------------------------------------------------
// Tồn đọng theo tuổi (ngày): 0–3 · 4–7 · 8–14 · >14
// ---------------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BacklogRow {
    pub stage: String,
    pub bucket: i32,
    pub docs: i64,
}

pub async fn backlog_buckets(pool: &PgPool, project: Uuid) -> sqlx::Result<Vec<BacklogRow>> {
    sqlx::query_as::<_, BacklogRow>(
        "WITH src AS ( \
            SELECT 'ocr' AS stage, uploaded_at AS since FROM documents WHERE project_id = $1 AND status = 'ocr_pending' \
          UNION ALL \
            SELECT 'review', COALESCE(pdfa_at, uploaded_at) FROM documents \
             WHERE project_id = $1 AND status NOT IN ('uploaded', 'ocr_pending', 'disposed') AND review_status = 'pending' \
          UNION ALL \
            SELECT 'sign', COALESCE(pdfa_at, uploaded_at) FROM documents WHERE project_id = $1 AND status = 'to_sign' \
          UNION ALL \
            SELECT 'package', COALESCE(pdfa_at, uploaded_at) FROM documents WHERE project_id = $1 AND status = 'signed' \
          UNION ALL \
            SELECT 'submit', updated_at FROM packages WHERE project_id = $1 AND status = 'built' AND submission_id IS NULL \
         ) \
         SELECT stage, \
                CASE WHEN age <= 3 THEN 0 WHEN age <= 7 THEN 1 WHEN age <= 14 THEN 2 ELSE 3 END AS bucket, \
                count(*)::bigint AS docs \
         FROM (SELECT stage, EXTRACT(DAY FROM now() - since)::int AS age FROM src) a \
         GROUP BY 1, 2 ORDER BY 1, 2",
    )
    .bind(project)
    .fetch_all(pool)
    .await
}

// ---------------------------------------------------------------------------------------------------------------
// SLA gói việc
// ---------------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, Default, sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkSla {
    pub open: i64,
    pub overdue: i64,
    pub due_soon3d: i64,
    pub closed30d: i64,
    pub closed_on_time30d: i64,
    pub pending_items: i64,
    pub items_over_sla: i64,
    pub avg_turnaround_hours: Option<f64>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverduePackage {
    pub package_id: Uuid,
    pub name: String,
    pub due_at: NaiveDate,
    pub pending_items: i64,
}

pub async fn work_sla(pool: &PgPool, project: Uuid, sla_hours: i32) -> sqlx::Result<(WorkSla, Vec<OverduePackage>)> {
    let today = digitext_core::dashboard::vn_today();
    let sla = sqlx::query_as::<_, WorkSla>(
        "WITH p AS (SELECT id, status, due_at, updated_at FROM work_packages WHERE project_id = $1), \
              pend AS (SELECT i.package_id, count(*) AS n FROM work_items i JOIN p ON p.id = i.package_id \
                       WHERE i.status = 'pending' GROUP BY i.package_id), \
              turn AS (SELECT i.id, min(c.at) AS claimed, min(d.at) AS done \
                       FROM work_items i JOIN p ON p.id = i.package_id \
                       JOIN work_item_events c ON c.item_id = i.id AND c.action = 'claim' \
                       JOIN work_item_events d ON d.item_id = i.id AND d.action IN ('done', 'redone') \
                       WHERE d.at >= now() - interval '30 days' GROUP BY i.id) \
         SELECT \
           (SELECT count(*) FROM p WHERE status IN ('open', 'under_review'))::bigint AS open, \
           (SELECT count(*) FROM p JOIN pend ON pend.package_id = p.id WHERE p.status IN ('open', 'under_review') AND p.due_at < $2)::bigint AS overdue, \
           (SELECT count(*) FROM p JOIN pend ON pend.package_id = p.id WHERE p.status IN ('open', 'under_review') AND p.due_at >= $2 AND p.due_at <= $2 + 3)::bigint AS due_soon3d, \
           (SELECT count(*) FROM p WHERE status IN ('done', 'closed') AND updated_at >= now() - interval '30 days')::bigint AS closed30d, \
           (SELECT count(*) FROM p WHERE status IN ('done', 'closed') AND updated_at >= now() - interval '30 days' \
                AND (due_at IS NULL OR (updated_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date <= due_at))::bigint AS closed_on_time30d, \
           (SELECT COALESCE(sum(n), 0) FROM pend)::bigint AS pending_items, \
           (SELECT count(*) FROM work_items i JOIN p ON p.id = i.package_id \
              WHERE i.status = 'pending' AND p.status IN ('open', 'under_review') \
                AND i.id IN (SELECT item_id FROM work_item_events WHERE action = 'claim' AND at < now() - ($3::int * interval '1 hour')))::bigint AS items_over_sla, \
           (SELECT avg(EXTRACT(EPOCH FROM done - claimed) / 3600.0) FROM turn WHERE done > claimed)::float8 AS avg_turnaround_hours",
    )
    .bind(project)
    .bind(today)
    .bind(sla_hours)
    .fetch_one(pool)
    .await?;
    let overdue = sqlx::query_as::<_, OverduePackage>(
        "SELECT p.id AS package_id, p.name, p.due_at, count(i.id)::bigint AS pending_items \
         FROM work_packages p JOIN work_items i ON i.package_id = p.id AND i.status = 'pending' \
         WHERE p.project_id = $1 AND p.status IN ('open', 'under_review') AND p.due_at < $2 \
         GROUP BY p.id ORDER BY p.due_at LIMIT 5",
    )
    .bind(project)
    .bind(today)
    .fetch_all(pool)
    .await?;
    Ok((sla, overdue))
}

// ---------------------------------------------------------------------------------------------------------------
// Cài đặt KPI theo dự án
// ---------------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct KpiSettingsRow {
    pub project_id: Uuid,
    pub quotas: serde_json::Value,
    pub excellent_pct: f64,
    pub pass_pct: f64,
    pub excellent_err_pct: f64,
    pub pass_err_pct: f64,
    pub rag_amber_lech: f64,
    pub rag_red_lech: f64,
    pub review_backlog_warn_days: i32,
    pub work_item_sla_hours: i32,
    pub group_min_users: i32,
    pub updated_by: Option<Uuid>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}
impl KpiSettingsRow {
    /// Mặc định khi dự án chưa cài đặt gì.
    pub fn default_for(project_id: Uuid) -> Self {
        KpiSettingsRow {
            project_id,
            quotas: serde_json::json!({}),
            excellent_pct: 100.0,
            pass_pct: 80.0,
            excellent_err_pct: 1.0,
            pass_err_pct: 3.0,
            rag_amber_lech: -5.0,
            rag_red_lech: -15.0,
            review_backlog_warn_days: 3,
            work_item_sla_hours: 48,
            group_min_users: 3,
            updated_by: None,
            updated_at: None,
        }
    }
    pub fn quota(&self, metric: &str) -> Option<f64> {
        self.quotas.get(metric).and_then(|v| v.as_f64()).filter(|q| *q > 0.0)
    }
}

const KPI_COLS: &str = "project_id, quotas, excellent_pct::float8 AS excellent_pct, pass_pct::float8 AS pass_pct, \
    excellent_err_pct::float8 AS excellent_err_pct, pass_err_pct::float8 AS pass_err_pct, \
    rag_amber_lech::float8 AS rag_amber_lech, rag_red_lech::float8 AS rag_red_lech, \
    review_backlog_warn_days, work_item_sla_hours, group_min_users, updated_by, updated_at";

pub async fn get_kpi_settings(pool: &PgPool, project: Uuid) -> sqlx::Result<KpiSettingsRow> {
    let row = sqlx::query_as::<_, KpiSettingsRow>(&format!("SELECT {KPI_COLS} FROM project_kpi_settings WHERE project_id = $1"))
        .bind(project)
        .fetch_optional(pool)
        .await?;
    Ok(row.unwrap_or_else(|| KpiSettingsRow::default_for(project)))
}

pub async fn kpi_settings_many(pool: &PgPool, projects: &[Uuid]) -> sqlx::Result<Vec<KpiSettingsRow>> {
    sqlx::query_as::<_, KpiSettingsRow>(&format!("SELECT {KPI_COLS} FROM project_kpi_settings WHERE project_id = ANY($1)"))
        .bind(projects)
        .fetch_all(pool)
        .await
}

pub async fn upsert_kpi_settings(pool: &PgPool, s: &KpiSettingsRow, by: Uuid) -> sqlx::Result<KpiSettingsRow> {
    sqlx::query_as::<_, KpiSettingsRow>(&format!(
        "INSERT INTO project_kpi_settings (project_id, quotas, excellent_pct, pass_pct, excellent_err_pct, pass_err_pct, \
            rag_amber_lech, rag_red_lech, review_backlog_warn_days, work_item_sla_hours, group_min_users, updated_by, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, now()) \
         ON CONFLICT (project_id) DO UPDATE SET quotas = EXCLUDED.quotas, excellent_pct = EXCLUDED.excellent_pct, \
            pass_pct = EXCLUDED.pass_pct, excellent_err_pct = EXCLUDED.excellent_err_pct, pass_err_pct = EXCLUDED.pass_err_pct, \
            rag_amber_lech = EXCLUDED.rag_amber_lech, rag_red_lech = EXCLUDED.rag_red_lech, \
            review_backlog_warn_days = EXCLUDED.review_backlog_warn_days, work_item_sla_hours = EXCLUDED.work_item_sla_hours, \
            group_min_users = EXCLUDED.group_min_users, updated_by = EXCLUDED.updated_by, updated_at = now() \
         RETURNING {KPI_COLS}"
    ))
    .bind(s.project_id)
    .bind(&s.quotas)
    .bind(s.excellent_pct)
    .bind(s.pass_pct)
    .bind(s.excellent_err_pct)
    .bind(s.pass_err_pct)
    .bind(s.rag_amber_lech)
    .bind(s.rag_red_lech)
    .bind(s.review_backlog_warn_days)
    .bind(s.work_item_sla_hours)
    .bind(s.group_min_users)
    .bind(by)
    .fetch_one(pool)
    .await
}

// ---------------------------------------------------------------------------------------------------------------
// Sự cố theo dự án (góc nhìn lãnh đạo): trang OCR lỗi, gói việc quá hạn, tệp bị từ chối khi đẩy lên.
// ---------------------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, Default, sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIssues {
    pub project_id: Uuid,
    pub ocr_failed_pages: i64,
    pub overdue_packages: i64,
    pub rejected_uploads: i64,
}

pub async fn issues_by_project(pool: &PgPool, projects: Option<&[Uuid]>) -> sqlx::Result<Vec<ProjectIssues>> {
    let projects: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    let today = digitext_core::dashboard::vn_today();
    sqlx::query_as::<_, ProjectIssues>(
        "SELECT p.id AS project_id, \
           (SELECT count(*) FROM ocr_jobs j JOIN documents d ON d.id = j.document_id \
              WHERE d.project_id = p.id AND j.kind = 'ocr' AND j.status IN ('page_failed', 'failed_transient'))::bigint AS ocr_failed_pages, \
           (SELECT count(*) FROM work_packages w WHERE w.project_id = p.id AND w.status IN ('open', 'under_review') AND w.due_at < $2 \
              AND EXISTS (SELECT 1 FROM work_items i WHERE i.package_id = w.id AND i.status = 'pending'))::bigint AS overdue_packages, \
           (SELECT count(*) FROM documents d WHERE d.project_id = p.id AND d.review_status = 'rejected')::bigint AS rejected_uploads \
         FROM projects p WHERE p.is_active AND ($1::uuid[] IS NULL OR p.id = ANY($1))",
    )
    .bind(projects)
    .bind(today)
    .fetch_all(pool)
    .await
}
