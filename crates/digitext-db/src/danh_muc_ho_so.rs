// Copyright (c) 2026 DigiText. All rights reserved.
//! Danh mục hồ sơ dự kiến của cơ quan — Điều 19.4 TT 05/2025/TT-BNV, migration 0031 (26/11/2026).
//!
//! Một danh mục cho mỗi (phông, năm) — chốt 26/11: mỗi phòng ban soạn phần mình, lưu trữ cơ quan trình, lãnh đạo duyệt và
//! ban hành một bản chung. Có phiên bản: "Phiên bản mới" chép dòng (`tien_than_id`), bản cũ → `het_hieu_luc` khi bản mới ban
//! hành. `ma_dong` (số ký hiệu hồ sơ dự kiến) giữ nguyên qua các phiên bản để đối chiếu "theo / ngoài danh mục".
//!
//!   du_thao ──trình──► cho_duyet ──duyệt──► da_duyet ──ban hành (số QĐ, ngày)──► ban_hanh ──phiên bản mới──► v+1 du_thao
//!      ▲                   │ trả lại                 │ trả lại                        └─► (bản cũ) het_hieu_luc
//!      └───────────────────┴──────────────────────────┘
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row as _};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DanhMucRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub phong_id: Uuid,
    pub nam: i32,
    pub phien_ban: i32,
    pub trang_thai: String,
    pub so_quyet_dinh: Option<String>,
    pub ngay_ban_hanh: Option<NaiveDate>,
    pub trinh_boi: Option<Uuid>,
    pub trinh_luc: Option<DateTime<Utc>>,
    pub duyet_boi: Option<Uuid>,
    pub duyet_luc: Option<DateTime<Utc>>,
    pub ban_hanh_boi: Option<Uuid>,
    pub ban_hanh_luc: Option<DateTime<Utc>>,
    pub ly_do_tra: Option<String>,
    pub ghi_chu: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

const COLS: &str = "id, project_id, phong_id, nam, phien_ban, trang_thai, so_quyet_dinh, ngay_ban_hanh, trinh_boi, trinh_luc, duyet_boi, duyet_luc, \
     ban_hanh_boi, ban_hanh_luc, ly_do_tra, ghi_chu, created_by, created_at, updated_at";

const DM_JSON: &str = "SELECT d.*, p.ma_du_an AS ma_phong, p.ten AS ten_phong, \
        (SELECT count(*) FROM lt_danh_muc_dong x WHERE x.danh_muc_id = d.id) AS so_dong, \
        (SELECT count(DISTINCT x.phong_ban_id) FROM lt_danh_muc_dong x WHERE x.danh_muc_id = d.id) AS so_phong_ban, \
        ut.display_name AS trinh_ten, ud.display_name AS duyet_ten, ub.display_name AS ban_hanh_ten \
     FROM lt_danh_muc d JOIN cl_phong p ON p.id = d.phong_id \
     LEFT JOIN users ut ON ut.id = d.trinh_boi LEFT JOIN users ud ON ud.id = d.duyet_boi LEFT JOIN users ub ON ub.id = d.ban_hanh_boi";

pub async fn get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<DanhMucRow>> {
    sqlx::query_as::<_, DanhMucRow>(&format!("SELECT {COLS} FROM lt_danh_muc WHERE id = $1")).bind(id).fetch_optional(pool).await
}

pub async fn get_json(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Value>> {
    let r = sqlx::query(&format!("SELECT to_jsonb(x) AS row FROM ({DM_JSON}) x WHERE x.id = $1")).bind(id).fetch_optional(pool).await?;
    Ok(r.and_then(|r| r.try_get::<Value, _>("row").ok()))
}

/// Các phiên bản danh mục của dự án (lọc phông / năm), mới nhất trước.
pub async fn list(pool: &PgPool, project: Uuid, phong: Option<Uuid>, nam: Option<i32>) -> sqlx::Result<Vec<Value>> {
    let rows = sqlx::query(&format!(
        "SELECT to_jsonb(x) AS row FROM ({DM_JSON}) x WHERE x.project_id = $1 AND ($2::uuid IS NULL OR x.phong_id = $2) AND ($3::int IS NULL OR x.nam = $3) \
         ORDER BY x.nam DESC, x.phong_id, x.phien_ban DESC"
    ))
    .bind(project)
    .bind(phong)
    .bind(nam)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect())
}

