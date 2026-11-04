-- 0025 (4/11/2026): nhập gói AIP vào Hệ thống — bản Lưu trữ GĐ3 (docs/KE-HOACH-BAN-LUU-TRU.md). Máy chủ cơ quan nhận gói
-- AIP/SIP đã đóng (TT 05/2025/TT-BNV Điều 9 khoản 7), dựng lại mục lục (cl_phong / cl_ho_so / cl_van_ban / cl_van_ban_file),
-- tài liệu (documents, trạng thái archived) và giữ zip gốc trong kho WORM (packages.nguon = 'nhap').
--
-- Áp tay (thứ tự sau 0024):
--   docker exec -i digitext-dev-postgres-1 psql -U digitext -d digitext -v ON_ERROR_STOP=1 < migrations/0025_nhap_goi_aip.sql
-- Máy chủ thi công áp cũng được (không đổi hành vi cũ: mọi gói hiện có nguon = 'dung').
BEGIN;

-- Gói do máy chủ này dựng ('dung') hay nhập từ nơi khác ('nhap'). Gói nhập có status 'built' ngay, worker không nhặt;
-- request_json của gói nhập là {"nguon":"nhap", ...} — không phải yêu cầu dựng gói (DIP dựng từ zip gốc, GĐ4).
ALTER TABLE packages ADD COLUMN IF NOT EXISTS nguon text NOT NULL DEFAULT 'dung';
ALTER TABLE packages DROP CONSTRAINT IF EXISTS packages_nguon_check;
ALTER TABLE packages ADD CONSTRAINT packages_nguon_check CHECK (nguon IN ('dung', 'nhap'));
CREATE INDEX IF NOT EXISTS ix_packages_objid ON packages (project_id, objid) WHERE objid IS NOT NULL;

-- Tài liệu nhập từ gói: gói nguồn + tình trạng chữ ký số của tệp PDF lúc nhập (pkgcheck xác minh).
ALTER TABLE documents ADD COLUMN IF NOT EXISTS goi_id uuid REFERENCES packages(id) ON DELETE SET NULL;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS ky_so text;
ALTER TABLE documents DROP CONSTRAINT IF EXISTS documents_ky_so_check;
ALTER TABLE documents ADD CONSTRAINT documents_ky_so_check CHECK (ky_so IS NULL OR ky_so IN ('hop_le', 'khong_hop_le', 'chua_ky'));
CREATE INDEX IF NOT EXISTS ix_documents_goi ON documents (goi_id) WHERE goi_id IS NOT NULL;

-- Hồ sơ ↔ gói đã nhập. Bảng riêng (không thêm cột vào cl_ho_so) để đồng bộ offline của desktop không đổi.
CREATE TABLE IF NOT EXISTS cl_ho_so_goi (
    ho_so_id      uuid PRIMARY KEY REFERENCES cl_ho_so(id) ON DELETE CASCADE,
    project_id    uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    package_id    uuid REFERENCES packages(id) ON DELETE SET NULL,
    objid         text NOT NULL,
    -- arcFileCode / fileCode trong EAD hồ sơ — mã hồ sơ TT05 đúng như gói ghi (cl_ho_so_v.ma_ho_so là mã tính lại).
    ma_ho_so_goc  text,
    so_tai_lieu   integer NOT NULL DEFAULT 0,
    so_chua_ky    integer NOT NULL DEFAULT 0,
    created_at    timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_ho_so_goi_objid ON cl_ho_so_goi (project_id, objid);
CREATE INDEX IF NOT EXISTS ix_cl_ho_so_goi_ma ON cl_ho_so_goi (project_id, lower(ma_ho_so_goc));

-- Lô nhập (một lần chạy lệnh / một lần tải lên) và kết quả từng gói — báo cáo nhập và nhật ký.
CREATE TABLE IF NOT EXISTS ingest_batches (
    id           uuid PRIMARY KEY,
    project_id   uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    created_by   uuid REFERENCES users(id) ON DELETE SET NULL,
    -- 'lenh' (digitext-api nhap-goi) | 'tai_len' (màn Nhập gói vào Hệ thống)
    nguon        text NOT NULL,
    ghi_chu      text,
    created_at   timestamptz NOT NULL DEFAULT now(),
    finished_at  timestamptz
);
CREATE INDEX IF NOT EXISTS ix_ingest_batches_project ON ingest_batches (project_id, created_at DESC);

CREATE TABLE IF NOT EXISTS ingest_items (
    id            uuid PRIMARY KEY,
    batch_id      uuid NOT NULL REFERENCES ingest_batches(id) ON DELETE CASCADE,
    project_id    uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    ten_tep       text NOT NULL,
    objid         text,
    loai_goi      text,
    sha256        text,
    size_bytes    bigint,
    -- nhap = đã nhập · da_co = gói (OBJID) đã có, bỏ qua · tu_choi = kiểm gói không đạt · loi = lỗi khi ghi
    ket_qua       text NOT NULL CHECK (ket_qua IN ('nhap', 'da_co', 'tu_choi', 'loi')),
    ket_luan_kiem text,
    so_loi        integer NOT NULL DEFAULT 0,
    so_canh_bao   integer NOT NULL DEFAULT 0,
    package_id    uuid REFERENCES packages(id) ON DELETE SET NULL,
    ho_so_id      uuid REFERENCES cl_ho_so(id) ON DELETE SET NULL,
    so_van_ban    integer NOT NULL DEFAULT 0,
    so_chua_ky    integer NOT NULL DEFAULT 0,
    thong_bao     text,
    chi_tiet      jsonb,
    created_at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS ix_ingest_items_batch ON ingest_items (batch_id, created_at);

-- Lớp chữ của PDF/A trong gói → ocr_results (tìm toàn văn GĐ4). active = false: worker OCR không bao giờ chọn dòng này.
INSERT INTO engine_versions (id, name, role, precision_mode, active, notes)
VALUES ('00000000-0000-4000-8000-0000000e0001', 'lop-chu-goi-nhap', 'recognition', 'none', false,
        'Lớp chữ PDF/A lấy nguyên từ gói AIP/SIP nhập vào Hệ thống (không chạy OCR)')
ON CONFLICT DO NOTHING;

COMMIT;
