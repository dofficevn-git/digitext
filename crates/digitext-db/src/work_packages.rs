// Copyright (c) 2026 DigiText. All rights reserved.
//! Gói việc & cộng tác viên xác nhận trường (migration 0011). Xem kế hoạch 29/9/2026.
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Khoá mềm một mục khi CTV đang mở (như doc_extract_pla: 10 phút).
pub const LEASE_SECS: i64 = 10 * 60;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Package {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub kind: String,
    pub mode: String,
    pub status: String,
    pub due_at: Option<NaiveDate>,
    pub note: Option<String>,
    pub source_package_id: Option<Uuid>,
    pub sample_pct: Option<i32>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Bộ đếm theo trạng thái mục (tính bằng LEFT JOIN).
    pub n_total: i64,
    pub n_pending: i64,
    pub n_done: i64,
    pub n_checked: i64,
    pub n_rejected: i64,
    pub n_approved: i64,
}
const PKG_SELECT: &str = "SELECT p.id, p.project_id, p.name, p.kind, p.mode, p.status, p.due_at, p.note, \
        p.source_package_id, p.sample_pct, p.created_by, p.created_at, p.updated_at, \
        count(i.id) AS n_total, \
        count(i.id) FILTER (WHERE i.status = 'pending') AS n_pending, \
        count(i.id) FILTER (WHERE i.status IN ('done', 'redone')) AS n_done, \
        count(i.id) FILTER (WHERE i.status IN ('checked_ok', 'checked_err')) AS n_checked, \
        count(i.id) FILTER (WHERE i.status = 'rejected') AS n_rejected, \
        count(i.id) FILTER (WHERE i.status = 'approved') AS n_approved \
     FROM work_packages p LEFT JOIN work_items i ON i.package_id = p.id";
const PKG_GROUP: &str = "GROUP BY p.id";

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Member {
    pub package_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Item {
    pub id: Uuid,
    pub package_id: Uuid,
    pub document_id: Uuid,
    pub ord: i32,
    pub status: String,
    pub locked_by: Option<Uuid>,
    pub lease_until: Option<DateTime<Utc>>,
    pub done_by: Option<Uuid>,
    pub done_at: Option<DateTime<Utc>>,
    pub checked_by: Option<Uuid>,
    pub checked_at: Option<DateTime<Utc>>,
    pub check_note: Option<String>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_note: Option<String>,
    pub filename: String,
    pub rel_path: Option<String>,
    pub ho_so_ref: Option<String>,
    pub page_count: Option<i32>,
    pub done_by_name: Option<String>,
    pub checked_by_name: Option<String>,
}
const ITEM_SELECT: &str = "SELECT i.id, i.package_id, i.document_id, i.ord, i.status, i.locked_by, i.lease_until, \
        i.done_by, i.done_at, i.checked_by, i.checked_at, i.check_note, i.reviewed_by, i.reviewed_at, i.review_note, \
        d.filename, d.rel_path, d.ho_so_ref, d.page_count, ud.display_name AS done_by_name, uc.display_name AS checked_by_name \
     FROM work_items i JOIN documents d ON d.id = i.document_id \
     LEFT JOIN users ud ON ud.id = i.done_by LEFT JOIN users uc ON uc.id = i.checked_by";

/// Tài liệu ứng viên để chia gói.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Candidate {
    pub id: Uuid,
    pub rel_path: Option<String>,
}

