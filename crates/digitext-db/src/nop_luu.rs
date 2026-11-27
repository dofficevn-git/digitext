// Copyright (c) 2026 DigiText. All rights reserved.
//! Nộp hồ sơ vào LƯU TRỮ HIỆN HÀNH (lưu trữ cơ quan) — Điều 19 TT 05/2025/TT-BNV, migration 0031 (26/11/2026).
//!
//! Một phiếu (`lt_nop_luu`) = một lần một phòng ban nộp một lô hồ sơ. Trạng thái nộp lưu nằm trên `cl_ho_so`
//! (`nop_luu_trang_thai`, `nop_luu_id`, `khoa_luc`) do ĐÚNG module này ghi — ba cột đó là `is_derived` với bộ máy sync nên bản
//! sao offline nhận qua pull mà không đẩy ngược được; `sync::apply_inner` từ chối sửa hồ sơ đã nộp (`locked`).
//!
//! Hồ sơ:  NULL ──nộp──► cho_tiep_nhan ──tiếp nhận (nhận)──► da_nop (khoa_luc = now)
//!                           │ rút → NULL        │ tiếp nhận (trả lại) / trả cả phiếu → tra_lai ──nộp lại──► phiếu mới
//! Phiếu:  cho_tiep_nhan ──► da_tiep_nhan | tra_lai | rut
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use digitext_core::luu_tru;
use serde_json::Value;
use sqlx::{PgPool, Row as _};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PhieuRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub phong_id: Uuid,
    pub phong_ban_id: Option<Uuid>,
    pub ma_phieu: String,
    pub nguoi_nop: Option<Uuid>,
    pub ghi_chu: Option<String>,
    pub trang_thai: String,
    pub tiep_nhan_boi: Option<Uuid>,
    pub tiep_nhan_luc: Option<DateTime<Utc>>,
    pub ly_do_tra: Option<String>,
    pub so_ho_so: i32,
    pub so_van_ban: Option<i32>,
    pub so_trang: Option<i32>,
    pub bien_ban_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

const COLS: &str = "id, project_id, phong_id, phong_ban_id, ma_phieu, nguoi_nop, ghi_chu, trang_thai, tiep_nhan_boi, tiep_nhan_luc, \
     ly_do_tra, so_ho_so, so_van_ban, so_trang, bien_ban_id, created_at, updated_at";

/// Ngày hôm nay theo giờ Việt Nam (máy chủ chạy UTC).
pub fn hom_nay_vn() -> NaiveDate {
    Utc::now().with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00")).date_naive()
}

pub struct LapPhieu<'a> {
    pub project_id: Uuid,
    pub phong_ban_id: Option<Uuid>,
    pub nguoi_nop: Uuid,
    pub nguoi_nop_ten: &'a str,
    pub ghi_chu: Option<&'a str>,
    pub ho_so_ids: &'a [Uuid],
}

