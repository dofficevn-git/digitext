// Copyright (c) 2026 DigiText. All rights reserved.
//! Báo cáo tiến độ dự án (đề nghị chỉnh sửa mục 10): hợp đồng + số liệu từng bước
//! (scan, PDF/A, ký số, OCR đã duyệt) tính trên kho máy chủ.
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct Contract {
    pub signed_on: Option<NaiveDate>,
    pub end_on: Option<NaiveDate>,
    pub deadline_on: Option<NaiveDate>,
    pub contract_pages: Option<i64>,
    pub contract_files: Option<i64>,
    /// Khối lượng hợp đồng theo TT05 (0024, 3/11/2026): hộp, hồ sơ, tờ — để nghiệm thu đối chiếu với mục lục.
    pub contract_boxes: Option<i64>,
    pub contract_dossiers: Option<i64>,
    pub contract_sheets: Option<i64>,
    pub manager_name: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProgressRow {
    pub project_id: Uuid,
    pub code: String,
    pub name: String,
    pub signed_on: Option<NaiveDate>,
    pub end_on: Option<NaiveDate>,
    pub deadline_on: Option<NaiveDate>,
    pub contract_pages: Option<i64>,
    pub contract_files: Option<i64>,
    pub contract_boxes: Option<i64>,
    pub contract_dossiers: Option<i64>,
    pub contract_sheets: Option<i64>,
    pub manager_name: Option<String>,
    pub note: Option<String>,
    pub files: i64,
    pub pages: i64,
    pub pdfa_files: i64,
    pub signed_files: i64,
    pub ocr_files: i64,
    pub approved_files: i64,
    pub rejected_files: i64,
}

/// Số liệu tiến độ theo dự án. `from`/`to` lọc theo giờ tài liệu lên máy chủ (`to` loại trừ).
pub async fn project_progress(
    pool: &PgPool,
    projects: Option<&[Uuid]>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
) -> sqlx::Result<Vec<ProgressRow>> {
    let projects: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    sqlx::query_as::<_, ProgressRow>(
        "SELECT p.id AS project_id, p.code, p.name, \
                c.signed_on, c.end_on, c.deadline_on, c.contract_pages, c.contract_files, \
                c.contract_boxes, c.contract_dossiers, c.contract_sheets, c.manager_name, c.note, \
                count(d.id) AS files, \
                COALESCE(sum(d.page_count), 0)::bigint AS pages, \
                count(d.id) FILTER (WHERE d.pdfa_key IS NOT NULL) AS pdfa_files, \
                count(d.id) FILTER (WHERE d.status IN ('signed', 'archived')) AS signed_files, \
                count(d.id) FILTER (WHERE d.status NOT IN ('uploaded', 'ocr_pending')) AS ocr_files, \
                count(d.id) FILTER (WHERE d.review_status = 'approved') AS approved_files, \
                count(d.id) FILTER (WHERE d.review_status = 'rejected') AS rejected_files \
         FROM projects p \
         LEFT JOIN project_contracts c ON c.project_id = p.id \
         LEFT JOIN documents d ON d.project_id = p.id AND d.status <> 'disposed' \
              AND ($2::timestamptz IS NULL OR d.uploaded_at >= $2) \
              AND ($3::timestamptz IS NULL OR d.uploaded_at < $3) \
         WHERE p.is_active AND ($1::uuid[] IS NULL OR p.id = ANY($1)) \
         GROUP BY p.id, c.project_id \
         ORDER BY p.code",
    )
    .bind(projects)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}

pub async fn upsert_contract(pool: &PgPool, project_id: Uuid, c: &Contract, by: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO project_contracts \
           (project_id, signed_on, end_on, deadline_on, contract_pages, contract_files, manager_name, note, updated_by, updated_at, \
            contract_boxes, contract_dossiers, contract_sheets) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now(), $10, $11, $12) \
         ON CONFLICT (project_id) DO UPDATE SET \
           signed_on = EXCLUDED.signed_on, end_on = EXCLUDED.end_on, deadline_on = EXCLUDED.deadline_on, \
           contract_pages = EXCLUDED.contract_pages, contract_files = EXCLUDED.contract_files, \
           contract_boxes = EXCLUDED.contract_boxes, contract_dossiers = EXCLUDED.contract_dossiers, contract_sheets = EXCLUDED.contract_sheets, \
           manager_name = EXCLUDED.manager_name, note = EXCLUDED.note, \
           updated_by = EXCLUDED.updated_by, updated_at = now()",
    )
    .bind(project_id)
    .bind(c.signed_on)
    .bind(c.end_on)
    .bind(c.deadline_on)
    .bind(c.contract_pages)
    .bind(c.contract_files)
    .bind(c.manager_name.as_deref())
    .bind(c.note.as_deref())
    .bind(by)
    .bind(c.contract_boxes)
    .bind(c.contract_dossiers)
    .bind(c.contract_sheets)
    .execute(pool)
    .await?;
    Ok(())
}

