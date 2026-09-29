-- 30/9/2026 — CHỈNH LÝ tài liệu giấy (đưa nghiệp vụ Digitext.2025 vào DigiText, mô hình lai):
-- danh mục (khối phông, phông, phòng ban, mảng CV, THBQ, ngôn ngữ, loại VB), hồ sơ tạm + văn bản +
-- lịch sử sửa từng trường, hệ thống hoá (hộp / bó / tập), kho / giá, mượn trả. Máy chủ là nguồn sự
-- thật; desktop giữ bản sao SQLite, làm offline rồi đồng bộ: mọi bảng cl_* có `ver` (chuỗi toàn cục
-- cl_ver_seq, gán bằng trigger), `deleted_at` (tombstone) và id uuid do client sinh.

CREATE SEQUENCE IF NOT EXISTS cl_ver_seq;

CREATE OR REPLACE FUNCTION cl_touch() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    NEW.ver := nextval('cl_ver_seq');
    NEW.updated_at := now();
    RETURN NEW;
END $$;

-- Khoá tư vấn theo dự án: ghi cl_* lấy khoá độc quyền, pull lấy khoá chia sẻ → không có "lỗ" ver
-- (giao dịch cấp ver=100 commit sau khi client đã kéo tới ver=101).
CREATE OR REPLACE FUNCTION cl_lock_key(p uuid) RETURNS bigint LANGUAGE sql IMMUTABLE AS $$
    SELECT hashtextextended('cl:' || COALESCE(p::text, 'global'), 0) $$;

-- Ngày dạng chữ đã chuẩn hoá bởi Rust: dd/MM/yyyy | MM/yyyy | yyyy. Không đọc được → NULL.
CREATE OR REPLACE FUNCTION cl_year_of(t text) RETURNS int LANGUAGE sql IMMUTABLE AS $$
    SELECT CASE WHEN t ~ '^\d{4}$' THEN t::int
                WHEN t ~ '^\d{1,2}/\d{4}$' THEN split_part(t, '/', 2)::int
                WHEN t ~ '^\d{1,2}/\d{1,2}/\d{4}$' THEN split_part(t, '/', 3)::int END $$;
CREATE OR REPLACE FUNCTION cl_date_of(t text) RETURNS date LANGUAGE sql IMMUTABLE AS $$
    SELECT CASE WHEN t ~ '^\d{4}$' THEN make_date(t::int, 1, 1)
                WHEN t ~ '^\d{1,2}/\d{4}$' THEN make_date(split_part(t, '/', 2)::int, split_part(t, '/', 1)::int, 1)
                WHEN t ~ '^\d{1,2}/\d{1,2}/\d{4}$'
                    THEN make_date(split_part(t, '/', 3)::int, split_part(t, '/', 2)::int, split_part(t, '/', 1)::int) END $$;
CREATE OR REPLACE FUNCTION cl_int_of(t text) RETURNS int LANGUAGE sql IMMUTABLE AS $$
    SELECT CASE WHEN btrim(t) ~ '^\d{1,9}$' THEN btrim(t)::int END $$;

-- Trạng thái THBQ tính ĐỘNG (bản cũ lưu cứng nên không bao giờ cập nhật).
-- trang_thai: 1 vĩnh viễn · 2 có thời hạn (gia_tri năm) · 3 đánh giá lại · 4 theo tuổi thọ công trình.
CREATE OR REPLACE FUNCTION cl_thbq_status(trang_thai smallint, gia_tri int, nam_kt int, nam_now int)
RETURNS text LANGUAGE sql IMMUTABLE AS $$
    SELECT CASE WHEN trang_thai IS NULL THEN NULL
                WHEN trang_thai = 1 THEN 'VV'
                WHEN trang_thai = 2 AND nam_kt IS NOT NULL AND (nam_now - nam_kt + 1) > COALESCE(gia_tri, 0) THEN 'HET'
                ELSE 'GL' END $$;

-- ============================================================ danh mục

