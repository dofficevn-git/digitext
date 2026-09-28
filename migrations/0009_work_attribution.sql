-- 28/9/2026 — đề nghị chỉnh sửa mục 11 (Báo cáo nhân viên): ghi nhận AI nhập / AI xác nhận từng
-- trường dữ liệu, và sản lượng GIAO cho từng người theo công việc để tính % hoàn thành + KPI.
-- (Người quét = documents.uploaded_by — máy quản trị đẩy hộ bằng onBehalfOf; người xác nhận tài
-- liệu = documents.reviewed_by — đã có sẵn.)
ALTER TABLE ocr_fields
    ADD COLUMN IF NOT EXISTS edited_by   uuid REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS edited_at   timestamptz,
    ADD COLUMN IF NOT EXISTS verified_by uuid REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS verified_at timestamptz;
CREATE INDEX IF NOT EXISTS ix_ocr_fields_edited ON ocr_fields (edited_by, edited_at) WHERE edited_by IS NOT NULL;
CREATE INDEX IF NOT EXISTS ix_documents_reviewed ON documents (reviewed_by, reviewed_at) WHERE reviewed_by IS NOT NULL;

CREATE TABLE IF NOT EXISTS work_targets (
    project_id  uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id     uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    task        text NOT NULL CHECK (task IN ('scan', 'kiem_tra_scan', 'nhap_lieu', 'kiem_tra_nhap_lieu', 'xac_nhan', 'kiem_tra_xac_nhan')),
    quantity    bigint NOT NULL CHECK (quantity >= 0),
    updated_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, user_id, task)
);
