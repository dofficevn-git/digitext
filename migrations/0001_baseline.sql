-- DigiText — baseline schema
--
-- File nay duoc sinh tu `pg_dump --schema-only` chay tren server production
-- (Postgres 16.14) ngay 2026-08-27, roi loc bo lenh meta cua psql va schema
-- `_sqlx_test`. Xem tools/deobfuscate/make_migration.py.
--
-- DAY LA BAN GOP (squashed) cua 35 migration goc — cac file .sql goc da
-- bi mat cung voi repo nguon. Danh sach migration goc (tu bang _sqlx_migrations):
--
--     1  identity
--     2  admin multi project
--     3  project tree
--     4  user stats
--     5  documents
--     6  refresh token device index
--     7  user permissions
--     8  ocr plane
--     9  ocr fields
--    10  make pdf2
--    11  document lifecycle
--    12  sign sessions
--    13  sign session attrs
--    14  review reason vlm
--    15  review reason empty page
--    16  packages
--    17  submissions
--    18  virus scan
--    19  notifications
--    20  records
--    21  dot5
--    22  khaithac
--    23  project admins
--    24  upload batches
--    25  disposal
--    26  dot8
--    27  sign intermediates
--    28  notifications info overdue
--    29  upload batch abandoned
--    30  document rel path
--    31  pk5 stt range
--    32  extract fields
--    33  document review
--    34  ocr jobs seq
--    35  change log
--
-- CANH BAO: KHONG chay file nay len database production. O do bang
-- _sqlx_migrations da ghi nhan 35 migration; sqlx se bao loi lech
-- checksum ngay o version 1. File nay danh cho database dung moi (dev/test).
--


-- FUNCTION: note_change()
CREATE FUNCTION public.note_change() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO change_log (topic) VALUES (TG_ARGV[0]);
    RETURN NULL; -- AFTER STATEMENT: giá trị trả về bị bỏ qua.
END;
$$;

-- TABLE: access_fee_rates
CREATE TABLE public.access_fee_rates (
    project_id uuid NOT NULL,
    phi_doc_tai_lieu bigint DEFAULT 0 NOT NULL,
    phi_sao_trang bigint DEFAULT 0 NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    phi_thong_tin bigint DEFAULT 0 NOT NULL,
    gioi_han_ket_qua integer DEFAULT 50 NOT NULL
);

-- TABLE: access_request_items
CREATE TABLE public.access_request_items (
    id uuid NOT NULL,
    request_id uuid NOT NULL,
    document_id uuid NOT NULL,
    stt integer NOT NULL,
    verdict text DEFAULT 'pending'::text NOT NULL,
    ly_do text,
    dieu_kien boolean DEFAULT false NOT NULL,
    user_copy_key text,
    deleted_at timestamp with time zone,
    CONSTRAINT access_request_items_verdict_check CHECK ((verdict = ANY (ARRAY['pending'::text, 'duyet'::text, 'tu_choi'::text])))
);

-- TABLE: access_requests
CREATE TABLE public.access_requests (
    id uuid NOT NULL,
    project_id uuid NOT NULL,
    requester uuid,
    kind text NOT NULL,
    muc_dich text NOT NULL,
    phuong_thuc text,
    uu_tien text,
    nguoi_nhan text,
    so_dinh_danh text,
    status text DEFAULT 'submitted'::text NOT NULL,
    review_note text,
    phi bigint,
    paid_at timestamp with time zone,
    han_su_dung timestamp with time zone,
    approved_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT access_requests_kind_check CHECK ((kind = ANY (ARRAY['doc'::text, 'ban_sao'::text, 'ban_sao_ky'::text, 'dip'::text]))),
    CONSTRAINT access_requests_status_check CHECK ((status = ANY (ARRAY['submitted'::text, 'approved'::text, 'refused'::text, 'expired'::text])))
);

-- TABLE: audit_log
CREATE TABLE public.audit_log (
    id bigint NOT NULL,
    actor_id uuid,
    device_id uuid,
    action text NOT NULL,
    target text,
    detail jsonb,
    ip text,
    at timestamp with time zone DEFAULT now() NOT NULL
);

-- SEQUENCE: audit_log_id_seq
CREATE SEQUENCE public.audit_log_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- SEQUENCE OWNED BY: audit_log_id_seq
ALTER SEQUENCE public.audit_log_id_seq OWNED BY public.audit_log.id;

-- TABLE: backup_runs
CREATE TABLE public.backup_runs (
    id uuid NOT NULL,
    loai text NOT NULL,
    nguon text NOT NULL,
    phuong_tien text,
    ket_qua text NOT NULL,
    chi_tiet text,
    reported_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT backup_runs_ket_qua_check CHECK ((ket_qua = ANY (ARRAY['dat'::text, 'khong_dat'::text]))),
    CONSTRAINT backup_runs_loai_check CHECK ((loai = ANY (ARRAY['gia_tang'::text, 'day_du'::text, 'day_du_3_nam'::text])))
);

-- TABLE: change_log
CREATE TABLE public.change_log (
    seq bigint NOT NULL,
    topic text NOT NULL,
    at timestamp with time zone DEFAULT now() NOT NULL
);

-- SEQUENCE: change_log_seq_seq
CREATE SEQUENCE public.change_log_seq_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- SEQUENCE OWNED BY: change_log_seq_seq
ALTER SEQUENCE public.change_log_seq_seq OWNED BY public.change_log.seq;

-- TABLE: devices
CREATE TABLE public.devices (
    id uuid NOT NULL,
    name text NOT NULL,
    platform text NOT NULL,
    app_version text,
    project_id uuid,
    registered_by uuid,
    first_seen timestamp with time zone DEFAULT now() NOT NULL,
    last_seen timestamp with time zone,
    revoked_at timestamp with time zone
);

-- TABLE: disposal_batches
CREATE TABLE public.disposal_batches (
    id uuid NOT NULL,
    project_id uuid NOT NULL,
    nam integer NOT NULL,
    status text DEFAULT 'hoi_dong'::text NOT NULL,
    hoi_dong_note text,
    tham_dinh_note text,
    tu_quyet boolean DEFAULT false NOT NULL,
    quyet_dinh_so text,
    decided_by uuid,
    decided_at timestamp with time zone,
    executed_by uuid,
    executed_at timestamp with time zone,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT disposal_batches_status_check CHECK ((status = ANY (ARRAY['hoi_dong'::text, 'tham_dinh'::text, 'quyet_dinh'::text, 'da_huy'::text])))
);

-- TABLE: disposal_entries
CREATE TABLE public.disposal_entries (
    id uuid NOT NULL,
    project_id uuid NOT NULL,
    document_id uuid NOT NULL,
    batch_id uuid,
    ly_do text NOT NULL,
    dup_of uuid,
    ghi_chu text,
    status text DEFAULT 'xem_xet'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT disposal_entries_ly_do_check CHECK ((ly_do = ANY (ARRAY['trung_lap'::text, 'het_thoi_han'::text]))),
    CONSTRAINT disposal_entries_status_check CHECK ((status = ANY (ARRAY['xem_xet'::text, 'giu_lai'::text, 'da_huy'::text])))
);

