// Copyright (c) 2026 DigiText. All rights reserved.
//! BÁO CÁO THỐNG KÊ theo Điều 44.3b TT 05/2025/TT-BNV (27/11/2026, đợt 3 kế hoạch đối chiếu Digifile TT05 — nhóm 3A).
//! Mỗi báo cáo một `async fn(pool, &BoLoc) -> Vec<Value>` đọc thẳng bảng (như `chinh_ly::thong_ke`), dòng là `to_jsonb(x)` snake_case;
//! định nghĩa cột / nhóm / bản áp dụng nằm ở `routes/bao_cao.rs` (desktop dựng bộ lọc và bảng từ đó). Không cache: báo cáo chạy
//! theo yêu cầu, bộ lọc đẩy xuống chỉ mục (project_id, thời gian) có sẵn; `luot_khai_thac` cần chỉ mục 0036.
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row as _};
use uuid::Uuid;

/// Bộ lọc chung. `nam_tu/nam_den` lọc theo năm kết thúc hồ sơ (báo cáo mục lục); `from/to` theo mốc thời gian sự kiện (báo cáo
/// tài liệu số, gói, kiểm tra, sao lưu, khai thác, mượn trả) — `to` đã cộng 1 ngày ở tầng route (kết thúc bao gồm).
#[derive(Debug, Clone, Default)]
pub struct BoLoc {
    pub project_id: Uuid,
    pub phong_id: Option<Uuid>,
    pub nam_tu: Option<i32>,
    pub nam_den: Option<i32>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

fn rows_json(rows: Vec<sqlx::postgres::PgRow>) -> Vec<Value> {
    rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect()
}

/// Điều kiện hồ sơ dùng chung: $1 dự án, $2 phông, $3/$4 năm kết thúc. Bí danh view là `h`.
const HS_WHERE: &str = "h.project_id = $1 AND h.deleted_at IS NULL AND ($2::uuid IS NULL OR h.phong_id = $2) \
    AND ($3::int IS NULL OR h.nam_ket_thuc >= $3) AND ($4::int IS NULL OR h.nam_ket_thuc <= $4)";

/// Tháng theo giờ Việt Nam từ một cột timestamptz.
fn thang(col: &str) -> String {
    format!("to_char({col} AT TIME ZONE 'Asia/Ho_Chi_Minh', 'YYYY-MM')")
}

async fn chay_hs(pool: &PgPool, sql: &str, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    Ok(rows_json(sqlx::query(sql).bind(b.project_id).bind(b.phong_id).bind(b.nam_tu).bind(b.nam_den).fetch_all(pool).await?))
}
async fn chay_tg(pool: &PgPool, sql: &str, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    Ok(rows_json(sqlx::query(sql).bind(b.project_id).bind(b.phong_id).bind(b.from).bind(b.to).fetch_all(pool).await?))
}

// ------------------------------------------------------------------------------------------------------ nhóm mục lục hồ sơ

/// #1 Tổng hợp theo phông: phòng ban, hộp, hồ sơ (VV / có thời hạn / hết THBQ), văn bản, tờ, trang, hồ sơ từ gói AIP, đã nộp lưu.
pub async fn phong(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = format!(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT p.id AS phong_id, p.ma_du_an, p.ten AS ten_phong, p.phong_so, \
                  (SELECT count(*) FROM cl_phong_ban pb WHERE pb.phong_id = p.id AND pb.deleted_at IS NULL) AS so_phong_ban, \
                  count(h.id) AS so_ho_so, count(h.id) FILTER (WHERE h.vinh_vien) AS hs_vv, count(h.id) FILTER (WHERE NOT h.vinh_vien) AS hs_co_thoi_han, \
                  count(h.id) FILTER (WHERE h.thbq_status = 'HET') AS hs_het, count(DISTINCT h.hop_id) AS so_hop, \
                  COALESCE(sum(h.so_van_ban), 0) AS so_van_ban, COALESCE(sum(cl_int_of(h.so_to)), 0) AS so_to, COALESCE(sum(h.so_luong_trang), 0) AS so_trang, \
                  count(g.ho_so_id) AS hs_tu_goi, count(h.id) FILTER (WHERE h.nop_luu_trang_thai = 'da_nop') AS hs_da_nop_luu \
           FROM cl_phong p \
           LEFT JOIN cl_ho_so_v h ON h.phong_id = p.id AND {HS_WHERE} \
           LEFT JOIN cl_ho_so_goi g ON g.ho_so_id = h.id \
           WHERE p.project_id = $1 AND p.deleted_at IS NULL AND ($2::uuid IS NULL OR p.id = $2) \
           GROUP BY p.id, p.ma_du_an, p.ten, p.phong_so, p.so_tt ORDER BY p.so_tt NULLS LAST, p.ten) x"
    );
    chay_hs(pool, &sql, b).await
}

