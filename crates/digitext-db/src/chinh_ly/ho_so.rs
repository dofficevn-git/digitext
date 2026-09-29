// Copyright (c) 2026 DigiText. All rights reserved.
//! Truy vấn hồ sơ chỉnh lý (view cl_ho_so_v): danh sách có bộ lọc như bản cũ (phông, phòng ban,
//! mảng, THBQ, VV/GL/HET, trùng thừa, tìm chữ, kho/giá, đã có hộp / đã đánh số), chi tiết kèm văn bản.

use serde_json::Value;
use sqlx::{PgPool, Row as _};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub project_id: Uuid,
    pub phong_id: Option<Uuid>,
    pub phong_ban_id: Option<Uuid>,
    pub mang_id: Option<Uuid>,
    /// Danh sách THBQ; rỗng = không lọc.
    pub thbq_ids: Vec<Uuid>,
    /// Chỉ THBQ loại "có thời hạn" (trang_thai = 2) — tương ứng lựa chọn "0" của bản cũ.
    pub thbq_co_thoi_han: bool,
    /// VV | GL | HET
    pub status: Option<String>,
    pub trung_thua: Option<bool>,
    pub q: Option<String>,
    pub kho_id: Option<Uuid>,
    pub gia_id: Option<Uuid>,
    pub has_hop: Option<bool>,
    pub has_ho_so_so: Option<bool>,
    pub nam: Option<i32>,
    pub hop_id: Option<Uuid>,
    pub ids: Vec<Uuid>,
    pub include_deleted: bool,
    /// so_tam | ho_so_so | ngay_kt | mang | thbq | updated
    pub sort: Option<String>,
    pub desc: bool,
    pub limit: i64,
    pub offset: i64,
}

pub const LIST_MAX: i64 = 50_000;

fn order_by(sort: Option<&str>, desc: bool) -> String {
    let dir = if desc { "DESC" } else { "ASC" };
    match sort.unwrap_or("so_tam") {
        "ho_so_so" => format!("v.hop_so_int {dir} NULLS LAST, v.hop_so COLLATE vi_so {dir}, v.ho_so_so COLLATE vi_so {dir} NULLS LAST, v.so_tam COLLATE vi_so"),
        "ngay_kt" => format!("v.ngay_kt_sort {dir} NULLS LAST, v.so_tam COLLATE vi_so"),
        "mang" => format!("v.mang_so_tt {dir} NULLS LAST, v.ten_mang {dir}, v.ngay_kt_sort NULLS LAST, v.so_tam COLLATE vi_so"),
        "thbq" => format!("v.thbq_so_tt {dir} NULLS LAST, v.mang_so_tt NULLS LAST, v.ngay_kt_sort NULLS LAST, v.so_tam COLLATE vi_so"),
        "updated" => format!("v.updated_at {dir}, v.so_tam COLLATE vi_so"),
        _ => format!("v.so_tam COLLATE vi_so {dir}, v.id"),
    }
}

const WHERE: &str = "v.project_id = $1 AND ($2::bool OR v.deleted_at IS NULL) \
    AND ($3::uuid IS NULL OR v.phong_id = $3) AND ($4::uuid IS NULL OR v.phong_ban_id = $4) AND ($5::uuid IS NULL OR v.mang_id = $5) \
    AND (cardinality($6::uuid[]) = 0 OR v.thbq_id = ANY($6)) AND (NOT $7::bool OR v.thbq_trang_thai = 2) \
    AND ($8::text IS NULL OR v.thbq_status = $8) AND ($9::bool IS NULL OR v.trung_thua = $9) \
    AND ($10::text IS NULL OR v.tieu_de ILIKE '%' || $10 || '%' OR v.so_tam ILIKE '%' || $10 || '%' OR v.ho_so_so ILIKE '%' || $10 || '%' \
         OR v.ma_ho_so ILIKE '%' || $10 || '%' OR v.ghi_chu ILIKE '%' || $10 || '%') \
    AND ($11::uuid IS NULL OR v.kho_eff_id = $11) AND ($12::uuid IS NULL OR v.gia_eff_id = $12) \
    AND ($13::bool IS NULL OR (v.hop_id IS NOT NULL) = $13) \
    AND ($14::bool IS NULL OR (v.ho_so_so IS NOT NULL AND btrim(v.ho_so_so) <> '') = $14) \
    AND ($15::int IS NULL OR v.nam_ket_thuc = $15) AND ($16::uuid IS NULL OR v.hop_id = $16) \
    AND (cardinality($17::uuid[]) = 0 OR v.id = ANY($17))";

