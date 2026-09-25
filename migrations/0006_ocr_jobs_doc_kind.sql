-- 25/9/2026: danh sách tài liệu (/documents, LATERAL "việc bóc trường mới nhất" mỗi tài liệu)
-- quét tuần tự ocr_jobs cho TỪNG tài liệu: 29.764 × 0,27 ms ≈ 8–14 s mỗi lần tải → màn
-- Máy chủ/OCR treo, máy chủ nghẽn, tải lên quá 15 s báo "Không kết nối được máy chủ".
-- CONCURRENTLY: không khoá bảng (chạy ngoài transaction).
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_ocr_jobs_doc_kind_created
    ON ocr_jobs (document_id, kind, created_at DESC, id DESC);