/// Lập phiếu nộp lưu. `Ok(Err(lý do))` khi dữ liệu không hợp lệ (hồ sơ không có / khác dự án / đang chờ ở phiếu khác /
/// đã nộp), `Err` chỉ khi CSDL lỗi.
pub async fn lap(pool: &PgPool, p: LapPhieu<'_>) -> sqlx::Result<Result<PhieuRow, String>> {
    if p.ho_so_ids.is_empty() {
        return Ok(Err("Chưa chọn hồ sơ nào để nộp".into()));
    }
    let mut tx = pool.begin().await?;
    // Khoá theo dự án để số phiếu trong năm không trùng khi hai người nộp cùng lúc. Câu SQL cố ý KHÁC chữ với câu của
    // records::create (bind uuid): sqlx cache prepared statement theo đúng chuỗi SQL trên từng kết nối, trùng chữ mà khác
    // kiểu tham số thì "incorrect binary data format in bind parameter 1".
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('nop_luu:' || $1::text, 0))").bind(p.project_id.to_string()).execute(&mut *tx).await?;
    let hs: Vec<(Uuid, Uuid, Option<Uuid>, Option<String>, Option<DateTime<Utc>>, String)> = sqlx::query_as(
        "SELECT id, phong_id, phong_ban_id, nop_luu_trang_thai, deleted_at, so_tam FROM cl_ho_so WHERE id = ANY($1) AND project_id = $2 FOR UPDATE",
    )
    .bind(p.ho_so_ids)
    .bind(p.project_id)
    .fetch_all(&mut *tx)
    .await?;
    if hs.len() != p.ho_so_ids.len() {
        return Ok(Err("Có hồ sơ không tồn tại hoặc thuộc dự án khác".into()));
    }
    let mut phong_id: Option<Uuid> = None;
    for (_, ph, pb, tt, da_xoa, so_tam) in &hs {
        if da_xoa.is_some() {
            return Ok(Err(format!("Hồ sơ số tạm {so_tam} đã bị xoá")));
        }
        match tt.as_deref() {
            Some("cho_tiep_nhan") => return Ok(Err(format!("Hồ sơ số tạm {so_tam} đang chờ tiếp nhận ở phiếu khác"))),
            Some("da_nop") => return Ok(Err(format!("Hồ sơ số tạm {so_tam} đã nộp lưu rồi"))),
            _ => {}
        }
        if let (Some(muon), Some(co)) = (p.phong_ban_id, pb) {
            if muon != *co {
                return Ok(Err(format!("Hồ sơ số tạm {so_tam} thuộc phòng ban khác")));
            }
        }
        match phong_id {
            None => phong_id = Some(*ph),
            Some(x) if x != *ph => return Ok(Err("Các hồ sơ trong một phiếu phải cùng một phông".into())),
            _ => {}
        }
    }
    let phong_id = phong_id.expect("có ít nhất một hồ sơ");
    // Phòng ban của phiếu: tham số, hoặc của hồ sơ đầu (mọi hồ sơ cùng phòng mới nộp một phiếu — kiểm ở trên nếu có tham số).
    let phong_ban_id = p.phong_ban_id.or_else(|| hs.iter().find_map(|h| h.2));
    let nam = hom_nay_vn().year();
    let stt: i64 = sqlx::query_scalar("SELECT count(*) + 1 FROM lt_nop_luu WHERE project_id = $1 AND extract(year FROM created_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::int = $2")
        .bind(p.project_id)
        .bind(nam)
        .fetch_one(&mut *tx)
        .await?;
    let ma_dv: Option<String> = match phong_ban_id {
        Some(pb) => sqlx::query_scalar("SELECT NULLIF(btrim(ma_don_vi), '') FROM cl_phong_ban WHERE id = $1").bind(pb).fetch_optional(&mut *tx).await?.flatten(),
        None => None,
    };
    let ma_phieu = format!("{stt:03}/NL-{}.{nam}", ma_dv.as_deref().unwrap_or("CQ"));
    let id = Uuid::now_v7();
    let (so_vb, so_trang): (Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM cl_van_ban v WHERE v.ho_so_id = ANY($1) AND v.deleted_at IS NULL), \
                (SELECT sum(so_luong_trang)::bigint FROM cl_ho_so WHERE id = ANY($1))",
    )
    .bind(p.ho_so_ids)
    .fetch_one(&mut *tx)
    .await?;
    let row = sqlx::query_as::<_, PhieuRow>(&format!(
        "INSERT INTO lt_nop_luu (id, project_id, phong_id, phong_ban_id, ma_phieu, nguoi_nop, ghi_chu, so_ho_so, so_van_ban, so_trang) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING {COLS}"
    ))
    .bind(id)
    .bind(p.project_id)
    .bind(phong_id)
    .bind(phong_ban_id)
    .bind(&ma_phieu)
    .bind(p.nguoi_nop)
    .bind(p.ghi_chu)
    .bind(p.ho_so_ids.len() as i32)
    .bind(so_vb.map(|n| n as i32))
    .bind(so_trang.map(|n| n as i32))
    .fetch_one(&mut *tx)
    .await?;
    for (i, h) in p.ho_so_ids.iter().enumerate() {
        sqlx::query("INSERT INTO lt_nop_luu_ho_so (id, nop_luu_id, ho_so_id, stt) VALUES ($1, $2, $3, $4)")
            .bind(Uuid::now_v7())
            .bind(id)
            .bind(h)
            .bind(i as i32 + 1)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("UPDATE cl_ho_so SET nop_luu_trang_thai = 'cho_tiep_nhan', nop_luu_id = $2, updated_by = $3 WHERE id = ANY($1)")
        .bind(p.ho_so_ids)
        .bind(id)
        .bind(p.nguoi_nop)
        .execute(&mut *tx)
        .await?;
    ghi_lich_su(&mut tx, p.project_id, phong_id, p.ho_so_ids, "nop_luu", p.nguoi_nop, p.nguoi_nop_ten, &format!("Nộp lưu phiếu {ma_phieu}")).await?;
    tx.commit().await?;
    Ok(Ok(row))
}

async fn ghi_lich_su(
    tx: &mut sqlx::PgConnection,
    project_id: Uuid,
    phong_id: Uuid,
    ho_so_ids: &[Uuid],
    action: &str,
    actor: Uuid,
    actor_name: &str,
    title: &str,
) -> sqlx::Result<()> {
    for h in ho_so_ids {
        crate::chinh_ly::lich_su::insert(
            tx,
            crate::chinh_ly::lich_su::NewEntry {
                project_id,
                entity: "ho_so",
                entity_id: *h,
                ho_so_id: *h,
                phong_id: Some(phong_id),
                action,
                actor_id: Some(actor),
                actor_name: Some(actor_name),
                at_client: None,
                source: "online",
                title: Some(title),
                diffs: vec![],
                n_new: 0,
                n_edited: 0,
            },
        )
        .await?;
    }
    Ok(())
}