-- TABLE: disposal_log
CREATE TABLE public.disposal_log (
    id uuid NOT NULL,
    project_id uuid NOT NULL,
    batch_id uuid,
    entry_id uuid,
    document_id uuid,
    action text NOT NULL,
    actor uuid,
    detail text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

-- TABLE: documents
CREATE TABLE public.documents (
    id uuid NOT NULL,
    project_id uuid NOT NULL,
    uploaded_by uuid,
    filename text NOT NULL,
    ho_so_ref text,
    page_count integer,
    size_bytes bigint NOT NULL,
    blake3 text NOT NULL,
    storage_path text NOT NULL,
    uploaded_at timestamp with time zone DEFAULT now() NOT NULL,
    status text DEFAULT 'ocr_pending'::text NOT NULL,
    batch_id uuid,
    het_han_luu_tru timestamp with time zone,
    rel_path text,
    review_status text DEFAULT 'pending'::text NOT NULL,
    reviewed_by uuid,
    reviewed_at timestamp with time zone,
    review_note text,
    CONSTRAINT ck_documents_review_trace CHECK ((((review_status = 'pending'::text) AND (reviewed_at IS NULL) AND (reviewed_by IS NULL)) OR ((review_status <> 'pending'::text) AND (reviewed_at IS NOT NULL)))),
    CONSTRAINT documents_review_status_check CHECK ((review_status = ANY (ARRAY['pending'::text, 'approved'::text, 'rejected'::text]))),
    CONSTRAINT documents_status_check CHECK ((status = ANY (ARRAY['uploaded'::text, 'ocr_pending'::text, 'ocr_done'::text, 'to_sign'::text, 'signing'::text, 'signed'::text, 'sign_rejected'::text, 'archived'::text, 'disposed'::text])))
);

-- TABLE: engine_versions
CREATE TABLE public.engine_versions (
    id uuid NOT NULL,
    name text NOT NULL,
    role text NOT NULL,
    precision_mode text DEFAULT 'fp16'::text NOT NULL,
    active boolean DEFAULT true NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT engine_versions_precision_mode_check CHECK ((precision_mode = ANY (ARRAY['fp16'::text, 'int8'::text, 'none'::text]))),
    CONSTRAINT engine_versions_role_check CHECK ((role = ANY (ARRAY['detection'::text, 'angle'::text, 'recognition'::text, 'fallback'::text, 'corrector'::text, 'audit'::text])))
);

-- TABLE: info_requests
CREATE TABLE public.info_requests (
    id uuid NOT NULL,
    project_id uuid NOT NULL,
    requester uuid,
    hinh_thuc text NOT NULL,
    chu_de text NOT NULL,
    muc_dich text NOT NULL,
    phuong_thuc text,
    status text DEFAULT 'submitted'::text NOT NULL,
    review_note text,
    phi bigint,
    paid_at timestamp with time zone,
    ket_qua_key text,
    bien_tap text,
    approved_at timestamp with time zone,
    fulfilled_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT info_requests_hinh_thuc_check CHECK ((hinh_thuc = ANY (ARRAY['danh_muc'::text, 'trich_xuat'::text, 'tong_hop'::text]))),
    CONSTRAINT info_requests_status_check CHECK ((status = ANY (ARRAY['submitted'::text, 'approved'::text, 'refused'::text, 'fulfilled'::text])))
);

-- TABLE: integrity_checks
CREATE TABLE public.integrity_checks (
    id uuid NOT NULL,
    project_id uuid NOT NULL,
    document_id uuid NOT NULL,
    ket_qua text NOT NULL,
    chi_tiet text,
    checked_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT integrity_checks_ket_qua_check CHECK ((ket_qua = ANY (ARRAY['dat'::text, 'loi'::text])))
);

-- TABLE: notifications
CREATE TABLE public.notifications (
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    submission_id uuid,
    kind text NOT NULL,
    message text NOT NULL,
    read_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    info_request_id uuid
);

-- TABLE: ocr_fields
CREATE TABLE public.ocr_fields (
    id uuid NOT NULL,
    document_id uuid NOT NULL,
    page_no integer NOT NULL,
    engine_version_id uuid NOT NULL,
    text_source text DEFAULT 'ocr_raw'::text NOT NULL,
    field_key text NOT NULL,
    value text NOT NULL,
    bbox jsonb,
    confidence real,
    verified boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT ocr_fields_confidence_check CHECK (((confidence >= (0)::double precision) AND (confidence <= (1)::double precision))),
    CONSTRAINT ocr_fields_field_key_check CHECK ((length(field_key) > 0)),
    CONSTRAINT ocr_fields_page_no_check CHECK ((page_no >= 1)),
    CONSTRAINT ocr_fields_text_source_check CHECK ((text_source = ANY (ARRAY['ocr_raw'::text, 'ocr_corrected'::text])))
);

-- TABLE: ocr_jobs
CREATE TABLE public.ocr_jobs (
    id uuid NOT NULL,
    document_id uuid NOT NULL,
    page_no integer NOT NULL,
    kind text DEFAULT 'ocr'::text NOT NULL,
    status text DEFAULT 'queued'::text NOT NULL,
    engine_version_id uuid,
    attempts integer DEFAULT 0 NOT NULL,
    last_error text,
    locked_by text,
    lease_until timestamp with time zone,
    priority integer DEFAULT 100 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    seq bigint NOT NULL,
    CONSTRAINT ocr_jobs_attempts_check CHECK ((attempts >= 0)),
    CONSTRAINT ocr_jobs_kind_check CHECK ((kind = ANY (ARRAY['ocr'::text, 're_ocr'::text, 'audit'::text, 'make_pdf2'::text, 'sign_wait'::text, 'extract_fields'::text]))),
    CONSTRAINT ocr_jobs_page_no_check CHECK (((page_no >= 1) OR (kind = ANY (ARRAY['make_pdf2'::text, 'sign_wait'::text, 'extract_fields'::text])))),
    CONSTRAINT ocr_jobs_status_check CHECK ((status = ANY (ARRAY['queued'::text, 'running'::text, 'done'::text, 'failed_transient'::text, 'page_failed'::text, 'cancelled'::text])))
);

-- SEQUENCE: ocr_jobs_seq_seq
CREATE SEQUENCE public.ocr_jobs_seq_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- SEQUENCE OWNED BY: ocr_jobs_seq_seq
ALTER SEQUENCE public.ocr_jobs_seq_seq OWNED BY public.ocr_jobs.seq;

-- TABLE: ocr_results
CREATE TABLE public.ocr_results (
    id uuid NOT NULL,
    document_id uuid NOT NULL,
    page_no integer NOT NULL,
    engine_version_id uuid NOT NULL,
    text_source text DEFAULT 'ocr_raw'::text NOT NULL,
    content text NOT NULL,
    mean_confidence real,
    low_conf_ratio real,
    boxes jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT ocr_results_low_conf_ratio_check CHECK (((low_conf_ratio >= (0)::double precision) AND (low_conf_ratio <= (1)::double precision))),
    CONSTRAINT ocr_results_mean_confidence_check CHECK (((mean_confidence >= (0)::double precision) AND (mean_confidence <= (1)::double precision))),
    CONSTRAINT ocr_results_page_no_check CHECK ((page_no >= 1)),
    CONSTRAINT ocr_results_text_source_check CHECK ((text_source = ANY (ARRAY['ocr_raw'::text, 'ocr_corrected'::text])))
);

-- TABLE: packages
CREATE TABLE public.packages (
    id uuid NOT NULL,
    project_id uuid NOT NULL,
    requested_by uuid,
    kind text NOT NULL,
    status text DEFAULT 'queued'::text NOT NULL,
    ho_so_ref text,
    label text NOT NULL,
    request_json jsonb NOT NULL,
    objid text,
    zip_key text,
    zip_name text,
    zip_sha256 text,
    zip_size bigint,
    document_count integer DEFAULT 0 NOT NULL,
    error text,
    attempts integer DEFAULT 0 NOT NULL,
    locked_by text,
    lease_until timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    submission_id uuid,
    converted_from uuid,
    virus_scan text,
    stt_from integer,
    stt_to integer,
    CONSTRAINT ck_packages_stt_range CHECK ((((stt_from IS NULL) = (stt_to IS NULL)) AND ((stt_from IS NULL) OR ((stt_from >= 1) AND (stt_from <= stt_to))))),
    CONSTRAINT packages_kind_check CHECK ((kind = ANY (ARRAY['aip_hoso'::text, 'aip_tailieu'::text, 'sip_hoso'::text, 'sip_tailieu'::text, 'dip'::text]))),
    CONSTRAINT packages_status_check CHECK ((status = ANY (ARRAY['queued'::text, 'building'::text, 'built'::text, 'failed'::text])))
);

-- TABLE: pages
CREATE TABLE public.pages (
    id uuid NOT NULL,
    document_id uuid NOT NULL,
    page_no integer NOT NULL,
    CONSTRAINT pages_page_no_check CHECK ((page_no >= 1))
);

-- TABLE: project_admins
CREATE TABLE public.project_admins (
    project_id uuid NOT NULL,
    admin_id uuid NOT NULL,
    granted_by uuid,
    granted_at timestamp with time zone DEFAULT now() NOT NULL
);

-- TABLE: project_members
CREATE TABLE public.project_members (
    project_id uuid NOT NULL,
    user_id uuid NOT NULL,
    added_by uuid,
    added_at timestamp with time zone DEFAULT now() NOT NULL
);

-- TABLE: project_trees
CREATE TABLE public.project_trees (
    project_id uuid NOT NULL,
    version integer DEFAULT 1 NOT NULL,
    payload jsonb NOT NULL,
    published_by uuid,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);

-- TABLE: projects
CREATE TABLE public.projects (
    id uuid NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

-- TABLE: records
CREATE TABLE public.records (
    id uuid NOT NULL,
    project_id uuid NOT NULL,
    created_by uuid,
    kind text NOT NULL,
    so_bien_ban text NOT NULL,
    title text NOT NULL,
    submission_id uuid,
    payload jsonb NOT NULL,
    pdf_key text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT records_kind_check CHECK ((kind = ANY (ARRAY['kiem_tra'::text, 'nhat_ky_sao_luu'::text, 'sao_luu'::text, 'su_co'::text, 'thu_nop'::text, 'thu_nop_truc_tiep'::text, 'giao_nhan'::text, 'tra_lai'::text])))
);

-- TABLE: refresh_tokens
CREATE TABLE public.refresh_tokens (
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    device_id uuid,
    token_hash text NOT NULL,
    issued_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    rotated_from uuid,
    revoked_at timestamp with time zone,
    revoked_reason text
);

-- TABLE: review_queue
CREATE TABLE public.review_queue (
    id uuid NOT NULL,
    document_id uuid NOT NULL,
    page_no integer NOT NULL,
    reason text NOT NULL,
    status text DEFAULT 'open'::text NOT NULL,
    assigned_to uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    resolved_at timestamp with time zone,
    CONSTRAINT review_queue_page_no_check CHECK ((page_no >= 1)),
    CONSTRAINT review_queue_reason_check CHECK ((reason = ANY (ARRAY['low_confidence'::text, 'heuristic_flag'::text, 'llm_disagreement'::text, 'page_failed'::text, 'vlm_fallback'::text, 'empty_page'::text, 'vlm_primary'::text]))),
    CONSTRAINT review_queue_status_check CHECK ((status = ANY (ARRAY['open'::text, 'in_review'::text, 'resolved'::text, 'skipped'::text])))
);

-- TABLE: sign_sessions
CREATE TABLE public.sign_sessions (
    id uuid NOT NULL,
    document_id uuid NOT NULL,
    signer_user_id uuid,
    status text DEFAULT 'open'::text NOT NULL,
    signer_cert bytea,
    tbs_digest bytea,
    prepared_key text,
    signed_key text,
    reject_reason text,
    lease_until timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    signed_attrs bytea,
    signer_intermediates bytea[],
    CONSTRAINT sign_sessions_status_check CHECK ((status = ANY (ARRAY['open'::text, 'signed'::text, 'rejected'::text, 'released'::text])))
);

-- TABLE: submission_items
CREATE TABLE public.submission_items (
    id uuid NOT NULL,
    submission_id uuid NOT NULL,
    stt integer NOT NULL,
    ma_ho_so text NOT NULL,
    tieu_de text NOT NULL,
    thoi_gian_bat_dau text,
    thoi_gian_ket_thuc text,
    so_trang integer,
    muc_do_tiep_can text,
    ghi_chu text,
    review_result text DEFAULT 'pending'::text NOT NULL,
    review_note text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT submission_items_review_result_check CHECK ((review_result = ANY (ARRAY['pending'::text, 'dat'::text, 'khong_dat'::text])))
);

-- TABLE: submissions
CREATE TABLE public.submissions (
    id uuid NOT NULL,
    project_id uuid NOT NULL,
    created_by uuid,
    ma_yeu_cau text NOT NULL,
    org_name text NOT NULL,
    org_code text NOT NULL,
    da_nop_giay boolean DEFAULT false NOT NULL,
    so_lan_nop integer DEFAULT 1 NOT NULL,
    summary text,
    fond_code text NOT NULL,
    phuong_an_phan_loai text,
    cach_thuc text DEFAULT 'truc_tuyen'::text NOT NULL,
    du_kien_thoi_gian text,
    lien_he text,
    ghi_chu text,
    total_pages bigint,
    total_bytes bigint,
    status text DEFAULT 'registered'::text NOT NULL,
    decision_note text,
    reconcile_note text,
    reception_note text,
    approval_doc text,
    approval_note text,
    confirmed_at timestamp with time zone,
    received_at timestamp with time zone,
    approved_at timestamp with time zone,
    completed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    han_nop timestamp with time zone,
    CONSTRAINT submissions_cach_thuc_check CHECK ((cach_thuc = ANY (ARRAY['truc_tiep'::text, 'truc_tuyen'::text]))),
    CONSTRAINT submissions_status_check CHECK ((status = ANY (ARRAY['registered'::text, 'confirmed'::text, 'refused'::text, 'awaiting_reception'::text, 'received'::text, 'approved'::text, 'completed'::text])))
);

-- TABLE: upload_batches
CREATE TABLE public.upload_batches (
    id uuid NOT NULL,
    forwarded_by uuid,
    device_id uuid,
    note text,
    status text DEFAULT 'open'::text NOT NULL,
    forwarded integer DEFAULT 0 NOT NULL,
    skipped integer DEFAULT 0 NOT NULL,
    failed integer DEFAULT 0 NOT NULL,
    rejected integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    completed_at timestamp with time zone,
    CONSTRAINT ck_upload_batch_status CHECK ((status = ANY (ARRAY['open'::text, 'completed'::text, 'abandoned'::text])))
);

-- TABLE: user_permission_overrides
CREATE TABLE public.user_permission_overrides (
    user_id uuid NOT NULL,
    permission text NOT NULL,
    effect text NOT NULL,
    granted_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT user_permission_overrides_effect_check CHECK ((effect = ANY (ARRAY['grant'::text, 'deny'::text])))
);

-- TABLE: user_stats
CREATE TABLE public.user_stats (
    user_id uuid NOT NULL,
    project_id uuid,
    pages_scanned bigint DEFAULT 0 NOT NULL,
    ho_so_count bigint DEFAULT 0 NOT NULL,
    pdf_count bigint DEFAULT 0 NOT NULL,
    pdf_bytes bigint DEFAULT 0 NOT NULL,
    reported_at timestamp with time zone DEFAULT now() NOT NULL
);

-- TABLE: users
CREATE TABLE public.users (
    id uuid NOT NULL,
    username text NOT NULL,
    email text,
    display_name text NOT NULL,
    pwd_hash text NOT NULL,
    role text NOT NULL,
    project_id uuid,
    is_active boolean DEFAULT true NOT NULL,
    token_version integer DEFAULT 0 NOT NULL,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    CONSTRAINT users_role_check CHECK ((role = ANY (ARRAY['super_admin'::text, 'admin'::text, 'user'::text, 'customer'::text])))
);

-- DEFAULT: audit_log id
ALTER TABLE ONLY public.audit_log ALTER COLUMN id SET DEFAULT nextval('public.audit_log_id_seq'::regclass);

-- DEFAULT: change_log seq
ALTER TABLE ONLY public.change_log ALTER COLUMN seq SET DEFAULT nextval('public.change_log_seq_seq'::regclass);

-- DEFAULT: ocr_jobs seq
ALTER TABLE ONLY public.ocr_jobs ALTER COLUMN seq SET DEFAULT nextval('public.ocr_jobs_seq_seq'::regclass);

-- CONSTRAINT: access_fee_rates access_fee_rates_pkey
ALTER TABLE ONLY public.access_fee_rates
    ADD CONSTRAINT access_fee_rates_pkey PRIMARY KEY (project_id);

-- CONSTRAINT: access_request_items access_request_items_pkey
ALTER TABLE ONLY public.access_request_items
    ADD CONSTRAINT access_request_items_pkey PRIMARY KEY (id);

-- CONSTRAINT: access_request_items access_request_items_request_id_document_id_key
ALTER TABLE ONLY public.access_request_items
    ADD CONSTRAINT access_request_items_request_id_document_id_key UNIQUE (request_id, document_id);

-- CONSTRAINT: access_requests access_requests_pkey
ALTER TABLE ONLY public.access_requests
    ADD CONSTRAINT access_requests_pkey PRIMARY KEY (id);

-- CONSTRAINT: audit_log audit_log_pkey
ALTER TABLE ONLY public.audit_log
    ADD CONSTRAINT audit_log_pkey PRIMARY KEY (id);

-- CONSTRAINT: backup_runs backup_runs_pkey
ALTER TABLE ONLY public.backup_runs
    ADD CONSTRAINT backup_runs_pkey PRIMARY KEY (id);

-- CONSTRAINT: change_log change_log_pkey
ALTER TABLE ONLY public.change_log
    ADD CONSTRAINT change_log_pkey PRIMARY KEY (seq);

-- CONSTRAINT: devices devices_pkey
ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_pkey PRIMARY KEY (id);

-- CONSTRAINT: disposal_batches disposal_batches_pkey
ALTER TABLE ONLY public.disposal_batches
    ADD CONSTRAINT disposal_batches_pkey PRIMARY KEY (id);

-- CONSTRAINT: disposal_entries disposal_entries_pkey
ALTER TABLE ONLY public.disposal_entries
    ADD CONSTRAINT disposal_entries_pkey PRIMARY KEY (id);

-- CONSTRAINT: disposal_log disposal_log_pkey
ALTER TABLE ONLY public.disposal_log
    ADD CONSTRAINT disposal_log_pkey PRIMARY KEY (id);

-- CONSTRAINT: documents documents_pkey
ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_pkey PRIMARY KEY (id);

-- CONSTRAINT: engine_versions engine_versions_name_precision_mode_key
ALTER TABLE ONLY public.engine_versions
    ADD CONSTRAINT engine_versions_name_precision_mode_key UNIQUE (name, precision_mode);

-- CONSTRAINT: engine_versions engine_versions_pkey
ALTER TABLE ONLY public.engine_versions
    ADD CONSTRAINT engine_versions_pkey PRIMARY KEY (id);

-- CONSTRAINT: info_requests info_requests_pkey
ALTER TABLE ONLY public.info_requests
    ADD CONSTRAINT info_requests_pkey PRIMARY KEY (id);

-- CONSTRAINT: integrity_checks integrity_checks_pkey
ALTER TABLE ONLY public.integrity_checks
    ADD CONSTRAINT integrity_checks_pkey PRIMARY KEY (id);

-- CONSTRAINT: notifications notifications_pkey
ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_pkey PRIMARY KEY (id);

-- CONSTRAINT: ocr_fields ocr_fields_pkey
ALTER TABLE ONLY public.ocr_fields
    ADD CONSTRAINT ocr_fields_pkey PRIMARY KEY (id);

-- CONSTRAINT: ocr_jobs ocr_jobs_pkey
ALTER TABLE ONLY public.ocr_jobs
    ADD CONSTRAINT ocr_jobs_pkey PRIMARY KEY (id);

-- CONSTRAINT: ocr_results ocr_results_pkey
ALTER TABLE ONLY public.ocr_results
    ADD CONSTRAINT ocr_results_pkey PRIMARY KEY (id);

-- CONSTRAINT: packages packages_pkey
ALTER TABLE ONLY public.packages
    ADD CONSTRAINT packages_pkey PRIMARY KEY (id);

-- CONSTRAINT: pages pages_document_id_page_no_key
ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_document_id_page_no_key UNIQUE (document_id, page_no);

-- CONSTRAINT: pages pages_pkey
ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_pkey PRIMARY KEY (id);

-- CONSTRAINT: project_admins project_admins_pkey
ALTER TABLE ONLY public.project_admins
    ADD CONSTRAINT project_admins_pkey PRIMARY KEY (project_id, admin_id);

-- CONSTRAINT: project_members project_members_pkey
ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_members_pkey PRIMARY KEY (project_id, user_id);

-- CONSTRAINT: project_trees project_trees_pkey
ALTER TABLE ONLY public.project_trees
    ADD CONSTRAINT project_trees_pkey PRIMARY KEY (project_id);

-- CONSTRAINT: projects projects_code_key
ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_code_key UNIQUE (code);

-- CONSTRAINT: projects projects_pkey
ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_pkey PRIMARY KEY (id);

-- CONSTRAINT: records records_pkey
ALTER TABLE ONLY public.records
    ADD CONSTRAINT records_pkey PRIMARY KEY (id);

-- CONSTRAINT: refresh_tokens refresh_tokens_pkey
ALTER TABLE ONLY public.refresh_tokens
    ADD CONSTRAINT refresh_tokens_pkey PRIMARY KEY (id);

-- CONSTRAINT: review_queue review_queue_pkey
ALTER TABLE ONLY public.review_queue
    ADD CONSTRAINT review_queue_pkey PRIMARY KEY (id);

-- CONSTRAINT: sign_sessions sign_sessions_pkey
ALTER TABLE ONLY public.sign_sessions
    ADD CONSTRAINT sign_sessions_pkey PRIMARY KEY (id);

-- CONSTRAINT: submission_items submission_items_pkey
ALTER TABLE ONLY public.submission_items
    ADD CONSTRAINT submission_items_pkey PRIMARY KEY (id);

-- CONSTRAINT: submissions submissions_pkey
ALTER TABLE ONLY public.submissions
    ADD CONSTRAINT submissions_pkey PRIMARY KEY (id);

-- CONSTRAINT: upload_batches upload_batches_pkey
ALTER TABLE ONLY public.upload_batches
    ADD CONSTRAINT upload_batches_pkey PRIMARY KEY (id);

-- CONSTRAINT: user_permission_overrides user_permission_overrides_pkey
ALTER TABLE ONLY public.user_permission_overrides
    ADD CONSTRAINT user_permission_overrides_pkey PRIMARY KEY (user_id, permission);

-- CONSTRAINT: user_stats user_stats_pkey
ALTER TABLE ONLY public.user_stats
    ADD CONSTRAINT user_stats_pkey PRIMARY KEY (user_id);

-- CONSTRAINT: users users_email_key
ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);

-- CONSTRAINT: users users_pkey
ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);

