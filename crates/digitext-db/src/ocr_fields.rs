// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use serde_json::Value as Json;
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tfb {
    pub id: Uuid,
    pub document_id: Uuid,
    pub page_no: i32,
    pub engine_version_id: Uuid,
    pub text_source: String,
    pub field_key: String,
    pub value: String,
    pub bbox: Option<Json>,
    pub confidence: Option<f32>,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
const COLS: &str = "id, document_id, page_no, engine_version_id, text_source, field_key, \
     value, bbox, confidence, verified, created_at, updated_at";
#[derive(Debug)]
pub struct Tdw<'a> {
    pub document_id: Uuid,
    pub page_no: i32,
    pub engine_version_id: Uuid,
    pub text_source: &'a str,
    pub field_key: &'a str,
    pub value: &'a str,
    pub bbox: Option<Json>,
    pub confidence: Option<f32>,
    pub verified: bool,
}
pub async fn upsert(r_26l: &PgPool, r_26m: Tdw<'_>) -> sqlx::Result<Tfb> {
    sqlx::query_as::<_, Tfb>(&format!(
        "INSERT INTO ocr_fields \
            (id, document_id, page_no, engine_version_id, text_source, field_key, \
             value, bbox, confidence, verified) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
         ON CONFLICT (document_id, page_no, engine_version_id, text_source, field_key) \
         DO UPDATE SET \
            value = EXCLUDED.value, \
            bbox = EXCLUDED.bbox, \
            confidence = EXCLUDED.confidence, \
            verified = EXCLUDED.verified, \
            updated_at = now() \
         RETURNING {COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(r_26m.document_id)
    .bind(r_26m.page_no)
    .bind(r_26m.engine_version_id)
    .bind(r_26m.text_source)
    .bind(r_26m.field_key)
    .bind(r_26m.value)
    .bind(r_26m.bbox)
    .bind(r_26m.confidence)
    .bind(r_26m.verified)
    .fetch_one(r_26l)
    .await
}
pub async fn get(
    r_26n: &PgPool,
    document_id: Uuid,
    page_no: i32,
    engine_version_id: Uuid,
    text_source: &str,
    field_key: &str,
) -> sqlx::Result<Option<Tfb>> {
    sqlx::query_as::<_, Tfb>(&format!(
        "SELECT {COLS} FROM ocr_fields \
         WHERE document_id = $1 AND page_no = $2 AND engine_version_id = $3 \
           AND text_source = $4 AND field_key = $5"
    ))
    .bind(document_id)
    .bind(page_no)
    .bind(engine_version_id)
    .bind(text_source)
    .bind(field_key)
    .fetch_optional(r_26n)
    .await
}
pub async fn f_1nj(
    r_26o: &PgPool,
    document_id: Uuid,
    page_no: i32,
    engine_version_id: Uuid,
    text_source: &str,
) -> sqlx::Result<Vec<Tfb>> {
    sqlx::query_as::<_, Tfb>(&format!(
        "SELECT {COLS} FROM ocr_fields \
         WHERE document_id = $1 AND page_no = $2 AND engine_version_id = $3 AND text_source = $4 \
         ORDER BY field_key"
    ))
    .bind(document_id)
    .bind(page_no)
    .bind(engine_version_id)
    .bind(text_source)
    .fetch_all(r_26o)
    .await
}
pub async fn f_1nh(r_26p: &PgPool, document_id: Uuid) -> sqlx::Result<Vec<Tfb>> {
    sqlx::query_as::<_, Tfb>(&format!(
        "SELECT {COLS} FROM ocr_fields WHERE document_id = $1 \
         ORDER BY page_no, text_source, field_key"
    ))
    .bind(document_id)
    .fetch_all(r_26p)
    .await
}
pub async fn count_for_document(r_26q: &PgPool, document_id: Uuid) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>("SELECT count(*) FROM ocr_fields WHERE document_id = $1")
        .bind(document_id)
        .fetch_one(r_26q)
        .await
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T8t {
    pub document_id: Uuid,
    pub page_no: i32,
    pub field_key: String,
    pub value: String,
    pub bbox: Option<Json>,
}
pub async fn f_1ak(
    r_26x: &PgPool,
    r_26z: Option<&[Uuid]>,
    r_26s: Option<(Uuid, i32, &str)>,
    r_26w: i64,
) -> sqlx::Result<Vec<T8t>> {
    let (r_26r, r_26u, r_26t) = match r_26s {
        Some((d, p, r_26v)) => (Some(d), p, r_26v),
        None => (None, 0, ""),
    };
    let r_26y: Option<Vec<Uuid>> = r_26z.map(|p| p.to_vec());
    sqlx::query_as::<_, T8t>(
        "SELECT DISTINCT ON (f.document_id, f.page_no, f.field_key) \
             f.document_id, f.page_no, f.field_key, f.value, f.bbox \
         FROM ocr_fields f JOIN documents d ON d.id = f.document_id \
         WHERE f.verified = TRUE AND f.text_source = 'ocr_corrected' \
           AND ($1::uuid[] IS NULL OR d.project_id = ANY($1)) \
           AND ($2::uuid IS NULL OR (f.document_id, f.page_no, f.field_key) > ($2, $3, $4)) \
         ORDER BY f.document_id, f.page_no, f.field_key, f.updated_at DESC \
         LIMIT $5",
    )
    .bind(r_26y)
    .bind(r_26r)
    .bind(r_26u)
    .bind(r_26t)
    .bind(r_26w)
    .fetch_all(r_26x)
    .await
}

/// Giá trị "tốt nhất" của từng trường cho nhiều tài liệu, một truy vấn: ưu tiên dòng đã có người xác nhận, rồi bản
/// `ocr_corrected` (Qwen / người sửa) hơn `ocr_raw`, rồi trang đầu, rồi bản mới nhất. Chỉ lấy giá trị không rỗng.
/// Dùng cho "Tạo văn bản từ OCR" (2/11/2026); gọi theo lô ≤ 1.000 id.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BestField {
    pub document_id: Uuid,
    pub field_key: String,
    pub value: String,
    pub verified: bool,
}
pub async fn best_fields_for_docs(pool: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<BestField>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as::<_, BestField>(
        "SELECT DISTINCT ON (f.document_id, f.field_key) f.document_id, f.field_key, f.value, f.verified \
         FROM ocr_fields f \
         WHERE f.document_id = ANY($1) AND length(btrim(f.value)) > 0 \
         ORDER BY f.document_id, f.field_key, f.verified DESC, (f.text_source = 'ocr_corrected') DESC, f.page_no ASC, f.updated_at DESC",
    )
    .bind(ids.to_vec())
    .fetch_all(pool)
    .await
}
