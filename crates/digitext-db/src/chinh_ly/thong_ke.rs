// Copyright (c) 2026 DigiText. All rights reserved.
//! Thống kê chỉnh lý (đọc thẳng cl_*): tổng kết chỉnh lý theo phòng ban, năng suất nhân viên (từ
//! cl_lich_su), tổng kết phần mềm theo phông, mượn trả.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row as _};
use uuid::Uuid;

fn rows_json(rows: Vec<sqlx::postgres::PgRow>) -> Vec<Value> {
    rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect()
}

/// Mỗi phòng ban của phông một dòng: hộp / hồ sơ vĩnh viễn, có thời hạn; bó / tập loại; hết hạn; trùng thừa; chưa đánh số.
pub async fn chinh_ly(pool: &PgPool, project: Uuid, phong: Option<Uuid>) -> sqlx::Result<Vec<Value>> {
    let rows = sqlx::query(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT pb.id AS phong_ban_id, pb.ten AS ten_phong_ban, pb.ma_don_vi, p.id AS phong_id, p.ten AS ten_phong, p.ma_du_an, \
                  count(DISTINCT h.hop_id) FILTER (WHERE t.trang_thai = 1) AS hop_vv, \
                  count(h.id) FILTER (WHERE t.trang_thai = 1) AS hs_vv, \
                  count(DISTINCT h.hop_id) FILTER (WHERE t.trang_thai = 2 AND t.gia_tri > 0) AS hop_co_thoi_han, \
                  count(h.id) FILTER (WHERE t.trang_thai = 2 AND t.gia_tri > 0) AS hs_co_thoi_han, \
                  count(DISTINCT h.bo_id) AS bo, count(DISTINCT h.tap_id) AS tap, \
                  count(h.id) FILTER (WHERE h.thbq_status = 'HET') AS hs_het, \
                  count(h.id) FILTER (WHERE h.trung_thua) AS hs_trung_thua, \
                  count(h.id) FILTER (WHERE h.ho_so_so IS NULL OR btrim(h.ho_so_so) = '') AS hs_chua_danh_so, \
                  count(h.id) AS hs_tong, \
                  count(DISTINCT h.hop_id) AS hop_tong, \
                  COALESCE(sum(h.so_van_ban), 0) AS vb_tong \
           FROM cl_phong_ban pb JOIN cl_phong p ON p.id = pb.phong_id \
           LEFT JOIN cl_ho_so_v h ON h.phong_ban_id = pb.id AND h.deleted_at IS NULL \
           LEFT JOIN cl_thbq t ON t.id = h.thbq_id \
           WHERE pb.project_id = $1 AND pb.deleted_at IS NULL AND p.deleted_at IS NULL AND ($2::uuid IS NULL OR p.id = $2) \
           GROUP BY pb.id, pb.ten, pb.ma_don_vi, pb.so_tt, p.id, p.ten, p.ma_du_an ORDER BY p.ten, pb.so_tt NULLS LAST, pb.ten) x",
    )
    .bind(project)
    .bind(phong)
    .fetch_all(pool)
    .await?;
    Ok(rows_json(rows))
}

/// Năng suất từng người từ lịch sử: số hồ sơ / văn bản đã chạm, trường thêm mới, trường sửa, ngày làm việc.
pub async fn nhan_vien(pool: &PgPool, project: Uuid, phong: Option<Uuid>, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>) -> sqlx::Result<Vec<Value>> {
    let rows = sqlx::query(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT COALESCE(l.actor_id::text, l.actor_name) AS actor_key, l.actor_id, COALESCE(u.display_name, l.actor_name) AS actor_name, u.username, \
                  count(DISTINCT l.ho_so_id) FILTER (WHERE l.entity = 'ho_so') AS so_ho_so, \
                  count(DISTINCT l.entity_id) FILTER (WHERE l.entity = 'van_ban') AS so_van_ban, \
                  count(*) FILTER (WHERE l.action = 'add') AS lan_them, count(*) FILTER (WHERE l.action = 'edit') AS lan_sua, \
                  COALESCE(sum(l.n_new), 0) AS truong_moi, COALESCE(sum(l.n_edited), 0) AS truong_sua, \
                  COALESCE(sum(l.n_new + l.n_edited), 0) AS truong_tong, \
                  count(DISTINCT (l.at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date) AS ngay_lam_viec, \
                  min(l.at) AS dau, max(l.at) AS cuoi \
           FROM cl_lich_su l LEFT JOIN users u ON u.id = l.actor_id \
           WHERE l.project_id = $1 AND ($2::uuid IS NULL OR l.phong_id = $2) AND ($3::timestamptz IS NULL OR l.at >= $3) AND ($4::timestamptz IS NULL OR l.at < $4) \
             AND l.action IN ('add', 'edit', 'import') \
           GROUP BY actor_key, l.actor_id, actor_name, u.username ORDER BY truong_tong DESC, actor_name) x",
    )
    .bind(project)
    .bind(phong)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    Ok(rows_json(rows))
}