/// Một dòng sản lượng: người × công việc (đề nghị chỉnh sửa mục 11).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StaffRow {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    /// scan | nhap_lieu | xac_nhan | kiem_tra_*
    pub task: String,
    /// Sản lượng thi công: scan = số trang; các việc khác = số tài liệu.
    pub done: i64,
    /// Số tài liệu liên quan (scan: số tệp; nhập liệu: số tài liệu có trường đã nhập).
    pub files: i64,
    /// Số trường đã nhập (chỉ việc nhập liệu).
    pub fields: i64,
    /// Số ngày có làm (để tính năng suất trung bình/ngày).
    pub days: i64,
    /// Số lỗi: tài liệu của người đó bị TỪ CHỐI khi xác nhận.
    pub errors: i64,
    /// Sản lượng giao (NULL = chưa giao).
    pub target: Option<i64>,
}

pub async fn staff_report(
    pool: &PgPool,
    projects: Option<&[Uuid]>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
) -> sqlx::Result<Vec<StaffRow>> {
    let projects: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    let mut rows = sqlx::query_as::<_, StaffRow>(
        "WITH d AS ( \
             SELECT * FROM documents \
             WHERE status <> 'disposed' AND ($1::uuid[] IS NULL OR project_id = ANY($1)) \
         ), work AS ( \
             SELECT uploaded_by AS user_id, 'scan' AS task, \
                    COALESCE(sum(page_count), 0)::bigint AS done, count(*) AS files, 0::bigint AS fields, \
                    count(DISTINCT (uploaded_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date) AS days, \
                    count(*) FILTER (WHERE review_status = 'rejected') AS errors \
             FROM d WHERE uploaded_by IS NOT NULL \
               AND ($2::timestamptz IS NULL OR uploaded_at >= $2) AND ($3::timestamptz IS NULL OR uploaded_at < $3) \
             GROUP BY uploaded_by \
           UNION ALL \
             SELECT f.edited_by, 'nhap_lieu', count(DISTINCT f.document_id), count(DISTINCT f.document_id), count(*), \
                    count(DISTINCT (f.edited_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date), \
                    count(DISTINCT f.document_id) FILTER (WHERE d.review_status = 'rejected') \
             FROM ocr_fields f JOIN d ON d.id = f.document_id \
             WHERE f.edited_by IS NOT NULL \
               AND ($2::timestamptz IS NULL OR f.edited_at >= $2) AND ($3::timestamptz IS NULL OR f.edited_at < $3) \
             GROUP BY f.edited_by \
           UNION ALL \
             SELECT reviewed_by, 'xac_nhan', count(*), count(*), 0, \
                    count(DISTINCT (reviewed_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date), 0 \
             FROM d WHERE reviewed_by IS NOT NULL AND review_status IN ('approved', 'rejected') \
               AND ($2::timestamptz IS NULL OR reviewed_at >= $2) AND ($3::timestamptz IS NULL OR reviewed_at < $3) \
             GROUP BY reviewed_by \
         ), targets AS ( \
             SELECT user_id, task, sum(quantity)::bigint AS target FROM work_targets \
             WHERE ($1::uuid[] IS NULL OR project_id = ANY($1)) GROUP BY user_id, task \
         ) \
         SELECT u.id AS user_id, u.username, u.display_name, \
                COALESCE(w.task, t.task) AS task, \
                COALESCE(w.done, 0) AS done, COALESCE(w.files, 0) AS files, COALESCE(w.fields, 0) AS fields, \
                COALESCE(w.days, 0) AS days, COALESCE(w.errors, 0) AS errors, t.target \
         FROM work w FULL JOIN targets t ON t.user_id = w.user_id AND t.task = w.task \
         JOIN users u ON u.id = COALESCE(w.user_id, t.user_id) \
         ORDER BY u.display_name, u.username, task",
    )
    .bind(projects.clone())
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    // Gói việc: "xác nhận" đếm theo mục CTV xác nhận (thay cho reviewed_by khi có gói), "kiểm tra
    // nhập liệu" đếm theo mục CTV chấm chéo. Gộp vào dòng cùng người × việc, hoặc thêm dòng mới.
    let work = crate::work_packages::staff_work(pool, projects.as_deref(), from, to).await?;
    if !work.is_empty() {
        let names: Vec<(Uuid, String, String)> = sqlx::query_as(
            "SELECT id, username, display_name FROM users WHERE id = ANY($1)",
        )
        .bind(work.iter().map(|w| w.user_id).collect::<Vec<_>>())
        .fetch_all(pool)
        .await?;
        for w in work {
            if let Some(r) = rows.iter_mut().find(|r| r.user_id == w.user_id && r.task == w.task) {
                r.done += w.done;
                r.files += w.done;
                r.days = r.days.max(w.days);
                r.errors += w.errors;
            } else if let Some((_, username, display_name)) = names.iter().find(|n| n.0 == w.user_id) {
                rows.push(StaffRow {
                    user_id: w.user_id,
                    username: username.clone(),
                    display_name: display_name.clone(),
                    task: w.task,
                    done: w.done,
                    files: w.done,
                    fields: 0,
                    days: w.days,
                    errors: w.errors,
                    target: None,
                });
            }
        }
        rows.sort_by(|a, b| (&a.display_name, &a.username, &a.task).cmp(&(&b.display_name, &b.username, &b.task)));
    }
    Ok(rows)
}

