-- 22/10/2026 — Dashboard 3 góc nhìn (lãnh đạo / quản lý dự án / nhân viên): bảng tổng hợp THEO NGÀY + cài đặt KPI
-- theo dự án. Trước đây mọi số liệu đều là ảnh chụp hiện tại (quét 160k documents / 600k ocr_jobs mỗi lần hỏi), không
-- có xu hướng, không dự báo được ngày xong, và nhân viên không có API nào để tự xem sản lượng của mình.
--
-- Áp tay (không có _sqlx_migrations), chạy ở chế độ autocommit để CREATE INDEX CONCURRENTLY hợp lệ — KHÔNG bọc BEGIN:
--   docker exec -i digitext-dev-postgres-1 psql -v ON_ERROR_STOP=1 -U digitext -d digitext < migrations/0021_dashboard_daily.sql
-- Rồi tổng hợp lại toàn bộ lịch sử MỘT LẦN (theo khối 30 ngày, ~1–3 phút):
--   SELECT d::date AS tu, sum(dashboard_rollup(d::date, (d + interval '29 days')::date)) FROM generate_series(
--     (SELECT min(uploaded_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date FROM documents), current_date, interval '30 days') d GROUP BY 1 ORDER BY 1;
-- Sau đó API tự roll hôm qua + hôm nay mỗi 2 phút và 35 ngày gần nhất lúc 00:15 (crates/digitext-api/src/cli.rs).

-- ---------------------------------------------------------------------------------------------------------------
-- 1. dashboard_daily — hạt: ngày (giờ Việt Nam) × dự án × người.
--    Dòng TỔNG DỰ ÁN dùng user_id = uuid_nil() (PK không nhận NULL, ON CONFLICT cần khoá duy nhất); dòng này tính
--    ĐỘC LẬP (gồm cả bản ghi không có người và các chỉ số chỉ có ở mức dự án), không phải tổng các dòng người.
--    user_id không có FK để giữ lịch sử của người đã bị xoá tài khoản.
-- ---------------------------------------------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS dashboard_daily (
    day                     date NOT NULL,
    project_id              uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id                 uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    -- Số hoá (documents.uploaded_at · uploaded_by)
    pages_scanned           bigint NOT NULL DEFAULT 0,
    docs_uploaded           bigint NOT NULL DEFAULT 0,
    -- OCR / PDF/A (chỉ có ở dòng dự án: không có người)
    pages_ocr_done          bigint NOT NULL DEFAULT 0,   -- ocr_jobs kind='ocr' status='done' theo updated_at
    docs_pdfa               bigint NOT NULL DEFAULT 0,   -- documents.pdfa_at
    -- Ký số (sign_sessions status='signed' · signer_user_id; đếm tài liệu, không đếm phiên)
    docs_signed             bigint NOT NULL DEFAULT 0,
    -- Duyệt bóc tách (documents.reviewed_at · reviewed_by)
    docs_reviewed_approved  bigint NOT NULL DEFAULT 0,
    docs_reviewed_rejected  bigint NOT NULL DEFAULT 0,
    -- Nhập / xác nhận trường (ocr_fields.edited_* / verified_*)
    fields_edited           bigint NOT NULL DEFAULT 0,
    fields_verified         bigint NOT NULL DEFAULT 0,
    -- Gói việc (work_item_events — chỉ ghi thêm, không sửa)
    work_items_done         bigint NOT NULL DEFAULT 0,   -- action done / redone (người làm)
    work_items_checked_ok   bigint NOT NULL DEFAULT 0,   -- check_ok (người kiểm)
    work_items_checked_err  bigint NOT NULL DEFAULT 0,   -- check_err (người kiểm)
    work_items_rejected     bigint NOT NULL DEFAULT 0,   -- reject, gán cho NGƯỜI LÀM mục (work_items.done_by)
    -- Chỉnh lý (cl_lich_su theo `at` máy chủ, không dùng at_client)
    cl_ho_so_new            bigint NOT NULL DEFAULT 0,   -- entity ho_so, action add / import
    cl_ho_so_edited         bigint NOT NULL DEFAULT 0,   -- entity ho_so, action edit
    cl_van_ban_new          bigint NOT NULL DEFAULT 0,   -- entity van_ban, action add / import
    cl_fields_entered       bigint NOT NULL DEFAULT 0,   -- sum(n_new + n_edited) mọi entity
    -- Đóng gói / nộp lưu
    packages_built          bigint NOT NULL DEFAULT 0,   -- packages status='built' theo updated_at · requested_by
    submissions_created     bigint NOT NULL DEFAULT 0,   -- submissions.created_at · created_by
    submissions_completed   bigint NOT NULL DEFAULT 0,   -- submissions.completed_at (dòng dự án)
    computed_at             timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, day, user_id)
);
CREATE INDEX IF NOT EXISTS ix_dashboard_daily_user ON dashboard_daily (user_id, day)
    WHERE user_id <> '00000000-0000-0000-0000-000000000000';

