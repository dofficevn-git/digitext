// Copyright (c) 2026 DigiText. All rights reserved.
//! Bảo quản đủ Điều 29–31 TT 05/2025/TT-BNV — migration 0032 (26/11/2026).
//!
//! - Kế hoạch kiểm tra (`bk_ke_hoach`): du_thao ──trình──► cho_duyet ──duyệt (chốt ảnh chụp tệp vào `bk_ke_hoach_tep`)──► da_duyet
//!   ──lần kiểm đầu──► dang_thuc_hien ──hết tệp `cho`──► hoan_thanh; từ chối → du_thao; huỷ từ du_thao / cho_duyet / da_duyet.
//!   Mọi lần kiểm (tự động, kế hoạch, tay, kiểm lại) vẫn ghi `integrity_checks` (+ `nguon`, `ke_hoach_id`), nên số liệu "đã kiểm
//!   trong 3 năm" của màn Tổng quan không đổi cách tính.
//! - Danh mục lỗi (`bk_loi`): một dòng ĐANG MỞ cho mỗi tệp / gói, mở từ bất kỳ lần kiểm lỗi nào (`ghi_loi_tu_kiem`); khắc phục →
//!   kiểm lại đạt / lỗi.
//! - Lịch sao lưu, phương tiện, sự cố: toàn máy chủ (backup_runs không có dự án). Toán hạn / trễ / sắp hết độ bền ở
//!   `digitext_core::bao_quan` (thuần, có test); đây chỉ đọc ghi.
use chrono::{DateTime, NaiveDate, Utc};
use digitext_core::bao_quan::PhamViKiemTra;
use serde_json::Value;
use sqlx::{PgPool, Row as _};
use uuid::Uuid;

// ------------------------------------------------------------------------------------------------------------------ kế hoạch

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct KhRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub nam: i32,
    pub ma: String,
    pub ten: String,
    pub pham_vi: Value,
    pub trang_thai: String,
    pub nguoi_lap: Option<Uuid>,
    pub so_tep: i32,
    pub so_dat: i32,
    pub so_loi: i32,
    pub record_id: Option<Uuid>,
}

const KH_COLS: &str = "id, project_id, nam, ma, ten, pham_vi, trang_thai, nguoi_lap, so_tep, so_dat, so_loi, record_id";

/// Dòng kế hoạch đủ cột + tên người (JSON thẳng ra client).
const KH_JSON: &str = "SELECT to_jsonb(k) || jsonb_build_object('nguoi_lap_ten', ul.display_name, 'duyet_ten', ud.display_name, \
     'so_cho', (SELECT count(*) FROM bk_ke_hoach_tep t WHERE t.ke_hoach_id = k.id AND t.trang_thai = 'cho'), \
     'so_loi_mo', (SELECT count(*) FROM bk_loi l WHERE l.ke_hoach_id = k.id AND l.trang_thai IN ('moi', 'da_khac_phuc', 'kiem_lai_loi'))) \
     FROM bk_ke_hoach k LEFT JOIN users ul ON ul.id = k.nguoi_lap LEFT JOIN users ud ON ud.id = k.duyet_boi";

pub async fn kh_list(pool: &PgPool, project_id: Uuid, nam: Option<i32>) -> sqlx::Result<Vec<Value>> {
    sqlx::query_scalar(&format!("{KH_JSON} WHERE k.project_id = $1 AND ($2::int IS NULL OR k.nam = $2) ORDER BY k.nam DESC, k.created_at DESC"))
        .bind(project_id)
        .bind(nam)
        .fetch_all(pool)
        .await
}

pub async fn kh_get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<KhRow>> {
    sqlx::query_as::<_, KhRow>(&format!("SELECT {KH_COLS} FROM bk_ke_hoach WHERE id = $1")).bind(id).fetch_optional(pool).await
}

pub async fn kh_get_json(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Value>> {
    sqlx::query_scalar(&format!("{KH_JSON} WHERE k.id = $1")).bind(id).fetch_optional(pool).await
}

pub struct KhMoi<'a> {
    pub project_id: Uuid,
    pub nam: i32,
    pub ten: &'a str,
    pub pham_vi: &'a Value,
    pub phuong_phap: Option<&'a str>,
    pub ghi_chu: Option<&'a str>,
    pub nguoi_lap: Uuid,
}