pub async fn get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<PhieuRow>> {
    sqlx::query_as::<_, PhieuRow>(&format!("SELECT {COLS} FROM lt_nop_luu WHERE id = $1")).bind(id).fetch_optional(pool).await
}

/// Phiếu dạng JSON kèm tên phòng ban, người nộp, người tiếp nhận, mã phông.
pub async fn get_json(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Value>> {
    let r = sqlx::query(&format!("SELECT to_jsonb(x) AS row FROM ({}) x WHERE x.id = $1", PHIEU_JSON)).bind(id).fetch_optional(pool).await?;
    Ok(r.and_then(|r| r.try_get::<Value, _>("row").ok()))
}

const PHIEU_JSON: &str = "SELECT n.*, pb.ten AS ten_phong_ban, pb.ma_don_vi, p.ma_du_an AS ma_phong, p.ten AS ten_phong, \
        un.display_name AS nguoi_nop_ten, ut.display_name AS tiep_nhan_ten, \
        (SELECT count(*) FROM lt_nop_luu_ho_so i WHERE i.nop_luu_id = n.id AND i.ket_qua = 'nhan') AS so_nhan, \
        (SELECT count(*) FROM lt_nop_luu_ho_so i WHERE i.nop_luu_id = n.id AND i.ket_qua = 'tra_lai') AS so_tra_lai, \
        r.so_bien_ban \
     FROM lt_nop_luu n \
     JOIN cl_phong p ON p.id = n.phong_id \
     LEFT JOIN cl_phong_ban pb ON pb.id = n.phong_ban_id \
     LEFT JOIN users un ON un.id = n.nguoi_nop \
     LEFT JOIN users ut ON ut.id = n.tiep_nhan_boi \
     LEFT JOIN records r ON r.id = n.bien_ban_id";

#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    pub project_id: Uuid,
    pub trang_thai: Option<String>,
    pub phong_ban_id: Option<Uuid>,
    pub nguoi_nop: Option<Uuid>,
    pub nam: Option<i32>,
    pub limit: i64,
    pub offset: i64,
    /// Phạm vi phông của người gọi (0038); None = không giới hạn.
    pub phong_ids: Option<Vec<Uuid>>,
}

pub async fn list(pool: &PgPool, f: &ListFilter) -> sqlx::Result<(Vec<Value>, i64)> {
    let dk = |ph: &str| {
        format!(
            "x.project_id = $1 AND ($2::text IS NULL OR x.trang_thai = $2) AND ($3::uuid IS NULL OR x.phong_ban_id = $3) \
             AND ($4::uuid IS NULL OR x.nguoi_nop = $4) AND ($5::int IS NULL OR extract(year FROM x.created_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::int = $5) \
             AND ({ph}::uuid[] IS NULL OR x.phong_id = ANY({ph}))"
        )
    };
    let sql = format!("SELECT to_jsonb(x) AS row FROM ({PHIEU_JSON}) x WHERE {} ORDER BY x.created_at DESC LIMIT $6 OFFSET $7", dk("$8"));
    let rows = sqlx::query(&sql)
        .bind(f.project_id)
        .bind(f.trang_thai.as_deref())
        .bind(f.phong_ban_id)
        .bind(f.nguoi_nop)
        .bind(f.nam)
        .bind(f.limit.clamp(1, 1000))
        .bind(f.offset.max(0))
        .bind(f.phong_ids.as_deref())
        .fetch_all(pool)
        .await?;
    let total: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM lt_nop_luu x WHERE {}", dk("$6")))
        .bind(f.project_id)
        .bind(f.trang_thai.as_deref())
        .bind(f.phong_ban_id)
        .bind(f.nguoi_nop)
        .bind(f.nam)
        .bind(f.phong_ids.as_deref())
        .fetch_one(pool)
        .await?;
    Ok((rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect(), total))
}

/// Hồ sơ trong phiếu (dòng view + kết quả chấm).
pub async fn ho_so_cua(pool: &PgPool, id: Uuid) -> sqlx::Result<Vec<Value>> {
    let rows = sqlx::query(
        "SELECT jsonb_build_object('id', i.id, 'hoSoId', i.ho_so_id, 'stt', i.stt, 'ketQua', i.ket_qua, 'lyDo', i.ly_do, \
                'soTam', v.so_tam, 'hoSoSo', v.ho_so_so, 'maHoSo', v.ma_ho_so, 'tieuDe', v.tieu_de, 'ngayBatDau', v.ngay_bat_dau, \
                'ngayKetThuc', v.ngay_ket_thuc, 'soTo', v.so_to, 'tenThbq', v.ten_thbq, 'soVanBan', v.so_van_ban, \
                'tenPhongBan', v.ten_phong_ban, 'nopLuuTrangThai', v.nop_luu_trang_thai, 'khoaLuc', v.khoa_luc) AS row \
         FROM lt_nop_luu_ho_so i JOIN cl_ho_so_v v ON v.id = i.ho_so_id WHERE i.nop_luu_id = $1 ORDER BY i.stt",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect())
}

