// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use digitext_core::{JobStatus, Tgn};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Ta2 {
    pub id: Uuid,
    pub document_id: Uuid,
    pub page_no: i32,
    pub kind: String,
    pub status: String,
    pub engine_version_id: Option<Uuid>,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub locked_by: Option<String>,
    pub lease_until: Option<DateTime<Utc>>,
    pub priority: i32,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub not_before: Option<DateTime<Utc>>,
}
impl Ta2 {
    pub fn status(&self) -> Result<JobStatus, Tgn> {
        self.status.parse()
    }
}
const COLS: &str = "id, document_id, page_no, kind, status, engine_version_id, attempts, \
     last_error, locked_by, lease_until, priority, seq, created_at, updated_at, not_before";
pub async fn f_19s(r_270: &PgPool, document_id: Uuid, page_no: i32) -> sqlx::Result<Ta2> {
    sqlx::query(
        "INSERT INTO ocr_jobs (id, document_id, page_no, kind) VALUES ($1, $2, $3, 'ocr') \
         ON CONFLICT (document_id, page_no) WHERE kind = 'ocr' DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(document_id)
    .bind(page_no)
    .execute(r_270)
    .await?;
    sqlx::query_as::<_, Ta2>(&format!(
        "SELECT {COLS} FROM ocr_jobs WHERE document_id = $1 AND page_no = $2 AND kind = 'ocr'"
    ))
    .bind(document_id)
    .bind(page_no)
    .fetch_one(r_270)
    .await
}
pub async fn f_19m(r_272: &PgPool, document_id: Uuid, r_271: i32) -> sqlx::Result<i64> {
    if r_271 <= 0 {
        return Ok(0);
    }
    let mut r_273 = r_272.begin().await?;
    for page_no in 1..=r_271 {
        sqlx::query(
            "INSERT INTO ocr_jobs (id, document_id, page_no, kind) VALUES ($1, $2, $3, 'ocr') \
             ON CONFLICT (document_id, page_no) WHERE kind = 'ocr' DO NOTHING",
        )
        .bind(Uuid::now_v7())
        .bind(document_id)
        .bind(page_no)
        .execute(&mut *r_273)
        .await?;
    }
    sqlx::query(
        "UPDATE documents SET status = 'ocr_pending' WHERE id = $1 AND status = 'uploaded'",
    )
    .bind(document_id)
    .execute(&mut *r_273)
    .await?;
    r_273.commit().await?;
    Ok(r_271 as i64)
}
/// Xếp việc OCR cho nhiều tài liệu một lượt: mỗi khối 20.000 trang là MỘT câu INSERT (UNNEST)
/// thay vì một câu mỗi trang. 25/9/2026: «OCR toàn bộ» 30.000 tài liệu / 70.000 trang chạy
/// nhiều phút, client hết giờ chờ → máy chủ huỷ giữa chừng (mới xếp 13.585 trang).
pub async fn enqueue_ocr_bulk(pool: &PgPool, docs: &[(Uuid, i32)]) -> sqlx::Result<i64> {
    let mut ids: Vec<Uuid> = Vec::new();
    let mut doc_ids: Vec<Uuid> = Vec::new();
    let mut pages: Vec<i32> = Vec::new();
    for &(doc, n) in docs {
        for page_no in 1..=n.max(0) {
            ids.push(Uuid::now_v7());
            doc_ids.push(doc);
            pages.push(page_no);
        }
    }
    let mut tx = pool.begin().await?;
    const CHUNK: usize = 20_000;
    let mut start = 0;
    while start < ids.len() {
        let end = (start + CHUNK).min(ids.len());
        sqlx::query(
            "INSERT INTO ocr_jobs (id, document_id, page_no, kind) \
             SELECT i, d, p, 'ocr' FROM UNNEST($1::uuid[], $2::uuid[], $3::int4[]) AS t(i, d, p) \
             ON CONFLICT (document_id, page_no) WHERE kind = 'ocr' DO NOTHING",
        )
        .bind(&ids[start..end])
        .bind(&doc_ids[start..end])
        .bind(&pages[start..end])
        .execute(&mut *tx)
        .await?;
        start = end;
    }
    let touched: Vec<Uuid> = docs.iter().filter(|(_, n)| *n > 0).map(|(d, _)| *d).collect();
    sqlx::query("UPDATE documents SET status = 'ocr_pending' WHERE id = ANY($1) AND status = 'uploaded'")
        .bind(&touched)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(ids.len() as i64)
}
pub const C_BG: i32 = 0;
/// Ưu tiên việc tạo PDF 2 lớp: TRƯỚC OCR (100). Tài liệu OCR xong trang cuối là ghép PDF/A
/// ngay (CPU máy chủ, ~0,4 s/trang) thay vì đợi cả kho OCR xong (25/9/2026: 800 việc đứng
/// sau 91.000 trang OCR, ~17 ngày).
const C_BH: i32 = 50;
const C_K8: i32 = 300;
const C_6Q: i32 = 150;
pub async fn f_19p(r_274: &PgPool, document_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO ocr_jobs (id, document_id, page_no, kind, priority) \
         VALUES ($1, $2, $3, 'extract_fields', $4) \
         ON CONFLICT (document_id) \
             WHERE kind = 'extract_fields' AND status IN ('queued', 'running') DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(document_id)
    .bind(C_BG)
    .bind(C_6Q)
    .execute(r_274)
    .await?;
    Ok(())
}
pub async fn f_11x(r_275: &PgPool, kind: &str, r_276: Option<&[Uuid]>) -> sqlx::Result<i64> {
    f_11y(r_275, kind, r_276, None).await
}
pub async fn f_11y(
    r_27a: &PgPool,
    kind: &str,
    r_27c: Option<&[Uuid]>,
    r_277: Option<&[Uuid]>,
) -> sqlx::Result<i64> {
    let r_27b: Option<Vec<Uuid>> = r_27c.map(|r_279| r_279.to_vec());
    let r_278: Option<Vec<Uuid>> = r_277.map(|d| d.to_vec());
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*)::bigint FROM documents d \
         WHERE d.status <> 'disposed' \
           AND ($2::uuid[] IS NULL OR d.project_id = ANY($2)) \
           AND ($3::uuid[] IS NULL OR d.id = ANY($3)) \
           AND EXISTS ( \
               SELECT 1 FROM ocr_jobs j \
               WHERE j.document_id = d.id AND j.kind = $1 AND j.status = 'page_failed' \
           ) \
           AND NOT EXISTS ( \
               SELECT 1 FROM ocr_jobs j WHERE j.document_id = d.id AND j.kind = $1 \
                 AND j.status IN ('queued', 'running', 'done') \
           )",
    )
    .bind(kind)
    .bind(r_27b)
    .bind(r_278)
    .fetch_one(r_27a)
    .await
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExtractFieldsProgress {
    pub done: i64,
    pub active: i64,
    pub failed: i64,
}
pub async fn f_1ax(r_27e: &PgPool, r_27g: Option<&[Uuid]>) -> sqlx::Result<ExtractFieldsProgress> {
    let r_27f: Option<Vec<Uuid>> = r_27g.map(|p| p.to_vec());
    let r_27h = sqlx::query_as::<_, (String, i64)>(
        "SELECT j.status, count(*)::bigint FROM ( \
             SELECT DISTINCT ON (o.document_id) o.document_id, o.status \
             FROM ocr_jobs o JOIN documents d ON d.id = o.document_id \
             WHERE o.kind = 'extract_fields' \
               AND ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
             ORDER BY o.document_id, o.created_at DESC, o.id DESC \
         ) j GROUP BY j.status",
    )
    .bind(r_27f)
    .fetch_all(r_27e)
    .await?;
    let mut p = ExtractFieldsProgress::default();
    for (status, r_27d) in r_27h {
        match status.as_str() {
            "done" => p.done += r_27d,
            "queued" | "running" => p.active += r_27d,
            "failed_transient" | "page_failed" => p.failed += r_27d,
            _ => {}
        }
    }
    Ok(p)
}
pub async fn f_19r(r_27i: &PgPool, document_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO ocr_jobs (id, document_id, page_no, kind, priority) \
         VALUES ($1, $2, $3, 'make_pdf2', $4) \
         ON CONFLICT (document_id) \
             WHERE kind = 'make_pdf2' AND status IN ('queued', 'running') DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(document_id)
    .bind(C_BG)
    .bind(C_BH)
    .execute(r_27i)
    .await?;
    Ok(())
}
pub async fn f_19u(r_27j: &PgPool, document_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO ocr_jobs (id, document_id, page_no, kind, priority) \
         VALUES ($1, $2, $3, 'sign_wait', $4) \
         ON CONFLICT (document_id) \
             WHERE kind = 'sign_wait' AND status IN ('queued', 'running') DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(document_id)
    .bind(C_BG)
    .bind(C_K8)
    .execute(r_27j)
    .await?;
    Ok(())
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OcrPageCounts {
    pub pending: i64,
    pub failed_transient: i64,
    pub page_failed: i64,
    pub done: i64,
}
pub async fn ocr_page_counts(r_27k: &PgPool, document_id: Uuid) -> sqlx::Result<OcrPageCounts> {
    let (pending, failed_transient, page_failed, done) =
        sqlx::query_as::<_, (i64, i64, i64, i64)>(
            "SELECT count(*) FILTER (WHERE status IN ('queued', 'running'))::bigint, \
                    count(*) FILTER (WHERE status = 'failed_transient')::bigint, \
                    count(*) FILTER (WHERE status = 'page_failed')::bigint, \
                    count(*) FILTER (WHERE status = 'done')::bigint \
             FROM ocr_jobs WHERE document_id = $1 AND kind IN ('ocr', 're_ocr')",
        )
        .bind(document_id)
        .fetch_one(r_27k)
        .await?;
    Ok(OcrPageCounts {
        pending,
        failed_transient,
        page_failed,
        done,
    })
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OcrFailedCounts {
    pub docs_failed: i64,
    pub pages_failed_transient: i64,
    pub pages_page_failed: i64,
    pub pages_queued: i64,
    pub pages_running: i64,
}
/// Tài liệu OCR xong nhưng ghép PDF 2 lớp bị TỪ CHỐI VĨNH VIỄN (page_failed — vd. bản quét
/// < 200 dpi, Điều 8.2.a TT05) và vẫn chưa có PDF/A. Trả (id, lỗi lần gần nhất). Bấm "PDF 2
/// lớp" lại cũng bị từ chối y hệt — phải quét lại (28/9/2026: 83 tài liệu, 665 lần thử lại).
pub async fn pdf2_rejected(
    pool: &PgPool,
    projects: Option<&[Uuid]>,
) -> sqlx::Result<Vec<(Uuid, String)>> {
    let projects: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    sqlx::query_as::<_, (Uuid, String)>(
        "SELECT DISTINCT ON (j.document_id) j.document_id, COALESCE(j.last_error, '') \
         FROM ocr_jobs j JOIN documents d ON d.id = j.document_id \
         WHERE j.kind = 'make_pdf2' AND j.status = 'page_failed' \
           AND d.pdfa_key IS NULL AND d.status = 'ocr_done' \
           AND ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
         ORDER BY j.document_id, j.updated_at DESC",
    )
    .bind(projects)
    .fetch_all(pool)
    .await
}
/// Một tài liệu bị từ chối ghép PDF 2 lớp (vd. bản quét dưới 200 dpi) — đủ cột để xuất báo cáo
/// giao người quét lại.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Pdf2RejectedRow {
    pub document_id: Uuid,
    pub project_code: String,
    pub rel_path: Option<String>,
    pub filename: String,
    pub page_count: Option<i32>,
    pub size_bytes: i64,
    pub uploaded_at: DateTime<Utc>,
    pub reason: String,
    /// "rejected" = chưa có PDF/A; "exception" = đã ghép theo ngoại lệ dự án, vẫn cần quét lại.
    pub outcome: String,
}
pub async fn pdf2_rejected_rows(
    pool: &PgPool,
    projects: Option<&[Uuid]>,
) -> sqlx::Result<Vec<Pdf2RejectedRow>> {
    let projects: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    sqlx::query_as::<_, Pdf2RejectedRow>(
        "SELECT j.document_id, p.code AS project_code, d.rel_path, d.filename, d.page_count, \
                d.size_bytes, d.uploaded_at, COALESCE(j.last_error, '') AS reason, \
                CASE WHEN j.status = 'done' THEN 'exception' ELSE 'rejected' END AS outcome \
         FROM (SELECT DISTINCT ON (document_id) document_id, status, last_error \
               FROM ocr_jobs WHERE kind = 'make_pdf2' \
                 AND (status = 'page_failed' OR (status = 'done' AND last_error LIKE 'Ngoại lệ dự án:%')) \
               ORDER BY document_id, updated_at DESC) j \
         JOIN documents d ON d.id = j.document_id \
         JOIN projects p ON p.id = d.project_id \
         WHERE (j.status = 'done' OR d.pdfa_key IS NULL) \
           AND ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
         ORDER BY p.code, d.rel_path NULLS LAST, d.filename",
    )
    .bind(projects)
    .fetch_all(pool)
    .await
}
pub async fn ocr_failed_counts(
    r_27k: &PgPool,
    projects: Option<&[Uuid]>,
) -> sqlx::Result<OcrFailedCounts> {
    let projects: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    let (docs_failed, pages_failed_transient, pages_page_failed, pages_queued, pages_running) =
        sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
            "SELECT count(DISTINCT j.document_id) \
                        FILTER (WHERE j.status IN ('failed_transient', 'page_failed'))::bigint, \
                    count(*) FILTER (WHERE j.status = 'failed_transient')::bigint, \
                    count(*) FILTER (WHERE j.status = 'page_failed')::bigint, \
                    count(*) FILTER (WHERE j.status = 'queued')::bigint, \
                    count(*) FILTER (WHERE j.status = 'running')::bigint \
             FROM ocr_jobs j JOIN documents d ON d.id = j.document_id \
             WHERE j.kind IN ('ocr', 're_ocr') AND d.status <> 'disposed' \
               AND ($1::uuid[] IS NULL OR d.project_id = ANY($1))",
        )
        .bind(projects)
        .fetch_one(r_27k)
        .await?;
    Ok(OcrFailedCounts {
        docs_failed,
        pages_failed_transient,
        pages_page_failed,
        pages_queued,
        pages_running,
    })
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RequeueResult {
    pub pages: u64,
    pub documents: u64,
    pub skipped_signed: u64,
}
pub async fn requeue_failed(
    r_27k: &PgPool,
    projects: Option<&[Uuid]>,
    docs: Option<&[Uuid]>,
    include_page_failed: bool,
) -> sqlx::Result<RequeueResult> {
    let projects: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    let docs: Option<Vec<Uuid>> = docs.map(|d| d.to_vec());
    let statuses: Vec<String> = if include_page_failed {
        vec!["failed_transient".into(), "page_failed".into()]
    } else {
        vec!["failed_transient".into()]
    };
    let mut tx = r_27k.begin().await?;
    let candidates = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT d.id, d.status FROM documents d \
         WHERE d.status <> 'disposed' \
           AND ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
           AND ($2::uuid[] IS NULL OR d.id = ANY($2)) \
           AND EXISTS ( \
               SELECT 1 FROM ocr_jobs j WHERE j.document_id = d.id \
                 AND j.kind IN ('ocr', 're_ocr') AND j.status = ANY($3) \
           ) \
         FOR UPDATE OF d",
    )
    .bind(&projects)
    .bind(&docs)
    .bind(&statuses)
    .fetch_all(&mut *tx)
    .await?;
    let mut eligible: Vec<Uuid> = Vec::new();
    let mut skipped_signed = 0u64;
    for (id, status) in candidates {
        match status.as_str() {
            "uploaded" | "ocr_pending" | "ocr_done" => eligible.push(id),
            _ => skipped_signed += 1,
        }
    }
    if eligible.is_empty() {
        tx.commit().await?;
        return Ok(RequeueResult {
            pages: 0,
            documents: 0,
            skipped_signed,
        });
    }
    let pages = sqlx::query(
        "UPDATE ocr_jobs SET status = 'queued', attempts = 0, last_error = NULL, \
             not_before = NULL, locked_by = NULL, lease_until = NULL, updated_at = now() \
         WHERE kind IN ('ocr', 're_ocr') AND status = ANY($2) AND document_id = ANY($1)",
    )
    .bind(&eligible)
    .bind(&statuses)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    sqlx::query(
        "UPDATE documents SET status = 'ocr_pending' \
         WHERE id = ANY($1) AND status IN ('uploaded', 'ocr_done')",
    )
    .bind(&eligible)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE ocr_jobs SET status = 'cancelled', locked_by = NULL, lease_until = NULL, \
             updated_at = now() \
         WHERE kind IN ('make_pdf2', 'sign_wait') AND status = 'queued' \
           AND document_id = ANY($1)",
    )
    .bind(&eligible)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(RequeueResult {
        pages,
        documents: eligible.len() as u64,
        skipped_signed,
    })
}
pub async fn claim_next(r_27m: &PgPool, r_27n: &str, r_27l: i64) -> sqlx::Result<Option<Ta2>> {
    f_zj(
        r_27m,
        r_27n,
        r_27l,
        &["ocr", "re_ocr", "audit", "make_pdf2"],
    )
    .await
}
pub async fn f_zj(
    r_27q: &PgPool,
    r_27r: &str,
    r_27p: i64,
    kinds: &[&str],
) -> sqlx::Result<Option<Ta2>> {
    let kinds: Vec<String> = kinds.iter().map(|r_27o| (*r_27o).to_owned()).collect();
    // UPDATE không chạm dòng nào vẫn làm trigger phát sự kiện "ocr" cho mọi máy trạm — chỉ UPDATE khi có việc chờ.
    let any: bool = sqlx::query_scalar(
        // Dò theo từng loại qua unnest (13/10/2026): `kind = ANY($1)` ở plan chung (câu lệnh chuẩn bị sẵn) bị ước
        // lượng như mọi việc chờ đều khớp → quét cả bảng ~870 nghìn dòng (≈200 ms) — 146 nghìn việc sign_wait chờ ký
        // làm Postgres ăn 4 nhân, kho MinIO treo ổ. Cần chỉ mục ix_ocr_jobs_queued_kind_nb (kind, not_before) WHERE status='queued'.
        "SELECT EXISTS(SELECT 1 FROM unnest($1::text[]) AS k(kind) WHERE EXISTS(SELECT 1 FROM ocr_jobs j \
         WHERE j.status = 'queued' AND j.kind = k.kind AND (j.not_before IS NULL OR j.not_before <= now())))",
    )
    .bind(&kinds)
    .fetch_one(r_27q)
    .await?;
    if !any {
        return Ok(None);
    }
    sqlx::query_as::<_, Ta2>(&format!(
        "UPDATE ocr_jobs SET status = 'running', locked_by = $1, \
             lease_until = now() + ($2 * interval '1 second'), updated_at = now() \
         WHERE id = ( \
             SELECT id FROM ocr_jobs \
             WHERE status = 'queued' AND kind = ANY($3) \
               AND (not_before IS NULL OR not_before <= now()) \
             ORDER BY priority, seq FOR UPDATE SKIP LOCKED LIMIT 1 \
         ) RETURNING {COLS}"
    ))
    .bind(r_27r)
    .bind(r_27p)
    .bind(&kinds)
    .fetch_optional(r_27q)
    .await
}
pub async fn f_1gk(r_27u: &PgPool, r_27s: Uuid, r_27w: &str, r_27t: i64) -> sqlx::Result<bool> {
    let r_27v = sqlx::query(
        "UPDATE ocr_jobs SET lease_until = now() + ($3 * interval '1 second'), updated_at = now() \
         WHERE id = $1 AND locked_by = $2 AND status = 'running'",
    )
    .bind(r_27s)
    .bind(r_27w)
    .bind(r_27t)
    .execute(r_27u)
    .await?;
    Ok(r_27v.rows_affected() > 0)
}
pub async fn requeue_expired(r_27y: &PgPool, r_27x: i32) -> sqlx::Result<u64> {
    // Như packages::requeue_expired: UPDATE 0 dòng vẫn làm trigger phát sự kiện "ocr" → chỉ chạy khi thật có.
    let any: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM ocr_jobs WHERE status = 'running' AND lease_until < now())")
        .fetch_one(r_27y)
        .await?;
    if !any {
        return Ok(0);
    }
    let r_27z = sqlx::query(
            "UPDATE ocr_jobs SET attempts = attempts + 1, last_error = COALESCE(last_error, 'lease hết hạn — worker không phản hồi'), status = CASE WHEN attempts + 1 > $1 THEN 'failed_transient' ELSE 'queued' END, not_before = CASE WHEN attempts + 1 > $1 THEN NULL ELSE now() + interval '5 seconds' END, locked_by = NULL, lease_until = NULL, updated_at = now() WHERE status = 'running' AND lease_until < now()",
        )
        .bind(r_27x)
        .execute(r_27y)
        .await?;
    Ok(r_27z.rows_affected())
}
/// Gỡ tài liệu kẹt sau OCR: mọi trang đã xong (không còn trang chờ / đang chạy / lỗi tạm thời) mà tài liệu vẫn
/// `ocr_pending`, hoặc đã `ocr_done` mà chưa từng có việc ghép PDF/A. Bước "trang cuối xong → ocr_done +
/// make_pdf2" của worker chạy một lần, CSDL chập chờn đúng lúc đó thì lỡ mãi (7/10/2026: 13 tài liệu kẹt từ
/// 2–4/10). Trả (số chuyển ocr_done, số xếp make_pdf2). Chỉ ghi khi có tài liệu kẹt — UPDATE 0 dòng vẫn làm
/// trigger phát sự kiện.
pub async fn sweep_stuck_after_ocr(pool: &PgPool, enqueue_pdf2: bool) -> sqlx::Result<(u64, u64)> {
    let pending: Vec<Uuid> = sqlx::query_scalar(
        "SELECT d.id FROM documents d \
          WHERE d.status = 'ocr_pending' AND d.pdfa_key IS NULL \
            AND EXISTS (SELECT 1 FROM ocr_jobs j WHERE j.document_id = d.id AND j.kind = 'ocr') \
            AND NOT EXISTS (SELECT 1 FROM ocr_jobs j WHERE j.document_id = d.id AND j.kind = 'ocr' \
                              AND j.status IN ('queued', 'running', 'failed_transient')) \
          LIMIT 500",
    )
    .fetch_all(pool)
    .await?;
    let mut done = 0;
    for id in &pending {
        if crate::documents::transition(pool, *id, digitext_core::DocStatus::OcrDone).await? {
            done += 1;
        }
    }
    let mut queued = 0;
    if enqueue_pdf2 {
        let no_pdf2: Vec<Uuid> = sqlx::query_scalar(
            "SELECT d.id FROM documents d \
              WHERE d.status = 'ocr_done' AND d.pdfa_key IS NULL \
                AND NOT EXISTS (SELECT 1 FROM ocr_jobs j WHERE j.document_id = d.id AND j.kind = 'make_pdf2') \
              LIMIT 500",
        )
        .fetch_all(pool)
        .await?;
        for id in &no_pdf2 {
            f_19r(pool, *id).await?;
            queued += 1;
        }
    }
    Ok((done, queued))
}
pub async fn f_1ra(r_281: &PgPool, r_280: Uuid, r_283: &str) -> sqlx::Result<bool> {
    let r_282 = sqlx::query(
        "UPDATE ocr_jobs SET status = 'done', locked_by = NULL, lease_until = NULL, \
             updated_at = now() WHERE id = $1 AND status = 'running' AND locked_by = $2",
    )
    .bind(r_280)
    .bind(r_283)
    .execute(r_281)
    .await?;
    Ok(r_282.rows_affected() > 0)
}
/// Tiền tố ghi chú khi job make_pdf2 XONG nhưng theo ngoại lệ dự án (bản quét dưới 200 dpi) —
/// báo cáo PDF/A vẫn liệt kê để chủ dự án biết cần quét lại.
pub const PDF2_EXCEPTION_PREFIX: &str = "Ngoại lệ dự án: đã ghép PDF/A dù ";
/// Đánh dấu job xong nhưng giữ ghi chú trong last_error (ngoại lệ dự án).
pub async fn done_with_note(pool: &PgPool, id: Uuid, worker_id: &str, note: &str) -> sqlx::Result<bool> {
    let r = sqlx::query(
        "UPDATE ocr_jobs SET status = 'done', last_error = $3, locked_by = NULL, lease_until = NULL, \
             updated_at = now() WHERE id = $1 AND status = 'running' AND locked_by = $2",
    )
    .bind(id)
    .bind(worker_id)
    .bind(note)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}
