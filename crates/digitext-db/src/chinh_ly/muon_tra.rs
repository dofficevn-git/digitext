// Copyright (c) 2026 DigiText. All rights reserved.
//! MƯỢN – TRẢ hồ sơ giấy CÓ DUYỆT (0034, 27/11/2026 — đợt 3 đối chiếu Digifile TT05, nhóm 4A). Máy trạng thái ở
//! `digitext_core::chinh_ly::muon_tra`; ở đây là các bước ghi (lập phiếu nhiều hồ sơ, duyệt / từ chối lẻ hoặc cả phiếu, giao,
//! gia hạn, trả, huỷ), nhắc hạn, tổng quan. Ghi THẲNG SQL (không qua `apply_op`) vì các cột duyệt / giao / gia hạn là cột máy chủ
//! (`spec::is_derived`); đường `apply_op` (REST /cl/rows và push offline) bị `sync::apply_inner` chặn đổi trạng thái trái bảng chuyển.
use chrono::{Datelike, NaiveDate, Utc};
use digitext_core::chinh_ly::muon_tra as mt;
use serde_json::{json, Value};
use sqlx::{PgPool, Row as _};
use uuid::Uuid;

/// Một dòng phiếu kèm hồ sơ và tên người (cùng dạng `thong_ke::phieu_muon`): to_jsonb(m) + tên hồ sơ / phông / người.
pub const PHIEU_JSON: &str = "SELECT to_jsonb(m) || jsonb_build_object('so_tam', h.so_tam, 'ho_so_so', h.ho_so_so, 'ma_ho_so', h.ma_ho_so, 'tieu_de', h.tieu_de, 'hop_so', h.hop_so, \
        'ten_phong', h.ten_phong, 'ten_phong_ban', h.ten_phong_ban, 'phong_id', h.phong_id, 'qua_han', (m.trang_thai = 'dang_muon' AND m.ngay_du_kien_tra < CURRENT_DATE), \
        'nguoi_lap_ten', u.display_name, 'nguoi_duyet_ten', ud.display_name, 'giao_boi_ten', ug.display_name) AS row \
     FROM cl_muon_tra m JOIN cl_ho_so_v h ON h.id = m.ho_so_id LEFT JOIN users u ON u.id = m.nguoi_lap_phieu \
     LEFT JOIN users ud ON ud.id = m.nguoi_duyet LEFT JOIN users ug ON ug.id = m.giao_boi";

fn rows_json(rows: Vec<sqlx::postgres::PgRow>) -> Vec<Value> {
    rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect()
}

pub async fn get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Value>> {
    let r = sqlx::query(&format!("{PHIEU_JSON} WHERE m.id = $1 AND m.deleted_at IS NULL")).bind(id).fetch_optional(pool).await?;
    Ok(r.and_then(|r| r.try_get::<Value, _>("row").ok()))
}

/// Mọi dòng của một phiếu (cùng `nhom_id`).
pub async fn nhom(pool: &PgPool, nhom_id: Uuid) -> sqlx::Result<Vec<Value>> {
    Ok(rows_json(sqlx::query(&format!("{PHIEU_JSON} WHERE m.nhom_id = $1 AND m.deleted_at IS NULL ORDER BY h.so_tam COLLATE vi_so")).bind(nhom_id).fetch_all(pool).await?))
}

pub struct LapPhieu<'a> {
    pub project_id: Uuid,
    pub ho_so_ids: &'a [Uuid],
    pub nguoi_muon: &'a str,
    pub don_vi: Option<&'a str>,
    pub dien_thoai: Option<&'a str>,
    pub email: Option<&'a str>,
    pub noi_dung: Option<&'a str>,
    pub ngay_muon: Option<NaiveDate>,
    pub ngay_du_kien_tra: Option<NaiveDate>,
    pub ghi_chu: Option<&'a str>,
    /// `cho_duyet` (mặc định) hoặc `dang_muon` (lập và giao luôn — người lập có `loan.approve`).
    pub trang_thai: &'a str,
    pub nguoi_lap: Uuid,
}

