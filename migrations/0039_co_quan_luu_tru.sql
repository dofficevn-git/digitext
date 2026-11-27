-- 0039 (27/11/2026): CẤU HÌNH CƠ QUAN LƯU TRỮ + LỊCH SỬ ĐƠN VỊ HÌNH THÀNH PHÔNG — đợt 5 đối chiếu Digifile TT05 (B01, B02).
-- `co_quan_luu_tru`: một dòng cho một dự án (= một cơ quan ở bản Lưu trữ): mã cơ quan lưu trữ (Phụ lục I), tên / địa chỉ, loại hình
--   (lưu trữ cơ quan | lưu trữ lịch sử), người ký + chức vụ (khối ký báo cáo, trang thể thức chuyển đổi), tham số hạn nộp lưu
--   (Điều 19: 60 ngày, nhắc 15/7/1, quyền sử dụng 360 ngày). KHÔNG CÓ DÒNG = mặc định như hôm nay (tên = projects.name, 60/15-7-1/360).
-- `phong_lich_su`: lịch sử đơn vị hình thành phông / lịch sử phông có PHIÊN BẢN (mỗi lần nộp bổ sung thêm một phiên bản, bản cũ giữ nguyên).
--   Chỉ ghi trực tuyến (không vào bộ máy cl_* / bản sao offline).
-- Áp bằng `digitext-api migrate` hoặc tay theo thứ tự.
BEGIN;

CREATE TABLE IF NOT EXISTS co_quan_luu_tru (
    project_id          uuid PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    ma_co_quan          text,
    ten                 text,
    ten_cap_tren        text,
    dia_chi             text,
    dien_thoai          text,
    email               text,
    loai_hinh           text NOT NULL DEFAULT 'luu_tru_co_quan' CHECK (loai_hinh IN ('luu_tru_co_quan', 'luu_tru_lich_su')),
    nguoi_ky            text,
    chuc_vu_nguoi_ky    text,
    han_nop_ngay        integer NOT NULL DEFAULT 60 CHECK (han_nop_ngay BETWEEN 1 AND 3650),
    nhac_truoc_ngay     integer[] NOT NULL DEFAULT '{15,7,1}',
    quyen_su_dung_ngay  integer NOT NULL DEFAULT 360 CHECK (quyen_su_dung_ngay BETWEEN 0 AND 3650),
    ghi_chu             text,
    updated_at          timestamptz NOT NULL DEFAULT now(),
    updated_by          uuid REFERENCES users(id) ON DELETE SET NULL
);

DROP TRIGGER IF EXISTS tg_co_quan_luu_tru_changed ON co_quan_luu_tru;
CREATE TRIGGER tg_co_quan_luu_tru_changed AFTER INSERT OR UPDATE OR DELETE ON co_quan_luu_tru
    FOR EACH STATEMENT EXECUTE FUNCTION note_change('projects');

CREATE TABLE IF NOT EXISTS phong_lich_su (
    id                      uuid PRIMARY KEY,
    project_id              uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phong_id                uuid NOT NULL REFERENCES cl_phong(id) ON DELETE CASCADE,
    phien_ban               integer NOT NULL,
    lich_su_don_vi          text,
    lich_su_phong           text,
    thoi_gian_tai_lieu      text,
    tong_so_tai_lieu        text,
    phuong_an_phan_loai     text,
    ngon_ngu                text,
    trang_thai_phong        text NOT NULL DEFAULT 'mo' CHECK (trang_thai_phong IN ('mo', 'dong')),
    nguon                   text,
    ghi_chu                 text,
    created_by              uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at              timestamptz NOT NULL DEFAULT now(),
    UNIQUE (phong_id, phien_ban)
);
CREATE INDEX IF NOT EXISTS ix_phong_lich_su_phong ON phong_lich_su (phong_id, phien_ban DESC);

DROP TRIGGER IF EXISTS tg_phong_lich_su_changed ON phong_lich_su;
CREATE TRIGGER tg_phong_lich_su_changed AFTER INSERT OR UPDATE OR DELETE ON phong_lich_su
    FOR EACH STATEMENT EXECUTE FUNCTION note_change('chinhLy');

COMMIT;
