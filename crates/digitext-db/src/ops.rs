// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
pub type T5a = (Uuid, Uuid, String, Option<i32>);
pub async fn f_17z(r_28u: &PgPool, r_28t: i64) -> sqlx::Result<Vec<T5a>> {
    sqlx::query_as(
        "SELECT d.id, d.project_id, d.filename, d.page_count \
         FROM documents d \
         LEFT JOIN LATERAL (SELECT max(checked_at) AS last FROM integrity_checks c \
                            WHERE c.document_id = d.id) c ON true \
         WHERE d.status <> 'disposed' \
           AND (c.last IS NULL OR c.last < now() - interval '3 years') \
         ORDER BY c.last ASC NULLS FIRST, d.uploaded_at ASC LIMIT $1",
    )
    .bind(r_28t)
    .fetch_all(r_28u)
    .await
}
pub async fn f_26a(
    r_28v: &PgPool,
    project_id: Uuid,
    document_id: Uuid,
    ket_qua: &str,
    chi_tiet: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO integrity_checks (id, project_id, document_id, ket_qua, chi_tiet) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(project_id)
    .bind(document_id)
    .bind(ket_qua)
    .bind(chi_tiet)
    .execute(r_28v)
    .await?;
    Ok(())
}
pub async fn stats(r_28w: &PgPool, r_28x: Option<&[Uuid]>) -> sqlx::Result<(i64, i64, i64)> {
    sqlx::query_as(
        "SELECT count(*) FILTER (WHERE true), \
                count(*) FILTER (WHERE c.last IS NOT NULL), \
                count(*) FILTER (WHERE c.ket_qua = 'loi') \
         FROM documents d \
         LEFT JOIN LATERAL ( \
             SELECT max(checked_at) AS last, \
                    (array_agg(ket_qua ORDER BY checked_at DESC))[1] AS ket_qua \
             FROM integrity_checks ic \
             WHERE ic.document_id = d.id AND ic.checked_at > now() - interval '3 years' \
         ) c ON true \
         WHERE d.status <> 'disposed' \
           AND ($1::uuid[] IS NULL OR d.project_id = ANY($1))",
    )
    .bind(r_28x)
    .fetch_one(r_28w)
    .await
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T7q {
    pub document_id: Uuid,
    pub filename: String,
    pub ho_so_ref: Option<String>,
    pub chi_tiet: Option<String>,
    pub checked_at: DateTime<Utc>,
}
pub async fn f_1bp(r_28z: &PgPool, r_290: Option<&[Uuid]>, r_28y: i64) -> sqlx::Result<Vec<T7q>> {
    sqlx::query_as::<_, T7q>(
        "SELECT DISTINCT ON (c.document_id) \
                c.document_id, d.filename, d.ho_so_ref, c.chi_tiet, c.checked_at \
         FROM integrity_checks c JOIN documents d ON d.id = c.document_id \
         WHERE ($1::uuid[] IS NULL OR c.project_id = ANY($1)) \
         ORDER BY c.document_id, c.checked_at DESC \
         LIMIT $2",
    )
    .bind(r_290)
    .bind(r_28y)
    .fetch_all(r_28z)
    .await
    .map(|r_292| {
        r_292
            .into_iter()
            .filter(|r_291| r_291.chi_tiet.is_some())
            .collect()
    })
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct T13 {
    pub id: Uuid,
    pub loai: String,
    pub nguon: String,
    pub phuong_tien: Option<String>,
    pub ket_qua: String,
    pub chi_tiet: Option<String>,
    pub reported_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
pub struct Tdr<'a> {
    pub loai: &'a str,
    pub nguon: &'a str,
    pub phuong_tien: Option<&'a str>,
    pub ket_qua: &'a str,
    pub chi_tiet: Option<&'a str>,
    pub reported_by: Option<Uuid>,
}
pub async fn f_269(r_294: &PgPool, r_293: Tdr<'_>) -> sqlx::Result<T13> {
    sqlx::query_as::<_, T13>(
        "INSERT INTO backup_runs (id, loai, nguon, phuong_tien, ket_qua, chi_tiet, reported_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         RETURNING id, loai, nguon, phuong_tien, ket_qua, chi_tiet, reported_by, created_at",
    )
    .bind(Uuid::now_v7())
    .bind(r_293.loai)
    .bind(r_293.nguon)
    .bind(r_293.phuong_tien)
    .bind(r_293.ket_qua)
    .bind(r_293.chi_tiet)
    .bind(r_293.reported_by)
    .fetch_one(r_294)
    .await
}
pub async fn backups(r_296: &PgPool, r_295: i64) -> sqlx::Result<Vec<T13>> {
    sqlx::query_as::<_, T13>(
        "SELECT id, loai, nguon, phuong_tien, ket_qua, chi_tiet, reported_by, created_at \
         FROM backup_runs ORDER BY created_at DESC LIMIT $1",
    )
    .bind(r_295)
    .fetch_all(r_296)
    .await
}
pub async fn f_1me(r_297: &PgPool) -> sqlx::Result<Option<DateTime<Utc>>> {
    sqlx::query_scalar("SELECT max(created_at) FROM backup_runs WHERE ket_qua = 'dat'")
        .fetch_one(r_297)
        .await
}
pub async fn f_ti(r_299: &PgPool, hours: i64) -> sqlx::Result<bool> {
    let r_298: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM notifications \
         WHERE kind = 'backup_overdue' AND created_at > now() - ($1 || ' hours')::interval",
    )
    .bind(hours.to_string())
    .fetch_one(r_299)
    .await?;
    Ok(r_298 > 0)
}
pub async fn f_rk(r_29a: &PgPool) -> sqlx::Result<Vec<Uuid>> {
    sqlx::query_scalar("SELECT DISTINCT admin_id FROM project_admins")
        .fetch_all(r_29a)
        .await
}