-- CONSTRAINT: users users_username_key
ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);

-- INDEX: ix_access_items_request
CREATE INDEX ix_access_items_request ON public.access_request_items USING btree (request_id, stt);

-- INDEX: ix_access_requests_expiry
CREATE INDEX ix_access_requests_expiry ON public.access_requests USING btree (han_su_dung) WHERE (status = 'approved'::text);

-- INDEX: ix_access_requests_project
CREATE INDEX ix_access_requests_project ON public.access_requests USING btree (project_id, created_at DESC);

-- INDEX: ix_audit_actor
CREATE INDEX ix_audit_actor ON public.audit_log USING btree (actor_id, at DESC);

-- INDEX: ix_backup_runs_created
CREATE INDEX ix_backup_runs_created ON public.backup_runs USING btree (created_at DESC);

-- INDEX: ix_change_log_at
CREATE INDEX ix_change_log_at ON public.change_log USING btree (at);

-- INDEX: ix_disposal_batches_project
CREATE INDEX ix_disposal_batches_project ON public.disposal_batches USING btree (project_id, created_at DESC);

-- INDEX: ix_disposal_entries_batch
CREATE INDEX ix_disposal_entries_batch ON public.disposal_entries USING btree (batch_id);

-- INDEX: ix_disposal_entries_project
CREATE INDEX ix_disposal_entries_project ON public.disposal_entries USING btree (project_id, created_at DESC);

