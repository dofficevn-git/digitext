-- 29/9/2026 — Gói việc & cộng tác viên xác nhận trường (đưa logic doc_extract_pla vào DigiText).
-- Quản trị tạo GÓI trong một dự án (theo hồ sơ liền mạch hoặc ngẫu nhiên), giao cho một hay nhiều
-- CTV (tài khoản Nhân viên là thành viên dự án). CTV nhận từng MỤC (= 1 tài liệu) một, khoá mềm
-- 10 phút, sửa 18 trường rồi "Xác nhận hoàn thành"; gói KIỂM TRA chọn mẫu để CTV khác soát chéo;
-- quản trị duyệt / từ chối (kèm lý do), CTV làm lại mục bị trả. Mọi thao tác ghi work_item_events.
CREATE TABLE IF NOT EXISTS work_packages (
    id                 uuid PRIMARY KEY,
    project_id         uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name               text NOT NULL,
    kind               text NOT NULL CHECK (kind IN ('xac_nhan', 'kiem_tra')),
    mode               text NOT NULL CHECK (mode IN ('hoso', 'ngau_nhien')),
    status             text NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'under_review', 'done', 'closed')),
    due_at             date,
    note               text,
    source_package_id  uuid REFERENCES work_packages(id) ON DELETE SET NULL,  -- gói kiem_tra soát gói nào
    sample_pct         integer CHECK (sample_pct IS NULL OR (sample_pct BETWEEN 1 AND 100)),
    created_by         uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS ix_work_packages_project ON work_packages (project_id, status);

-- Thành viên (CTV) của gói — bảng riêng, KHÔNG FK từ gói sang người (xoá CTV không hỏng gói).
CREATE TABLE IF NOT EXISTS work_package_members (
    package_id  uuid NOT NULL REFERENCES work_packages(id) ON DELETE CASCADE,
    user_id     uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    added_by    uuid REFERENCES users(id) ON DELETE SET NULL,
    added_at    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (package_id, user_id)
);
CREATE INDEX IF NOT EXISTS ix_work_package_members_user ON work_package_members (user_id);

CREATE TABLE IF NOT EXISTS work_items (
    id            uuid PRIMARY KEY,
    package_id    uuid NOT NULL REFERENCES work_packages(id) ON DELETE CASCADE,
    kind          text NOT NULL CHECK (kind IN ('xac_nhan', 'kiem_tra')),   -- chép từ gói (cho chỉ mục duy nhất)
    document_id   uuid NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    ord           integer NOT NULL,
    -- pending → done (CTV xác nhận) → checked_ok/checked_err (gói kiểm tra) → approved/rejected (quản trị)
    -- → redone (CTV làm lại mục bị trả); cancelled khi đóng gói dở.
    status        text NOT NULL DEFAULT 'pending'
                  CHECK (status IN ('pending', 'done', 'redone', 'checked_ok', 'checked_err', 'approved', 'rejected', 'cancelled')),
    locked_by     uuid REFERENCES users(id) ON DELETE SET NULL,
    lease_until   timestamptz,
    done_by       uuid REFERENCES users(id) ON DELETE SET NULL,
    done_at       timestamptz,
    checked_by    uuid REFERENCES users(id) ON DELETE SET NULL,
    checked_at    timestamptz,
    check_note    text,
    reviewed_by   uuid REFERENCES users(id) ON DELETE SET NULL,
    reviewed_at   timestamptz,
    review_note   text,
    UNIQUE (package_id, document_id)
);
CREATE INDEX IF NOT EXISTS ix_work_items_package ON work_items (package_id, ord);
CREATE INDEX IF NOT EXISTS ix_work_items_document ON work_items (document_id);
-- Một tài liệu chỉ nằm trong MỘT gói xác nhận còn dang dở (chống giao trùng).
CREATE UNIQUE INDEX IF NOT EXISTS ux_work_items_open_doc ON work_items (document_id)
    WHERE kind = 'xac_nhan' AND status NOT IN ('approved', 'cancelled');

CREATE TABLE IF NOT EXISTS work_item_events (
    id        bigserial PRIMARY KEY,
    item_id   uuid NOT NULL REFERENCES work_items(id) ON DELETE CASCADE,
    actor_id  uuid REFERENCES users(id) ON DELETE SET NULL,
    action    text NOT NULL,   -- claim | release | done | redone | check_ok | check_err | approve | reject | cancel
    note      text,
    at        timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS ix_work_item_events_item ON work_item_events (item_id, at);
