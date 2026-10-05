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

/// Ví dụ CÙNG DỰ ÁN với tài liệu đang bóc được cộng điểm này vào độ giống (khoảng cách trigram
/// 0..1 nhỏ = giống): ví dụ cùng dự án giống vừa phải vẫn thắng ví dụ chung (plakie) giống hơn
/// một chút, nhưng không thắng ví dụ chung giống hẳn khi dự án chưa có mẫu nào tương tự.
pub const SAME_PROJECT_BONUS: f32 = 0.2;

/// `k` ví dụ có chữ OCR giống nhất (độ giống trigram), bỏ chính tài liệu đang bóc, ưu tiên ví dụ
/// cùng dự án với `exclude_doc`. Ghi nhận lượt dùng (used_count) cho các ví dụ được chọn.
pub async fn similar(pool: &PgPool, text: &str, k: i64, exclude_doc: Option<Uuid>) -> sqlx::Result<Vec<Example>> {
    sqlx::query_as::<_, Example>(
        "WITH cur AS (SELECT project_id FROM documents WHERE id = $3), \
         pick AS ( \
           SELECT e.id, e.ocr_text, e.fields, \
             (e.ocr_text <-> $1) \
             - CASE WHEN d.project_id IS NOT NULL AND d.project_id = (SELECT project_id FROM cur) \
                    THEN $4::real ELSE 0 END AS score \
           FROM kie_examples e \
           LEFT JOIN documents d ON d.id = e.document_id \
           WHERE e.active AND ($3::uuid IS NULL OR e.document_id IS DISTINCT FROM $3) \
           ORDER BY score LIMIT $2), \
         used AS ( \
           UPDATE kie_examples SET used_count = used_count + 1, last_used_at = now() \
           WHERE id IN (SELECT id FROM pick)) \
         SELECT ocr_text, fields FROM pick ORDER BY score",
    )
    .bind(head(text))
    .bind(k)
    .bind(exclude_doc)
    .bind(SAME_PROJECT_BONUS)
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
