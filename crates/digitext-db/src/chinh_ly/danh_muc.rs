// Copyright (c) 2026 DigiText. All rights reserved.
//! Danh mục chỉnh lý: đọc gộp mọi bảng mã của một dự án (kèm dòng dùng chung).

use serde_json::Value;
use sqlx::{PgPool, Row as _};
use uuid::Uuid;

/// Các bảng danh mục trả về trong `GET /cl/catalog`.
pub const CATALOG_TABLES: &[&str] = &["cl_khoi_phong", "cl_phong", "cl_phong_ban", "cl_mang", "cl_thbq", "cl_ngon_ngu", "cl_loai_van_ban", "cl_kho", "cl_gia", "cl_hop", "cl_bo_tap"];

/// Mọi dòng sống của các bảng danh mục, theo dự án (bảng mã dùng chung có project_id NULL).
pub async fn catalog(pool: &PgPool, project_id: Uuid) -> sqlx::Result<serde_json::Map<String, Value>> {
    let mut out = serde_json::Map::new();
    for t in CATALOG_TABLES {
        let order = match *t {
            "cl_thbq" | "cl_ngon_ngu" | "cl_loai_van_ban" => "so_tt NULLS LAST, ten",
            "cl_hop" => "phong_ban_id, hop_so_int NULLS LAST, hop_so COLLATE vi_so",
            "cl_bo_tap" => "phong_ban_id, loai, so",
            "cl_gia" => "kho_id, so_tt NULLS LAST, tu_hop NULLS LAST, ten",
            _ => "so_tt NULLS LAST, ten",
        };
        let rows = sqlx::query(&format!(
            "SELECT to_jsonb(t) AS row FROM {t} t WHERE (t.project_id = $1 OR t.project_id IS NULL) AND t.deleted_at IS NULL ORDER BY {order}"
        ))
        .bind(project_id)
        .fetch_all(pool)
        .await?;
        let key = t.trim_start_matches("cl_").to_owned();
        out.insert(key, Value::Array(rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect()));
    }
    Ok(out)
}
