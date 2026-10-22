-- 22/10/2026 — Vai trò động: bộ quyền của 5 vai trò hệ thống thành dữ liệu sửa được, thêm được vai trò tuỳ chỉnh.
-- Trước đây quyền mặc định nằm cứng trong digitext-core::permission (granted_by_default); muốn một nhóm người có
-- bộ quyền riêng phải tick từng quyền cho từng người (user_permission_overrides).
--
-- Mô hình: `users.role` vẫn là VAI TRÒ GỐC (super_admin / manager / admin / user / customer) quyết định tầng
-- (Perm) và phạm vi dự án — mọi kiểm tra cấp hệ thống không đổi. `users.role_code` trỏ tới một dòng `roles`
-- (NULL = dùng đúng vai trò gốc). Quyền hiệu lực = role_permissions[role_code ?? role] ∪ cấp riêng − chặn riêng.
-- 5 vai trò hệ thống được API tự chèn khi khởi động (digitext_db::roles::ensure_system) từ granted_by_default,
-- nên không seed ở đây để khỏi lệch hai nơi.
--
-- Áp tay: docker exec -i digitext-dev-postgres-1 psql -v ON_ERROR_STOP=1 -U digitext -d digitext < migrations/0022_roles.sql

CREATE TABLE IF NOT EXISTS roles (
    code         text PRIMARY KEY CHECK (code ~ '^[a-z][a-z0-9_]{1,39}$'),
    name         text NOT NULL,
    description  text,
    base_role    text NOT NULL CHECK (base_role IN ('super_admin', 'manager', 'admin', 'user', 'customer')),
    is_system    boolean NOT NULL DEFAULT false,
    sort_order   integer NOT NULL DEFAULT 100,
    created_by   uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_code    text NOT NULL REFERENCES roles(code) ON DELETE CASCADE,
    permission   text NOT NULL,
    PRIMARY KEY (role_code, permission)
);

ALTER TABLE users ADD COLUMN IF NOT EXISTS role_code text REFERENCES roles(code) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS ix_users_role_code ON users (role_code) WHERE role_code IS NOT NULL;

-- Đổi vai trò / bộ quyền → máy trạm nạp lại danh sách người dùng và vai trò (topic "users" sẵn có).
DROP TRIGGER IF EXISTS tg_roles_changed ON roles;
CREATE TRIGGER tg_roles_changed AFTER INSERT OR UPDATE OR DELETE ON roles
    FOR EACH STATEMENT EXECUTE FUNCTION note_change('users');
DROP TRIGGER IF EXISTS tg_role_permissions_changed ON role_permissions;
CREATE TRIGGER tg_role_permissions_changed AFTER INSERT OR UPDATE OR DELETE ON role_permissions
    FOR EACH STATEMENT EXECUTE FUNCTION note_change('users');
