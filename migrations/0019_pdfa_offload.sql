-- 15/10/2026: bản PDF/A đã xuất ra lưu trữ ngoài (ổ ngoài, xếp theo hồ sơ) và xoá khỏi kho máy chủ.
-- Trạng thái tài liệu giữ nguyên (đã PDF/A / đã ký); bản ký số vẫn ở kho để đóng gói.
ALTER TABLE public.documents ADD COLUMN IF NOT EXISTS pdfa_offloaded_at timestamptz;
ALTER TABLE public.documents ADD COLUMN IF NOT EXISTS pdfa_offload_note text;
