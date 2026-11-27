-- 0040 (27/11/2026) — Đợt 6: bản Lưu trữ đủ như Digifile (docs/DOI-CHIEU-DIGIFILE-94.md). Áp bằng `digitext-api migrate`
-- hoặc tay sau 0039. Không khoá bảng lớn; có DROP/CREATE lại view cl_ho_so_v (máy trạm mất kết nối vài giây).
--
-- 1. cl_ma_tt05: bảng mã TT05 còn thiếu (Digifile B03) — chế độ sử dụng, mức độ tin cậy, tình trạng vật lý, tình trạng dự phòng.
--    Dùng chung (project_id NULL) + dự án tự thêm. Cột chữ trên cl_ho_so / cl_van_ban vẫn lưu TÊN (không đổi dữ liệu cũ).
-- 2. cl_ho_so / cl_van_ban: dự phòng (riskRecovery / riskRecoveryStatus — B04, B05), quy trình xử lý văn bản (process — B05),
--    gói tài liệu rời lẻ (B09: la_goi_tai_lieu, ma_goi). View thêm ma_ho_so_tt05 (có mã cơ quan lưu trữ — B06) và ma_ho_so_giay_tt05.
-- 3. Thu nộp lưu trữ lịch sử: co_quan_nop (D11) + khoá API (D12), users.co_quan_nop_id, submissions.co_quan_nop_id,
--    submission_files (tệp đính kèm Điều 20 — D02), submission_items.ho_so_id (nộp cùng hệ thống — D04), trung_trong_dot (D07).
-- 4. Khai thác: duyệt hai cấp (F04: trạng thái cho_cap_2), phân công người xử lý (F12), giao nhận bản sao giấy (F10).
-- 5. audit_log: chuỗi băm prev_hash / hash (H08).