/// Người nộp rút phiếu khi còn chờ: hồ sơ về chưa nộp.
pub async fn rut(pool: &PgPool, id: Uuid, user: Uuid, user_name: &str) -> sqlx::Result<Result<PhieuRow, String>> {
    let mut tx = pool.begin().await?;
    let Some(p) = sqlx::query_as::<_, PhieuRow>(&format!("SELECT {COLS} FROM lt_nop_luu WHERE id = $1 FOR UPDATE")).bind(id).fetch_optional(&mut *tx).await? else {
        return Ok(Err("Không có phiếu".into()));
    };
    if p.trang_thai != "cho_tiep_nhan" {
        return Ok(Err("Phiếu không còn ở trạng thái chờ tiếp nhận".into()));
    }
    let ids: Vec<Uuid> = sqlx::query_scalar("SELECT ho_so_id FROM lt_nop_luu_ho_so WHERE nop_luu_id = $1").bind(id).fetch_all(&mut *tx).await?;
    sqlx::query("UPDATE lt_nop_luu_ho_so SET ket_qua = 'tra_lai', ly_do = 'Rút phiếu' WHERE nop_luu_id = $1 AND ket_qua = 'cho'").bind(id).execute(&mut *tx).await?;
    sqlx::query("UPDATE cl_ho_so SET nop_luu_trang_thai = NULL, nop_luu_id = NULL, updated_by = $2 WHERE id = ANY($1) AND nop_luu_id = $3")
        .bind(&ids)
        .bind(user)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let row = sqlx::query_as::<_, PhieuRow>(&format!("UPDATE lt_nop_luu SET trang_thai = 'rut', updated_at = now() WHERE id = $1 RETURNING {COLS}"))
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    ghi_lich_su(&mut tx, p.project_id, p.phong_id, &ids, "rut_nop_luu", user, user_name, &format!("Rút phiếu nộp lưu {}", p.ma_phieu)).await?;
    tx.commit().await?;
    Ok(Ok(row))
}

/// Cán bộ lưu trữ chấm từng hồ sơ trước khi hoàn tất: `nhan` | `tra_lai` (bắt buộc lý do) | `cho` (bỏ chấm).
pub async fn cham(pool: &PgPool, id: Uuid, ho_so_id: Uuid, ket_qua: &str, ly_do: Option<&str>) -> sqlx::Result<Result<(), String>> {
    if !matches!(ket_qua, "nhan" | "tra_lai" | "cho") {
        return Ok(Err("Kết quả phải là nhan | tra_lai | cho".into()));
    }
    if ket_qua == "tra_lai" && ly_do.map(str::trim).filter(|s| !s.is_empty()).is_none() {
        return Ok(Err("Trả lại phải ghi lý do".into()));
    }
    let n = sqlx::query(
        "UPDATE lt_nop_luu_ho_so i SET ket_qua = $3, ly_do = $4 FROM lt_nop_luu n \
         WHERE i.nop_luu_id = n.id AND n.id = $1 AND i.ho_so_id = $2 AND n.trang_thai = 'cho_tiep_nhan'",
    )
    .bind(id)
    .bind(ho_so_id)
    .bind(ket_qua)
    .bind(ly_do.map(str::trim))
    .execute(pool)
    .await?
    .rows_affected();
    Ok(if n > 0 { Ok(()) } else { Err("Phiếu không còn chờ tiếp nhận hoặc hồ sơ không thuộc phiếu".into()) })
}