/// Tài liệu trong dự án ĐÃ bóc trường, CHƯA nằm trong gói xác nhận dang dở, trong phạm vi
/// `prefixes` (tiền tố rel_path; rỗng = cả dự án). Sắp theo đường dẫn để chia liền mạch theo hồ sơ.
pub async fn candidates(pool: &PgPool, project_id: Uuid, prefixes: &[String]) -> sqlx::Result<Vec<Candidate>> {
    let pats: Vec<String> = prefixes
        .iter()
        .map(|p| p.trim().trim_end_matches('/').to_owned())
        .filter(|p| !p.is_empty())
        .collect();
    sqlx::query_as::<_, Candidate>(
        "SELECT d.id, d.rel_path FROM documents d \
         WHERE d.project_id = $1 AND d.status <> 'disposed' \
           AND EXISTS (SELECT 1 FROM ocr_fields f WHERE f.document_id = d.id) \
           AND NOT EXISTS (SELECT 1 FROM work_items i WHERE i.document_id = d.id AND i.kind = 'xac_nhan' \
                           AND i.status NOT IN ('approved', 'cancelled')) \
           AND (cardinality($2::text[]) = 0 OR EXISTS (SELECT 1 FROM unnest($2::text[]) p \
                WHERE COALESCE(d.rel_path, '') = p OR left(COALESCE(d.rel_path, ''), length(p) + 1) = p || '/')) \
         ORDER BY d.rel_path COLLATE vi_so NULLS LAST, d.filename COLLATE vi_so",
    )
    .bind(project_id)
    .bind(pats)
    .fetch_all(pool)
    .await
}

/// Mục đã xác nhận (done/redone/checked/approved) của một gói xác nhận — ứng viên cho gói kiểm tra.
pub async fn done_candidates(pool: &PgPool, source_package: Uuid) -> sqlx::Result<Vec<Candidate>> {
    sqlx::query_as::<_, Candidate>(
        "SELECT d.id, d.rel_path FROM work_items i JOIN documents d ON d.id = i.document_id \
         WHERE i.package_id = $1 AND i.status IN ('done', 'redone', 'checked_ok', 'checked_err', 'approved') \
         ORDER BY i.ord",
    )
    .bind(source_package)
    .fetch_all(pool)
    .await
}

/// Thư mục hồ sơ của một tài liệu = rel_path (đã là thư mục phông/hộp/hồ sơ).
fn hoso_key(c: &Candidate) -> &str {
    c.rel_path.as_deref().unwrap_or("")
}

/// Chia liền mạch theo hồ sơ: gói ≥ `size` tài liệu nhưng KHÔNG cắt giữa hồ sơ (lấy trọn hồ sơ cuối),
/// tối đa `count` gói. Ứng viên phải đã sắp theo đường dẫn.
pub fn split_hoso(cands: &[Candidate], size: usize, count: usize) -> Vec<Vec<Uuid>> {
    let mut out: Vec<Vec<Uuid>> = Vec::new();
    let mut cur: Vec<Uuid> = Vec::new();
    let size = size.max(1);
    let mut i = 0;
    while i < cands.len() && out.len() < count {
        cur.push(cands[i].id);
        let last_of_hoso = i + 1 >= cands.len() || hoso_key(&cands[i + 1]) != hoso_key(&cands[i]);
        if cur.len() >= size && last_of_hoso {
            out.push(std::mem::take(&mut cur));
        }
        i += 1;
    }
    if !cur.is_empty() && out.len() < count {
        out.push(cur);
    }
    out
}

/// Chia ngẫu nhiên: xáo rồi cắt `count` gói, mỗi gói `size`.
pub fn split_random(cands: &[Candidate], size: usize, count: usize, seed: u64) -> Vec<Vec<Uuid>> {
    let mut ids: Vec<Uuid> = cands.iter().map(|c| c.id).collect();
    // Fisher–Yates với xorshift (không kéo thêm crate rand).
    let mut s = seed | 1;
    for k in (1..ids.len()).rev() {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        let j = (s % (k as u64 + 1)) as usize;
        ids.swap(k, j);
    }
    ids.chunks(size.max(1)).take(count).map(|c| c.to_vec()).collect()
}

#[derive(Debug, Clone)]
pub struct NewPackage<'a> {
    pub project_id: Uuid,
    pub name: &'a str,
    pub kind: &'a str,
    pub mode: &'a str,
    pub due_at: Option<NaiveDate>,
    pub note: Option<&'a str>,
    pub source_package_id: Option<Uuid>,
    pub sample_pct: Option<i32>,
    pub members: &'a [Uuid],
    pub created_by: Uuid,
}

