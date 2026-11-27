-- 0034 (27/11/2026): MƯỢN – TRẢ hồ sơ giấy CÓ DUYỆT — đợt 3 kế hoạch đối chiếu Digifile TT05 (nhóm 4A).
-- Trước: phiếu chỉ có dang_muon → da_tra, một phiếu một hồ sơ. Nay: cho_duyet → da_duyet (duyệt, `loan.approve`) → dang_muon (giao)
-- → da_tra; từ chối / huỷ; gia hạn (giữ `han_tra_goc`, nhật ký `gia_han_log`); nhắc sắp hạn / quá hạn (`nhac_lan_cuoi`).
-- Một phiếu nhiều hồ sơ = nhiều dòng cùng `nhom_id` + `so_phieu` ("MT-2026-0001"); duyệt lẻ = từng dòng. Giữ nguyên bảng để
-- không đụng sync.rs / validate.rs / view cl_ho_so_v.dang_muon / thống kê cũ. Máy trạng thái: digitext_core::chinh_ly::muon_tra.
-- Áp bằng `digitext-api migrate` hoặc tay theo thứ tự. Phiếu cũ: nhom_id = id (phiếu một hồ sơ, đã giao), so_phieu NULL.
BEGIN;

ALTER TABLE cl_muon_tra DROP CONSTRAINT IF EXISTS cl_muon_tra_trang_thai_check;
ALTER TABLE cl_muon_tra ADD CONSTRAINT cl_muon_tra_trang_thai_check
    CHECK (trang_thai IN ('cho_duyet', 'da_duyet', 'tu_choi', 'dang_muon', 'da_tra', 'huy'));
ALTER TABLE cl_muon_tra ALTER COLUMN trang_thai SET DEFAULT 'cho_duyet';

ALTER TABLE cl_muon_tra ADD COLUMN IF NOT EXISTS nhom_id         uuid;
ALTER TABLE cl_muon_tra ADD COLUMN IF NOT EXISTS so_phieu        text;
ALTER TABLE cl_muon_tra ADD COLUMN IF NOT EXISTS nguoi_duyet     uuid REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE cl_muon_tra ADD COLUMN IF NOT EXISTS duyet_at        timestamptz;
ALTER TABLE cl_muon_tra ADD COLUMN IF NOT EXISTS ly_do_tu_choi   text;
ALTER TABLE cl_muon_tra ADD COLUMN IF NOT EXISTS giao_boi        uuid REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE cl_muon_tra ADD COLUMN IF NOT EXISTS giao_at         timestamptz;
ALTER TABLE cl_muon_tra ADD COLUMN IF NOT EXISTS han_tra_goc     date;
ALTER TABLE cl_muon_tra ADD COLUMN IF NOT EXISTS so_lan_gia_han  integer NOT NULL DEFAULT 0;
ALTER TABLE cl_muon_tra ADD COLUMN IF NOT EXISTS gia_han_log     jsonb NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE cl_muon_tra ADD COLUMN IF NOT EXISTS nhac_lan_cuoi   date;
ALTER TABLE cl_muon_tra ADD COLUMN IF NOT EXISTS tra_truoc_han   boolean
    GENERATED ALWAYS AS (ngay_thuc_te_tra IS NOT NULL AND ngay_du_kien_tra IS NOT NULL AND ngay_thuc_te_tra < ngay_du_kien_tra) STORED;

UPDATE cl_muon_tra SET nhom_id = id WHERE nhom_id IS NULL;

CREATE INDEX IF NOT EXISTS ix_cl_muon_tra_nhom ON cl_muon_tra (nhom_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_muon_tra_han  ON cl_muon_tra (project_id, ngay_du_kien_tra) WHERE trang_thai = 'dang_muon' AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_muon_tra_so   ON cl_muon_tra (project_id, so_phieu);

COMMIT;
