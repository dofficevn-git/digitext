-- 25/9/2026: hai tệp CÙNG nội dung (cùng blake3) nhưng KHÁC tên / khác vị trí là hai văn bản
-- khác nhau trong hồ sơ (vd. bản quét lưu nhầm hai số thứ tự) — phải lưu đủ cả hai.
-- Khoá trùng cũ (project_id, blake3) gộp chúng làm một → 7 tệp LTLSBN không lên máy chủ.
-- Khoá mới: cùng dự án + cùng nội dung + cùng tên + cùng đường dẫn mới là một tài liệu.
-- Mỗi tài liệu vẫn có object riêng trong kho (<project>/<doc_id>.pdf) nên không đụng nhau.
BEGIN;
CREATE UNIQUE INDEX IF NOT EXISTS ux_documents_hash_name
    ON documents (project_id, blake3, filename, (COALESCE(rel_path, '')));
CREATE INDEX IF NOT EXISTS ix_documents_hash ON documents (project_id, blake3);
DROP INDEX IF EXISTS ux_documents_hash;
COMMIT;
