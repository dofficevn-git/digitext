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
}
impl Ta2 {
    pub fn status(&self) -> Result<JobStatus, Tgn> {
        self.status.parse()
    }
}
const COLS: &str = "id, document_id, page_no, kind, status, engine_version_id, attempts, \
     last_error, locked_by, lease_until, priority, seq, created_at, updated_at";
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
pub const C_BG: i32 = 0;
const C_BH: i32 = 200;
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
pub async fn f_2na(r_27k: &PgPool, document_id: Uuid) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM ocr_jobs \
         WHERE document_id = $1 AND kind IN ('ocr', 're_ocr') \
           AND status IN ('queued', 'running')",
    )
    .bind(document_id)
    .fetch_one(r_27k)
    .await
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
    sqlx::query_as::<_, Ta2>(&format!(
        "UPDATE ocr_jobs SET status = 'running', locked_by = $1, \
             lease_until = now() + ($2 * interval '1 second'), updated_at = now() \
         WHERE id = ( \
             SELECT id FROM ocr_jobs \
             WHERE status = 'queued' AND kind = ANY($3) \
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
    let r_27z = sqlx::query(
            "UPDATE ocr_jobs SET attempts = attempts + 1, last_error = COALESCE(last_error, 'lease hết hạn — worker không phản hồi'), status = CASE WHEN attempts + 1 > $1 THEN 'failed_transient' ELSE 'queued' END, locked_by = NULL, lease_until = NULL, updated_at = now() WHERE status = 'running' AND lease_until < now()",
        )
        .bind(r_27x)
        .execute(r_27y)
        .await?;
    Ok(r_27z.rows_affected())
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
pub async fn f_1bn(
    r_28c: &PgPool,
    r_28a: Uuid,
    r_28d: &str,
    r_289: &str,
    r_28b: i32,
) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar::<_, String>(
        "UPDATE ocr_jobs SET attempts = attempts + 1, last_error = $3, \
             status = CASE WHEN attempts + 1 > $4 THEN 'failed_transient' ELSE 'queued' END, \
             locked_by = NULL, lease_until = NULL, updated_at = now() \
         WHERE id = $1 AND status = 'running' AND locked_by = $2 RETURNING status",
    )
    .bind(r_28a)
    .bind(r_28d)
    .bind(r_289)
    .bind(r_28b)
    .fetch_optional(r_28c)
    .await
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
