// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use serde_json::Value as Json;
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tfd {
    pub id: Uuid,
    pub document_id: Uuid,
    pub page_no: i32,
    pub engine_version_id: Uuid,
    pub text_source: String,
    pub content: String,
    pub mean_confidence: Option<f32>,
    pub low_conf_ratio: Option<f32>,
    pub boxes: Option<Json>,
    pub created_at: DateTime<Utc>,
    pub ink_coverage: Option<f32>,
}
const COLS: &str = "id, document_id, page_no, engine_version_id, text_source, content, \
     mean_confidence, low_conf_ratio, boxes, created_at, ink_coverage";
#[derive(Debug)]
pub struct Tdx<'a> {
    pub document_id: Uuid,
    pub page_no: i32,
    pub engine_version_id: Uuid,
    pub text_source: &'a str,
    pub content: &'a str,
    pub mean_confidence: Option<f32>,
    pub low_conf_ratio: Option<f32>,
    pub boxes: Option<Json>,
    pub ink_coverage: Option<f32>,
}
pub async fn upsert(r_28k: &PgPool, r: Tdx<'_>) -> sqlx::Result<Tfd> {
    sqlx::query_as::<_, Tfd>(&format!(
        "INSERT INTO ocr_results \
            (id, document_id, page_no, engine_version_id, text_source, content, \
             mean_confidence, low_conf_ratio, boxes, ink_coverage) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
         ON CONFLICT (document_id, page_no, engine_version_id, text_source) DO UPDATE SET \
            content = EXCLUDED.content, \
            mean_confidence = EXCLUDED.mean_confidence, \
            low_conf_ratio = EXCLUDED.low_conf_ratio, \
            boxes = EXCLUDED.boxes, \
            ink_coverage = EXCLUDED.ink_coverage \
         RETURNING {COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(r.document_id)
    .bind(r.page_no)
    .bind(r.engine_version_id)
    .bind(r.text_source)
    .bind(r.content)
    .bind(r.mean_confidence)
    .bind(r.low_conf_ratio)
    .bind(r.boxes)
    .bind(r.ink_coverage)
    .fetch_one(r_28k)
    .await
}
pub async fn f_1ni(
    r_28l: &PgPool,
    document_id: Uuid,
    engine_version_id: Uuid,
    text_source: &str,
) -> sqlx::Result<Vec<Tfd>> {
    sqlx::query_as::<_, Tfd>(&format!(
        "SELECT {COLS} FROM ocr_results \
         WHERE document_id = $1 AND engine_version_id = $2 AND text_source = $3 \
         ORDER BY page_no"
    ))
    .bind(document_id)
    .bind(engine_version_id)
    .bind(text_source)
    .fetch_all(r_28l)
    .await
}
pub async fn f_1nt(r_28m: &PgPool, document_id: Uuid, text_source: &str) -> sqlx::Result<Vec<Tfd>> {
    sqlx::query_as::<_, Tfd>(&format!(
        "SELECT DISTINCT ON (page_no) {COLS} FROM ocr_results \
         WHERE document_id = $1 AND text_source = $2 \
         ORDER BY page_no, created_at DESC, id DESC"
    ))
    .bind(document_id)
    .bind(text_source)
    .fetch_all(r_28m)
    .await
}
pub async fn get(
    r_28n: &PgPool,
    document_id: Uuid,
    page_no: i32,
    engine_version_id: Uuid,
    text_source: &str,
) -> sqlx::Result<Option<Tfd>> {
    sqlx::query_as::<_, Tfd>(&format!(
        "SELECT {COLS} FROM ocr_results \
         WHERE document_id = $1 AND page_no = $2 AND engine_version_id = $3 AND text_source = $4"
    ))
    .bind(document_id)
    .bind(page_no)
    .bind(engine_version_id)
    .bind(text_source)
    .fetch_optional(r_28n)
    .await
}
pub async fn count_for_document(r_28o: &PgPool, document_id: Uuid) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>("SELECT count(*) FROM ocr_results WHERE document_id = $1")
        .bind(document_id)
        .fetch_one(r_28o)
        .await
}
/// Độ phủ mực theo trang (kết quả `ocr_raw` MỚI NHẤT của mỗi trang): (số trang đã đo,
/// số trang dưới ngưỡng `min`). Trang OCR trước 21/9/2026 chưa đo (NULL) không tính.
pub async fn ink_coverage_counts(
    pool: &PgPool,
    projects: Option<&[Uuid]>,
    min: f32,
) -> sqlx::Result<(i64, i64)> {
    let projects: Option<Vec<Uuid>> = projects.map(|p| p.to_vec());
    sqlx::query_as::<_, (i64, i64)>(
        "SELECT count(*) FILTER (WHERE t.ink_coverage IS NOT NULL)::bigint, \
                count(*) FILTER (WHERE t.ink_coverage < $2)::bigint \
         FROM (SELECT DISTINCT ON (r.document_id, r.page_no) r.ink_coverage \
               FROM ocr_results r JOIN documents d ON d.id = r.document_id \
               WHERE r.text_source = 'ocr_raw' AND d.status <> 'disposed' \
                 AND ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
               ORDER BY r.document_id, r.page_no, r.created_at DESC) t",
    )
    .bind(projects)
    .bind(min)
    .fetch_one(pool)
    .await
}
pub async fn f_1y3(r_28q: &PgPool, r_28s: Option<&[Uuid]>) -> sqlx::Result<i64> {
    let r_28r: Option<Vec<Uuid>> = r_28s.map(|r_28p| r_28p.to_vec());
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM ocr_results r \
         JOIN documents d ON d.id = r.document_id \
         WHERE r.text_source = 'ocr_raw' \
           AND ($1::uuid[] IS NULL OR d.project_id = ANY($1))",
    )
    .bind(r_28r)
    .fetch_one(r_28q)
    .await
}