/// #2 Hồ sơ theo năm kết thúc.
pub async fn ho_so_nam(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = format!(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT COALESCE(h.nam_ket_thuc::text, '(không rõ)') AS nam, count(*) AS so_ho_so, count(*) FILTER (WHERE h.vinh_vien) AS hs_vv, \
                  count(*) FILTER (WHERE NOT h.vinh_vien) AS hs_co_thoi_han, count(*) FILTER (WHERE h.thbq_status = 'HET') AS hs_het, \
                  count(DISTINCT h.hop_id) AS so_hop, COALESCE(sum(h.so_van_ban), 0) AS so_van_ban, COALESCE(sum(h.so_luong_trang), 0) AS so_trang \
           FROM cl_ho_so_v h WHERE {HS_WHERE} GROUP BY h.nam_ket_thuc ORDER BY h.nam_ket_thuc NULLS LAST) x"
    );
    chay_hs(pool, &sql, b).await
}

/// #4 Theo thời hạn bảo quản: hồ sơ, văn bản, trang; hết THBQ; sẽ hết trong 5 năm tới (có thời hạn).
pub async fn thbq(pool: &PgPool, b: &BoLoc, nam_now: i32) -> sqlx::Result<Vec<Value>> {
    let sql = format!(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT t.id AS thbq_id, t.ten AS ten_thbq, t.ma_tt05, \
                  CASE t.trang_thai WHEN 1 THEN 'Vĩnh viễn' WHEN 2 THEN 'Có thời hạn' WHEN 3 THEN 'Đánh giá lại' ELSE 'Theo tuổi thọ công trình' END AS loai, \
                  t.gia_tri AS so_nam, count(h.id) AS so_ho_so, COALESCE(sum(h.so_van_ban), 0) AS so_van_ban, COALESCE(sum(h.so_luong_trang), 0) AS so_trang, \
                  count(h.id) FILTER (WHERE h.thbq_status = 'HET') AS hs_het, \
                  count(h.id) FILTER (WHERE t.trang_thai = 2 AND h.nam_ket_thuc IS NOT NULL AND h.nam_ket_thuc + t.gia_tri BETWEEN $5 AND $5 + 5) AS hs_het_5_nam \
           FROM cl_thbq t LEFT JOIN cl_ho_so_v h ON h.thbq_id = t.id AND {HS_WHERE} \
           WHERE (t.project_id = $1 OR t.project_id IS NULL) AND t.deleted_at IS NULL \
           GROUP BY t.id, t.ten, t.ma_tt05, t.trang_thai, t.gia_tri, t.so_tt ORDER BY t.so_tt NULLS LAST, t.trang_thai, t.gia_tri DESC) x"
    );
    Ok(rows_json(sqlx::query(&sql).bind(b.project_id).bind(b.phong_id).bind(b.nam_tu).bind(b.nam_den).bind(nam_now).fetch_all(pool).await?))
}

/// #5 / #6: một giá trị một dòng, đếm hồ sơ và văn bản (cột khác nhau giữa hồ sơ và văn bản).
async fn theo_gia_tri(pool: &PgPool, b: &BoLoc, cot_hs: &str, cot_vb: &str) -> sqlx::Result<Vec<Value>> {
    let sql = format!(
        "WITH hs AS (SELECT COALESCE(NULLIF(btrim(h.{cot_hs}), ''), '(chưa ghi)') AS k, count(*) AS n FROM cl_ho_so_v h WHERE {HS_WHERE} GROUP BY 1), \
              vb AS (SELECT COALESCE(NULLIF(btrim(v.{cot_vb}), ''), '(chưa ghi)') AS k, count(*) AS n, COALESCE(sum(v.so_trang), 0) AS trang \
                     FROM cl_van_ban v JOIN cl_ho_so_v h ON h.id = v.ho_so_id WHERE v.deleted_at IS NULL AND {HS_WHERE} GROUP BY 1) \
         SELECT to_jsonb(x) AS row FROM ( \
           SELECT COALESCE(hs.k, vb.k) AS gia_tri, COALESCE(hs.n, 0) AS so_ho_so, COALESCE(vb.n, 0) AS so_van_ban, COALESCE(vb.trang, 0) AS so_trang \
           FROM hs FULL JOIN vb ON vb.k = hs.k ORDER BY 1) x"
    );
    chay_hs(pool, &sql, b).await
}
pub async fn che_do_su_dung(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    theo_gia_tri(pool, b, "che_do_su_dung", "che_do_su_dung").await
}
pub async fn do_tin_cay(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    theo_gia_tri(pool, b, "muc_do_tin_cay", "do_tin_cay").await
}

