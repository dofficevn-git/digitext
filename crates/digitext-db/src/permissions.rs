// Copyright (c) 2026 DigiText. All rights reserved.
use digitext_core::{Permission, PermissionOverrides};
use sqlx::PgPool;
use uuid::Uuid;
pub async fn f_1o5(r_2a4: &PgPool, user_id: Uuid) -> sqlx::Result<PermissionOverrides> {
    let r_2a5 = sqlx::query_as::<_, (String, String)>(
        "SELECT permission, effect FROM user_permission_overrides WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(r_2a4)
    .await?;
    let mut r_2a1 = PermissionOverrides::default();
    for (r_2a3, effect) in r_2a5 {
        let Ok(r_2a2) = r_2a3.parse::<Permission>() else {
            continue;
        };
        match effect.as_str() {
            "grant" => {
                r_2a1.grants.insert(r_2a2);
            }
            "deny" => {
                r_2a1.denies.insert(r_2a2);
            }
            _ => {}
        }
    }
    Ok(r_2a1.sanitized())
}
pub async fn f_2e7(
    r_2a8: &PgPool,
    user_id: Uuid,
    r_2a7: &PermissionOverrides,
    granted_by: Option<Uuid>,
) -> sqlx::Result<()> {
    let r_2a6 = r_2a7.clone().sanitized();
    let mut r_2a9 = r_2a8.begin().await?;
    sqlx::query("DELETE FROM user_permission_overrides WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *r_2a9)
        .await?;
    for p in &r_2a6.grants {
        f_1if(&mut r_2a9, user_id, *p, "grant", granted_by).await?;
    }
    for p in &r_2a6.denies {
        f_1if(&mut r_2a9, user_id, *p, "deny", granted_by).await?;
    }
    r_2a9.commit().await?;
    Ok(())
}
async fn f_1if(
    r_2ab: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    r_2aa: Permission,
    effect: &str,
    granted_by: Option<Uuid>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO user_permission_overrides (user_id, permission, effect, granted_by) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(r_2aa.as_str())
    .bind(effect)
    .bind(granted_by)
    .execute(&mut **r_2ab)
    .await?;
    Ok(())
}
