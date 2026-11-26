-- 0031 (26/11/2026): LƯU TRỮ HIỆN HÀNH — Điều 19 TT 05/2025/TT-BNV: nộp hồ sơ vào lưu trữ cơ quan (hạn 60 ngày, đồng ý /
-- trả lại, khoá sửa, quyền sử dụng 360 ngày, sổ nộp lưu) và khoản 4: danh mục hồ sơ dự kiến có trình – duyệt – ban hành.
-- Kế hoạch: nhóm 1 trong C:\Users\HieuPC\.claude\plans\...functional-cherny.md (đối chiếu Digifile TT05).
--
-- Chốt với anh Hiếu 26/11/2026: hạn tính theo NGÀY LỊCH từ NGÀY KẾT THÚC hồ sơ (năm → 31/12, tháng → cuối tháng); hồ sơ
-- KHOÁ sửa ngay khi nộp (rút phiếu được); MỘT danh mục / phông / năm, mỗi phòng ban soạn phần mình; MỘT người MỘT phòng ban.
--
-- Bảng mới mang tiền tố lt_ để không rơi vào bộ máy generic cl_* (0012: table_name ép cl_, vòng DO tạo trigger theo danh
-- sách cứng, rows::ident_ok). lt_* chỉ online, như submissions. Trạng thái nộp lưu nằm trên cl_ho_so dưới dạng 3 cột MÁY CHỦ
-- SINH (is_derived trong digitext-core spec.rs) → trôi xuống bản sao offline qua pull, không bị đẩy ngược.
--
-- Áp bằng `digitext-api migrate` LÚC BẢO TRÌ: mục 8 DROP/CREATE lại view cl_ho_so_v (SELECT h.* — thêm cột là phải tạo lại,
-- như 0015) nên API đang chạy lỗi vài giây.
BEGIN;

-- 1. Ngày CUỐI kỳ của chuỗi ngày không đầy đủ (cl_date_of của 0012 là ĐẦU kỳ). Hạn nộp = ngày này + 60 (tính ở Rust,
--    digitext_core::luu_tru::HAN_NOP_NGAY, đổi một chỗ).
CREATE OR REPLACE FUNCTION cl_date_end_of(t text) RETURNS date LANGUAGE sql IMMUTABLE AS $$
    SELECT CASE WHEN t ~ '^\d{4}$' THEN make_date(t::int, 12, 31)
                WHEN t ~ '^\d{1,2}/\d{4}$'
                    THEN (make_date(split_part(t, '/', 2)::int, split_part(t, '/', 1)::int, 1) + interval '1 month' - interval '1 day')::date
                WHEN t ~ '^\d{1,2}/\d{1,2}/\d{4}$'
                    THEN make_date(split_part(t, '/', 3)::int, split_part(t, '/', 2)::int, split_part(t, '/', 1)::int) END $$;

-- 2. Danh mục hồ sơ dự kiến: một phiên bản của (phông, năm). Phiên bản mới chép dòng (tien_than_id); bản cũ → het_hieu_luc.
CREATE TABLE IF NOT EXISTS lt_danh_muc (
    id              uuid PRIMARY KEY,
    project_id      uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phong_id        uuid NOT NULL REFERENCES cl_phong(id) ON DELETE CASCADE,
    nam             integer NOT NULL,
    phien_ban       integer NOT NULL DEFAULT 1,
    trang_thai      text NOT NULL DEFAULT 'du_thao'
                    CHECK (trang_thai IN ('du_thao', 'cho_duyet', 'da_duyet', 'ban_hanh', 'het_hieu_luc')),
    so_quyet_dinh   text,
    ngay_ban_hanh   date,
    trinh_boi       uuid REFERENCES users(id) ON DELETE SET NULL,
    trinh_luc       timestamptz,
    duyet_boi       uuid REFERENCES users(id) ON DELETE SET NULL,
    duyet_luc       timestamptz,
    ban_hanh_boi    uuid REFERENCES users(id) ON DELETE SET NULL,
    ban_hanh_luc    timestamptz,
    ly_do_tra       text,
    ghi_chu         text,
    created_by      uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    UNIQUE (phong_id, nam, phien_ban)
);
-- Một bản đang hiệu lực cho mỗi (phông, năm).
CREATE UNIQUE INDEX IF NOT EXISTS ux_lt_danh_muc_hieu_luc ON lt_danh_muc (phong_id, nam) WHERE trang_thai = 'ban_hanh';
CREATE INDEX IF NOT EXISTS ix_lt_danh_muc_du_an ON lt_danh_muc (project_id, nam DESC, phien_ban DESC);

