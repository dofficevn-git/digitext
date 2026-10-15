-- 15/10/2026: vai trò "Quản lý" (manager) — xem và điều hành mọi dự án, không được xoá.
ALTER TABLE public.users DROP CONSTRAINT IF EXISTS users_role_check;
ALTER TABLE public.users ADD CONSTRAINT users_role_check
    CHECK ((role = ANY (ARRAY['super_admin'::text, 'manager'::text, 'admin'::text, 'user'::text, 'customer'::text])));
