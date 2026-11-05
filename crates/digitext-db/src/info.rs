// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T9v {
    pub id: Uuid,
    pub project_id: Uuid,
    pub requester: Option<Uuid>,
    pub hinh_thuc: String,
    pub chu_de: String,
    pub muc_dich: String,
    pub phuong_thuc: Option<String>,
    pub status: String,
    pub review_note: Option<String>,
    pub phi: Option<i64>,
    pub paid_at: Option<DateTime<Utc>>,
    pub ket_qua_key: Option<String>,
    pub bien_tap: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl T9v {
    pub fn fee_settled(&self) -> bool {
        self.phi.unwrap_or(0) == 0 || self.paid_at.is_some()
    }
}
const COLS: &str = "id, project_id, requester, hinh_thuc, chu_de, muc_dich, phuong_thuc, \
     status, review_note, phi, paid_at, ket_qua_key, bien_tap, approved_at, fulfilled_at, \
     created_at, updated_at";
pub struct Tdu<'a> {
    pub project_id: Uuid,
    pub requester: Option<Uuid>,
    pub hinh_thuc: &'a str,
    pub chu_de: &'a str,
    pub muc_dich: &'a str,
    pub phuong_thuc: Option<&'a str>,
}
pub async fn create(r_258: &PgPool, r: Tdu<'_>) -> sqlx::Result<T9v> {
    sqlx::query_as::<_, T9v>(&format!(
        "INSERT INTO info_requests (id, project_id, requester, hinh_thuc, chu_de, muc_dich, \
             phuong_thuc) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING {COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(r.project_id)
    .bind(r.requester)
    .bind(r.hinh_thuc)
    .bind(r.chu_de)
    .bind(r.muc_dich)
    .bind(r.phuong_thuc)
    .fetch_one(r_258)
    .await
}
pub async fn get(r_259: &PgPool, id: Uuid) -> sqlx::Result<Option<T9v>> {
    sqlx::query_as::<_, T9v>(&format!("SELECT {COLS} FROM info_requests WHERE id = $1"))
        .bind(id)
        .fetch_optional(r_259)
        .await
}
pub async fn list(
    r_25c: &PgPool,
    r_25d: Option<&[Uuid]>,
    r_25a: i64,
    r_25b: i64,
) -> sqlx::Result<Vec<T9v>> {
    sqlx::query_as::<_, T9v>(&format!(
        "SELECT {COLS} FROM info_requests \
         WHERE ($1::uuid[] IS NULL OR project_id = ANY($1)) \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(r_25d)
    .bind(r_25a)
    .bind(r_25b)
    .fetch_all(r_25c)
    .await
}
pub async fn approve(r_25f: &PgPool, id: Uuid, r_25e: Option<&str>) -> sqlx::Result<Option<T9v>> {
    sqlx::query_as::<_, T9v>(&format!(
        "UPDATE info_requests r SET status = 'approved', review_note = $2, \
             phi = COALESCE((SELECT f.phi_thong_tin FROM access_fee_rates f \
                             WHERE f.project_id = r.project_id), 0), \
             approved_at = now(), updated_at = now() \
         WHERE r.id = $1 AND r.status = 'submitted' RETURNING {COLS}"
    ))
    .bind(id)
    .bind(r_25e)
    .fetch_optional(r_25f)
    .await
}
pub async fn refuse(r_25h: &PgPool, id: Uuid, r_25g: &str) -> sqlx::Result<bool> {
    let r = sqlx::query(
        "UPDATE info_requests SET status = 'refused', review_note = $2, updated_at = now() \
         WHERE id = $1 AND status = 'submitted'",
    )
    .bind(id)
    .bind(r_25g)
    .execute(r_25h)
    .await?;
    Ok(r.rows_affected() > 0)
}
pub async fn confirm_payment(r_25i: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let r = sqlx::query(
        "UPDATE info_requests SET paid_at = now(), updated_at = now() \
         WHERE id = $1 AND status = 'approved' AND paid_at IS NULL",
    )
    .bind(id)
    .execute(r_25i)
    .await?;
    Ok(r.rows_affected() > 0)
}
pub async fn f_2ee(
    r_25j: &PgPool,
    id: Uuid,
    ket_qua_key: &str,
    bien_tap: Option<&str>,
) -> sqlx::Result<bool> {
    let r = sqlx::query(
        "UPDATE info_requests SET ket_qua_key = $2, bien_tap = $3, updated_at = now() \
         WHERE id = $1 AND status = 'approved'",
    )
    .bind(id)
    .bind(ket_qua_key)
    .bind(bien_tap)
    .execute(r_25j)
    .await?;
    Ok(r.rows_affected() > 0)
}
pub async fn fulfill(r_25k: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let r = sqlx::query(
        "UPDATE info_requests SET status = 'fulfilled', fulfilled_at = now(), \
             updated_at = now() \
         WHERE id = $1 AND status = 'approved' AND ket_qua_key IS NOT NULL",
    )
    .bind(id)
    .execute(r_25k)
    .await?;
    Ok(r.rows_affected() > 0)
}
pub async fn f_25g(r_25l: &PgPool, project_id: Uuid) -> sqlx::Result<(i64, i64)> {
    let r_25n: Option<(i32, i64)> = sqlx::query_as(
        "SELECT gioi_han_ket_qua, phi_thong_tin FROM access_fee_rates WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_optional(r_25l)
    .await?;
    let (r_25m, phi) = r_25n.unwrap_or((50, 0));
    Ok((r_25m as i64, phi))
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T2k {
    pub ho_so_ref: Option<String>,
    pub filename: String,
    pub page_count: Option<i32>,
    pub uploaded_at: DateTime<Utc>,
}
pub(crate) fn f_1a9(r_25o: &str) -> String {
    r_25o
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
pub async fn f_x8(
    r_25q: &PgPool,
    project_id: Uuid,
    chu_de: &str,
    r_25p: i64,
) -> sqlx::Result<Vec<T2k>> {
    sqlx::query_as::<_, T2k>(
        "SELECT COALESCE(m.ma_vb, d.ho_so_ref) AS ho_so_ref, \
                COALESCE(NULLIF(btrim(m.trich_yeu), ''), d.filename) AS filename, d.page_count, d.uploaded_at \
         FROM documents d \
         LEFT JOIN LATERAL ( \
             SELECT g.ma_ho_so_goc || '.' || lpad(v.so_thu_tu::text, 7, '0') AS ma_vb, v.trich_yeu, \
                    cl_vb_tim(v.trich_yeu, v.so_ky_hieu, v.co_quan_ban_hanh, v.loai_vb, v.tu_khoa) || ' ' || \
                    cl_hs_tim(h.tieu_de, h.so_ky_hieu_hs, h.tu_khoa, h.so_tam) AS tim \
             FROM cl_van_ban_file f JOIN cl_van_ban v ON v.id = f.van_ban_id AND v.deleted_at IS NULL \
             JOIN cl_ho_so h ON h.id = v.ho_so_id LEFT JOIN cl_ho_so_goi g ON g.ho_so_id = h.id \
             WHERE f.document_id = d.id AND f.deleted_at IS NULL LIMIT 1) m ON true \
         WHERE d.project_id = $1 AND d.status <> 'disposed' \
           AND (khong_dau(d.filename || ' ' || COALESCE(d.ho_so_ref, '')) LIKE '%' || khong_dau($2) || '%' ESCAPE '\\' \
                OR m.tim LIKE '%' || khong_dau($2) || '%' ESCAPE '\\') \
         ORDER BY 1 NULLS LAST, 2 LIMIT $3",
    )
    .bind(project_id)
    .bind(f_1a9(chu_de))
    .bind(r_25p)
    .fetch_all(r_25q)
    .await
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T75 {
    pub document_id: Uuid,
    pub filename: String,
    pub ho_so_ref: Option<String>,
    pub page_no: i32,
    pub content: String,
}
pub async fn f_1b9(
    r_25s: &PgPool,
    project_id: Uuid,
    chu_de: &str,
    r_25r: i64,
) -> sqlx::Result<Vec<T75>> {
    sqlx::query_as::<_, T75>(
        "SELECT DISTINCT ON (r.document_id, r.page_no) \
                r.document_id, d.filename, COALESCE(g.ma_ho_so_goc, d.ho_so_ref) AS ho_so_ref, r.page_no, r.content \
         FROM ocr_results r JOIN documents d ON d.id = r.document_id \
         LEFT JOIN LATERAL (SELECT g.ma_ho_so_goc FROM cl_van_ban_file f JOIN cl_van_ban v ON v.id = f.van_ban_id \
                            JOIN cl_ho_so_goi g ON g.ho_so_id = v.ho_so_id \
                            WHERE f.document_id = d.id AND f.deleted_at IS NULL LIMIT 1) g ON true \
         WHERE d.project_id = $1 AND d.status <> 'disposed' \
           AND khong_dau(r.content) LIKE '%' || khong_dau($2) || '%' ESCAPE '\\' \
         ORDER BY r.document_id, r.page_no, \
                  (r.text_source = 'ocr_corrected') DESC, r.created_at DESC \
         LIMIT $3",
    )
    .bind(project_id)
    .bind(f_1a9(chu_de))
    .bind(r_25r)
    .fetch_all(r_25s)
    .await
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tma {
    pub ho_so_ref: Option<String>,
    pub so_tai_lieu: i64,
    pub tong_trang: i64,
    pub so_trang_khop: i64,
}
pub async fn f_2i4(
    r_25u: &PgPool,
    project_id: Uuid,
    chu_de: &str,
    r_25t: i64,
) -> sqlx::Result<Vec<Tma>> {
    sqlx::query_as::<_, Tma>(
        "SELECT d.ho_so_ref, count(DISTINCT d.id) AS so_tai_lieu, \
                COALESCE(sum(DISTINCT d.page_count), 0)::bigint AS tong_trang, \
                count(DISTINCT (r.document_id, r.page_no)) AS so_trang_khop \
         FROM documents d \
         LEFT JOIN ocr_results r ON r.document_id = d.id \
             AND khong_dau(r.content) LIKE '%' || khong_dau($2) || '%' ESCAPE '\\' \
         WHERE d.project_id = $1 AND d.status <> 'disposed' \
           AND (khong_dau(d.filename || ' ' || COALESCE(d.ho_so_ref, '')) LIKE '%' || khong_dau($2) || '%' ESCAPE '\\' \
                OR r.id IS NOT NULL) \
         GROUP BY d.ho_so_ref ORDER BY d.ho_so_ref NULLS LAST LIMIT $3",
    )
    .bind(project_id)
    .bind(f_1a9(chu_de))
    .bind(r_25t)
    .fetch_all(r_25u)
    .await
}
pub type T9u = (Uuid, Option<Uuid>, Uuid, String, DateTime<Utc>);
pub async fn f_1wq(r_25v: &PgPool) -> sqlx::Result<Vec<T9u>> {
    sqlx::query_as(
        "SELECT id, requester, project_id, chu_de, created_at FROM info_requests \
         WHERE status = 'submitted' AND created_at < now() - interval '6 days'",
    )
    .fetch_all(r_25v)
    .await
}
