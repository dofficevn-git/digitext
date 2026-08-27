// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
pub mod cli;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use std::cmp::Ordering;
use std::collections::HashMap;
use uuid::Uuid;
pub fn f_2qu(r_2wd: i64, r_2we: i64, weight: u32) -> f64 {
    (r_2we as f64 - 2.0 * r_2wd as f64) / weight.max(1) as f64
}
#[derive(Debug, Clone, PartialEq)]
pub struct Thr {
    pub project_id: Uuid,
    pub done: i64,
    pub weight: u32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Th2 {
    pub job_id: Uuid,
    pub project_id: Uuid,
    pub deadline: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}
fn f_105(r_2wf: Option<NaiveDate>, r_2wg: Option<NaiveDate>) -> Ordering {
    match (r_2wf, r_2wg) {
        (Some(r_2wh), Some(r_2wi)) => r_2wh.cmp(&r_2wi),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}
pub fn f_1bq(r_2wp: &[Th2], r_2wq: &[Thr]) -> Vec<Uuid> {
    let r_2wr: i64 = r_2wq.iter().map(|s| s.done).sum();
    let r_2wm: HashMap<Uuid, f64> = r_2wq
        .iter()
        .map(|s| (s.project_id, f_2qu(s.done, r_2wr, s.weight)))
        .collect();
    let mut r_2wo: Vec<&Th2> = r_2wp.iter().collect();
    r_2wo.sort_by(|r_2wj, r_2wk| {
        let r_2wl = r_2wm.get(&r_2wj.project_id).copied().unwrap_or(0.0);
        let r_2wn = r_2wm.get(&r_2wk.project_id).copied().unwrap_or(0.0);
        r_2wn
            .partial_cmp(&r_2wl)
            .unwrap_or(Ordering::Equal)
            .then_with(|| f_105(r_2wj.deadline, r_2wk.deadline))
            .then_with(|| r_2wj.created_at.cmp(&r_2wk.created_at))
            .then_with(|| r_2wj.job_id.cmp(&r_2wk.job_id))
    });
    r_2wo.into_iter().map(|j| j.job_id).collect()
}
pub async fn f_248(r_2ws: &PgPool) -> sqlx::Result<Vec<Thr>> {
    let r_2wt: Vec<(Uuid, i64)> = sqlx::query_as(
        "SELECT d.project_id, count(*) FILTER (WHERE j.status = 'done') \
         FROM ocr_jobs j JOIN documents d ON j.document_id = d.id \
         GROUP BY d.project_id \
         HAVING count(*) FILTER (WHERE j.status = 'queued') > 0",
    )
    .fetch_all(r_2ws)
    .await?;
    Ok(r_2wt
        .into_iter()
        .map(|(project_id, done)| Thr {
            project_id,
            done,
            weight: 1,
        })
        .collect())
}
pub async fn reprioritize(r_2wy: &PgPool) -> sqlx::Result<u64> {
    let r_2x2 = f_248(r_2wy).await?;
    if r_2x2.is_empty() {
        return Ok(0);
    }
    let r_2x3: i64 = r_2x2.iter().map(|s| s.done).sum();
    let mut r_2x1: Vec<&Thr> = r_2x2.iter().collect();
    r_2x1.sort_by(|r_2wu, r_2wv| {
        let r_2ww = f_2qu(r_2wu.done, r_2x3, r_2wu.weight);
        let r_2wx = f_2qu(r_2wv.done, r_2x3, r_2wv.weight);
        r_2wx
            .partial_cmp(&r_2ww)
            .unwrap_or(Ordering::Equal)
            .then_with(|| r_2wu.project_id.cmp(&r_2wv.project_id))
    });
    for (r_2x0, s) in r_2x1.iter().enumerate() {
        let r_2wz = ((r_2x0 as i32) + 1).min(10);
        sqlx::query(
            "UPDATE ocr_jobs SET priority = $1, updated_at = now() \
             WHERE status = 'queued' AND priority > 0 \
               AND document_id IN (SELECT id FROM documents WHERE project_id = $2)",
        )
        .bind(r_2wz)
        .bind(s.project_id)
        .execute(r_2wy)
        .await?;
    }
    Ok(r_2x1.len() as u64)
}
