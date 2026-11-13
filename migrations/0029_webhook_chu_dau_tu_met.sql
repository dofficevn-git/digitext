-- 11/11/2026 — Hai ô hệ thống ngoài xin thêm cho webhook dự án (thư DO365 ngày 11/11).
--
-- 1. projects.chu_dau_tu — tên cơ quan chủ đầu tư.
--    Trước đó bên gửi phải nhồi tên cơ quan vào `name` ("CƠ QUAN · Nội dung công việc") vì không
--    có chỗ nào khác, làm danh sách dự án bên này toàn dòng na ná nhau. Tách ra thì lọc được
--    theo cơ quan, và `name` quay về đúng nghĩa tên riêng của dự án.
--    KHÔNG phải khoá: một cơ quan có nhiều dự án, và tên cơ quan bên kia gõ tay nên không bền.
--
-- 2. project_contracts.contract_meters — khối lượng hợp đồng tính bằng MÉT GIÁ (mét tài liệu).
--    Phần lớn hợp đồng chỉnh lý khai bằng mét, mà 0008 chỉ có trang/tệp và 0019 thêm
--    hộp/hồ sơ/tờ ⇒ số mét bị bỏ không gửi được.
--    Kiểu numeric(12,2) chứ không phải bigint: mét giá có số lẻ (150.5 m là bình thường).
--    numeric chứ không phải double precision: đây là con số trong hợp đồng, cộng trừ phải khớp
--    đến hai chữ số thập phân, không chấp nhận sai số nhị phân.
BEGIN;

ALTER TABLE projects
    ADD COLUMN IF NOT EXISTS chu_dau_tu text;

ALTER TABLE project_contracts
    ADD COLUMN IF NOT EXISTS contract_meters numeric(12,2);

-- Lọc danh sách dự án theo cơ quan. Chỉ mục trên biểu thức không dấu để gõ "ubnd bac giang"
-- cũng ra: `khong_dau` đã có từ 0026; nếu máy chủ chưa áp 0026 thì bỏ chỉ mục này cũng không sao,
-- cột vẫn dùng được. Dùng DO khối để không chết khi hàm chưa tồn tại.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_proc WHERE proname = 'khong_dau') THEN
        EXECUTE 'CREATE INDEX IF NOT EXISTS ix_projects_chu_dau_tu ON projects (khong_dau(chu_dau_tu)) WHERE chu_dau_tu IS NOT NULL';
    ELSE
        EXECUTE 'CREATE INDEX IF NOT EXISTS ix_projects_chu_dau_tu ON projects (lower(chu_dau_tu)) WHERE chu_dau_tu IS NOT NULL';
    END IF;
END $$;

COMMIT;
