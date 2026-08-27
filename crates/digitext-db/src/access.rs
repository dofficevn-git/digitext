// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Ta {
    pub id: Uuid,
    pub project_id: Uuid,
    pub requester: Option<Uuid>,
    pub kind: String,
    pub muc_dich: String,
    pub phuong_thuc: Option<String>,
    pub uu_tien: Option<String>,
    pub nguoi_nhan: Option<String>,
    pub so_dinh_danh: Option<String>,
    pub status: String,
    pub review_note: Option<String>,
    pub phi: Option<i64>,
    pub paid_at: Option<DateTime<Utc>>,
    pub han_su_dung: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl Ta {
    pub fn fee_settled(&self) -> bool {
        self.phi.unwrap_or(0) == 0 || self.paid_at.is_some()
    }
    pub fn within_deadline(&self) -> bool {
        self.han_su_dung.is_none_or(|r_20a| Utc::now() <= r_20a)
    }
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T9 {
    pub id: Uuid,
    pub request_id: Uuid,
    pub document_id: Uuid,
    pub stt: i32,
    pub verdict: String,
    pub ly_do: Option<String>,
    pub dieu_kien: bool,
    pub user_copy_key: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
}
const COLS: &str = "id, project_id, requester, kind, muc_dich, phuong_thuc, uu_tien, \
     nguoi_nhan, so_dinh_danh, status, review_note, phi, paid_at, han_su_dung, approved_at, \
     created_at, updated_at";
const ITEM_COLS: &str =
    "id, request_id, document_id, stt, verdict, ly_do, dieu_kien, user_copy_key, deleted_at";
pub struct Tdq<'a> {
    pub project_id: Uuid,
    pub requester: Option<Uuid>,
    pub kind: &'a str,
    pub muc_dich: &'a str,
    pub phuong_thuc: Option<&'a str>,
    pub uu_tien: Option<&'a str>,
    pub nguoi_nhan: Option<&'a str>,
    pub so_dinh_danh: Option<&'a str>,
}
pub async fn create(r_20d: &PgPool, r_20e: Tdq<'_>, r_20b: &[Uuid]) -> sqlx::Result<(Ta, Vec<T9>)> {
    let mut r_20g = r_20d.begin().await?;
    let r_20f = sqlx::query_as::<_, Ta>(&format!(
        "INSERT INTO access_requests (id, project_id, requester, kind, muc_dich, phuong_thuc, \
             uu_tien, nguoi_nhan, so_dinh_danh) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING {COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(r_20e.project_id)
    .bind(r_20e.requester)
    .bind(r_20e.kind)
    .bind(r_20e.muc_dich)
    .bind(r_20e.phuong_thuc)
    .bind(r_20e.uu_tien)
    .bind(r_20e.nguoi_nhan)
    .bind(r_20e.so_dinh_danh)
    .fetch_one(&mut *r_20g)
    .await?;
    let mut items = Vec::with_capacity(r_20b.len());
    for (i, doc) in r_20b.iter().enumerate() {
        let r_20c = sqlx::query_as::<_, T9>(&format!(
            "INSERT INTO access_request_items (id, request_id, document_id, stt) \
             VALUES ($1, $2, $3, $4) RETURNING {ITEM_COLS}"
        ))
        .bind(Uuid::now_v7())
        .bind(r_20f.id)
        .bind(doc)
        .bind((i + 1) as i32)
        .fetch_one(&mut *r_20g)
        .await?;
        items.push(r_20c);
    }
    r_20g.commit().await?;
    Ok((r_20f, items))
}
pub async fn get(r_20h: &PgPool, id: Uuid) -> sqlx::Result<Option<Ta>> {
    sqlx::query_as::<_, Ta>(&format!("SELECT {COLS} FROM access_requests WHERE id = $1"))
        .bind(id)
        .fetch_optional(r_20h)
        .await
}
pub async fn items(r_20i: &PgPool, request_id: Uuid) -> sqlx::Result<Vec<T9>> {
    sqlx::query_as::<_, T9>(&format!(
        "SELECT {ITEM_COLS} FROM access_request_items WHERE request_id = $1 ORDER BY stt"
    ))
    .bind(request_id)
    .fetch_all(r_20i)
    .await
}
pub async fn list(
    r_20l: &PgPool,
    r_20m: Option<&[Uuid]>,
    r_20j: i64,
    r_20k: i64,
) -> sqlx::Result<Vec<Ta>> {
    sqlx::query_as::<_, Ta>(&format!(
        "SELECT {COLS} FROM access_requests \
         WHERE ($1::uuid[] IS NULL OR project_id = ANY($1)) \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(r_20m)
    .bind(r_20j)
    .bind(r_20k)
    .fetch_all(r_20l)
    .await
}
pub async fn review_item(
    r_20o: &PgPool,
    r_20n: Uuid,
    verdict: &str,
    ly_do: Option<&str>,
    dieu_kien: bool,
) -> sqlx::Result<bool> {
    if verdict != "duyet" && verdict != "tu_choi" {
        return Ok(false);
    }
    let r_20p = sqlx::query(
        "UPDATE access_request_items i SET verdict = $2, ly_do = $3, dieu_kien = $4 \
         FROM access_requests q \
         WHERE i.id = $1 AND q.id = i.request_id AND q.status = 'submitted'",
    )
    .bind(r_20n)
    .bind(verdict)
    .bind(ly_do)
    .bind(dieu_kien)
    .execute(r_20o)
    .await?;
    Ok(r_20p.rows_affected() > 0)
}
pub async fn f_1c0(r_20q: &PgPool, project_id: Uuid) -> sqlx::Result<(i64, i64)> {
    let r_20r: Option<(i64, i64)> = sqlx::query_as(
        "SELECT phi_doc_tai_lieu, phi_sao_trang FROM access_fee_rates WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_optional(r_20q)
    .await?;
    Ok(r_20r.unwrap_or((0, 0)))
}
pub async fn f_2e2(
    r_20s: &PgPool,
    project_id: Uuid,
    phi_doc_tai_lieu: i64,
    phi_sao_trang: i64,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO access_fee_rates (project_id, phi_doc_tai_lieu, phi_sao_trang) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (project_id) DO UPDATE \
             SET phi_doc_tai_lieu = $2, phi_sao_trang = $3, updated_at = now()",
    )
    .bind(project_id)
    .bind(phi_doc_tai_lieu)
    .bind(phi_sao_trang)
    .execute(r_20s)
    .await?;
    Ok(())
}
pub async fn approve(r_20y: &PgPool, id: Uuid, r_20u: Option<&str>) -> sqlx::Result<Option<Ta>> {
    let Some(r_20z) = get(r_20y, id).await? else {
        return Ok(None);
    };
    if r_20z.status != "submitted" {
        return Ok(None);
    }
    let mut r_211 = r_20y.begin().await?;
    let (r_20t, r_20v): (i64, i64) = sqlx::query_as(
        "SELECT count(*), COALESCE(sum(COALESCE(d.page_count, 0)), 0)::bigint \
         FROM access_request_items i JOIN documents d ON d.id = i.document_id \
         WHERE i.request_id = $1 AND i.verdict = 'duyet'",
    )
    .bind(id)
    .fetch_one(&mut *r_211)
    .await?;
    if r_20t == 0 {
        r_211.rollback().await?;
        return Ok(None);
    }
    let (r_20w, r_20x): (i64, i64) = sqlx::query_as(
        "SELECT phi_doc_tai_lieu, phi_sao_trang FROM access_fee_rates WHERE project_id = $1",
    )
    .bind(r_20z.project_id)
    .fetch_optional(&mut *r_211)
    .await?
    .unwrap_or((0, 0));
    let phi = if r_20z.kind == "doc" {
        r_20w.saturating_mul(r_20t)
    } else {
        r_20x.saturating_mul(r_20v)
    };
    let r_210 = sqlx::query_as::<_, Ta>(&format!(
        "UPDATE access_requests SET status = 'approved', review_note = $2, phi = $3, \
             approved_at = now(), han_su_dung = now() + interval '15 days', \
             updated_at = now() \
         WHERE id = $1 AND status = 'submitted' RETURNING {COLS}"
    ))
    .bind(id)
    .bind(r_20u)
    .bind(phi)
    .fetch_optional(&mut *r_211)
    .await?;
    r_211.commit().await?;
    Ok(r_210)
}
pub async fn refuse(r_213: &PgPool, id: Uuid, r_212: &str) -> sqlx::Result<bool> {
    let r_214 = sqlx::query(
        "UPDATE access_requests SET status = 'refused', review_note = $2, updated_at = now() \
         WHERE id = $1 AND status = 'submitted'",
    )
    .bind(id)
    .bind(r_212)
    .execute(r_213)
    .await?;
    Ok(r_214.rows_affected() > 0)
}
pub async fn confirm_payment(r_215: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let r_216 = sqlx::query(
        "UPDATE access_requests SET paid_at = now(), updated_at = now() \
         WHERE id = $1 AND status = 'approved' AND paid_at IS NULL",
    )
    .bind(id)
    .execute(r_215)
    .await?;
    Ok(r_216.rows_affected() > 0)
}
pub async fn f_2ej(r_219: &PgPool, r_217: Uuid, r_218: &str) -> sqlx::Result<()> {
    sqlx::query("UPDATE access_request_items SET user_copy_key = $2 WHERE id = $1")
        .bind(r_217)
        .bind(r_218)
        .execute(r_219)
        .await?;
    Ok(())
}
pub type T71 = (Uuid, String);
pub async fn f_1ag(r_21a: &PgPool) -> sqlx::Result<Vec<T71>> {
    sqlx::query_as(
        "SELECT i.id, i.user_copy_key \
         FROM access_request_items i JOIN access_requests q ON q.id = i.request_id \
         WHERE q.status = 'approved' AND q.han_su_dung < now() - interval '5 days' \
           AND i.user_copy_key IS NOT NULL AND i.deleted_at IS NULL",
    )
    .fetch_all(r_21a)
    .await
}
pub async fn f_1r9(r_21c: &PgPool, r_21b: &[Uuid]) -> sqlx::Result<u64> {
    let r_21d =
        sqlx::query("UPDATE access_request_items SET deleted_at = now() WHERE id = ANY($1)")
            .bind(r_21b)
            .execute(r_21c)
            .await?;
    Ok(r_21d.rows_affected())
}
pub async fn f_1rc(r_21e: &PgPool) -> sqlx::Result<u64> {
    let r_21f = sqlx::query(
        "UPDATE access_requests q SET status = 'expired', updated_at = now() \
         WHERE q.status = 'approved' AND q.han_su_dung < now() - interval '5 days' \
           AND NOT EXISTS (SELECT 1 FROM access_request_items i \
                           WHERE i.request_id = q.id \
                             AND i.user_copy_key IS NOT NULL AND i.deleted_at IS NULL)",
    )
    .execute(r_21e)
    .await?;
    Ok(r_21f.rows_affected())
}
