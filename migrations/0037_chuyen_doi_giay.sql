-- 0037 (27/11/2026): SỔ CHUYỂN ĐỔI tài liệu lưu trữ số ra giấy — Điều 13–16, Phụ lục VI TT 05/2025/TT-BNV (đợt 4 đối chiếu Digifile, nhóm 3B).
-- Mỗi lần in tài liệu số ra giấy (Điều 16 khoản 1.b: tài liệu số có thể chuyển đổi sang tài liệu giấy để phục vụ sử dụng) ghi một
-- dòng: số chuyển đổi "CD-{năm}-{stt:06}" theo cơ quan (dự án) / năm, tài liệu gốc, mục đích, người nhận, kết luận chữ ký số tại thời
-- điểm chuyển đổi, người thực hiện; PDF đã ghép trang thể thức "TÀI LIỆU LƯU TRỮ CHUYỂN ĐỔI" + chân trang lưu ở `pdf_key`
-- (`{project}-chuyen-doi/{id}.pdf`, không ký số lại — bản in ký tay + đóng dấu). Nhật ký khai thác ghi thêm `khai_thac.chuyen_doi`.
-- Áp bằng `digitext-api migrate` hoặc tay theo thứ tự.
BEGIN;

CREATE TABLE IF NOT EXISTS chuyen_doi_giay (
    id               uuid PRIMARY KEY,
    project_id       uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    document_id      uuid REFERENCES documents(id) ON DELETE SET NULL,
    ho_so_id         uuid REFERENCES cl_ho_so(id) ON DELETE SET NULL,
    van_ban_id       uuid REFERENCES cl_van_ban(id) ON DELETE SET NULL,
    phong_id         uuid REFERENCES cl_phong(id) ON DELETE SET NULL,
    nam              integer NOT NULL,
    so_chuyen_doi    text NOT NULL,
    ma_tai_lieu      text,
    ten_tai_lieu     text,
    ma_ho_so         text,
    so_trang         integer,
    muc_dich         text,
    nguoi_nhan       text,
    can_cu           text,
    ky_so_ket_luan   text,
    ky_so_chi_tiet   text,
    nguoi_thuc_hien  uuid REFERENCES users(id) ON DELETE SET NULL,
    chuc_vu          text,
    pdf_key          text NOT NULL,
    dung_luong_bytes bigint,
    created_at       timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_chuyen_doi_so ON chuyen_doi_giay (project_id, so_chuyen_doi);
CREATE INDEX IF NOT EXISTS ix_chuyen_doi_du_an ON chuyen_doi_giay (project_id, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_chuyen_doi_tai_lieu ON chuyen_doi_giay (document_id) WHERE document_id IS NOT NULL;

DROP TRIGGER IF EXISTS tg_chuyen_doi_giay_changed ON chuyen_doi_giay;
CREATE TRIGGER tg_chuyen_doi_giay_changed AFTER INSERT OR UPDATE OR DELETE ON chuyen_doi_giay
    FOR EACH STATEMENT EXECUTE FUNCTION note_change('records');

COMMIT;
