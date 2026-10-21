-- 21/10/2026: OCR trang của tài liệu NHẸ trước. Ảnh trang phải chuyển qua USB Wi-Fi của máy chủ (~14 MB/s)
-- sang máy GPU; bản quét khổ lớn (3–16 MB/trang) chiếm đường truyền, trang nhẹ chờ sau. Trang 'ocr' mới
-- xếp hàng ở mức mặc định 100 được hạ theo dung lượng trung bình mỗi trang của tài liệu:
--   < 1 MB/trang → 80 (chạy trước), 1–3 MB/trang → 90, còn lại giữ 100. claim_next lấy priority nhỏ trước.
-- Không đụng job đã đặt mức riêng (≠ 100) — vd. chạy lại trang lỗi ở mức 50.
CREATE OR REPLACE FUNCTION public.ocr_jobs_uu_tien_tep_nhe() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
  mb_trang numeric;
BEGIN
  IF NEW.kind = 'ocr' AND NEW.priority = 100 THEN
    SELECT d.size_bytes::numeric / d.page_count INTO mb_trang
      FROM public.documents d WHERE d.id = NEW.document_id AND d.page_count > 0;
    IF mb_trang IS NOT NULL AND mb_trang > 0 THEN
      IF mb_trang < 1000000 THEN
        NEW.priority := 80;
      ELSIF mb_trang < 3000000 THEN
        NEW.priority := 90;
      END IF;
    END IF;
  END IF;
  RETURN NEW;
END $$;

DROP TRIGGER IF EXISTS tg_ocr_jobs_uu_tien_tep_nhe ON public.ocr_jobs;
CREATE TRIGGER tg_ocr_jobs_uu_tien_tep_nhe
  BEFORE INSERT ON public.ocr_jobs
  FOR EACH ROW EXECUTE FUNCTION public.ocr_jobs_uu_tien_tep_nhe();
