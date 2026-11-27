// Copyright (c) 2026 DigiText. All rights reserved.
//! KHO / GIÁ có sức chứa + gán hộp cho hồ sơ nhập gói (0035, 27/11/2026 — đợt 3 đối chiếu Digifile TT05, nhóm 4B).
//! Số hộp / hồ sơ đang trên giá tính từ `cl_ho_so_v.gia_eff_id` (giá gán tay hoặc suy từ dải hộp) — một lượt gom theo dự án rồi
//! nối với `cl_gia`, không dùng view để điều kiện dự án đẩy được xuống chỉ mục.
use serde_json::Value;
use sqlx::{PgPool, Row as _};
use uuid::Uuid;

fn rows_json(rows: Vec<sqlx::postgres::PgRow>) -> Vec<Value> {
    rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect()
}

/// Mỗi giá một dòng: kho, phòng ban / phông, dải hộp, `suc_chua` (cột hoặc suy từ dải), số hộp / hồ sơ đang chứa, `pct_day`,
/// `gan_day` (≥ `canh_bao_pct`), `vuot` (> 100 %).
pub async fn tong_hop(pool: &PgPool, project: Uuid, kho: Option<Uuid>, phong: Option<Uuid>, phong_ids: Option<&[Uuid]>) -> sqlx::Result<Vec<Value>> {
    let rows = sqlx::query(
        "WITH hs AS (SELECT v.gia_eff_id, count(DISTINCT v.hop_id) AS so_hop, count(*) AS so_ho_so \
                     FROM cl_ho_so_v v WHERE v.project_id = $1 AND v.deleted_at IS NULL AND v.gia_eff_id IS NOT NULL GROUP BY v.gia_eff_id) \
         SELECT to_jsonb(x) AS row FROM ( \
           SELECT g.id, g.kho_id, k.ten AS ten_kho, k.suc_chua_hop AS kho_suc_chua, g.phong_ban_id, pb.ten AS ten_phong_ban, p.id AS phong_id, p.ten AS ten_phong, p.ma_du_an, \
                  g.ten, g.tu_hop, g.den_hop, g.vinh_vien, g.so_tt, g.canh_bao_pct, g.suc_chua_hop, \
                  COALESCE(g.suc_chua_hop, CASE WHEN g.tu_hop IS NOT NULL AND g.den_hop IS NOT NULL THEN g.den_hop - g.tu_hop + 1 END) AS suc_chua, \
                  COALESCE(hs.so_hop, 0) AS so_hop, COALESCE(hs.so_ho_so, 0) AS so_ho_so, \
                  CASE WHEN COALESCE(g.suc_chua_hop, g.den_hop - g.tu_hop + 1) > 0 \
                       THEN round(100.0 * COALESCE(hs.so_hop, 0) / COALESCE(g.suc_chua_hop, g.den_hop - g.tu_hop + 1), 1) END AS pct_day, \
                  (COALESCE(g.suc_chua_hop, g.den_hop - g.tu_hop + 1) > 0 AND 100.0 * COALESCE(hs.so_hop, 0) / COALESCE(g.suc_chua_hop, g.den_hop - g.tu_hop + 1) >= g.canh_bao_pct) AS gan_day, \
                  (COALESCE(g.suc_chua_hop, g.den_hop - g.tu_hop + 1) > 0 AND COALESCE(hs.so_hop, 0) > COALESCE(g.suc_chua_hop, g.den_hop - g.tu_hop + 1)) AS vuot \
           FROM cl_gia g JOIN cl_kho k ON k.id = g.kho_id \
           LEFT JOIN cl_phong_ban pb ON pb.id = g.phong_ban_id LEFT JOIN cl_phong p ON p.id = pb.phong_id \
           LEFT JOIN hs ON hs.gia_eff_id = g.id \
           WHERE g.project_id = $1 AND g.deleted_at IS NULL AND k.deleted_at IS NULL AND ($2::uuid IS NULL OR g.kho_id = $2) AND ($3::uuid IS NULL OR p.id = $3) \
             AND ($4::uuid[] IS NULL OR p.id = ANY($4)) \
           ORDER BY k.so_tt NULLS LAST, k.ten, g.so_tt NULLS LAST, g.tu_hop NULLS LAST, g.ten) x",
    )
    .bind(project)
    .bind(kho)
    .bind(phong)
    .bind(phong_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows_json(rows))
}

/// Giá gần đầy / vượt sức chứa của một dự án (cho nhắc `kho_day`): (id giá, tên kho, tên giá, số hộp, sức chứa, %).
pub async fn gia_gan_day(pool: &PgPool, project: Uuid) -> sqlx::Result<Vec<(Uuid, String, String, i64, i64, f64)>> {
    let rows = tong_hop(pool, project, None, None, None).await?;
    Ok(rows
        .into_iter()
        .filter(|r| r["gan_day"].as_bool() == Some(true))
        .filter_map(|r| {
            Some((
                r["id"].as_str().and_then(|s| Uuid::parse_str(s).ok())?,
                r["ten_kho"].as_str().unwrap_or("").to_string(),
                r["ten"].as_str().unwrap_or("").to_string(),
                r["so_hop"].as_i64().unwrap_or(0),
                r["suc_chua"].as_i64().unwrap_or(0),
                r["pct_day"].as_f64().unwrap_or(0.0),
            ))
        })
        .collect())
}