/// Hoàn tất tiếp nhận: dòng chưa chấm coi là nhận; hồ sơ nhận → `da_nop` + khoá; hồ sơ trả lại → `tra_lai`. Phiếu thành
/// `da_tiep_nhan` khi có ít nhất một hồ sơ nhận, ngược lại `tra_lai`.
pub async fn tiep_nhan(pool: &PgPool, id: Uuid, user: Uuid, user_name: &str, ghi_chu: Option<&str>) -> sqlx::Result<Result<PhieuRow, String>> {
    let mut tx = pool.begin().await?;
    let Some(p) = sqlx::query_as::<_, PhieuRow>(&format!("SELECT {COLS} FROM lt_nop_luu WHERE id = $1 FOR UPDATE")).bind(id).fetch_optional(&mut *tx).await? else {
        return Ok(Err("Không có phiếu".into()));
    };
    if p.trang_thai != "cho_tiep_nhan" {
        return Ok(Err("Phiếu không còn ở trạng thái chờ tiếp nhận".into()));
    }
    sqlx::query("UPDATE lt_nop_luu_ho_so SET ket_qua = 'nhan' WHERE nop_luu_id = $1 AND ket_qua = 'cho'").bind(id).execute(&mut *tx).await?;
    let nhan: Vec<Uuid> = sqlx::query_scalar("SELECT ho_so_id FROM lt_nop_luu_ho_so WHERE nop_luu_id = $1 AND ket_qua = 'nhan'").bind(id).fetch_all(&mut *tx).await?;
    let tra: Vec<Uuid> = sqlx::query_scalar("SELECT ho_so_id FROM lt_nop_luu_ho_so WHERE nop_luu_id = $1 AND ket_qua = 'tra_lai'").bind(id).fetch_all(&mut *tx).await?;
    sqlx::query("UPDATE cl_ho_so SET nop_luu_trang_thai = 'da_nop', khoa_luc = now(), updated_by = $2 WHERE id = ANY($1)")
        .bind(&nhan)
        .bind(user)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE cl_ho_so SET nop_luu_trang_thai = 'tra_lai', updated_by = $2 WHERE id = ANY($1)")
        .bind(&tra)
        .bind(user)
        .execute(&mut *tx)
        .await?;
    let tt = if nhan.is_empty() { "tra_lai" } else { "da_tiep_nhan" };
    let row = sqlx::query_as::<_, PhieuRow>(&format!(
        "UPDATE lt_nop_luu SET trang_thai = $2, tiep_nhan_boi = $3, tiep_nhan_luc = now(), ghi_chu = COALESCE($4, ghi_chu), updated_at = now() \
         WHERE id = $1 RETURNING {COLS}"
    ))
    .bind(id)
    .bind(tt)
    .bind(user)
    .bind(ghi_chu)
    .fetch_one(&mut *tx)
    .await?;
    ghi_lich_su(&mut tx, p.project_id, p.phong_id, &nhan, "tiep_nhan", user, user_name, &format!("Tiếp nhận phiếu {}", p.ma_phieu)).await?;
    ghi_lich_su(&mut tx, p.project_id, p.phong_id, &tra, "tra_lai_nop_luu", user, user_name, &format!("Trả lại từ phiếu {}", p.ma_phieu)).await?;
    tx.commit().await?;
    Ok(Ok(row))
}

/// Trả lại cả phiếu (mọi hồ sơ chưa chấm thành trả lại với cùng lý do) rồi hoàn tất.
pub async fn tra_lai(pool: &PgPool, id: Uuid, user: Uuid, user_name: &str, ly_do: &str) -> sqlx::Result<Result<PhieuRow, String>> {
    let ly_do = ly_do.trim();
    if ly_do.is_empty() {
        return Ok(Err("Trả lại phải ghi lý do".into()));
    }
    sqlx::query("UPDATE lt_nop_luu_ho_so i SET ket_qua = 'tra_lai', ly_do = $2 FROM lt_nop_luu n WHERE i.nop_luu_id = n.id AND n.id = $1 AND n.trang_thai = 'cho_tiep_nhan' AND i.ket_qua <> 'nhan'")
        .bind(id)
        .bind(ly_do)
        .execute(pool)
        .await?;
    sqlx::query("UPDATE lt_nop_luu SET ly_do_tra = $2 WHERE id = $1").bind(id).bind(ly_do).execute(pool).await?;
    tiep_nhan(pool, id, user, user_name, None).await
}

/// Mở khoá ngoại lệ một hồ sơ đã nộp (quyền `luutru.unlock`, API ghi audit). Hồ sơ về chưa nộp để sửa rồi nộp lại.
pub async fn mo_khoa(pool: &PgPool, ho_so_id: Uuid, user: Uuid) -> sqlx::Result<bool> {
    let n = sqlx::query("UPDATE cl_ho_so SET nop_luu_trang_thai = NULL, khoa_luc = NULL, updated_by = $2 WHERE id = $1 AND nop_luu_trang_thai IN ('cho_tiep_nhan', 'da_nop')")
        .bind(ho_so_id)
        .bind(user)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}

pub async fn set_bien_ban(pool: &PgPool, id: Uuid, record_id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE lt_nop_luu SET bien_ban_id = $2, updated_at = now() WHERE id = $1").bind(id).bind(record_id).execute(pool).await?;
    Ok(())
}

