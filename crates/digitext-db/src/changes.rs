// Copyright (c) 2026 DigiText. All rights reserved.
use sqlx::{PgPool, Row};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changes {
    pub seq: i64,
    pub topics: Vec<String>,
    pub full: bool,
}
pub const C_F6: f64 = 2.0;
pub const C_I8: f64 = 3600.0;
pub async fn since(r_21m: &PgPool, since: Option<i64>, r_21l: f64) -> sqlx::Result<Changes> {
    let r_21n = sqlx::query(
            "SELECT \
           coalesce(pg_sequence_last_value(pg_get_serial_sequence('change_log', 'seq')), 0)::bigint AS seq, \
           (SELECT min(seq) FROM change_log) AS min_seq, \
           coalesce(( \
             SELECT array_agg(DISTINCT topic) FROM change_log \
             WHERE $1::bigint IS NOT NULL \
               AND (seq > $1::bigint OR at > now() - make_interval(secs => $2::float8)) \
           ), ARRAY[]::text[]) AS topics",
        )
        .bind(since)
        .bind(r_21l)
        .fetch_one(r_21m)
        .await?;
    let seq: i64 = r_21n.try_get("seq")?;
    let min_seq: Option<i64> = r_21n.try_get("min_seq")?;
    let mut topics: Vec<String> = r_21n.try_get("topics")?;
    topics.sort();
    let r_21k = min_seq.unwrap_or(seq.saturating_add(1));
    let full = match since {
        Some(r_21o) => r_21o.saturating_add(1) < r_21k,
        None => false,
    };
    Ok(Changes { seq, topics, full })
}
pub async fn prune(r_21q: &PgPool, r_21p: f64) -> sqlx::Result<u64> {
    let r_21r =
        sqlx::query("DELETE FROM change_log WHERE at < now() - make_interval(secs => $1::float8)")
            .bind(r_21p)
            .execute(r_21q)
            .await?;
    Ok(r_21r.rows_affected())
}
pub async fn note(r_21s: &PgPool, topic: &str) -> sqlx::Result<i64> {
    let r_21t = sqlx::query("INSERT INTO change_log (topic) VALUES ($1) RETURNING seq")
        .bind(topic)
        .fetch_one(r_21s)
        .await?;
    r_21t.try_get("seq")
}
