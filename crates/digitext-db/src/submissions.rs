// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{DateTime, Datelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tm8 {
    pub id: Uuid,
    pub project_id: Uuid,
    pub created_by: Option<Uuid>,
    /// 0040: cơ quan nộp lưu (danh mục co_quan_nop); None = đợt nộp khai tay / cũ.
    pub co_quan_nop_id: Option<Uuid>,
    pub ma_yeu_cau: String,
    pub org_name: String,
    pub org_code: String,
    pub da_nop_giay: bool,
    pub so_lan_nop: i32,
    pub summary: Option<String>,
    pub fond_code: String,
    pub phuong_an_phan_loai: Option<String>,
    pub cach_thuc: String,
    pub du_kien_thoi_gian: Option<String>,
    pub lien_he: Option<String>,
    pub ghi_chu: Option<String>,
    pub total_pages: Option<i64>,
    pub total_bytes: Option<i64>,
    pub han_nop: Option<DateTime<Utc>>,
    pub status: String,
    pub decision_note: Option<String>,
    pub reconcile_note: Option<String>,
    pub reception_note: Option<String>,
    pub approval_doc: Option<String>,
    pub approval_note: Option<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub received_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tm7 {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub stt: i32,
    pub ma_ho_so: String,
    pub tieu_de: String,
    pub thoi_gian_bat_dau: Option<String>,
    pub thoi_gian_ket_thuc: Option<String>,
    pub so_trang: Option<i32>,
    pub muc_do_tiep_can: Option<String>,
    pub ghi_chu: Option<String>,
    pub review_result: String,
    pub review_note: Option<String>,
    pub created_at: DateTime<Utc>,
    /// 0040: hồ sơ đang có trong hệ thống (nộp cùng hệ thống, Điều 22.4); trùng mã trong cùng đợt (Điều 24).
    pub ho_so_id: Option<Uuid>,
    pub trung_trong_dot: bool,
}
const COLS: &str = "id, project_id, created_by, co_quan_nop_id, ma_yeu_cau, org_name, org_code, da_nop_giay, \
     so_lan_nop, summary, fond_code, phuong_an_phan_loai, cach_thuc, du_kien_thoi_gian, lien_he, \
     ghi_chu, total_pages, total_bytes, han_nop, status, decision_note, reconcile_note, reception_note, \
     approval_doc, approval_note, confirmed_at, received_at, approved_at, completed_at, \
     created_at, updated_at";
const ITEM_COLS: &str = "id, submission_id, stt, ma_ho_so, tieu_de, thoi_gian_bat_dau, \
     thoi_gian_ket_thuc, so_trang, muc_do_tiep_can, ghi_chu, review_result, review_note, \
     created_at, ho_so_id, trung_trong_dot";