-- INDEX: ix_disposal_log_batch
CREATE INDEX ix_disposal_log_batch ON public.disposal_log USING btree (batch_id, created_at);

-- INDEX: ix_disposal_log_project
CREATE INDEX ix_disposal_log_project ON public.disposal_log USING btree (project_id, created_at);

-- INDEX: ix_documents_batch
CREATE INDEX ix_documents_batch ON public.documents USING btree (batch_id) WHERE (batch_id IS NOT NULL);

-- INDEX: ix_documents_het_han
CREATE INDEX ix_documents_het_han ON public.documents USING btree (het_han_luu_tru) WHERE ((het_han_luu_tru IS NOT NULL) AND (status <> 'disposed'::text));

-- INDEX: ix_documents_project
CREATE INDEX ix_documents_project ON public.documents USING btree (project_id);

-- INDEX: ix_documents_review
CREATE INDEX ix_documents_review ON public.documents USING btree (project_id, review_status);

-- INDEX: ix_documents_status
CREATE INDEX ix_documents_status ON public.documents USING btree (status);

-- INDEX: ix_documents_uploaded_at
CREATE INDEX ix_documents_uploaded_at ON public.documents USING btree (uploaded_at);

-- INDEX: ix_info_requests_project
CREATE INDEX ix_info_requests_project ON public.info_requests USING btree (project_id, created_at DESC);

