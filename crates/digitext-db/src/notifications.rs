// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tec {
    pub id: Uuid,
    pub user_id: Uuid,
    pub submission_id: Option<Uuid>,
    pub info_request_id: Option<Uuid>,
    pub kind: String,
    pub message: String,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
const COLS: &str =
    "id, user_id, submission_id, info_request_id, kind, message, read_at, created_at";
pub async fn f_1v8(
    r_25x: &PgPool,
    user_id: Uuid,
    submission_id: Option<Uuid>,
    kind: &str,
    message: &str,
) -> sqlx::Result<bool> {
    let r_25y = sqlx::query(
        "INSERT INTO notifications (id, user_id, submission_id, kind, message) \
         VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(user_id)
    .bind(submission_id)
    .bind(kind)
    .bind(message)
    .execute(r_25x)
    .await?;
    Ok(r_25y.rows_affected() > 0)
}
pub async fn f_1v9(
    r_25z: &PgPool,
    user_id: Uuid,
    info_request_id: Uuid,
    message: &str,
) -> sqlx::Result<bool> {
    let r_260 = sqlx::query(
        "INSERT INTO notifications (id, user_id, info_request_id, kind, message) \
         VALUES ($1, $2, $3, 'sla_info_overdue', $4) ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(user_id)
    .bind(info_request_id)
    .bind(message)
    .execute(r_25z)
    .await?;
    Ok(r_260.rows_affected() > 0)
}
pub async fn f_1vb(
    r_261: &PgPool,
    r_262: &crate::submissions::Tm8,
    message: &str,
) -> sqlx::Result<()> {
    if let Some(user_id) = r_262.created_by {
        f_1v8(r_261, user_id, Some(r_262.id), "submission_status", message).await?;
    }
    Ok(())
}
pub async fn f_1nk(r_264: &PgPool, user_id: Uuid, r_263: i64) -> sqlx::Result<Vec<Tec>> {
    sqlx::query_as::<_, Tec>(&format!(
        "SELECT {COLS} FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2"
    ))
    .bind(user_id)
    .bind(r_263)
    .fetch_all(r_264)
    .await
}
pub async fn f_2n9(r_265: &PgPool, user_id: Uuid) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT count(*) FROM notifications WHERE user_id = $1 AND read_at IS NULL")
        .bind(user_id)
        .fetch_one(r_265)
        .await
}
pub async fn f_1rg(r_266: &PgPool, user_id: Uuid, id: Uuid) -> sqlx::Result<bool> {
    let r_267 = sqlx::query(
        "UPDATE notifications SET read_at = now() \
         WHERE id = $1 AND user_id = $2 AND read_at IS NULL",
    )
    .bind(id)
    .bind(user_id)
    .execute(r_266)
    .await?;
    Ok(r_267.rows_affected() > 0)
}
pub async fn f_1r4(r_268: &PgPool, user_id: Uuid) -> sqlx::Result<u64> {
    let r_269 = sqlx::query(
        "UPDATE notifications SET read_at = now() WHERE user_id = $1 AND read_at IS NULL",
    )
    .bind(user_id)
    .execute(r_268)
    .await?;
    Ok(r_269.rows_affected())
}
pub async fn list_for_submission(r_26a: &PgPool, submission_id: Uuid) -> sqlx::Result<Vec<Tec>> {
    sqlx::query_as::<_, Tec>(&format!(
        "SELECT {COLS} FROM notifications WHERE submission_id = $1 ORDER BY created_at"
    ))
    .bind(submission_id)
    .fetch_all(r_26a)
    .await
}
type Tmc = (Uuid, Option<Uuid>, Vec<Uuid>, String, String, DateTime<Utc>);
fn f_2qc(r_26b: DateTime<Utc>) -> chrono::NaiveDate {
    r_26b
        .with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00"))
        .date_naive()
}
pub async fn f_2fx(
    r_26g: &PgPool,
    r_26e: &std::collections::BTreeSet<chrono::NaiveDate>,
) -> sqlx::Result<u64> {
    let mut r_26d = 0u64;
    let r_26c: Vec<Tmc> = sqlx::query_as(
            "SELECT s.id, s.created_by, \
                ARRAY(SELECT pa.admin_id FROM project_admins pa WHERE pa.project_id = s.project_id) \
                    AS admin_ids, \
                s.ma_yeu_cau, s.status, \
                CASE WHEN s.status = 'registered' THEN s.created_at ELSE s.received_at END \
         FROM submissions s \
         WHERE (s.status = 'registered' AND s.created_at < now() - interval '7 days') \
            OR (s.status = 'received' AND s.received_at < now() - interval '60 days')",
        )
        .fetch_all(r_26g)
        .await?;
    let r_26j = f_2qc(Utc::now());
    for (r_26i, created_by, admin_ids, ma_yeu_cau, status, since) in r_26c {
        let (kind, message) = if status == "registered" {
            if crate::workdays::f_2ri(f_2qc(since), r_26j, r_26e) <= 7 {
                continue;
            }
            (
                "sla_confirm_overdue",
                format!(
                    "TOG6p24gbuG7mXAg{ma_yeu_cau}OiBxdcOhIDA3IG5nw6B5IGzDoG0gdmnhu4djIGNoxrBhIHRy4bqjIGzhu51pIMSRxINuZyBrw70gKMSQaeG7gXUgMjEuNCk="
                ),
            )
        } else {
            (
                "sla_approve_overdue",
                format!(
                    "TOG6p24gbuG7mXAg{ma_yeu_cau}OiBxdcOhIDYwIG5nw6B5IGvhu4MgdOG7qyB0aeG6v3Agbmjhuq1uIGNoxrBhIHBow6ogZHV54buHdCAoxJBp4buBdSAyNS42KQ=="
                ),
            )
        };
        for user in created_by.into_iter().chain(admin_ids) {
            if f_1v8(r_26g, user, Some(r_26i), kind, &message).await? {
                r_26d += 1;
            }
        }
    }
    for (r_26f, r_26h, project_id, chu_de, since) in crate::info::f_1wq(r_26g).await? {
        if crate::workdays::f_2ri(f_2qc(since), r_26j, r_26e) <= 6 {
            continue;
        }
        let message = format!(
            "WcOqdSBj4bqndSBjdW5nIGPhuqVwIHRow7RuZyB0aW4gIg=={chu_de}IjogcXXDoSAwNiBuZ8OgeSBsw6BtIHZp4buHYyBjaMawYSB4w6l0IGR1eeG7h3QgKMSQaeG7gXUgNDYuNSk="
        );
        let mut r_26k: Vec<Uuid> = crate::projects::f_re(r_26g, project_id).await?;
        r_26k.extend(r_26h);
        for user in r_26k {
            if f_1v9(r_26g, user, r_26f, &message).await? {
                r_26d += 1;
            }
        }
    }
    Ok(r_26d)
}