CREATE TABLE IF NOT EXISTS lt_danh_muc_dong (
    id              uuid PRIMARY KEY,
    project_id      uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    danh_muc_id     uuid NOT NULL REFERENCES lt_danh_muc(id) ON DELETE CASCADE,
    phong_ban_id    uuid NOT NULL REFERENCES cl_phong_ban(id) ON DELETE CASCADE,
    mang_id         uuid REFERENCES cl_mang(id) ON DELETE SET NULL,
    -- Số ký hiệu hồ sơ dự kiến — GIỮ NGUYÊN qua các phiên bản để đối chiếu "theo / ngoài danh mục" không lệ thuộc phiên bản.
    ma_dong         text NOT NULL,
    tieu_de         text NOT NULL,
    thbq_id         uuid REFERENCES cl_thbq(id) ON DELETE SET NULL,
    nguoi_lap       text,
    ghi_chu         text,
    so_tt           integer,
    tien_than_id    uuid REFERENCES lt_danh_muc_dong(id) ON DELETE SET NULL,
    created_by      uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    UNIQUE (danh_muc_id, ma_dong)
);
CREATE INDEX IF NOT EXISTS ix_lt_danh_muc_dong_pb ON lt_danh_muc_dong (danh_muc_id, phong_ban_id, so_tt);