/// Tạo gói + thành viên + mục (ord theo thứ tự `doc_ids`) trong một giao dịch.
pub async fn create(pool: &PgPool, p: &NewPackage<'_>, doc_ids: &[Uuid]) -> sqlx::Result<Uuid> {
    let mut tx = pool.begin().await?;
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO work_packages (id, project_id, name, kind, mode, due_at, note, source_package_id, sample_pct, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(id)
    .bind(p.project_id)
    .bind(p.name)
    .bind(p.kind)
    .bind(p.mode)
    .bind(p.due_at)
    .bind(p.note)
    .bind(p.source_package_id)
    .bind(p.sample_pct)
    .bind(p.created_by)
    .execute(&mut *tx)
    .await?;
    for m in p.members {
        sqlx::query("INSERT INTO work_package_members (package_id, user_id, added_by) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
            .bind(id)
            .bind(m)
            .bind(p.created_by)
            .execute(&mut *tx)
            .await?;
    }
    let ords: Vec<i32> = (1..=doc_ids.len() as i32).collect();
    let ids: Vec<Uuid> = doc_ids.iter().map(|_| Uuid::now_v7()).collect();
    sqlx::query(
        "INSERT INTO work_items (id, package_id, kind, document_id, ord) \
         SELECT i, $2, $3, d, o FROM unnest($1::uuid[], $4::uuid[], $5::int[]) AS t(i, d, o)",
    )
    .bind(&ids)
    .bind(id)
    .bind(p.kind)
    .bind(doc_ids)
    .bind(&ords)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(id)
}

pub async fn get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Package>> {
    sqlx::query_as::<_, Package>(&format!("{PKG_SELECT} WHERE p.id = $1 {PKG_GROUP}"))
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list_for_projects(pool: &PgPool, projects: Option<&[Uuid]>) -> sqlx::Result<Vec<Package>> {
    let projects: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    sqlx::query_as::<_, Package>(&format!(
        "{PKG_SELECT} WHERE ($1::uuid[] IS NULL OR p.project_id = ANY($1)) {PKG_GROUP} ORDER BY p.created_at DESC"
    ))
    .bind(projects)
    .fetch_all(pool)
    .await
}

/// Gói mà `user` là thành viên và chưa đóng.
pub async fn list_for_member(pool: &PgPool, user: Uuid) -> sqlx::Result<Vec<Package>> {
    sqlx::query_as::<_, Package>(&format!(
        "{PKG_SELECT} WHERE p.status <> 'closed' AND EXISTS (SELECT 1 FROM work_package_members m WHERE m.package_id = p.id AND m.user_id = $1) \
         {PKG_GROUP} ORDER BY p.due_at NULLS LAST, p.created_at"
    ))
    .bind(user)
    .fetch_all(pool)
    .await
}

pub async fn members(pool: &PgPool, packages: &[Uuid]) -> sqlx::Result<Vec<Member>> {
    sqlx::query_as::<_, Member>(
        "SELECT m.package_id, m.user_id, u.username, u.display_name FROM work_package_members m \
         JOIN users u ON u.id = m.user_id WHERE m.package_id = ANY($1) ORDER BY u.display_name",
    )
    .bind(packages)
    .fetch_all(pool)
    .await
}

pub async fn set_members(pool: &PgPool, package: Uuid, users: &[Uuid], by: Uuid) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM work_package_members WHERE package_id = $1 AND user_id <> ALL($2)")
        .bind(package)
        .bind(users)
        .execute(&mut *tx)
        .await?;
    for u in users {
        sqlx::query("INSERT INTO work_package_members (package_id, user_id, added_by) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
            .bind(package)
            .bind(u)
            .bind(by)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Người có thể được giao việc trong dự án: thuộc dự án (users.project_id), là thành viên, hoặc quản trị dự án.
pub async fn is_project_user(pool: &PgPool, project: Uuid, user: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM users u WHERE u.id = $2 AND u.is_active AND (u.project_id = $1 \
                OR EXISTS (SELECT 1 FROM project_members m WHERE m.project_id = $1 AND m.user_id = $2) \
                OR EXISTS (SELECT 1 FROM project_admins a WHERE a.project_id = $1 AND a.admin_id = $2) \
                OR u.role = 'super_admin'))",
    )
    .bind(project)
    .bind(user)
    .fetch_one(pool)
    .await
}

pub async fn is_member(pool: &PgPool, package: Uuid, user: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM work_package_members WHERE package_id = $1 AND user_id = $2)",
    )
    .bind(package)
    .bind(user)
    .fetch_one(pool)
    .await
}

/// CTV có được chạm tài liệu này không: là thành viên một gói chưa đóng có chứa tài liệu.
pub async fn can_work_on(pool: &PgPool, user: Uuid, document: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM work_items i JOIN work_packages p ON p.id = i.package_id \
                        JOIN work_package_members m ON m.package_id = p.id \
                        WHERE i.document_id = $2 AND m.user_id = $1 AND p.status <> 'closed')",
    )
    .bind(user)
    .bind(document)
    .fetch_one(pool)
    .await
}

