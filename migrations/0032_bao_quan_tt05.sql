-- 0032 (26/11/2026): BẢO QUẢN đủ Điều 29–31 TT 05/2025/TT-BNV — đợt 2 kế hoạch đối chiếu Digifile TT05.
--   Điều 29: kế hoạch kiểm tra hằng năm có duyệt (bk_ke_hoach), ảnh chụp tệp/gói chốt lúc duyệt (bk_ke_hoach_tep), danh mục lỗi
--            và khắc phục từng dòng (bk_loi); integrity_checks ghi nguồn (tự động / kế hoạch / tay / kiểm lại) và kế hoạch.
--   Điều 30: lịch sao lưu (bk_lich_sao_luu: ngày / tháng / 3 năm, số bản giữ, độ trễ cho phép), backup_runs thêm đường dẫn,
--            dung lượng, mã băm, lịch, phương tiện, dấu đã xoá bản cũ; phương tiện lưu trữ có hạn độ bền (bk_phuong_tien, 30.4).
--   Điều 31: sự cố và phục hồi gắn bản sao lưu đã dùng (bk_su_co).
-- Chốt 26/11/2026 (theo đề xuất kế hoạch, anh Hiếu chưa phản đối): "đơn vị" trong kế hoạch = phông của dự án; quyền duyệt riêng
-- bk.approve (0030 tự bổ sung cho vai trò hệ thống); xoá bản sao lưu cũ CHỈ nhắc + ghi nhận, xoá thật do script.
-- Lịch sao lưu, phương tiện, sự cố là TOÀN MÁY CHỦ (backup_runs không có project_id); kế hoạch kiểm tra theo dự án (kho).
-- Áp bằng `digitext-api migrate` (không DROP view, API chạy tiếp được).
BEGIN;

