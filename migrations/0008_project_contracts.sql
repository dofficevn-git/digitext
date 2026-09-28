-- 28/9/2026 — đề nghị chỉnh sửa mục 10 (Báo cáo tiến độ dự án): thông tin hợp đồng của từng dự án
-- để tính % hoàn thành theo khối lượng hợp đồng và tiến độ thời gian.
CREATE TABLE IF NOT EXISTS project_contracts (
    project_id      uuid PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    signed_on       date,          -- ngày ký hợp đồng
    end_on          date,          -- ngày kết thúc hợp đồng
    deadline_on     date,          -- hạn chót hoàn thành
    contract_pages  bigint,        -- số trang theo hợp đồng
    contract_files  bigint,        -- số tệp (tài liệu) theo hợp đồng, nếu có
    manager_name    text,          -- cán bộ phụ trách
    note            text,
    updated_by      uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at      timestamptz NOT NULL DEFAULT now()
);
