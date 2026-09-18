// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use digitext_core::DocStatus;
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Document {
    pub id: Uuid,
    pub project_id: Uuid,
    pub uploaded_by: Option<Uuid>,
    pub filename: String,
    pub ho_so_ref: Option<String>,
    pub page_count: Option<i32>,
    pub size_bytes: i64,
    pub blake3: String,
    pub storage_path: String,
    pub status: String,
    pub uploaded_at: DateTime<Utc>,
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DocRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub filename: String,
    pub ho_so_ref: Option<String>,
    pub page_count: Option<i32>,
    pub size_bytes: i64,
    pub status: String,
    pub uploaded_by_name: Option<String>,
    pub uploaded_at: DateTime<Utc>,
    pub review_status: String,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_note: Option<String>,
}
#[derive(Debug)]
pub struct Tdt<'a> {
    pub project_id: Uuid,
    pub uploaded_by: Option<Uuid>,
    pub filename: &'a str,
    pub ho_so_ref: Option<&'a str>,
    pub page_count: Option<i32>,
    pub size_bytes: i64,
    pub blake3: &'a str,
    pub storage_path: &'a str,
    pub batch_id: Option<Uuid>,
    pub rel_path: Option<&'a str>,
}
pub async fn find_by_hash(
    r_23l: &PgPool,
    project_id: Uuid,
    blake3: &str,
) -> sqlx::Result<Option<Document>> {
    sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE project_id = $1 AND blake3 = $2")
        .bind(project_id)
        .bind(blake3)
        .fetch_optional(r_23l)
        .await
}
pub async fn insert(r_23m: &PgPool, d: Tdt<'_>) -> sqlx::Result<Option<Document>> {
    sqlx::query_as::<
        _,
        Document,
    >(
            "INSERT INTO documents \
            (id, project_id, uploaded_by, filename, ho_so_ref, page_count, size_bytes, blake3, storage_path, batch_id, rel_path) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) \
         ON CONFLICT (project_id, blake3) DO NOTHING \
         RETURNING *",
        )
        .bind(Uuid::now_v7())
        .bind(d.project_id)
        .bind(d.uploaded_by)
        .bind(d.filename)
        .bind(d.ho_so_ref)
        .bind(d.page_count)
        .bind(d.size_bytes)
        .bind(d.blake3)
        .bind(d.storage_path)
        .bind(d.batch_id)
        .bind(d.rel_path)
        .fetch_optional(r_23m)
        .await
}
pub async fn get(r_23n: &PgPool, id: Uuid) -> sqlx::Result<Option<Document>> {
    sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
        .bind(id)
        .fetch_optional(r_23n)
        .await
}
/// Khoá object của bản PDF/A (NULL khi chưa ghép PDF 2 lớp).
pub async fn pdfa_key(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar::<_, Option<String>>("SELECT pdfa_key FROM documents WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map(|r| r.flatten())
}
const DOC_COLS: &str = "d.id, d.project_id, d.filename, d.ho_so_ref, d.page_count, \
     d.size_bytes, d.status, u.display_name AS uploaded_by_name, d.uploaded_at, \
     d.review_status, d.reviewed_at, d.review_note";
pub async fn transition(r_23p: &PgPool, id: Uuid, r_23q: DocStatus) -> sqlx::Result<bool> {
    let r_23o: Vec<String> = DocStatus::ALL
        .iter()
        .filter(|s| s.can_transition_to(r_23q))
        .map(|s| s.as_str().to_owned())
        .collect();
    let r = sqlx::query("UPDATE documents SET status = $1 WHERE id = $2 AND status = ANY($3)")
        .bind(r_23q.as_str())
        .bind(id)
        .bind(&r_23o)
        .execute(r_23p)
        .await?;
    Ok(r.rows_affected() > 0)
}
#[derive(Debug, Clone, Copy)]
pub struct Tgy<'a> {
    pub key: &'a str,
    pub pages: i32,
    pub pdfa_2b: bool,
}
pub async fn f_1rf(r_23s: &PgPool, id: Uuid, r_23r: Tgy<'_>) -> sqlx::Result<bool> {
    let r = sqlx::query(
        "UPDATE documents \
            SET pdfa_key = $2, pdfa_pages = $3, pdfa_2b = $4, pdfa_at = now() \
          WHERE id = $1",
    )
    .bind(id)
    .bind(r_23r.key)
    .bind(r_23r.pages)
    .bind(r_23r.pdfa_2b)
    .execute(r_23s)
    .await?;
    Ok(r.rows_affected() > 0)
}
pub async fn list_all(r_23v: &PgPool, r_23t: i64, r_23u: i64) -> sqlx::Result<Vec<DocRow>> {
    sqlx::query_as::<_, DocRow>(&format!(
        "SELECT {DOC_COLS} FROM documents d LEFT JOIN users u ON u.id = d.uploaded_by \
         ORDER BY d.uploaded_at DESC LIMIT $1 OFFSET $2"
    ))
    .bind(r_23t)
    .bind(r_23u)
    .fetch_all(r_23v)
    .await
}
pub async fn f_1n8(
    r_23x: &PgPool,
    status: &str,
    projects: Option<&[Uuid]>,
    r_23w: i64,
) -> sqlx::Result<Vec<DocRow>> {
    f_1n9(r_23x, status, projects, None, r_23w).await
}
pub async fn f_1n9(
    r_240: &PgPool,
    status: &str,
    projects: Option<&[Uuid]>,
    docs: Option<&[Uuid]>,
    r_23z: i64,
) -> sqlx::Result<Vec<DocRow>> {
    let r_241: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    let r_23y: Option<Vec<Uuid>> = docs.map(|d| d.to_vec());
    sqlx::query_as::<_, DocRow>(&format!(
        "SELECT {DOC_COLS} FROM documents d LEFT JOIN users u ON u.id = d.uploaded_by \
         WHERE d.status = $1 AND ($2::uuid[] IS NULL OR d.project_id = ANY($2)) \
           AND ($4::uuid[] IS NULL OR d.id = ANY($4)) \
         ORDER BY d.uploaded_at DESC LIMIT $3"
    ))
    .bind(status)
    .bind(r_241)
    .bind(r_23z)
    .bind(r_23y)
    .fetch_all(r_240)
    .await
}
pub async fn status_counts(
    r_242: &PgPool,
    projects: Option<&[Uuid]>,
) -> sqlx::Result<Vec<(String, i64, i64)>> {
    let r_243: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    sqlx::query_as::<_, (String, i64, i64)>(
        "SELECT d.status, count(*)::bigint, COALESCE(SUM(d.page_count), 0)::bigint \
         FROM documents d \
         WHERE ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
         GROUP BY d.status",
    )
    .bind(r_243)
    .fetch_all(r_242)
    .await
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tiu {
    pub project_id: Uuid,
    pub status: String,
    pub review_status: String,
    pub docs: i64,
    pub pages: i64,
    pub docs_ocr_full: i64,
    pub ocr_pages: i64,
    pub docs_fields: i64,
    pub docs_pdfa: i64,
    pub docs_pdfa_old: i64,
}
pub async fn f_281(
    r_245: &PgPool,
    projects: Option<&[Uuid]>,
    r_244: Option<DateTime<Utc>>,
    r_247: Option<DateTime<Utc>>,
) -> sqlx::Result<Vec<Tiu>> {
    let r_246: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    sqlx::query_as::<_, Tiu>(
        "SELECT d.project_id, d.status, d.review_status, count(*)::bigint AS docs, \
                COALESCE(SUM(d.page_count), 0)::bigint AS pages, \
                count(*) FILTER ( \
                    WHERE o.n > 0 AND (d.page_count IS NULL OR o.n >= d.page_count) \
                )::bigint AS docs_ocr_full, \
                COALESCE(SUM(o.n), 0)::bigint AS ocr_pages, \
                count(*) FILTER (WHERE k.n > 0)::bigint AS docs_fields, \
                count(*) FILTER (WHERE d.pdfa_at IS NOT NULL)::bigint AS docs_pdfa, \
                count(*) FILTER (WHERE d.pdfa_2b IS FALSE)::bigint AS docs_pdfa_old \
         FROM documents d \
         LEFT JOIN LATERAL ( \
             SELECT count(DISTINCT r.page_no)::bigint AS n \
             FROM ocr_results r WHERE r.document_id = d.id \
         ) o ON TRUE \
         LEFT JOIN LATERAL ( \
             SELECT count(DISTINCT k.page_no)::bigint AS n \
             FROM ocr_fields k \
             WHERE k.document_id = d.id AND length(btrim(k.value)) > 0 \
         ) k ON TRUE \
         WHERE ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
           AND ($2::timestamptz IS NULL OR d.uploaded_at >= $2) \
           AND ($3::timestamptz IS NULL OR d.uploaded_at < $3) \
         GROUP BY d.project_id, d.status, d.review_status",
    )
    .bind(r_246)
    .bind(r_244)
    .bind(r_247)
    .fetch_all(r_245)
    .await
}
pub async fn f_2ef(
    r_24a: &PgPool,
    id: Uuid,
    status: digitext_core::ReviewStatus,
    r_248: Option<Uuid>,
    r_249: Option<&str>,
) -> sqlx::Result<Option<Tja>> {
    let r_24b = status.has_trace();
    sqlx::query_as::<_, Tja>(
        "UPDATE documents \
         SET review_status = $2, \
             reviewed_by = CASE WHEN $4 THEN $3 ELSE NULL END, \
             reviewed_at = CASE WHEN $4 THEN now() ELSE NULL END, \
             review_note = CASE WHEN $4 THEN $5 ELSE NULL END \
         WHERE id = $1 \
         RETURNING review_status, reviewed_at, review_note",
    )
    .bind(id)
    .bind(status.as_str())
    .bind(r_248)
    .bind(r_24b)
    .bind(r_249)
    .fetch_optional(r_24a)
    .await
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tja {
    pub review_status: String,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_note: Option<String>,
}
pub async fn f_2eg(
    r_24f: &PgPool,
    r_24d: &[Uuid],
    projects: Option<&[Uuid]>,
    status: digitext_core::ReviewStatus,
    r_24c: Option<Uuid>,
    r_24e: Option<&str>,
) -> sqlx::Result<Vec<Uuid>> {
    if r_24d.is_empty() {
        return Ok(Vec::new());
    }
    let r_24g: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    sqlx::query_scalar::<_, Uuid>(
        "UPDATE documents \
         SET review_status = $3, \
             reviewed_by = CASE WHEN $5 THEN $4 ELSE NULL END, \
             reviewed_at = CASE WHEN $5 THEN now() ELSE NULL END, \
             review_note = CASE WHEN $5 THEN $6 ELSE NULL END \
         WHERE id = ANY($1) \
           AND ($2::uuid[] IS NULL OR project_id = ANY($2)) \
         RETURNING id",
    )
    .bind(r_24d)
    .bind(r_24g)
    .bind(status.as_str())
    .bind(r_24c)
    .bind(status.has_trace())
    .bind(r_24e)
    .fetch_all(r_24f)
    .await
}
pub async fn f_1o0(
    r_24i: &PgPool,
    projects: Option<&[Uuid]>,
    r_24h: i64,
) -> sqlx::Result<Vec<DocRow>> {
    let r_24j: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    sqlx::query_as::<_, DocRow>(&format!(
        "SELECT {DOC_COLS} FROM documents d LEFT JOIN users u ON u.id = d.uploaded_by \
         WHERE d.status IN ('uploaded', 'ocr_pending') \
           AND ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
           AND NOT EXISTS ( \
               SELECT 1 FROM ocr_jobs j WHERE j.document_id = d.id AND j.kind = 'ocr' \
           ) \
         ORDER BY d.uploaded_at ASC LIMIT $2"
    ))
    .bind(r_24j)
    .bind(r_24h)
    .fetch_all(r_24i)
    .await
}
pub async fn f_1nx(
    r_24l: &PgPool,
    projects: Option<&[Uuid]>,
    r_24k: i64,
    r_24m: bool,
) -> sqlx::Result<Vec<DocRow>> {
    f_1ny(r_24l, projects, None, r_24k, r_24m).await
}
pub async fn f_1ny(
    r_24p: &PgPool,
    projects: Option<&[Uuid]>,
    docs: Option<&[Uuid]>,
    r_24o: i64,
    r_24r: bool,
) -> sqlx::Result<Vec<DocRow>> {
    let r_24q: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    let r_24n: Option<Vec<Uuid>> = docs.map(|d| d.to_vec());
    let chan: &[&str] = if r_24r {
        &["queued", "running", "done"]
    } else {
        &["queued", "running", "done", "page_failed"]
    };
    let chan: Vec<String> = chan.iter().map(|r_24s| (*r_24s).to_owned()).collect();
    sqlx::query_as::<_, DocRow>(&format!(
        "SELECT {DOC_COLS} FROM documents d LEFT JOIN users u ON u.id = d.uploaded_by \
         WHERE d.status <> 'disposed' \
           AND d.page_count >= 1 \
           AND ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
           AND ($4::uuid[] IS NULL OR d.id = ANY($4)) \
           AND NOT EXISTS ( \
               SELECT 1 FROM ocr_jobs j WHERE j.document_id = d.id \
                 AND j.kind = 'extract_fields' AND j.status = ANY($3) \
           ) \
         ORDER BY d.uploaded_at ASC LIMIT $2"
    ))
    .bind(r_24q)
    .bind(r_24o)
    .bind(&chan)
    .bind(r_24n)
    .fetch_all(r_24p)
    .await
}
pub async fn f_120(r_24t: &PgPool, projects: Option<&[Uuid]>) -> sqlx::Result<i64> {
    f_121(r_24t, projects, None).await
}
pub async fn f_121(
    r_24v: &PgPool,
    projects: Option<&[Uuid]>,
    docs: Option<&[Uuid]>,
) -> sqlx::Result<i64> {
    let r_24w: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    let r_24u: Option<Vec<Uuid>> = docs.map(|d| d.to_vec());
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM documents d \
         WHERE d.status <> 'disposed' \
           AND (d.page_count IS NULL OR d.page_count < 1) \
           AND ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
           AND ($2::uuid[] IS NULL OR d.id = ANY($2))",
    )
    .bind(r_24w)
    .bind(r_24u)
    .fetch_one(r_24v)
    .await
}
pub async fn list_for_projects(
    r_24z: &PgPool,
    r_250: &[Uuid],
    r_24x: i64,
    r_24y: i64,
) -> sqlx::Result<Vec<DocRow>> {
    sqlx::query_as::<_, DocRow>(&format!(
        "SELECT {DOC_COLS} FROM documents d LEFT JOIN users u ON u.id = d.uploaded_by \
         WHERE d.project_id = ANY($1) ORDER BY d.uploaded_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(r_250)
    .bind(r_24x)
    .bind(r_24y)
    .fetch_all(r_24z)
    .await
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TreeDocRow {
    pub id: Uuid,
    pub filename: String,
    pub rel_path: Option<String>,
    pub ho_so_ref: Option<String>,
    pub page_count: Option<i32>,
    pub size_bytes: i64,
    pub status: String,
    pub project_id: Uuid,
    pub project_code: String,
    pub project_name: String,
    pub fields_status: Option<String>,
    pub review_status: String,
    pub ocr_pages: i64,
    pub field_pages: i64,
    pub pdfa_pages: Option<i32>,
    pub pdfa_2b: Option<bool>,
}
pub async fn f_1ol(r_251: &PgPool, projects: Option<&[Uuid]>) -> sqlx::Result<Vec<TreeDocRow>> {
    let r_252: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    sqlx::query_as::<
        _,
        TreeDocRow,
    >(
            "SELECT d.id, d.filename, d.rel_path, d.ho_so_ref, d.page_count, d.size_bytes, d.status, \
                d.project_id, p.code AS project_code, p.name AS project_name, f.status AS fields_status, \
                d.review_status, COALESCE(o.n, 0) AS ocr_pages, COALESCE(k.n, 0) AS field_pages, \
                d.pdfa_pages, d.pdfa_2b \
         FROM documents d \
         JOIN projects p ON p.id = d.project_id \
         LEFT JOIN LATERAL ( \
             SELECT count(DISTINCT r.page_no)::bigint AS n \
             FROM ocr_results r WHERE r.document_id = d.id \
         ) o ON TRUE \
         LEFT JOIN LATERAL ( \
             SELECT count(DISTINCT k.page_no)::bigint AS n \
             FROM ocr_fields k \
             WHERE k.document_id = d.id AND length(btrim(k.value)) > 0 \
         ) k ON TRUE \
         LEFT JOIN LATERAL ( \
             SELECT o.status FROM ocr_jobs o \
             WHERE o.document_id = d.id AND o.kind = 'extract_fields' \
             ORDER BY o.created_at DESC, o.id DESC LIMIT 1 \
         ) f ON TRUE \
         WHERE ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
         ORDER BY p.code, d.rel_path NULLS LAST, d.filename, d.id",
        )
        .bind(r_252)
        .fetch_all(r_251)
        .await
}
