-- 5/11/2026 — Dự án H31.111 XÃ VIỆT YÊN: gỡ dây chuyền kẹt khiến hai phông 04 và 05 "Chờ đẩy" mãi.
--
-- Anh Hiếu đã bấm xoá đúng những dòng này trong app nhưng bị từ chối theo dây chuyền:
--   hai phông mới mã 04/05   <- mã đang thuộc XA HAI THANG / XA THIEN PHIEN
--   hai phông cũ đó          <- còn 3 phòng ban đang dùng
--   ba phòng ban             <- mỗi phòng còn 1 hộp
--   ba hộp (đều RỖNG)        <- lệnh xoá bị từ chối vì lúc đó máy ngoại tuyến
--
-- Ở đây xoá mềm đúng thứ tự (hộp -> phòng ban -> phông), giống hệt rows::soft_delete: đặt
-- deleted_at + updated_by, KHÔNG tăng ver. Không ghi cl_lich_su vì bảng đó chỉ nhận
-- entity 'ho_so' / 'van_ban', và spec của các bảng danh mục đều để history = false —
-- app cũng không ghi lịch sử cho phông / phòng ban / hộp.
-- Chỉ số ux_cl_phong_ma bỏ qua dòng đã xoá nên mã 04 và 05 được trả lại.

\set ON_ERROR_STOP on

BEGIN;

\set du_an '01a17607-181e-7452-a3f7-d1dc4c85aaed'
\set nguoi '01a0ad12-e95c-7021-ab02-ffbdce378845'

-- Chốt an toàn: dừng nếu còn hồ sơ nào nằm trong hộp / phòng ban / phông sắp xoá.
DO $$
DECLARE n int;
BEGIN
    SELECT count(*) INTO n FROM cl_ho_so
     WHERE deleted_at IS NULL
       AND (hop_id IN ('01a19f4c-9066-7b43-9cd2-147ceab6f3e3','01a19f4c-90a2-7f03-b299-aac20e18bc5c','01a19f4e-5ccd-7e53-a7b9-87377972ced8')
         OR phong_ban_id IN ('01a19f4c-8b69-7901-a8db-5220131fd039','01a19f4c-8e46-73a0-9003-834afdcae041','01a19f4e-55d0-7d40-aa8f-46ac306225e1')
         OR phong_id IN ('01a19f4a-8212-79a1-8ae0-00997aaf9aa3','01a19f4a-a4c8-7873-9b0d-d65905d645d6'));
    IF n > 0 THEN
        RAISE EXCEPTION 'DUNG LAI: còn % hồ sơ đang nằm trong hộp/phòng ban/phông sắp xoá', n;
    END IF;
END $$;

-- 1. Ba hộp rỗng của Nội vụ, Tài nguyên, Tài chính - Kế toán.
UPDATE cl_hop SET deleted_at = now(), updated_by = :'nguoi'
 WHERE id IN ('01a19f4c-9066-7b43-9cd2-147ceab6f3e3','01a19f4c-90a2-7f03-b299-aac20e18bc5c','01a19f4e-5ccd-7e53-a7b9-87377972ced8')
   AND deleted_at IS NULL;

-- 2. Ba phòng ban.
UPDATE cl_phong_ban SET deleted_at = now(), updated_by = :'nguoi'
 WHERE id IN ('01a19f4c-8b69-7901-a8db-5220131fd039','01a19f4c-8e46-73a0-9003-834afdcae041','01a19f4e-55d0-7d40-aa8f-46ac306225e1')
   AND deleted_at IS NULL;

-- 3. Hai phông cũ đang giữ mã 04 và 05.
UPDATE cl_phong SET deleted_at = now(), updated_by = :'nguoi'
 WHERE id IN ('01a19f4a-8212-79a1-8ae0-00997aaf9aa3','01a19f4a-a4c8-7873-9b0d-d65905d645d6')
   AND deleted_at IS NULL;

-- Kiểm trước khi chốt: mã 04 và 05 phải không còn dòng sống nào.
DO $$
DECLARE n int;
BEGIN
    SELECT count(*) INTO n FROM cl_phong
     WHERE project_id = '01a17607-181e-7452-a3f7-d1dc4c85aaed'
       AND deleted_at IS NULL AND ma_du_an IN ('04','05');
    IF n > 0 THEN
        RAISE EXCEPTION 'DUNG LAI: mã 04/05 vẫn còn % dòng sống', n;
    END IF;
END $$;

COMMIT;

-- Danh sách phông còn lại sau khi gỡ.
SELECT ma_du_an AS ma, ten FROM cl_phong
 WHERE project_id = '01a17607-181e-7452-a3f7-d1dc4c85aaed' AND deleted_at IS NULL
 ORDER BY ma_du_an, ten;
