// Copyright (c) 2026 DigiText. All rights reserved.
//! PHÂN QUYỀN THEO PHÔNG (0038, 27/11/2026 — đợt 4 đối chiếu Digifile, nhóm 4C). Bảng `user_phong (user_id, phong_id, project_id)`:
//! không có dòng cho (người, dự án) = không giới hạn (mọi tài khoản hôm nay); có dòng = chỉ thấy các phông đó. super_admin / manager
//! không bao giờ bị giới hạn (tầng route quyết, module này chỉ đọc / ghi).
use crate::chinh_ly::{rows, Row};
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

/// Phông mà một dòng `cl_*` thuộc về (đợt 5, 27/11/2026 — bịt lỗ `/cl/rows`, push đồng bộ): `cl_phong` → chính nó; `cl_phong_ban`,
/// `cl_ho_so` → `phong_id`; `cl_mang` / `cl_hop` / `cl_bo_tap` / `cl_gia` → qua phòng ban; `cl_van_ban` / `cl_muon_tra` → qua hồ sơ;
/// `cl_van_ban_file` → qua văn bản. Bảng khác (khối phông, THBQ, ngôn ngữ, loại văn bản, kho) hoặc thiếu khoá → `None` = không ràng phông.
/// `row` là dòng ĐANG ghi (op mới) hoặc dòng hiện có; khoá vắng trong `row` thì tra dòng hiện có theo `id`.
pub async fn phong_cua_dong(pool: &PgPool, table: &str, id: Uuid, row: &Row) -> sqlx::Result<Option<Uuid>> {
    let khoa = |k: &str| rows::uuid_of(row, k);
    Ok(match table {
        "cl_phong" => Some(id),
        "cl_phong_ban" | "cl_ho_so" => match khoa("phong_id") {
            Some(p) => Some(p),
            None => sqlx::query_scalar(&format!("SELECT phong_id FROM {table} WHERE id = $1")).bind(id).fetch_optional(pool).await?,
        },
        "cl_mang" | "cl_hop" | "cl_bo_tap" | "cl_gia" => {
            let pb: Option<Uuid> = match khoa("phong_ban_id") {
                Some(p) => Some(p),
                None => sqlx::query_scalar(&format!("SELECT phong_ban_id FROM {table} WHERE id = $1")).bind(id).fetch_optional(pool).await?.flatten(),
            };
            match pb {
                Some(pb) => sqlx::query_scalar("SELECT phong_id FROM cl_phong_ban WHERE id = $1").bind(pb).fetch_optional(pool).await?,
                None => None,
            }
        }
        "cl_van_ban" | "cl_muon_tra" => {
            let hs: Option<Uuid> = match khoa("ho_so_id") {
                Some(h) => Some(h),
                None => sqlx::query_scalar(&format!("SELECT ho_so_id FROM {table} WHERE id = $1")).bind(id).fetch_optional(pool).await?,
            };
            match hs {
                Some(h) => sqlx::query_scalar("SELECT phong_id FROM cl_ho_so WHERE id = $1").bind(h).fetch_optional(pool).await?,
                None => None,
            }
        }
        "cl_van_ban_file" => {
            let vb: Option<Uuid> = match khoa("van_ban_id") {
                Some(v) => Some(v),
                None => sqlx::query_scalar("SELECT van_ban_id FROM cl_van_ban_file WHERE id = $1").bind(id).fetch_optional(pool).await?,
            };
            match vb {
                Some(v) => {
                    sqlx::query_scalar("SELECT h.phong_id FROM cl_van_ban v JOIN cl_ho_so h ON h.id = v.ho_so_id WHERE v.id = $1").bind(v).fetch_optional(pool).await?
                }
                None => None,
            }
        }
        _ => None,
    })
}

/// Phông của từng tài liệu đã nối mục lục (`cl_van_ban_file` → `cl_van_ban` → `cl_ho_so.phong_id`): (document_id, phong_id).
/// Tài liệu chưa nối mục lục không có trong kết quả → tầng route coi là không ràng phông (kho tệp thi công theo dự án).
pub async fn phong_cua_tai_lieu(pool: &PgPool, document_ids: &[Uuid]) -> sqlx::Result<Vec<(Uuid, Uuid)>> {
    if document_ids.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as(
        "SELECT DISTINCT f.document_id, h.phong_id FROM cl_van_ban_file f JOIN cl_van_ban v ON v.id = f.van_ban_id JOIN cl_ho_so h ON h.id = v.ho_so_id          WHERE f.document_id = ANY($1) AND f.deleted_at IS NULL",
    )
    .bind(document_ids)
    .fetch_all(pool)
    .await
}

/// Mọi tài liệu trong danh sách có nối mục lục đều thuộc phạm vi `scope` không (None = không giới hạn).
pub async fn tai_lieu_trong_pham_vi(pool: &PgPool, scope: Option<&[Uuid]>, document_ids: &[Uuid]) -> sqlx::Result<bool> {
    let Some(sc) = scope else { return Ok(true) };
    Ok(phong_cua_tai_lieu(pool, document_ids).await?.iter().all(|(_, p)| sc.contains(p)))
}