/// #7 Theo loại văn bản (mã TT05 trong `cl_van_ban.loai_vb`, tên tra `cl_loai_van_ban` của dự án rồi dùng chung).
pub async fn loai_van_ban(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = format!(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT COALESCE(NULLIF(btrim(v.loai_vb), ''), '(chưa ghi)') AS ma, COALESCE(l.ten, NULLIF(btrim(v.loai_vb), ''), '(chưa ghi)') AS ten_loai, \
                  count(*) AS so_van_ban, count(DISTINCT v.ho_so_id) AS so_ho_so, COALESCE(sum(v.so_trang), 0) AS so_trang, \
                  count(*) FILTER (WHERE EXISTS (SELECT 1 FROM cl_van_ban_file f WHERE f.van_ban_id = v.id AND f.deleted_at IS NULL AND f.document_id IS NOT NULL)) AS vb_co_tai_lieu_so \
           FROM cl_van_ban v JOIN cl_ho_so_v h ON h.id = v.ho_so_id \
           LEFT JOIN LATERAL (SELECT l.ten FROM cl_loai_van_ban l WHERE l.ma = v.loai_vb AND (l.project_id = $1 OR l.project_id IS NULL) AND l.deleted_at IS NULL ORDER BY l.project_id NULLS LAST LIMIT 1) l ON true \
           WHERE v.deleted_at IS NULL AND {HS_WHERE} GROUP BY 1, 2 ORDER BY so_van_ban DESC, ten_loai) x"
    );
    chay_hs(pool, &sql, b).await
}

// ------------------------------------------------------------------------------------------------------ nhóm tài liệu số / gói

/// #8 Tài liệu số theo phông: số tệp, dung lượng, trang, PDF/A, chữ ký số, quét virus, đã lưu trữ. Tài liệu chưa nối `cl_van_ban_file` → "(chưa gắn phông)".
pub async fn tai_lieu_so(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = "SELECT to_jsonb(x) AS row FROM ( \
           SELECT COALESCE(p.ma_du_an, '') AS ma_du_an, COALESCE(p.ten, '(chưa gắn phông)') AS ten_phong, \
                  count(*) AS so_tai_lieu, round(COALESCE(sum(d.size_bytes), 0) / 1048576.0, 1) AS dung_luong_mb, COALESCE(sum(d.page_count), 0) AS so_trang, \
                  count(*) FILTER (WHERE d.pdfa_2b) AS pdfa_dat, count(*) FILTER (WHERE d.pdfa_2b = false) AS pdfa_khong_dat, count(*) FILTER (WHERE d.pdfa_2b IS NULL) AS pdfa_chua_kiem, \
                  count(*) FILTER (WHERE d.ky_so = 'hop_le') AS ky_hop_le, count(*) FILTER (WHERE d.ky_so = 'khong_hop_le') AS ky_khong_hop_le, \
                  count(*) FILTER (WHERE d.ky_so IS NULL OR d.ky_so = 'chua_ky') AS chua_ky, \
                  count(*) FILTER (WHERE d.virus_scan LIKE 'clean%') AS virus_sach, count(*) FILTER (WHERE d.virus_scan LIKE 'infected%') AS virus_nhiem, \
                  count(*) FILTER (WHERE d.virus_scan IS NULL) AS virus_chua_quet, count(*) FILTER (WHERE d.status = 'archived') AS da_luu_tru \
           FROM documents d \
           LEFT JOIN LATERAL (SELECT h.phong_id FROM cl_van_ban_file f JOIN cl_van_ban v ON v.id = f.van_ban_id JOIN cl_ho_so h ON h.id = v.ho_so_id \
                              WHERE f.document_id = d.id AND f.deleted_at IS NULL LIMIT 1) l ON true \
           LEFT JOIN cl_phong p ON p.id = l.phong_id \
           WHERE d.project_id = $1 AND d.status <> 'disposed' AND ($2::uuid IS NULL OR p.id = $2) \
             AND ($3::timestamptz IS NULL OR d.uploaded_at >= $3) AND ($4::timestamptz IS NULL OR d.uploaded_at < $4) \
           GROUP BY p.id, p.ma_du_an, p.ten, p.so_tt ORDER BY p.so_tt NULLS LAST, p.ten NULLS LAST) x";
    chay_tg(pool, sql, b).await
}

