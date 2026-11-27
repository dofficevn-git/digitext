// Copyright (c) 2026 DigiText. All rights reserved.
// Bảng mã TT05 còn thiếu (0040, Digifile B03): chế độ sử dụng, mức độ tin cậy, tình trạng vật lý, tình trạng dự phòng.
// Dòng dùng chung có project_id NULL (seed trong migration); dự án thêm mã riêng. Cột trên hồ sơ / văn bản lưu TÊN, nên đổi tên
// mã không đụng dữ liệu cũ.
use sqlx::PgPool;
use uuid::Uuid;

pub const NHOM: &[&str] = &["che_do_su_dung", "muc_do_tin_cay", "tinh_trang_vat_ly", "tinh_trang_du_phong"];

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaRow {
    pub id: Uuid,
    pub project_id: Option<Uuid>,
    pub nhom: String,
    pub ma: String,
    pub ten: String,
    pub so_tt: Option<i32>,
    pub ghi_chu: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Moi {
    pub nhom: String,
    pub ma: String,
    pub ten: String,
    #[serde(default)]
    pub so_tt: Option<i32>,
    #[serde(default)]
    pub ghi_chu: Option<String>,
}

pub fn nhom_hop_le(n: &str) -> bool {
    NHOM.contains(&n)
}

/// Mã dùng chung + mã riêng của dự án, theo nhóm (None = mọi nhóm).
pub async fn list(pool: &PgPool, project_id: Uuid, nhom: Option<&str>) -> sqlx::Result<Vec<MaRow>> {
    sqlx::query_as::<_, MaRow>(
        "SELECT id, project_id, nhom, ma, ten, so_tt, ghi_chu FROM cl_ma_tt05 \
         WHERE deleted_at IS NULL AND (project_id IS NULL OR project_id = $1) AND ($2::text IS NULL OR nhom = $2) \
         ORDER BY nhom, COALESCE(so_tt, 999), ma",
    )
    .bind(project_id)
    .bind(nhom)
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<MaRow>> {
    sqlx::query_as::<_, MaRow>("SELECT id, project_id, nhom, ma, ten, so_tt, ghi_chu FROM cl_ma_tt05 WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Thêm mã riêng cho dự án (`dung_chung` = true → dòng dùng chung, chỉ super admin).
pub async fn them(pool: &PgPool, project_id: Option<Uuid>, m: &Moi, boi: Uuid) -> sqlx::Result<MaRow> {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO cl_ma_tt05 (id, project_id, nhom, ma, ten, so_tt, ghi_chu, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(id)
    .bind(project_id)
    .bind(&m.nhom)
    .bind(m.ma.trim())
    .bind(m.ten.trim())
    .bind(m.so_tt)
    .bind(m.ghi_chu.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(boi)
    .execute(pool)
    .await?;
    Ok(get(pool, id).await?.expect("vừa chèn"))
}

pub async fn sua(pool: &PgPool, id: Uuid, m: &Moi) -> sqlx::Result<Option<MaRow>> {
    let n = sqlx::query(
        "UPDATE cl_ma_tt05 SET ma = $2, ten = $3, so_tt = $4, ghi_chu = $5, updated_at = now() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(m.ma.trim())
    .bind(m.ten.trim())
    .bind(m.so_tt)
    .bind(m.ghi_chu.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 {
        return Ok(None);
    }
    get(pool, id).await
}

pub async fn xoa(pool: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let n = sqlx::query("UPDATE cl_ma_tt05 SET deleted_at = now(), updated_at = now() WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}
