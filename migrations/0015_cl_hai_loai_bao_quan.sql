-- 0015 — Chỉnh lý: đánh số riêng cho 2 loại bảo quản lớn — VĨNH VIỄN và CÓ THỜI HẠN (anh Hiếu 5/10/2026).
--
-- Mục lục thật đánh lại từ đầu cho mỗi loại: tệp Vĩnh viễn hồ sơ 1–352 / hộp 01–89, tệp có thời hạn hồ sơ 1–104 /
-- hộp 01–35, cùng một phòng ban. THBQ đổi theo dòng trong một tệp (hộp 01 có cả 50 năm lẫn 20 năm) nên phạm vi là
-- LOẠI (vĩnh viễn hay không), không phải từng THBQ. Loại suy từ cl_thbq.trang_thai = 1.
--
-- Áp tay: docker exec -i digitext-dev-postgres-1 psql -U digitext -d digitext < migrations/0015_cl_hai_loai_bao_quan.sql
BEGIN;

-- 1. Loại của hồ sơ: cột do trigger đặt theo THBQ (máy chủ quyết định; desktop nhận qua row_json).
ALTER TABLE cl_ho_so ADD COLUMN IF NOT EXISTS vinh_vien boolean NOT NULL DEFAULT false;

CREATE OR REPLACE FUNCTION cl_ho_so_loai() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    hop_vv boolean;
    hop_so text;
BEGIN
    NEW.vinh_vien := COALESCE((SELECT t.trang_thai = 1 FROM cl_thbq t WHERE t.id = NEW.thbq_id), false);
    -- Hộp thuộc đúng một loại: không cho hồ sơ nằm trong hộp khác loại (vd. đổi THBQ vĩnh viễn → 20 năm khi đã xếp hộp).
    IF NEW.hop_id IS NOT NULL AND NEW.deleted_at IS NULL THEN
        SELECT h.vinh_vien, h.hop_so INTO hop_vv, hop_so FROM cl_hop h WHERE h.id = NEW.hop_id;
        IF hop_vv IS NOT NULL AND hop_vv <> NEW.vinh_vien THEN
            RAISE EXCEPTION 'Hồ sơ đã xếp hộp % (%) — gỡ khỏi hộp trước khi đổi sang loại bảo quản khác',
                hop_so, CASE WHEN hop_vv THEN 'vĩnh viễn' ELSE 'có thời hạn' END
                USING ERRCODE = 'check_violation', CONSTRAINT = 'ck_cl_ho_so_hop_loai';
        END IF;
    END IF;
    RETURN NEW;
END $$;

DROP TRIGGER IF EXISTS tg_cl_ho_so_loai ON cl_ho_so;
CREATE TRIGGER tg_cl_ho_so_loai BEFORE INSERT OR UPDATE OF thbq_id, hop_id, deleted_at ON cl_ho_so
    FOR EACH ROW EXECUTE FUNCTION cl_ho_so_loai();

-- Backfill (không chạm ver/updated_at: chỉ cột suy ra). Tắt trigger touch tạm để không làm bẩn đồng bộ.
ALTER TABLE cl_ho_so DISABLE TRIGGER USER;
UPDATE cl_ho_so h SET vinh_vien = COALESCE(t.trang_thai = 1, false)
FROM cl_ho_so h2 LEFT JOIN cl_thbq t ON t.id = h2.thbq_id
WHERE h2.id = h.id AND h.vinh_vien IS DISTINCT FROM COALESCE(t.trang_thai = 1, false);
ALTER TABLE cl_ho_so ENABLE TRIGGER USER;

-- Hộp hiện có: loại theo hồ sơ trong hộp (đa số), giữ nguyên nếu hộp rỗng.
ALTER TABLE cl_hop DISABLE TRIGGER USER;
UPDATE cl_hop hp SET vinh_vien = x.vv
FROM (SELECT hop_id, bool_or(vinh_vien) AS vv FROM cl_ho_so WHERE hop_id IS NOT NULL AND deleted_at IS NULL GROUP BY hop_id) x
WHERE x.hop_id = hp.id AND hp.vinh_vien IS DISTINCT FROM x.vv;
ALTER TABLE cl_hop ENABLE TRIGGER USER;

-- 2. Duy nhất theo loại (giữ tên index để thông báo lỗi khớp).
DROP INDEX IF EXISTS ux_cl_ho_so_so_tam;
CREATE UNIQUE INDEX ux_cl_ho_so_so_tam ON cl_ho_so (phong_id, vinh_vien, lower(btrim(so_tam))) WHERE deleted_at IS NULL;
DROP INDEX IF EXISTS ux_cl_hop_so;
CREATE UNIQUE INDEX ux_cl_hop_so ON cl_hop (phong_ban_id, vinh_vien, lower(btrim(hop_so))) WHERE deleted_at IS NULL;
DROP INDEX IF EXISTS ix_cl_hop_int;
CREATE INDEX ix_cl_hop_int ON cl_hop (phong_ban_id, vinh_vien, hop_so_int) WHERE deleted_at IS NULL;