/// #9 Gói tin theo loại × nguồn (đóng / nhập): số gói, đã dựng, lỗi, đang dựng, tài liệu, dung lượng zip, đã huỷ theo quyết định.
pub async fn goi_tin(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = "SELECT to_jsonb(x) AS row FROM ( \
           SELECT p.kind AS loai, CASE p.nguon WHEN 'nhap' THEN 'Nhập Hệ thống' ELSE 'Đóng gói' END AS nguon, count(*) AS so_goi, \
                  count(*) FILTER (WHERE p.status = 'built') AS da_dung, count(*) FILTER (WHERE p.status = 'failed') AS loi, \
                  count(*) FILTER (WHERE p.status IN ('queued', 'building')) AS dang_dung, \
                  COALESCE(sum(p.document_count) FILTER (WHERE p.status = 'built'), 0) AS so_tai_lieu, \
                  round(COALESCE(sum(p.zip_size), 0) / 1048576.0, 1) AS dung_luong_mb, count(*) FILTER (WHERE p.huy_at IS NOT NULL) AS da_huy, \
                  count(*) FILTER (WHERE p.virus_scan LIKE 'infected%') AS virus_nhiem \
           FROM packages p WHERE p.project_id = $1 AND ($2::uuid IS NULL OR true) \
             AND ($3::timestamptz IS NULL OR p.created_at >= $3) AND ($4::timestamptz IS NULL OR p.created_at < $4) \
           GROUP BY p.kind, p.nguon ORDER BY p.kind, p.nguon) x";
    chay_tg(pool, sql, b).await
}

/// #12a Thu nộp vào lưu trữ lịch sử (bản thi công): theo tháng đăng ký.
pub async fn thu_nop(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = format!(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT {th} AS thang, count(*) AS so_dang_ky, count(*) FILTER (WHERE s.status = 'refused') AS tu_choi, \
                  count(*) FILTER (WHERE s.status IN ('confirmed', 'awaiting_reception')) AS cho_nhan, count(*) FILTER (WHERE s.status = 'received') AS da_nhan, \
                  count(*) FILTER (WHERE s.status = 'approved') AS da_duyet, count(*) FILTER (WHERE s.status = 'completed') AS hoan_tat, \
                  COALESCE(sum(s.total_pages), 0) AS so_trang, round(COALESCE(sum(s.total_bytes), 0) / 1048576.0, 1) AS dung_luong_mb, \
                  count(*) FILTER (WHERE s.cach_thuc = 'truc_tiep') AS truc_tiep \
           FROM submissions s WHERE s.project_id = $1 AND ($2::uuid IS NULL OR true) \
             AND ($3::timestamptz IS NULL OR s.created_at >= $3) AND ($4::timestamptz IS NULL OR s.created_at < $4) \
           GROUP BY 1 ORDER BY 1) x",
        th = thang("s.created_at")
    );
    chay_tg(pool, &sql, b).await
}