/// Dòng của một danh mục, kèm tên phòng ban / mảng / THBQ và số hồ sơ đã gắn vào dòng.
pub async fn dong_cua(pool: &PgPool, id: Uuid, phong_ban: Option<Uuid>) -> sqlx::Result<Vec<Value>> {
    let rows = sqlx::query(
        "SELECT to_jsonb(x) || jsonb_build_object('ten_phong_ban', pb.ten, 'ma_don_vi', pb.ma_don_vi, 'ten_mang', m.ten, 'ten_thbq', t.ten, \
                'so_ho_so', (SELECT count(*) FROM cl_ho_so h WHERE h.dm_dong_id = x.id AND h.deleted_at IS NULL)) AS row \
         FROM lt_danh_muc_dong x LEFT JOIN cl_phong_ban pb ON pb.id = x.phong_ban_id LEFT JOIN cl_mang m ON m.id = x.mang_id LEFT JOIN cl_thbq t ON t.id = x.thbq_id \
         WHERE x.danh_muc_id = $1 AND ($2::uuid IS NULL OR x.phong_ban_id = $2) \
         ORDER BY pb.so_tt NULLS LAST, pb.ten, x.so_tt NULLS LAST, x.ma_dong COLLATE vi_so",
    )
    .bind(id)
    .bind(phong_ban)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect())
}

/// Dòng của bản ĐANG HIỆU LỰC (ban hành) của (phông, năm) — ô "Dòng danh mục" trong biểu mẫu hồ sơ.
pub async fn dong_hieu_luc(pool: &PgPool, phong: Uuid, nam: i32, phong_ban: Option<Uuid>) -> sqlx::Result<Vec<Value>> {
    let rows = sqlx::query(
        "SELECT jsonb_build_object('id', x.id, 'maDong', x.ma_dong, 'tieuDe', x.tieu_de, 'phongBanId', x.phong_ban_id, 'mangId', x.mang_id, 'thbqId', x.thbq_id) AS row \
         FROM lt_danh_muc_dong x JOIN lt_danh_muc d ON d.id = x.danh_muc_id \
         WHERE d.phong_id = $1 AND d.nam = $2 AND d.trang_thai = 'ban_hanh' AND ($3::uuid IS NULL OR x.phong_ban_id = $3) \
         ORDER BY x.so_tt NULLS LAST, x.ma_dong COLLATE vi_so",
    )
    .bind(phong)
    .bind(nam)
    .bind(phong_ban)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect())
}

/// Tạo danh mục dự thảo (phiên bản kế tiếp của (phông, năm)); `sao_chep_tu` = chép dòng từ một danh mục khác (thường bản ban
/// hành năm trước). Từ chối khi đang có bản chưa ban hành của cùng (phông, năm).
pub async fn tao(pool: &PgPool, project: Uuid, phong: Uuid, nam: i32, user: Uuid, sao_chep_tu: Option<Uuid>) -> sqlx::Result<Result<DanhMucRow, String>> {
    let mut tx = pool.begin().await?;
    // SQL khác chữ với records::create / nop_luu (xem chú thích ở nop_luu::lap về cache prepared statement của sqlx).
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('danh_muc:' || $1::text || ':' || $2::text, 0))")
        .bind(phong.to_string())
        .bind(nam.to_string())
        .execute(&mut *tx)
        .await?;
    let dang_mo: i64 = sqlx::query_scalar("SELECT count(*) FROM lt_danh_muc WHERE phong_id = $1 AND nam = $2 AND trang_thai IN ('du_thao', 'cho_duyet', 'da_duyet')")
        .bind(phong)
        .bind(nam)
        .fetch_one(&mut *tx)
        .await?;
    if dang_mo > 0 {
        return Ok(Err("Đang có một bản danh mục của năm này chưa ban hành — sửa tiếp bản đó".into()));
    }
    let pb: i32 = sqlx::query_scalar("SELECT COALESCE(max(phien_ban), 0) + 1 FROM lt_danh_muc WHERE phong_id = $1 AND nam = $2").bind(phong).bind(nam).fetch_one(&mut *tx).await?;
    let id = Uuid::now_v7();
    let row = sqlx::query_as::<_, DanhMucRow>(&format!("INSERT INTO lt_danh_muc (id, project_id, phong_id, nam, phien_ban, created_by) VALUES ($1, $2, $3, $4, $5, $6) RETURNING {COLS}"))
        .bind(id)
        .bind(project)
        .bind(phong)
        .bind(nam)
        .bind(pb)
        .bind(user)
        .fetch_one(&mut *tx)
        .await?;
    if let Some(nguon) = sao_chep_tu {
        chep_dong(&mut tx, nguon, id, project, user).await?;
    }
    tx.commit().await?;
    Ok(Ok(row))
}

