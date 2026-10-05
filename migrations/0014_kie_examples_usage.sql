-- 5/10/2026 — Kho ví dụ bóc trường: đếm số lần mỗi ví dụ được đưa cho VLM (màn quản lý ví dụ cần biết
-- ví dụ nào đang được dùng nhiều — một ví dụ sai mà được dùng nhiều thì phải tắt trước tiên).
ALTER TABLE kie_examples ADD COLUMN IF NOT EXISTS used_count   bigint NOT NULL DEFAULT 0;
ALTER TABLE kie_examples ADD COLUMN IF NOT EXISTS last_used_at timestamptz;