-- INDEX: ix_integrity_document
CREATE INDEX ix_integrity_document ON public.integrity_checks USING btree (document_id, checked_at DESC);

-- INDEX: ix_integrity_project
CREATE INDEX ix_integrity_project ON public.integrity_checks USING btree (project_id, checked_at DESC);

-- INDEX: ix_notifications_unread
CREATE INDEX ix_notifications_unread ON public.notifications USING btree (user_id) WHERE (read_at IS NULL);

-- INDEX: ix_notifications_user
CREATE INDEX ix_notifications_user ON public.notifications USING btree (user_id, created_at DESC);

-- INDEX: ix_ocr_fields_document
CREATE INDEX ix_ocr_fields_document ON public.ocr_fields USING btree (document_id);

-- INDEX: ix_ocr_fields_verified
CREATE INDEX ix_ocr_fields_verified ON public.ocr_fields USING btree (field_key) WHERE (verified = true);

-- INDEX: ix_ocr_jobs_claimable
CREATE INDEX ix_ocr_jobs_claimable ON public.ocr_jobs USING btree (priority, seq) WHERE (status = 'queued'::text);

-- INDEX: ix_ocr_jobs_running_lease
CREATE INDEX ix_ocr_jobs_running_lease ON public.ocr_jobs USING btree (lease_until) WHERE (status = 'running'::text);

-- INDEX: ix_ocr_results_document
CREATE INDEX ix_ocr_results_document ON public.ocr_results USING btree (document_id);

-- INDEX: ix_packages_building_lease
CREATE INDEX ix_packages_building_lease ON public.packages USING btree (lease_until) WHERE (status = 'building'::text);

-- INDEX: ix_packages_claim
CREATE INDEX ix_packages_claim ON public.packages USING btree (created_at) WHERE (status = 'queued'::text);

-- INDEX: ix_packages_project
CREATE INDEX ix_packages_project ON public.packages USING btree (project_id, created_at DESC);

-- INDEX: ix_packages_submission
CREATE INDEX ix_packages_submission ON public.packages USING btree (submission_id);

-- INDEX: ix_pages_document
CREATE INDEX ix_pages_document ON public.pages USING btree (document_id);

-- INDEX: ix_project_admins_admin
CREATE INDEX ix_project_admins_admin ON public.project_admins USING btree (admin_id);

-- INDEX: ix_records_project
CREATE INDEX ix_records_project ON public.records USING btree (project_id, created_at DESC);

-- INDEX: ix_records_submission
CREATE INDEX ix_records_submission ON public.records USING btree (submission_id);

-- INDEX: ix_refresh_device_active
CREATE INDEX ix_refresh_device_active ON public.refresh_tokens USING btree (device_id) WHERE (revoked_at IS NULL);

-- INDEX: ix_refresh_user_active
CREATE INDEX ix_refresh_user_active ON public.refresh_tokens USING btree (user_id) WHERE (revoked_at IS NULL);

-- INDEX: ix_review_queue_open
CREATE INDEX ix_review_queue_open ON public.review_queue USING btree (created_at) WHERE (status = 'open'::text);

-- INDEX: ix_sign_sessions_document
CREATE INDEX ix_sign_sessions_document ON public.sign_sessions USING btree (document_id);

-- INDEX: ix_sign_sessions_open_lease
CREATE INDEX ix_sign_sessions_open_lease ON public.sign_sessions USING btree (lease_until) WHERE (status = 'open'::text);

-- INDEX: ix_submissions_project
CREATE INDEX ix_submissions_project ON public.submissions USING btree (project_id, created_at DESC);

-- INDEX: ix_upload_batches_by
CREATE INDEX ix_upload_batches_by ON public.upload_batches USING btree (forwarded_by, created_at DESC);

-- INDEX: ix_upload_batches_open
CREATE INDEX ix_upload_batches_open ON public.upload_batches USING btree (created_at) WHERE (status = 'open'::text);

-- INDEX: ix_user_perm_overrides_user
CREATE INDEX ix_user_perm_overrides_user ON public.user_permission_overrides USING btree (user_id);

-- INDEX: ix_user_stats_project
CREATE INDEX ix_user_stats_project ON public.user_stats USING btree (project_id);

-- INDEX: ux_disposal_entries_active
CREATE UNIQUE INDEX ux_disposal_entries_active ON public.disposal_entries USING btree (document_id) WHERE (status = 'xem_xet'::text);

