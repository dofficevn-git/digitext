-- 0023 (2/11/2026): văn bản tạo hàng loạt từ kết quả OCR (POST /cl/van-ban/tu-ocr).
--   cl_van_ban.nguon      'ocr' = máy tạo từ trường OCR; NULL = nhập tay.
--   cl_van_ban.duyet_luc  người soát xác nhận (bấm "Đã duyệt" hoặc Lưu ở màn soát OCR); NULL = chưa duyệt.
--   cl_van_ban.duyet_boi  người duyệt.
--   cl_van_ban_file: một tài liệu máy chủ chỉ gắn một lần vào một văn bản.
--   cl_lich_su: nguồn / hành động 'ocr' — tách khỏi 'add'/'import' để thống kê nhân viên (dashboard_daily 4.10)
--   không cộng văn bản máy tạo cho người bấm nút.
ALTER TABLE cl_van_ban
    ADD COLUMN IF NOT EXISTS nguon     text CHECK (nguon IS NULL OR nguon IN ('ocr')),
    ADD COLUMN IF NOT EXISTS duyet_luc timestamptz,
    ADD COLUMN IF NOT EXISTS duyet_boi uuid REFERENCES users(id) ON DELETE SET NULL;

CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_van_ban_file_doc
    ON cl_van_ban_file (van_ban_id, document_id) WHERE deleted_at IS NULL AND document_id IS NOT NULL;

ALTER TABLE cl_lich_su DROP CONSTRAINT IF EXISTS cl_lich_su_action_check;
ALTER TABLE cl_lich_su ADD CONSTRAINT cl_lich_su_action_check
    CHECK (action IN ('add', 'edit', 'delete', 'restore', 'import', 'systematize', 'boxing', 'assign_gia', 'ocr'));
ALTER TABLE cl_lich_su DROP CONSTRAINT IF EXISTS cl_lich_su_source_check;
ALTER TABLE cl_lich_su ADD CONSTRAINT cl_lich_su_source_check
    CHECK (source IN ('online', 'sync', 'import', 'ocr'));
