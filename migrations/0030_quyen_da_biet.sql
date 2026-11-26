-- 0030 (26/11/2026): bảng "quyền đã biết" để API tự bổ sung QUYỀN MỚI vào 5 vai trò hệ thống (kế hoạch lấp chức năng
-- theo Digifile TT05 — nhóm 0 việc dùng chung, C:\Users\HieuPC\.claude\plans\...functional-cherny.md).
--
-- Vì sao cần: roles::ensure_system (0022) chỉ chèn vai trò CHƯA có và không ghi đè bộ quyền đã có người sửa. Hệ quả: thêm
-- một biến thể Permission mới trong code thì trên mọi máy chủ đang chạy KHÔNG vai trò nào có quyền đó cho tới khi quản trị
-- tick tay — nút mới cứ mờ, dễ tưởng lỗi. Mỗi đợt sắp tới thêm 1–6 quyền (luutru.*, danhmuc.*, bk.approve, loan.approve).
--
-- Cách phân biệt "quyền mới" với "quyền quản trị cố ý bỏ khỏi vai trò": bảng này giữ danh sách mọi quyền code đã từng
-- biết. Lúc khởi động, API lấy Permission::ALL trừ bảng này = quyền mới → chèn vào vai trò hệ thống theo mặc định
-- (granted_by_default) rồi ghi vào bảng. Quyền đã có trong bảng không bao giờ bị chèn lại, nên bỏ tay vẫn giữ.
--
-- Seed đủ 64 quyền của bản 0.25.15 để lần khởi động đầu sau khi áp KHÔNG coi chúng là mới (kẻo trả lại quyền đã bỏ).
--
-- Áp tay: `digitext-api migrate` (hoặc psql -f như các bản trước). Không phụ thuộc migration bản Lưu trữ (0025/0026).
BEGIN;

CREATE TABLE IF NOT EXISTS permissions_seen (
    permission  text PRIMARY KEY,
    seen_at     timestamptz NOT NULL DEFAULT now()
);

INSERT INTO permissions_seen (permission) VALUES
    ('dossier.read'), ('dossier.create'), ('dossier.update'), ('dossier.delete'), ('dossier.export'),
    ('catalog.read'), ('catalog.create'), ('catalog.update'), ('catalog.delete'), ('catalog.import'), ('catalog.export'),
    ('scan.view'), ('scan.execute'), ('editor.view'), ('editor.edit'), ('upload.execute'),
    ('sign.view'), ('sign.execute'), ('drive.view'), ('drive.download'),
    ('document.list'), ('document.download'), ('document.review'),
    ('package.view'), ('package.create'), ('package.download'),
    ('submission.view'), ('submission.create'), ('submission.review'), ('submission.approve'),
    ('record.view'), ('record.create'),
    ('access.view'), ('access.request'), ('access.review'),
    ('disposal.view'), ('disposal.review'), ('disposal.decide'),
    ('bk.view'), ('bk.manage'),
    ('tree.view'), ('tree.assign'),
    ('stats.view'), ('stats.view_project'),
    ('user.view'), ('user.create'), ('user.update'), ('user.revoke'), ('user.reset_password'), ('user.grant_permissions'),
    ('project.view'), ('project.create'), ('project.update'), ('project.assign_admin'),
    ('settings.view'), ('settings.update'),
    ('device.view'), ('device.revoke'),
    ('work.execute'), ('work.manage'),
    ('dossier.systematize'), ('khogia.manage'),
    ('loan.view'), ('loan.manage')
ON CONFLICT (permission) DO NOTHING;

COMMIT;
