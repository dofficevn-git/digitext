-- 0038 (27/11/2026): PHÂN QUYỀN THEO PHÔNG — đợt 4 đối chiếu Digifile TT05 (nhóm 4C, Digifile L04).
-- `user_phong`: người dùng được thấy những phông nào trong một dự án. KHÔNG CÓ DÒNG = không giới hạn (tương thích ngược với mọi tài
-- khoản hôm nay); super_admin / manager luôn thấy hết. Áp ở các đường ĐỌC MỤC LỤC: /cl/ho-so, /cl/van-ban, /cl/lich-su, /cl/catalog,
-- /tra-cuu*, /tra-cuu/tai-lieu/{id}/doc, mượn trả, kho / giá, báo cáo, thống kê, hệ thống hoá, nhập mục lục, và cả /cl/sync pull
-- (bản sao offline chỉ còn phông được cấp). Tài liệu chưa nối mục lục (kho tệp thi công) vẫn theo dự án.
-- Áp bằng `digitext-api migrate` hoặc tay theo thứ tự.
BEGIN;

CREATE TABLE IF NOT EXISTS user_phong (
    user_id     uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    phong_id    uuid NOT NULL REFERENCES cl_phong(id) ON DELETE CASCADE,
    project_id  uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    added_by    uuid REFERENCES users(id) ON DELETE SET NULL,
    added_at    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, phong_id)
);
CREATE INDEX IF NOT EXISTS ix_user_phong_du_an ON user_phong (project_id, user_id);

DROP TRIGGER IF EXISTS tg_user_phong_changed ON user_phong;
CREATE TRIGGER tg_user_phong_changed AFTER INSERT OR UPDATE OR DELETE ON user_phong
    FOR EACH STATEMENT EXECUTE FUNCTION note_change('users');

COMMIT;
