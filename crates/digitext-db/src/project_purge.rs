//! "Xoá dữ liệu dự án" (15/10/2026): xoá sạch tài liệu + chỉnh lý + gói… của MỘT dự án, giữ lại vỏ dự án
//! (dự án, quản trị, thành viên, hợp đồng, tài khoản, thiết bị, nhật ký). Tệp trong kho do bên gọi xoá TRƯỚC
//! từng lô tài liệu (`page_keys_of` / `object_keys_of`), nên đứt giữa chừng chạy lại được.
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

/// Số liệu trước khi xoá (chạy thử).
#[derive(Debug, Default, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PurgeCounts {
    pub documents: i64,
    pub pages: i64,
    pub source_bytes: i64,
    pub signed: i64,
    pub packages: i64,
    pub ho_so: i64,
    pub van_ban: i64,
    pub submissions: i64,
    /// Việc OCR / PDF/A / đóng gói đang chờ hoặc đang chạy (trừ việc "chờ ký").
    pub active_jobs: i64,
    pub open_sign_sessions: i64,
}

pub async fn counts(pool: &PgPool, project_id: Uuid) -> sqlx::Result<PurgeCounts> {
    sqlx::query_as::<_, PurgeCounts>(
        "SELECT \
           (SELECT count(*) FROM documents WHERE project_id = $1) AS documents, \
           (SELECT count(*) FROM pages p JOIN documents d ON d.id = p.document_id WHERE d.project_id = $1) AS pages, \
           (SELECT COALESCE(sum(size_bytes), 0)::bigint FROM documents WHERE project_id = $1 AND source_purged_at IS NULL) AS source_bytes, \
           (SELECT count(*) FROM documents WHERE project_id = $1 AND status = 'signed') AS signed, \
           (SELECT count(*) FROM packages WHERE project_id = $1) AS packages, \
           (SELECT count(*) FROM cl_ho_so WHERE project_id = $1) AS ho_so, \
           (SELECT count(*) FROM cl_van_ban WHERE project_id = $1) AS van_ban, \
           (SELECT count(*) FROM submissions WHERE project_id = $1) AS submissions, \
           (SELECT count(*) FROM ocr_jobs j JOIN documents d ON d.id = j.document_id \
              WHERE d.project_id = $1 AND j.status IN ('queued', 'running') AND j.kind <> 'sign_wait') AS active_jobs, \
           (SELECT count(*) FROM sign_sessions s JOIN documents d ON d.id = s.document_id \
              WHERE d.project_id = $1 AND s.status = 'open' AND s.lease_until > now()) AS open_sign_sessions",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await
}

/// Lô id tài liệu tiếp theo của dự án (theo id tăng dần).
pub async fn document_batch(pool: &PgPool, project_id: Uuid, limit: i64) -> sqlx::Result<Vec<Uuid>> {
    sqlx::query_scalar("SELECT id FROM documents WHERE project_id = $1 ORDER BY id LIMIT $2")
        .bind(project_id)
        .bind(limit)
        .fetch_all(pool)
        .await
}

/// Khoá tệp trong kho của một lô tài liệu: bản gốc, PDF/A, bản ký số (đúng khoá đã lưu), ảnh trang.
pub async fn object_keys_of(pool: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar(
        "SELECT k FROM ( \
           SELECT storage_path AS k FROM documents WHERE id = ANY($1) \
           UNION SELECT pdfa_key FROM documents WHERE id = ANY($1) AND pdfa_key IS NOT NULL \
           UNION SELECT signed_key FROM sign_sessions WHERE document_id = ANY($1) AND signed_key IS NOT NULL \
           UNION SELECT prepared_key FROM sign_sessions WHERE document_id = ANY($1) AND prepared_key IS NOT NULL \
           UNION SELECT document_id::text || '/' || page_no::text || '.jpg' FROM pages WHERE document_id = ANY($1) \
         ) t WHERE k IS NOT NULL AND k <> ''",
    )
    .bind(ids)
    .fetch_all(pool)
    .await
}

/// Xoá một lô tài liệu (các bảng con tự xoá theo CASCADE). Gỡ trước dòng tiêu huỷ trỏ `dup_of` vào lô
/// (khoá ngoại NO ACTION).
pub async fn delete_documents(pool: &PgPool, ids: &[Uuid]) -> sqlx::Result<u64> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE disposal_entries SET dup_of = NULL WHERE dup_of = ANY($1)")
        .bind(ids)
        .execute(&mut *tx)
        .await?;
    let n = sqlx::query("DELETE FROM documents WHERE id = ANY($1)")
        .bind(ids)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    tx.commit().await?;
    Ok(n)
}

/// Bảng dữ liệu theo dự án xoá sau khi hết tài liệu. KHÔNG có: project_admins, project_members,
/// project_contracts, users, devices, user_stats, disposal_log (nhật ký) — vỏ dự án giữ nguyên.
const BANG_DU_LIEU: &[&str] = &[
    "disposal_entries",
    "disposal_batches",
    "integrity_checks",
    "access_requests",
    "access_fee_rates",
    "info_requests",
    "records",
    "packages",
    "submissions",
    "work_packages",
    "work_targets",
    "project_trees",
    "cl_van_ban_file",
    "cl_muon_tra",
    "cl_van_ban",
    "cl_ho_so",
    "cl_hop",
    "cl_mang",
    "cl_phong_ban",
    "cl_phong",
    "cl_khoi_phong",
    "cl_bo_tap",
    "cl_kho",
    "cl_gia",
    "cl_lich_su",
    "cl_loai_van_ban",
    "cl_ngon_ngu",
    "cl_thbq",
    "cl_sync_ops",
];

/// Khoá gói (zip) của dự án — xoá trên kho trước `delete_rest`.
pub async fn package_keys(pool: &PgPool, project_id: Uuid) -> sqlx::Result<Vec<Uuid>> {
    sqlx::query_scalar("SELECT id FROM packages WHERE project_id = $1")
        .bind(project_id)
        .fetch_all(pool)
        .await
}

/// Xoá phần dữ liệu còn lại của dự án (sau khi hết tài liệu), tắt dự án và gắn "[ĐÃ XOÁ]" vào tên.
pub async fn delete_rest(pool: &PgPool, project_id: Uuid) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;
    for bang in BANG_DU_LIEU {
        sqlx::query(&format!("DELETE FROM {bang} WHERE project_id = $1"))
            .bind(project_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "UPDATE projects SET is_active = false, \
           name = CASE WHEN name LIKE '[ĐÃ XOÁ]%' THEN name ELSE '[ĐÃ XOÁ] ' || name END \
         WHERE id = $1",
    )
    .bind(project_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}