-- 3. Giá theo khoảng hộp: áp cho một loại hoặc cả hai (NULL).
ALTER TABLE cl_gia ADD COLUMN IF NOT EXISTS vinh_vien boolean;

-- 4. Mục lục số riêng cho loại có thời hạn (mã hồ sơ: ... .<hồ sơ số>.<mục lục số>).
ALTER TABLE cl_phong ADD COLUMN IF NOT EXISTS muc_luc_so_cth text;

-- 5. View: h.* có cột mới → tạo lại.
DROP VIEW IF EXISTS cl_ho_so_v;
CREATE VIEW cl_ho_so_v AS
SELECT h.*,
       p.ma_du_an, p.phong_so, p.muc_luc_so, p.muc_luc_so_cth, p.ten AS ten_phong,
       pb.ten AS ten_phong_ban, pb.ma_don_vi,
       m.ma_mang, m.ten AS ten_mang, m.so_tt AS mang_so_tt,
       t.ten AS ten_thbq, t.trang_thai AS thbq_trang_thai, t.gia_tri AS thbq_gia_tri, t.so_tt AS thbq_so_tt, t.ma_tt05 AS thbq_ma_tt05,
       cl_thbq_status(t.trang_thai, t.gia_tri, h.nam_ket_thuc, extract(year from now())::int) AS thbq_status,
       hp.hop_so, hp.hop_so_int,
       bo.so AS bo_so, tp.so AS tap_so,
       COALESCE(h.gia_id, gd.id) AS gia_eff_id,
       COALESCE(gm.ten, gd.ten) AS ten_gia,
       COALESCE(km.id, kd.id) AS kho_eff_id,
       COALESCE(km.ten, kd.ten) AS ten_kho,
       CASE WHEN h.ho_so_so IS NOT NULL AND btrim(h.ho_so_so) <> '' AND h.nam_ket_thuc IS NOT NULL
            THEN concat_ws('.', COALESCE(NULLIF(btrim(p.ma_du_an), ''), '00'), COALESCE(NULLIF(btrim(p.phong_so), ''), '00'),
                           h.nam_ket_thuc::text, btrim(h.ho_so_so),
                           CASE WHEN h.vinh_vien THEN NULLIF(btrim(p.muc_luc_so), '')
                                ELSE COALESCE(NULLIF(btrim(p.muc_luc_so_cth), ''), NULLIF(btrim(p.muc_luc_so), '')) END) END AS ma_ho_so,
       (SELECT count(*) FROM cl_van_ban v WHERE v.ho_so_id = h.id AND v.deleted_at IS NULL) AS so_van_ban,
       EXISTS (SELECT 1 FROM cl_muon_tra mt WHERE mt.ho_so_id = h.id AND mt.trang_thai = 'dang_muon' AND mt.deleted_at IS NULL) AS dang_muon,
       cu.display_name AS created_by_name, uu.display_name AS updated_by_name
FROM cl_ho_so h
JOIN cl_phong p ON p.id = h.phong_id
LEFT JOIN cl_phong_ban pb ON pb.id = h.phong_ban_id
LEFT JOIN cl_mang m ON m.id = h.mang_id
LEFT JOIN cl_thbq t ON t.id = h.thbq_id
LEFT JOIN cl_hop hp ON hp.id = h.hop_id
LEFT JOIN cl_bo_tap bo ON bo.id = h.bo_id
LEFT JOIN cl_bo_tap tp ON tp.id = h.tap_id
LEFT JOIN cl_gia gm ON gm.id = h.gia_id
LEFT JOIN cl_kho km ON km.id = gm.kho_id
LEFT JOIN LATERAL (
    SELECT g.id, g.ten, g.kho_id FROM cl_gia g
    WHERE h.gia_id IS NULL AND g.deleted_at IS NULL AND g.phong_ban_id = h.phong_ban_id
      AND (g.vinh_vien IS NULL OR g.vinh_vien = hp.vinh_vien)
      AND hp.hop_so_int IS NOT NULL AND hp.hop_so_int BETWEEN g.tu_hop AND g.den_hop
    ORDER BY g.tu_hop LIMIT 1
) gd ON true
LEFT JOIN cl_kho kd ON kd.id = gd.kho_id
LEFT JOIN users cu ON cu.id = h.created_by
LEFT JOIN users uu ON uu.id = h.updated_by;

COMMIT;