/// Mỗi phông một dòng: thời gian khởi tạo, hồ sơ / văn bản tạo trong khoảng, hộp / hồ sơ VV và có thời hạn, bó / tập.
pub async fn phan_mem(pool: &PgPool, project: Uuid, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>) -> sqlx::Result<Vec<Value>> {
    let rows = sqlx::query(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT p.id AS phong_id, p.ma_du_an, p.ten AS ten_phong, p.created_at AS khoi_tao, \
                  count(h.id) AS hs_tao, COALESCE(sum(h.so_van_ban), 0) AS vb_tong, \
                  count(DISTINCT h.hop_id) FILTER (WHERE t.trang_thai = 1) AS hop_vv, count(h.id) FILTER (WHERE t.trang_thai = 1 AND h.ho_so_so IS NOT NULL) AS hs_vv, \
                  count(DISTINCT h.hop_id) FILTER (WHERE t.trang_thai = 2) AS hop_co_thoi_han, count(h.id) FILTER (WHERE t.trang_thai = 2 AND h.ho_so_so IS NOT NULL) AS hs_co_thoi_han, \
                  count(DISTINCT h.bo_id) AS bo, count(DISTINCT h.tap_id) AS tap, \
                  (SELECT count(*) FROM cl_phong_ban pb WHERE pb.phong_id = p.id AND pb.deleted_at IS NULL) AS so_phong_ban \
           FROM cl_phong p LEFT JOIN cl_ho_so_v h ON h.phong_id = p.id AND h.deleted_at IS NULL \
                AND ($2::timestamptz IS NULL OR h.created_at >= $2) AND ($3::timestamptz IS NULL OR h.created_at < $3) \
           LEFT JOIN cl_thbq t ON t.id = h.thbq_id \
           WHERE p.project_id = $1 AND p.deleted_at IS NULL \
           GROUP BY p.id, p.ma_du_an, p.ten, p.created_at, p.so_tt ORDER BY p.so_tt NULLS LAST, p.ten) x",
    )
    .bind(project)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    Ok(rows_json(rows))
}

/// Mượn trả: đang mượn, quá hạn, đã trả trong khoảng; kèm danh sách phiếu đang mượn.
pub async fn muon_tra(pool: &PgPool, project: Uuid, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>) -> sqlx::Result<Value> {
    let r = sqlx::query(
        "SELECT count(*) FILTER (WHERE trang_thai = 'dang_muon') AS dang_muon, \
                count(*) FILTER (WHERE trang_thai = 'dang_muon' AND ngay_du_kien_tra < CURRENT_DATE) AS qua_han, \
                count(*) FILTER (WHERE trang_thai = 'da_tra' AND ($2::timestamptz IS NULL OR updated_at >= $2) AND ($3::timestamptz IS NULL OR updated_at < $3)) AS da_tra, \
                count(*) FILTER (WHERE ($2::timestamptz IS NULL OR created_at >= $2) AND ($3::timestamptz IS NULL OR created_at < $3)) AS lap_phieu \
         FROM cl_muon_tra WHERE project_id = $1 AND deleted_at IS NULL",
    )
    .bind(project)
    .bind(from)
    .bind(to)
    .fetch_one(pool)
    .await?;
    Ok(serde_json::json!({
        "dangMuon": r.try_get::<i64, _>("dang_muon")?, "quaHan": r.try_get::<i64, _>("qua_han")?,
        "daTra": r.try_get::<i64, _>("da_tra")?, "lapPhieu": r.try_get::<i64, _>("lap_phieu")?,
    }))
}

/// Danh sách phiếu mượn (kèm hồ sơ) theo trạng thái: dang_muon | da_tra | qua_han | (tất cả).
pub async fn phieu_muon(pool: &PgPool, project: Uuid, trang_thai: Option<&str>, q: Option<&str>, limit: i64, offset: i64) -> sqlx::Result<(Vec<Value>, i64)> {
    let where_sql = "m.project_id = $1 AND m.deleted_at IS NULL \
        AND ($2::text IS NULL OR ($2 = 'qua_han' AND m.trang_thai = 'dang_muon' AND m.ngay_du_kien_tra < CURRENT_DATE) OR ($2 <> 'qua_han' AND m.trang_thai = $2)) \
        AND ($3::text IS NULL OR m.nguoi_muon ILIKE '%' || $3 || '%' OR m.don_vi ILIKE '%' || $3 || '%' OR h.tieu_de ILIKE '%' || $3 || '%' OR h.so_tam ILIKE '%' || $3 || '%' OR h.ma_ho_so ILIKE '%' || $3 || '%')";
    let rows = sqlx::query(&format!(
        "SELECT to_jsonb(m) || jsonb_build_object('so_tam', h.so_tam, 'ho_so_so', h.ho_so_so, 'ma_ho_so', h.ma_ho_so, 'tieu_de', h.tieu_de, 'hop_so', h.hop_so, \
            'ten_phong', h.ten_phong, 'ten_phong_ban', h.ten_phong_ban, 'qua_han', (m.trang_thai = 'dang_muon' AND m.ngay_du_kien_tra < CURRENT_DATE), \
            'nguoi_lap_ten', u.display_name) AS row \
         FROM cl_muon_tra m JOIN cl_ho_so_v h ON h.id = m.ho_so_id LEFT JOIN users u ON u.id = m.nguoi_lap_phieu \
         WHERE {where_sql} ORDER BY m.trang_thai = 'dang_muon' DESC, m.ngay_muon DESC, m.ver DESC LIMIT $4 OFFSET $5"
    ))
    .bind(project)
    .bind(trang_thai)
    .bind(q.map(str::trim).filter(|s| !s.is_empty()))
    .bind(limit.clamp(1, 10_000))
    .bind(offset.max(0))
    .fetch_all(pool)
    .await?;
    let total: i64 = sqlx::query(&format!("SELECT count(*) FROM cl_muon_tra m JOIN cl_ho_so_v h ON h.id = m.ho_so_id WHERE {where_sql}"))
        .bind(project)
        .bind(trang_thai)
        .bind(q.map(str::trim).filter(|s| !s.is_empty()))
        .fetch_one(pool)
        .await?
        .try_get(0)?;
    Ok((rows_json(rows), total))
}
