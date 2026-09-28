// Copyright (c) 2026 DigiText. All rights reserved.
//! Kho ví dụ "dạy bằng ví dụ" cho bóc trường (migration 0010).
use sqlx::PgPool;
use uuid::Uuid;

/// Số ký tự chữ OCR đầu trang dùng để so khớp và đưa vào câu hỏi.
pub const HEAD_CHARS: usize = 1500;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Example {
    pub ocr_text: String,
    pub fields: serde_json::Value,
}

/// Cắt chữ OCR về phần đầu trang (theo ký tự, không cắt giữa ký tự UTF-8).
pub fn head(text: &str) -> String {
    text.chars().take(HEAD_CHARS).collect()
}

/// `k` ví dụ có chữ OCR giống nhất (độ giống trigram), bỏ chính tài liệu đang bóc.
pub async fn similar(pool: &PgPool, text: &str, k: i64, exclude_doc: Option<Uuid>) -> sqlx::Result<Vec<Example>> {
    sqlx::query_as::<_, Example>(
        "SELECT ocr_text, fields FROM kie_examples \
         WHERE active AND ($3::uuid IS NULL OR document_id IS DISTINCT FROM $3) \
         ORDER BY ocr_text <-> $1 LIMIT $2",
    )
    .bind(head(text))
    .bind(k)
    .bind(exclude_doc)
    .fetch_all(pool)
    .await
}

/// Chữ OCR mới nhất của một trang.
pub async fn page_text(pool: &PgPool, document_id: Uuid, page_no: i32) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar::<_, String>(
        "SELECT content FROM ocr_results WHERE document_id = $1 AND page_no = $2 \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(document_id)
    .bind(page_no)
    .fetch_optional(pool)
    .await
}

/// Người dùng Duyệt trường → lấy các trường ĐÃ DUYỆT của trang làm ví dụ (tạo mới / cập nhật).
/// Trả false khi trang chưa có chữ OCR hoặc chưa có trường nào được duyệt.
pub async fn upsert_from_document(pool: &PgPool, document_id: Uuid, page_no: i32, by: Uuid) -> sqlx::Result<bool> {
    let Some(text) = page_text(pool, document_id, page_no).await? else {
        return Ok(false);
    };
    let fields: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT jsonb_object_agg(field_key, value) FROM ( \
             SELECT DISTINCT ON (field_key) field_key, value FROM ocr_fields \
             WHERE document_id = $1 AND page_no = $2 AND verified AND text_source = 'ocr_corrected' \
             ORDER BY field_key, updated_at DESC) f",
    )
    .bind(document_id)
    .bind(page_no)
    .fetch_one(pool)
    .await?;
    let Some(fields) = fields.filter(|f| f.as_object().is_some_and(|o| !o.is_empty())) else {
        return Ok(false);
    };
    sqlx::query(
        "INSERT INTO kie_examples (id, source, document_id, page_no, ocr_text, fields, created_by) \
         VALUES ($1, 'duyet', $2, $3, $4, $5, $6) \
         ON CONFLICT (document_id, page_no) WHERE document_id IS NOT NULL DO UPDATE SET \
           ocr_text = EXCLUDED.ocr_text, fields = EXCLUDED.fields, active = true, updated_at = now()",
    )
    .bind(Uuid::now_v7())
    .bind(document_id)
    .bind(page_no)
    .bind(head(&text))
    .bind(fields)
    .bind(by)
    .execute(pool)
    .await?;
    Ok(true)
}