-- INDEX: ux_documents_hash
CREATE UNIQUE INDEX ux_documents_hash ON public.documents USING btree (project_id, blake3);

-- INDEX: ux_notifications_sla
CREATE UNIQUE INDEX ux_notifications_sla ON public.notifications USING btree (submission_id, user_id, kind) WHERE (kind ~~ 'sla_%'::text);

-- INDEX: ux_notifications_sla_info
CREATE UNIQUE INDEX ux_notifications_sla_info ON public.notifications USING btree (info_request_id, user_id, kind) WHERE (kind = 'sla_info_overdue'::text);

-- INDEX: ux_ocr_fields_key
CREATE UNIQUE INDEX ux_ocr_fields_key ON public.ocr_fields USING btree (document_id, page_no, engine_version_id, text_source, field_key);

-- INDEX: ux_ocr_jobs_active_extract_fields
CREATE UNIQUE INDEX ux_ocr_jobs_active_extract_fields ON public.ocr_jobs USING btree (document_id) WHERE ((kind = 'extract_fields'::text) AND (status = ANY (ARRAY['queued'::text, 'running'::text])));

-- INDEX: ux_ocr_jobs_active_make_pdf2
CREATE UNIQUE INDEX ux_ocr_jobs_active_make_pdf2 ON public.ocr_jobs USING btree (document_id) WHERE ((kind = 'make_pdf2'::text) AND (status = ANY (ARRAY['queued'::text, 'running'::text])));

-- INDEX: ux_ocr_jobs_active_sign_wait
CREATE UNIQUE INDEX ux_ocr_jobs_active_sign_wait ON public.ocr_jobs USING btree (document_id) WHERE ((kind = 'sign_wait'::text) AND (status = ANY (ARRAY['queued'::text, 'running'::text])));

-- INDEX: ux_ocr_jobs_one_ocr_per_page
CREATE UNIQUE INDEX ux_ocr_jobs_one_ocr_per_page ON public.ocr_jobs USING btree (document_id, page_no) WHERE (kind = 'ocr'::text);

-- INDEX: ux_ocr_results_page_engine_source
CREATE UNIQUE INDEX ux_ocr_results_page_engine_source ON public.ocr_results USING btree (document_id, page_no, engine_version_id, text_source);

-- INDEX: ux_records_so_bien_ban
CREATE UNIQUE INDEX ux_records_so_bien_ban ON public.records USING btree (project_id, so_bien_ban);

-- INDEX: ux_refresh_token_hash
CREATE UNIQUE INDEX ux_refresh_token_hash ON public.refresh_tokens USING btree (token_hash);

-- INDEX: ux_review_open_per_page
CREATE UNIQUE INDEX ux_review_open_per_page ON public.review_queue USING btree (document_id, page_no) WHERE (status = ANY (ARRAY['open'::text, 'in_review'::text]));

-- INDEX: ux_sign_sessions_open_per_doc
CREATE UNIQUE INDEX ux_sign_sessions_open_per_doc ON public.sign_sessions USING btree (document_id) WHERE (status = 'open'::text);

-- INDEX: ux_submission_items_ma_ho_so
CREATE UNIQUE INDEX ux_submission_items_ma_ho_so ON public.submission_items USING btree (submission_id, ma_ho_so);

-- INDEX: ux_submission_items_stt
CREATE UNIQUE INDEX ux_submission_items_stt ON public.submission_items USING btree (submission_id, stt);

-- INDEX: ux_submissions_ma_yeu_cau
CREATE UNIQUE INDEX ux_submissions_ma_yeu_cau ON public.submissions USING btree (project_id, ma_yeu_cau);

-- TRIGGER: access_fee_rates tg_access_fee_rates_changed
CREATE TRIGGER tg_access_fee_rates_changed AFTER INSERT OR DELETE OR UPDATE ON public.access_fee_rates FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('access');

-- TRIGGER: access_request_items tg_access_request_items_changed
CREATE TRIGGER tg_access_request_items_changed AFTER INSERT OR DELETE OR UPDATE ON public.access_request_items FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('access');

-- TRIGGER: access_requests tg_access_requests_changed
CREATE TRIGGER tg_access_requests_changed AFTER INSERT OR DELETE OR UPDATE ON public.access_requests FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('access');

-- TRIGGER: backup_runs tg_backup_runs_changed
CREATE TRIGGER tg_backup_runs_changed AFTER INSERT OR DELETE OR UPDATE ON public.backup_runs FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('ops');

-- TRIGGER: devices tg_devices_changed
CREATE TRIGGER tg_devices_changed AFTER INSERT OR DELETE OR UPDATE ON public.devices FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('devices');

-- TRIGGER: disposal_batches tg_disposal_batches_changed
CREATE TRIGGER tg_disposal_batches_changed AFTER INSERT OR DELETE OR UPDATE ON public.disposal_batches FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('disposal');

-- TRIGGER: disposal_entries tg_disposal_entries_changed
CREATE TRIGGER tg_disposal_entries_changed AFTER INSERT OR DELETE OR UPDATE ON public.disposal_entries FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('disposal');

-- TRIGGER: disposal_log tg_disposal_log_changed
CREATE TRIGGER tg_disposal_log_changed AFTER INSERT OR DELETE OR UPDATE ON public.disposal_log FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('disposal');

-- TRIGGER: documents tg_documents_changed
CREATE TRIGGER tg_documents_changed AFTER INSERT OR DELETE OR UPDATE ON public.documents FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('docs');

-- TRIGGER: info_requests tg_info_requests_changed
CREATE TRIGGER tg_info_requests_changed AFTER INSERT OR DELETE OR UPDATE ON public.info_requests FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('info');

-- TRIGGER: integrity_checks tg_integrity_checks_changed
CREATE TRIGGER tg_integrity_checks_changed AFTER INSERT OR DELETE OR UPDATE ON public.integrity_checks FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('ops');

-- TRIGGER: notifications tg_notifications_changed
CREATE TRIGGER tg_notifications_changed AFTER INSERT OR DELETE OR UPDATE ON public.notifications FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('notifications');

-- TRIGGER: ocr_fields tg_ocr_fields_changed
CREATE TRIGGER tg_ocr_fields_changed AFTER INSERT OR DELETE OR UPDATE ON public.ocr_fields FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('ocrFields');

-- TRIGGER: ocr_jobs tg_ocr_jobs_changed
CREATE TRIGGER tg_ocr_jobs_changed AFTER INSERT OR DELETE OR UPDATE ON public.ocr_jobs FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('ocr');

-- TRIGGER: ocr_results tg_ocr_results_changed
CREATE TRIGGER tg_ocr_results_changed AFTER INSERT OR DELETE OR UPDATE ON public.ocr_results FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('ocr');

-- TRIGGER: packages tg_packages_changed
CREATE TRIGGER tg_packages_changed AFTER INSERT OR DELETE OR UPDATE ON public.packages FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('packages');

-- TRIGGER: pages tg_pages_changed
CREATE TRIGGER tg_pages_changed AFTER INSERT OR DELETE OR UPDATE ON public.pages FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('docs');

-- TRIGGER: project_admins tg_project_admins_changed
CREATE TRIGGER tg_project_admins_changed AFTER INSERT OR DELETE OR UPDATE ON public.project_admins FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('projects');

-- TRIGGER: project_members tg_project_members_changed
CREATE TRIGGER tg_project_members_changed AFTER INSERT OR DELETE OR UPDATE ON public.project_members FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('projects');