-- 1. Kế hoạch kiểm tra hằng năm (Điều 29.2): dự thảo → chờ duyệt → đã duyệt (chốt ảnh chụp tệp) → đang thực hiện → hoàn thành.
CREATE TABLE IF NOT EXISTS bk_ke_hoach (
    id              uuid PRIMARY KEY,
    project_id      uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    nam             integer NOT NULL,
    ma              text NOT NULL,
    ten             text NOT NULL,
    -- {"phongIds": [uuid…], "xacSuat": 0–100 (% tệp lấy mẫu, trống = toàn bộ), "lauChuaKiem": true (chỉ tệp chưa kiểm 3 năm),
    --  "goi": true (kiểm cả zip gói AIP)}
    pham_vi         jsonb NOT NULL DEFAULT '{}'::jsonb,
    phuong_phap     text,
    trang_thai      text NOT NULL DEFAULT 'du_thao'
                    CHECK (trang_thai IN ('du_thao', 'cho_duyet', 'da_duyet', 'dang_thuc_hien', 'hoan_thanh', 'huy')),
    nguoi_lap       uuid REFERENCES users(id) ON DELETE SET NULL,
    trinh_luc       timestamptz,
    duyet_boi       uuid REFERENCES users(id) ON DELETE SET NULL,
    duyet_luc       timestamptz,
    ly_do_tu_choi   text,
    bat_dau_luc     timestamptz,
    ket_thuc_luc    timestamptz,
    so_tep          integer NOT NULL DEFAULT 0,
    so_dat          integer NOT NULL DEFAULT 0,
    so_loi          integer NOT NULL DEFAULT 0,
    record_id       uuid REFERENCES records(id) ON DELETE SET NULL,
    ghi_chu         text,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS ix_bk_ke_hoach_du_an ON bk_ke_hoach (project_id, nam DESC, created_at DESC);
CREATE UNIQUE INDEX IF NOT EXISTS ux_bk_ke_hoach_ma ON bk_ke_hoach (project_id, ma);

-- Ảnh chụp tệp / gói thuộc kế hoạch, chốt lúc DUYỆT (tệp thêm sau không vào kế hoạch này).
CREATE TABLE IF NOT EXISTS bk_ke_hoach_tep (
    id              uuid PRIMARY KEY,
    ke_hoach_id     uuid NOT NULL REFERENCES bk_ke_hoach(id) ON DELETE CASCADE,
    document_id     uuid REFERENCES documents(id) ON DELETE CASCADE,
    package_id      uuid REFERENCES packages(id) ON DELETE CASCADE,
    trang_thai      text NOT NULL DEFAULT 'cho' CHECK (trang_thai IN ('cho', 'dat', 'loi', 'bo_qua')),
    check_id        uuid REFERENCES integrity_checks(id) ON DELETE SET NULL,
    chi_tiet        text,
    kiem_luc        timestamptz,
    CHECK ((document_id IS NULL) <> (package_id IS NULL))
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_bk_kh_tep_doc ON bk_ke_hoach_tep (ke_hoach_id, document_id) WHERE document_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS ux_bk_kh_tep_goi ON bk_ke_hoach_tep (ke_hoach_id, package_id) WHERE package_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS ix_bk_kh_tep_cho ON bk_ke_hoach_tep (ke_hoach_id) WHERE trang_thai = 'cho';

-- 2. Danh mục lỗi và khắc phục từng dòng (Điều 29.4.đ, 29.4.g). Mỗi tệp / gói chỉ một dòng lỗi ĐANG MỞ.
CREATE TABLE IF NOT EXISTS bk_loi (
    id                  uuid PRIMARY KEY,
    project_id          uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    ke_hoach_id         uuid REFERENCES bk_ke_hoach(id) ON DELETE SET NULL,
    document_id         uuid REFERENCES documents(id) ON DELETE CASCADE,
    package_id          uuid REFERENCES packages(id) ON DELETE CASCADE,
    check_id            uuid REFERENCES integrity_checks(id) ON DELETE SET NULL,
    chi_tiet            text,
    phat_hien_luc       timestamptz NOT NULL DEFAULT now(),
    trang_thai          text NOT NULL DEFAULT 'moi'
                        CHECK (trang_thai IN ('moi', 'da_khac_phuc', 'kiem_lai_dat', 'kiem_lai_loi', 'bo_qua')),
    khac_phuc           text,
    khac_phuc_boi       uuid REFERENCES users(id) ON DELETE SET NULL,
    khac_phuc_luc       timestamptz,
    kiem_lai_check_id   uuid REFERENCES integrity_checks(id) ON DELETE SET NULL,
    kiem_lai_luc        timestamptz,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now(),
    CHECK ((document_id IS NULL) <> (package_id IS NULL))
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_bk_loi_mo_doc ON bk_loi (document_id)
    WHERE document_id IS NOT NULL AND trang_thai IN ('moi', 'da_khac_phuc', 'kiem_lai_loi');
CREATE UNIQUE INDEX IF NOT EXISTS ux_bk_loi_mo_goi ON bk_loi (package_id)
    WHERE package_id IS NOT NULL AND trang_thai IN ('moi', 'da_khac_phuc', 'kiem_lai_loi');
CREATE INDEX IF NOT EXISTS ix_bk_loi_du_an ON bk_loi (project_id, trang_thai, phat_hien_luc DESC);

-- 3. Lần kiểm biết nguồn và kế hoạch.
ALTER TABLE integrity_checks ADD COLUMN IF NOT EXISTS ke_hoach_id uuid REFERENCES bk_ke_hoach(id) ON DELETE SET NULL;
ALTER TABLE integrity_checks ADD COLUMN IF NOT EXISTS nguon text NOT NULL DEFAULT 'tu_dong';
ALTER TABLE integrity_checks DROP CONSTRAINT IF EXISTS integrity_checks_nguon_check;
ALTER TABLE integrity_checks ADD CONSTRAINT integrity_checks_nguon_check CHECK (nguon IN ('tu_dong', 'ke_hoach', 'tay', 'kiem_lai'));
CREATE INDEX IF NOT EXISTS ix_integrity_checks_ke_hoach ON integrity_checks (ke_hoach_id) WHERE ke_hoach_id IS NOT NULL;

-- 4. Phương tiện lưu trữ (Điều 30.4): hạn độ bền để nhắc chuyển đổi trước 1 năm (tính lúc truy vấn, không lưu trạng thái "sắp hết").
CREATE TABLE IF NOT EXISTS bk_phuong_tien (
    id                  uuid PRIMARY KEY,
    ma                  text NOT NULL UNIQUE,
    loai                text NOT NULL,              -- o_cung | bang_tu | nas | dam_may | quang | khac (chữ tự do, gợi ý ở UI)
    mo_ta               text,
    dung_luong_bytes    bigint,
    noi_bao_quan        text,
    ngay_dua_vao        date,
    han_do_ben          date,
    trang_thai          text NOT NULL DEFAULT 'dang_dung' CHECK (trang_thai IN ('dang_dung', 'da_thay', 'hong', 'loai_bo')),
    thay_boi_id         uuid REFERENCES bk_phuong_tien(id) ON DELETE SET NULL,
    nhac_luc            timestamptz,
    ghi_chu             text,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now()
);

-- 5. Lịch sao lưu (Điều 30.1: gia tăng hằng ngày, đầy đủ hằng tháng, đầy đủ 3 năm). Toàn máy chủ.
CREATE TABLE IF NOT EXISTS bk_lich_sao_luu (
    id                  uuid PRIMARY KEY,
    loai                text NOT NULL CHECK (loai IN ('gia_tang', 'day_du', 'day_du_3_nam')),
    ten                 text NOT NULL,
    chu_ky_gio          integer NOT NULL CHECK (chu_ky_gio > 0),
    do_tre_cho_phep_gio integer NOT NULL DEFAULT 2 CHECK (do_tre_cho_phep_gio >= 0),
    giu_so_ban          integer NOT NULL DEFAULT 14 CHECK (giu_so_ban >= 1),
    bat                 boolean NOT NULL DEFAULT true,
    nguon               text,
    phuong_tien_id      uuid REFERENCES bk_phuong_tien(id) ON DELETE SET NULL,
    ghi_chu             text,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now()
);
-- Ba dòng khớp sao-luu.ps1 / sao-luu.sh (chạy 01:00 hằng đêm: gia tăng kho + đầy đủ CSDL; đầy đủ 3 năm làm tay).
INSERT INTO bk_lich_sao_luu (id, loai, ten, chu_ky_gio, do_tre_cho_phep_gio, giu_so_ban, nguon, ghi_chu)
SELECT gen_random_uuid(), 'gia_tang', 'Sao lưu gia tăng hằng ngày', 24, 2, 14, 'Kho tài liệu', 'Điều 30.1.a — robocopy/rsync kho lúc 01:00'
WHERE NOT EXISTS (SELECT 1 FROM bk_lich_sao_luu WHERE loai = 'gia_tang');
INSERT INTO bk_lich_sao_luu (id, loai, ten, chu_ky_gio, do_tre_cho_phep_gio, giu_so_ban, nguon, ghi_chu)
SELECT gen_random_uuid(), 'day_du', 'Sao lưu đầy đủ CSDL hằng ngày', 24, 2, 14, 'CSDL PostgreSQL (pg_dump)', 'Điều 30.1.b — pg_dump lúc 01:00; giữ 14 bản'
WHERE NOT EXISTS (SELECT 1 FROM bk_lich_sao_luu WHERE loai = 'day_du');
INSERT INTO bk_lich_sao_luu (id, loai, ten, chu_ky_gio, do_tre_cho_phep_gio, giu_so_ban, nguon, ghi_chu)
SELECT gen_random_uuid(), 'day_du_3_nam', 'Sao lưu đầy đủ ra phương tiện riêng (3 năm)', 24 * 365 * 3, 24 * 30, 3, 'CSDL + kho + mã nguồn phần mềm', 'Điều 30.1.c — làm tay, ghi nhận ở màn Bảo quản'
WHERE NOT EXISTS (SELECT 1 FROM bk_lich_sao_luu WHERE loai = 'day_du_3_nam');

-- 6. Sổ sao lưu đủ cột Mẫu 02/03: lịch, phương tiện, đường dẫn, dung lượng, mã băm, thời gian, người, đã xoá bản cũ.
ALTER TABLE backup_runs ADD COLUMN IF NOT EXISTS lich_id uuid REFERENCES bk_lich_sao_luu(id) ON DELETE SET NULL;
ALTER TABLE backup_runs ADD COLUMN IF NOT EXISTS phuong_tien_id uuid REFERENCES bk_phuong_tien(id) ON DELETE SET NULL;
ALTER TABLE backup_runs ADD COLUMN IF NOT EXISTS duong_dan text;
ALTER TABLE backup_runs ADD COLUMN IF NOT EXISTS dung_luong_bytes bigint;
ALTER TABLE backup_runs ADD COLUMN IF NOT EXISTS ma_bam text;
ALTER TABLE backup_runs ADD COLUMN IF NOT EXISTS bat_dau_at timestamptz;
ALTER TABLE backup_runs ADD COLUMN IF NOT EXISTS ket_thuc_at timestamptz;
ALTER TABLE backup_runs ADD COLUMN IF NOT EXISTS nguoi_thuc_hien text;
ALTER TABLE backup_runs ADD COLUMN IF NOT EXISTS da_xoa_at timestamptz;
ALTER TABLE backup_runs ADD COLUMN IF NOT EXISTS xoa_boi uuid REFERENCES users(id) ON DELETE SET NULL;
-- Bản cũ của script trước 0032 gắn vào lịch cùng loại để tính số bản cần xoá.
UPDATE backup_runs r SET lich_id = l.id FROM bk_lich_sao_luu l WHERE r.lich_id IS NULL AND l.loai = r.loai;
CREATE INDEX IF NOT EXISTS ix_backup_runs_lich ON backup_runs (lich_id, created_at DESC) WHERE ket_qua = 'dat' AND da_xoa_at IS NULL;

-- 7. Sự cố và phục hồi (Điều 31): gắn bản sao lưu đã dùng để phục hồi; biên bản Mẫu 04 lập từ đây.
CREATE TABLE IF NOT EXISTS bk_su_co (
    id                  uuid PRIMARY KEY,
    ma                  text NOT NULL UNIQUE,
    project_id          uuid REFERENCES projects(id) ON DELETE SET NULL,
    thoi_diem           timestamptz NOT NULL,
    mo_ta               text NOT NULL,
    nguyen_nhan         text,
    muc_do              text NOT NULL DEFAULT 'trung_binh' CHECK (muc_do IN ('thap', 'trung_binh', 'cao', 'nghiem_trong')),
    pham_vi             text,
    bien_phap           text,
    backup_run_id       uuid REFERENCES backup_runs(id) ON DELETE SET NULL,
    phuong_phap_phuc_hoi text,
    phuc_hoi_luc        timestamptz,
    ket_qua_kiem_tra    text,
    du_lieu_mat         text,
    trang_thai          text NOT NULL DEFAULT 'moi' CHECK (trang_thai IN ('moi', 'dang_xu_ly', 'da_phuc_hoi', 'dong')),
    nguoi_xu_ly         uuid REFERENCES users(id) ON DELETE SET NULL,
    record_id           uuid REFERENCES records(id) ON DELETE SET NULL,
    created_by          uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS ix_bk_su_co_tt ON bk_su_co (trang_thai, thoi_diem DESC);

-- 8. Màn Bảo quản tự làm mới theo topic 'ops' (như integrity_checks / backup_runs).
DO $$
DECLARE t text;
BEGIN
    FOREACH t IN ARRAY ARRAY['bk_ke_hoach', 'bk_ke_hoach_tep', 'bk_loi', 'bk_lich_sao_luu', 'bk_phuong_tien', 'bk_su_co'] LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS tg_%1$s_changed ON %1$s', t);
        EXECUTE format('CREATE TRIGGER tg_%1$s_changed AFTER INSERT OR DELETE OR UPDATE ON %1$s FOR EACH STATEMENT EXECUTE FUNCTION note_change(''ops'')', t);
    END LOOP;
END $$;

COMMIT;