pub const LOI_TRUNG_DOT: &str = "Trùng mã hồ sơ với dòng khác trong cùng đợt nộp — tự động không đạt (Điều 24.2)";
pub struct Te0<'a> {
    pub project_id: Uuid,
    pub created_by: Option<Uuid>,
    pub co_quan_nop_id: Option<Uuid>,
    pub org_name: &'a str,
    pub org_code: &'a str,
    pub da_nop_giay: bool,
    pub summary: Option<&'a str>,
    pub fond_code: &'a str,
    pub phuong_an_phan_loai: Option<&'a str>,
    pub cach_thuc: &'a str,
    pub du_kien_thoi_gian: Option<&'a str>,
    pub lien_he: Option<&'a str>,
    pub ghi_chu: Option<&'a str>,
    pub total_pages: Option<i64>,
    pub total_bytes: Option<i64>,
    pub han_nop: Option<DateTime<Utc>>,
}
pub struct Tdv<'a> {
    pub ma_ho_so: &'a str,
    pub tieu_de: &'a str,
    pub thoi_gian_bat_dau: Option<&'a str>,
    pub thoi_gian_ket_thuc: Option<&'a str>,
    pub so_trang: Option<i32>,
    pub muc_do_tiep_can: Option<&'a str>,
    pub ghi_chu: Option<&'a str>,
    pub ho_so_id: Option<Uuid>,
}
pub async fn register(
    r_2ch: &PgPool,
    r_2ck: Te0<'_>,
    items: &[Tdv<'_>],
) -> sqlx::Result<(Tm8, Vec<Tm7>)> {
    let mut r_2cm = r_2ch.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))")
        .bind(r_2ck.project_id)
        .execute(&mut *r_2cm)
        .await?;
    let year = Utc::now().year();
    let r_2cl: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM submissions \
         WHERE project_id = $1 AND date_part('year', created_at) = $2",
    )
    .bind(r_2ck.project_id)
    .bind(year as f64)
    .fetch_one(&mut *r_2cm)
    .await?;
    let so_lan_nop: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM submissions WHERE project_id = $1 AND org_code = $2",
    )
    .bind(r_2ck.project_id)
    .bind(r_2ck.org_code)
    .fetch_one(&mut *r_2cm)
    .await?;
    let ma_yeu_cau = format!(
        "{:02}.{}.{:02}.{}",
        r_2cl + 1,
        r_2ck.org_code,
        so_lan_nop + 1,
        year
    );
    let r_2cj = sqlx::query_as::<_, Tm8>(&format!(
        "INSERT INTO submissions (id, project_id, created_by, ma_yeu_cau, org_name, org_code, \
             da_nop_giay, so_lan_nop, summary, fond_code, phuong_an_phan_loai, cach_thuc, \
             du_kien_thoi_gian, lien_he, ghi_chu, total_pages, total_bytes, han_nop, co_quan_nop_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, \
             $18, $19) \
         RETURNING {COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(r_2ck.project_id)
    .bind(r_2ck.created_by)
    .bind(&ma_yeu_cau)
    .bind(r_2ck.org_name)
    .bind(r_2ck.org_code)
    .bind(r_2ck.da_nop_giay)
    .bind((so_lan_nop + 1) as i32)
    .bind(r_2ck.summary)
    .bind(r_2ck.fond_code)
    .bind(r_2ck.phuong_an_phan_loai)
    .bind(r_2ck.cach_thuc)
    .bind(r_2ck.du_kien_thoi_gian)
    .bind(r_2ck.lien_he)
    .bind(r_2ck.ghi_chu)
    .bind(r_2ck.total_pages)
    .bind(r_2ck.total_bytes)
    .bind(r_2ck.han_nop)
    .bind(r_2ck.co_quan_nop_id)
    .fetch_one(&mut *r_2cm)
    .await?;
    let r_2cg = chen_items(&mut r_2cm, r_2cj.id, 0, &std::collections::HashSet::new(), items).await?;
    r_2cm.commit().await?;
    Ok((r_2cj, r_2cg))
}
/// Chèn dòng mục lục nộp từ `stt_tu + 1`; mã hồ sơ đã có trong đợt (hoặc lặp trong chính lô) → `trung_trong_dot`, không đạt sẵn.
async fn chen_items(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    submission_id: Uuid,
    stt_tu: i32,
    da_co: &std::collections::HashSet<String>,
    items: &[Tdv<'_>],
) -> sqlx::Result<Vec<Tm7>> {
    let mut seen: std::collections::HashSet<String> = da_co.clone();
    let mut out = Vec::with_capacity(items.len());
    for (i, it) in items.iter().enumerate() {
        let ma = it.ma_ho_so.trim().to_lowercase();
        let trung = !seen.insert(ma);
        let row = sqlx::query_as::<_, Tm7>(&format!(
            "INSERT INTO submission_items (id, submission_id, stt, ma_ho_so, tieu_de, \
                 thoi_gian_bat_dau, thoi_gian_ket_thuc, so_trang, muc_do_tiep_can, ghi_chu, ho_so_id, trung_trong_dot, \
                 review_result, review_note) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, \
                 CASE WHEN $12 THEN 'khong_dat' ELSE 'pending' END, CASE WHEN $12 THEN $13 ELSE NULL END) RETURNING {ITEM_COLS}"
        ))
        .bind(Uuid::now_v7())
        .bind(submission_id)
        .bind(stt_tu + (i as i32) + 1)
        .bind(it.ma_ho_so)
        .bind(it.tieu_de)
        .bind(it.thoi_gian_bat_dau)
        .bind(it.thoi_gian_ket_thuc)
        .bind(it.so_trang)
        .bind(it.muc_do_tiep_can)
        .bind(it.ghi_chu)
        .bind(it.ho_so_id)
        .bind(trung)
        .bind(LOI_TRUNG_DOT)
        .fetch_one(&mut **tx)
        .await?;
        out.push(row);
    }
    Ok(out)
}
/// 0040 (D04): thêm dòng vào đợt đang đăng ký (nộp cùng hệ thống từ hồ sơ có sẵn, hoặc khai thêm).
pub async fn them_items(pool: &PgPool, submission_id: Uuid, items: &[Tdv<'_>]) -> sqlx::Result<Vec<Tm7>> {
    let mut tx = pool.begin().await?;
    let (stt_max, da_co): (Option<i32>, Vec<String>) = {
        let m: Option<i32> = sqlx::query_scalar("SELECT max(stt) FROM submission_items WHERE submission_id = $1")
            .bind(submission_id)
            .fetch_one(&mut *tx)
            .await?;
        let d: Vec<String> = sqlx::query_scalar("SELECT lower(btrim(ma_ho_so)) FROM submission_items WHERE submission_id = $1")
            .bind(submission_id)
            .fetch_all(&mut *tx)
            .await?;
        (m, d)
    };
    let out = chen_items(&mut tx, submission_id, stt_max.unwrap_or(0), &da_co.into_iter().collect(), items).await?;
    sqlx::query("UPDATE submissions SET updated_at = now() WHERE id = $1").bind(submission_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(out)
}
/// Hồ sơ trong hệ thống để nộp cùng hệ thống: lấy theo id trong dự án, trả (id, mã hồ sơ, tiêu đề, bắt đầu, kết thúc, số tờ, chế độ sử dụng).
pub async fn ho_so_de_nop(pool: &PgPool, project_id: Uuid, ids: &[Uuid]) -> sqlx::Result<Vec<(Uuid, Option<String>, String, Option<String>, Option<String>, Option<String>, Option<String>)>> {
    sqlx::query_as(
        "SELECT id, COALESCE(ma_ho_so_tt05, ma_ho_so, so_tam), tieu_de, ngay_bat_dau, ngay_ket_thuc, so_to, che_do_su_dung \
         FROM cl_ho_so_v WHERE project_id = $1 AND id = ANY($2) AND deleted_at IS NULL ORDER BY array_position($2, id)",
    )
    .bind(project_id)
    .bind(ids)
    .fetch_all(pool)
    .await
}
/// 0040 (D07): so sánh một dòng mục lục nộp với hồ sơ đang có trong kho — cùng mã, hoặc cùng tiêu đề (không dấu) và năm kết thúc.
pub async fn so_sanh(pool: &PgPool, project_id: Uuid, item: &Tm7) -> sqlx::Result<Vec<serde_json::Value>> {
    sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT to_jsonb(x) FROM ( \
           SELECT h.id, h.ma_ho_so, h.ma_ho_so_tt05, h.tieu_de, h.ngay_bat_dau, h.ngay_ket_thuc, h.so_to, h.ten_phong, h.ten_thbq, \
                  h.so_van_ban, h.hop_so, h.created_at, \
                  CASE WHEN lower(btrim(COALESCE(h.ma_ho_so_tt05, ''))) = lower(btrim($2)) OR lower(btrim(COALESCE(h.ma_ho_so, ''))) = lower(btrim($2)) \
                       THEN 'ma' ELSE 'tieu_de' END AS khop \
           FROM cl_ho_so_v h \
           WHERE h.project_id = $1 AND h.deleted_at IS NULL AND ( \
                 lower(btrim(COALESCE(h.ma_ho_so_tt05, ''))) = lower(btrim($2)) OR lower(btrim(COALESCE(h.ma_ho_so, ''))) = lower(btrim($2)) \
                 OR (khong_dau(h.tieu_de) = khong_dau($3) AND ($4::int IS NULL OR h.nam_ket_thuc = $4))) \
           ORDER BY khop, h.created_at LIMIT 10) x",
    )
    .bind(project_id)
    .bind(&item.ma_ho_so)
    .bind(&item.tieu_de)
    .bind(item.thoi_gian_ket_thuc.as_deref().and_then(|s| s.rsplit(['/', '-']).next()).and_then(|y| y.trim().parse::<i32>().ok()))
    .fetch_all(pool)
    .await
}
/// 0040 (D06): 10 thông tin tiếp nhận theo Điều 23 (khoá camelCase để màn hình dùng thẳng).
pub async fn tong_quan(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<serde_json::Value>> {
    sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT to_jsonb(x) FROM (            SELECT s.id, s.ma_yeu_cau AS \"maYeuCau\", s.org_name AS \"orgName\", s.org_code AS \"orgCode\", s.so_lan_nop AS \"soLanNop\", s.cach_thuc AS \"cachThuc\", s.status,                   (SELECT count(*) FROM packages p WHERE p.submission_id = s.id) AS \"tongGoi\",                   (SELECT count(*) FROM submission_items i WHERE i.submission_id = s.id) AS \"hoSoDangKy\",                   (SELECT count(*) FROM submission_items i WHERE i.submission_id = s.id AND i.review_result = 'dat') AS \"hoSoDaDuyet\",                   (SELECT count(*) FROM submission_items i WHERE i.submission_id = s.id AND i.review_result = 'khong_dat') AS \"hoSoKhongDat\",                   (SELECT count(*) FROM submission_items i WHERE i.submission_id = s.id AND i.trung_trong_dot) AS \"hoSoTrungDot\",                   (SELECT count(*) FROM submission_items i WHERE i.submission_id = s.id AND (i.ho_so_id IS NOT NULL                         OR EXISTS (SELECT 1 FROM packages p WHERE p.project_id = s.project_id AND p.ho_so_ref = i.ma_ho_so AND p.kind IN ('aip_hoso','aip_tailieu') AND p.status = 'built'))) AS \"hoSoDaVaoHeThong\",                   COALESCE((SELECT sum(p.zip_size) FROM packages p WHERE p.submission_id = s.id), 0)::bigint AS \"dungLuongGoi\",                   s.total_pages AS \"totalPages\", s.total_bytes AS \"totalBytes\", s.created_at AS \"createdAt\", s.confirmed_at AS \"confirmedAt\",                   s.received_at AS \"receivedAt\", s.approved_at AS \"approvedAt\", s.completed_at AS \"completedAt\", s.han_nop AS \"hanNop\",                   (SELECT count(*) FROM submission_files f WHERE f.submission_id = s.id) AS \"soTepDinhKem\"            FROM submissions s WHERE s.id = $1) x",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TepDinhKem {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub loai: String,
    pub filename: String,
    pub storage_key: String,
    pub size_bytes: i64,
    pub blake3: Option<String>,
    pub ghi_chu: Option<String>,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
pub const LOAI_TEP: &[&str] = &["van_ban_de_nghi", "lich_su_phong", "phuong_an_phan_loai", "huong_dan_xdgt", "quyet_dinh", "khac"];
pub async fn files_list(pool: &PgPool, submission_id: Uuid) -> sqlx::Result<Vec<TepDinhKem>> {
    sqlx::query_as::<_, TepDinhKem>(
        "SELECT id, submission_id, loai, filename, storage_key, size_bytes, blake3, ghi_chu, uploaded_by, created_at \
         FROM submission_files WHERE submission_id = $1 ORDER BY created_at",
    )
    .bind(submission_id)
    .fetch_all(pool)
    .await
}
pub async fn file_get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<TepDinhKem>> {
    sqlx::query_as::<_, TepDinhKem>(
        "SELECT id, submission_id, loai, filename, storage_key, size_bytes, blake3, ghi_chu, uploaded_by, created_at FROM submission_files WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
pub async fn file_them(pool: &PgPool, id: Uuid, submission_id: Uuid, loai: &str, filename: &str, storage_key: &str, size: i64, blake3: &str, ghi_chu: Option<&str>, boi: Uuid) -> sqlx::Result<TepDinhKem> {
    sqlx::query_as::<_, TepDinhKem>(
        "INSERT INTO submission_files (id, submission_id, loai, filename, storage_key, size_bytes, blake3, ghi_chu, uploaded_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
         RETURNING id, submission_id, loai, filename, storage_key, size_bytes, blake3, ghi_chu, uploaded_by, created_at",
    )
    .bind(id)
    .bind(submission_id)
    .bind(loai)
    .bind(filename)
    .bind(storage_key)
    .bind(size)
    .bind(blake3)
    .bind(ghi_chu)
    .bind(boi)
    .fetch_one(pool)
    .await
}
pub async fn file_xoa(pool: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    Ok(sqlx::query("DELETE FROM submission_files WHERE id = $1").bind(id).execute(pool).await?.rows_affected() > 0)
}
pub async fn get(r_2cn: &PgPool, id: Uuid) -> sqlx::Result<Option<Tm8>> {
    sqlx::query_as::<_, Tm8>(&format!("SELECT {COLS} FROM submissions WHERE id = $1"))
        .bind(id)
        .fetch_optional(r_2cn)
        .await
}
pub async fn items(r_2co: &PgPool, submission_id: Uuid) -> sqlx::Result<Vec<Tm7>> {
    sqlx::query_as::<_, Tm7>(&format!(
        "SELECT {ITEM_COLS} FROM submission_items WHERE submission_id = $1 ORDER BY stt"
    ))
    .bind(submission_id)
    .fetch_all(r_2co)
    .await
}
pub async fn f_2hy(
    r_2cp: &PgPool,
    submission_id: Uuid,
    ma_ho_so: &str,
) -> sqlx::Result<Option<i32>> {
    sqlx::query_scalar(
        "SELECT stt FROM submission_items WHERE submission_id = $1 AND ma_ho_so = $2",
    )
    .bind(submission_id)
    .bind(ma_ho_so)
    .fetch_optional(r_2cp)
    .await
}
pub async fn f_2hx(r_2cr: &PgPool, submission_id: Uuid) -> sqlx::Result<Option<(i32, i32)>> {
    let r_2cs: Option<(Option<i32>, Option<i32>)> =
        sqlx::query_as("SELECT min(stt), max(stt) FROM submission_items WHERE submission_id = $1")
            .bind(submission_id)
            .fetch_optional(r_2cr)
            .await?;
    Ok(r_2cs.and_then(|(a, r_2cq)| a.zip(r_2cq)))
}
pub async fn list(
    r_2cv: &PgPool,
    r_2cw: Option<&[Uuid]>,
    r_2ct: i64,
    r_2cu: i64,
    co_quan_nop: Option<Uuid>,
) -> sqlx::Result<Vec<Tm8>> {
    sqlx::query_as::<_, Tm8>(&format!(
        "SELECT {COLS} FROM submissions \
         WHERE ($1::uuid[] IS NULL OR project_id = ANY($1)) AND ($4::uuid IS NULL OR co_quan_nop_id = $4) \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(r_2cw)
    .bind(r_2ct)
    .bind(r_2cu)
    .bind(co_quan_nop)
    .fetch_all(r_2cv)
    .await
}
async fn transition(
    r_2cy: &PgPool,
    id: Uuid,
    from: &[&str],
    r_2d1: &str,
    extra_set: &str,
    r_2cx: Option<&str>,
) -> sqlx::Result<bool> {
    let from: Vec<String> = from.iter().map(|r_2d0| r_2d0.to_string()).collect();
    let r_2cz = sqlx::query(&format!(
        "UPDATE submissions SET status = $2, updated_at = now(){extra_set} \
         WHERE id = $1 AND status = ANY($3)"
    ))
    .bind(id)
    .bind(r_2d1)
    .bind(&from)
    .bind(r_2cx)
    .execute(r_2cy)
    .await?;
    Ok(r_2cz.rows_affected() > 0)
}
async fn f_1va(r_2d5: &PgPool, id: Uuid, r_2d2: impl Fn(&Tm8) -> String) {
    if let Ok(Some(r_2d6)) = get(r_2d5, id).await {
        let r_2d4 = r_2d2(&r_2d6);
        if let Err(r_2d3) = crate::notifications::f_1vb(r_2d5, &r_2d6, &r_2d4).await {
            tracing::warn!(
                submission = % id, error = % r_2d3,
                "phát thông báo trạng thái lỗi"
            );
        }
    }
}
pub async fn confirm(r_2d9: &PgPool, id: Uuid, r_2d7: Option<&str>) -> sqlx::Result<bool> {
    let r_2d8 = transition(
        r_2d9,
        id,
        &["registered"],
        "confirmed",
        ", decision_note = $4, confirmed_at = now()",
        r_2d7,
    )
    .await?;
    if r_2d8 {
        f_1va(
                r_2d9,
                id,
                |r_2da| {
                    format!(
                        "Lần nộp {}: ĐÃ ĐƯỢC XÁC NHẬN thu (Điều 21.2) — có thể nộp gói SIP",
                        r_2da.ma_yeu_cau
                    )
                },
            )
            .await;
    }
    Ok(r_2d8)
}
pub async fn refuse(r_2dd: &PgPool, id: Uuid, r_2db: Option<&str>) -> sqlx::Result<bool> {
    let r_2dc = transition(
        r_2dd,
        id,
        &["registered"],
        "refused",
        ", decision_note = $4",
        r_2db,
    )
    .await?;
    if r_2dc {
        f_1va(
                r_2dd,
                id,
                |r_2de| {
                    format!(
                        "Lần nộp {}: BỊ TỪ CHỐI thu (Điều 21.3) — lý do: {}",
                        r_2de.ma_yeu_cau, r_2de.decision_note.as_deref()
                        .unwrap_or("(không ghi)")
                    )
                },
            )
            .await;
    }
    Ok(r_2dc)
}
#[derive(Debug, PartialEq, Eq)]
pub struct Tig {
    pub registered: i64,
    pub submitted: i64,
    pub enough: bool,
    pub advanced: bool,
}
pub async fn reconcile(r_2dg: &PgPool, id: Uuid) -> sqlx::Result<Option<Tig>> {
    let Some(r_2di) = get(r_2dg, id).await? else {
        return Ok(None);
    };
    if r_2di.status != "confirmed" && r_2di.status != "awaiting_reception" {
        return Ok(None);
    }
    let registered: i64 =
        sqlx::query_scalar("SELECT count(*) FROM submission_items WHERE submission_id = $1")
            .bind(id)
            .fetch_one(r_2dg)
            .await?;
    let submitted: i64 = sqlx::query_scalar(
        "SELECT COALESCE(sum(document_count), 0) FROM packages \
         WHERE submission_id = $1 AND status = 'built' AND kind IN ('sip_hoso', 'sip_tailieu')",
    )
    .bind(id)
    .fetch_one(r_2dg)
    .await?;
    let enough = submitted >= registered && registered > 0;
    let r_2df = if enough {
        "đủ".to_string()
    } else {
        format!(
            "chưa đủ, còn thiếu {}",
            registered - submitted
        )
    };
    let status = if enough {
        "awaiting_reception"
    } else {
        "confirmed"
    };
    sqlx::query(
        "UPDATE submissions SET status = $2, reconcile_note = $3, updated_at = now() \
         WHERE id = $1 AND status IN ('confirmed', 'awaiting_reception')",
    )
    .bind(id)
    .bind(status)
    .bind(&r_2df)
    .execute(r_2dg)
    .await?;
    let advanced = enough && r_2di.status == "confirmed";
    if advanced {
        f_1va(
                r_2dg,
                id,
                |r_2dh| {
                    format!(
                        "Lần nộp {}: đã nộp ĐỦ gói theo Mục lục — chờ tiếp nhận (Điều 22.3.d)",
                        r_2dh.ma_yeu_cau
                    )
                },
            )
            .await;
    }
    Ok(Some(Tig {
        registered,
        submitted,
        enough,
        advanced,
    }))
}
pub async fn receive(r_2dl: &PgPool, id: Uuid, r_2dj: Option<&str>) -> sqlx::Result<bool> {
    let r_2dk = transition(
        r_2dl,
        id,
        &["awaiting_reception"],
        "received",
        ", reception_note = $4, received_at = now()",
        r_2dj,
    )
    .await?;
    if r_2dk {
        f_1va(r_2dl, id, |r_2dm| {
            format!(
                "Lần nộp {}: đã TIẾP NHẬN (Điều 23)",
                r_2dm.ma_yeu_cau
            )
        })
        .await;
    }
    Ok(r_2dk)
}
pub async fn review_item(
    r_2dp: &PgPool,
    r_2dn: Uuid,
    r_2dr: &str,
    r_2do: Option<&str>,
) -> sqlx::Result<bool> {
    if r_2dr != "dat" && r_2dr != "khong_dat" {
        return Ok(false);
    }
    let r_2dq = sqlx::query(
        "UPDATE submission_items SET review_result = $2, review_note = $3 WHERE id = $1",
    )
    .bind(r_2dn)
    .bind(r_2dr)
    .bind(r_2do)
    .execute(r_2dp)
    .await?;
    Ok(r_2dq.rows_affected() > 0)
}
pub async fn approve(
    r_2du: &PgPool,
    id: Uuid,
    approval_doc: &str,
    r_2ds: Option<&str>,
) -> sqlx::Result<bool> {
    let r_2dv = sqlx::query(
        "UPDATE submissions SET status = 'approved', approval_doc = $2, approval_note = $3, \
             approved_at = now(), updated_at = now() \
         WHERE id = $1 AND status = 'received'",
    )
    .bind(id)
    .bind(approval_doc)
    .bind(r_2ds)
    .execute(r_2du)
    .await?;
    let r_2dt = r_2dv.rows_affected() > 0;
    if r_2dt {
        f_1va(
                r_2du,
                id,
                |r_2dw| {
                    format!(
                        "Lần nộp {}: đã PHÊ DUYỆT ({}) — chuyển gói SIP thành AIP (Điều 25.3.a)",
                        r_2dw.ma_yeu_cau, r_2dw.approval_doc.as_deref().unwrap_or("")
                    )
                },
            )
            .await;
    }
    Ok(r_2dt)
}
pub async fn complete(r_2dy: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let r_2dz = sqlx::query(
        "UPDATE submissions SET status = 'completed', completed_at = now(), updated_at = now() \
         WHERE id = $1 AND status = 'approved'",
    )
    .bind(id)
    .execute(r_2dy)
    .await?;
    let r_2dx = r_2dz.rows_affected() > 0;
    if r_2dx {
        f_1va(r_2dy, id, |r_2e0| {
            format!(
                "Lần nộp {}: HOÀN THÀNH thu nộp (Điều 25.5)",
                r_2e0.ma_yeu_cau
            )
        })
        .await;
    }
    Ok(r_2dx)
}
pub async fn f_18c(
    r_2e1: &PgPool,
    project_id: Uuid,
    submission_id: Uuid,
) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar(
        "SELECT DISTINCT i.ma_ho_so FROM submission_items i \
         JOIN packages p ON p.project_id = $1 \
             AND p.ho_so_ref = i.ma_ho_so \
             AND p.kind IN ('aip_hoso', 'aip_tailieu') \
             AND p.status = 'built' \
             AND (p.submission_id IS DISTINCT FROM $2) \
         WHERE i.submission_id = $2 ORDER BY 1",
    )
    .bind(project_id)
    .bind(submission_id)
    .fetch_all(r_2e1)
    .await
}
