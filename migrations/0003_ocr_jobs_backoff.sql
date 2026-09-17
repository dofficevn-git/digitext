-- Thêm cột not_before cho ocr_jobs: thời điểm sớm nhất job được claim lại.
--
-- VÌ SAO: lượt chạy 110 tài liệu ngày 2026-09-17 cho thấy khi dịch vụ OCR
-- không phản hồi, worker đốt hết 5 lần thử trong vài giây (không có độ trễ
-- giữa các lần), rồi bỏ trang đó vĩnh viễn (failed_transient). Backoff phải
-- nằm trong DB chứ không trong tiến trình: có nhiều lane claim song song, và
-- restart worker không được xoá độ trễ đang chờ.
--
-- Cột nullable, cộng thêm — code cũ bỏ qua cột này nên rollback = để nguyên.

ALTER TABLE public.ocr_jobs
    ADD COLUMN IF NOT EXISTS not_before timestamp with time zone;

-- Claim lọc theo not_before; chỉ số ít job đang chờ backoff nên index bán phần.
CREATE INDEX IF NOT EXISTS ix_ocr_jobs_not_before
    ON public.ocr_jobs (not_before)
    WHERE status = 'queued' AND not_before IS NOT NULL;
