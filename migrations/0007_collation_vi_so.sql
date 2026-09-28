-- 28/9/2026: sắp tên thư mục/tệp tiếng Việt theo SỐ ("HO SO 2" trước "HO SO 10") — dùng cho
-- thứ tự xếp hàng OCR (phông → hộp → hồ sơ → tệp → trang), thay vì theo giờ tải lên.
CREATE COLLATION IF NOT EXISTS vi_so (provider = icu, locale = 'vi-u-kn-true');
