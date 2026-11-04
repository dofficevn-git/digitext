// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Datelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RecordRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub created_by: Option<Uuid>,
    pub kind: String,
    pub so_bien_ban: String,
    pub title: String,
    pub submission_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub pdf_key: String,
    pub created_at: DateTime<Utc>,
    /// Khoá phụ để lọc (0024): mã phông / ngày YYYY-MM-DD / "lần n".
    #[sqlx(rename = "ref")]
    pub ref_: Option<String>,
}
const COLS: &str =
    "id, project_id, created_by, kind, so_bien_ban, title, submission_id, payload, pdf_key, \
     created_at, ref";
pub fn f_1m5(kind: &str) -> &'static str {
    match kind {
        "kiem_tra" => "KT",
        "nhat_ky_sao_luu" => "NK",
        "sao_luu" => "SL",
        "su_co" => "SC",
        "thu_nop" => "TN",
        "thu_nop_truc_tiep" => "TT",
        "giao_nhan" => "GN",
        "tra_lai" => "TL",
        // Thi công số hoá Điều 9 TT05 (0024):
        "giao_nhan_giay" => "GNG",
        "tra_tai_lieu_giay" => "TRG",
        "nhat_ky_so_hoa" => "NKS",
        "kiem_tra_san_pham" => "KTS",
        "nghiem_thu" => "NT",
        "ban_giao_lan" => "BG",
        _ => "BB",
    }
}
pub struct Tdz<'a> {
    pub project_id: Uuid,
    pub created_by: Option<Uuid>,
    pub kind: &'a str,
    pub title: &'a str,
    pub submission_id: Option<Uuid>,
    pub payload: &'a serde_json::Value,
    pub pdf_key: &'a str,
    pub ref_: Option<&'a str>,
}
pub async fn create(r_2b1: &PgPool, r_2b2: Tdz<'_>) -> sqlx::Result<RecordRow> {
    let mut r_2b5 = r_2b1.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))")
        .bind(r_2b2.project_id)
        .execute(&mut *r_2b5)
        .await?;
    let year = Utc::now().year();
    let r_2b4: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM records \
         WHERE project_id = $1 AND date_part('year', created_at) = $2",
    )
    .bind(r_2b2.project_id)
    .bind(year as f64)
    .fetch_one(&mut *r_2b5)
    .await?;
    let so_bien_ban = format!("{:02}/BB-{}.{}", r_2b4 + 1, f_1m5(r_2b2.kind), year);
    let r_2b3 = sqlx::query_as::<_, RecordRow>(&format!(
        "INSERT INTO records (id, project_id, created_by, kind, so_bien_ban, title, \
             submission_id, payload, pdf_key, ref) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING {COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(r_2b2.project_id)
    .bind(r_2b2.created_by)
    .bind(r_2b2.kind)
    .bind(&so_bien_ban)
    .bind(r_2b2.title)
    .bind(r_2b2.submission_id)
    .bind(r_2b2.payload)
    .bind(r_2b2.pdf_key)
    .bind(r_2b2.ref_)
    .fetch_one(&mut *r_2b5)
    .await?;
    r_2b5.commit().await?;
    Ok(r_2b3)
}
pub async fn get(r_2b6: &PgPool, id: Uuid) -> sqlx::Result<Option<RecordRow>> {
    sqlx::query_as::<_, RecordRow>(&format!("SELECT {COLS} FROM records WHERE id = $1"))
        .bind(id)
        .fetch_optional(r_2b6)
        .await
}
/// Lọc thêm (0024): `kinds` rỗng = mọi loại; `ref_` = đúng khoá phụ; `project` = một dự án trong phạm vi.
pub struct ListFilter<'a> {
    pub kinds: &'a [String],
    pub ref_: Option<&'a str>,
    pub project: Option<Uuid>,
}
pub async fn list(
    r_2b9: &PgPool,
    r_2ba: Option<&[Uuid]>,
    r_2b7: i64,
    r_2b8: i64,
) -> sqlx::Result<Vec<RecordRow>> {
    list_filtered(r_2b9, r_2ba, &ListFilter { kinds: &[], ref_: None, project: None }, r_2b7, r_2b8).await
}
pub async fn list_filtered(
    pool: &PgPool,
    scope: Option<&[Uuid]>,
    f: &ListFilter<'_>,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<RecordRow>> {
    let kinds: Option<Vec<String>> = if f.kinds.is_empty() { None } else { Some(f.kinds.to_vec()) };
    sqlx::query_as::<_, RecordRow>(&format!(
        "SELECT {COLS} FROM records \
         WHERE ($1::uuid[] IS NULL OR project_id = ANY($1)) \
           AND ($4::uuid IS NULL OR project_id = $4) \
           AND ($5::text[] IS NULL OR kind = ANY($5)) \
           AND ($6::text IS NULL OR ref = $6) \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(scope)
    .bind(limit)
    .bind(offset)
    .bind(f.project)
    .bind(kinds)
    .bind(f.ref_)
    .fetch_all(pool)
    .await
}
pub async fn list_for_submission(
    r_2bb: &PgPool,
    submission_id: Uuid,
) -> sqlx::Result<Vec<RecordRow>> {
    sqlx::query_as::<_, RecordRow>(&format!(
        "SELECT {COLS} FROM records WHERE submission_id = $1 ORDER BY created_at"
    ))
    .bind(submission_id)
    .fetch_all(r_2bb)
    .await
}
