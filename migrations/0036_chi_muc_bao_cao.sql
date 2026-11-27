-- 0036 (27/11/2026): CHỈ MỤC cho BÁO CÁO THỐNG KÊ TT05 (Điều 44.3b) — đợt 3 kế hoạch đối chiếu Digifile TT05 (nhóm 3A).
-- Báo cáo "lượt khai thác" đọc audit_log theo dự án (detail->>'projectId') và thời gian; 0026 chỉ có chỉ mục theo thời gian toàn máy chủ.
-- Các bảng khác (integrity_checks, access_requests, info_requests, packages) đã có chỉ mục (project_id, thời gian) từ 0001.
-- CONCURRENTLY → KHÔNG bọc BEGIN/COMMIT (digitext-api migrate chạy từng câu autocommit; áp tay cũng từng câu).
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_audit_log_khai_thac_du_an ON audit_log ((detail->>'projectId'), at DESC) WHERE action LIKE 'khai_thac.%';
