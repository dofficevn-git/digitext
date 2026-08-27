// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T4w {
    pub id: Uuid,
    pub project_id: Uuid,
    pub document_id: Uuid,
    pub batch_id: Option<Uuid>,
    pub ly_do: String,
    pub dup_of: Option<Uuid>,
    pub ghi_chu: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub filename: String,
    pub ho_so_ref: Option<String>,
    pub page_count: Option<i32>,
    pub het_han_luu_tru: Option<DateTime<Utc>>,
    pub doc_status: String,
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T4v {
    pub id: Uuid,
    pub project_id: Uuid,
    pub nam: i32,
    pub status: String,
    pub hoi_dong_note: Option<String>,
    pub tham_dinh_note: Option<String>,
    pub tu_quyet: bool,
    pub quyet_dinh_so: Option<String>,
    pub decided_by: Option<Uuid>,
    pub decided_at: Option<DateTime<Utc>>,
    pub executed_by: Option<Uuid>,
    pub executed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T4x {
    pub id: Uuid,
    pub batch_id: Option<Uuid>,
    pub entry_id: Option<Uuid>,
    pub document_id: Option<Uuid>,
    pub action: String,
    pub actor: Option<Uuid>,
    pub detail: Option<String>,
    pub created_at: DateTime<Utc>,
}
const ENTRY_COLS: &str = "e.id, e.project_id, e.document_id, e.batch_id, e.ly_do, e.dup_of, \
     e.ghi_chu, e.status, e.created_at, e.updated_at, d.filename, d.ho_so_ref, d.page_count, \
     d.het_han_luu_tru, d.status AS doc_status";
const BATCH_COLS: &str = "id, project_id, nam, status, hoi_dong_note, tham_dinh_note, tu_quyet, \
     quyet_dinh_so, decided_by, decided_at, executed_by, executed_at, created_by, created_at, \
     updated_at";
#[derive(Debug, Clone, Copy, Default)]
pub struct Tbb {
    pub batch_id: Option<Uuid>,
    pub entry_id: Option<Uuid>,
    pub document_id: Option<Uuid>,
}
pub async fn log(
    r_222: &PgPool,
    project_id: Uuid,
    r_223: Tbb,
    action: &str,
    actor: Option<Uuid>,
    detail: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO disposal_log (id, project_id, batch_id, entry_id, document_id, action, \
             actor, detail) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(Uuid::now_v7())
    .bind(project_id)
    .bind(r_223.batch_id)
    .bind(r_223.entry_id)
    .bind(r_223.document_id)
    .bind(action)
    .bind(actor)
    .bind(detail)
    .execute(r_222)
    .await?;
    Ok(())
}
pub async fn f_1pf(r_224: &PgPool, batch_id: Uuid) -> sqlx::Result<Vec<T4x>> {
    sqlx::query_as::<_, T4x>(
        "SELECT id, batch_id, entry_id, document_id, action, actor, detail, created_at \
         FROM disposal_log WHERE batch_id = $1 ORDER BY created_at",
    )
    .bind(batch_id)
    .fetch_all(r_224)
    .await
}
pub async fn f_18b(r_226: &PgPool) -> sqlx::Result<Vec<(Uuid, Uuid)>> {
    let r_228: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "INSERT INTO disposal_entries (id, project_id, document_id, ly_do, dup_of) \
         SELECT gen_random_uuid(), d.project_id, d.id, 'trung_lap', dup.id \
         FROM documents d \
         JOIN LATERAL ( \
             SELECT o.id FROM documents o \
             WHERE o.project_id = d.project_id AND o.id <> d.id \
               AND o.ho_so_ref IS NOT DISTINCT FROM d.ho_so_ref \
               AND o.filename = d.filename \
               AND o.uploaded_at < d.uploaded_at AND o.status <> 'disposed' \
             ORDER BY o.uploaded_at LIMIT 1 \
         ) dup ON true \
         WHERE d.status <> 'disposed' AND d.ho_so_ref IS NOT NULL \
           AND NOT EXISTS (SELECT 1 FROM disposal_entries e \
                           WHERE e.document_id = d.id) \
         RETURNING project_id, document_id",
    )
    .fetch_all(r_226)
    .await?;
    for (r_227, r_225) in &r_228 {
        log(
                r_226,
                *r_227,
                Tbb {
                    document_id: Some(*r_225),
                    ..Default::default()
                },
                "phat_hien_trung_lap",
                None,
                Some(
                    "Hệ thống tự động đưa vào Danh mục — Điều 48.2",
                ),
            )
            .await?;
    }
    Ok(r_228)
}
pub async fn f_1ah(r_22a: &PgPool) -> sqlx::Result<Vec<(Uuid, Uuid)>> {
    let r_22c: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "INSERT INTO disposal_entries (id, project_id, document_id, ly_do) \
         SELECT gen_random_uuid(), d.project_id, d.id, 'het_thoi_han' \
         FROM documents d \
         WHERE d.het_han_luu_tru IS NOT NULL AND d.het_han_luu_tru < now() \
           AND d.status <> 'disposed' \
           AND NOT EXISTS (SELECT 1 FROM disposal_entries e \
                           WHERE e.document_id = d.id) \
         RETURNING project_id, document_id",
    )
    .fetch_all(r_22a)
    .await?;
    for (r_22b, r_229) in &r_22c {
        log(
                r_22a,
                *r_22b,
                Tbb {
                    document_id: Some(*r_229),
                    ..Default::default()
                },
                "het_thoi_han_luu_tru",
                None,
                Some(
                    "Hệ thống tự động đưa vào Danh mục — Điều 48.3",
                ),
            )
            .await?;
    }
    Ok(r_22c)
}
pub async fn set_retention(
    r_22e: &PgPool,
    document_id: Uuid,
    r_22d: Option<DateTime<Utc>>,
) -> sqlx::Result<bool> {
    let r_22f = sqlx::query(
        "UPDATE documents SET het_han_luu_tru = $2 WHERE id = $1 AND status <> 'disposed'",
    )
    .bind(document_id)
    .bind(r_22d)
    .execute(r_22e)
    .await?;
    Ok(r_22f.rows_affected() > 0)
}
pub async fn entries(
    r_22i: &PgPool,
    r_22j: Option<&[Uuid]>,
    r_22g: i64,
    r_22h: i64,
) -> sqlx::Result<Vec<T4w>> {
    sqlx::query_as::<_, T4w>(&format!(
        "SELECT {ENTRY_COLS} FROM disposal_entries e JOIN documents d ON d.id = e.document_id \
         WHERE ($1::uuid[] IS NULL OR e.project_id = ANY($1)) \
         ORDER BY e.created_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(r_22j)
    .bind(r_22g)
    .bind(r_22h)
    .fetch_all(r_22i)
    .await
}
pub async fn entry(r_22k: &PgPool, id: Uuid) -> sqlx::Result<Option<T4w>> {
    sqlx::query_as::<_, T4w>(&format!(
        "SELECT {ENTRY_COLS} FROM disposal_entries e JOIN documents d ON d.id = e.document_id \
         WHERE e.id = $1"
    ))
    .bind(id)
    .fetch_optional(r_22k)
    .await
}
pub async fn set_note(r_22l: &PgPool, entry_id: Uuid, ghi_chu: Option<&str>) -> sqlx::Result<bool> {
    let r_22m =
        sqlx::query("UPDATE disposal_entries SET ghi_chu = $2, updated_at = now() WHERE id = $1")
            .bind(entry_id)
            .bind(ghi_chu)
            .execute(r_22l)
            .await?;
    Ok(r_22m.rows_affected() > 0)
}
pub async fn keep_entry(
    r_22o: &PgPool,
    entry_id: Uuid,
    actor: Option<Uuid>,
    r_22q: &str,
) -> sqlx::Result<bool> {
    let r_22r: Option<(Uuid, Uuid)> = sqlx::query_as(
        "UPDATE disposal_entries SET status = 'giu_lai', updated_at = now() \
         WHERE id = $1 AND status = 'xem_xet' AND batch_id IS NULL \
         RETURNING project_id, document_id",
    )
    .bind(entry_id)
    .fetch_optional(r_22o)
    .await?;
    if let Some((r_22p, r_22n)) = r_22r {
        log(
            r_22o,
            r_22p,
            Tbb {
                batch_id: None,
                entry_id: Some(entry_id),
                document_id: Some(r_22n),
            },
            "giu_lai",
            actor,
            Some(r_22q),
        )
        .await?;
        return Ok(true);
    }
    Ok(false)
}
pub async fn create_batch(
    r_22t: &PgPool,
    project_id: Uuid,
    nam: i32,
    r_22s: &[Uuid],
    created_by: Option<Uuid>,
) -> sqlx::Result<Option<T4v>> {
    let mut r_22v = r_22t.begin().await?;
    let batch = sqlx::query_as::<_, T4v>(&format!(
        "INSERT INTO disposal_batches (id, project_id, nam, created_by) \
         VALUES ($1, $2, $3, $4) RETURNING {BATCH_COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(project_id)
    .bind(nam)
    .bind(created_by)
    .fetch_one(&mut *r_22v)
    .await?;
    let r_22u = sqlx::query(
        "UPDATE disposal_entries SET batch_id = $1, updated_at = now() \
         WHERE id = ANY($2) AND project_id = $3 AND status = 'xem_xet' AND batch_id IS NULL",
    )
    .bind(batch.id)
    .bind(r_22s)
    .bind(project_id)
    .execute(&mut *r_22v)
    .await?;
    if r_22u.rows_affected() != r_22s.len() as u64 || r_22s.is_empty() {
        r_22v.rollback().await?;
        return Ok(None);
    }
    r_22v.commit().await?;
    log(
        r_22t,
        project_id,
        Tbb {
            batch_id: Some(batch.id),
            ..Default::default()
        },
        "lap_dot_xet_huy",
        created_by,
        Some(&format!(
            "{} tài liệu — Điều 49.1",
            r_22s.len()
        )),
    )
    .await?;
    Ok(Some(batch))
}
pub async fn batch(r_22w: &PgPool, id: Uuid) -> sqlx::Result<Option<T4v>> {
    sqlx::query_as::<_, T4v>(&format!(
        "SELECT {BATCH_COLS} FROM disposal_batches WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(r_22w)
    .await
}
pub async fn batches(
    r_22z: &PgPool,
    r_230: Option<&[Uuid]>,
    r_22x: i64,
    r_22y: i64,
) -> sqlx::Result<Vec<T4v>> {
    sqlx::query_as::<_, T4v>(&format!(
        "SELECT {BATCH_COLS} FROM disposal_batches \
         WHERE ($1::uuid[] IS NULL OR project_id = ANY($1)) \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(r_230)
    .bind(r_22x)
    .bind(r_22y)
    .fetch_all(r_22z)
    .await
}
pub async fn f_u7(r_231: &PgPool, batch_id: Uuid) -> sqlx::Result<Vec<T4w>> {
    sqlx::query_as::<_, T4w>(&format!(
        "SELECT {ENTRY_COLS} FROM disposal_entries e JOIN documents d ON d.id = e.document_id \
         WHERE e.batch_id = $1 ORDER BY e.created_at"
    ))
    .bind(batch_id)
    .fetch_all(r_231)
    .await
}
pub async fn council(
    r_233: &PgPool,
    batch_id: Uuid,
    r_232: &str,
    actor: Option<Uuid>,
) -> sqlx::Result<bool> {
    let r_235: Option<(Uuid,)> = sqlx::query_as(
        "UPDATE disposal_batches SET status = 'tham_dinh', hoi_dong_note = $2, \
             updated_at = now() \
         WHERE id = $1 AND status = 'hoi_dong' RETURNING project_id",
    )
    .bind(batch_id)
    .bind(r_232)
    .fetch_optional(r_233)
    .await?;
    if let Some((r_234,)) = r_235 {
        log(
            r_233,
            r_234,
            Tbb {
                batch_id: Some(batch_id),
                ..Default::default()
            },
            "hoi_dong_tong_hop",
            actor,
            Some(r_232),
        )
        .await?;
        return Ok(true);
    }
    Ok(false)
}
pub async fn appraise(
    r_237: &PgPool,
    batch_id: Uuid,
    r_236: &str,
    actor: Option<Uuid>,
) -> sqlx::Result<bool> {
    let r_239: Option<(Uuid,)> = sqlx::query_as(
        "UPDATE disposal_batches SET tham_dinh_note = $2, updated_at = now() \
         WHERE id = $1 AND status = 'tham_dinh' RETURNING project_id",
    )
    .bind(batch_id)
    .bind(r_236)
    .fetch_optional(r_237)
    .await?;
    if let Some((r_238,)) = r_239 {
        log(
            r_237,
            r_238,
            Tbb {
                batch_id: Some(batch_id),
                ..Default::default()
            },
            "y_kien_tham_dinh",
            actor,
            Some(r_236),
        )
        .await?;
        return Ok(true);
    }
    Ok(false)
}
pub async fn decide(
    r_23a: &PgPool,
    batch_id: Uuid,
    quyet_dinh_so: &str,
    tu_quyet: bool,
    actor: Option<Uuid>,
) -> sqlx::Result<bool> {
    let r_23c: Option<(Uuid,)> = sqlx::query_as(
        "UPDATE disposal_batches SET status = 'quyet_dinh', quyet_dinh_so = $2, \
             tu_quyet = $3, decided_by = $4, decided_at = now(), updated_at = now() \
         WHERE id = $1 AND status = 'tham_dinh' \
           AND ($3 OR tham_dinh_note IS NOT NULL) \
         RETURNING project_id",
    )
    .bind(batch_id)
    .bind(quyet_dinh_so)
    .bind(tu_quyet)
    .bind(actor)
    .fetch_optional(r_23a)
    .await?;
    if let Some((r_23b,)) = r_23c {
        log(
            r_23a,
            r_23b,
            Tbb {
                batch_id: Some(batch_id),
                ..Default::default()
            },
            "quyet_dinh_huy",
            actor,
            Some(&format!(
                "Quyết định {quyet_dinh_so}{}",
                if tu_quyet {
                    " (tự quyết — Điều 49.8)"
                } else {
                    " (sau thẩm định — Điều 49.6)"
                }
            )),
        )
        .await?;
        return Ok(true);
    }
    Ok(false)
}
pub async fn f_1ad(
    r_23e: &PgPool,
    batch_id: Uuid,
    actor: Option<Uuid>,
) -> sqlx::Result<Option<Vec<T4w>>> {
    let r_23g: Option<(Uuid,)> = sqlx::query_as(
        "UPDATE disposal_batches SET status = 'da_huy', executed_by = $2, \
             executed_at = now(), updated_at = now() \
         WHERE id = $1 AND status = 'quyet_dinh' RETURNING project_id",
    )
    .bind(batch_id)
    .bind(actor)
    .fetch_optional(r_23e)
    .await?;
    match r_23g {
        Some((r_23f,)) => {
            log(
                r_23e,
                r_23f,
                Tbb {
                    batch_id: Some(batch_id),
                    ..Default::default()
                },
                "bat_dau_huy",
                actor,
                Some("Thao tác hủy trong Hệ thống — Điều 49.7.a"),
            )
            .await?;
        }
        None => match batch(r_23e, batch_id).await? {
            Some(r_23d) if r_23d.status == "da_huy" => {}
            _ => return Ok(None),
        },
    }
    Ok(Some(f_u7(r_23e, batch_id).await?))
}
pub async fn f_1d0(
    r_23j: &PgPool,
    entry: &T4w,
    actor: Option<Uuid>,
    r_23h: &str,
) -> sqlx::Result<bool> {
    let mut r_23k = r_23j.begin().await?;
    let r_23i = sqlx::query(
        "UPDATE disposal_entries SET status = 'da_huy', updated_at = now() \
         WHERE id = $1 AND status = 'xem_xet'",
    )
    .bind(entry.id)
    .execute(&mut *r_23k)
    .await?;
    if r_23i.rows_affected() == 0 {
        r_23k.rollback().await?;
        return Ok(false);
    }
    sqlx::query("UPDATE documents SET status = 'disposed' WHERE id = $1")
        .bind(entry.document_id)
        .execute(&mut *r_23k)
        .await?;
    sqlx::query(
        "INSERT INTO disposal_log (id, project_id, batch_id, entry_id, document_id, action, \
             actor, detail) VALUES ($1, $2, $3, $4, $5, 'da_huy_tai_lieu', $6, $7)",
    )
    .bind(Uuid::now_v7())
    .bind(entry.project_id)
    .bind(entry.batch_id)
    .bind(entry.id)
    .bind(entry.document_id)
    .bind(actor)
    .bind(r_23h)
    .execute(&mut *r_23k)
    .await?;
    r_23k.commit().await?;
    Ok(true)
}
