-- 0035 (27/11/2026): KHO / GIÁ có SỨC CHỨA — đợt 3 kế hoạch đối chiếu Digifile TT05 (nhóm 4B, Digifile L07/L21).
-- `cl_gia.suc_chua_hop`: số hộp giá chứa được (NULL = suy từ dải hộp den_hop - tu_hop + 1); `canh_bao_pct`: ngưỡng báo "giá gần đầy"
-- (thông báo `kho_day` cho quản trị dự án, mỗi giá một lần mỗi tuần). `cl_kho.suc_chua_hop`: sức chứa cả kho (chỉ hiển thị).
-- Số hộp / hồ sơ đang chứa tính lúc truy vấn trong digitext_db::chinh_ly::kho_gia (không thêm view để lọc theo dự án đẩy được xuống).
-- Gán hộp cho hồ sơ nhập gói (phong_ban_id NULL) đi qua route /cl/ho-so/gan-hop: đặt phòng ban rồi gọi he_thong_hoa::boxing — không đổi lược đồ.
BEGIN;

ALTER TABLE cl_gia ADD COLUMN IF NOT EXISTS suc_chua_hop integer;
ALTER TABLE cl_gia DROP CONSTRAINT IF EXISTS cl_gia_suc_chua_check;
ALTER TABLE cl_gia ADD CONSTRAINT cl_gia_suc_chua_check CHECK (suc_chua_hop IS NULL OR suc_chua_hop > 0);
ALTER TABLE cl_gia ADD COLUMN IF NOT EXISTS canh_bao_pct integer NOT NULL DEFAULT 90;
ALTER TABLE cl_gia DROP CONSTRAINT IF EXISTS cl_gia_canh_bao_check;
ALTER TABLE cl_gia ADD CONSTRAINT cl_gia_canh_bao_check CHECK (canh_bao_pct BETWEEN 1 AND 100);
ALTER TABLE cl_kho ADD COLUMN IF NOT EXISTS suc_chua_hop integer;

COMMIT;
