-- 0027 (5/11/2026): quét virus MỘT LẦN cho mỗi tài liệu thay vì quét lại cả gói zip mỗi lần đóng gói.
--
-- Trước: worker đóng gói gửi trọn tệp zip sang clamd sau khi dựng xong. Một hồ sơ đóng cả AIP lẫn SIP
-- thì cùng những tệp PDF đó bị quét hai lần, và việc quét nằm ngay trên đường găng của đóng gói —
-- đo ngày 5/11 thấy clamd ăn gần trọn một nhân CPU cho mỗi luồng đóng gói.
--
-- Sau: kết quả quét của từng tài liệu lưu ở đây. Worker có vòng nền quét dần tài liệu chưa quét; lúc
-- đóng gói chỉ cần đối chiếu, tài liệu nào chưa có kết quả thì quét ngay tại chỗ (bytes đã nằm sẵn
-- trong bộ nhớ) rồi ghi lại để lần đóng gói sau khỏi quét nữa.
--
-- virus_scan: NULL = chưa quét; 'clean (clamd)' = sạch; 'infected: <chữ ký>' = có mã độc;
--             'scan_error: <lý do>' = quét lỗi, sẽ thử lại.

ALTER TABLE documents ADD COLUMN IF NOT EXISTS virus_scan text;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS virus_scan_at timestamptz;

-- Hàng chờ quét: chỉ tài liệu còn sống và chưa có kết quả sạch.
CREATE INDEX IF NOT EXISTS ix_documents_can_quet
    ON documents (uploaded_at)
    WHERE virus_scan IS NULL AND status <> 'disposed';

-- Tra cứu nhanh lúc đóng gói (đối chiếu theo lô id).
CREATE INDEX IF NOT EXISTS ix_documents_virus_scan
    ON documents (id)
    WHERE virus_scan IS NOT NULL;