CREATE TABLE IF NOT EXISTS cl_khoi_phong (
    id          uuid PRIMARY KEY,
    project_id  uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    ma          text NOT NULL,
    ten         text NOT NULL,
    so_tt       integer,
    ver         bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at  timestamptz NOT NULL DEFAULT now(),
    created_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    updated_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at  timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_khoi_phong_ma ON cl_khoi_phong (project_id, lower(ma)) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_khoi_phong_ver ON cl_khoi_phong (project_id, ver);

-- Phông (= DuAn của bản cũ). ma_du_an dùng để tạo mã hồ sơ.
CREATE TABLE IF NOT EXISTS cl_phong (
    id          uuid PRIMARY KEY,
    project_id  uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    khoi_id     uuid REFERENCES cl_khoi_phong(id) ON DELETE SET NULL,
    ma_du_an    text NOT NULL,
    ten         text NOT NULL,
    phong_so    text,
    muc_luc_so  text,
    logo_key    text,
    ghi_chu     text,
    so_tt       integer,
    ver         bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at  timestamptz NOT NULL DEFAULT now(),
    created_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    updated_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at  timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_phong_ma ON cl_phong (project_id, lower(ma_du_an)) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_phong_ver ON cl_phong (project_id, ver);

CREATE TABLE IF NOT EXISTS cl_phong_ban (
    id          uuid PRIMARY KEY,
    project_id  uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phong_id    uuid NOT NULL REFERENCES cl_phong(id) ON DELETE CASCADE,
    ma_don_vi   text,
    ten         text NOT NULL,
    nhiem_ky    text,
    so_tt       integer,
    ver         bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at  timestamptz NOT NULL DEFAULT now(),
    created_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    updated_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at  timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_phong_ban_ma ON cl_phong_ban (phong_id, lower(ma_don_vi))
    WHERE deleted_at IS NULL AND ma_don_vi IS NOT NULL AND ma_don_vi <> '';
CREATE INDEX IF NOT EXISTS ix_cl_phong_ban_phong ON cl_phong_ban (phong_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_phong_ban_ver ON cl_phong_ban (project_id, ver);

CREATE TABLE IF NOT EXISTS cl_mang (
    id            uuid PRIMARY KEY,
    project_id    uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phong_ban_id  uuid NOT NULL REFERENCES cl_phong_ban(id) ON DELETE CASCADE,
    ma_mang       text NOT NULL DEFAULT '00',
    ten           text NOT NULL,
    so_tt         integer,
    ver           bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at    timestamptz NOT NULL DEFAULT now(),
    created_by    uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at    timestamptz NOT NULL DEFAULT now(),
    updated_by    uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at    timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_mang_ma ON cl_mang (phong_ban_id, lower(ma_mang)) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_mang_ver ON cl_mang (project_id, ver);

-- Ba bảng mã: dòng seed có project_id NULL (dùng chung), dòng người dùng thêm có project_id.
CREATE TABLE IF NOT EXISTS cl_thbq (
    id          uuid PRIMARY KEY,
    project_id  uuid REFERENCES projects(id) ON DELETE CASCADE,
    ma_tt05     text NOT NULL DEFAULT '07',
    ten         text NOT NULL,
    trang_thai  smallint NOT NULL CHECK (trang_thai BETWEEN 1 AND 4),
    gia_tri     integer NOT NULL DEFAULT 0,
    so_tt       integer,
    ver         bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at  timestamptz NOT NULL DEFAULT now(),
    created_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    updated_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at  timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_thbq_ten ON cl_thbq (COALESCE(project_id, '00000000-0000-0000-0000-000000000000'::uuid), lower(ten))
    WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_thbq_ver ON cl_thbq (project_id, ver);

CREATE TABLE IF NOT EXISTS cl_ngon_ngu (
    id          uuid PRIMARY KEY,
    project_id  uuid REFERENCES projects(id) ON DELETE CASCADE,
    ma          text NOT NULL,
    ten         text NOT NULL,
    so_tt       integer,
    ver         bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at  timestamptz NOT NULL DEFAULT now(),
    created_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    updated_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at  timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_ngon_ngu_ma ON cl_ngon_ngu (COALESCE(project_id, '00000000-0000-0000-0000-000000000000'::uuid), lower(ma))
    WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_ngon_ngu_ver ON cl_ngon_ngu (project_id, ver);

CREATE TABLE IF NOT EXISTS cl_loai_van_ban (
    id          uuid PRIMARY KEY,
    project_id  uuid REFERENCES projects(id) ON DELETE CASCADE,
    ma          text NOT NULL,
    ten         text NOT NULL,
    so_tt       integer,
    ver         bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at  timestamptz NOT NULL DEFAULT now(),
    created_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    updated_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at  timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_loai_van_ban_ma ON cl_loai_van_ban (COALESCE(project_id, '00000000-0000-0000-0000-000000000000'::uuid), lower(ma))
    WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_loai_van_ban_ver ON cl_loai_van_ban (project_id, ver);

-- ============================================================ hộp / bó / tập (hệ thống hoá, chỉ online)

CREATE TABLE IF NOT EXISTS cl_hop (
    id            uuid PRIMARY KEY,
    project_id    uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phong_ban_id  uuid NOT NULL REFERENCES cl_phong_ban(id) ON DELETE CASCADE,
    hop_so        text NOT NULL,
    hop_so_int    integer GENERATED ALWAYS AS (cl_int_of(hop_so)) STORED,
    noi_bao_quan  text,
    vi_tri_gia    text,
    vi_tri_ke     text,
    phong_so      text,
    nam           text,
    tu_ho_so      text,
    den_ho_so     text,
    vinh_vien     boolean NOT NULL DEFAULT false,
    ghi_chu       text,
    ver           bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at    timestamptz NOT NULL DEFAULT now(),
    created_by    uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at    timestamptz NOT NULL DEFAULT now(),
    updated_by    uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at    timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_hop_so ON cl_hop (phong_ban_id, lower(btrim(hop_so))) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_hop_int ON cl_hop (phong_ban_id, hop_so_int) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_hop_ver ON cl_hop (project_id, ver);

CREATE TABLE IF NOT EXISTS cl_bo_tap (
    id            uuid PRIMARY KEY,
    project_id    uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phong_ban_id  uuid NOT NULL REFERENCES cl_phong_ban(id) ON DELETE CASCADE,
    loai          text NOT NULL CHECK (loai IN ('bo', 'tap')),
    so            integer NOT NULL CHECK (so > 0),
    ghi_chu       text,
    ver           bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at    timestamptz NOT NULL DEFAULT now(),
    created_by    uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at    timestamptz NOT NULL DEFAULT now(),
    updated_by    uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at    timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_bo_tap_so ON cl_bo_tap (phong_ban_id, loai, so) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_bo_tap_ver ON cl_bo_tap (project_id, ver);

-- ============================================================ kho / giá (chỉ online)

CREATE TABLE IF NOT EXISTS cl_kho (
    id          uuid PRIMARY KEY,
    project_id  uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    ten         text NOT NULL,
    so_do_key   text,
    ghi_chu     text,
    so_tt       integer,
    ver         bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at  timestamptz NOT NULL DEFAULT now(),
    created_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    updated_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at  timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_kho_ten ON cl_kho (project_id, lower(ten)) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_kho_ver ON cl_kho (project_id, ver);

-- Giá = dải hộp [tu_hop..den_hop] của MỘT phòng ban trong một kho. Giá của hồ sơ suy ra từ số hộp.
CREATE TABLE IF NOT EXISTS cl_gia (
    id            uuid PRIMARY KEY,
    project_id    uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    kho_id        uuid NOT NULL REFERENCES cl_kho(id) ON DELETE CASCADE,
    phong_ban_id  uuid REFERENCES cl_phong_ban(id) ON DELETE SET NULL,
    ten           text NOT NULL,
    tu_hop        integer,
    den_hop       integer,
    ghi_chu       text,
    so_tt         integer,
    ver           bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at    timestamptz NOT NULL DEFAULT now(),
    created_by    uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at    timestamptz NOT NULL DEFAULT now(),
    updated_by    uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at    timestamptz,
    CHECK (tu_hop IS NULL OR den_hop IS NULL OR tu_hop <= den_hop)
);
CREATE INDEX IF NOT EXISTS ix_cl_gia_kho ON cl_gia (kho_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_gia_dai ON cl_gia (phong_ban_id, tu_hop, den_hop) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_gia_ver ON cl_gia (project_id, ver);

-- ============================================================ hồ sơ

CREATE TABLE IF NOT EXISTS cl_ho_so (
    id              uuid PRIMARY KEY,
    project_id      uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phong_id        uuid NOT NULL REFERENCES cl_phong(id) ON DELETE CASCADE,
    phong_ban_id    uuid REFERENCES cl_phong_ban(id) ON DELETE SET NULL,
    mang_id         uuid REFERENCES cl_mang(id) ON DELETE SET NULL,
    thbq_id         uuid REFERENCES cl_thbq(id) ON DELETE SET NULL,
    -- hệ thống hoá — CHỈ ghi online (push đổi giá trị bị từ chối):
    ho_so_so        text,                 -- số hồ sơ chính thức, có thể "572A"
    thu_tu_sap_xep  integer,
    hop_id          uuid REFERENCES cl_hop(id) ON DELETE SET NULL,
    bo_id           uuid REFERENCES cl_bo_tap(id) ON DELETE SET NULL,
    tap_id          uuid REFERENCES cl_bo_tap(id) ON DELETE SET NULL,
    gia_id          uuid REFERENCES cl_gia(id) ON DELETE SET NULL,   -- gán giá thủ công; NULL = suy ra từ hộp
    -- mô tả (sửa offline được):
    so_tam          text NOT NULL,
    so_ky_hieu_hs   text,
    tieu_de         text NOT NULL DEFAULT '',
    ngay_bat_dau    text,                 -- dd/MM/yyyy | MM/yyyy | yyyy (Rust chuẩn hoá)
    ngay_ket_thuc   text,
    nam_ket_thuc    integer GENERATED ALWAYS AS (cl_year_of(ngay_ket_thuc)) STORED,
    ngay_kt_sort    date    GENERATED ALWAYS AS (cl_date_of(ngay_ket_thuc)) STORED,
    so_to           text,
    tong_so_vb      integer,
    so_luong_trang  integer,
    che_do_su_dung  text,
    ngon_ngu        text,
    tinh_trang_vat_ly text,
    trung_thua      boolean NOT NULL DEFAULT false,
    ghi_chu         text,
    -- TT 05/2025:
    tu_khoa         text,
    ky_hieu_thong_tin text,
    muc_do_tin_cay  text,
    ma_ho_so_giay   text,
    -- chứng từ kết thúc (cột riêng để merge / lịch sử / nhập Excel theo từng trường):
    ct_so_to_bang_chu       text,
    ct_danh_so_tu           integer,
    ct_den_so               integer,
    ct_so_trung             text,
    ct_so_khuyet            text,
    ct_so_muc_luc_vb        integer,
    ct_so_muc_luc_vb_bang_chu text,
    ct_dac_diem_tai_lieu    text,
    ct_dia_danh             text,
    ct_ngay                 integer,
    ct_thang                integer,
    ct_nam                  integer,
    ct_nguoi_lap_hs         text,
    ct_phan_bo_sung         text,
    ver             bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at      timestamptz NOT NULL DEFAULT now(),
    created_by      uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at      timestamptz NOT NULL DEFAULT now(),
    updated_by      uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at      timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_ho_so_so_tam ON cl_ho_so (phong_id, lower(btrim(so_tam))) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_ho_so_ver      ON cl_ho_so (project_id, ver);
CREATE INDEX IF NOT EXISTS ix_cl_ho_so_phong    ON cl_ho_so (phong_id, phong_ban_id, mang_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_ho_so_hop      ON cl_ho_so (hop_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_ho_so_thbq     ON cl_ho_so (thbq_id, nam_ket_thuc) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_ho_so_ho_so_so ON cl_ho_so (phong_id, ho_so_so) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_ho_so_tieu_de  ON cl_ho_so USING gin (tieu_de gin_trgm_ops);

CREATE TABLE IF NOT EXISTS cl_van_ban (
    id                uuid PRIMARY KEY,
    project_id        uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    ho_so_id          uuid NOT NULL REFERENCES cl_ho_so(id) ON DELETE CASCADE,
    so_thu_tu         integer NOT NULL,
    loai_vb           text,
    so_van_ban        text,
    so_ky_hieu        text,
    ngay_thang        text,
    ngay_thang_sort   date GENERATED ALWAYS AS (cl_date_of(ngay_thang)) STORED,
    co_quan_ban_hanh  text,
    co_quan_chu_quan  text,
    nguoi_ky          text,
    trich_yeu         text,
    to_so             integer,
    so_trang          integer,
    trang_so          integer,
    ngon_ngu          text,
    che_do_su_dung    text,
    do_tin_cay        text,
    but_tich          text,
    tinh_trang_vat_ly text,
    tai_lieu_so       text,
    tu_khoa           text,
    ky_hieu_thong_tin text,
    ghi_chu           text,
    ver               bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at        timestamptz NOT NULL DEFAULT now(),
    created_by        uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at        timestamptz NOT NULL DEFAULT now(),
    updated_by        uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at        timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_van_ban_stt ON cl_van_ban (ho_so_id, so_thu_tu) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_van_ban_ho_so ON cl_van_ban (ho_so_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_van_ban_ver   ON cl_van_ban (project_id, ver);
CREATE INDEX IF NOT EXISTS ix_cl_van_ban_trich_yeu ON cl_van_ban USING gin (trich_yeu gin_trgm_ops);

-- Tệp của văn bản: nối tài liệu đã nộp lên DigiText (document_id) hoặc chỉ mô tả (path/name).
CREATE TABLE IF NOT EXISTS cl_van_ban_file (
    id           uuid PRIMARY KEY,
    project_id   uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    van_ban_id   uuid NOT NULL REFERENCES cl_van_ban(id) ON DELETE CASCADE,
    document_id  uuid REFERENCES documents(id) ON DELETE SET NULL,
    path         text,
    name         text NOT NULL,
    size         bigint,
    format       text,
    so_trang     integer,
    kho_giay     text CHECK (kho_giay IS NULL OR kho_giay IN ('A0', 'A1', 'A2', 'A3', 'A4', 'A5')),
    so_tt        integer,
    ver          bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at   timestamptz NOT NULL DEFAULT now(),
    created_by   uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at   timestamptz NOT NULL DEFAULT now(),
    updated_by   uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at   timestamptz
);
CREATE INDEX IF NOT EXISTS ix_cl_van_ban_file_vb  ON cl_van_ban_file (van_ban_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_van_ban_file_ver ON cl_van_ban_file (project_id, ver);

-- ============================================================ mượn / trả (chỉ online)

CREATE TABLE IF NOT EXISTS cl_muon_tra (
    id                 uuid PRIMARY KEY,
    project_id         uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    ho_so_id           uuid NOT NULL REFERENCES cl_ho_so(id) ON DELETE CASCADE,
    nguoi_muon         text NOT NULL,
    don_vi             text,
    dien_thoai         text,
    email              text,
    noi_dung           text,
    ngay_muon          date NOT NULL DEFAULT CURRENT_DATE,
    ngay_du_kien_tra   date,
    ngay_thuc_te_tra   date,
    trang_thai         text NOT NULL DEFAULT 'dang_muon' CHECK (trang_thai IN ('dang_muon', 'da_tra')),
    nguoi_lap_phieu    uuid REFERENCES users(id) ON DELETE SET NULL,
    ghi_chu            text,
    ver                bigint NOT NULL DEFAULT nextval('cl_ver_seq'),
    created_at         timestamptz NOT NULL DEFAULT now(),
    created_by         uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at         timestamptz NOT NULL DEFAULT now(),
    updated_by         uuid REFERENCES users(id) ON DELETE SET NULL,
    deleted_at         timestamptz
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_cl_muon_tra_mo ON cl_muon_tra (ho_so_id) WHERE trang_thai = 'dang_muon' AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_muon_tra_tt  ON cl_muon_tra (project_id, trang_thai) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_cl_muon_tra_ver ON cl_muon_tra (project_id, ver);

-- ============================================================ lịch sử (append-only, máy chủ sinh)

CREATE TABLE IF NOT EXISTS cl_lich_su (
    id          uuid PRIMARY KEY,
    project_id  uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    entity      text NOT NULL CHECK (entity IN ('ho_so', 'van_ban')),
    entity_id   uuid NOT NULL,
    ho_so_id    uuid NOT NULL,          -- không FK: giữ lịch sử sau khi dọn tombstone
    phong_id    uuid,
    action      text NOT NULL CHECK (action IN ('add', 'edit', 'delete', 'restore', 'import', 'systematize', 'boxing', 'assign_gia')),
    actor_id    uuid REFERENCES users(id) ON DELETE SET NULL,
    actor_name  text,
    at          timestamptz NOT NULL DEFAULT now(),
    at_client   timestamptz,            -- giờ máy trạm lúc sửa offline
    source      text NOT NULL DEFAULT 'online' CHECK (source IN ('online', 'sync', 'import')),
    title       text,
    diffs       jsonb NOT NULL DEFAULT '[]'::jsonb,   -- [{"f": "tieu_de", "o": "cũ", "n": "mới"}, ...]
    n_new       integer NOT NULL DEFAULT 0,           -- số trường rỗng → có
    n_edited    integer NOT NULL DEFAULT 0,           -- số trường có → khác
    ver         bigint NOT NULL DEFAULT nextval('cl_ver_seq')
);
CREATE INDEX IF NOT EXISTS ix_cl_lich_su_ver   ON cl_lich_su (project_id, ver);
CREATE INDEX IF NOT EXISTS ix_cl_lich_su_hoso  ON cl_lich_su (ho_so_id, at);
CREATE INDEX IF NOT EXISTS ix_cl_lich_su_actor ON cl_lich_su (actor_id, at);

-- ============================================================ đồng bộ

-- Idempotency của push: mỗi op_id xử lý đúng một lần.
CREATE TABLE IF NOT EXISTS cl_sync_ops (
    op_id       uuid PRIMARY KEY,
    project_id  uuid REFERENCES projects(id) ON DELETE CASCADE,
    user_id     uuid,
    device_id   uuid,
    tbl         text NOT NULL,
    row_id      uuid NOT NULL,
    outcome     text NOT NULL,
    result      jsonb,
    at          timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS ix_cl_sync_ops_at ON cl_sync_ops (at);

-- Mốc dọn tombstone: client có since < pruned_ver phải kéo lại toàn bộ.
CREATE TABLE IF NOT EXISTS cl_sync_meta (
    project_key  uuid PRIMARY KEY,
    pruned_ver   bigint NOT NULL DEFAULT 0,
    pruned_at    timestamptz
);

-- ============================================================ trigger ver / live

DO $$
DECLARE t text;
BEGIN
    FOREACH t IN ARRAY ARRAY['cl_khoi_phong', 'cl_phong', 'cl_phong_ban', 'cl_mang', 'cl_thbq', 'cl_ngon_ngu',
                             'cl_loai_van_ban', 'cl_hop', 'cl_bo_tap', 'cl_kho', 'cl_gia', 'cl_ho_so', 'cl_van_ban',
                             'cl_van_ban_file', 'cl_muon_tra']
    LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS tg_%s_touch ON %s', t, t);
        EXECUTE format('CREATE TRIGGER tg_%s_touch BEFORE INSERT OR UPDATE ON %s FOR EACH ROW EXECUTE FUNCTION cl_touch()', t, t);
        EXECUTE format('DROP TRIGGER IF EXISTS tg_%s_changed ON %s', t, t);
        EXECUTE format('CREATE TRIGGER tg_%s_changed AFTER INSERT OR UPDATE OR DELETE ON %s FOR EACH STATEMENT EXECUTE FUNCTION note_change(''chinhLy'')', t, t);
    END LOOP;
    EXECUTE 'DROP TRIGGER IF EXISTS tg_cl_lich_su_changed ON cl_lich_su';
    EXECUTE 'CREATE TRIGGER tg_cl_lich_su_changed AFTER INSERT ON cl_lich_su FOR EACH STATEMENT EXECUTE FUNCTION note_change(''chinhLy'')';
END $$;

-- ============================================================ view hồ sơ đầy đủ

CREATE OR REPLACE VIEW cl_ho_so_v AS
SELECT h.*,
       p.ma_du_an, p.phong_so, p.muc_luc_so, p.ten AS ten_phong,
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
                           h.nam_ket_thuc::text, btrim(h.ho_so_so), NULLIF(btrim(p.muc_luc_so), '')) END AS ma_ho_so,
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
      AND hp.hop_so_int IS NOT NULL AND hp.hop_so_int BETWEEN g.tu_hop AND g.den_hop
    ORDER BY g.tu_hop LIMIT 1
) gd ON true
LEFT JOIN cl_kho kd ON kd.id = gd.kho_id
LEFT JOIN users cu ON cu.id = h.created_by
LEFT JOIN users uu ON uu.id = h.updated_by;

-- ============================================================ seed bảng mã (uuid cố định, dùng chung)

INSERT INTO cl_thbq (id, project_id, ma_tt05, ten, trang_thai, gia_tri, so_tt) VALUES
    ('00000000-0000-4000-8000-00000c010001', NULL, '01', 'Vĩnh viễn', 1, 0, 1),
    ('00000000-0000-4000-8000-00000c010002', NULL, '07', '5 năm', 2, 5, 2),
    ('00000000-0000-4000-8000-00000c010003', NULL, '06', '10 năm', 2, 10, 3),
    ('00000000-0000-4000-8000-00000c010004', NULL, '07', '15 năm', 2, 15, 4),
    ('00000000-0000-4000-8000-00000c010005', NULL, '05', '20 năm', 2, 20, 5),
    ('00000000-0000-4000-8000-00000c010006', NULL, '03', '50 năm', 2, 50, 6),
    ('00000000-0000-4000-8000-00000c010007', NULL, '02', '70 năm', 2, 70, 7),
    ('00000000-0000-4000-8000-00000c010008', NULL, '07', '05 năm đánh giá', 3, 0, 8),
    ('00000000-0000-4000-8000-00000c010009', NULL, '07', '10 năm đánh giá', 3, 0, 9),
    ('00000000-0000-4000-8000-00000c010010', NULL, '07', '15 năm đánh giá', 3, 0, 10),
    ('00000000-0000-4000-8000-00000c010011', NULL, '07', '20 năm đánh giá', 3, 0, 11),
    ('00000000-0000-4000-8000-00000c010012', NULL, '07', '50 năm đánh giá', 3, 0, 12),
    ('00000000-0000-4000-8000-00000c010013', NULL, '07', '70 năm đánh giá', 3, 0, 13),
    ('00000000-0000-4000-8000-00000c010014', NULL, '07', 'Theo tuổi thọ công trình', 4, 0, 14)
ON CONFLICT (id) DO NOTHING;

INSERT INTO cl_ngon_ngu (id, project_id, ma, ten, so_tt) VALUES
    ('00000000-0000-4000-8000-00000c020001', NULL, '01', 'Tiếng Việt', 1),
    ('00000000-0000-4000-8000-00000c020002', NULL, '02', 'Tiếng Anh', 2),
    ('00000000-0000-4000-8000-00000c020003', NULL, '03', 'Tiếng Pháp', 3),
    ('00000000-0000-4000-8000-00000c020004', NULL, '04', 'Tiếng Nga', 4),
    ('00000000-0000-4000-8000-00000c020005', NULL, '05', 'Tiếng Trung', 5),
    ('00000000-0000-4000-8000-00000c020006', NULL, '06', 'Việt - Anh', 6),
    ('00000000-0000-4000-8000-00000c020007', NULL, '07', 'Việt - Nga', 7),
    ('00000000-0000-4000-8000-00000c020008', NULL, '08', 'Việt - Pháp', 8),
    ('00000000-0000-4000-8000-00000c020009', NULL, '09', 'Hán Nôm', 9),
    ('00000000-0000-4000-8000-00000c020010', NULL, '10', 'Việt - Trung', 10),
    ('00000000-0000-4000-8000-00000c020011', NULL, '11', 'Khác', 11)
ON CONFLICT (id) DO NOTHING;

INSERT INTO cl_loai_van_ban (id, project_id, ma, ten, so_tt) VALUES
    ('00000000-0000-4000-8000-00000c030001', NULL, '01', 'Nghị quyết', 1),
    ('00000000-0000-4000-8000-00000c030002', NULL, '02', 'Quyết định', 2),
    ('00000000-0000-4000-8000-00000c030003', NULL, '03', 'Chỉ thị', 3),
    ('00000000-0000-4000-8000-00000c030004', NULL, '04', 'Quy chế', 4),
    ('00000000-0000-4000-8000-00000c030005', NULL, '05', 'Quy định', 5),
    ('00000000-0000-4000-8000-00000c030006', NULL, '06', 'Thông cáo', 6),
    ('00000000-0000-4000-8000-00000c030007', NULL, '07', 'Thông báo', 7),
    ('00000000-0000-4000-8000-00000c030008', NULL, '08', 'Hướng dẫn', 8),
    ('00000000-0000-4000-8000-00000c030009', NULL, '09', 'Chương trình', 9),
    ('00000000-0000-4000-8000-00000c030010', NULL, '10', 'Kế hoạch', 10),
    ('00000000-0000-4000-8000-00000c030011', NULL, '11', 'Phương án', 11),
    ('00000000-0000-4000-8000-00000c030012', NULL, '12', 'Đề án', 12),
    ('00000000-0000-4000-8000-00000c030013', NULL, '13', 'Dự án', 13),
    ('00000000-0000-4000-8000-00000c030014', NULL, '14', 'Báo cáo', 14),
    ('00000000-0000-4000-8000-00000c030015', NULL, '15', 'Tờ trình', 15),
    ('00000000-0000-4000-8000-00000c030016', NULL, '16', 'Giấy ủy quyền', 16),
    ('00000000-0000-4000-8000-00000c030017', NULL, '17', 'Phiếu gửi', 17),
    ('00000000-0000-4000-8000-00000c030018', NULL, '18', 'Phiếu chuyển', 18),
    ('00000000-0000-4000-8000-00000c030019', NULL, '19', 'Phiếu báo', 19),
    ('00000000-0000-4000-8000-00000c030020', NULL, '20', 'Biên bản', 20),
    ('00000000-0000-4000-8000-00000c030021', NULL, '21', 'Hợp đồng', 21),
    ('00000000-0000-4000-8000-00000c030022', NULL, '22', 'Công văn', 22),
    ('00000000-0000-4000-8000-00000c030023', NULL, '23', 'Công điện', 23),
    ('00000000-0000-4000-8000-00000c030024', NULL, '24', 'Bản ghi nhớ', 24),
    ('00000000-0000-4000-8000-00000c030025', NULL, '25', 'Bản thỏa thuận', 25),
    ('00000000-0000-4000-8000-00000c030026', NULL, '26', 'Giấy mời', 26),
    ('00000000-0000-4000-8000-00000c030027', NULL, '27', 'Giấy giới thiệu', 27),
    ('00000000-0000-4000-8000-00000c030028', NULL, '28', 'Giấy nghỉ phép', 28),
    ('00000000-0000-4000-8000-00000c030029', NULL, '29', 'Thư công', 29),
    ('00000000-0000-4000-8000-00000c030030', NULL, '30', 'Bản đồ', 30),
    ('00000000-0000-4000-8000-00000c030031', NULL, '31', 'Bản vẽ kỹ thuật', 31),
    ('00000000-0000-4000-8000-00000c030032', NULL, '32', 'Khác', 32)
ON CONFLICT (id) DO NOTHING;