/// Hồ sơ của phòng ban còn phải nộp (chưa nộp hoặc bị trả lại), kèm hạn nộp và số ngày còn lại (âm = quá hạn).
/// `loc`: `den_han` (hạn trong 15 ngày tới hoặc hôm nay) | `qua_han` | None (tất cả).
pub async fn ho_so_can_nop(pool: &PgPool, project: Uuid, phong_ban: Option<Uuid>, loc: Option<&str>, limit: i64, offset: i64, phong_ids: Option<&[Uuid]>) -> sqlx::Result<(Vec<Value>, i64)> {
    let hom_nay = hom_nay_vn();
    // Hạn nộp theo cấu hình cơ quan (0039), $5 = mặc định 60; cửa sổ "đến hạn" = mốc nhắc lớn nhất (mặc định 15).
    let han_sql = "COALESCE((SELECT c.han_nop_ngay FROM co_quan_luu_tru c WHERE c.project_id = v.project_id), $5)";
    let cua_so = "COALESCE((SELECT max(m) FROM co_quan_luu_tru c, unnest(c.nhac_truoc_ngay) m WHERE c.project_id = v.project_id), 15)";
    let dk = |ph: &str| {
        format!(
            "v.project_id = $1 AND v.deleted_at IS NULL AND (v.nop_luu_trang_thai IS NULL OR v.nop_luu_trang_thai = 'tra_lai') \
             AND ($2::uuid IS NULL OR v.phong_ban_id = $2) \
             AND ($3::text IS NULL \
                  OR ($3 = 'den_han' AND v.ngay_kt_cuoi IS NOT NULL AND v.ngay_kt_cuoi + {han_sql} BETWEEN $4::date AND $4::date + {cua_so}) \
                  OR ($3 = 'qua_han' AND v.ngay_kt_cuoi IS NOT NULL AND v.ngay_kt_cuoi + {han_sql} < $4::date)) \
             AND ({ph}::uuid[] IS NULL OR v.phong_id = ANY({ph}))"
        )
    };
    let sql = format!(
        "SELECT to_jsonb(v) || jsonb_build_object('han_nop', v.ngay_kt_cuoi + {han_sql}, 'con_ngay', (v.ngay_kt_cuoi + {han_sql}) - $4::date) AS row \
         FROM cl_ho_so_v v WHERE {} ORDER BY v.ngay_kt_cuoi NULLS LAST, v.so_tam COLLATE vi_so LIMIT $6 OFFSET $7",
        dk("$8")
    );
    let han: i32 = luu_tru::HAN_NOP_NGAY as i32;
    let rows = sqlx::query(&sql).bind(project).bind(phong_ban).bind(loc).bind(hom_nay).bind(han).bind(limit.clamp(1, 5000)).bind(offset.max(0)).bind(phong_ids).fetch_all(pool).await?;
    let total: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM cl_ho_so_v v WHERE {}", dk("$6")))
        .bind(project)
        .bind(phong_ban)
        .bind(loc)
        .bind(hom_nay)
        .bind(han)
        .bind(phong_ids)
        .fetch_one(pool)
        .await?;
    Ok((rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect(), total))
}

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TongQuan {
    pub den_han: i64,
    pub qua_han: i64,
    pub chua_nop: i64,
    pub phieu_cho_tiep_nhan: i64,
    pub quyen_sap_het: i64,
}

pub async fn tong_quan(pool: &PgPool, project: Uuid, phong_ban: Option<Uuid>, phong_ids: Option<&[Uuid]>) -> sqlx::Result<TongQuan> {
    let hom_nay = hom_nay_vn();
    let han: i32 = luu_tru::HAN_NOP_NGAY as i32;
    let qsd: i32 = luu_tru::QUYEN_SU_DUNG_NGAY as i32;
    let han_sql = "COALESCE((SELECT c.han_nop_ngay FROM co_quan_luu_tru c WHERE c.project_id = v.project_id), $4)";
    let qsd_sql = "COALESCE((SELECT c.quyen_su_dung_ngay FROM co_quan_luu_tru c WHERE c.project_id = v.project_id), $5)";
    let cua_so = "COALESCE((SELECT max(m) FROM co_quan_luu_tru c, unnest(c.nhac_truoc_ngay) m WHERE c.project_id = v.project_id), 15)";
    let r: (i64, i64, i64, i64, i64) = sqlx::query_as(&format!(
        "SELECT \
            count(*) FILTER (WHERE (v.nop_luu_trang_thai IS NULL OR v.nop_luu_trang_thai = 'tra_lai') AND v.ngay_kt_cuoi + {han_sql} BETWEEN $3::date AND $3::date + {cua_so}), \
            count(*) FILTER (WHERE (v.nop_luu_trang_thai IS NULL OR v.nop_luu_trang_thai = 'tra_lai') AND v.ngay_kt_cuoi + {han_sql} < $3::date), \
            count(*) FILTER (WHERE v.nop_luu_trang_thai IS NULL OR v.nop_luu_trang_thai = 'tra_lai'), \
            (SELECT count(*) FROM lt_nop_luu n WHERE n.project_id = $1 AND n.trang_thai = 'cho_tiep_nhan' AND ($2::uuid IS NULL OR n.phong_ban_id = $2) \
                AND ($6::uuid[] IS NULL OR n.phong_id = ANY($6))), \
            count(*) FILTER (WHERE v.nop_luu_trang_thai = 'da_nop' AND (v.khoa_luc AT TIME ZONE 'Asia/Ho_Chi_Minh')::date + {qsd_sql} BETWEEN $3::date AND $3::date + 7) \
         FROM cl_ho_so_v v WHERE v.project_id = $1 AND v.deleted_at IS NULL AND ($2::uuid IS NULL OR v.phong_ban_id = $2) AND ($6::uuid[] IS NULL OR v.phong_id = ANY($6))"
    ))
    .bind(project)
    .bind(phong_ban)
    .bind(hom_nay)
    .bind(han)
    .bind(qsd)
    .bind(phong_ids)
    .fetch_one(pool)
    .await?;
    Ok(TongQuan { den_han: r.0, qua_han: r.1, chua_nop: r.2, phieu_cho_tiep_nhan: r.3, quyen_sap_het: r.4 })
}

