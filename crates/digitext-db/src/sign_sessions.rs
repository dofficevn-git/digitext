// Copyright (c) 2026 DigiText. All rights reserved.
use crate::ocr_jobs;
use chrono::{DateTime, Utc};
use digitext_core::{DocStatus, JobKind};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tlq {
    pub id: Uuid,
    pub document_id: Uuid,
    pub signer_user_id: Option<Uuid>,
    pub status: String,
    pub signer_cert: Option<Vec<u8>>,
    pub signer_intermediates: Option<Vec<Vec<u8>>>,
    pub tbs_digest: Option<Vec<u8>>,
    pub signed_attrs: Option<Vec<u8>>,
    pub prepared_key: Option<String>,
    pub signed_key: Option<String>,
    pub reject_reason: Option<String>,
    pub lease_until: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
const COLS: &str = "id, document_id, signer_user_id, status, signer_cert, signer_intermediates, \
     tbs_digest, signed_attrs, prepared_key, signed_key, reject_reason, lease_until, created_at, \
     updated_at";
#[allow(clippy::too_many_arguments)]
pub async fn begin(
    r_2bo: &PgPool,
    document_id: Uuid,
    signer_user_id: Option<Uuid>,
    signer_cert: Option<&[u8]>,
    signer_intermediates: Option<&[Vec<u8>]>,
    r_2bm: i64,
) -> sqlx::Result<Option<Tlq>> {
    let mut r_2bq = r_2bo.begin().await?;
    let r_2bn = sqlx::query("UPDATE documents SET status = $2 WHERE id = $1 AND status = $3")
        .bind(document_id)
        .bind(DocStatus::Signing.as_str())
        .bind(DocStatus::ToSign.as_str())
        .execute(&mut *r_2bq)
        .await?;
    if r_2bn.rows_affected() == 0 {
        return Ok(None);
    }
    sqlx::query(
        "UPDATE ocr_jobs SET status = 'running', updated_at = now() \
         WHERE document_id = $1 AND kind = $2 AND status = 'queued'",
    )
    .bind(document_id)
    .bind(JobKind::SignWait.as_str())
    .execute(&mut *r_2bq)
    .await?;
    let r_2bp = sqlx::query_as::<_, Tlq>(&format!(
        "INSERT INTO sign_sessions \
           (id, document_id, signer_user_id, signer_cert, signer_intermediates, lease_until) \
         VALUES ($1, $2, $3, $4, $5, now() + ($6 * interval '1 second')) RETURNING {COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(document_id)
    .bind(signer_user_id)
    .bind(signer_cert)
    .bind(signer_intermediates)
    .bind(r_2bm)
    .fetch_one(&mut *r_2bq)
    .await?;
    r_2bq.commit().await?;
    Ok(Some(r_2bp))
}
pub async fn f_2ea(
    r_2br: &PgPool,
    r_2bt: Uuid,
    tbs_digest: &[u8],
    signed_attrs: &[u8],
    prepared_key: &str,
) -> sqlx::Result<bool> {
    let r_2bs = sqlx::query(
        "UPDATE sign_sessions SET tbs_digest = $2, signed_attrs = $3, prepared_key = $4, \
             updated_at = now() WHERE id = $1 AND status = 'open'",
    )
    .bind(r_2bt)
    .bind(tbs_digest)
    .bind(signed_attrs)
    .bind(prepared_key)
    .execute(r_2br)
    .await?;
    Ok(r_2bs.rows_affected() > 0)
}
pub async fn complete(r_2bu: &PgPool, r_2bv: Uuid, signed_key: &str) -> sqlx::Result<bool> {
    let mut r_2bw = r_2bu.begin().await?;
    let doc_id = sqlx::query_scalar::<_, Uuid>(
        "UPDATE sign_sessions SET status = 'signed', signed_key = $2, updated_at = now() \
         WHERE id = $1 AND status = 'open' RETURNING document_id",
    )
    .bind(r_2bv)
    .bind(signed_key)
    .fetch_optional(&mut *r_2bw)
    .await?;
    let Some(doc_id) = doc_id else {
        return Ok(false);
    };
    f_162(&mut r_2bw, doc_id, DocStatus::Signed, DocStatus::Signing).await?;
    f_2f6(&mut r_2bw, doc_id, "done", "running").await?;
    r_2bw.commit().await?;
    Ok(true)
}
pub async fn f_26q(r_2bx: &PgPool, r_2bz: Uuid, r_2by: &str) -> sqlx::Result<bool> {
    let mut r_2c0 = r_2bx.begin().await?;
    let doc_id = sqlx::query_scalar::<_, Uuid>(
        "UPDATE sign_sessions SET status = 'rejected', reject_reason = $2, updated_at = now() \
         WHERE id = $1 AND status = 'open' RETURNING document_id",
    )
    .bind(r_2bz)
    .bind(r_2by)
    .fetch_optional(&mut *r_2c0)
    .await?;
    let Some(doc_id) = doc_id else {
        return Ok(false);
    };
    f_162(
        &mut r_2c0,
        doc_id,
        DocStatus::SignRejected,
        DocStatus::Signing,
    )
    .await?;
    f_2f6(&mut r_2c0, doc_id, "done", "running").await?;
    r_2c0.commit().await?;
    Ok(true)
}
pub async fn f_272(r_2c3: &PgPool) -> sqlx::Result<u64> {
    let mut r_2c4 = r_2c3.begin().await?;
    let r_2c2: Vec<Uuid> = sqlx::query_scalar(
        "UPDATE sign_sessions SET status = 'released', updated_at = now() \
         WHERE status = 'open' AND lease_until < now() RETURNING document_id",
    )
    .fetch_all(&mut *r_2c4)
    .await?;
    for &r_2c1 in &r_2c2 {
        f_162(&mut r_2c4, r_2c1, DocStatus::ToSign, DocStatus::Signing).await?;
        f_2f6(&mut r_2c4, r_2c1, "queued", "running").await?;
    }
    r_2c4.commit().await?;
    Ok(r_2c2.len() as u64)
}
/// Trả tài liệu đang kẹt phiên ký MỞ của chính người ký này về "chờ ký" (ký lỗi giữa chừng: rút
/// token, mạng đứt…) — không phải đợi hết hạn phiên 30 phút mới ký lại được.
pub async fn release_open_by_signer(pool: &PgPool, document_id: Uuid, signer: Uuid) -> sqlx::Result<bool> {
    let mut tx = pool.begin().await?;
    let released: Option<Uuid> = sqlx::query_scalar(
        "UPDATE sign_sessions SET status = 'released', updated_at = now()          WHERE document_id = $1 AND status = 'open' AND signer_user_id = $2 RETURNING document_id",
    )
    .bind(document_id)
    .bind(signer)
    .fetch_optional(&mut *tx)
    .await?;
    if released.is_none() {
        return Ok(false);
    }
    f_162(&mut tx, document_id, DocStatus::ToSign, DocStatus::Signing).await?;
    f_2f6(&mut tx, document_id, "queued", "running").await?;
    tx.commit().await?;
    Ok(true)
}
/// Huỷ chữ ký vừa ký của CHÍNH người ký (bấm "Huỷ" giữa lô ký, 13/10/2026): phiên `signed` → `released`,
/// tài liệu `signed` → `to_sign`, việc sign_wait `done` → `queued`. Chỉ khi tài liệu còn đúng "đã ký" (chưa đóng
/// gói / nộp lưu). Trả (signed_key, prepared_key) để bên gọi xoá object; None = không huỷ được.
pub async fn unsign_by_signer(pool: &PgPool, document_id: Uuid, signer: Uuid) -> sqlx::Result<Option<(Option<String>, Option<String>)>> {
    let mut tx = pool.begin().await?;
    let keys: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "UPDATE sign_sessions s SET status = 'released', signed_key = NULL, prepared_key = NULL, updated_at = now() \
         FROM (SELECT id, signed_key, prepared_key FROM sign_sessions \
               WHERE document_id = $1 AND status = 'signed' AND signer_user_id = $2 \
               ORDER BY updated_at DESC LIMIT 1 FOR UPDATE) old \
         WHERE s.id = old.id RETURNING old.signed_key, old.prepared_key",
    )
    .bind(document_id)
    .bind(signer)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(keys) = keys else {
        return Ok(None);
    };
    let doc = sqlx::query("UPDATE documents SET status = $2 WHERE id = $1 AND status = $3")
        .bind(document_id)
        .bind(DocStatus::ToSign.as_str())
        .bind(DocStatus::Signed.as_str())
        .execute(&mut *tx)
        .await?;
    if doc.rows_affected() == 0 {
        // Tài liệu đã đi tiếp (đóng gói / nộp lưu…) → không huỷ.
        tx.rollback().await?;
        return Ok(None);
    }
    f_2f6(&mut tx, document_id, "queued", "done").await?;
    tx.commit().await?;
    Ok(Some(keys))
}
pub async fn f_29r(r_2c5: &PgPool, document_id: Uuid) -> sqlx::Result<bool> {
    let r_2c6 = sqlx::query("UPDATE documents SET status = $2 WHERE id = $1 AND status = $3")
        .bind(document_id)
        .bind(DocStatus::ToSign.as_str())
        .bind(DocStatus::SignRejected.as_str())
        .execute(r_2c5)
        .await?;
    if r_2c6.rows_affected() == 0 {
        return Ok(false);
    }
    ocr_jobs::f_19u(r_2c5, document_id).await?;
    Ok(true)
}
pub async fn get(r_2c7: &PgPool, id: Uuid) -> sqlx::Result<Option<Tlq>> {
    sqlx::query_as::<_, Tlq>(&format!("SELECT {COLS} FROM sign_sessions WHERE id = $1"))
        .bind(id)
        .fetch_optional(r_2c7)
        .await
}
pub async fn f_1er(r_2c8: &PgPool, document_id: Uuid) -> sqlx::Result<Option<Tlq>> {
    sqlx::query_as::<_, Tlq>(&format!(
        "SELECT {COLS} FROM sign_sessions WHERE document_id = $1 AND status = 'open'"
    ))
    .bind(document_id)
    .fetch_optional(r_2c8)
    .await
}
async fn f_162(
    r_2c9: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document_id: Uuid,
    to: DocStatus,
    from: DocStatus,
) -> sqlx::Result<()> {
    debug_assert!(
        from.can_transition_to(to),
        "{from} → {to} không hợp lệ (§S7.1)"
    );
    sqlx::query("UPDATE documents SET status = $2 WHERE id = $1 AND status = $3")
        .bind(document_id)
        .bind(to.as_str())
        .bind(from.as_str())
        .execute(&mut **r_2c9)
        .await?;
    Ok(())
}
async fn f_2f6(
    r_2ca: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document_id: Uuid,
    to: &str,
    from: &str,
) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE ocr_jobs SET status = $3, updated_at = now() \
         WHERE document_id = $1 AND kind = $2 AND status = $4",
    )
    .bind(document_id)
    .bind(JobKind::SignWait.as_str())
    .bind(to)
    .bind(from)
    .execute(&mut **r_2ca)
    .await?;
    Ok(())
}
