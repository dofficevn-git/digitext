-- Độ phủ mực theo trang + lý do soát `low_ink_coverage`.
--
-- VÌ SAO: bộ phát hiện dòng (DBNet) sót chữ viết tay — chữ không có khung thì không vào
-- lớp text PDF/A. Từ 21/9/2026 dịch vụ OCR có lưới an toàn tách dòng theo nét mực
-- (ocr/inklines.py) và trả `ink_coverage` = phần nét mực của trang nằm trong một khung
-- chữ. Đây là thước đo "đã bắt hết chữ chưa"; trang dưới ngưỡng
-- (DIGITEXT_REVIEW_MIN_INK_COVERAGE, mặc định 0,90) được đưa vào hàng soát.
--
-- Cột nullable, cộng thêm: trang OCR trước ngày này (và engine Triton) để NULL = "chưa đo".
-- PHẢI áp trước khi chạy worker bản mới (INSERT có cột này).

ALTER TABLE public.ocr_results
    ADD COLUMN IF NOT EXISTS ink_coverage real
    CHECK (ink_coverage IS NULL OR (ink_coverage >= 0 AND ink_coverage <= 1));

ALTER TABLE public.review_queue DROP CONSTRAINT IF EXISTS review_queue_reason_check;
ALTER TABLE public.review_queue ADD CONSTRAINT review_queue_reason_check CHECK (
    reason = ANY (ARRAY[
        'low_confidence'::text, 'heuristic_flag'::text, 'llm_disagreement'::text,
        'page_failed'::text, 'vlm_fallback'::text, 'empty_page'::text, 'vlm_primary'::text,
        'low_ink_coverage'::text
    ])
);