/// Lập kế hoạch dự thảo; mã `KT-<năm>-<stt 2 số>` theo dự án và năm (khoá tư vấn để hai người cùng lập không trùng số).
pub async fn kh_tao(pool: &PgPool, k: KhMoi<'_>) -> sqlx::Result<KhRow> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('bk_ke_hoach:' || $1::text || ':' || $2::text, 0))")
        .bind(k.project_id.to_string())
        .bind(k.nam.to_string())
        .execute(&mut *tx)
        .await?;
    let stt: i64 = sqlx::query_scalar("SELECT count(*) + 1 FROM bk_ke_hoach WHERE project_id = $1 AND nam = $2")
        .bind(k.project_id)
        .bind(k.nam)
        .fetch_one(&mut *tx)
        .await?;
    let ma = format!("KT-{}-{:02}", k.nam, stt);
    let row = sqlx::query_as::<_, KhRow>(&format!(
        "INSERT INTO bk_ke_hoach (id, project_id, nam, ma, ten, pham_vi, phuong_phap, ghi_chu, nguoi_lap) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING {KH_COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(k.project_id)
    .bind(k.nam)
    .bind(&ma)
    .bind(k.ten.trim())
    .bind(k.pham_vi)
    .bind(k.phuong_phap)
    .bind(k.ghi_chu)
    .bind(k.nguoi_lap)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}

/// Sửa khi còn dự thảo. Trả false nếu không ở dự thảo.
pub async fn kh_sua(pool: &PgPool, id: Uuid, ten: &str, pham_vi: &Value, phuong_phap: Option<&str>, ghi_chu: Option<&str>) -> sqlx::Result<bool> {
    let r = sqlx::query(
        "UPDATE bk_ke_hoach SET ten = $2, pham_vi = $3, phuong_phap = $4, ghi_chu = $5, updated_at = now() WHERE id = $1 AND trang_thai = 'du_thao'",
    )
    .bind(id)
    .bind(ten.trim())
    .bind(pham_vi)
    .bind(phuong_phap)
    .bind(ghi_chu)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

fn phong_ids(pv: &PhamViKiemTra) -> Option<Vec<Uuid>> {
    if pv.phong_ids.is_empty() {
        None
    } else {
        Some(pv.phong_ids.clone())
    }
}

/// Số tài liệu / gói sẽ rơi vào kế hoạch (trước lấy mẫu) — để người lập cân chỉnh phạm vi. Trả (tài liệu, gói, ước lượng sau lấy mẫu).
pub async fn kh_xem_truoc(pool: &PgPool, project_id: Uuid, pv: &PhamViKiemTra) -> sqlx::Result<(i64, i64, i64)> {
    let tai_lieu: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM documents d WHERE d.project_id = $1 AND d.status <> 'disposed' \
           AND ($2::uuid[] IS NULL OR EXISTS (SELECT 1 FROM cl_van_ban_file f JOIN cl_van_ban v ON v.id = f.van_ban_id \
                                               JOIN cl_ho_so h ON h.id = v.ho_so_id WHERE f.document_id = d.id AND h.phong_id = ANY($2))) \
           AND (NOT $3::bool OR NOT EXISTS (SELECT 1 FROM integrity_checks c WHERE c.document_id = d.id AND c.checked_at > now() - interval '3 years'))",
    )
    .bind(project_id)
    .bind(phong_ids(pv))
    .bind(pv.lau_chua_kiem)
    .fetch_one(pool)
    .await?;
    let goi: i64 = if pv.goi {
        sqlx::query_scalar(
            "SELECT count(*) FROM packages p WHERE p.project_id = $1 AND p.status = 'built' AND p.huy_at IS NULL AND p.zip_key IS NOT NULL AND p.zip_sha256 IS NOT NULL \
               AND (NOT $2::bool OR NOT EXISTS (SELECT 1 FROM integrity_checks c WHERE c.package_id = p.id AND c.checked_at > now() - interval '3 years'))",
        )
        .bind(project_id)
        .bind(pv.lau_chua_kiem)
        .fetch_one(pool)
        .await?
    } else {
        0
    };
    let uoc = ((tai_lieu as f64) * pv.ty_le()).round() as i64 + goi;
    Ok((tai_lieu, goi, uoc))
}

/// Trình duyệt: dự thảo → chờ duyệt. `Ok(Err)` khi sai trạng thái.
pub async fn kh_trinh(pool: &PgPool, id: Uuid) -> sqlx::Result<Result<(), String>> {
    let r = sqlx::query("UPDATE bk_ke_hoach SET trang_thai = 'cho_duyet', trinh_luc = now(), ly_do_tu_choi = NULL, updated_at = now() WHERE id = $1 AND trang_thai = 'du_thao'")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(if r.rows_affected() > 0 { Ok(()) } else { Err("Kế hoạch không ở trạng thái dự thảo".into()) })
}

/// Duyệt: chờ duyệt → đã duyệt, đồng thời CHỐT ảnh chụp tệp / gói theo phạm vi (tệp thêm sau không vào kế hoạch này).
/// Trả số tệp đã chốt.
pub async fn kh_duyet(pool: &PgPool, id: Uuid, boi: Uuid) -> sqlx::Result<Result<i64, String>> {
    let Some(k) = kh_get(pool, id).await? else { return Ok(Err("Không có kế hoạch".into())) };
    if k.trang_thai != "cho_duyet" {
        return Ok(Err("Kế hoạch không ở trạng thái chờ duyệt".into()));
    }
    let pv: PhamViKiemTra = serde_json::from_value(k.pham_vi.clone()).unwrap_or_default();
    let mut tx = pool.begin().await?;
    let n_doc = sqlx::query(
        "INSERT INTO bk_ke_hoach_tep (id, ke_hoach_id, document_id) \
         SELECT gen_random_uuid(), $1, d.id FROM documents d WHERE d.project_id = $2 AND d.status <> 'disposed' \
           AND ($3::uuid[] IS NULL OR EXISTS (SELECT 1 FROM cl_van_ban_file f JOIN cl_van_ban v ON v.id = f.van_ban_id \
                                               JOIN cl_ho_so h ON h.id = v.ho_so_id WHERE f.document_id = d.id AND h.phong_id = ANY($3))) \
           AND (NOT $4::bool OR NOT EXISTS (SELECT 1 FROM integrity_checks c WHERE c.document_id = d.id AND c.checked_at > now() - interval '3 years')) \
           AND ($5::float8 >= 1 OR random() < $5) \
         ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(k.project_id)
    .bind(phong_ids(&pv))
    .bind(pv.lau_chua_kiem)
    .bind(pv.ty_le())
    .execute(&mut *tx)
    .await?
    .rows_affected();
    let n_goi = if pv.goi {
        sqlx::query(
            "INSERT INTO bk_ke_hoach_tep (id, ke_hoach_id, package_id) \
             SELECT gen_random_uuid(), $1, p.id FROM packages p WHERE p.project_id = $2 AND p.status = 'built' AND p.huy_at IS NULL \
               AND p.zip_key IS NOT NULL AND p.zip_sha256 IS NOT NULL \
               AND (NOT $3::bool OR NOT EXISTS (SELECT 1 FROM integrity_checks c WHERE c.package_id = p.id AND c.checked_at > now() - interval '3 years')) \
             ON CONFLICT DO NOTHING",
        )
        .bind(id)
        .bind(k.project_id)
        .bind(pv.lau_chua_kiem)
        .execute(&mut *tx)
        .await?
        .rows_affected()
    } else {
        0
    };
    let so_tep = (n_doc + n_goi) as i64;
    sqlx::query("UPDATE bk_ke_hoach SET trang_thai = 'da_duyet', duyet_boi = $2, duyet_luc = now(), so_tep = $3, so_dat = 0, so_loi = 0, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(boi)
        .bind(so_tep as i32)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Ok(so_tep))
}

pub async fn kh_tu_choi(pool: &PgPool, id: Uuid, boi: Uuid, ly_do: &str) -> sqlx::Result<Result<(), String>> {
    let r = sqlx::query("UPDATE bk_ke_hoach SET trang_thai = 'du_thao', duyet_boi = $2, ly_do_tu_choi = $3, updated_at = now() WHERE id = $1 AND trang_thai = 'cho_duyet'")
        .bind(id)
        .bind(boi)
        .bind(ly_do.trim())
        .execute(pool)
        .await?;
    Ok(if r.rows_affected() > 0 { Ok(()) } else { Err("Kế hoạch không ở trạng thái chờ duyệt".into()) })
}

pub async fn kh_huy(pool: &PgPool, id: Uuid) -> sqlx::Result<Result<(), String>> {
    let r = sqlx::query("UPDATE bk_ke_hoach SET trang_thai = 'huy', updated_at = now() WHERE id = $1 AND trang_thai IN ('du_thao', 'cho_duyet', 'da_duyet')")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(if r.rows_affected() > 0 { Ok(()) } else { Err("Kế hoạch đang thực hiện hoặc đã xong, không huỷ được".into()) })
}

/// Một tệp / gói còn chờ kiểm trong kế hoạch, kèm dữ liệu để băm lại.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TepCho {
    pub id: Uuid,
    pub document_id: Option<Uuid>,
    pub package_id: Option<Uuid>,
    pub page_count: Option<i32>,
    /// blake3 gốc chỉ với tài liệu nhập từ gói (như `ops::f_17z`).
    pub blake3: Option<String>,
    pub zip_key: Option<String>,
    pub zip_sha256: Option<String>,
}

pub async fn kh_tep_cho(pool: &PgPool, ke_hoach_id: Uuid, limit: i64) -> sqlx::Result<Vec<TepCho>> {
    sqlx::query_as::<_, TepCho>(
        "SELECT t.id, t.document_id, t.package_id, d.page_count, CASE WHEN d.goi_id IS NOT NULL THEN d.blake3 END AS blake3, p.zip_key, p.zip_sha256 \
         FROM bk_ke_hoach_tep t LEFT JOIN documents d ON d.id = t.document_id LEFT JOIN packages p ON p.id = t.package_id \
         WHERE t.ke_hoach_id = $1 AND t.trang_thai = 'cho' ORDER BY t.id LIMIT $2",
    )
    .bind(ke_hoach_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Kế hoạch đã duyệt / đang thực hiện còn tệp chờ — worker lấy mỗi giờ.
pub async fn kh_dang_chay(pool: &PgPool) -> sqlx::Result<Vec<KhRow>> {
    sqlx::query_as::<_, KhRow>(&format!(
        "SELECT {KH_COLS} FROM bk_ke_hoach k WHERE k.trang_thai IN ('da_duyet', 'dang_thuc_hien') \
           AND EXISTS (SELECT 1 FROM bk_ke_hoach_tep t WHERE t.ke_hoach_id = k.id AND t.trang_thai = 'cho') ORDER BY k.duyet_luc"
    ))
    .fetch_all(pool)
    .await
}

/// Ghi một lần kiểm thuộc kế hoạch: integrity_checks (nguon = 'ke_hoach') + cập nhật dòng tệp + bộ đếm + bk_loi.
/// Trả `true` khi lần này làm kế hoạch HOÀN THÀNH (hết tệp chờ) — để báo người lập.
pub async fn kh_ghi_ket_qua(pool: &PgPool, k: &KhRow, tep: &TepCho, ket_qua: &str, chi_tiet: Option<&str>) -> sqlx::Result<bool> {
    let check_id = Uuid::now_v7();
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO integrity_checks (id, project_id, document_id, package_id, ket_qua, chi_tiet, ke_hoach_id, nguon) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'ke_hoach')",
    )
    .bind(check_id)
    .bind(k.project_id)
    .bind(tep.document_id)
    .bind(tep.package_id)
    .bind(ket_qua)
    .bind(chi_tiet)
    .bind(k.id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE bk_ke_hoach_tep SET trang_thai = $2, check_id = $3, chi_tiet = $4, kiem_luc = now() WHERE id = $1")
        .bind(tep.id)
        .bind(ket_qua)
        .bind(check_id)
        .bind(chi_tiet)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE bk_ke_hoach SET so_dat = so_dat + ($2 = 'dat')::int, so_loi = so_loi + ($2 = 'loi')::int, \
                trang_thai = CASE WHEN trang_thai = 'da_duyet' THEN 'dang_thuc_hien' ELSE trang_thai END, \
                bat_dau_luc = coalesce(bat_dau_luc, now()), updated_at = now() WHERE id = $1",
    )
    .bind(k.id)
    .bind(ket_qua)
    .execute(&mut *tx)
    .await?;
    let xong = sqlx::query(
        "UPDATE bk_ke_hoach SET trang_thai = 'hoan_thanh', ket_thuc_luc = now(), updated_at = now() \
         WHERE id = $1 AND trang_thai = 'dang_thuc_hien' AND NOT EXISTS (SELECT 1 FROM bk_ke_hoach_tep WHERE ke_hoach_id = $1 AND trang_thai = 'cho')",
    )
    .bind(k.id)
    .execute(&mut *tx)
    .await?
    .rows_affected()
        > 0;
    tx.commit().await?;
    ghi_loi_tu_kiem(pool, k.project_id, tep.document_id, tep.package_id, check_id, Some(k.id), ket_qua, chi_tiet).await?;
    Ok(xong)
}

/// Tệp / gói của kế hoạch kèm tên, mã hồ sơ và địa chỉ kho (để in danh mục tệp lỗi — Điều 29.4.đ).
pub async fn kh_tep(pool: &PgPool, ke_hoach_id: Uuid, trang_thai: Option<&str>, limit: i64, offset: i64) -> sqlx::Result<(Vec<Value>, i64)> {
    let rows = sqlx::query_scalar(
        "SELECT jsonb_build_object('id', t.id, 'documentId', t.document_id, 'packageId', t.package_id, 'trangThai', t.trang_thai, \
                'chiTiet', t.chi_tiet, 'kiemLuc', t.kiem_luc, 'tenTep', coalesce(d.filename, p.label), 'maHoSo', d.ho_so_ref, \
                'objid', p.objid, 'projectId', coalesce(d.project_id, p.project_id)) \
         FROM bk_ke_hoach_tep t LEFT JOIN documents d ON d.id = t.document_id LEFT JOIN packages p ON p.id = t.package_id \
         WHERE t.ke_hoach_id = $1 AND ($2::text IS NULL OR t.trang_thai = $2) \
         ORDER BY (t.trang_thai = 'loi') DESC, t.kiem_luc DESC NULLS LAST, t.id LIMIT $3 OFFSET $4",
    )
    .bind(ke_hoach_id)
    .bind(trang_thai)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM bk_ke_hoach_tep WHERE ke_hoach_id = $1 AND ($2::text IS NULL OR trang_thai = $2)")
        .bind(ke_hoach_id)
        .bind(trang_thai)
        .fetch_one(pool)
        .await?;
    Ok((rows, total))
}

pub async fn kh_set_record(pool: &PgPool, id: Uuid, record_id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE bk_ke_hoach SET record_id = $2, updated_at = now() WHERE id = $1").bind(id).bind(record_id).execute(pool).await?;
    Ok(())
}

/// Tên các phông trong phạm vi (để in biên bản).
pub async fn ten_phong(pool: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<String>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    sqlx::query_scalar("SELECT ten FROM cl_phong WHERE id = ANY($1) ORDER BY ten").bind(ids).fetch_all(pool).await
}

// ------------------------------------------------------------------------------------------------------------------ bk_loi

/// Sau MỌI lần kiểm (tự động / kế hoạch / tay): lỗi → mở dòng bk_loi (hoặc đánh "kiểm lại lỗi" nếu đã khắc phục); đạt → đóng
/// dòng đã khắc phục thành "kiểm lại đạt". Gọi từ `ops::f_26a`, `ops::ghi_kiem_goi` và `kh_ghi_ket_qua`.
#[allow(clippy::too_many_arguments)]
pub async fn ghi_loi_tu_kiem(
    pool: &PgPool,
    project_id: Uuid,
    document_id: Option<Uuid>,
    package_id: Option<Uuid>,
    check_id: Uuid,
    ke_hoach_id: Option<Uuid>,
    ket_qua: &str,
    chi_tiet: Option<&str>,
) -> sqlx::Result<()> {
    if ket_qua == "loi" {
        let n = sqlx::query(
            "UPDATE bk_loi SET chi_tiet = $3, check_id = $4, ke_hoach_id = coalesce($5, ke_hoach_id), updated_at = now(), \
                    trang_thai = CASE WHEN trang_thai = 'da_khac_phuc' THEN 'kiem_lai_loi' ELSE trang_thai END, \
                    kiem_lai_check_id = CASE WHEN trang_thai = 'da_khac_phuc' THEN $4 ELSE kiem_lai_check_id END, \
                    kiem_lai_luc = CASE WHEN trang_thai = 'da_khac_phuc' THEN now() ELSE kiem_lai_luc END \
             WHERE (($1::uuid IS NOT NULL AND document_id = $1) OR ($2::uuid IS NOT NULL AND package_id = $2)) \
               AND trang_thai IN ('moi', 'da_khac_phuc', 'kiem_lai_loi')",
        )
        .bind(document_id)
        .bind(package_id)
        .bind(chi_tiet)
        .bind(check_id)
        .bind(ke_hoach_id)
        .execute(pool)
        .await?
        .rows_affected();
        if n == 0 {
            sqlx::query("INSERT INTO bk_loi (id, project_id, ke_hoach_id, document_id, package_id, check_id, chi_tiet) VALUES ($1, $2, $3, $4, $5, $6, $7)")
                .bind(Uuid::now_v7())
                .bind(project_id)
                .bind(ke_hoach_id)
                .bind(document_id)
                .bind(package_id)
                .bind(check_id)
                .bind(chi_tiet)
                .execute(pool)
                .await?;
        }
    } else {
        sqlx::query(
            "UPDATE bk_loi SET trang_thai = 'kiem_lai_dat', kiem_lai_check_id = $3, kiem_lai_luc = now(), updated_at = now() \
             WHERE (($1::uuid IS NOT NULL AND document_id = $1) OR ($2::uuid IS NOT NULL AND package_id = $2)) AND trang_thai = 'da_khac_phuc'",
        )
        .bind(document_id)
        .bind(package_id)
        .bind(check_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

const LOI_JSON: &str = "SELECT to_jsonb(l) || jsonb_build_object('ten_tep', coalesce(d.filename, p.label), 'ma_ho_so', d.ho_so_ref, 'objid', p.objid, \
     'khac_phuc_ten', u.display_name, 'ke_hoach_ma', k.ma) \
     FROM bk_loi l LEFT JOIN documents d ON d.id = l.document_id LEFT JOIN packages p ON p.id = l.package_id \
     LEFT JOIN users u ON u.id = l.khac_phuc_boi LEFT JOIN bk_ke_hoach k ON k.id = l.ke_hoach_id";

/// `mo = true` chỉ dòng đang mở (mới / đã khắc phục / kiểm lại lỗi).
pub async fn loi_list(pool: &PgPool, projects: Option<&[Uuid]>, ke_hoach_id: Option<Uuid>, mo: bool, limit: i64, offset: i64) -> sqlx::Result<Vec<Value>> {
    sqlx::query_scalar(&format!(
        "{LOI_JSON} WHERE ($1::uuid[] IS NULL OR l.project_id = ANY($1)) AND ($2::uuid IS NULL OR l.ke_hoach_id = $2) \
           AND (NOT $3::bool OR l.trang_thai IN ('moi', 'da_khac_phuc', 'kiem_lai_loi')) \
         ORDER BY l.phat_hien_luc DESC LIMIT $4 OFFSET $5"
    ))
    .bind(projects)
    .bind(ke_hoach_id)
    .bind(mo)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LoiRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub ke_hoach_id: Option<Uuid>,
    pub document_id: Option<Uuid>,
    pub package_id: Option<Uuid>,
    pub trang_thai: String,
    pub page_count: Option<i32>,
    pub blake3: Option<String>,
    pub zip_key: Option<String>,
    pub zip_sha256: Option<String>,
}

pub async fn loi_get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<LoiRow>> {
    sqlx::query_as::<_, LoiRow>(
        "SELECT l.id, l.project_id, l.ke_hoach_id, l.document_id, l.package_id, l.trang_thai, d.page_count, \
                CASE WHEN d.goi_id IS NOT NULL THEN d.blake3 END AS blake3, p.zip_key, p.zip_sha256 \
         FROM bk_loi l LEFT JOIN documents d ON d.id = l.document_id LEFT JOIN packages p ON p.id = l.package_id WHERE l.id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn loi_khac_phuc(pool: &PgPool, id: Uuid, boi: Uuid, khac_phuc: &str) -> sqlx::Result<bool> {
    let r = sqlx::query(
        "UPDATE bk_loi SET trang_thai = 'da_khac_phuc', khac_phuc = $3, khac_phuc_boi = $2, khac_phuc_luc = now(), updated_at = now() \
         WHERE id = $1 AND trang_thai IN ('moi', 'kiem_lai_loi')",
    )
    .bind(id)
    .bind(boi)
    .bind(khac_phuc.trim())
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn loi_bo_qua(pool: &PgPool, id: Uuid, boi: Uuid, ly_do: &str) -> sqlx::Result<bool> {
    let r = sqlx::query(
        "UPDATE bk_loi SET trang_thai = 'bo_qua', khac_phuc = $3, khac_phuc_boi = $2, khac_phuc_luc = now(), updated_at = now() \
         WHERE id = $1 AND trang_thai IN ('moi', 'da_khac_phuc', 'kiem_lai_loi')",
    )
    .bind(id)
    .bind(boi)
    .bind(ly_do.trim())
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

/// Kiểm lại một dòng lỗi (sau khắc phục): ghi integrity_checks nguon = 'kiem_lai' rồi để `ghi_loi_tu_kiem` đóng / mở lại.
pub async fn loi_kiem_lai_ghi(pool: &PgPool, l: &LoiRow, ket_qua: &str, chi_tiet: Option<&str>) -> sqlx::Result<()> {
    let check_id = Uuid::now_v7();
    sqlx::query("INSERT INTO integrity_checks (id, project_id, document_id, package_id, ket_qua, chi_tiet, ke_hoach_id, nguon) VALUES ($1, $2, $3, $4, $5, $6, $7, 'kiem_lai')")
        .bind(check_id)
        .bind(l.project_id)
        .bind(l.document_id)
        .bind(l.package_id)
        .bind(ket_qua)
        .bind(chi_tiet)
        .bind(l.ke_hoach_id)
        .execute(pool)
        .await?;
    // Dòng đang 'moi' (chưa khai khắc phục) mà kiểm lại đạt thì cũng đóng: hạ về 'da_khac_phuc' trước để quy tắc chung xử lý.
    if ket_qua == "dat" {
        sqlx::query("UPDATE bk_loi SET trang_thai = 'da_khac_phuc', khac_phuc = coalesce(khac_phuc, 'Kiểm lại đạt'), khac_phuc_luc = coalesce(khac_phuc_luc, now()) WHERE id = $1 AND trang_thai IN ('moi', 'kiem_lai_loi')")
            .bind(l.id)
            .execute(pool)
            .await?;
    }
    ghi_loi_tu_kiem(pool, l.project_id, l.document_id, l.package_id, check_id, l.ke_hoach_id, ket_qua, chi_tiet).await
}

// ------------------------------------------------------------------------------------------------------------------ lịch sao lưu

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LichRow {
    pub id: Uuid,
    pub loai: String,
    pub ten: String,
    pub chu_ky_gio: i32,
    pub do_tre_cho_phep_gio: i32,
    pub giu_so_ban: i32,
    pub bat: bool,
    pub nguon: Option<String>,
    pub phuong_tien_id: Option<Uuid>,
    pub ghi_chu: Option<String>,
    /// Lần sao lưu ĐẠT gần nhất thuộc lịch.
    pub lan_dat: Option<DateTime<Utc>>,
    /// Số bản đạt còn giữ (chưa đánh dấu đã xoá).
    pub so_ban: i64,
    pub phuong_tien_ma: Option<String>,
}

pub async fn lich_list(pool: &PgPool) -> sqlx::Result<Vec<LichRow>> {
    sqlx::query_as::<_, LichRow>(
        "SELECT l.id, l.loai, l.ten, l.chu_ky_gio, l.do_tre_cho_phep_gio, l.giu_so_ban, l.bat, l.nguon, l.phuong_tien_id, l.ghi_chu, \
                (SELECT max(r.created_at) FROM backup_runs r WHERE r.lich_id = l.id AND r.ket_qua = 'dat') AS lan_dat, \
                (SELECT count(*) FROM backup_runs r WHERE r.lich_id = l.id AND r.ket_qua = 'dat' AND r.da_xoa_at IS NULL) AS so_ban, \
                pt.ma AS phuong_tien_ma \
         FROM bk_lich_sao_luu l LEFT JOIN bk_phuong_tien pt ON pt.id = l.phuong_tien_id ORDER BY l.chu_ky_gio, l.created_at",
    )
    .fetch_all(pool)
    .await
}

pub struct LichSua<'a> {
    pub ten: &'a str,
    pub chu_ky_gio: i32,
    pub do_tre_cho_phep_gio: i32,
    pub giu_so_ban: i32,
    pub bat: bool,
    pub nguon: Option<&'a str>,
    pub phuong_tien_id: Option<Uuid>,
    pub ghi_chu: Option<&'a str>,
}

pub async fn lich_tao(pool: &PgPool, loai: &str, s: LichSua<'_>) -> sqlx::Result<Uuid> {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO bk_lich_sao_luu (id, loai, ten, chu_ky_gio, do_tre_cho_phep_gio, giu_so_ban, bat, nguon, phuong_tien_id, ghi_chu) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(id)
    .bind(loai)
    .bind(s.ten.trim())
    .bind(s.chu_ky_gio)
    .bind(s.do_tre_cho_phep_gio)
    .bind(s.giu_so_ban)
    .bind(s.bat)
    .bind(s.nguon)
    .bind(s.phuong_tien_id)
    .bind(s.ghi_chu)
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn lich_sua(pool: &PgPool, id: Uuid, s: LichSua<'_>) -> sqlx::Result<bool> {
    let r = sqlx::query(
        "UPDATE bk_lich_sao_luu SET ten = $2, chu_ky_gio = $3, do_tre_cho_phep_gio = $4, giu_so_ban = $5, bat = $6, nguon = $7, phuong_tien_id = $8, ghi_chu = $9, updated_at = now() WHERE id = $1",
    )
    .bind(id)
    .bind(s.ten.trim())
    .bind(s.chu_ky_gio)
    .bind(s.do_tre_cho_phep_gio)
    .bind(s.giu_so_ban)
    .bind(s.bat)
    .bind(s.nguon)
    .bind(s.phuong_tien_id)
    .bind(s.ghi_chu)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

/// Lịch theo loại (script sao lưu cũ không gửi lich_id → gắn theo loại).
pub async fn lich_theo_loai(pool: &PgPool, loai: &str) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar("SELECT id FROM bk_lich_sao_luu WHERE loai = $1 AND bat ORDER BY created_at LIMIT 1").bind(loai).fetch_optional(pool).await
}

const RUN_JSON: &str = "SELECT to_jsonb(r) || jsonb_build_object('lich_ten', l.ten, 'phuong_tien_ma', pt.ma, 'reported_ten', u.display_name) \
     FROM backup_runs r LEFT JOIN bk_lich_sao_luu l ON l.id = r.lich_id LEFT JOIN bk_phuong_tien pt ON pt.id = r.phuong_tien_id \
     LEFT JOIN users u ON u.id = r.reported_by";

pub async fn runs(pool: &PgPool, lich_id: Option<Uuid>, limit: i64, offset: i64) -> sqlx::Result<Vec<Value>> {
    sqlx::query_scalar(&format!("{RUN_JSON} WHERE ($1::uuid IS NULL OR r.lich_id = $1) ORDER BY r.created_at DESC LIMIT $2 OFFSET $3"))
        .bind(lich_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}

/// Bản đạt cũ nhất vượt số bản phải giữ của lịch — "cần xoá" (chỉ nhắc + ghi nhận; xoá thật do script / người).
pub async fn ban_cu_can_xoa(pool: &PgPool, lich_id: Uuid, giu_so_ban: i32) -> sqlx::Result<Vec<Value>> {
    sqlx::query_scalar(&format!(
        "{RUN_JSON} WHERE r.id IN (SELECT id FROM backup_runs WHERE lich_id = $1 AND ket_qua = 'dat' AND da_xoa_at IS NULL \
                                   ORDER BY created_at DESC OFFSET $2) ORDER BY r.created_at"
    ))
    .bind(lich_id)
    .bind(giu_so_ban.max(1) as i64)
    .fetch_all(pool)
    .await
}

pub async fn run_da_xoa(pool: &PgPool, run_id: Uuid, boi: Uuid) -> sqlx::Result<bool> {
    let r = sqlx::query("UPDATE backup_runs SET da_xoa_at = now(), xoa_boi = $2 WHERE id = $1 AND da_xoa_at IS NULL")
        .bind(run_id)
        .bind(boi)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

pub struct RunMoi<'a> {
    pub loai: &'a str,
    pub nguon: &'a str,
    pub phuong_tien: Option<&'a str>,
    pub ket_qua: &'a str,
    pub chi_tiet: Option<&'a str>,
    pub reported_by: Option<Uuid>,
    pub lich_id: Option<Uuid>,
    pub phuong_tien_id: Option<Uuid>,
    pub duong_dan: Option<&'a str>,
    pub dung_luong_bytes: Option<i64>,
    pub ma_bam: Option<&'a str>,
    pub bat_dau_at: Option<DateTime<Utc>>,
    pub ket_thuc_at: Option<DateTime<Utc>>,
    pub nguoi_thuc_hien: Option<&'a str>,
}

/// Ghi một lần sao lưu đủ cột 0032 (lịch tự gắn theo loại nếu không truyền).
pub async fn run_ghi(pool: &PgPool, r: RunMoi<'_>) -> sqlx::Result<Value> {
    let lich = match r.lich_id {
        Some(l) => Some(l),
        None => lich_theo_loai(pool, r.loai).await?,
    };
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO backup_runs (id, loai, nguon, phuong_tien, ket_qua, chi_tiet, reported_by, lich_id, phuong_tien_id, duong_dan, dung_luong_bytes, ma_bam, bat_dau_at, ket_thuc_at, nguoi_thuc_hien) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
    )
    .bind(id)
    .bind(r.loai)
    .bind(r.nguon)
    .bind(r.phuong_tien)
    .bind(r.ket_qua)
    .bind(r.chi_tiet)
    .bind(r.reported_by)
    .bind(lich)
    .bind(r.phuong_tien_id)
    .bind(r.duong_dan)
    .bind(r.dung_luong_bytes)
    .bind(r.ma_bam)
    .bind(r.bat_dau_at)
    .bind(r.ket_thuc_at)
    .bind(r.nguoi_thuc_hien)
    .execute(pool)
    .await?;
    sqlx::query_scalar(&format!("{RUN_JSON} WHERE r.id = $1")).bind(id).fetch_one(pool).await
}

// ------------------------------------------------------------------------------------------------------------------ phương tiện

const PT_JSON: &str = "SELECT to_jsonb(pt) || jsonb_build_object('so_lan_dung', (SELECT count(*) FROM backup_runs r WHERE r.phuong_tien_id = pt.id), \
     'thay_boi_ma', t2.ma) FROM bk_phuong_tien pt LEFT JOIN bk_phuong_tien t2 ON t2.id = pt.thay_boi_id";

pub async fn pt_list(pool: &PgPool) -> sqlx::Result<Vec<Value>> {
    sqlx::query_scalar(&format!("{PT_JSON} ORDER BY (pt.trang_thai = 'dang_dung') DESC, pt.han_do_ben NULLS LAST, pt.ma")).fetch_all(pool).await
}

pub struct PtMoi<'a> {
    pub ma: &'a str,
    pub loai: &'a str,
    pub mo_ta: Option<&'a str>,
    pub dung_luong_bytes: Option<i64>,
    pub noi_bao_quan: Option<&'a str>,
    pub ngay_dua_vao: Option<NaiveDate>,
    pub han_do_ben: Option<NaiveDate>,
    pub trang_thai: &'a str,
    pub thay_boi_id: Option<Uuid>,
    pub ghi_chu: Option<&'a str>,
}

pub async fn pt_tao(pool: &PgPool, p: PtMoi<'_>) -> sqlx::Result<Value> {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO bk_phuong_tien (id, ma, loai, mo_ta, dung_luong_bytes, noi_bao_quan, ngay_dua_vao, han_do_ben, trang_thai, thay_boi_id, ghi_chu) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(id)
    .bind(p.ma.trim())
    .bind(p.loai.trim())
    .bind(p.mo_ta)
    .bind(p.dung_luong_bytes)
    .bind(p.noi_bao_quan)
    .bind(p.ngay_dua_vao)
    .bind(p.han_do_ben)
    .bind(p.trang_thai)
    .bind(p.thay_boi_id)
    .bind(p.ghi_chu)
    .execute(pool)
    .await?;
    sqlx::query_scalar(&format!("{PT_JSON} WHERE pt.id = $1")).bind(id).fetch_one(pool).await
}

pub async fn pt_sua(pool: &PgPool, id: Uuid, p: PtMoi<'_>) -> sqlx::Result<Option<Value>> {
    let n = sqlx::query(
        "UPDATE bk_phuong_tien SET ma = $2, loai = $3, mo_ta = $4, dung_luong_bytes = $5, noi_bao_quan = $6, ngay_dua_vao = $7, han_do_ben = $8, \
                trang_thai = $9, thay_boi_id = $10, ghi_chu = $11, updated_at = now() WHERE id = $1",
    )
    .bind(id)
    .bind(p.ma.trim())
    .bind(p.loai.trim())
    .bind(p.mo_ta)
    .bind(p.dung_luong_bytes)
    .bind(p.noi_bao_quan)
    .bind(p.ngay_dua_vao)
    .bind(p.han_do_ben)
    .bind(p.trang_thai)
    .bind(p.thay_boi_id)
    .bind(p.ghi_chu)
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 {
        return Ok(None);
    }
    sqlx::query_scalar(&format!("{PT_JSON} WHERE pt.id = $1")).bind(id).fetch_optional(pool).await
}

/// Phương tiện đang dùng có hạn độ bền trong vòng `truoc_ngay` ngày (kể cả đã quá) — worker nhắc chuyển đổi (Điều 30.4).
pub async fn pt_sap_het(pool: &PgPool, hom_nay: NaiveDate, truoc_ngay: i64) -> sqlx::Result<Vec<(Uuid, String, NaiveDate)>> {
    sqlx::query_as("SELECT id, ma, han_do_ben FROM bk_phuong_tien WHERE trang_thai = 'dang_dung' AND han_do_ben IS NOT NULL AND han_do_ben <= $1 + $2::int ORDER BY han_do_ben")
        .bind(hom_nay)
        .bind(truoc_ngay as i32)
        .fetch_all(pool)
        .await
}

pub async fn pt_danh_dau_nhac(pool: &PgPool, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE bk_phuong_tien SET nhac_luc = now() WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

// ------------------------------------------------------------------------------------------------------------------ sự cố

const SC_JSON: &str = "SELECT to_jsonb(s) || jsonb_build_object('nguoi_xu_ly_ten', u.display_name, 'project_code', p.code, \
     'backup_run', CASE WHEN r.id IS NULL THEN NULL ELSE jsonb_build_object('id', r.id, 'loai', r.loai, 'nguon', r.nguon, 'created_at', r.created_at, 'duong_dan', r.duong_dan) END, \
     'so_bien_ban', rec.so_bien_ban) \
     FROM bk_su_co s LEFT JOIN users u ON u.id = s.nguoi_xu_ly LEFT JOIN projects p ON p.id = s.project_id \
     LEFT JOIN backup_runs r ON r.id = s.backup_run_id LEFT JOIN records rec ON rec.id = s.record_id";

pub async fn sc_list(pool: &PgPool, projects: Option<&[Uuid]>, mo: bool, limit: i64, offset: i64) -> sqlx::Result<Vec<Value>> {
    sqlx::query_scalar(&format!(
        "{SC_JSON} WHERE ($1::uuid[] IS NULL OR s.project_id IS NULL OR s.project_id = ANY($1)) AND (NOT $2::bool OR s.trang_thai <> 'dong') \
         ORDER BY s.thoi_diem DESC LIMIT $3 OFFSET $4"
    ))
    .bind(projects)
    .bind(mo)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn sc_get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Value>> {
    sqlx::query_scalar(&format!("{SC_JSON} WHERE s.id = $1")).bind(id).fetch_optional(pool).await
}

/// Cột sửa được của sự cố (PUT nhận JSON camelCase từng phần).
pub struct ScSua<'a> {
    pub project_id: Option<Uuid>,
    pub thoi_diem: DateTime<Utc>,
    pub mo_ta: &'a str,
    pub nguyen_nhan: Option<&'a str>,
    pub muc_do: &'a str,
    pub pham_vi: Option<&'a str>,
    pub bien_phap: Option<&'a str>,
    pub backup_run_id: Option<Uuid>,
    pub phuong_phap_phuc_hoi: Option<&'a str>,
    pub phuc_hoi_luc: Option<DateTime<Utc>>,
    pub ket_qua_kiem_tra: Option<&'a str>,
    pub du_lieu_mat: Option<&'a str>,
    pub trang_thai: &'a str,
    pub nguoi_xu_ly: Option<Uuid>,
}

/// Mã `SC-<năm>-<stt 3 số>` theo năm (toàn máy chủ).
pub async fn sc_tao(pool: &PgPool, s: ScSua<'_>, created_by: Uuid) -> sqlx::Result<Value> {
    let nam = s.thoi_diem.with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00")).format("%Y").to_string();
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('bk_su_co:' || $1::text, 0))").bind(&nam).execute(&mut *tx).await?;
    let stt: i64 = sqlx::query_scalar("SELECT count(*) + 1 FROM bk_su_co WHERE ma LIKE $1").bind(format!("SC-{nam}-%")).fetch_one(&mut *tx).await?;
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO bk_su_co (id, ma, project_id, thoi_diem, mo_ta, nguyen_nhan, muc_do, pham_vi, bien_phap, backup_run_id, phuong_phap_phuc_hoi, phuc_hoi_luc, \
                ket_qua_kiem_tra, du_lieu_mat, trang_thai, nguoi_xu_ly, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)",
    )
    .bind(id)
    .bind(format!("SC-{nam}-{stt:03}"))
    .bind(s.project_id)
    .bind(s.thoi_diem)
    .bind(s.mo_ta.trim())
    .bind(s.nguyen_nhan)
    .bind(s.muc_do)
    .bind(s.pham_vi)
    .bind(s.bien_phap)
    .bind(s.backup_run_id)
    .bind(s.phuong_phap_phuc_hoi)
    .bind(s.phuc_hoi_luc)
    .bind(s.ket_qua_kiem_tra)
    .bind(s.du_lieu_mat)
    .bind(s.trang_thai)
    .bind(s.nguoi_xu_ly.or(Some(created_by)))
    .bind(created_by)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    sqlx::query_scalar(&format!("{SC_JSON} WHERE s.id = $1")).bind(id).fetch_one(pool).await
}

pub async fn sc_sua(pool: &PgPool, id: Uuid, s: ScSua<'_>) -> sqlx::Result<Option<Value>> {
    let n = sqlx::query(
        "UPDATE bk_su_co SET project_id = $2, thoi_diem = $3, mo_ta = $4, nguyen_nhan = $5, muc_do = $6, pham_vi = $7, bien_phap = $8, backup_run_id = $9, \
                phuong_phap_phuc_hoi = $10, phuc_hoi_luc = $11, ket_qua_kiem_tra = $12, du_lieu_mat = $13, trang_thai = $14, nguoi_xu_ly = $15, updated_at = now() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(s.project_id)
    .bind(s.thoi_diem)
    .bind(s.mo_ta.trim())
    .bind(s.nguyen_nhan)
    .bind(s.muc_do)
    .bind(s.pham_vi)
    .bind(s.bien_phap)
    .bind(s.backup_run_id)
    .bind(s.phuong_phap_phuc_hoi)
    .bind(s.phuc_hoi_luc)
    .bind(s.ket_qua_kiem_tra)
    .bind(s.du_lieu_mat)
    .bind(s.trang_thai)
    .bind(s.nguoi_xu_ly)
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 {
        return Ok(None);
    }
    sc_get(pool, id).await
}

pub async fn sc_set_record(pool: &PgPool, id: Uuid, record_id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE bk_su_co SET record_id = $2, updated_at = now() WHERE id = $1").bind(id).bind(record_id).execute(pool).await?;
    Ok(())
}

// ------------------------------------------------------------------------------------------------------------------ tổng quan

/// Số đếm cho thẻ Tổng quan bảo quản (lịch trễ / phương tiện sắp hết tính ở route bằng `digitext_core::bao_quan`).
pub async fn tong_quan(pool: &PgPool, projects: Option<&[Uuid]>) -> sqlx::Result<(i64, i64, i64, i64)> {
    let row = sqlx::query(
        "SELECT (SELECT count(*) FROM bk_ke_hoach k WHERE k.trang_thai = 'cho_duyet' AND ($1::uuid[] IS NULL OR k.project_id = ANY($1))) AS kh_cho_duyet, \
                (SELECT count(*) FROM bk_ke_hoach k WHERE k.trang_thai IN ('da_duyet', 'dang_thuc_hien') AND ($1::uuid[] IS NULL OR k.project_id = ANY($1))) AS kh_dang_chay, \
                (SELECT count(*) FROM bk_loi l WHERE l.trang_thai IN ('moi', 'da_khac_phuc', 'kiem_lai_loi') AND ($1::uuid[] IS NULL OR l.project_id = ANY($1))) AS loi_mo, \
                (SELECT count(*) FROM bk_su_co s WHERE s.trang_thai <> 'dong' AND ($1::uuid[] IS NULL OR s.project_id IS NULL OR s.project_id = ANY($1))) AS su_co_mo",
    )
    .bind(projects)
    .fetch_one(pool)
    .await?;
    Ok((row.get("kh_cho_duyet"), row.get("kh_dang_chay"), row.get("loi_mo"), row.get("su_co_mo")))
}