/// Lập một phiếu cho nhiều hồ sơ: một `nhom_id`, một `so_phieu` "MT-{năm}-{stt}", mỗi hồ sơ một dòng.
/// `Ok(Err(lý do))` khi dữ liệu không hợp lệ; trả các dòng vừa tạo.
pub async fn lap(pool: &PgPool, p: LapPhieu<'_>) -> sqlx::Result<Result<(Uuid, String, Vec<Value>), String>> {
    if p.ho_so_ids.is_empty() {
        return Ok(Err("Chưa chọn hồ sơ nào".into()));
    }
    if p.nguoi_muon.trim().is_empty() {
        return Ok(Err("Thiếu tên người mượn".into()));
    }
    if !mt::hop_le(p.trang_thai) || !matches!(p.trang_thai, mt::CHO_DUYET | mt::DANG_MUON) {
        return Ok(Err("Trạng thái lập phiếu phải là chờ duyệt hoặc đang mượn".into()));
    }
    let hom_nay = crate::nop_luu::hom_nay_vn();
    let ngay_muon = p.ngay_muon.unwrap_or(hom_nay);
    let han = p.ngay_du_kien_tra.unwrap_or(ngay_muon + chrono::Duration::days(7));
    if han < ngay_muon {
        return Ok(Err("Hạn trả phải từ ngày mượn trở đi".into()));
    }
    let mut ids: Vec<Uuid> = p.ho_so_ids.to_vec();
    ids.sort();
    ids.dedup();
    let mut tx = pool.begin().await?;
    let co: Vec<(Uuid, Option<String>, bool)> = sqlx::query_as(
        "SELECT h.id, h.tieu_de, EXISTS (SELECT 1 FROM cl_muon_tra m WHERE m.ho_so_id = h.id AND m.trang_thai = 'dang_muon' AND m.deleted_at IS NULL) \
         FROM cl_ho_so h WHERE h.id = ANY($1) AND h.project_id = $2 AND h.deleted_at IS NULL",
    )
    .bind(&ids)
    .bind(p.project_id)
    .fetch_all(&mut *tx)
    .await?;
    if co.len() != ids.len() {
        return Ok(Err("Có hồ sơ không thuộc dự án hoặc đã xoá".into()));
    }
    if p.trang_thai == mt::DANG_MUON {
        if let Some((_, td, _)) = co.iter().find(|(_, _, dang)| *dang) {
            return Ok(Err(format!("Hồ sơ \"{}\" đang có phiếu mượn chưa trả", td.clone().unwrap_or_default())));
        }
    }
    let nam = ngay_muon.year();
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('cl_muon_tra:' || $1::text || ':' || $2::text, 0))")
        .bind(p.project_id.to_string())
        .bind(nam.to_string())
        .execute(&mut *tx)
        .await?;
    let stt: i64 = sqlx::query_scalar("SELECT count(DISTINCT so_phieu) + 1 FROM cl_muon_tra WHERE project_id = $1 AND so_phieu LIKE $2")
        .bind(p.project_id)
        .bind(format!("MT-{nam}-%"))
        .fetch_one(&mut *tx)
        .await?;
    let so_phieu = mt::so_phieu(nam, stt);
    let nhom_id = Uuid::now_v7();
    let giao = p.trang_thai == mt::DANG_MUON;
    for id in &ids {
        sqlx::query(
            "INSERT INTO cl_muon_tra (id, project_id, ho_so_id, nguoi_muon, don_vi, dien_thoai, email, noi_dung, ngay_muon, ngay_du_kien_tra, trang_thai, \
                nguoi_lap_phieu, ghi_chu, nhom_id, so_phieu, created_by, updated_by, nguoi_duyet, duyet_at, giao_boi, giao_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $12, $12, \
                CASE WHEN $16 THEN $12 END, CASE WHEN $16 THEN now() END, CASE WHEN $16 THEN $12 END, CASE WHEN $16 THEN now() END)",
        )
        .bind(Uuid::now_v7())
        .bind(p.project_id)
        .bind(id)
        .bind(p.nguoi_muon.trim())
        .bind(p.don_vi.map(str::trim).filter(|s| !s.is_empty()))
        .bind(p.dien_thoai.map(str::trim).filter(|s| !s.is_empty()))
        .bind(p.email.map(str::trim).filter(|s| !s.is_empty()))
        .bind(p.noi_dung.map(str::trim).filter(|s| !s.is_empty()))
        .bind(ngay_muon)
        .bind(han)
        .bind(p.trang_thai)
        .bind(p.nguoi_lap)
        .bind(p.ghi_chu.map(str::trim).filter(|s| !s.is_empty()))
        .bind(nhom_id)
        .bind(&so_phieu)
        .bind(giao)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    let rows = nhom(pool, nhom_id).await?;
    Ok(Ok((nhom_id, so_phieu, rows)))
}

/// Tham số kèm theo một bước chuyển: lý do (từ chối / huỷ), ngày (giao / trả).
#[derive(Default)]
pub struct Buoc<'a> {
    pub ly_do: Option<&'a str>,
    pub ngay: Option<NaiveDate>,
}

