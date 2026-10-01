-- 0013 (1/10/2026): gom sự kiện "thay đổi" — note_change() bỏ qua khi cùng topic đã có sự kiện trong khoảng
-- ngắn vừa rồi (ocr / ocrFields: 15 s; còn lại: 3 s). Trước đây worker OCR xong mỗi trang là một sự kiện
-- (≈ 7.000 / giờ), mọi máy trạm nạp lại thống kê 160.000 tài liệu theo từng sự kiện → Postgres kín CPU,
-- đăng nhập và danh sách chậm, thanh trạng thái báo "Mất kết nối".
-- Áp bằng: docker exec -i digitext-dev-postgres-1 psql -U digitext -d digitext < migrations/0013_change_log_throttle.sql

CREATE INDEX IF NOT EXISTS ix_change_log_topic_at ON change_log (topic, at DESC);

CREATE OR REPLACE FUNCTION note_change() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    t   text     := TG_ARGV[0];
    gap interval := CASE t
                        WHEN 'ocr' THEN interval '15 seconds'
                        WHEN 'ocrFields' THEN interval '15 seconds'
                        ELSE interval '3 seconds'
                    END;
BEGIN
    IF EXISTS (SELECT 1 FROM change_log WHERE topic = t AND at > clock_timestamp() - gap) THEN
        RETURN NULL;
    END IF;
    INSERT INTO change_log (topic, at) VALUES (t, clock_timestamp());
    RETURN NULL; -- AFTER STATEMENT: giá trị trả về bị bỏ qua.
END;
$$;