pub async fn items(pool: &PgPool, package: Uuid) -> sqlx::Result<Vec<Item>> {
    sqlx::query_as::<_, Item>(&format!("{ITEM_SELECT} WHERE i.package_id = $1 ORDER BY i.ord"))
        .bind(package)
        .fetch_all(pool)
        .await
}

pub async fn item(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Item>> {
    sqlx::query_as::<_, Item>(&format!("{ITEM_SELECT} WHERE i.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await
}

async fn event(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, item: Uuid, actor: Uuid, action: &str, note: Option<&str>) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO work_item_events (item_id, actor_id, action, note) VALUES ($1, $2, $3, $4)")
        .bind(item)
        .bind(actor)
        .bind(action)
        .bind(note)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Trạng thái mục mà CTV còn phải làm trong gói loại này.
fn workable(kind: &str) -> &'static str {
    match kind {
        "kiem_tra" => "('pending')",
        _ => "('pending', 'rejected')",
    }
}

/// Nhận mục kế tiếp: ưu tiên mục đang do chính mình khoá còn hạn; không thì mục nhỏ nhất theo `ord`
/// chưa ai khoá (hoặc khoá đã hết hạn). MỘT câu UPDATE nguyên tử (FOR UPDATE SKIP LOCKED).
pub async fn claim_next(pool: &PgPool, package: Uuid, kind: &str, user: Uuid) -> sqlx::Result<Option<Item>> {
    let mut tx = pool.begin().await?;
    let id: Option<Uuid> = sqlx::query_scalar(&format!(
        "UPDATE work_items SET locked_by = $2, lease_until = now() + ($3 * interval '1 second') \
         WHERE id = (SELECT id FROM work_items WHERE package_id = $1 AND status IN {} \
                     AND (locked_by = $2 OR lease_until IS NULL OR lease_until < now()) \
                     ORDER BY (locked_by = $2 AND lease_until >= now()) DESC, ord \
                     FOR UPDATE SKIP LOCKED LIMIT 1) \
         RETURNING id",
        workable(kind)
    ))
    .bind(package)
    .bind(user)
    .bind(LEASE_SECS)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(id) = id {
        event(&mut tx, id, user, "claim", None).await?;
    }
    tx.commit().await?;
    match id {
        Some(id) => item(pool, id).await,
        None => Ok(None),
    }
}

/// Nhận ĐÍCH DANH một mục (vd. mục bị trả trong hộp thư): còn làm được và chưa ai khác giữ khoá.
pub async fn claim_item(pool: &PgPool, id: Uuid, kind: &str, user: Uuid) -> sqlx::Result<Option<Item>> {
    let mut tx = pool.begin().await?;
    let got: Option<Uuid> = sqlx::query_scalar(&format!(
        "UPDATE work_items SET locked_by = $2, lease_until = now() + ($3 * interval '1 second') \
         WHERE id = $1 AND status IN {} AND (locked_by = $2 OR lease_until IS NULL OR lease_until < now()) \
         RETURNING id",
        workable(kind)
    ))
    .bind(id)
    .bind(user)
    .bind(LEASE_SECS)
    .fetch_optional(&mut *tx)
    .await?;
    if got.is_some() {
        event(&mut tx, id, user, "claim", None).await?;
    }
    tx.commit().await?;
    match got {
        Some(id) => item(pool, id).await,
        None => Ok(None),
    }
}

/// Chuyển sang mục kề (dir = -1 / +1) trong cùng gói: nhả mục hiện tại, khoá mục kề nếu còn làm được.
pub async fn navigate(pool: &PgPool, current: Uuid, kind: &str, user: Uuid, dir: i32) -> sqlx::Result<Option<Item>> {
    let Some(cur) = item(pool, current).await? else {
        return Ok(None);
    };
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE work_items SET locked_by = NULL, lease_until = NULL WHERE id = $1 AND locked_by = $2")
        .bind(current)
        .bind(user)
        .execute(&mut *tx)
        .await?;
    let (cmp, order) = if dir < 0 { ("<", "DESC") } else { (">", "ASC") };
    let next: Option<Uuid> = sqlx::query_scalar(&format!(
        "UPDATE work_items SET locked_by = $3, lease_until = now() + ($4 * interval '1 second') \
         WHERE id = (SELECT id FROM work_items WHERE package_id = $1 AND ord {cmp} $2 AND status IN {} \
                     AND (locked_by = $3 OR lease_until IS NULL OR lease_until < now()) \
                     ORDER BY ord {order} FOR UPDATE SKIP LOCKED LIMIT 1) RETURNING id",
        workable(kind)
    ))
    .bind(cur.package_id)
    .bind(cur.ord)
    .bind(user)
    .bind(LEASE_SECS)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(n) = next {
        event(&mut tx, n, user, "claim", None).await?;
    }
    tx.commit().await?;
    match next {
        Some(n) => item(pool, n).await,
        None => Ok(None),
    }
}

pub async fn release(pool: &PgPool, id: Uuid, user: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE work_items SET locked_by = NULL, lease_until = NULL WHERE id = $1 AND locked_by = $2")
        .bind(id)
        .bind(user)
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    /// Mục không ở trạng thái làm được, hoặc không do người này khoá (bấm hai lần / hết khoá).
    Conflict,
}

/// CTV xác nhận hoàn thành: pending → done, rejected → redone. Phải đang giữ khoá.
pub async fn mark_done(pool: &PgPool, id: Uuid, user: Uuid) -> sqlx::Result<Outcome> {
    let mut tx = pool.begin().await?;
    let new_status: Option<String> = sqlx::query_scalar(
        "UPDATE work_items SET status = CASE WHEN status = 'rejected' THEN 'redone' ELSE 'done' END, \
             done_by = $2, done_at = now(), locked_by = NULL, lease_until = NULL \
         WHERE id = $1 AND status IN ('pending', 'rejected') AND locked_by = $2 AND lease_until >= now() \
         RETURNING status",
    )
    .bind(id)
    .bind(user)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(s) = new_status else {
        return Ok(Outcome::Conflict);
    };
    event(&mut tx, id, user, if s == "redone" { "redone" } else { "done" }, None).await?;
    tx.commit().await?;
    recompute_status_of_item(pool, id).await?;
    Ok(Outcome::Ok)
}

/// Gói kiểm tra: CTV chấm mục đang khoá (pending) → checked_ok / checked_err.
pub async fn mark_checked(pool: &PgPool, id: Uuid, user: Uuid, ok: bool, note: Option<&str>) -> sqlx::Result<Outcome> {
    let mut tx = pool.begin().await?;
    let n = sqlx::query(
        "UPDATE work_items SET status = $3, checked_by = $2, checked_at = now(), check_note = $4, locked_by = NULL, lease_until = NULL \
         WHERE id = $1 AND kind = 'kiem_tra' AND status = 'pending' AND locked_by = $2 AND lease_until >= now()",
    )
    .bind(id)
    .bind(user)
    .bind(if ok { "checked_ok" } else { "checked_err" })
    .bind(note)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if n == 0 {
        return Ok(Outcome::Conflict);
    }
    event(&mut tx, id, user, if ok { "check_ok" } else { "check_err" }, note).await?;
    tx.commit().await?;
    recompute_status_of_item(pool, id).await?;
    Ok(Outcome::Ok)
}

/// Quản trị duyệt / từ chối các mục (đã done/redone/checked). Không duyệt mục do CHÍNH MÌNH xác nhận.
/// Trả (số mục đổi, số mục bỏ qua, danh sách document_id đã duyệt).
pub async fn review(
    pool: &PgPool,
    package: Uuid,
    item_ids: Option<&[Uuid]>,
    approve: bool,
    note: Option<&str>,
    by: Uuid,
) -> sqlx::Result<(usize, usize, Vec<Uuid>)> {
    let ids: Option<Vec<Uuid>> = item_ids.map(|x| x.to_vec());
    let mut tx = pool.begin().await?;
    let rows: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "UPDATE work_items SET status = $3, reviewed_by = $4, reviewed_at = now(), review_note = $5, locked_by = NULL, lease_until = NULL \
         WHERE package_id = $1 AND ($2::uuid[] IS NULL OR id = ANY($2)) \
           AND status IN ('done', 'redone', 'checked_ok', 'checked_err') AND done_by IS DISTINCT FROM $4 \
         RETURNING id, document_id",
    )
    .bind(package)
    .bind(&ids)
    .bind(if approve { "approved" } else { "rejected" })
    .bind(by)
    .bind(note)
    .fetch_all(&mut *tx)
    .await?;
    for (id, _) in &rows {
        event(&mut tx, *id, by, if approve { "approve" } else { "reject" }, note).await?;
    }
    tx.commit().await?;
    let asked = match &ids {
        Some(v) => v.len(),
        None => rows.len(),
    };
    recompute_status(pool, package).await?;
    Ok((rows.len(), asked.saturating_sub(rows.len()), rows.into_iter().map(|(_, d)| d).collect()))
}

/// Hộp thư "mục bị trả" của một CTV (mọi gói chưa đóng mà người này là thành viên).
pub async fn rejected_for(pool: &PgPool, user: Uuid) -> sqlx::Result<Vec<Item>> {
    sqlx::query_as::<_, Item>(&format!(
        "{ITEM_SELECT} JOIN work_packages p ON p.id = i.package_id \
         WHERE i.status = 'rejected' AND p.status <> 'closed' \
           AND EXISTS (SELECT 1 FROM work_package_members m WHERE m.package_id = p.id AND m.user_id = $1) \
         ORDER BY i.reviewed_at"
    ))
    .bind(user)
    .fetch_all(pool)
    .await
}

async fn recompute_status_of_item(pool: &PgPool, item: Uuid) -> sqlx::Result<()> {
    let pkg: Option<Uuid> = sqlx::query_scalar("SELECT package_id FROM work_items WHERE id = $1")
        .bind(item)
        .fetch_optional(pool)
        .await?;
    match pkg {
        Some(p) => recompute_status(pool, p).await,
        None => Ok(()),
    }
}

/// open → under_review (không còn mục pending/rejected) → done (mọi mục approved). Gói đã closed giữ nguyên.
pub async fn recompute_status(pool: &PgPool, package: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE work_packages p SET status = s.new_status, updated_at = now() FROM ( \
             SELECT $1::uuid AS id, CASE \
                 WHEN count(*) FILTER (WHERE status NOT IN ('approved', 'cancelled')) = 0 THEN 'done' \
                 WHEN count(*) FILTER (WHERE status IN ('pending', 'rejected')) = 0 THEN 'under_review' \
                 ELSE 'open' END AS new_status \
             FROM work_items WHERE package_id = $1) s \
         WHERE p.id = s.id AND p.status <> 'closed' AND p.status <> s.new_status",
    )
    .bind(package)
    .execute(pool)
    .await?;
    Ok(())
}