/// Chuyển một dòng phiếu sang `den` theo bảng `mt::chuyen_duoc`. `Ok(Err(lý do))` khi trái bảng / thiếu lý do / hồ sơ đang mượn.
pub async fn chuyen(pool: &PgPool, id: Uuid, den: &str, by: Uuid, b: Buoc<'_>) -> sqlx::Result<Result<Value, String>> {
    let mut tx = pool.begin().await?;
    let r = sqlx::query("SELECT trang_thai, ho_so_id, ngay_du_kien_tra FROM cl_muon_tra WHERE id = $1 AND deleted_at IS NULL FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    let Some(r) = r else { return Ok(Err("Không thấy phiếu mượn".into())) };
    let tu: String = r.try_get("trang_thai")?;
    if !mt::chuyen_duoc(&tu, den) {
        return Ok(Err(format!("Phiếu đang ở trạng thái \"{}\", không chuyển sang \"{}\" được", mt::ten_trang_thai(&tu), mt::ten_trang_thai(den))));
    }
    let ly_do = b.ly_do.map(str::trim).filter(|s| !s.is_empty());
    if matches!(den, mt::TU_CHOI | mt::HUY) && ly_do.is_none() {
        return Ok(Err("Cần ghi lý do".into()));
    }
    let hom_nay = crate::nop_luu::hom_nay_vn();
    let ngay = b.ngay.unwrap_or(hom_nay);
    let res = match den {
        mt::DA_DUYET => sqlx::query("UPDATE cl_muon_tra SET trang_thai = 'da_duyet', nguoi_duyet = $2, duyet_at = now(), ly_do_tu_choi = NULL, updated_by = $2, updated_at = now(), ver = nextval('cl_ver_seq') WHERE id = $1").bind(id).bind(by).execute(&mut *tx).await,
        mt::TU_CHOI => sqlx::query("UPDATE cl_muon_tra SET trang_thai = 'tu_choi', nguoi_duyet = $2, duyet_at = now(), ly_do_tu_choi = $3, updated_by = $2, updated_at = now(), ver = nextval('cl_ver_seq') WHERE id = $1").bind(id).bind(by).bind(ly_do).execute(&mut *tx).await,
        mt::HUY => sqlx::query("UPDATE cl_muon_tra SET trang_thai = 'huy', ly_do_tu_choi = $3, updated_by = $2, updated_at = now(), ver = nextval('cl_ver_seq') WHERE id = $1").bind(id).bind(by).bind(ly_do).execute(&mut *tx).await,
        mt::DANG_MUON => {
            // Giao: ngày mượn thực = ngày giao; hạn trả giữ nếu còn sau ngày giao, không thì + 7 ngày.
            sqlx::query(
                "UPDATE cl_muon_tra SET trang_thai = 'dang_muon', giao_boi = $2, giao_at = now(), ngay_muon = $3, \
                    ngay_du_kien_tra = CASE WHEN ngay_du_kien_tra IS NULL OR ngay_du_kien_tra < $3 THEN $3 + 7 ELSE ngay_du_kien_tra END, \
                    updated_by = $2, updated_at = now(), ver = nextval('cl_ver_seq') WHERE id = $1",
            )
            .bind(id)
            .bind(by)
            .bind(ngay)
            .execute(&mut *tx)
            .await
        }
        mt::DA_TRA => sqlx::query("UPDATE cl_muon_tra SET trang_thai = 'da_tra', ngay_thuc_te_tra = $3, updated_by = $2, updated_at = now(), ver = nextval('cl_ver_seq') WHERE id = $1").bind(id).bind(by).bind(ngay).execute(&mut *tx).await,
        _ => return Ok(Err("Trạng thái đích không hợp lệ".into())),
    };
    match res {
        Ok(_) => {}
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") => return Ok(Err("Hồ sơ đang có phiếu mượn khác chưa trả".into())),
        Err(e) => return Err(e),
    }
    tx.commit().await?;
    Ok(Ok(get(pool, id).await?.unwrap_or_default()))
}

/// Duyệt / từ chối CẢ PHIẾU: mọi dòng còn `cho_duyet` của nhóm. Trả số dòng đã chuyển.
pub async fn chuyen_nhom(pool: &PgPool, nhom_id: Uuid, den: &str, by: Uuid, ly_do: Option<&str>) -> sqlx::Result<Result<usize, String>> {
    if !matches!(den, mt::DA_DUYET | mt::TU_CHOI | mt::HUY) {
        return Ok(Err("Cả phiếu chỉ duyệt, từ chối hoặc huỷ".into()));
    }
    let ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM cl_muon_tra WHERE nhom_id = $1 AND trang_thai = 'cho_duyet' AND deleted_at IS NULL ORDER BY created_at")
        .bind(nhom_id)
        .fetch_all(pool)
        .await?;
    if ids.is_empty() {
        return Ok(Err("Phiếu không còn dòng nào chờ duyệt".into()));
    }
    let mut n = 0;
    for id in ids {
        if chuyen(pool, id, den, by, Buoc { ly_do, ngay: None }).await?.is_ok() {
            n += 1;
        }
    }
    Ok(Ok(n))
}

/// Gia hạn: chỉ khi đang mượn, hạn mới sau hạn hiện tại, chưa quá `GIA_HAN_TOI_DA` lần; giữ `han_tra_goc`, ghi `gia_han_log`.
pub async fn gia_han(pool: &PgPool, id: Uuid, by: Uuid, by_ten: &str, han_moi: NaiveDate, ly_do: Option<&str>) -> sqlx::Result<Result<Value, String>> {
    let mut tx = pool.begin().await?;
    let r = sqlx::query("SELECT trang_thai, ngay_du_kien_tra, so_lan_gia_han FROM cl_muon_tra WHERE id = $1 AND deleted_at IS NULL FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    let Some(r) = r else { return Ok(Err("Không thấy phiếu mượn".into())) };
    let tt: String = r.try_get("trang_thai")?;
    if tt != mt::DANG_MUON {
        return Ok(Err("Chỉ gia hạn phiếu đang mượn".into()));
    }
    let han_cu: Option<NaiveDate> = r.try_get("ngay_du_kien_tra")?;
    let so_lan: i32 = r.try_get("so_lan_gia_han")?;
    if so_lan >= mt::GIA_HAN_TOI_DA {
        return Ok(Err(format!("Đã gia hạn {so_lan} lần (tối đa {}), đề nghị trả rồi lập phiếu mới", mt::GIA_HAN_TOI_DA)));
    }
    if let Some(c) = han_cu {
        if han_moi <= c {
            return Ok(Err(format!("Hạn mới phải sau hạn hiện tại ({})", c.format("%d/%m/%Y"))));
        }
    }
    let log = json!({ "tu": han_cu, "den": han_moi, "lyDo": ly_do.map(str::trim).filter(|s| !s.is_empty()), "boi": by, "boiTen": by_ten, "luc": Utc::now() });
    sqlx::query(
        "UPDATE cl_muon_tra SET han_tra_goc = COALESCE(han_tra_goc, ngay_du_kien_tra), ngay_du_kien_tra = $2, so_lan_gia_han = so_lan_gia_han + 1, \
            gia_han_log = gia_han_log || $3::jsonb, nhac_lan_cuoi = NULL, updated_by = $4, updated_at = now(), ver = nextval('cl_ver_seq') WHERE id = $1",
    )
    .bind(id)
    .bind(han_moi)
    .bind(json!([log]))
    .bind(by)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Ok(get(pool, id).await?.unwrap_or_default()))
}

/// Tổng quan một dự án: chờ duyệt, chờ giao, đang mượn, quá hạn, sắp hạn (≤ NHAC_TRUOC_NGAY), đã trả 30 ngày.
pub async fn tong_quan(pool: &PgPool, project: Uuid) -> sqlx::Result<Value> {
    let r = sqlx::query(
        "SELECT count(*) FILTER (WHERE trang_thai = 'cho_duyet') AS cho_duyet, count(*) FILTER (WHERE trang_thai = 'da_duyet') AS cho_giao, \
                count(*) FILTER (WHERE trang_thai = 'dang_muon') AS dang_muon, \
                count(*) FILTER (WHERE trang_thai = 'dang_muon' AND ngay_du_kien_tra < CURRENT_DATE) AS qua_han, \
                count(*) FILTER (WHERE trang_thai = 'dang_muon' AND ngay_du_kien_tra >= CURRENT_DATE AND ngay_du_kien_tra <= CURRENT_DATE + $2) AS sap_han, \
                count(*) FILTER (WHERE trang_thai = 'da_tra' AND updated_at >= now() - interval '30 days') AS da_tra_30_ngay, \
                count(DISTINCT nhom_id) FILTER (WHERE trang_thai = 'cho_duyet') AS phieu_cho_duyet \
         FROM cl_muon_tra WHERE project_id = $1 AND deleted_at IS NULL",
    )
    .bind(project)
    .bind(mt::NHAC_TRUOC_NGAY as i32)
    .fetch_one(pool)
    .await?;
    Ok(json!({
        "choDuyet": r.try_get::<i64, _>("cho_duyet")?, "phieuChoDuyet": r.try_get::<i64, _>("phieu_cho_duyet")?, "choGiao": r.try_get::<i64, _>("cho_giao")?,
        "dangMuon": r.try_get::<i64, _>("dang_muon")?, "quaHan": r.try_get::<i64, _>("qua_han")?, "sapHan": r.try_get::<i64, _>("sap_han")?,
        "daTra30Ngay": r.try_get::<i64, _>("da_tra_30_ngay")?,
    }))
}

/// Nhắc hạn mượn (chạy mỗi giờ, idempotent theo ngày): sắp hạn (còn đúng NHAC_TRUOC_NGAY ngày) nhắc một lần; quá hạn nhắc lại mỗi
/// NHAC_QUA_HAN_MOI_NGAY ngày. Người nhận: quản trị dự án + người lập phiếu. Trả số thông báo đã chèn.
pub async fn nhac(pool: &PgPool, hom_nay: NaiveDate) -> sqlx::Result<u64> {
    let rows = sqlx::query(
        "SELECT m.id, m.project_id, m.so_phieu, m.nguoi_muon, m.ngay_du_kien_tra, m.nguoi_lap_phieu, COALESCE(h.tieu_de, '') AS tieu_de \
         FROM cl_muon_tra m JOIN cl_ho_so h ON h.id = m.ho_so_id \
         WHERE m.trang_thai = 'dang_muon' AND m.deleted_at IS NULL AND m.ngay_du_kien_tra IS NOT NULL \
           AND ((m.ngay_du_kien_tra < $1 AND (m.nhac_lan_cuoi IS NULL OR m.nhac_lan_cuoi <= $1 - $3)) \
             OR (m.ngay_du_kien_tra = $1 + $2 AND (m.nhac_lan_cuoi IS NULL OR m.nhac_lan_cuoi < $1))) \
         ORDER BY m.project_id, m.ngay_du_kien_tra LIMIT 2000",
    )
    .bind(hom_nay)
    .bind(mt::NHAC_TRUOC_NGAY as i32)
    .bind(mt::NHAC_QUA_HAN_MOI_NGAY as i32)
    .fetch_all(pool)
    .await?;
    let mut n = 0u64;
    let mut admins: std::collections::HashMap<Uuid, Vec<Uuid>> = std::collections::HashMap::new();
    for r in rows {
        let id: Uuid = r.try_get("id")?;
        let project: Uuid = r.try_get("project_id")?;
        let so_phieu: Option<String> = r.try_get("so_phieu")?;
        let nguoi_muon: String = r.try_get("nguoi_muon")?;
        let han: NaiveDate = r.try_get("ngay_du_kien_tra")?;
        let nguoi_lap: Option<Uuid> = r.try_get("nguoi_lap_phieu")?;
        let tieu_de: String = r.try_get("tieu_de")?;
        let qua = han < hom_nay;
        let kind = if qua { "loan_overdue" } else { "loan_due_soon" };
        let msg = if qua {
            format!("Phiếu mượn {} ({}) quá hạn trả {} ngày — hồ sơ \"{}\", hạn {}", so_phieu.clone().unwrap_or_else(|| "—".into()), nguoi_muon, (hom_nay - han).num_days(), tieu_de, han.format("%d/%m/%Y"))
        } else {
            format!("Phiếu mượn {} ({}) đến hạn trả ngày {} — hồ sơ \"{}\"", so_phieu.clone().unwrap_or_else(|| "—".into()), nguoi_muon, han.format("%d/%m/%Y"), tieu_de)
        };
        let ref_ = format!("{id}:{hom_nay}");
        if !admins.contains_key(&project) {
            admins.insert(project, crate::projects::f_re(pool, project).await?);
        }
        let mut nguoi: Vec<Uuid> = admins[&project].clone();
        if let Some(u) = nguoi_lap {
            if !nguoi.contains(&u) {
                nguoi.push(u);
            }
        }
        for u in nguoi {
            if crate::notifications::f_1v8_ref(pool, u, kind, &msg, &ref_).await? {
                n += 1;
            }
        }
        sqlx::query("UPDATE cl_muon_tra SET nhac_lan_cuoi = $2 WHERE id = $1").bind(id).bind(hom_nay).execute(pool).await?;
    }
    Ok(n)
}
