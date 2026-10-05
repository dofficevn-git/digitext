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

/// Một dòng của màn quản lý ví dụ.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub id: Uuid,
    pub source: String,
    pub project_id: Option<Uuid>,
    pub project_code: Option<String>,
    pub document_id: Option<Uuid>,
    pub filename: Option<String>,
    pub ext_ref: Option<String>,
    pub page_no: Option<i32>,
    pub ocr_text: String,
    pub fields: serde_json::Value,
    pub active: bool,
    pub used_count: i64,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_by_name: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Bộ lọc màn quản lý ví dụ. `projects`: None = mọi dự án; Some = chỉ các dự án này
/// (cộng ví dụ chung không thuộc dự án nào nếu `include_general`).
#[derive(Debug, Default)]
pub struct ListFilter<'a> {
    pub projects: Option<&'a [Uuid]>,
    pub include_general: bool,
    pub only_general: bool,
    pub active: Option<bool>,
    pub q: Option<&'a str>,
    /// "used" (dùng nhiều nhất) | "recent" (mới cập nhật) — mặc định "used".
    pub order: &'a str,
    pub limit: i64,
    pub offset: i64,
}

const LIST_WHERE: &str = "FROM kie_examples e \
     LEFT JOIN documents d ON d.id = e.document_id \
     LEFT JOIN projects p ON p.id = d.project_id \
     LEFT JOIN users u ON u.id = e.created_by \
     WHERE ($1::uuid[] IS NULL OR d.project_id = ANY($1) OR ($2 AND e.document_id IS NULL)) \
       AND (NOT $3 OR e.document_id IS NULL) \
       AND ($4::bool IS NULL OR e.active = $4) \
       AND ($5::text IS NULL OR e.ocr_text ILIKE '%' || $5 || '%' OR e.fields::text ILIKE '%' || $5 || '%' \
            OR d.filename ILIKE '%' || $5 || '%' OR e.ext_ref ILIKE '%' || $5 || '%')";

/// Danh sách ví dụ + tổng số dòng khớp bộ lọc.
pub async fn list(pool: &PgPool, f: &ListFilter<'_>) -> sqlx::Result<(i64, Vec<ListItem>)> {
    let q = f.q.map(str::trim).filter(|s| !s.is_empty());
    let total: i64 = sqlx::query_scalar(&format!("SELECT count(*) {LIST_WHERE}"))
        .bind(f.projects)
        .bind(f.include_general)
        .bind(f.only_general)
        .bind(f.active)
        .bind(q)
        .fetch_one(pool)
        .await?;
    let order = if f.order == "recent" {
        "e.updated_at DESC, e.id"
    } else {
        "e.used_count DESC, e.last_used_at DESC NULLS LAST, e.id"
    };
    let items = sqlx::query_as::<_, ListItem>(&format!(
        "SELECT e.id, e.source, d.project_id, p.code AS project_code, e.document_id, d.filename, e.ext_ref, \
         e.page_no, e.ocr_text, e.fields, e.active, e.used_count, e.last_used_at, \
         coalesce(u.display_name, u.username) AS created_by_name, e.updated_at \
         {LIST_WHERE} ORDER BY {order} LIMIT $6 OFFSET $7"
    ))
    .bind(f.projects)
    .bind(f.include_general)
    .bind(f.only_general)
    .bind(f.active)
    .bind(q)
    .bind(f.limit)
    .bind(f.offset)
    .fetch_all(pool)
    .await?;
    Ok((total, items))
}

/// Dự án của ví dụ (None = ví dụ chung, ví dụ plakie). Ok(None) ngoài cùng = không có ví dụ đó.
pub async fn project_of(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Option<Uuid>>> {
    sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT d.project_id FROM kie_examples e LEFT JOIN documents d ON d.id = e.document_id WHERE e.id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Bật / tắt một ví dụ (ví dụ tắt không được đưa cho VLM nữa, vẫn giữ để bật lại).
pub async fn set_active(pool: &PgPool, id: Uuid, active: bool) -> sqlx::Result<bool> {
    let r = sqlx::query("UPDATE kie_examples SET active = $2, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(active)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}