/// Đóng gói dở: mục chưa duyệt → cancelled (mở lại chỗ cho tài liệu vào gói khác).
pub async fn close(pool: &PgPool, package: Uuid, by: Uuid) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;
    let ids: Vec<Uuid> = sqlx::query_scalar(
        "UPDATE work_items SET status = 'cancelled', locked_by = NULL, lease_until = NULL \
         WHERE package_id = $1 AND status <> 'approved' RETURNING id",
    )
    .bind(package)
    .fetch_all(&mut *tx)
    .await?;
    for id in ids {
        event(&mut tx, id, by, "cancel", None).await?;
    }
    sqlx::query("UPDATE work_packages SET status = 'closed', updated_at = now() WHERE id = $1")
        .bind(package)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn update_meta(pool: &PgPool, package: Uuid, name: Option<&str>, due_at: Option<Option<NaiveDate>>, note: Option<&str>) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE work_packages SET name = COALESCE($2, name), due_at = CASE WHEN $3 THEN $4 ELSE due_at END, \
             note = COALESCE($5, note), updated_at = now() WHERE id = $1",
    )
    .bind(package)
    .bind(name)
    .bind(due_at.is_some())
    .bind(due_at.flatten())
    .bind(note)
    .execute(pool)
    .await?;
    Ok(())
}