-- ---------------------------------------------------------------------------------------------------------------
-- 2. project_kpi_settings — ngưỡng KPI + định mức/người/ngày, Quản lý / Quản trị dự án sửa được (thay hằng KPI cứng
--    trong desktop StaffReportView). quotas = {"pages_scanned": 800, "fields_edited": 360, ...} — khoá thuộc danh
--    sách cột đo của dashboard_daily, API kiểm trước khi ghi.
-- ---------------------------------------------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS project_kpi_settings (
    project_id                uuid PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    quotas                    jsonb NOT NULL DEFAULT '{}'::jsonb,
    excellent_pct             numeric(5,1) NOT NULL DEFAULT 100,  -- Xuất sắc: hoàn thành ≥ và lỗi ≤
    pass_pct                  numeric(5,1) NOT NULL DEFAULT 80,   -- Đạt: hoàn thành ≥ và lỗi ≤
    excellent_err_pct         numeric(5,1) NOT NULL DEFAULT 1,
    pass_err_pct              numeric(5,1) NOT NULL DEFAULT 3,
    rag_amber_lech            numeric(5,1) NOT NULL DEFAULT -5,   -- lệch = % hoàn thành − % thời gian đã qua
    rag_red_lech              numeric(5,1) NOT NULL DEFAULT -15,
    review_backlog_warn_days  integer NOT NULL DEFAULT 3,
    work_item_sla_hours       integer NOT NULL DEFAULT 48,
    group_min_users           integer NOT NULL DEFAULT 3,         -- ẩn "trung bình nhóm" khi ít hơn số người này
    updated_by                uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at                timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ck_kpi_rag CHECK (rag_red_lech < rag_amber_lech AND rag_amber_lech <= 0),
    CONSTRAINT ck_kpi_group CHECK (group_min_users >= 2)
);

-- ---------------------------------------------------------------------------------------------------------------
-- 3. Chỉ mục theo mốc thời gian cho các truy vấn tổng hợp theo ngày (các chỉ mục (by, at) có sẵn không phục vụ được
--    quét theo `at` một mình). CONCURRENTLY: không khoá bảng đang được worker ghi.
-- ---------------------------------------------------------------------------------------------------------------
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_ocr_jobs_ocr_done_at      ON ocr_jobs (updated_at) WHERE status = 'done' AND kind = 'ocr';
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_sign_sessions_signed_at   ON sign_sessions (updated_at) WHERE status = 'signed';
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_documents_reviewed_at     ON documents (reviewed_at) WHERE reviewed_at IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_ocr_fields_edited_at      ON ocr_fields (edited_at) WHERE edited_at IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_ocr_fields_verified_at    ON ocr_fields (verified_at) WHERE verified_at IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_work_item_events_at       ON work_item_events (at);
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_cl_lich_su_project_at     ON cl_lich_su (project_id, at);
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_packages_built_at         ON packages (updated_at) WHERE status = 'built';
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_submissions_completed_at  ON submissions (completed_at) WHERE completed_at IS NOT NULL;