pub async fn f_1re(r_286: &PgPool, r_285: Uuid, r_288: &str, r_284: &str) -> sqlx::Result<bool> {
    let r_287 = sqlx::query(
        "UPDATE ocr_jobs SET status = 'page_failed', last_error = $3, locked_by = NULL, \
             lease_until = NULL, updated_at = now() \
         WHERE id = $1 AND status = 'running' AND locked_by = $2",
    )
    .bind(r_285)
    .bind(r_288)
    .bind(r_284)
    .execute(r_286)
    .await?;
    Ok(r_287.rows_affected() > 0)
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransientOutcome {
    pub status: String,
    pub attempts: i32,
}
impl TransientOutcome {
    pub fn requeued(&self) -> bool {
        self.status == "queued"
    }
}
pub async fn f_1bn(
    r_28c: &PgPool,
    r_28a: Uuid,
    r_28d: &str,
    r_289: &str,
    r_28b: i32,
    delay_ms: i64,
) -> sqlx::Result<Option<TransientOutcome>> {
    let row = sqlx::query_as::<_, (String, i32)>(
        "UPDATE ocr_jobs SET attempts = attempts + 1, last_error = $3, \
             status = CASE WHEN attempts + 1 > $4 THEN 'failed_transient' ELSE 'queued' END, \
             not_before = CASE WHEN attempts + 1 > $4 THEN NULL \
                               ELSE now() + ($5 * interval '1 millisecond') END, \
             locked_by = NULL, lease_until = NULL, updated_at = now() \
         WHERE id = $1 AND status = 'running' AND locked_by = $2 \
         RETURNING status, attempts",
    )
    .bind(r_28a)
    .bind(r_28d)
    .bind(r_289)
    .bind(r_28b)
    .bind(delay_ms.max(0))
    .fetch_optional(r_28c)
    .await?;
    Ok(row.map(|(status, attempts)| TransientOutcome { status, attempts }))
}
pub async fn cancel(r_28f: &PgPool, r_28e: Uuid) -> sqlx::Result<bool> {
    let r_28g = sqlx::query(
        "UPDATE ocr_jobs SET status = 'cancelled', locked_by = NULL, lease_until = NULL, \
             updated_at = now() \
         WHERE id = $1 AND status IN ('queued', 'running', 'failed_transient')",
    )
    .bind(r_28e)
    .execute(r_28f)
    .await?;
    Ok(r_28g.rows_affected() > 0)
}
pub async fn get(r_28i: &PgPool, r_28h: Uuid) -> sqlx::Result<Option<Ta2>> {
    sqlx::query_as::<_, Ta2>(&format!("SELECT {COLS} FROM ocr_jobs WHERE id = $1"))
        .bind(r_28h)
        .fetch_optional(r_28i)
        .await
}
pub async fn count_by_status(r_28j: &PgPool) -> sqlx::Result<Vec<(String, i64)>> {
    sqlx::query_as::<_, (String, i64)>("SELECT status, count(*) FROM ocr_jobs GROUP BY status")
        .fetch_all(r_28j)
        .await
}
#[cfg(test)]
mod tests {
    use super::*;
    // Chạy khi có DIGITEXT_TEST_DATABASE_URL (CSDL đã áp migrations/). Không có thì bỏ qua
    // lặng lẽ — repo không có CI, và test này tạo dữ liệu thật nên phải trỏ vào CSDL thử.
    async fn pool() -> Option<PgPool> {
        let url = std::env::var("DIGITEXT_TEST_DATABASE_URL").ok()?;
        Some(crate::connect(&url).await.expect("kết nối CSDL thử"))
    }
    async fn fixture_document(pool: &PgPool, pages: i32) -> (Uuid, Uuid) {
        let project = Uuid::now_v7();
        let doc = Uuid::now_v7();
        sqlx::query("INSERT INTO projects (id, code, name) VALUES ($1, $2, $3)")
            .bind(project)
            .bind(format!("T{}", &project.simple().to_string()[..8]))
            .bind("test ocr_jobs")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO documents (id, project_id, filename, size_bytes, blake3, storage_path, \
                 page_count, status) VALUES ($1, $2, 'a.pdf', 1, $3, 'x', $4, 'uploaded')",
        )
        .bind(doc)
        .bind(project)
        .bind(doc.simple().to_string())
        .bind(pages)
        .execute(pool)
        .await
        .unwrap();
        (project, doc)
    }
    async fn cleanup(pool: &PgPool, project: Uuid) {
        sqlx::query("DELETE FROM ocr_jobs WHERE document_id IN (SELECT id FROM documents WHERE project_id = $1)")
            .bind(project).execute(pool).await.unwrap();
        sqlx::query("DELETE FROM documents WHERE project_id = $1")
            .bind(project).execute(pool).await.unwrap();
        sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(project).execute(pool).await.unwrap();
    }
    async fn doc_status(pool: &PgPool, doc: Uuid) -> String {
        sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
            .bind(doc)
            .fetch_one(pool)
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn backoff_blocks_claim_then_requeue_failed_restores() {
        let Some(pool) = pool().await else { return };
        let (project, doc) = fixture_document(&pool, 2).await;
        let worker = format!("t-{}", Uuid::now_v7());
        let kinds = &["ocr"];
        assert_eq!(f_19m(&pool, doc, 2).await.unwrap(), 2);
        assert_eq!(doc_status(&pool, doc).await, "ocr_pending");

        // Lượt 1: lỗi tạm thời với backoff 60 s → job về queued nhưng chưa claim được.
        let job = f_zj(&pool, &worker, 60, kinds).await.unwrap().unwrap();
        let out = f_1bn(&pool, job.id, &worker, "timeout", 5, 60_000).await.unwrap().unwrap();
        assert!(out.requeued());
        assert_eq!(out.attempts, 1);
        // Job còn lại (trang 2) claim được; job vừa lỗi thì không.
        let other = f_zj(&pool, &worker, 60, kinds).await.unwrap().unwrap();
        assert_ne!(other.id, job.id);
        assert!(f_zj(&pool, &worker, 60, kinds).await.unwrap().is_none());
        f_1ra(&pool, other.id, &worker).await.unwrap();

        // Hết số lần thử → failed_transient, không còn not_before.
        sqlx::query("UPDATE ocr_jobs SET not_before = NULL, attempts = 5 WHERE id = $1")
            .bind(job.id).execute(&pool).await.unwrap();
        let job = f_zj(&pool, &worker, 60, kinds).await.unwrap().unwrap();
        let out = f_1bn(&pool, job.id, &worker, "timeout", 5, 60_000).await.unwrap().unwrap();
        assert_eq!(out.status, "failed_transient");
        let c = ocr_page_counts(&pool, doc).await.unwrap();
        assert_eq!(c, OcrPageCounts { pending: 0, failed_transient: 1, page_failed: 0, done: 1 });
        let f = ocr_failed_counts(&pool, Some(&[project])).await.unwrap();
        assert_eq!(f.docs_failed, 1);
        assert_eq!(f.pages_failed_transient, 1);

        // Giả lập bug cũ: tài liệu bị ghi ocr_done dù thiếu trang → requeue_failed sửa lại.
        sqlx::query("UPDATE documents SET status = 'ocr_done' WHERE id = $1")
            .bind(doc).execute(&pool).await.unwrap();
        let r = requeue_failed(&pool, Some(&[project]), None, false).await.unwrap();
        assert_eq!(r, RequeueResult { pages: 1, documents: 1, skipped_signed: 0 });
        assert_eq!(doc_status(&pool, doc).await, "ocr_pending");
        let job = f_zj(&pool, &worker, 60, kinds).await.unwrap().unwrap();
        assert_eq!(job.attempts, 0);
        assert!(job.not_before.is_none());

        // Tài liệu đã sang to_sign thì bị bỏ qua, không kéo ngược.
        f_1bn(&pool, job.id, &worker, "timeout", 0, 0).await.unwrap();
        sqlx::query("UPDATE documents SET status = 'to_sign' WHERE id = $1")
            .bind(doc).execute(&pool).await.unwrap();
        let r = requeue_failed(&pool, Some(&[project]), None, false).await.unwrap();
        assert_eq!(r, RequeueResult { pages: 0, documents: 0, skipped_signed: 1 });
        assert_eq!(doc_status(&pool, doc).await, "to_sign");

        cleanup(&pool, project).await;
    }
}