async fn chep_dong(tx: &mut sqlx::PgConnection, tu: Uuid, den: Uuid, project: Uuid, user: Uuid) -> sqlx::Result<u64> {
    let n = sqlx::query(
        "INSERT INTO lt_danh_muc_dong (id, project_id, danh_muc_id, phong_ban_id, mang_id, ma_dong, tieu_de, thbq_id, nguoi_lap, ghi_chu, so_tt, tien_than_id, created_by) \
         SELECT gen_random_uuid(), $3, $2, phong_ban_id, mang_id, ma_dong, tieu_de, thbq_id, nguoi_lap, ghi_chu, so_tt, id, $4 \
         FROM lt_danh_muc_dong WHERE danh_muc_id = $1 ORDER BY so_tt NULLS LAST, ma_dong",
    )
    .bind(tu)
    .bind(den)
    .bind(project)
    .bind(user)
    .execute(tx)
    .await?
    .rows_affected();
    Ok(n)
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DongMoi {
    pub phong_ban_id: Uuid,
    #[serde(default)]
    pub mang_id: Option<Uuid>,
    pub ma_dong: String,
    pub tieu_de: String,
    #[serde(default)]
    pub thbq_id: Option<Uuid>,
    #[serde(default)]
    pub nguoi_lap: Option<String>,
    #[serde(default)]
    pub ghi_chu: Option<String>,
    #[serde(default)]
    pub so_tt: Option<i32>,
}

fn kiem_dong(d: &DongMoi) -> Result<(), String> {
    if d.ma_dong.trim().is_empty() {
        return Err("Thiếu số ký hiệu hồ sơ dự kiến".into());
    }
    if d.tieu_de.trim().is_empty() {
        return Err("Thiếu tiêu đề hồ sơ dự kiến".into());
    }
    Ok(())
}

/// Danh mục còn sửa được không, và dòng có thuộc phạm vi phòng ban của người gọi không (`None` = mọi phòng).
async fn cho_sua(tx: &mut sqlx::PgConnection, danh_muc_id: Uuid, phong_ban: Option<Uuid>, dong_pb: Uuid) -> sqlx::Result<Result<(), String>> {
    let tt: Option<String> = sqlx::query_scalar("SELECT trang_thai FROM lt_danh_muc WHERE id = $1 FOR UPDATE").bind(danh_muc_id).fetch_optional(tx).await?;
    match tt.as_deref() {
        None => return Ok(Err("Không có danh mục".into())),
        Some("du_thao") => {}
        Some(_) => return Ok(Err("Danh mục đã trình, không sửa dòng được (trả lại để sửa)".into())),
    }
    if phong_ban.is_some_and(|pb| pb != dong_pb) {
        return Ok(Err("Chỉ sửa được dòng của phòng ban mình".into()));
    }
    Ok(Ok(()))
}

pub async fn them_dong(pool: &PgPool, danh_muc_id: Uuid, project: Uuid, user: Uuid, phong_ban: Option<Uuid>, d: &DongMoi) -> sqlx::Result<Result<Uuid, String>> {
    if let Err(e) = kiem_dong(d) {
        return Ok(Err(e));
    }
    let mut tx = pool.begin().await?;
    if let Err(e) = cho_sua(&mut tx, danh_muc_id, phong_ban, d.phong_ban_id).await? {
        return Ok(Err(e));
    }
    let id = Uuid::now_v7();
    let r = sqlx::query(
        "INSERT INTO lt_danh_muc_dong (id, project_id, danh_muc_id, phong_ban_id, mang_id, ma_dong, tieu_de, thbq_id, nguoi_lap, ghi_chu, so_tt, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(id)
    .bind(project)
    .bind(danh_muc_id)
    .bind(d.phong_ban_id)
    .bind(d.mang_id)
    .bind(d.ma_dong.trim())
    .bind(d.tieu_de.trim())
    .bind(d.thbq_id)
    .bind(d.nguoi_lap.as_deref().map(str::trim))
    .bind(d.ghi_chu.as_deref().map(str::trim))
    .bind(d.so_tt)
    .bind(user)
    .execute(&mut *tx)
    .await;
    match r {
        Ok(_) => {}
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") => return Ok(Err(format!("Số ký hiệu {} đã có trong danh mục", d.ma_dong.trim()))),
        Err(e) => return Err(e),
    }
    sqlx::query("UPDATE lt_danh_muc SET updated_at = now() WHERE id = $1").bind(danh_muc_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Ok(id))
}

pub async fn sua_dong(pool: &PgPool, danh_muc_id: Uuid, dong_id: Uuid, phong_ban: Option<Uuid>, d: &DongMoi) -> sqlx::Result<Result<(), String>> {
    if let Err(e) = kiem_dong(d) {
        return Ok(Err(e));
    }
    let mut tx = pool.begin().await?;
    let Some(pb_cu): Option<Uuid> = sqlx::query_scalar("SELECT phong_ban_id FROM lt_danh_muc_dong WHERE id = $1 AND danh_muc_id = $2").bind(dong_id).bind(danh_muc_id).fetch_optional(&mut *tx).await? else {
        return Ok(Err("Không có dòng".into()));
    };
    if let Err(e) = cho_sua(&mut tx, danh_muc_id, phong_ban, pb_cu).await? {
        return Ok(Err(e));
    }
    if phong_ban.is_some_and(|pb| pb != d.phong_ban_id) {
        return Ok(Err("Không chuyển dòng sang phòng ban khác được".into()));
    }
    let r = sqlx::query(
        "UPDATE lt_danh_muc_dong SET phong_ban_id = $3, mang_id = $4, ma_dong = $5, tieu_de = $6, thbq_id = $7, nguoi_lap = $8, ghi_chu = $9, so_tt = $10, updated_at = now() \
         WHERE id = $1 AND danh_muc_id = $2",
    )
    .bind(dong_id)
    .bind(danh_muc_id)
    .bind(d.phong_ban_id)
    .bind(d.mang_id)
    .bind(d.ma_dong.trim())
    .bind(d.tieu_de.trim())
    .bind(d.thbq_id)
    .bind(d.nguoi_lap.as_deref().map(str::trim))
    .bind(d.ghi_chu.as_deref().map(str::trim))
    .bind(d.so_tt)
    .execute(&mut *tx)
    .await;
    match r {
        Ok(_) => {}
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") => return Ok(Err(format!("Số ký hiệu {} đã có trong danh mục", d.ma_dong.trim()))),
        Err(e) => return Err(e),
    }
    sqlx::query("UPDATE lt_danh_muc SET updated_at = now() WHERE id = $1").bind(danh_muc_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Ok(()))
}

pub async fn xoa_dong(pool: &PgPool, danh_muc_id: Uuid, dong_id: Uuid, phong_ban: Option<Uuid>) -> sqlx::Result<Result<(), String>> {
    let mut tx = pool.begin().await?;
    let Some(pb_cu): Option<Uuid> = sqlx::query_scalar("SELECT phong_ban_id FROM lt_danh_muc_dong WHERE id = $1 AND danh_muc_id = $2").bind(dong_id).bind(danh_muc_id).fetch_optional(&mut *tx).await? else {
        return Ok(Err("Không có dòng".into()));
    };
    if let Err(e) = cho_sua(&mut tx, danh_muc_id, phong_ban, pb_cu).await? {
        return Ok(Err(e));
    }
    sqlx::query("DELETE FROM lt_danh_muc_dong WHERE id = $1").bind(dong_id).execute(&mut *tx).await?;
    sqlx::query("UPDATE lt_danh_muc SET updated_at = now() WHERE id = $1").bind(danh_muc_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Ok(()))
}

/// Nhập nhiều dòng (dán từ bảng tính): trùng `ma_dong` trong danh mục → cập nhật; lỗi từng dòng trả về theo chỉ số (0-based).
pub async fn nhap_dong(pool: &PgPool, danh_muc_id: Uuid, project: Uuid, user: Uuid, phong_ban: Option<Uuid>, rows: &[DongMoi]) -> sqlx::Result<Result<(u64, u64, Vec<(usize, String)>), String>> {
    let mut tx = pool.begin().await?;
    let tt: Option<String> = sqlx::query_scalar("SELECT trang_thai FROM lt_danh_muc WHERE id = $1 FOR UPDATE").bind(danh_muc_id).fetch_optional(&mut *tx).await?;
    match tt.as_deref() {
        None => return Ok(Err("Không có danh mục".into())),
        Some("du_thao") => {}
        Some(_) => return Ok(Err("Danh mục đã trình, không nhập dòng được".into())),
    }
    let (mut them, mut sua, mut loi) = (0u64, 0u64, Vec::new());
    for (i, d) in rows.iter().enumerate() {
        if let Err(e) = kiem_dong(d) {
            loi.push((i, e));
            continue;
        }
        if phong_ban.is_some_and(|pb| pb != d.phong_ban_id) {
            loi.push((i, "Dòng thuộc phòng ban khác".into()));
            continue;
        }
        let n = sqlx::query(
            "INSERT INTO lt_danh_muc_dong (id, project_id, danh_muc_id, phong_ban_id, mang_id, ma_dong, tieu_de, thbq_id, nguoi_lap, ghi_chu, so_tt, created_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) \
             ON CONFLICT (danh_muc_id, ma_dong) DO UPDATE SET phong_ban_id = EXCLUDED.phong_ban_id, mang_id = EXCLUDED.mang_id, tieu_de = EXCLUDED.tieu_de, \
                 thbq_id = EXCLUDED.thbq_id, nguoi_lap = EXCLUDED.nguoi_lap, ghi_chu = EXCLUDED.ghi_chu, so_tt = EXCLUDED.so_tt, updated_at = now() \
             RETURNING (xmax = 0) AS moi",
        )
        .bind(Uuid::now_v7())
        .bind(project)
        .bind(danh_muc_id)
        .bind(d.phong_ban_id)
        .bind(d.mang_id)
        .bind(d.ma_dong.trim())
        .bind(d.tieu_de.trim())
        .bind(d.thbq_id)
        .bind(d.nguoi_lap.as_deref().map(str::trim))
        .bind(d.ghi_chu.as_deref().map(str::trim))
        .bind(d.so_tt.or(Some(i as i32 + 1)))
        .bind(user)
        .fetch_one(&mut *tx)
        .await;
        match n {
            Ok(r) => {
                if r.try_get::<bool, _>("moi").unwrap_or(true) {
                    them += 1
                } else {
                    sua += 1
                }
            }
            Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23503") => loi.push((i, "Phòng ban / mảng / THBQ không tồn tại".into())),
            Err(e) => return Err(e),
        }
    }
    sqlx::query("UPDATE lt_danh_muc SET updated_at = now() WHERE id = $1").bind(danh_muc_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Ok((them, sua, loi)))
}

/// Chuyển trạng thái chung: `from` → `to`, ghi người và thời điểm theo cột tương ứng.
async fn chuyen(pool: &PgPool, id: Uuid, from: &[&str], to: &str, user: Uuid, extra_set: &str, ly_do: Option<&str>) -> sqlx::Result<Option<DanhMucRow>> {
    let from: Vec<String> = from.iter().map(|s| s.to_string()).collect();
    sqlx::query_as::<_, DanhMucRow>(&format!(
        "UPDATE lt_danh_muc SET trang_thai = $2, updated_at = now(){extra_set} WHERE id = $1 AND trang_thai = ANY($3) RETURNING {COLS}"
    ))
    .bind(id)
    .bind(to)
    .bind(&from)
    .bind(user)
    .bind(ly_do)
    .fetch_optional(pool)
    .await
}

pub async fn trinh(pool: &PgPool, id: Uuid, user: Uuid) -> sqlx::Result<Result<DanhMucRow, String>> {
    let so_dong: i64 = sqlx::query_scalar("SELECT count(*) FROM lt_danh_muc_dong WHERE danh_muc_id = $1").bind(id).fetch_one(pool).await?;
    if so_dong == 0 {
        return Ok(Err("Danh mục chưa có dòng nào".into()));
    }
    Ok(chuyen(pool, id, &["du_thao"], "cho_duyet", user, ", trinh_boi = $4, trinh_luc = now(), ly_do_tra = NULL", None)
        .await?
        .ok_or_else(|| "Danh mục không ở trạng thái dự thảo".to_string()))
}

pub async fn duyet(pool: &PgPool, id: Uuid, user: Uuid) -> sqlx::Result<Result<DanhMucRow, String>> {
    Ok(chuyen(pool, id, &["cho_duyet"], "da_duyet", user, ", duyet_boi = $4, duyet_luc = now()", None)
        .await?
        .ok_or_else(|| "Danh mục không ở trạng thái chờ duyệt".to_string()))
}

pub async fn tra_lai(pool: &PgPool, id: Uuid, user: Uuid, ly_do: &str) -> sqlx::Result<Result<DanhMucRow, String>> {
    if ly_do.trim().is_empty() {
        return Ok(Err("Trả lại phải ghi lý do".into()));
    }
    Ok(chuyen(pool, id, &["cho_duyet", "da_duyet"], "du_thao", user, ", ly_do_tra = $5, trinh_boi = NULL, trinh_luc = NULL, duyet_boi = NULL, duyet_luc = NULL", Some(ly_do.trim()))
        .await?
        .ok_or_else(|| "Danh mục không ở trạng thái chờ duyệt / đã duyệt".to_string()))
}

/// Ban hành: bản ban hành cũ của cùng (phông, năm) → `het_hieu_luc` trong cùng giao dịch (chỉ mục duy nhất ux_lt_danh_muc_hieu_luc).
pub async fn ban_hanh(pool: &PgPool, id: Uuid, user: Uuid, so_quyet_dinh: &str, ngay: NaiveDate) -> sqlx::Result<Result<DanhMucRow, String>> {
    let so = so_quyet_dinh.trim();
    if so.is_empty() {
        return Ok(Err("Thiếu số quyết định ban hành".into()));
    }
    let mut tx = pool.begin().await?;
    let Some(d) = sqlx::query_as::<_, DanhMucRow>(&format!("SELECT {COLS} FROM lt_danh_muc WHERE id = $1 FOR UPDATE")).bind(id).fetch_optional(&mut *tx).await? else {
        return Ok(Err("Không có danh mục".into()));
    };
    if d.trang_thai != "da_duyet" {
        return Ok(Err("Danh mục chưa được duyệt".into()));
    }
    sqlx::query("UPDATE lt_danh_muc SET trang_thai = 'het_hieu_luc', updated_at = now() WHERE phong_id = $1 AND nam = $2 AND trang_thai = 'ban_hanh' AND id <> $3")
        .bind(d.phong_id)
        .bind(d.nam)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let row = sqlx::query_as::<_, DanhMucRow>(&format!(
        "UPDATE lt_danh_muc SET trang_thai = 'ban_hanh', so_quyet_dinh = $2, ngay_ban_hanh = $3, ban_hanh_boi = $4, ban_hanh_luc = now(), updated_at = now() WHERE id = $1 RETURNING {COLS}"
    ))
    .bind(id)
    .bind(so)
    .bind(ngay)
    .bind(user)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Ok(row))
}

/// Phiên bản mới từ một bản đã ban hành: dự thảo v+1 chép toàn bộ dòng.
pub async fn phien_ban_moi(pool: &PgPool, id: Uuid, user: Uuid) -> sqlx::Result<Result<DanhMucRow, String>> {
    let Some(d) = get(pool, id).await? else { return Ok(Err("Không có danh mục".into())) };
    if d.trang_thai != "ban_hanh" {
        return Ok(Err("Chỉ lập phiên bản mới từ bản đã ban hành".into()));
    }
    tao(pool, d.project_id, d.phong_id, d.nam, user, Some(id)).await
}

/// Đối chiếu danh mục với hồ sơ đã lập trong năm: mỗi dòng → số hồ sơ gắn (theo `dm_dong_id`, hoặc cùng `ma_dong` ở phiên bản
/// khác của cùng (phông, năm)); và danh sách hồ sơ NGOÀI danh mục (không gắn dòng nào).
pub async fn doi_chieu(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Value>> {
    let Some(d) = get(pool, id).await? else { return Ok(None) };
    let theo = sqlx::query(
        "SELECT jsonb_build_object('dongId', x.id, 'maDong', x.ma_dong, 'tieuDe', x.tieu_de, 'phongBanId', x.phong_ban_id, 'tenPhongBan', pb.ten, \
                'soHoSo', (SELECT count(*) FROM cl_ho_so h JOIN lt_danh_muc_dong y ON y.id = h.dm_dong_id JOIN lt_danh_muc dy ON dy.id = y.danh_muc_id \
                           WHERE h.deleted_at IS NULL AND dy.phong_id = $2 AND dy.nam = $3 AND y.ma_dong = x.ma_dong)) AS row \
         FROM lt_danh_muc_dong x LEFT JOIN cl_phong_ban pb ON pb.id = x.phong_ban_id WHERE x.danh_muc_id = $1 \
         ORDER BY pb.so_tt NULLS LAST, pb.ten, x.so_tt NULLS LAST, x.ma_dong COLLATE vi_so",
    )
    .bind(id)
    .bind(d.phong_id)
    .bind(d.nam)
    .fetch_all(pool)
    .await?;
    let ngoai = sqlx::query(
        "SELECT jsonb_build_object('id', v.id, 'soTam', v.so_tam, 'tieuDe', v.tieu_de, 'phongBanId', v.phong_ban_id, 'tenPhongBan', v.ten_phong_ban, \
                'ngayKetThuc', v.ngay_ket_thuc, 'nopLuuTrangThai', v.nop_luu_trang_thai) AS row \
         FROM cl_ho_so_v v WHERE v.phong_id = $1 AND v.deleted_at IS NULL AND v.dm_dong_id IS NULL AND v.nam_ket_thuc = $2 \
         ORDER BY v.ten_phong_ban, v.so_tam COLLATE vi_so",
    )
    .bind(d.phong_id)
    .bind(d.nam)
    .fetch_all(pool)
    .await?;
    let lay = |rows: Vec<sqlx::postgres::PgRow>| -> Vec<Value> { rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect() };
    Ok(Some(serde_json::json!({ "theoDanhMuc": lay(theo), "ngoaiDanhMuc": lay(ngoai) })))
}

/// Danh mục chờ duyệt quá `qua_ngay_lam_viec` ngày làm việc → nhắc quản trị dự án (worker). Trả số thông báo tạo.
pub async fn nhac_cho_duyet_lau(pool: &PgPool, hom_nay: NaiveDate, nghi: &std::collections::BTreeSet<NaiveDate>, qua_ngay_lam_viec: i64) -> sqlx::Result<u64> {
    let rows: Vec<(Uuid, Uuid, i32, i32, String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT d.id, d.project_id, d.nam, d.phien_ban, p.ma_du_an, d.trinh_luc FROM lt_danh_muc d JOIN cl_phong p ON p.id = d.phong_id \
         WHERE d.trang_thai = 'cho_duyet' AND d.trinh_luc IS NOT NULL",
    )
    .fetch_all(pool)
    .await?;
    let mut tao = 0u64;
    for (id, project, nam, pb, ma_phong, trinh_luc) in rows {
        let tu = trinh_luc.with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00")).date_naive();
        if crate::workdays::f_2ri(tu, hom_nay, nghi) <= qua_ngay_lam_viec {
            continue;
        }
        let msg = format!("Danh mục hồ sơ {ma_phong} năm {nam} (phiên bản {pb}) chờ duyệt đã quá {qua_ngay_lam_viec} ngày làm việc");
        let tuan = hom_nay.iso_week();
        let ref_ = format!("danh_muc:{id}:{}-{:02}", tuan.year(), tuan.week());
        for u in crate::projects::f_re(pool, project).await? {
            if crate::notifications::f_1v8_ref(pool, u, "danh_muc_cho_duyet_lau", &msg, &ref_).await? {
                tao += 1;
            }
        }
    }
    Ok(tao)
}