fn bind<'q>(q: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>, f: &'q Filter) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    q.bind(f.project_id)
        .bind(f.include_deleted)
        .bind(f.phong_id)
        .bind(f.phong_ban_id)
        .bind(f.mang_id)
        .bind(&f.thbq_ids)
        .bind(f.thbq_co_thoi_han)
        .bind(f.status.as_deref())
        .bind(f.trung_thua)
        .bind(f.q.as_deref().map(str::trim).filter(|s| !s.is_empty()))
        .bind(f.kho_id)
        .bind(f.gia_id)
        .bind(f.has_hop)
        .bind(f.has_ho_so_so)
        .bind(f.nam)
        .bind(f.hop_id)
        .bind(&f.ids)
}

/// Danh sách hồ sơ (dòng JSON của view) + tổng số khớp bộ lọc.
pub async fn list(pool: &PgPool, f: &Filter) -> sqlx::Result<(Vec<Value>, i64)> {
    let sql = format!("SELECT to_jsonb(v) AS row FROM cl_ho_so_v v WHERE {WHERE} ORDER BY {} LIMIT $18 OFFSET $19", order_by(f.sort.as_deref(), f.desc));
    let rows = bind(sqlx::query(&sql), f).bind(f.limit.clamp(1, LIST_MAX)).bind(f.offset.max(0)).fetch_all(pool).await?;
    let total: i64 = bind(sqlx::query(&format!("SELECT count(*) FROM cl_ho_so_v v WHERE {WHERE}")), f).fetch_one(pool).await?.try_get(0)?;
    Ok((rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect(), total))
}

/// Một hồ sơ (view) hoặc None.
pub async fn get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Value>> {
    let r = sqlx::query("SELECT to_jsonb(v) AS row FROM cl_ho_so_v v WHERE v.id = $1").bind(id).fetch_optional(pool).await?;
    Ok(r.and_then(|r| r.try_get::<Value, _>("row").ok()))
}

/// Văn bản của hồ sơ (kèm mảng `files` và mã TT05), theo số thứ tự.
pub async fn van_ban_of(pool: &PgPool, ho_so_id: Uuid, include_deleted: bool) -> sqlx::Result<Vec<Value>> {
    let rows = sqlx::query(
        "SELECT to_jsonb(v) || jsonb_build_object( \
            'files', COALESCE((SELECT jsonb_agg(to_jsonb(f) ORDER BY f.so_tt NULLS LAST, f.name) FROM cl_van_ban_file f WHERE f.van_ban_id = v.id AND f.deleted_at IS NULL), '[]'::jsonb), \
            'ma_van_ban', CASE WHEN h.ma_ho_so IS NOT NULL THEN h.ma_ho_so || '.' || lpad(v.so_thu_tu::text, 7, '0') END, \
            'ten_loai_vb', (SELECT l.ten FROM cl_loai_van_ban l WHERE l.ma = v.loai_vb AND (l.project_id = v.project_id OR l.project_id IS NULL) AND l.deleted_at IS NULL ORDER BY l.project_id NULLS LAST LIMIT 1)) AS row \
         FROM cl_van_ban v JOIN cl_ho_so_v h ON h.id = v.ho_so_id \
         WHERE v.ho_so_id = $1 AND ($2::bool OR v.deleted_at IS NULL) ORDER BY v.so_thu_tu, v.ver",
    )
    .bind(ho_so_id)
    .bind(include_deleted)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect())
}

/// Phiếu mượn đang mở của hồ sơ.
pub async fn open_loan(pool: &PgPool, ho_so_id: Uuid) -> sqlx::Result<Option<Value>> {
    let r = sqlx::query("SELECT to_jsonb(m) AS row FROM cl_muon_tra m WHERE m.ho_so_id = $1 AND m.trang_thai = 'dang_muon' AND m.deleted_at IS NULL LIMIT 1")
        .bind(ho_so_id)
        .fetch_optional(pool)
        .await?;
    Ok(r.and_then(|r| r.try_get::<Value, _>("row").ok()))
}