/// #12b Nhập Hệ thống (gói AIP): theo tháng.
pub async fn nhap_he_thong(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = format!(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT {th} AS thang, count(DISTINCT i.batch_id) AS so_dot, count(*) AS so_goi, count(*) FILTER (WHERE i.ket_qua = 'nhap') AS da_nhap, \
                  count(*) FILTER (WHERE i.ket_qua = 'da_co') AS da_co, count(*) FILTER (WHERE i.ket_qua = 'tu_choi') AS tu_choi, count(*) FILTER (WHERE i.ket_qua = 'loi') AS loi, \
                  COALESCE(sum(i.so_van_ban) FILTER (WHERE i.ket_qua = 'nhap'), 0) AS so_van_ban, COALESCE(sum(i.so_chua_ky) FILTER (WHERE i.ket_qua = 'nhap'), 0) AS so_chua_ky, \
                  round(COALESCE(sum(i.size_bytes) FILTER (WHERE i.ket_qua = 'nhap'), 0) / 1048576.0, 1) AS dung_luong_mb \
           FROM ingest_items i WHERE i.project_id = $1 AND ($2::uuid IS NULL OR true) \
             AND ($3::timestamptz IS NULL OR i.created_at >= $3) AND ($4::timestamptz IS NULL OR i.created_at < $4) \
           GROUP BY 1 ORDER BY 1) x",
        th = thang("i.created_at")
    );
    chay_tg(pool, &sql, b).await
}

// ------------------------------------------------------------------------------------------------------ nhóm bảo quản / tiêu huỷ

/// #10 Kiểm tra toàn vẹn (Điều 29) theo tháng × nguồn.
pub async fn kiem_tra(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = format!(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT {th} AS thang, CASE c.nguon WHEN 'ke_hoach' THEN 'Theo kế hoạch' WHEN 'tay' THEN 'Chạy tay' WHEN 'kiem_lai' THEN 'Kiểm lại sau khắc phục' ELSE 'Tự động' END AS nguon, \
                  count(*) AS so_lan, count(*) FILTER (WHERE c.ket_qua = 'dat') AS dat, count(*) FILTER (WHERE c.ket_qua = 'loi') AS loi, \
                  count(DISTINCT c.document_id) AS so_tai_lieu, count(DISTINCT c.package_id) AS so_goi, count(DISTINCT c.ke_hoach_id) AS so_ke_hoach \
           FROM integrity_checks c WHERE c.project_id = $1 AND ($2::uuid IS NULL OR true) \
             AND ($3::timestamptz IS NULL OR c.checked_at >= $3) AND ($4::timestamptz IS NULL OR c.checked_at < $4) \
           GROUP BY 1, 2 ORDER BY 1, 2) x",
        th = thang("c.checked_at")
    );
    chay_tg(pool, &sql, b).await
}

/// #11 Sao lưu (Điều 30) theo tháng × loại — toàn máy chủ (sổ sao lưu không theo dự án).
pub async fn sao_luu(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = format!(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT {th} AS thang, CASE r.loai WHEN 'gia_tang' THEN 'Gia tăng' WHEN 'day_du' THEN 'Đầy đủ' ELSE 'Đầy đủ 3 năm' END AS loai, \
                  count(*) AS so_lan, count(*) FILTER (WHERE r.ket_qua = 'dat') AS dat, count(*) FILTER (WHERE r.ket_qua = 'khong_dat') AS khong_dat, \
                  round(COALESCE(sum(r.dung_luong_bytes), 0) / 1048576.0, 1) AS dung_luong_mb, count(*) FILTER (WHERE r.da_xoa_at IS NOT NULL) AS da_xoa, \
                  count(DISTINCT r.phuong_tien_id) AS so_phuong_tien \
           FROM backup_runs r WHERE ($1::uuid IS NULL OR true) AND ($2::uuid IS NULL OR true) \
             AND ($3::timestamptz IS NULL OR r.created_at >= $3) AND ($4::timestamptz IS NULL OR r.created_at < $4) \
           GROUP BY 1, 2 ORDER BY 1, 2) x",
        th = thang("r.created_at")
    );
    chay_tg(pool, &sql, b).await
}

/// #17 Tiêu huỷ (Điều 48–50): mỗi đợt một dòng.
pub async fn tieu_huy(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = "SELECT to_jsonb(x) AS row FROM ( \
           SELECT d.nam, CASE d.status WHEN 'hoi_dong' THEN 'Hội đồng xác định' WHEN 'tham_dinh' THEN 'Thẩm định' WHEN 'quyet_dinh' THEN 'Đã quyết định' ELSE 'Đã huỷ' END AS trang_thai, \
                  d.quyet_dinh_so, CASE WHEN d.tu_quyet THEN 'Tự quyết' ELSE 'Thẩm định' END AS hinh_thuc, \
                  (SELECT count(*) FROM disposal_entries e WHERE e.batch_id = d.id) AS so_tai_lieu, \
                  (SELECT count(*) FROM disposal_entries e WHERE e.batch_id = d.id AND e.ly_do = 'het_thoi_han') AS het_han, \
                  (SELECT count(*) FROM disposal_entries e WHERE e.batch_id = d.id AND e.ly_do = 'trung_lap') AS trung_lap, \
                  (SELECT count(*) FROM disposal_entries e WHERE e.batch_id = d.id AND e.status = 'da_huy') AS da_huy, \
                  (SELECT count(*) FROM disposal_entries e WHERE e.batch_id = d.id AND e.status = 'giu_lai') AS giu_lai, \
                  d.created_at, d.decided_at, d.executed_at \
           FROM disposal_batches d WHERE d.project_id = $1 AND ($2::uuid IS NULL OR true) \
             AND ($3::int IS NULL OR d.nam >= $3) AND ($4::int IS NULL OR d.nam <= $4) \
           ORDER BY d.nam DESC, d.created_at DESC) x";
    chay_hs(pool, sql, b).await
}

