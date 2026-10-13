-- 13/10/2026: worker dò "còn việc chờ" theo loại (ocr_jobs::f_zj) — 146 nghìn việc sign_wait chờ ký làm câu
-- EXISTS quét cả bảng (~870 nghìn dòng/lần), Postgres ăn 4 nhân, MinIO treo ổ. Chỉ mục phủ đủ cột → Index Only Scan.
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_ocr_jobs_queued_kind_nb ON public.ocr_jobs USING btree (kind, not_before) WHERE (status = 'queued'::text);
CREATE STATISTICS IF NOT EXISTS st_ocr_jobs_status_kind (mcv, dependencies) ON status, kind FROM public.ocr_jobs;