/// Số thứ tự văn bản tiếp theo trong hồ sơ.
pub async fn next_stt(pool: &PgPool, ho_so_id: Uuid) -> sqlx::Result<i32> {
    let n: Option<i32> = sqlx::query_scalar("SELECT max(so_thu_tu) FROM cl_van_ban WHERE ho_so_id = $1 AND deleted_at IS NULL").bind(ho_so_id).fetch_one(pool).await?;
    Ok(n.unwrap_or(0) + 1)
}

/// Tìm văn bản chéo hồ sơ (khai thác).
#[derive(Debug, Clone, Default)]
pub struct VanBanFilter {
    pub project_id: Uuid,
    pub phong_id: Option<Uuid>,
    pub phong_ban_id: Option<Uuid>,
    pub ho_so_id: Option<Uuid>,
    pub q: Option<String>,
    pub loai_vb: Option<String>,
    pub co_quan: Option<String>,
    pub tu_ngay: Option<chrono::NaiveDate>,
    pub den_ngay: Option<chrono::NaiveDate>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn search_van_ban(pool: &PgPool, f: &VanBanFilter) -> sqlx::Result<(Vec<Value>, i64)> {
    let where_sql = "v.project_id = $1 AND v.deleted_at IS NULL AND h.deleted_at IS NULL \
        AND ($2::uuid IS NULL OR h.phong_id = $2) AND ($3::uuid IS NULL OR h.phong_ban_id = $3) AND ($4::uuid IS NULL OR v.ho_so_id = $4) \
        AND ($5::text IS NULL OR v.trich_yeu ILIKE '%' || $5 || '%' OR v.so_ky_hieu ILIKE '%' || $5 || '%' OR v.so_van_ban ILIKE '%' || $5 || '%' OR v.tu_khoa ILIKE '%' || $5 || '%') \
        AND ($6::text IS NULL OR v.loai_vb = $6) AND ($7::text IS NULL OR v.co_quan_ban_hanh ILIKE '%' || $7 || '%') \
        AND ($8::date IS NULL OR v.ngay_thang_sort >= $8) AND ($9::date IS NULL OR v.ngay_thang_sort <= $9)";
    let sql = format!(
        "SELECT to_jsonb(v) || jsonb_build_object('so_tam', h.so_tam, 'ho_so_so', h.ho_so_so, 'hop_so', h.hop_so, 'tieu_de_ho_so', h.tieu_de, \
            'ma_ho_so', h.ma_ho_so, 'ten_phong', h.ten_phong, 'ten_phong_ban', h.ten_phong_ban, \
            'ma_van_ban', CASE WHEN h.ma_ho_so IS NOT NULL THEN h.ma_ho_so || '.' || lpad(v.so_thu_tu::text, 7, '0') END) AS row \
         FROM cl_van_ban v JOIN cl_ho_so_v h ON h.id = v.ho_so_id WHERE {where_sql} \
         ORDER BY h.hop_so_int NULLS LAST, h.ho_so_so COLLATE vi_so NULLS LAST, h.so_tam COLLATE vi_so, v.so_thu_tu LIMIT $10 OFFSET $11"
    );
    let count_sql = format!("SELECT count(*) FROM cl_van_ban v JOIN cl_ho_so_v h ON h.id = v.ho_so_id WHERE {where_sql}");
    fn bind<'q>(q: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>, f: &'q VanBanFilter) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        q.bind(f.project_id)
            .bind(f.phong_id)
            .bind(f.phong_ban_id)
            .bind(f.ho_so_id)
            .bind(f.q.as_deref().map(str::trim).filter(|s| !s.is_empty()))
            .bind(f.loai_vb.as_deref())
            .bind(f.co_quan.as_deref().map(str::trim).filter(|s| !s.is_empty()))
            .bind(f.tu_ngay)
            .bind(f.den_ngay)
    }
    let rows = bind(sqlx::query(&sql), f).bind(f.limit.clamp(1, LIST_MAX)).bind(f.offset.max(0)).fetch_all(pool).await?;
    let total: i64 = bind(sqlx::query(&count_sql), f).fetch_one(pool).await?.try_get(0)?;
    Ok((rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect(), total))
}
