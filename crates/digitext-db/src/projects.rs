// Copyright (c) 2026 DigiText. All rights reserved.
use crate::users::User;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Project {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
pub async fn create(r_2af: &PgPool, code: &str, name: &str) -> sqlx::Result<Project> {
    f_12a(r_2af, code, name, None).await
}
pub async fn f_12a(
    r_2ag: &PgPool,
    code: &str,
    name: &str,
    admin_id: Option<Uuid>,
) -> sqlx::Result<Project> {
    let mut r_2ai = r_2ag.begin().await?;
    let r_2ah = sqlx::query_as::<_, Project>(
        "INSERT INTO projects (id, code, name) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(Uuid::now_v7())
    .bind(code)
    .bind(name)
    .fetch_one(&mut *r_2ai)
    .await?;
    if let Some(admin_id) = admin_id {
        sqlx::query(
            "INSERT INTO project_admins (project_id, admin_id, granted_by) VALUES ($1, $2, $2)",
        )
        .bind(r_2ah.id)
        .bind(admin_id)
        .execute(&mut *r_2ai)
        .await?;
    }
    r_2ai.commit().await?;
    Ok(r_2ah)
}
pub async fn f_1ng(r_2aj: &PgPool, admin_id: Uuid) -> sqlx::Result<Vec<Project>> {
    sqlx::query_as::<_, Project>(
        "SELECT p.* FROM projects p \
         JOIN project_admins pa ON pa.project_id = p.id \
         WHERE pa.admin_id = $1 ORDER BY p.code",
    )
    .bind(admin_id)
    .fetch_all(r_2aj)
    .await
}
pub async fn f_re(r_2ak: &PgPool, project_id: Uuid) -> sqlx::Result<Vec<Uuid>> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT admin_id FROM project_admins WHERE project_id = $1 ORDER BY granted_at, admin_id",
    )
    .bind(project_id)
    .fetch_all(r_2ak)
    .await
}
pub async fn f_rd(r_2al: &PgPool, r_2am: &[Uuid]) -> sqlx::Result<Vec<(Uuid, Uuid)>> {
    sqlx::query_as::<_, (Uuid, Uuid)>(
        "SELECT project_id, admin_id FROM project_admins \
         WHERE project_id = ANY($1) ORDER BY granted_at, admin_id",
    )
    .bind(r_2am)
    .fetch_all(r_2al)
    .await
}
pub async fn f_2dy(
    r_2ao: &PgPool,
    project_id: Uuid,
    r_2an: &[Uuid],
    granted_by: Option<Uuid>,
) -> sqlx::Result<()> {
    let mut r_2ap = r_2ao.begin().await?;
    sqlx::query("DELETE FROM project_admins WHERE project_id = $1 AND admin_id <> ALL($2)")
        .bind(project_id)
        .bind(r_2an)
        .execute(&mut *r_2ap)
        .await?;
    for admin_id in r_2an {
        sqlx::query(
            "INSERT INTO project_admins (project_id, admin_id, granted_by) VALUES ($1, $2, $3) \
             ON CONFLICT (project_id, admin_id) DO NOTHING",
        )
        .bind(project_id)
        .bind(admin_id)
        .bind(granted_by)
        .execute(&mut *r_2ap)
        .await?;
    }
    r_2ap.commit().await?;
    Ok(())
}
pub async fn get(r_2aq: &PgPool, id: Uuid) -> sqlx::Result<Option<Project>> {
    sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
        .bind(id)
        .fetch_optional(r_2aq)
        .await
}
pub async fn f_1ef(r_2ar: &PgPool, code: &str) -> sqlx::Result<Option<Project>> {
    sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE code = $1")
        .bind(code)
        .fetch_optional(r_2ar)
        .await
}
pub async fn list(r_2as: &PgPool) -> sqlx::Result<Vec<Project>> {
    sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY code")
        .fetch_all(r_2as)
        .await
}
pub async fn f_1n7(r_2au: &PgPool, r_2at: &[Uuid]) -> sqlx::Result<Vec<Project>> {
    sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ANY($1) ORDER BY code")
        .bind(r_2at)
        .fetch_all(r_2au)
        .await
}
pub async fn update(
    r_2av: &PgPool,
    id: Uuid,
    name: Option<&str>,
    is_active: Option<bool>,
) -> sqlx::Result<Option<Project>> {
    sqlx::query_as::<_, Project>(
        "UPDATE projects SET \
            name      = COALESCE($2, name), \
            is_active = COALESCE($3, is_active) \
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(name)
    .bind(is_active)
    .fetch_optional(r_2av)
    .await
}
pub async fn f_qm(
    r_2aw: &PgPool,
    project_id: Uuid,
    admin_id: Uuid,
    granted_by: Option<Uuid>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO project_admins (project_id, admin_id, granted_by) VALUES ($1, $2, $3) \
         ON CONFLICT (project_id, admin_id) DO NOTHING",
    )
    .bind(project_id)
    .bind(admin_id)
    .bind(granted_by)
    .execute(r_2aw)
    .await?;
    Ok(())
}
pub async fn f_275(r_2ax: &PgPool, project_id: Uuid, admin_id: Uuid) -> sqlx::Result<bool> {
    let r_2ay = sqlx::query("DELETE FROM project_admins WHERE project_id = $1 AND admin_id = $2")
        .bind(project_id)
        .bind(admin_id)
        .execute(r_2ax)
        .await?;
    Ok(r_2ay.rows_affected() > 0)
}
pub async fn add_member(
    r_2az: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
    added_by: Option<Uuid>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO project_members (project_id, user_id, added_by) VALUES ($1, $2, $3) \
         ON CONFLICT (project_id, user_id) DO NOTHING",
    )
    .bind(project_id)
    .bind(user_id)
    .bind(added_by)
    .execute(r_2az)
    .await?;
    Ok(())
}
pub async fn list_members(r_2b0: &PgPool, project_id: Uuid) -> sqlx::Result<Vec<User>> {
    sqlx::query_as::<_, User>(
        "SELECT u.id, u.username, u.email, u.display_name, u.role, u.project_id, \
                u.is_active, u.token_version, u.created_by, u.created_at, u.updated_at \
         FROM project_members pm JOIN users u ON u.id = pm.user_id \
         WHERE pm.project_id = $1 ORDER BY u.username",
    )
    .bind(project_id)
    .fetch_all(r_2b0)
    .await
}