-- TRIGGER: project_trees tg_project_trees_changed
CREATE TRIGGER tg_project_trees_changed AFTER INSERT OR DELETE OR UPDATE ON public.project_trees FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('tree');

-- TRIGGER: projects tg_projects_changed
CREATE TRIGGER tg_projects_changed AFTER INSERT OR DELETE OR UPDATE ON public.projects FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('projects');

-- TRIGGER: records tg_records_changed
CREATE TRIGGER tg_records_changed AFTER INSERT OR DELETE OR UPDATE ON public.records FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('records');

-- TRIGGER: review_queue tg_review_queue_changed
CREATE TRIGGER tg_review_queue_changed AFTER INSERT OR DELETE OR UPDATE ON public.review_queue FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('ocr');

-- TRIGGER: sign_sessions tg_sign_sessions_changed
CREATE TRIGGER tg_sign_sessions_changed AFTER INSERT OR DELETE OR UPDATE ON public.sign_sessions FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('sign');

-- TRIGGER: submission_items tg_submission_items_changed
CREATE TRIGGER tg_submission_items_changed AFTER INSERT OR DELETE OR UPDATE ON public.submission_items FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('submissions');

-- TRIGGER: submissions tg_submissions_changed
CREATE TRIGGER tg_submissions_changed AFTER INSERT OR DELETE OR UPDATE ON public.submissions FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('submissions');

-- TRIGGER: upload_batches tg_upload_batches_changed
CREATE TRIGGER tg_upload_batches_changed AFTER INSERT OR DELETE OR UPDATE ON public.upload_batches FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('uploadBatches');

-- TRIGGER: user_permission_overrides tg_user_permission_overrides_changed
CREATE TRIGGER tg_user_permission_overrides_changed AFTER INSERT OR DELETE OR UPDATE ON public.user_permission_overrides FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('users');

-- TRIGGER: user_stats tg_user_stats_changed
CREATE TRIGGER tg_user_stats_changed AFTER INSERT OR DELETE OR UPDATE ON public.user_stats FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('stats');

-- TRIGGER: users tg_users_changed
CREATE TRIGGER tg_users_changed AFTER INSERT OR DELETE OR UPDATE ON public.users FOR EACH STATEMENT EXECUTE FUNCTION public.note_change('users');

-- FK CONSTRAINT: access_fee_rates access_fee_rates_project_id_fkey
ALTER TABLE ONLY public.access_fee_rates
    ADD CONSTRAINT access_fee_rates_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: access_request_items access_request_items_document_id_fkey
ALTER TABLE ONLY public.access_request_items
    ADD CONSTRAINT access_request_items_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

-- FK CONSTRAINT: access_request_items access_request_items_request_id_fkey
ALTER TABLE ONLY public.access_request_items
    ADD CONSTRAINT access_request_items_request_id_fkey FOREIGN KEY (request_id) REFERENCES public.access_requests(id) ON DELETE CASCADE;

-- FK CONSTRAINT: access_requests access_requests_project_id_fkey
ALTER TABLE ONLY public.access_requests
    ADD CONSTRAINT access_requests_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: access_requests access_requests_requester_fkey
ALTER TABLE ONLY public.access_requests
    ADD CONSTRAINT access_requests_requester_fkey FOREIGN KEY (requester) REFERENCES public.users(id);

-- FK CONSTRAINT: audit_log audit_log_actor_id_fkey
ALTER TABLE ONLY public.audit_log
    ADD CONSTRAINT audit_log_actor_id_fkey FOREIGN KEY (actor_id) REFERENCES public.users(id);

-- FK CONSTRAINT: backup_runs backup_runs_reported_by_fkey
ALTER TABLE ONLY public.backup_runs
    ADD CONSTRAINT backup_runs_reported_by_fkey FOREIGN KEY (reported_by) REFERENCES public.users(id);

-- FK CONSTRAINT: devices devices_project_id_fkey
ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id);

-- FK CONSTRAINT: devices devices_registered_by_fkey
ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_registered_by_fkey FOREIGN KEY (registered_by) REFERENCES public.users(id);

-- FK CONSTRAINT: disposal_batches disposal_batches_created_by_fkey
ALTER TABLE ONLY public.disposal_batches
    ADD CONSTRAINT disposal_batches_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);

-- FK CONSTRAINT: disposal_batches disposal_batches_decided_by_fkey
ALTER TABLE ONLY public.disposal_batches
    ADD CONSTRAINT disposal_batches_decided_by_fkey FOREIGN KEY (decided_by) REFERENCES public.users(id);

-- FK CONSTRAINT: disposal_batches disposal_batches_executed_by_fkey
ALTER TABLE ONLY public.disposal_batches
    ADD CONSTRAINT disposal_batches_executed_by_fkey FOREIGN KEY (executed_by) REFERENCES public.users(id);

-- FK CONSTRAINT: disposal_batches disposal_batches_project_id_fkey
ALTER TABLE ONLY public.disposal_batches
    ADD CONSTRAINT disposal_batches_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: disposal_entries disposal_entries_batch_id_fkey
ALTER TABLE ONLY public.disposal_entries
    ADD CONSTRAINT disposal_entries_batch_id_fkey FOREIGN KEY (batch_id) REFERENCES public.disposal_batches(id) ON DELETE SET NULL;

-- FK CONSTRAINT: disposal_entries disposal_entries_document_id_fkey
ALTER TABLE ONLY public.disposal_entries
    ADD CONSTRAINT disposal_entries_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

-- FK CONSTRAINT: disposal_entries disposal_entries_dup_of_fkey
ALTER TABLE ONLY public.disposal_entries
    ADD CONSTRAINT disposal_entries_dup_of_fkey FOREIGN KEY (dup_of) REFERENCES public.documents(id);

-- FK CONSTRAINT: disposal_entries disposal_entries_project_id_fkey
ALTER TABLE ONLY public.disposal_entries
    ADD CONSTRAINT disposal_entries_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: documents documents_batch_id_fkey
ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_batch_id_fkey FOREIGN KEY (batch_id) REFERENCES public.upload_batches(id) ON DELETE SET NULL;

-- FK CONSTRAINT: documents documents_project_id_fkey
ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: documents documents_reviewed_by_fkey
ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id);

-- FK CONSTRAINT: documents documents_uploaded_by_fkey
ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_uploaded_by_fkey FOREIGN KEY (uploaded_by) REFERENCES public.users(id);

-- FK CONSTRAINT: info_requests info_requests_project_id_fkey
ALTER TABLE ONLY public.info_requests
    ADD CONSTRAINT info_requests_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: info_requests info_requests_requester_fkey
ALTER TABLE ONLY public.info_requests
    ADD CONSTRAINT info_requests_requester_fkey FOREIGN KEY (requester) REFERENCES public.users(id);

-- FK CONSTRAINT: integrity_checks integrity_checks_document_id_fkey
ALTER TABLE ONLY public.integrity_checks
    ADD CONSTRAINT integrity_checks_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

-- FK CONSTRAINT: integrity_checks integrity_checks_project_id_fkey
ALTER TABLE ONLY public.integrity_checks
    ADD CONSTRAINT integrity_checks_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: notifications notifications_info_request_id_fkey
ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_info_request_id_fkey FOREIGN KEY (info_request_id) REFERENCES public.info_requests(id) ON DELETE CASCADE;

-- FK CONSTRAINT: notifications notifications_submission_id_fkey
ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_submission_id_fkey FOREIGN KEY (submission_id) REFERENCES public.submissions(id) ON DELETE CASCADE;

-- FK CONSTRAINT: notifications notifications_user_id_fkey
ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