// ------------------------------------------------------------------------------------------------------ nhóm khai thác / mượn trả

/// #13 Yêu cầu khai thác theo hình thức: số yêu cầu theo trạng thái, tài liệu duyệt / từ chối, phí.
pub async fn khai_thac_yeu_cau(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = "SELECT to_jsonb(x) AS row FROM ( \
           SELECT CASE r.kind WHEN 'doc' THEN 'Đọc tại chỗ' WHEN 'ban_sao' THEN 'Bản sao' WHEN 'ban_sao_ky' THEN 'Bản sao có chứng thực' ELSE 'Gói DIP' END AS hinh_thuc, \
                  count(*) AS so_yeu_cau, count(*) FILTER (WHERE r.status = 'submitted') AS cho_duyet, count(*) FILTER (WHERE r.status = 'approved') AS da_duyet, \
                  count(*) FILTER (WHERE r.status = 'refused') AS tu_choi, count(*) FILTER (WHERE r.status = 'expired') AS het_han, \
                  (SELECT count(*) FROM access_request_items i WHERE i.request_id = ANY(array_agg(r.id)) AND i.deleted_at IS NULL) AS so_tai_lieu, \
                  (SELECT count(*) FROM access_request_items i WHERE i.request_id = ANY(array_agg(r.id)) AND i.deleted_at IS NULL AND i.verdict = 'duyet') AS tl_duyet, \
                  (SELECT count(*) FROM access_request_items i WHERE i.request_id = ANY(array_agg(r.id)) AND i.deleted_at IS NULL AND i.verdict = 'tu_choi') AS tl_tu_choi, \
                  COALESCE(sum(r.phi), 0) AS phi, COALESCE(sum(r.phi) FILTER (WHERE r.paid_at IS NOT NULL), 0) AS phi_da_thu \
           FROM access_requests r WHERE r.project_id = $1 AND ($2::uuid IS NULL OR true) \
             AND ($3::timestamptz IS NULL OR r.created_at >= $3) AND ($4::timestamptz IS NULL OR r.created_at < $4) \
           GROUP BY r.kind ORDER BY r.kind) x";
    chay_tg(pool, sql, b).await
}

/// #14 Lượt khai thác (nhật ký `audit_log` `khai_thac.*`) theo tháng × việc. Cần chỉ mục 0036.
pub async fn luot_khai_thac(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = format!(
        "SELECT to_jsonb(x) AS row FROM ( \
           SELECT {th} AS thang, CASE split_part(a.action, '.', 2) WHEN 'xem' THEN 'Xem' WHEN 'doc' THEN 'Đọc bản sao' WHEN 'tai' THEN 'Tải bản sao' \
                       WHEN 'in' THEN 'In' WHEN 'chuyen_doi' THEN 'Chuyển đổi ra giấy' ELSE split_part(a.action, '.', 2) END AS viec, \
                  count(*) AS so_luot, count(DISTINCT a.target) AS so_tai_lieu, count(DISTINCT a.actor_id) AS so_nguoi \
           FROM audit_log a WHERE a.action LIKE 'khai_thac.%' AND a.detail->>'projectId' = $1::text AND ($2::uuid IS NULL OR true) \
             AND ($3::timestamptz IS NULL OR a.at >= $3) AND ($4::timestamptz IS NULL OR a.at < $4) \
           GROUP BY 1, 2 ORDER BY 1, 2) x",
        th = thang("a.at")
    );
    chay_tg(pool, &sql, b).await
}