/// Đặt phòng ban cho hồ sơ chưa có (hồ sơ nhập từ gói AIP) trước khi gán hộp; hồ sơ đã có phòng ban khác → lỗi theo dòng.
/// Trả (số hồ sơ vừa đặt, lỗi [(ho_so_id, lý do)]).
pub async fn dat_phong_ban(pool: &PgPool, project: Uuid, phong_ban: Uuid, ho_so_ids: &[Uuid], by: Uuid) -> sqlx::Result<(u64, Vec<(Uuid, String)>)> {
    let mut loi = Vec::new();
    let pb_phong: Option<Uuid> = sqlx::query_scalar("SELECT phong_id FROM cl_phong_ban WHERE id = $1 AND project_id = $2 AND deleted_at IS NULL")
        .bind(phong_ban)
        .bind(project)
        .fetch_optional(pool)
        .await?;
    let Some(pb_phong) = pb_phong else {
        return Ok((0, ho_so_ids.iter().map(|id| (*id, "Phòng ban không có trong dự án".to_string())).collect()));
    };
    let rows = sqlx::query("SELECT id, phong_id, phong_ban_id FROM cl_ho_so WHERE id = ANY($1) AND project_id = $2 AND deleted_at IS NULL")
        .bind(ho_so_ids)
        .bind(project)
        .fetch_all(pool)
        .await?;
    let mut can_dat: Vec<Uuid> = Vec::new();
    for id in ho_so_ids {
        match rows.iter().find(|r| r.try_get::<Uuid, _>("id").ok() == Some(*id)) {
            None => loi.push((*id, "Không thấy hồ sơ trong dự án".into())),
            Some(r) => {
                let phong: Uuid = r.try_get("phong_id")?;
                let pb: Option<Uuid> = r.try_get("phong_ban_id")?;
                if phong != pb_phong {
                    loi.push((*id, "Phòng ban không thuộc phông của hồ sơ".into()));
                } else if pb.is_some() && pb != Some(phong_ban) {
                    loi.push((*id, "Hồ sơ đã thuộc phòng ban khác".into()));
                } else if pb.is_none() {
                    can_dat.push(*id);
                }
            }
        }
    }
    let n = if can_dat.is_empty() {
        0
    } else {
        sqlx::query("UPDATE cl_ho_so SET phong_ban_id = $2, updated_by = $3, updated_at = now(), ver = nextval('cl_ver_seq') WHERE id = ANY($1) AND phong_ban_id IS NULL")
            .bind(&can_dat)
            .bind(phong_ban)
            .bind(by)
            .execute(pool)
            .await?
            .rows_affected()
    };
    Ok((n, loi))
}

/// Hồ sơ chưa gán hộp của một phông (ưu tiên hồ sơ nhập từ gói), để màn "Gán hộp" liệt kê.
pub async fn ho_so_chua_hop(pool: &PgPool, project: Uuid, phong: Uuid, limit: i64) -> sqlx::Result<Vec<Value>> {
    let rows = sqlx::query(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT h.id, h.so_tam, h.ma_ho_so, h.tieu_de, h.ngay_ket_thuc, h.ten_thbq, h.vinh_vien, h.phong_ban_id, h.ten_phong_ban, g.ma_ho_so_goc, g.objid \
           FROM cl_ho_so_v h LEFT JOIN cl_ho_so_goi g ON g.ho_so_id = h.id \
           WHERE h.project_id = $1 AND h.phong_id = $2 AND h.deleted_at IS NULL AND h.hop_id IS NULL \
           ORDER BY g.ho_so_id IS NULL, h.ngay_kt_sort NULLS LAST, h.so_tam COLLATE vi_so LIMIT $3) x",
    )
    .bind(project)
    .bind(phong)
    .bind(limit.clamp(1, 20_000))
    .fetch_all(pool)
    .await?;
    Ok(rows_json(rows))
}

/// Nhắc giá gần đầy / vượt sức chứa cho quản trị từng dự án đang hoạt động — một thông báo một giá mỗi tuần ISO (ref). Trả số đã gửi.
pub async fn nhac_kho_day(pool: &PgPool, hom_nay: chrono::NaiveDate) -> sqlx::Result<u64> {
    use chrono::Datelike;
    let tuan = format!("{}-W{:02}", hom_nay.iso_week().year(), hom_nay.iso_week().week());
    let projects: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM projects WHERE is_active ORDER BY created_at").fetch_all(pool).await?;
    let mut n = 0u64;
    for p in projects {
        let gia = gia_gan_day(pool, p).await?;
        if gia.is_empty() {
            continue;
        }
        let admins = crate::projects::f_re(pool, p).await?;
        for (id, kho, ten, so_hop, suc_chua, pct) in gia {
            let msg = format!("Giá \"{ten}\" (kho {kho}) đang chứa {so_hop}/{suc_chua} hộp ({pct:.0} %) — gần đầy, cần bố trí giá mới");
            let ref_ = format!("{id}:{tuan}");
            for u in &admins {
                if crate::notifications::f_1v8_ref(pool, *u, "kho_day", &msg, &ref_).await? {
                    n += 1;
                }
            }
        }
    }
    Ok(n)
}