-- 3. Phiếu nộp lưu: một lần một phòng ban nộp một lô hồ sơ. Số phiếu "{stt:03}/NL-{mã đơn vị}.{năm}" sinh ở Rust.
CREATE TABLE IF NOT EXISTS lt_nop_luu (
    id              uuid PRIMARY KEY,
    project_id      uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phong_id        uuid NOT NULL REFERENCES cl_phong(id) ON DELETE CASCADE,
    phong_ban_id    uuid REFERENCES cl_phong_ban(id) ON DELETE SET NULL,
    ma_phieu        text NOT NULL,
    nguoi_nop       uuid REFERENCES users(id) ON DELETE SET NULL,
    ghi_chu         text,
    trang_thai      text NOT NULL DEFAULT 'cho_tiep_nhan'
                    CHECK (trang_thai IN ('cho_tiep_nhan', 'da_tiep_nhan', 'tra_lai', 'rut')),
    tiep_nhan_boi   uuid REFERENCES users(id) ON DELETE SET NULL,
    tiep_nhan_luc   timestamptz,
    ly_do_tra       text,
    so_ho_so        integer NOT NULL DEFAULT 0,
    so_van_ban      integer,
    so_trang        integer,
    bien_ban_id     uuid REFERENCES records(id) ON DELETE SET NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    UNIQUE (project_id, ma_phieu)
);
CREATE INDEX IF NOT EXISTS ix_lt_nop_luu_tt ON lt_nop_luu (project_id, trang_thai, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_lt_nop_luu_pb ON lt_nop_luu (phong_ban_id, created_at DESC);

CREATE TABLE IF NOT EXISTS lt_nop_luu_ho_so (
    id              uuid PRIMARY KEY,
    nop_luu_id      uuid NOT NULL REFERENCES lt_nop_luu(id) ON DELETE CASCADE,
    ho_so_id        uuid NOT NULL REFERENCES cl_ho_so(id) ON DELETE CASCADE,
    stt             integer,
    ket_qua         text NOT NULL DEFAULT 'cho' CHECK (ket_qua IN ('cho', 'nhan', 'tra_lai')),
    ly_do           text,
    UNIQUE (nop_luu_id, ho_so_id)
);
-- Một hồ sơ không nằm trong hai phiếu đang chờ.
CREATE UNIQUE INDEX IF NOT EXISTS ux_lt_nop_luu_ho_so_cho ON lt_nop_luu_ho_so (ho_so_id) WHERE ket_qua = 'cho';
CREATE INDEX IF NOT EXISTS ix_lt_nop_luu_ho_so_hs ON lt_nop_luu_ho_so (ho_so_id);

-- 4. cl_ho_so: 3 cột máy chủ sinh (trạng thái nộp lưu) + 1 cột nghiệp vụ chỉ online (dòng danh mục).
ALTER TABLE cl_ho_so
    ADD COLUMN IF NOT EXISTS nop_luu_trang_thai text CHECK (nop_luu_trang_thai IN ('cho_tiep_nhan', 'da_nop', 'tra_lai')),
    ADD COLUMN IF NOT EXISTS nop_luu_id         uuid REFERENCES lt_nop_luu(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS khoa_luc           timestamptz,
    ADD COLUMN IF NOT EXISTS dm_dong_id         uuid REFERENCES lt_danh_muc_dong(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS ix_cl_ho_so_nop_luu ON cl_ho_so (project_id, nop_luu_trang_thai) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_ho_so_dm_dong ON cl_ho_so (dm_dong_id) WHERE dm_dong_id IS NOT NULL;

-- 5. Người dùng thuộc phòng ban (một người một phòng; kiêm nhiệm thì thêm tài khoản).
ALTER TABLE users ADD COLUMN IF NOT EXISTS phong_ban_id uuid REFERENCES cl_phong_ban(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS ix_users_phong_ban ON users (phong_ban_id) WHERE phong_ban_id IS NOT NULL;

-- 6. Loại biên bản (0024 + 3) và hành động lịch sử hồ sơ (0023 + 4).
ALTER TABLE records DROP CONSTRAINT IF EXISTS records_kind_check;
ALTER TABLE records ADD CONSTRAINT records_kind_check CHECK (kind = ANY (ARRAY[
    'kiem_tra', 'nhat_ky_sao_luu', 'sao_luu', 'su_co', 'thu_nop', 'thu_nop_truc_tiep', 'giao_nhan', 'tra_lai',
    'giao_nhan_giay', 'tra_tai_lieu_giay', 'nhat_ky_so_hoa', 'kiem_tra_san_pham', 'nghiem_thu', 'ban_giao_lan',
    -- lưu trữ hiện hành (Điều 19):
    'giao_nhan_nop_luu',   -- biên bản giao nhận hồ sơ nộp lưu (phòng ban → lưu trữ cơ quan), ref = mã phiếu
    'so_nop_luu',          -- sổ nộp lưu một năm, ref = năm
    'danh_muc_ho_so'       -- bản in danh mục hồ sơ ban hành, ref = <mã phông>-<năm>
]::text[]));
ALTER TABLE cl_lich_su DROP CONSTRAINT IF EXISTS cl_lich_su_action_check;
ALTER TABLE cl_lich_su ADD CONSTRAINT cl_lich_su_action_check
    CHECK (action IN ('add', 'edit', 'delete', 'restore', 'import', 'systematize', 'boxing', 'assign_gia', 'ocr',
                      'nop_luu', 'tiep_nhan', 'tra_lai_nop_luu', 'rut_nop_luu'));

-- 7. Thông báo có khoá tham chiếu để nhắc hạn không lặp (như ux_notifications_sla của thu nộp):
--    ref = 'ho_so:<uuid>:nhac15' | 'ho_so:<uuid>:qua_han:<tuần ISO>' | 'nop_luu:<uuid>' | 'danh_muc:<uuid>' ...
ALTER TABLE notifications ADD COLUMN IF NOT EXISTS ref text;
CREATE UNIQUE INDEX IF NOT EXISTS ux_notifications_ref ON notifications (user_id, kind, ref) WHERE ref IS NOT NULL;

-- 8. View: chép nguyên 0015, thêm ngay_kt_cuoi, dm_ma_dong / dm_tieu_de, nop_luu_ma_phieu.
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
       nl.ma_phieu AS nop_luu_ma_phieu
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
LEFT JOIN lt_nop_luu nl ON nl.id = h.nop_luu_id;

-- 9. Máy trạm tự nạp lại khi có thay đổi (topic mới "nopLuu", "danhMuc").
DROP TRIGGER IF EXISTS tg_lt_nop_luu_changed ON lt_nop_luu;
CREATE TRIGGER tg_lt_nop_luu_changed AFTER INSERT OR UPDATE OR DELETE ON lt_nop_luu
    FOR EACH STATEMENT EXECUTE FUNCTION note_change('nopLuu');
DROP TRIGGER IF EXISTS tg_lt_nop_luu_ho_so_changed ON lt_nop_luu_ho_so;
CREATE TRIGGER tg_lt_nop_luu_ho_so_changed AFTER INSERT OR UPDATE OR DELETE ON lt_nop_luu_ho_so
    FOR EACH STATEMENT EXECUTE FUNCTION note_change('nopLuu');
DROP TRIGGER IF EXISTS tg_lt_danh_muc_changed ON lt_danh_muc;
CREATE TRIGGER tg_lt_danh_muc_changed AFTER INSERT OR UPDATE OR DELETE ON lt_danh_muc
    FOR EACH STATEMENT EXECUTE FUNCTION note_change('danhMuc');
DROP TRIGGER IF EXISTS tg_lt_danh_muc_dong_changed ON lt_danh_muc_dong;
CREATE TRIGGER tg_lt_danh_muc_dong_changed AFTER INSERT OR UPDATE OR DELETE ON lt_danh_muc_dong
    FOR EACH STATEMENT EXECUTE FUNCTION note_change('danhMuc');

COMMIT;