-- ============================================================ 1. bảng mã TT05
CREATE TABLE IF NOT EXISTS cl_ma_tt05 (
    id          uuid PRIMARY KEY,
    project_id  uuid REFERENCES projects(id) ON DELETE CASCADE,
    nhom        text NOT NULL CHECK (nhom IN ('che_do_su_dung', 'muc_do_tin_cay', 'tinh_trang_vat_ly', 'tinh_trang_du_phong')),
    ma          text NOT NULL,
    ten         text NOT NULL,
    so_tt       integer,
    ghi_chu     text,
    created_at  timestamptz NOT NULL DEFAULT now(),
    created_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    deleted_at  timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_ma_tt05 ON cl_ma_tt05 (COALESCE(project_id, '00000000-0000-0000-0000-000000000000'::uuid), nhom, ma) WHERE deleted_at IS NULL;
INSERT INTO cl_ma_tt05 (id, project_id, nhom, ma, ten, so_tt) VALUES
    ('00000000-0000-4000-8000-00000c050101', NULL, 'che_do_su_dung', '01', 'Công khai', 1),
    ('00000000-0000-4000-8000-00000c050102', NULL, 'che_do_su_dung', '02', 'Sử dụng có điều kiện', 2),
    ('00000000-0000-4000-8000-00000c050103', NULL, 'che_do_su_dung', '03', 'Mật', 3),
    ('00000000-0000-4000-8000-00000c050201', NULL, 'muc_do_tin_cay', '01', 'Gốc điện tử', 1),
    ('00000000-0000-4000-8000-00000c050202', NULL, 'muc_do_tin_cay', '02', 'Số hoá', 2),
    ('00000000-0000-4000-8000-00000c050203', NULL, 'muc_do_tin_cay', '03', 'Hỗn hợp', 3),
    ('00000000-0000-4000-8000-00000c050301', NULL, 'tinh_trang_vat_ly', '01', 'Tốt', 1),
    ('00000000-0000-4000-8000-00000c050302', NULL, 'tinh_trang_vat_ly', '02', 'Bình thường', 2),
    ('00000000-0000-4000-8000-00000c050303', NULL, 'tinh_trang_vat_ly', '03', 'Hỏng', 3),
    ('00000000-0000-4000-8000-00000c050401', NULL, 'tinh_trang_du_phong', '01', 'Đã dự phòng', 1),
    ('00000000-0000-4000-8000-00000c050402', NULL, 'tinh_trang_du_phong', '02', 'Chưa dự phòng', 2)
ON CONFLICT DO NOTHING;
DROP TRIGGER IF EXISTS tg_cl_ma_tt05_changed ON cl_ma_tt05;
CREATE TRIGGER tg_cl_ma_tt05_changed AFTER INSERT OR UPDATE OR DELETE ON cl_ma_tt05
    FOR EACH STATEMENT EXECUTE FUNCTION note_change('chinhLy');

-- ============================================================ 2. hồ sơ / văn bản
ALTER TABLE cl_ho_so ADD COLUMN IF NOT EXISTS du_phong boolean;
ALTER TABLE cl_ho_so ADD COLUMN IF NOT EXISTS du_phong_trang_thai text;
ALTER TABLE cl_ho_so ADD COLUMN IF NOT EXISTS la_goi_tai_lieu boolean NOT NULL DEFAULT false;
ALTER TABLE cl_ho_so ADD COLUMN IF NOT EXISTS ma_goi text;
ALTER TABLE cl_van_ban ADD COLUMN IF NOT EXISTS du_phong boolean;
ALTER TABLE cl_van_ban ADD COLUMN IF NOT EXISTS du_phong_trang_thai text;
ALTER TABLE cl_van_ban ADD COLUMN IF NOT EXISTS quy_trinh_xu_ly text;

-- View: chép nguyên 0031, thêm ma_co_quan_lt, ma_ho_so_tt05 (mã cơ quan lưu trữ + mã hồ sơ — Phụ lục I arcFileCode),
-- ma_ho_so_giay_tt05 ([mã CQ].[kho/giá/hộp].[số hồ sơ giấy] — paperFileCode).
DROP VIEW IF EXISTS cl_ho_so_v;
CREATE VIEW cl_ho_so_v AS
SELECT h.*,
       p.ma_du_an, p.phong_so, p.muc_luc_so, p.muc_luc_so_cth, p.ten AS ten_phong,
       pb.ten AS ten_phong_ban, pb.ma_don_vi,
       m.ma_mang, m.ten AS ten_mang, m.so_tt AS mang_so_tt,
       t.ten AS ten_thbq, t.trang_thai AS thbq_trang_thai, t.gia_tri AS thbq_gia_tri, t.so_tt AS thbq_so_tt, t.ma_tt05 AS thbq_ma_tt05,
       cl_thbq_status(t.trang_thai, t.gia_tri, h.nam_ket_thuc, extract(year from now())::int) AS thbq_status,
       hp.hop_so, hp.hop_so_int,
       bo.so AS bo_so, tp.so AS tap_so,
       COALESCE(h.gia_id, gd.id) AS gia_eff_id,
       COALESCE(gm.ten, gd.ten) AS ten_gia,
       COALESCE(km.id, kd.id) AS kho_eff_id,
       COALESCE(km.ten, kd.ten) AS ten_kho,
       CASE WHEN h.ho_so_so IS NOT NULL AND btrim(h.ho_so_so) <> '' AND h.nam_ket_thuc IS NOT NULL
            THEN concat_ws('.', COALESCE(NULLIF(btrim(p.ma_du_an), ''), '00'), COALESCE(NULLIF(btrim(p.phong_so), ''), '00'),
                           h.nam_ket_thuc::text, btrim(h.ho_so_so),
                           CASE WHEN h.vinh_vien THEN NULLIF(btrim(p.muc_luc_so), '')
                                ELSE COALESCE(NULLIF(btrim(p.muc_luc_so_cth), ''), NULLIF(btrim(p.muc_luc_so), '')) END) END AS ma_ho_so,
       (SELECT count(*) FROM cl_van_ban v WHERE v.ho_so_id = h.id AND v.deleted_at IS NULL) AS so_van_ban,
       EXISTS (SELECT 1 FROM cl_muon_tra mt WHERE mt.ho_so_id = h.id AND mt.trang_thai = 'dang_muon' AND mt.deleted_at IS NULL) AS dang_muon,
       cu.display_name AS created_by_name, uu.display_name AS updated_by_name,
       -- 0031: lưu trữ hiện hành
       cl_date_end_of(h.ngay_ket_thuc) AS ngay_kt_cuoi,
       dm.ma_dong AS dm_ma_dong, dm.tieu_de AS dm_tieu_de,
       nl.ma_phieu AS nop_luu_ma_phieu,
       -- 0040: mã theo Phụ lục I có mã cơ quan lưu trữ (B06)
       cq.ma_co_quan AS ma_co_quan_lt,
       CASE WHEN h.ho_so_so IS NOT NULL AND btrim(h.ho_so_so) <> '' AND h.nam_ket_thuc IS NOT NULL
            THEN concat_ws('.', NULLIF(btrim(cq.ma_co_quan), ''),
                           COALESCE(NULLIF(btrim(p.ma_du_an), ''), '00'), COALESCE(NULLIF(btrim(p.phong_so), ''), '00'),
                           h.nam_ket_thuc::text, btrim(h.ho_so_so),
                           CASE WHEN h.vinh_vien THEN NULLIF(btrim(p.muc_luc_so), '')
                                ELSE COALESCE(NULLIF(btrim(p.muc_luc_so_cth), ''), NULLIF(btrim(p.muc_luc_so), '')) END) END AS ma_ho_so_tt05,
       CASE WHEN hp.hop_so IS NOT NULL
            THEN concat_ws('.', NULLIF(btrim(cq.ma_co_quan), ''),
                           concat_ws('/', COALESCE(km.ten, kd.ten), COALESCE(gm.ten, gd.ten), hp.hop_so),
                           COALESCE(NULLIF(btrim(h.ma_ho_so_giay), ''), NULLIF(btrim(h.ho_so_so), ''), h.so_tam)) END AS ma_ho_so_giay_tt05
FROM cl_ho_so h
JOIN cl_phong p ON p.id = h.phong_id
LEFT JOIN cl_phong_ban pb ON pb.id = h.phong_ban_id
LEFT JOIN cl_mang m ON m.id = h.mang_id
LEFT JOIN cl_thbq t ON t.id = h.thbq_id
LEFT JOIN cl_hop hp ON hp.id = h.hop_id
LEFT JOIN cl_bo_tap bo ON bo.id = h.bo_id
LEFT JOIN cl_bo_tap tp ON tp.id = h.tap_id
LEFT JOIN cl_gia gm ON gm.id = h.gia_id
LEFT JOIN cl_kho km ON km.id = gm.kho_id
LEFT JOIN LATERAL (
    SELECT g.id, g.ten, g.kho_id FROM cl_gia g
    WHERE h.gia_id IS NULL AND g.deleted_at IS NULL AND g.phong_ban_id = h.phong_ban_id
      AND (g.vinh_vien IS NULL OR g.vinh_vien = hp.vinh_vien)
      AND hp.hop_so_int IS NOT NULL AND hp.hop_so_int BETWEEN g.tu_hop AND g.den_hop
    ORDER BY g.tu_hop LIMIT 1
) gd ON true
LEFT JOIN cl_kho kd ON kd.id = gd.kho_id
LEFT JOIN users cu ON cu.id = h.created_by
LEFT JOIN users uu ON uu.id = h.updated_by
LEFT JOIN lt_danh_muc_dong dm ON dm.id = h.dm_dong_id
LEFT JOIN lt_nop_luu nl ON nl.id = h.nop_luu_id
LEFT JOIN co_quan_luu_tru cq ON cq.project_id = h.project_id;

-- ============================================================ 3. thu nộp lưu trữ lịch sử
CREATE TABLE IF NOT EXISTS co_quan_nop (
    id              uuid PRIMARY KEY,
    project_id      uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    ma              text NOT NULL,
    ten             text NOT NULL,
    dia_chi         text,
    nguoi_lien_he   text,
    dien_thoai      text,
    email           text,
    ghi_chu         text,
    api_key_hash    text,                 -- SHA-256 hex của khoá API; NULL = chưa cấp
    api_key_tao_at  timestamptz,
    api_key_goi_at  timestamptz,          -- lần gọi API gần nhất
    created_by      uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_co_quan_nop_ma ON co_quan_nop (project_id, lower(ma)) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS ux_co_quan_nop_api_key ON co_quan_nop (api_key_hash) WHERE api_key_hash IS NOT NULL;
DROP TRIGGER IF EXISTS tg_co_quan_nop_changed ON co_quan_nop;
CREATE TRIGGER tg_co_quan_nop_changed AFTER INSERT OR UPDATE OR DELETE ON co_quan_nop
    FOR EACH STATEMENT EXECUTE FUNCTION note_change('submissions');

ALTER TABLE users ADD COLUMN IF NOT EXISTS co_quan_nop_id uuid REFERENCES co_quan_nop(id) ON DELETE SET NULL;
ALTER TABLE submissions ADD COLUMN IF NOT EXISTS co_quan_nop_id uuid REFERENCES co_quan_nop(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS ix_submissions_co_quan_nop ON submissions (co_quan_nop_id) WHERE co_quan_nop_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS submission_files (
    id              uuid PRIMARY KEY,
    submission_id   uuid NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    loai            text NOT NULL CHECK (loai IN ('van_ban_de_nghi', 'lich_su_phong', 'phuong_an_phan_loai', 'huong_dan_xdgt', 'quyet_dinh', 'khac')),
    filename        text NOT NULL,
    storage_key     text NOT NULL,
    size_bytes      bigint NOT NULL DEFAULT 0,
    blake3          text,
    ghi_chu         text,
    uploaded_by     uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at      timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS ix_submission_files_sub ON submission_files (submission_id);

ALTER TABLE submission_items ADD COLUMN IF NOT EXISTS ho_so_id uuid REFERENCES cl_ho_so(id) ON DELETE SET NULL;
ALTER TABLE submission_items ADD COLUMN IF NOT EXISTS trung_trong_dot boolean NOT NULL DEFAULT false;
-- Trùng mã trong cùng đợt (Điều 24.2): trước đây bị chặn bằng khoá duy nhất → không ghi nhận được; nay lưu dòng, tự đánh không đạt.
DROP INDEX IF EXISTS ux_submission_items_ma_ho_so;
CREATE INDEX IF NOT EXISTS ix_submission_items_ma_ho_so ON submission_items (submission_id, ma_ho_so);

-- ============================================================ 4. khai thác
ALTER TABLE access_requests DROP CONSTRAINT IF EXISTS access_requests_status_check;
ALTER TABLE access_requests ADD CONSTRAINT access_requests_status_check
    CHECK (status IN ('submitted', 'cho_cap_2', 'approved', 'refused', 'expired'));
ALTER TABLE access_requests ADD COLUMN IF NOT EXISTS nguoi_xu_ly uuid REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE access_requests ADD COLUMN IF NOT EXISTS cap_2_boi uuid REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE access_requests ADD COLUMN IF NOT EXISTS cap_2_at timestamptz;
ALTER TABLE access_requests ADD COLUMN IF NOT EXISTS giao_nhan jsonb;   -- bản sao giấy: {hinhThuc, diaChi, nguoiNhan, ngayGiao, ghiChu}
ALTER TABLE info_requests ADD COLUMN IF NOT EXISTS nguoi_xu_ly uuid REFERENCES users(id) ON DELETE SET NULL;

-- ============================================================ 5. chuỗi băm nhật ký
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS prev_hash text;
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS hash text;
