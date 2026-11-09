-- 9/11/2026 — Webhook nhận dự án từ hệ thống ngoài (CRM/quản lý hợp đồng).
-- Hệ thống kia gọi POST /api/v1/hooks/du-an, Hệ thống tự tạo dự án + ghi khối lượng hợp đồng.
-- Độc lập với 0026 (bản Lưu trữ) — áp được ngay dù 0026 chưa áp.
BEGIN;

-- Loại việc của dự án và dấu vết nguồn gốc khi dự án do máy khác bắn về.
ALTER TABLE projects
    ADD COLUMN IF NOT EXISTS loai       text,
    ADD COLUMN IF NOT EXISTS nguon      text,
    ADD COLUMN IF NOT EXISTS ma_ngoai   text,
    -- true = mã dự án do Hệ thống tự đặt vì gói tin không gửi mã; người quản trị nên xem lại.
    ADD COLUMN IF NOT EXISTS ma_tu_sinh boolean NOT NULL DEFAULT false;

COMMENT ON COLUMN projects.loai IS
    'Loại việc: so_hoa | chinh_ly | chinh_ly_so_hoa | luu_tru | khac';
COMMENT ON COLUMN projects.nguon IS
    'Hệ thống bắn dự án về qua webhook (api_clients.nguon); NULL = tạo trong Hệ thống';
COMMENT ON COLUMN projects.ma_ngoai IS
    'Mã dự án/hợp đồng ở hệ thống nguồn — khoá chống tạo trùng khi webhook gửi lại';

-- Một mã ngoài chỉ ứng với một dự án. Webhook bao giờ cũng gửi lại khi nghi thất bại,
-- nên chỗ này là chốt duy nhất chặn việc tạo bốn dự án giống nhau.
CREATE UNIQUE INDEX IF NOT EXISTS ux_projects_nguon_ma_ngoai
    ON projects (nguon, ma_ngoai)
    WHERE nguon IS NOT NULL AND ma_ngoai IS NOT NULL;

-- Khoá cấp cho từng hệ thống ngoài. Thu hồi bằng is_active = false, không phải khởi động lại API.
CREATE TABLE IF NOT EXISTS api_clients (
    id               uuid PRIMARY KEY,
    -- Tên nguồn ngắn, không dấu: đi vào projects.nguon. Mỗi hệ thống một dòng.
    nguon            text NOT NULL UNIQUE,
    ten              text NOT NULL,
    -- Phần công khai của khoá, hệ thống kia gửi ở đầu header X-DigiText-Key.
    key_id           text NOT NULL UNIQUE,
    -- argon2 của phần bí mật — không lưu bí mật dạng rõ.
    secret_hash      text NOT NULL,
    -- Bí mật ký HMAC-SHA256 (hex). NULL = không bắt buộc ký, chỉ cần khoá đúng.
    secret_hmac      text,
    -- Dự án tạo qua webhook được gán cho người này làm quản trị; NULL = chỉ super admin thấy.
    default_admin_id uuid REFERENCES users(id) ON DELETE SET NULL,
    -- Danh sách IP được phép gọi; NULL/rỗng = không lọc IP.
    ip_allow         text[],
    is_active        boolean NOT NULL DEFAULT true,
    ghi_chu          text,
    created_at       timestamptz NOT NULL DEFAULT now(),
    last_seen_at     timestamptz
);

-- Nhật ký nhận: gói gốc và kết quả, để đối chiếu khi bên kia bảo "đã bắn rồi".
CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id         bigserial PRIMARY KEY,
    client_id  uuid REFERENCES api_clients(id) ON DELETE SET NULL,
    nguon      text,
    event      text,
    ma_ngoai   text,
    payload    jsonb,
    status     integer NOT NULL,
    ket_qua    jsonb,
    loi        text,
    project_id uuid REFERENCES projects(id) ON DELETE SET NULL,
    ip         text,
    nhan_luc   timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_webhook_deliveries_luc
    ON webhook_deliveries (nhan_luc DESC);
CREATE INDEX IF NOT EXISTS ix_webhook_deliveries_ngoai
    ON webhook_deliveries (nguon, ma_ngoai, nhan_luc DESC);

COMMIT;
