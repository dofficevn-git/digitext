// Copyright (c) 2026 DigiText. All rights reserved.
//! PHÂN QUYỀN THEO PHÔNG (0038, 27/11/2026 — đợt 4 đối chiếu Digifile, nhóm 4C). Bảng `user_phong (user_id, phong_id, project_id)`:
//! không có dòng cho (người, dự án) = không giới hạn (mọi tài khoản hôm nay); có dòng = chỉ thấy các phông đó. super_admin / manager
//! không bao giờ bị giới hạn (tầng route quyết, module này chỉ đọc / ghi).
use sqlx::PgPool;
use uuid::Uuid;

/// Danh sách phông được cấp của một người trong một dự án; rỗng = không giới hạn.
pub async fn cua(pool: &PgPool, user: Uuid, project: Uuid) -> sqlx::Result<Vec<Uuid>> {
    sqlx::query_scalar("SELECT phong_id FROM user_phong WHERE user_id = $1 AND project_id = $2 ORDER BY added_at")
        .bind(user)
        .bind(project)
        .fetch_all(pool)
        .await
}

/// Mọi dự án của một người có giới hạn phông: (project_id, phong_id).
pub async fn tat_ca(pool: &PgPool, user: Uuid) -> sqlx::Result<Vec<(Uuid, Uuid)>> {
    sqlx::query_as("SELECT project_id, phong_id FROM user_phong WHERE user_id = $1 ORDER BY project_id, added_at").bind(user).fetch_all(pool).await
}

/// Đặt lại danh sách phông của (người, dự án): rỗng = bỏ giới hạn. Phông phải thuộc dự án (phông lạ bị bỏ qua). Trả số phông đã ghi.
pub async fn dat(pool: &PgPool, user: Uuid, project: Uuid, phong_ids: &[Uuid], by: Uuid) -> sqlx::Result<usize> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM user_phong WHERE user_id = $1 AND project_id = $2").bind(user).bind(project).execute(&mut *tx).await?;
    let hop_le: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM cl_phong WHERE id = ANY($1) AND project_id = $2 AND deleted_at IS NULL")
        .bind(phong_ids)
        .bind(project)
        .fetch_all(&mut *tx)
        .await?;
    for p in &hop_le {
        sqlx::query("INSERT INTO user_phong (user_id, phong_id, project_id, added_by) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING")
            .bind(user)
            .bind(p)
            .bind(project)
            .bind(by)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(hop_le.len())
}

/// Tên phông (mã — tên) cho hiển thị chip.
pub async fn ten_phong(pool: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<(Uuid, String)>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as("SELECT id, ma_du_an || ' — ' || ten FROM cl_phong WHERE id = ANY($1) ORDER BY so_tt NULLS LAST, ten").bind(ids).fetch_all(pool).await
}
