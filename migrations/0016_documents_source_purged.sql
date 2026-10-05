-- 5/10/2026 — Không giữ PDF gốc sau khi đã có bản PDF/A-2b (bản PDF/A nhúng nguyên ảnh JPEG gốc; ký số
-- thì giữ thêm bản đã ký). Cột ghi lúc PDF gốc bị xoá khỏi kho: tải "bản gốc", OCR lại, bóc trường, ghép
-- lại PDF/A đều chuyển sang đọc bản PDF/A. NULL = PDF gốc vẫn còn.
ALTER TABLE documents ADD COLUMN IF NOT EXISTS source_purged_at timestamptz;