/// #15 Cung cấp thông tin theo hình thức: trạng thái, phí, thời gian xử lý trung bình (ngày).
pub async fn cung_cap_thong_tin(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = "SELECT to_jsonb(x) AS row FROM ( \
           SELECT CASE r.hinh_thuc WHEN 'danh_muc' THEN 'Danh mục' WHEN 'trich_xuat' THEN 'Trích xuất' ELSE 'Tổng hợp' END AS hinh_thuc, \
                  count(*) AS so_yeu_cau, count(*) FILTER (WHERE r.status = 'submitted') AS dang_xu_ly, count(*) FILTER (WHERE r.status = 'approved') AS da_duyet, \
                  count(*) FILTER (WHERE r.status = 'fulfilled') AS da_cung_cap, count(*) FILTER (WHERE r.status = 'refused') AS tu_choi, \
                  COALESCE(sum(r.phi), 0) AS phi, COALESCE(sum(r.phi) FILTER (WHERE r.paid_at IS NOT NULL), 0) AS phi_da_thu, \
                  round(COALESCE(avg(EXTRACT(EPOCH FROM (r.fulfilled_at - r.created_at)) / 86400.0) FILTER (WHERE r.fulfilled_at IS NOT NULL), 0)::numeric, 1) AS ngay_xu_ly_tb \
           FROM info_requests r WHERE r.project_id = $1 AND ($2::uuid IS NULL OR true) \
             AND ($3::timestamptz IS NULL OR r.created_at >= $3) AND ($4::timestamptz IS NULL OR r.created_at < $4) \
           GROUP BY r.hinh_thuc ORDER BY r.hinh_thuc) x";
    chay_tg(pool, sql, b).await
}

/// #16 Sổ mượn – trả hồ sơ giấy (một dòng một hồ sơ trên phiếu; 0034). Lọc theo ngày mượn và phông.
pub async fn muon_tra(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    let sql = "SELECT to_jsonb(x) AS row FROM ( \
           SELECT m.so_phieu, m.ngay_muon, m.nguoi_muon, m.don_vi, m.noi_dung, h.ma_ho_so, h.so_tam, h.tieu_de, h.hop_so, h.ten_phong, \
                  m.ngay_du_kien_tra, m.han_tra_goc, m.so_lan_gia_han, m.ngay_thuc_te_tra, \
                  CASE m.trang_thai WHEN 'cho_duyet' THEN 'Chờ duyệt' WHEN 'da_duyet' THEN 'Đã duyệt, chờ giao' WHEN 'tu_choi' THEN 'Từ chối' \
                       WHEN 'dang_muon' THEN CASE WHEN m.ngay_du_kien_tra < CURRENT_DATE THEN 'Quá hạn' ELSE 'Đang mượn' END WHEN 'da_tra' THEN 'Đã trả' ELSE 'Đã huỷ' END AS trang_thai, \
                  (m.trang_thai = 'dang_muon' AND m.ngay_du_kien_tra < CURRENT_DATE) AS qua_han, m.tra_truoc_han, u.display_name AS nguoi_lap, ud.display_name AS nguoi_duyet_ten \
           FROM cl_muon_tra m JOIN cl_ho_so_v h ON h.id = m.ho_so_id LEFT JOIN users u ON u.id = m.nguoi_lap_phieu LEFT JOIN users ud ON ud.id = m.nguoi_duyet \
           WHERE m.project_id = $1 AND m.deleted_at IS NULL AND ($2::uuid IS NULL OR h.phong_id = $2) \
             AND ($3::timestamptz IS NULL OR m.ngay_muon >= ($3 AT TIME ZONE 'Asia/Ho_Chi_Minh')::date) AND ($4::timestamptz IS NULL OR m.ngay_muon < ($4 AT TIME ZONE 'Asia/Ho_Chi_Minh')::date) \
           ORDER BY m.ngay_muon, m.so_phieu NULLS LAST, m.created_at) x";
    chay_tg(pool, sql, b).await
}

/// #18 Kho – giá – hộp: mỗi giá một dòng, sức chứa và % đầy (0035; cùng câu với màn Kho và giá).
pub async fn kho_gia(pool: &PgPool, b: &BoLoc) -> sqlx::Result<Vec<Value>> {
    crate::chinh_ly::kho_gia::tong_hop(pool, b.project_id, None, b.phong_id).await
}