/// Sản lượng CTV từ gói việc (cho báo cáo nhân viên): (user_id, task, done, days, errors).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StaffWork {
    pub user_id: Uuid,
    pub task: String,
    pub done: i64,
    pub days: i64,
    pub errors: i64,
}
pub async fn staff_work(pool: &PgPool, projects: Option<&[Uuid]>, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>) -> sqlx::Result<Vec<StaffWork>> {
    let projects: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    sqlx::query_as::<_, StaffWork>(
        "SELECT i.done_by AS user_id, 'xac_nhan' AS task, count(*) AS done, \
                count(DISTINCT (i.done_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date) AS days, \
                count(*) FILTER (WHERE i.status IN ('rejected', 'checked_err') OR i.review_note IS NOT NULL AND i.status = 'redone') AS errors \
         FROM work_items i JOIN work_packages p ON p.id = i.package_id \
         WHERE i.done_by IS NOT NULL AND i.kind = 'xac_nhan' AND ($1::uuid[] IS NULL OR p.project_id = ANY($1)) \
           AND ($2::timestamptz IS NULL OR i.done_at >= $2) AND ($3::timestamptz IS NULL OR i.done_at < $3) \
         GROUP BY i.done_by \
         UNION ALL \
         SELECT i.checked_by, 'kiem_tra_nhap_lieu', count(*), \
                count(DISTINCT (i.checked_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date), 0 \
         FROM work_items i JOIN work_packages p ON p.id = i.package_id \
         WHERE i.checked_by IS NOT NULL AND ($1::uuid[] IS NULL OR p.project_id = ANY($1)) \
           AND ($2::timestamptz IS NULL OR i.checked_at >= $2) AND ($3::timestamptz IS NULL OR i.checked_at < $3) \
         GROUP BY i.checked_by",
    )
    .bind(projects)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    fn c(id: u128, hs: &str) -> Candidate {
        Candidate { id: Uuid::from_u128(id), rel_path: Some(hs.to_owned()) }
    }
    #[test]
    fn chia_theo_ho_so_khong_cat_giua_ho_so() {
        let v = vec![c(1, "P/H1/HS1"), c(2, "P/H1/HS1"), c(3, "P/H1/HS1"), c(4, "P/H1/HS2"), c(5, "P/H1/HS2"), c(6, "P/H1/HS3")];
        let g = split_hoso(&v, 2, 10);
        assert_eq!(g.len(), 3);
        assert_eq!(g[0].len(), 3, "gói 1 lấy trọn HS1 dù cỡ 2");
        assert_eq!(g[1].len(), 2);
        assert_eq!(g[2].len(), 1, "phần đuôi thành gói cuối");
        assert_eq!(split_hoso(&v, 2, 1).len(), 1, "tôn trọng số gói");
        assert!(split_hoso(&[], 5, 3).is_empty());
    }
    #[test]
    fn chia_ngau_nhien_du_va_khong_trung() {
        let v: Vec<Candidate> = (1..=10).map(|i| c(i, "x")).collect();
        let g = split_random(&v, 4, 2, 7);
        assert_eq!(g.len(), 2);
        assert!(g.iter().all(|x| x.len() == 4));
        let mut all: Vec<Uuid> = g.concat();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), 8);
        assert_ne!(split_random(&v, 10, 1, 1)[0], split_random(&v, 10, 1, 2)[0], "hạt khác → thứ tự khác");
    }
}
