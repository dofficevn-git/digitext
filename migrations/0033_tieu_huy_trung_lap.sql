-- 0033 (26/11/2026): QUÉT TRÙNG LẶP đủ Điều 48.2 TT 05/2025/TT-BNV — đợt 2 kế hoạch đối chiếu Digifile TT05.
-- Trước đây worker chỉ bắt trùng (dự án, mã hồ sơ, tên tệp). Nay thêm hai nhánh trong `digitext_db::disposal`:
--   • trùng nội dung tệp: cùng dự án, cùng blake3 (tài liệu nhập từ gói AIP luôn có blake3 của bản gốc);
--   • trùng mục lục: hai hồ sơ cùng phông, cùng tiêu đề (không dấu), cùng số ký hiệu, cùng năm kết thúc → tài liệu của hồ sơ lập
--     SAU vào danh mục xem xét, `dup_ho_so_id` trỏ hồ sơ gốc. Chỉ đưa vào `xem_xet`, KHÔNG tự đánh cl_ho_so.trung_thua (cột đó
--     đồng bộ offline, người chỉnh lý quyết).
-- Phụ thuộc 0026 (hàm khong_dau). Tách khỏi 0032 để máy chủ thi công áp độc lập.
BEGIN;

ALTER TABLE disposal_entries ADD COLUMN IF NOT EXISTS phat_hien text NOT NULL DEFAULT 'ten_tep';
ALTER TABLE disposal_entries DROP CONSTRAINT IF EXISTS disposal_entries_phat_hien_check;
ALTER TABLE disposal_entries ADD CONSTRAINT disposal_entries_phat_hien_check CHECK (phat_hien IN ('ten_tep', 'blake3', 'muc_luc', 'han', 'tay'));
UPDATE disposal_entries SET phat_hien = 'han' WHERE ly_do = 'het_thoi_han' AND phat_hien = 'ten_tep';
ALTER TABLE disposal_entries ADD COLUMN IF NOT EXISTS dup_ho_so_id uuid REFERENCES cl_ho_so(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS ix_documents_blake3 ON documents (project_id, blake3) WHERE status <> 'disposed';
CREATE INDEX IF NOT EXISTS ix_cl_ho_so_trung ON cl_ho_so (phong_id, khong_dau(tieu_de), so_ky_hieu_hs, nam_ket_thuc) WHERE deleted_at IS NULL;

COMMIT;