-- ---------------------------------------------------------------------------------------------------------------
-- 4. dashboard_rollup(từ, đến) — tổng hợp lại trọn các ngày trong khoảng (cả hai đầu). Idempotent: đặt 0 → ghi đè
--    từng nguồn → xoá dòng toàn 0. Dùng chung cho job định kỳ và backfill. Mỗi nguồn một INSERT với GROUPING SETS:
--    tập (dự án, ngày, người) bỏ người NULL; tập (dự án, ngày) là dòng tổng dự án (user_id = uuid_nil()).
--    Mốc ngày: ts >= lo AND ts < hi (dùng được chỉ mục), gom theo (ts AT TIME ZONE 'Asia/Ho_Chi_Minh')::date.
-- ---------------------------------------------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION dashboard_rollup(p_from date, p_to date) RETURNS bigint
LANGUAGE plpgsql AS $$
DECLARE
    tz  constant text := 'Asia/Ho_Chi_Minh';
    nil constant uuid := '00000000-0000-0000-0000-000000000000';
    lo  timestamptz := p_from::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh';
    hi  timestamptz := (p_to + 1)::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh';
    n   bigint := 0;
    r   bigint;
BEGIN
    IF p_to < p_from THEN RETURN 0; END IF;

    UPDATE dashboard_daily SET
        pages_scanned = 0, docs_uploaded = 0, pages_ocr_done = 0, docs_pdfa = 0, docs_signed = 0,
        docs_reviewed_approved = 0, docs_reviewed_rejected = 0, fields_edited = 0, fields_verified = 0,
        work_items_done = 0, work_items_checked_ok = 0, work_items_checked_err = 0, work_items_rejected = 0,
        cl_ho_so_new = 0, cl_ho_so_edited = 0, cl_van_ban_new = 0, cl_fields_entered = 0,
        packages_built = 0, submissions_created = 0, submissions_completed = 0, computed_at = now()
    WHERE day BETWEEN p_from AND p_to;

    -- 4.1 Số hoá: trang quét + tệp đưa lên (theo người quét = uploaded_by)
    WITH src AS (
        SELECT project_id, (uploaded_at AT TIME ZONE tz)::date AS d, uploaded_by AS uid, COALESCE(page_count, 0) AS pages
        FROM documents WHERE status <> 'disposed' AND uploaded_at >= lo AND uploaded_at < hi)
    INSERT INTO dashboard_daily (project_id, day, user_id, pages_scanned, docs_uploaded)
    SELECT project_id, d, CASE WHEN GROUPING(uid) = 1 THEN nil ELSE uid END, sum(pages), count(*)
    FROM src GROUP BY GROUPING SETS ((project_id, d, uid), (project_id, d))
    HAVING GROUPING(uid) = 1 OR uid IS NOT NULL
    ON CONFLICT (project_id, day, user_id) DO UPDATE
        SET pages_scanned = EXCLUDED.pages_scanned, docs_uploaded = EXCLUDED.docs_uploaded, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    -- 4.2 OCR: trang OCR xong (ocr_jobs không có project_id → nối documents; chỉ dòng dự án)
    INSERT INTO dashboard_daily (project_id, day, user_id, pages_ocr_done)
    SELECT d.project_id, (j.updated_at AT TIME ZONE tz)::date, nil, count(*)
    FROM ocr_jobs j JOIN documents d ON d.id = j.document_id
    WHERE j.status = 'done' AND j.kind = 'ocr' AND j.updated_at >= lo AND j.updated_at < hi AND d.status <> 'disposed'
    GROUP BY 1, 2
    ON CONFLICT (project_id, day, user_id) DO UPDATE SET pages_ocr_done = EXCLUDED.pages_ocr_done, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    -- 4.3 PDF/A (chỉ dòng dự án)
    INSERT INTO dashboard_daily (project_id, day, user_id, docs_pdfa)
    SELECT project_id, (pdfa_at AT TIME ZONE tz)::date, nil, count(*)
    FROM documents WHERE status <> 'disposed' AND pdfa_at >= lo AND pdfa_at < hi
    GROUP BY 1, 2
    ON CONFLICT (project_id, day, user_id) DO UPDATE SET docs_pdfa = EXCLUDED.docs_pdfa, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    -- 4.4 Ký số: tài liệu đã ký theo người ký (một tài liệu ký lại nhiều phiên trong ngày vẫn đếm 1)
    WITH src AS (
        SELECT DISTINCT d.project_id, (s.updated_at AT TIME ZONE tz)::date AS d, s.signer_user_id AS uid, s.document_id
        FROM sign_sessions s JOIN documents d ON d.id = s.document_id
        WHERE s.status = 'signed' AND s.updated_at >= lo AND s.updated_at < hi AND d.status <> 'disposed')
    INSERT INTO dashboard_daily (project_id, day, user_id, docs_signed)
    SELECT project_id, d, CASE WHEN GROUPING(uid) = 1 THEN nil ELSE uid END, count(DISTINCT document_id)
    FROM src GROUP BY GROUPING SETS ((project_id, d, uid), (project_id, d))
    HAVING GROUPING(uid) = 1 OR uid IS NOT NULL
    ON CONFLICT (project_id, day, user_id) DO UPDATE SET docs_signed = EXCLUDED.docs_signed, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    -- 4.5 Duyệt bóc tách (trạng thái hiện tại: duyệt lại sẽ dời sang ngày mới — chấp nhận, đêm roll lại 35 ngày)
    WITH src AS (
        SELECT project_id, (reviewed_at AT TIME ZONE tz)::date AS d, reviewed_by AS uid, review_status
        FROM documents WHERE status <> 'disposed' AND reviewed_at >= lo AND reviewed_at < hi
          AND review_status IN ('approved', 'rejected'))
    INSERT INTO dashboard_daily (project_id, day, user_id, docs_reviewed_approved, docs_reviewed_rejected)
    SELECT project_id, d, CASE WHEN GROUPING(uid) = 1 THEN nil ELSE uid END,
           count(*) FILTER (WHERE review_status = 'approved'), count(*) FILTER (WHERE review_status = 'rejected')
    FROM src GROUP BY GROUPING SETS ((project_id, d, uid), (project_id, d))
    HAVING GROUPING(uid) = 1 OR uid IS NOT NULL
    ON CONFLICT (project_id, day, user_id) DO UPDATE
        SET docs_reviewed_approved = EXCLUDED.docs_reviewed_approved, docs_reviewed_rejected = EXCLUDED.docs_reviewed_rejected, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    -- 4.6 Trường đã nhập (người sửa cuối cùng của trường)
    WITH src AS (
        SELECT d.project_id, (f.edited_at AT TIME ZONE tz)::date AS d, f.edited_by AS uid
        FROM ocr_fields f JOIN documents d ON d.id = f.document_id
        WHERE f.edited_at >= lo AND f.edited_at < hi AND d.status <> 'disposed')
    INSERT INTO dashboard_daily (project_id, day, user_id, fields_edited)
    SELECT project_id, d, CASE WHEN GROUPING(uid) = 1 THEN nil ELSE uid END, count(*)
    FROM src GROUP BY GROUPING SETS ((project_id, d, uid), (project_id, d))
    HAVING GROUPING(uid) = 1 OR uid IS NOT NULL
    ON CONFLICT (project_id, day, user_id) DO UPDATE SET fields_edited = EXCLUDED.fields_edited, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    -- 4.7 Trường đã xác nhận
    WITH src AS (
        SELECT d.project_id, (f.verified_at AT TIME ZONE tz)::date AS d, f.verified_by AS uid
        FROM ocr_fields f JOIN documents d ON d.id = f.document_id
        WHERE f.verified_at >= lo AND f.verified_at < hi AND d.status <> 'disposed')
    INSERT INTO dashboard_daily (project_id, day, user_id, fields_verified)
    SELECT project_id, d, CASE WHEN GROUPING(uid) = 1 THEN nil ELSE uid END, count(*)
    FROM src GROUP BY GROUPING SETS ((project_id, d, uid), (project_id, d))
    HAVING GROUPING(uid) = 1 OR uid IS NOT NULL
    ON CONFLICT (project_id, day, user_id) DO UPDATE SET fields_verified = EXCLUDED.fields_verified, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    -- 4.8 Gói việc theo NGƯỜI THAO TÁC (làm xong / kiểm đạt / kiểm lỗi)
    WITH src AS (
        SELECT p.project_id, (e.at AT TIME ZONE tz)::date AS d, e.actor_id AS uid, e.action
        FROM work_item_events e JOIN work_items i ON i.id = e.item_id JOIN work_packages p ON p.id = i.package_id
        WHERE e.at >= lo AND e.at < hi AND e.action IN ('done', 'redone', 'check_ok', 'check_err'))
    INSERT INTO dashboard_daily (project_id, day, user_id, work_items_done, work_items_checked_ok, work_items_checked_err)
    SELECT project_id, d, CASE WHEN GROUPING(uid) = 1 THEN nil ELSE uid END,
           count(*) FILTER (WHERE action IN ('done', 'redone')),
           count(*) FILTER (WHERE action = 'check_ok'), count(*) FILTER (WHERE action = 'check_err')
    FROM src GROUP BY GROUPING SETS ((project_id, d, uid), (project_id, d))
    HAVING GROUPING(uid) = 1 OR uid IS NOT NULL
    ON CONFLICT (project_id, day, user_id) DO UPDATE
        SET work_items_done = EXCLUDED.work_items_done, work_items_checked_ok = EXCLUDED.work_items_checked_ok,
            work_items_checked_err = EXCLUDED.work_items_checked_err, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    -- 4.9 Mục bị trả: gán cho NGƯỜI LÀM mục (work_items.done_by), không phải người duyệt
    WITH src AS (
        SELECT p.project_id, (e.at AT TIME ZONE tz)::date AS d, i.done_by AS uid
        FROM work_item_events e JOIN work_items i ON i.id = e.item_id JOIN work_packages p ON p.id = i.package_id
        WHERE e.at >= lo AND e.at < hi AND e.action = 'reject')
    INSERT INTO dashboard_daily (project_id, day, user_id, work_items_rejected)
    SELECT project_id, d, CASE WHEN GROUPING(uid) = 1 THEN nil ELSE uid END, count(*)
    FROM src GROUP BY GROUPING SETS ((project_id, d, uid), (project_id, d))
    HAVING GROUPING(uid) = 1 OR uid IS NOT NULL
    ON CONFLICT (project_id, day, user_id) DO UPDATE SET work_items_rejected = EXCLUDED.work_items_rejected, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    -- 4.10 Chỉnh lý (cl_lich_su có sẵn project_id)
    WITH src AS (
        SELECT project_id, (at AT TIME ZONE tz)::date AS d, actor_id AS uid, entity, action,
               COALESCE(n_new, 0) + COALESCE(n_edited, 0) AS nf
        FROM cl_lich_su WHERE at >= lo AND at < hi)
    INSERT INTO dashboard_daily (project_id, day, user_id, cl_ho_so_new, cl_ho_so_edited, cl_van_ban_new, cl_fields_entered)
    SELECT project_id, d, CASE WHEN GROUPING(uid) = 1 THEN nil ELSE uid END,
           count(*) FILTER (WHERE entity = 'ho_so' AND action IN ('add', 'import')),
           count(*) FILTER (WHERE entity = 'ho_so' AND action = 'edit'),
           count(*) FILTER (WHERE entity = 'van_ban' AND action IN ('add', 'import')),
           sum(nf)
    FROM src GROUP BY GROUPING SETS ((project_id, d, uid), (project_id, d))
    HAVING GROUPING(uid) = 1 OR uid IS NOT NULL
    ON CONFLICT (project_id, day, user_id) DO UPDATE
        SET cl_ho_so_new = EXCLUDED.cl_ho_so_new, cl_ho_so_edited = EXCLUDED.cl_ho_so_edited,
            cl_van_ban_new = EXCLUDED.cl_van_ban_new, cl_fields_entered = EXCLUDED.cl_fields_entered, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    -- 4.11 Gói tin đã dựng
    WITH src AS (
        SELECT project_id, (updated_at AT TIME ZONE tz)::date AS d, requested_by AS uid
        FROM packages WHERE status = 'built' AND updated_at >= lo AND updated_at < hi)
    INSERT INTO dashboard_daily (project_id, day, user_id, packages_built)
    SELECT project_id, d, CASE WHEN GROUPING(uid) = 1 THEN nil ELSE uid END, count(*)
    FROM src GROUP BY GROUPING SETS ((project_id, d, uid), (project_id, d))
    HAVING GROUPING(uid) = 1 OR uid IS NOT NULL
    ON CONFLICT (project_id, day, user_id) DO UPDATE SET packages_built = EXCLUDED.packages_built, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    -- 4.12 Nộp lưu: lần nộp lập (theo người lập) + lần nộp hoàn thành (dòng dự án)
    WITH src AS (
        SELECT project_id, (created_at AT TIME ZONE tz)::date AS d, created_by AS uid
        FROM submissions WHERE created_at >= lo AND created_at < hi)
    INSERT INTO dashboard_daily (project_id, day, user_id, submissions_created)
    SELECT project_id, d, CASE WHEN GROUPING(uid) = 1 THEN nil ELSE uid END, count(*)
    FROM src GROUP BY GROUPING SETS ((project_id, d, uid), (project_id, d))
    HAVING GROUPING(uid) = 1 OR uid IS NOT NULL
    ON CONFLICT (project_id, day, user_id) DO UPDATE SET submissions_created = EXCLUDED.submissions_created, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    INSERT INTO dashboard_daily (project_id, day, user_id, submissions_completed)
    SELECT project_id, (completed_at AT TIME ZONE tz)::date, nil, count(*)
    FROM submissions WHERE completed_at >= lo AND completed_at < hi
    GROUP BY 1, 2
    ON CONFLICT (project_id, day, user_id) DO UPDATE SET submissions_completed = EXCLUDED.submissions_completed, computed_at = now();
    GET DIAGNOSTICS r = ROW_COUNT; n := n + r;

    -- 4.13 Dọn dòng không còn số liệu nào
    DELETE FROM dashboard_daily WHERE day BETWEEN p_from AND p_to
      AND pages_scanned = 0 AND docs_uploaded = 0 AND pages_ocr_done = 0 AND docs_pdfa = 0 AND docs_signed = 0
      AND docs_reviewed_approved = 0 AND docs_reviewed_rejected = 0 AND fields_edited = 0 AND fields_verified = 0
      AND work_items_done = 0 AND work_items_checked_ok = 0 AND work_items_checked_err = 0 AND work_items_rejected = 0
      AND cl_ho_so_new = 0 AND cl_ho_so_edited = 0 AND cl_van_ban_new = 0 AND cl_fields_entered = 0
      AND packages_built = 0 AND submissions_created = 0 AND submissions_completed = 0;

    RETURN n;
END $$;

-- 5. Trang OCR lỗi theo dự án (dashboard "tồn đọng"): ocr_jobs không có project_id, chỉ mục bộ phận nhỏ để nối sang
--    documents mà không quét 600k dòng.
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_ocr_jobs_failed_doc ON ocr_jobs (document_id) WHERE status IN ('page_failed', 'failed_transient');
