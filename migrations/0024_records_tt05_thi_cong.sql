-- 0024 (3/11/2026): biên bản thi công số hoá theo Điều 9 TT 05/2025/TT-BNV (giao nhận / trả tài liệu giấy, nhật ký số hoá,
-- kiểm tra sản phẩm, nghiệm thu, bàn giao từng lần) + khối lượng hợp đồng theo hộp / hồ sơ / tờ để nghiệm thu.
--
-- Áp tay (thứ tự sau 0023), ví dụ trên OCR PC:
--   psql -h 192.168.100.10 -U digitext -d digitext -v ON_ERROR_STOP=1 -f migrations/0024_records_tt05_thi_cong.sql
-- hoặc máy dev: docker exec -i digitext-dev-postgres-1 psql -U digitext -d digitext -v ON_ERROR_STOP=1 < migrations/0024_records_tt05_thi_cong.sql
--
-- LƯU Ý: thêm loại biên bản mới về sau = thêm kind vào CHECK dưới đây + mẫu ở desktop/src/features/records/recordForms.ts
-- (máy chủ dựng PDF theo khối "generic" trong payload, không cần sửa bienban.rs).
BEGIN;

ALTER TABLE records DROP CONSTRAINT IF EXISTS records_kind_check;
ALTER TABLE records ADD CONSTRAINT records_kind_check CHECK (kind = ANY (ARRAY[
    'kiem_tra', 'nhat_ky_sao_luu', 'sao_luu', 'su_co', 'thu_nop', 'thu_nop_truc_tiep', 'giao_nhan', 'tra_lai',
    -- thi công số hoá (Điều 9):
    'giao_nhan_giay',      -- khoản 1: biên bản giao nhận tài liệu giấy (đơn vị → nhà thầu)
    'tra_tai_lieu_giay',   -- khoản 4 điểm đ: trả tài liệu gốc về kho
    'nhat_ky_so_hoa',      -- khoản 4: nhật ký số hoá theo ngày
    'kiem_tra_san_pham',   -- khoản 5: biên bản kiểm tra sản phẩm
    'nghiem_thu',          -- khoản 8: biên bản nghiệm thu
    'ban_giao_lan'         -- khoản 8: biên bản bàn giao từng lần
]::text[]));

-- Khoá phụ để lọc: mã phông (giao nhận / trả / kiểm tra), ngày YYYY-MM-DD (nhật ký), "lần n" (bàn giao).
ALTER TABLE records ADD COLUMN IF NOT EXISTS ref text;
CREATE INDEX IF NOT EXISTS ix_records_kind_ref ON records (project_id, kind, ref);

-- Khối lượng hợp đồng theo đơn vị TT05 (hộp / hồ sơ / tờ) cạnh trang / tệp đã có (0008).
ALTER TABLE project_contracts
    ADD COLUMN IF NOT EXISTS contract_boxes    bigint,
    ADD COLUMN IF NOT EXISTS contract_dossiers bigint,
    ADD COLUMN IF NOT EXISTS contract_sheets   bigint;

COMMIT;
