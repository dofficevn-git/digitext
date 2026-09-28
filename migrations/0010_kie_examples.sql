-- 28/9/2026 — "Dạy bằng ví dụ" cho bóc trường (KIE): kho ví dụ đã được người kiểm tra.
-- Khi bóc một trang, worker tìm vài ví dụ có chữ OCR GIỐNG NHẤT (cùng kiểu đầu trang / cùng cơ
-- quan) và đưa kèm cho VLM bắt chước. Người dùng sửa + Duyệt trường của một tài liệu → trang đó
-- tự thành ví dụ mới, có tác dụng ngay, không phải huấn luyện lại. Nguồn đầu tiên: 370 ảnh đã gán
-- nhãn của doc_extract_pla (plakie). Đo 47 ảnh riêng: đúng hoàn toàn 44% → 74%.
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE IF NOT EXISTS kie_examples (
    id           uuid PRIMARY KEY,
    source       text NOT NULL,                 -- 'plakie/<bộ>' | 'duyet' (người dùng duyệt trong DigiText)
    ext_ref      text,                          -- tên ảnh gốc (plakie) để nhập lại không nhân đôi
    document_id  uuid REFERENCES documents(id) ON DELETE CASCADE,
    page_no      integer,
    ocr_text     text NOT NULL,                 -- chữ OCR phần đầu trang (tối đa ~1500 ký tự)
    fields       jsonb NOT NULL,                -- {khoá trường: giá trị đúng}
    active       boolean NOT NULL DEFAULT true, -- tắt ví dụ sai mà không phải xoá
    created_by   uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_kie_examples_doc_page ON kie_examples (document_id, page_no) WHERE document_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS ux_kie_examples_ext ON kie_examples (source, ext_ref) WHERE ext_ref IS NOT NULL;
CREATE INDEX IF NOT EXISTS ix_kie_examples_trgm ON kie_examples USING gist (ocr_text gist_trgm_ops) WHERE active;