-- FK CONSTRAINT: ocr_fields ocr_fields_document_id_fkey
ALTER TABLE ONLY public.ocr_fields
    ADD CONSTRAINT ocr_fields_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

-- FK CONSTRAINT: ocr_fields ocr_fields_engine_version_id_fkey
ALTER TABLE ONLY public.ocr_fields
    ADD CONSTRAINT ocr_fields_engine_version_id_fkey FOREIGN KEY (engine_version_id) REFERENCES public.engine_versions(id);

-- FK CONSTRAINT: ocr_jobs ocr_jobs_document_id_fkey
ALTER TABLE ONLY public.ocr_jobs
    ADD CONSTRAINT ocr_jobs_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

-- FK CONSTRAINT: ocr_jobs ocr_jobs_engine_version_id_fkey
ALTER TABLE ONLY public.ocr_jobs
    ADD CONSTRAINT ocr_jobs_engine_version_id_fkey FOREIGN KEY (engine_version_id) REFERENCES public.engine_versions(id);

-- FK CONSTRAINT: ocr_results ocr_results_document_id_fkey
ALTER TABLE ONLY public.ocr_results
    ADD CONSTRAINT ocr_results_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

-- FK CONSTRAINT: ocr_results ocr_results_engine_version_id_fkey
ALTER TABLE ONLY public.ocr_results
    ADD CONSTRAINT ocr_results_engine_version_id_fkey FOREIGN KEY (engine_version_id) REFERENCES public.engine_versions(id);

-- FK CONSTRAINT: packages packages_converted_from_fkey
ALTER TABLE ONLY public.packages
    ADD CONSTRAINT packages_converted_from_fkey FOREIGN KEY (converted_from) REFERENCES public.packages(id) ON DELETE SET NULL;

-- FK CONSTRAINT: packages packages_project_id_fkey
ALTER TABLE ONLY public.packages
    ADD CONSTRAINT packages_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: packages packages_requested_by_fkey
ALTER TABLE ONLY public.packages
    ADD CONSTRAINT packages_requested_by_fkey FOREIGN KEY (requested_by) REFERENCES public.users(id);

-- FK CONSTRAINT: packages packages_submission_id_fkey
ALTER TABLE ONLY public.packages
    ADD CONSTRAINT packages_submission_id_fkey FOREIGN KEY (submission_id) REFERENCES public.submissions(id) ON DELETE SET NULL;

-- FK CONSTRAINT: pages pages_document_id_fkey
ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

-- FK CONSTRAINT: project_admins project_admins_admin_id_fkey
ALTER TABLE ONLY public.project_admins
    ADD CONSTRAINT project_admins_admin_id_fkey FOREIGN KEY (admin_id) REFERENCES public.users(id) ON DELETE CASCADE;

-- FK CONSTRAINT: project_admins project_admins_granted_by_fkey
ALTER TABLE ONLY public.project_admins
    ADD CONSTRAINT project_admins_granted_by_fkey FOREIGN KEY (granted_by) REFERENCES public.users(id);

-- FK CONSTRAINT: project_admins project_admins_project_id_fkey
ALTER TABLE ONLY public.project_admins
    ADD CONSTRAINT project_admins_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: project_members project_members_added_by_fkey
ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_members_added_by_fkey FOREIGN KEY (added_by) REFERENCES public.users(id);

-- FK CONSTRAINT: project_members project_members_project_id_fkey
ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_members_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: project_members project_members_user_id_fkey
ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_members_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

-- FK CONSTRAINT: project_trees project_trees_project_id_fkey
ALTER TABLE ONLY public.project_trees
    ADD CONSTRAINT project_trees_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: project_trees project_trees_published_by_fkey
ALTER TABLE ONLY public.project_trees
    ADD CONSTRAINT project_trees_published_by_fkey FOREIGN KEY (published_by) REFERENCES public.users(id);

-- FK CONSTRAINT: records records_created_by_fkey
ALTER TABLE ONLY public.records
    ADD CONSTRAINT records_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);

-- FK CONSTRAINT: records records_project_id_fkey
ALTER TABLE ONLY public.records
    ADD CONSTRAINT records_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: records records_submission_id_fkey
ALTER TABLE ONLY public.records
    ADD CONSTRAINT records_submission_id_fkey FOREIGN KEY (submission_id) REFERENCES public.submissions(id) ON DELETE SET NULL;

-- FK CONSTRAINT: refresh_tokens refresh_tokens_device_id_fkey
ALTER TABLE ONLY public.refresh_tokens
    ADD CONSTRAINT refresh_tokens_device_id_fkey FOREIGN KEY (device_id) REFERENCES public.devices(id);

-- FK CONSTRAINT: refresh_tokens refresh_tokens_user_id_fkey
ALTER TABLE ONLY public.refresh_tokens
    ADD CONSTRAINT refresh_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

-- FK CONSTRAINT: review_queue review_queue_assigned_to_fkey
ALTER TABLE ONLY public.review_queue
    ADD CONSTRAINT review_queue_assigned_to_fkey FOREIGN KEY (assigned_to) REFERENCES public.users(id);

-- FK CONSTRAINT: review_queue review_queue_document_id_fkey
ALTER TABLE ONLY public.review_queue
    ADD CONSTRAINT review_queue_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

-- FK CONSTRAINT: sign_sessions sign_sessions_document_id_fkey
ALTER TABLE ONLY public.sign_sessions
    ADD CONSTRAINT sign_sessions_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

-- FK CONSTRAINT: sign_sessions sign_sessions_signer_user_id_fkey
ALTER TABLE ONLY public.sign_sessions
    ADD CONSTRAINT sign_sessions_signer_user_id_fkey FOREIGN KEY (signer_user_id) REFERENCES public.users(id);

-- FK CONSTRAINT: submission_items submission_items_submission_id_fkey
ALTER TABLE ONLY public.submission_items
    ADD CONSTRAINT submission_items_submission_id_fkey FOREIGN KEY (submission_id) REFERENCES public.submissions(id) ON DELETE CASCADE;

-- FK CONSTRAINT: submissions submissions_created_by_fkey
ALTER TABLE ONLY public.submissions
    ADD CONSTRAINT submissions_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);

-- FK CONSTRAINT: submissions submissions_project_id_fkey
ALTER TABLE ONLY public.submissions
    ADD CONSTRAINT submissions_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;

-- FK CONSTRAINT: upload_batches upload_batches_forwarded_by_fkey
ALTER TABLE ONLY public.upload_batches
    ADD CONSTRAINT upload_batches_forwarded_by_fkey FOREIGN KEY (forwarded_by) REFERENCES public.users(id);

-- FK CONSTRAINT: user_permission_overrides user_permission_overrides_granted_by_fkey
ALTER TABLE ONLY public.user_permission_overrides
    ADD CONSTRAINT user_permission_overrides_granted_by_fkey FOREIGN KEY (granted_by) REFERENCES public.users(id);

-- FK CONSTRAINT: user_permission_overrides user_permission_overrides_user_id_fkey
ALTER TABLE ONLY public.user_permission_overrides
    ADD CONSTRAINT user_permission_overrides_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

-- FK CONSTRAINT: user_stats user_stats_project_id_fkey
ALTER TABLE ONLY public.user_stats
    ADD CONSTRAINT user_stats_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE SET NULL;

-- FK CONSTRAINT: user_stats user_stats_user_id_fkey
ALTER TABLE ONLY public.user_stats
    ADD CONSTRAINT user_stats_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

-- FK CONSTRAINT: users users_created_by_fkey
ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);

-- FK CONSTRAINT: users users_project_id_fkey
ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- PostgreSQL database dump complete
--
