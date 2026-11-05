-- 0026 (5/11/2026): tra cứu và 4 module Lưu trữ số trên dữ liệu nhập — bản Lưu trữ GĐ4 (docs/KE-HOACH-BAN-LUU-TRU.md).
--   1. Tìm không dấu: hàm khong_dau() (unaccent + lower, IMMUTABLE để dựng chỉ mục) + chỉ mục trigram trên mục lục
--      (văn bản, hồ sơ) và lớp chữ (ocr_results). Dùng ở GET /tra-cuu và tìm của Cung cấp thông tin.
--   2. Bảo quản: integrity_checks ghi được cả kết quả kiểm zip gói (package_id), không chỉ từng tài liệu.
--   3. Tiêu huỷ: packages.huy_* — zip gói AIP chỉ bị xoá khỏi kho WORM khi thực hiện quyết định tiêu huỷ đã huỷ
--      HẾT tài liệu của gói (chốt với anh Hiếu 5/11/2026).
--
-- Áp tay (thứ tự sau 0025):
--   docker exec -i digitext-dev-postgres-1 psql -U digitext -d digitext -v ON_ERROR_STOP=1 < migrations/0026_tra_cuu_luu_tru.sql
-- Máy chủ thi công: API từ bản này dùng khong_dau() trong tìm của Cung cấp thông tin → PHẢI áp. Chỉ mục trên ocr_results
-- dựng có thể lâu khi bảng lớn (OCR PC) — áp ngoài giờ.
BEGIN;

CREATE EXTENSION IF NOT EXISTS unaccent;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- unaccent() chỉ STABLE (phụ thuộc search_path) → bọc với từ điển ghi rõ schema để khai IMMUTABLE. 'Đ/đ' → 'd' có sẵn
-- trong unaccent.rules.
CREATE OR REPLACE FUNCTION khong_dau(t text) RETURNS text
    LANGUAGE sql IMMUTABLE PARALLEL SAFE
    AS $$ SELECT lower(public.unaccent('public.unaccent'::regdictionary, coalesce(t, ''))) $$;

-- Chữ tìm của một văn bản / hồ sơ. Câu tìm trong Rust gọi ĐÚNG các hàm này với ĐÚNG thứ tự cột để trúng chỉ mục.
CREATE OR REPLACE FUNCTION cl_vb_tim(trich_yeu text, so_ky_hieu text, co_quan text, loai_vb text, tu_khoa text) RETURNS text
    LANGUAGE sql IMMUTABLE PARALLEL SAFE
    AS $$ SELECT khong_dau(coalesce(trich_yeu, '') || ' ' || coalesce(so_ky_hieu, '') || ' ' || coalesce(co_quan, '') || ' '
                          || coalesce(loai_vb, '') || ' ' || coalesce(tu_khoa, '')) $$;
CREATE OR REPLACE FUNCTION cl_hs_tim(tieu_de text, so_ky_hieu text, tu_khoa text, so_tam text) RETURNS text
    LANGUAGE sql IMMUTABLE PARALLEL SAFE
    AS $$ SELECT khong_dau(coalesce(tieu_de, '') || ' ' || coalesce(so_ky_hieu, '') || ' ' || coalesce(tu_khoa, '') || ' '
                          || coalesce(so_tam, '')) $$;

CREATE INDEX IF NOT EXISTS ix_cl_van_ban_tim ON cl_van_ban
    USING gin (cl_vb_tim(trich_yeu, so_ky_hieu, co_quan_ban_hanh, loai_vb, tu_khoa) gin_trgm_ops) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_ho_so_tim ON cl_ho_so
    USING gin (cl_hs_tim(tieu_de, so_ky_hieu_hs, tu_khoa, so_tam) gin_trgm_ops) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_ocr_results_khong_dau ON ocr_results USING gin (khong_dau(content) gin_trgm_ops);

-- Kiểm toàn vẹn zip gói: dòng có package_id, document_id rỗng.
ALTER TABLE integrity_checks ALTER COLUMN document_id DROP NOT NULL;
ALTER TABLE integrity_checks ADD COLUMN IF NOT EXISTS package_id uuid REFERENCES packages(id) ON DELETE CASCADE;
ALTER TABLE integrity_checks DROP CONSTRAINT IF EXISTS integrity_checks_doi_tuong_check;
ALTER TABLE integrity_checks ADD CONSTRAINT integrity_checks_doi_tuong_check CHECK (document_id IS NOT NULL OR package_id IS NOT NULL);
CREATE INDEX IF NOT EXISTS ix_integrity_checks_package ON integrity_checks (package_id, checked_at DESC) WHERE package_id IS NOT NULL;

-- Zip gói đã huỷ theo quyết định tiêu huỷ: dòng packages giữ lại làm vết (ai, khi nào, quyết định số).
ALTER TABLE packages ADD COLUMN IF NOT EXISTS huy_at timestamptz;
ALTER TABLE packages ADD COLUMN IF NOT EXISTS huy_quyet_dinh text;

-- Nhật ký khai thác (xem / tải / cấp DIP) nằm trong audit_log với action 'khai_thac.*'.
CREATE INDEX IF NOT EXISTS ix_audit_log_khai_thac ON audit_log (at DESC) WHERE action LIKE 'khai_thac.%';

COMMIT;
