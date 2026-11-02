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
    pub rel_path: Option<String>,
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
/// Cùng dự án + cùng nội dung + cùng tên + cùng đường dẫn = cùng một tài liệu (khoá
/// `ux_documents_hash_name`, migration 0005). Cùng nội dung mà khác tên/vị trí là tài liệu khác.
pub async fn find_same_file(
    pool: &PgPool,
    project_id: Uuid,
    blake3: &str,
    filename: &str,
    rel_path: Option<&str>,
) -> sqlx::Result<Option<Document>> {
    sqlx::query_as::<_, Document>(
        "SELECT * FROM documents WHERE project_id = $1 AND blake3 = $2 AND filename = $3 \
         AND COALESCE(rel_path, '') = COALESCE($4, '')",
    )
    .bind(project_id)
    .bind(blake3)
    .bind(filename)
    .bind(rel_path)
    .fetch_optional(pool)
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
         ON CONFLICT (project_id, blake3, filename, (COALESCE(rel_path, ''))) DO NOTHING \
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
/// Bản PDF/A đã chuyển ra lưu trữ ngoài (migration 0019): (lúc chuyển, ghi chú nơi lưu).
pub async fn pdfa_offload(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<(chrono::DateTime<chrono::Utc>, Option<String>)>> {
    sqlx::query_as::<_, (Option<chrono::DateTime<chrono::Utc>>, Option<String>)>(
        "SELECT pdfa_offloaded_at, pdfa_offload_note FROM documents WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map(|r| r.and_then(|(at, note)| at.map(|at| (at, note))))
}
const DOC_COLS: &str = "d.id, d.project_id, d.filename, d.ho_so_ref, d.page_count, \
     d.size_bytes, d.status, u.display_name AS uploaded_by_name, d.uploaded_at, \
     d.review_status, d.reviewed_at, d.review_note, d.rel_path";
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
         ORDER BY d.project_id, d.rel_path COLLATE vi_so NULLS LAST, d.filename COLLATE vi_so, d.uploaded_at LIMIT $3"
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
         ORDER BY d.project_id, d.rel_path COLLATE vi_so NULLS LAST, d.filename COLLATE vi_so, d.uploaded_at \
         LIMIT $2"
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
/// Cây tài liệu dẫn xuất phông / hộp / hồ sơ ĐÚNG như máy trạm từng tính (reviewScope.ts
/// `phongOf` + `V4ki`): phông = tầng 1 của rel_path khi có ≥ 2 tầng; hộp/hồ sơ lấy từ ho_so_ref
/// ("<hộp>/<hồ sơ>" hoặc chỉ "<hồ sơ>"), không có thì tầng 2/3 của rel_path.
const REVIEW_DERIVED_CTE: &str = "WITH base AS ( \
      SELECT d.id, d.project_id, d.filename, d.rel_path, d.ho_so_ref, d.page_count, d.size_bytes, \
             d.status, d.review_status, d.pdfa_pages, d.pdfa_2b, \
             COALESCE(o.n, 0)::bigint AS ocr_pages, COALESCE(k.n, 0)::bigint AS field_pages, \
             f.status AS fields_status, \
             array_remove(string_to_array(COALESCE(d.rel_path, ''), '/'), '') AS seg, \
             NULLIF(btrim(COALESCE(d.ho_so_ref, '')), '') AS hsr \
      FROM documents d \
      LEFT JOIN (SELECT document_id, count(DISTINCT page_no)::bigint AS n FROM ocr_results GROUP BY document_id) o ON o.document_id = d.id \
      LEFT JOIN (SELECT document_id, count(DISTINCT page_no)::bigint AS n FROM ocr_fields WHERE length(btrim(value)) > 0 GROUP BY document_id) k ON k.document_id = d.id \
      LEFT JOIN (SELECT DISTINCT ON (document_id) document_id, status FROM ocr_jobs WHERE kind = 'extract_fields' ORDER BY document_id, created_at DESC, id DESC) f ON f.document_id = d.id \
      WHERE d.status <> 'disposed' AND ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
    ), derived AS ( \
      SELECT b.*, \
        CASE WHEN cardinality(seg) >= 2 THEN btrim(seg[1]) ELSE '' END AS phong, \
        CASE WHEN hsr IS NOT NULL THEN CASE WHEN position('/' IN hsr) > 1 THEN btrim(split_part(hsr, '/', 1)) ELSE '' END \
             WHEN cardinality(seg) >= 3 THEN btrim(seg[2]) ELSE '' END AS hop, \
        CASE WHEN hsr IS NOT NULL THEN CASE WHEN position('/' IN hsr) > 1 THEN btrim(substr(hsr, position('/' IN hsr) + 1)) ELSE hsr END \
             WHEN cardinality(seg) >= 3 THEN btrim(seg[3]) ELSE '' END AS ho_so \
      FROM base b \
    ) ";
/// Một dòng hồ sơ trong màn Soát & duyệt: số liệu gộp sẵn (máy trạm không phải tải cả cây).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReviewSummaryRow {
    pub project_id: Uuid,
    pub project_code: String,
    pub project_name: String,
    pub phong: String,
    pub hop: String,
    pub ho_so: String,
    pub files: i64,
    pub pages: i64,
    pub unknown_pages: i64,
    pub size_bytes: i64,
    pub ocr_pages: i64,
    pub field_docs: i64,
    pub pdfa_docs: i64,
    pub pdfa_pages: i64,
    pub pdfa_cu: i64,
    pub ocr_denom: i64,
    pub approved: i64,
    pub rejected: i64,
    pub ocr_pending_docs: i64,
    pub ocr_done_docs: i64,
    pub pdf2_done_docs: i64,
    pub fields_done: i64,
    pub fields_active: i64,
    pub fields_failed: i64,
    pub signed_docs: i64,
    pub pdfa_only_docs: i64,
}
pub async fn review_summary(
    pool: &PgPool,
    projects: Option<&[Uuid]>,
    q: Option<&str>,
    pending_only: bool,
) -> sqlx::Result<Vec<ReviewSummaryRow>> {
    let projects: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    let like = q.map(|x| x.trim()).filter(|x| !x.is_empty()).map(|x| format!("%{}%", x.replace('%', "\\%").replace('_', "\\_")));
    let sql = format!(
        "{REVIEW_DERIVED_CTE} \
         SELECT dd.project_id, p.code AS project_code, p.name AS project_name, dd.phong, dd.hop, dd.ho_so, \
                count(*)::bigint AS files, COALESCE(sum(dd.page_count), 0)::bigint AS pages, \
                count(*) FILTER (WHERE dd.page_count IS NULL)::bigint AS unknown_pages, \
                COALESCE(sum(dd.size_bytes), 0)::bigint AS size_bytes, COALESCE(sum(dd.ocr_pages), 0)::bigint AS ocr_pages, \
                count(*) FILTER (WHERE dd.field_pages > 0)::bigint AS field_docs, \
                count(*) FILTER (WHERE COALESCE(dd.pdfa_pages, 0) > 0)::bigint AS pdfa_docs, \
                COALESCE(sum(COALESCE(dd.pdfa_pages, 0)), 0)::bigint AS pdfa_pages, \
                count(*) FILTER (WHERE COALESCE(dd.pdfa_pages, 0) > 0 AND dd.pdfa_2b = false)::bigint AS pdfa_cu, \
                COALESCE(sum(COALESCE(dd.page_count, dd.ocr_pages)), 0)::bigint AS ocr_denom, \
                count(*) FILTER (WHERE dd.review_status = 'approved')::bigint AS approved, \
                count(*) FILTER (WHERE dd.review_status = 'rejected')::bigint AS rejected, \
                count(*) FILTER (WHERE dd.status IN ('uploaded', 'ocr_pending'))::bigint AS ocr_pending_docs, \
                count(*) FILTER (WHERE dd.status IN ('ocr_done', 'to_sign', 'signing', 'signed', 'sign_rejected', 'archived'))::bigint AS ocr_done_docs, \
                count(*) FILTER (WHERE dd.status IN ('to_sign', 'signing', 'signed', 'sign_rejected', 'archived'))::bigint AS pdf2_done_docs, \
                count(*) FILTER (WHERE dd.fields_status = 'done')::bigint AS fields_done, \
                count(*) FILTER (WHERE dd.fields_status IN ('queued', 'running'))::bigint AS fields_active, \
                count(*) FILTER (WHERE dd.fields_status IN ('failed_transient', 'page_failed'))::bigint AS fields_failed, \
                count(*) FILTER (WHERE dd.status IN ('signed', 'archived'))::bigint AS signed_docs, \
                count(*) FILTER (WHERE dd.status NOT IN ('signed', 'archived') AND COALESCE(dd.pdfa_pages, 0) > 0)::bigint AS pdfa_only_docs \
         FROM derived dd JOIN projects p ON p.id = dd.project_id \
         WHERE ($2::text IS NULL OR dd.filename ILIKE $2 OR COALESCE(dd.ho_so_ref, '') ILIKE $2 OR COALESCE(dd.rel_path, '') ILIKE $2) \
           AND (NOT $3::bool OR dd.review_status NOT IN ('approved', 'rejected')) \
         GROUP BY 1, 2, 3, 4, 5, 6 \
         ORDER BY p.code, dd.phong COLLATE vi_so, dd.hop COLLATE vi_so, dd.ho_so COLLATE vi_so"
    );
    sqlx::query_as::<_, ReviewSummaryRow>(&sql)
        .bind(projects)
        .bind(like)
        .bind(pending_only)
        .fetch_all(pool)
        .await
}
/// Tài liệu của các nhóm (phông, hộp, hồ sơ) đã chọn trong một dự án — nạp khi người dùng mở
/// một hồ sơ hoặc tích chọn cả hồ sơ.
pub async fn review_group_docs(
    pool: &PgPool,
    project_id: Uuid,
    groups: &[(String, String, String)],
) -> sqlx::Result<Vec<TreeDocRow>> {
    let phongs: Vec<String> = groups.iter().map(|g| g.0.clone()).collect();
    let hops: Vec<String> = groups.iter().map(|g| g.1.clone()).collect();
    let hosos: Vec<String> = groups.iter().map(|g| g.2.clone()).collect();
    let sql = format!(
        "{REVIEW_DERIVED_CTE} \
         SELECT dd.id, dd.filename, dd.rel_path, dd.ho_so_ref, dd.page_count, dd.size_bytes, dd.status, \
                dd.project_id, p.code AS project_code, p.name AS project_name, dd.fields_status, \
                dd.review_status, dd.ocr_pages, dd.field_pages, dd.pdfa_pages, dd.pdfa_2b \
         FROM derived dd JOIN projects p ON p.id = dd.project_id \
         WHERE EXISTS (SELECT 1 FROM unnest($2::text[], $3::text[], $4::text[]) AS g(a, b, c) \
                       WHERE g.a = dd.phong AND g.b = dd.hop AND g.c = dd.ho_so) \
         ORDER BY dd.phong COLLATE vi_so, dd.hop COLLATE vi_so, dd.ho_so COLLATE vi_so, dd.filename COLLATE vi_so"
    );
    sqlx::query_as::<_, TreeDocRow>(&sql)
        .bind(Some(vec![project_id]))
        .bind(phongs)
        .bind(hops)
        .bind(hosos)
        .fetch_all(pool)
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
         LEFT JOIN ( \
             SELECT document_id, count(DISTINCT page_no)::bigint AS n \
             FROM ocr_results GROUP BY document_id \
         ) o ON o.document_id = d.id \
         LEFT JOIN ( \
             SELECT document_id, count(DISTINCT page_no)::bigint AS n \
             FROM ocr_fields WHERE length(btrim(value)) > 0 GROUP BY document_id \
         ) k ON k.document_id = d.id \
         LEFT JOIN ( \
             SELECT DISTINCT ON (document_id) document_id, status \
             FROM ocr_jobs WHERE kind = 'extract_fields' \
             ORDER BY document_id, created_at DESC, id DESC \
         ) f ON f.document_id = d.id \
         WHERE ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
         ORDER BY p.code, d.rel_path NULLS LAST, d.filename, d.id",
        )
        .bind(r_252)
        .fetch_all(r_251)
        .await
}
/// Ghi nhận PDF gốc đã bị xoá khỏi kho (đã có bản PDF/A-2b thay thế) — migration 0016.
pub async fn mark_source_purged(pool: &PgPool, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE documents SET source_purged_at = now() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Tài liệu của dự án kèm hộp / hồ sơ suy từ `ho_so_ref` (hoặc `rel_path`) — cùng cách tính với màn Soát & duyệt
/// (`REVIEW_DERIVED_CTE`). Dùng cho "Tạo văn bản từ OCR" (2/11/2026): ghép tài liệu vào hồ sơ chỉnh lý theo (hộp, hồ sơ).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DocRefRow {
    pub id: Uuid,
    pub filename: String,
    pub phong: String,
    pub hop: String,
    pub ho_so: String,
    pub page_count: Option<i32>,
    pub status: String,
}
pub async fn list_for_project_refs(pool: &PgPool, project: Uuid) -> sqlx::Result<Vec<DocRefRow>> {
    let sql = format!("{REVIEW_DERIVED_CTE} SELECT id, filename, phong, hop, ho_so, page_count, status FROM derived ORDER BY filename, id");
    sqlx::query_as::<_, DocRefRow>(&sql).bind(Some(vec![project])).fetch_all(pool).await
}