pub const TASKS: [&str; 6] = ["scan", "kiem_tra_scan", "nhap_lieu", "kiem_tra_nhap_lieu", "xac_nhan", "kiem_tra_xac_nhan"];

/// Đặt sản lượng giao cho một người × công việc trong dự án (`quantity` None = xoá).
pub async fn set_target(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
    task: &str,
    quantity: Option<i64>,
    by: Uuid,
) -> sqlx::Result<()> {
    match quantity {
        None => {
            sqlx::query("DELETE FROM work_targets WHERE project_id = $1 AND user_id = $2 AND task = $3")
                .bind(project_id)
                .bind(user_id)
                .bind(task)
                .execute(pool)
                .await?;
        }
        Some(q) => {
            sqlx::query(
                "INSERT INTO work_targets (project_id, user_id, task, quantity, updated_by, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, now()) \
                 ON CONFLICT (project_id, user_id, task) DO UPDATE SET \
                   quantity = EXCLUDED.quantity, updated_by = EXCLUDED.updated_by, updated_at = now()",
            )
            .bind(project_id)
            .bind(user_id)
            .bind(task)
            .bind(q)
            .bind(by)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

/// Ghi nhận người nhập / người xác nhận một trường (gọi sau khi người dùng lưu trường).
pub async fn mark_field_edit(pool: &PgPool, field_id: Uuid, user: Uuid, verified: bool) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE ocr_fields SET edited_by = $2, edited_at = now(), \
           verified_by = CASE WHEN $3 THEN COALESCE(verified_by, $2) ELSE NULL END, \
           verified_at = CASE WHEN $3 THEN COALESCE(verified_at, now()) ELSE NULL END \
         WHERE id = $1",
    )
    .bind(field_id)
    .bind(user)
    .bind(verified)
    .execute(pool)
    .await?;
    Ok(())
}