/// Sổ nộp lưu một năm: một dòng một phiếu đã tiếp nhận / trả lại.
pub async fn so_nop_luu(pool: &PgPool, project: Uuid, nam: i32, phong_ban: Option<Uuid>, phong_ids: Option<&[Uuid]>) -> sqlx::Result<Vec<Value>> {
    let sql = format!(
        "SELECT to_jsonb(x) AS row FROM ({PHIEU_JSON}) x \
         WHERE x.project_id = $1 AND x.trang_thai IN ('da_tiep_nhan', 'tra_lai') AND ($3::uuid IS NULL OR x.phong_ban_id = $3) \
           AND ($4::uuid[] IS NULL OR x.phong_id = ANY($4)) \
           AND extract(year FROM COALESCE(x.tiep_nhan_luc, x.created_at) AT TIME ZONE 'Asia/Ho_Chi_Minh')::int = $2 \
         ORDER BY COALESCE(x.tiep_nhan_luc, x.created_at)"
    );
    let rows = sqlx::query(&sql).bind(project).bind(nam).bind(phong_ban).bind(phong_ids).fetch_all(pool).await?;
    Ok(rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect())
}

// ---------------------------------------------------------------------------------------------------- nhắc hạn (worker)

/// Một lượt nhắc theo lịch (worker gọi mỗi giờ; idempotent nhờ `ux_notifications_ref`):
/// 1. hồ sơ chưa nộp, kết thúc từ năm `tu_nam`: còn 15 / 7 / 1 ngày → `nop_luu_nhac_<n>`; quá hạn → `nop_luu_qua_han` theo tuần;
///    người nhận = người lập + người cùng phòng ban;
/// 2. mỗi ngày một dòng tổng hợp cho quản trị dự án (`nop_luu_tong_hop`);
/// 3. quyền sử dụng hồ sơ đã nộp còn 7 / 0 ngày → `quyen_su_dung_het` cho người lập và người nộp.
/// Trả số thông báo đã tạo.
pub async fn quet_nhac(pool: &PgPool, hom_nay: NaiveDate, tu_nam: i32) -> sqlx::Result<u64> {
    let mut tao = 0u64;
    let han: i32 = luu_tru::HAN_NOP_NGAY as i32;
    // Hạn nộp + mốc nhắc theo cấu hình cơ quan (0039); dự án chưa khai dùng mặc định 60 / 15-7-1.
    let moc_du_an = crate::co_quan::moc_nhac_theo_du_an(pool).await?;
    let moc_mac_dinh: Vec<u64> = luu_tru::NHAC_TRUOC_NGAY.to_vec();
    let han_sql = "COALESCE((SELECT c.han_nop_ngay FROM co_quan_luu_tru c WHERE c.project_id = v.project_id), $3)";
    let cua_so = "COALESCE((SELECT max(m) FROM co_quan_luu_tru c, unnest(c.nhac_truoc_ngay) m WHERE c.project_id = v.project_id), 15)";
    // 1. Hồ sơ chưa nộp đã tới cửa sổ nhắc (hạn ≤ hôm nay + mốc lớn nhất).
    let hs: Vec<(Uuid, Uuid, Option<Uuid>, Option<Uuid>, String, String, NaiveDate)> = sqlx::query_as(&format!(
        "SELECT v.id, v.project_id, v.phong_ban_id, v.created_by, v.so_tam, COALESCE(v.tieu_de, ''), v.ngay_kt_cuoi + {han_sql} \
         FROM cl_ho_so_v v \
         WHERE v.deleted_at IS NULL AND (v.nop_luu_trang_thai IS NULL OR v.nop_luu_trang_thai = 'tra_lai') \
           AND v.ngay_kt_cuoi IS NOT NULL AND v.nam_ket_thuc >= $2 AND v.ngay_kt_cuoi + {han_sql} <= $1::date + {cua_so} \
         ORDER BY v.project_id, v.phong_ban_id"
    ))
    .bind(hom_nay)
    .bind(tu_nam)
    .bind(han)
    .fetch_all(pool)
    .await?;
    let mut tong: std::collections::BTreeMap<Uuid, (i64, i64)> = std::collections::BTreeMap::new();
    for (id, project, pb, created_by, so_tam, tieu_de, han_nop) in &hs {
        let moc = moc_du_an.get(project).unwrap_or(&moc_mac_dinh);
        let (kind, ref_, msg) = match luu_tru::tinh_trang_han_voi(*han_nop, hom_nay, moc) {
            luu_tru::TinhTrangHan::QuaHan(n) => {
                tong.entry(*project).or_default().0 += 1;
                let tuan = hom_nay.iso_week();
                (
                    "nop_luu_qua_han".to_string(),
                    format!("ho_so:{id}:qh:{}-{:02}", tuan.year(), tuan.week()),
                    format!("Hồ sơ số tạm {so_tam} «{tieu_de}» đã quá hạn nộp lưu {n} ngày (hạn {}) — Điều 19", han_nop.format("%d/%m/%Y")),
                )
            }
            luu_tru::TinhTrangHan::DenHan(_) => {
                tong.entry(*project).or_default().1 += 1;
                let Some(m) = luu_tru::moc_nhac_voi(*han_nop, hom_nay, moc) else { continue };
                (
                    format!("nop_luu_nhac_{m}"),
                    format!("ho_so:{id}:nhac{m}"),
                    format!("Hồ sơ số tạm {so_tam} «{tieu_de}» còn {m} ngày tới hạn nộp lưu ({})", han_nop.format("%d/%m/%Y")),
                )
            }
            luu_tru::TinhTrangHan::ChuaDen => continue,
        };
        let mut nguoi: Vec<Uuid> = created_by.into_iter().copied().collect();
        if let Some(pb) = pb {
            nguoi.extend(crate::users::cua_phong_ban(pool, *project, *pb).await?);
        }
        nguoi.sort();
        nguoi.dedup();
        for u in nguoi {
            if crate::notifications::f_1v8_ref(pool, u, &kind, &msg, &ref_).await? {
                tao += 1;
            }
        }
    }
    // 2. Tổng hợp ngày cho quản trị dự án.
    for (project, (qua, den)) in tong {
        if qua + den == 0 {
            continue;
        }
        let msg = format!("Lưu trữ hiện hành: {qua} hồ sơ quá hạn nộp lưu, {den} hồ sơ sắp đến hạn");
        let ref_ = format!("du_an:{project}:{}", hom_nay.format("%Y-%m-%d"));
        for u in crate::projects::f_re(pool, project).await? {
            if crate::notifications::f_1v8_ref(pool, u, "nop_luu_tong_hop", &msg, &ref_).await? {
                tao += 1;
            }
        }
    }
    // 3. Quyền sử dụng sắp hết.
    let qsd: i32 = luu_tru::QUYEN_SU_DUNG_NGAY as i32;
    let sap_het: Vec<(Uuid, Option<Uuid>, Option<Uuid>, String, NaiveDate)> = sqlx::query_as(
        "SELECT h.id, h.created_by, n.nguoi_nop, h.so_tam, (h.khoa_luc AT TIME ZONE 'Asia/Ho_Chi_Minh')::date + COALESCE((SELECT c.quyen_su_dung_ngay FROM co_quan_luu_tru c WHERE c.project_id = h.project_id), $2) \
         FROM cl_ho_so h LEFT JOIN lt_nop_luu n ON n.id = h.nop_luu_id \
         WHERE h.deleted_at IS NULL AND h.nop_luu_trang_thai = 'da_nop' AND h.khoa_luc IS NOT NULL \
           AND (h.khoa_luc AT TIME ZONE 'Asia/Ho_Chi_Minh')::date + COALESCE((SELECT c.quyen_su_dung_ngay FROM co_quan_luu_tru c WHERE c.project_id = h.project_id), $2) IN ($1::date, $1::date + 7)",
    )
    .bind(hom_nay)
    .bind(qsd)
    .fetch_all(pool)
    .await?;
    for (id, created_by, nguoi_nop, so_tam, het) in sap_het {
        let con = (het - hom_nay).num_days();
        let msg = if con == 0 {
            format!("Hôm nay hết quyền sử dụng hồ sơ số tạm {so_tam} đã nộp lưu (360 ngày) — từ mai tra cứu qua Yêu cầu khai thác")
        } else {
            format!("Còn {con} ngày là hết quyền sử dụng hồ sơ số tạm {so_tam} đã nộp lưu (360 ngày)")
        };
        let ref_ = format!("ho_so:{id}:qsd{con}");
        let mut nguoi: Vec<Uuid> = created_by.into_iter().chain(nguoi_nop).collect();
        nguoi.sort();
        nguoi.dedup();
        for u in nguoi {
            if crate::notifications::f_1v8_ref(pool, u, "quyen_su_dung_het", &msg, &ref_).await? {
                tao += 1;
            }
        }
    }
    Ok(tao)
}
