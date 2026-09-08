-- Bổ sung bốn cột pdfa_* cho bảng documents.
--
-- VÌ SAO CẦN: schema baseline (0001) được khôi phục bằng pg_dump từ server
-- production ngày 2026-08-27, nhưng mã nguồn trong repo là một ảnh chụp KHÁC —
-- hai bên đã lệch. Code ghi và đọc pdfa_key/pdfa_pages/pdfa_2b/pdfa_at mà
-- schema không có, nên job `make_pdf2` chết với:
--
--     column "pdfa_key" of relation "documents" does not exist
--
-- Lỗi chỉ lộ ra lúc CHẠY, không lộ lúc biên dịch, vì dự án không dùng macro
-- query! của sqlx. Đây chính là cái giá của lựa chọn đó.
--
-- Kiểm kê toàn bộ bằng tools/deobfuscate/audit_schema.py cho thấy đúng bốn cột
-- này thiếu, không còn chỗ nào khác — độ lệch hẹp, chỉ một tính năng.
--
-- Kiểu suy ra từ code, không phải đoán:
--   pdfa_key    crates/digitext-db/src/documents.rs:109  bind(&str)
--   pdfa_pages  crates/digitext-db/src/documents.rs:411  Option<i32>
--   pdfa_2b     crates/digitext-db/src/documents.rs:412  Option<bool>
--   pdfa_at     crates/digitext-db/src/documents.rs:210  dùng trong
--               count(*) FILTER (WHERE d.pdfa_at IS NOT NULL)
-- Tất cả đều nullable: tài liệu chưa ghép PDF hai lớp thì cả bốn đều NULL.

ALTER TABLE public.documents
    ADD COLUMN IF NOT EXISTS pdfa_key   text,
    ADD COLUMN IF NOT EXISTS pdfa_pages integer,
    ADD COLUMN IF NOT EXISTS pdfa_2b    boolean,
    ADD COLUMN IF NOT EXISTS pdfa_at    timestamp with time zone;

-- Truy vấn thống kê ở documents.rs:210-211 lọc theo hai cột này trên toàn bảng.
CREATE INDEX IF NOT EXISTS ix_documents_pdfa_at
    ON public.documents (pdfa_at)
    WHERE pdfa_at IS NOT NULL;
