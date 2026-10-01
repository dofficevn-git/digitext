# DigiText — Bản đồ dự án

> Tài liệu này do Claude khảo sát và sinh ra ngày 2026-08-27 tại commit `3634dc6`.
> Mục đích: cho người (và AI) mới vào hiểu dự án mà không phải đọc lại 98k dòng.
> Khi cấu trúc thay đổi lớn, cập nhật lại file này.

---

## 0. ĐỌC MỤC NÀY TRƯỚC — source đã bị obfuscate

**Toàn bộ mã nguồn trong repo đang ở trạng thái ĐÃ OBFUSCATE.** Đây không phải
lỗi, mà là kết quả của bộ công cụ trong [dev/obfuscate/](../dev/obfuscate/). Hệ quả
với bất kỳ ai đọc code:

| Ngôn ngữ | Bị đổi tên | Còn giữ nguyên |
| --- | --- | --- |
| Rust | hàm `f_2qu`, kiểu `Tgr`/`Th2`, biến `r_25w`, hằng `C_MO` | tên module, `pub` API, tên trait/method public (`ObjectStore::put`) |
| TypeScript | component/biến `V4p9`, `V47d`, `V4og` | đường dẫn file, tên file, một số export (`Icon`, `isTauri`) |
| Python | biến `p_983`, hàm nội bộ `q_ea` | hàm public (`iou`, `nms`, `ctc_greedy_decode`) |

Thêm hai phép biến đổi nữa:

1. **Chuỗi bị mã hoá base64.** Mọi thông báo lỗi tiếng Việt đã thành base64. Ví dụ
   trong [crates/digitext-storage/src/lib.rs](../crates/digitext-storage/src/lib.rs):
   ```rust
   #[error("a2V5IGtow7RuZyBo4bujcCBs4buHOiA={0:?}")]  // = "key không hợp lệ: "
   ```
   Muốn đọc: `echo '<chuỗi>' | base64 -d`. Lưu ý chuỗi thường bị **cắt làm nhiều
   mảnh** quanh các `{placeholder}` — phải decode từng mảnh rồi ghép.

2. **Comment và docstring đã bị xoá sạch**, chỉ chừa header copyright. Đó là lý do
   repo không có một dòng chú thích nào.

### Công cụ obfuscate

| File | Việc |
| --- | --- |
| [dev/obfuscate/rust-rename/](../dev/obfuscate/rust-rename/) | crate Rust riêng, đổi tên định danh Rust theo scope |
| [dev/obfuscate/rename_ts.mjs](../dev/obfuscate/rename_ts.mjs) | đổi tên TS/TSX |
| [dev/obfuscate/rename_py.py](../dev/obfuscate/rename_py.py) | đổi tên Python |
| [dev/obfuscate/encode_strings.mjs](../dev/obfuscate/encode_strings.mjs) | mã hoá chuỗi sang base64 |
| [dev/obfuscate/strip_comments.py](../dev/obfuscate/strip_comments.py) | xoá comment/docstring |
| [dev/obfuscate/run_strip.py](../dev/obfuscate/run_strip.py) | driver: `--check` (thử) / `--apply` (ghi đè tại chỗ) |

`run_strip.py` có **invariant an toàn**: output sau khi bỏ whitespace phải là
subsequence của input — nên stripper không thể tự ý sắp xếp lại code hay phá string
literal. Riêng Python còn so khớp AST. Đây là thiết kế tốt, giữ lại khi sửa.

### ⚠️ Đây là bản chụp đã làm sạch để giao đi, không phải repo làm việc

Bốn dấu vết dẫn tới kết luận này:

1. **Toàn bộ test của code sản phẩm đã bị gỡ.** `crates/`, `desktop/src`, `ml/`
   đều không có test nào. Chỉ còn test của chính công cụ obfuscate.
2. **Không còn file `.sql` nào** — trong khi `strip_config.py` có hẳn hàm
   `strip_sql()` và docstring nhắc thẳng chữ *"migration"*. File SQL từng tồn tại
   rồi bị loại khỏi bản này. Đây cũng là lý do không dựng lại được CSDL (mục 8).
3. **Đúng 1 commit**, message chỉ là `"commit"`, ngày 2026-08-26. Không có lịch sử
   phát triển — lịch sử nằm ở repo khác.
4. `strip_config.py` xử lý đúng bộ định dạng có mặt ở đây (yaml, toml, sql, sh,
   css, tf, pbtxt) — công cụ được viết riêng cho codebase này.

Suy ra: tồn tại một repo gốc có tên thật, comment, test, migrations và lịch sử git.
Bản này là bản sao đã chạy qua `run_strip.py --apply` rồi commit một phát.

**Hệ quả về cách làm việc:** nếu đây là bản giao thì **mọi sửa đổi phải làm ở repo
gốc**, sửa ở đây sẽ bị ghi đè ở lần làm sạch sau. Cần xác nhận repo gốc nằm ở đâu
trước khi bắt tay sửa bất cứ thứ gì.

Một điểm chưa khớp: bản giao mà vẫn còn `infra/terraform/`, `infra/ansible/`,
`infra/vault/policies/`, script huấn luyện ML, và cả công cụ obfuscate (chưa bị
obfuscate, còn nguyên docstring). Đó là tài sản nội bộ. Hoặc bản này giao cho bên
rất thân cận, hoặc quá trình làm sạch chưa xong.

---

## 1. Dự án này là gì

Hệ thống **số hoá và quản lý tài liệu lưu trữ** theo nghiệp vụ lưu trữ nhà nước
Việt Nam. Suy ra từ domain model, không phải từ tài liệu (repo không có README).

Vòng đời tài liệu chạy qua hệ thống:

```
Quét (scanner)
   └─> Thu thập / nộp lưu (submission)  ── biên bản giao nhận
         └─> OCR + trích xuất trường (KIE)
               └─> Duyệt (review queue)
                     └─> Đóng gói AIP (METS/PREMIS/EAD/TT02)
                           └─> Ký số PAdES-LTV (CMS/TSA/OCSP/DSS)
                                 └─> Lưu WORM (immutable object store)
                                       ├─> Khai thác (access request → phí → DIP)
                                       └─> Tiêu huỷ (hội đồng → thẩm định → quyết định)
```

Từ vựng nghiệp vụ xuất hiện khắp code: *hồ sơ*, *văn bản*, *nộp lưu*, *khai thác*,
*tiêu huỷ*, *chỉnh lý*, *gói tin*, *số hoá*, *phông*.

**TT02** = Thông tư 02/2019/TT-BNV về chuẩn dữ liệu thông tin đầu vào tài liệu lưu
trữ số. Có hẳn module riêng: [crates/digitext-package/src/tt02.rs](../crates/digitext-package/src/tt02.rs)
và [crates/digitext-cli/src/tt02.rs](../crates/digitext-cli/src/tt02.rs).

---

## 2. Bố cục kho mã

| Thư mục | Nội dung | Quy mô |
| --- | --- | --- |
| [crates/](../crates/) | 14 crate Rust — backend server | ~32k LOC |
| [desktop/src/](../desktop/src/) | React 18 + TS — giao diện (0.17.1) | ~38k LOC |
| [desktop/src-tauri/](../desktop/src-tauri/) | Tauri v2 — lõi app desktop (Rust) | ~22k LOC |
| [ml/](../ml/) | Python — fine-tune, benchmark, sinh data | ~6.9k LOC |
| [triton/](../triton/) | Cấu hình model Triton Inference Server | ~1.1k LOC |
| [infra/](../infra/) | Ansible, Terraform, Vault, compose, observability | — |
| [dev/](../dev/), [scripts/](../scripts/) | Script build, obfuscate, deploy | — |

Workspace Rust ở [Cargo.toml](../Cargo.toml) gốc **loại trừ** `desktop` và
`dev/obfuscate/rust-rename` — hai cái đó là workspace riêng.

---

## 3. Backend: 14 crate Rust

Sắp theo tầng phụ thuộc, dưới không phụ thuộc trên:

### Tầng nền (không phụ thuộc crate nội bộ nào)

| Crate | LOC | Việc |
| --- | --- | --- |
| [digitext-core](../crates/digitext-core/) | 836 | Kiểu domain thuần: `DocStatus`, `VanBanField`, `JobKind`/`JobStatus`, `Role`/`Perm`, `Permission` + hàm `effective()` tính quyền hiệu dụng, chuẩn hoá text |
| [digitext-pdf](../crates/digitext-pdf/) | 2015 | Sinh PDF trên `lopdf`. Có `pdfa.rs` (tuân thủ PDF/A), `icc.rs` (color profile), `glyphless.rs` (lớp text vô hình phủ lên ảnh scan) |
| [digitext-storage](../crates/digitext-storage/) | 544 | Trait `ObjectStore` (put/get/delete/list/stream có byte-range). Có chế độ **WORM** — chặn xoá object gói tin |
| [digitext-vi](../crates/digitext-vi/) | 111 | Xử lý tiếng Việt: bỏ dấu, chuẩn hoá vị trí dấu (`hoà` vs `hòa`), NFD/NFC |
| [digitext-telemetry](../crates/digitext-telemetry/) | 159 | OpenTelemetry (trace/metric/log) qua OTLP gRPC, bật bằng `DIGITEXT_OTEL_ENABLED` |

### Tầng dữ liệu

| Crate | LOC | Việc |
| --- | --- | --- |
| [digitext-db](../crates/digitext-db/) | 4622 | sqlx + Postgres. **28 module, mỗi module một bảng/nhóm nghiệp vụ**: `documents`, `submissions`, `disposal`, `access`, `ocr_jobs`, `ocr_results`, `ocr_fields`, `packages`, `sign_sessions`, `review_queue`, `users`, `permissions`, `audit`, `notifications`, `projects`, `trees`, `records`, `upload_batches`, `devices`, `tokens`, `stats`, `workdays`, `engine_versions`, `info`, `ops`, `changes`, `disposal`, `pool` |

### Tầng nghiệp vụ

| Crate | LOC | Việc |
| --- | --- | --- |
| [digitext-package](../crates/digitext-package/) | 3291 | **Đóng gói SIP/AIP/DIP (E-ARK CSIP).** `mets.rs` (METS gốc + rep), `premis.rs` (PREMIS đầy đủ), `ead.rs` (EAD mô tả lưu trữ), `tt02.rs` (chuẩn VN), `builder.rs`, `validate.rs`, `schemas.rs` (+ `src/schemas/*.xsd` chép nguyên từ chuẩn, nhúng bằng `include_str!`); `examples/sample_pack.rs` dựng gói mẫu từ một PDF để soi bằng `ml/tools/check_tt05_package.py` |
| [digitext-sign-verify](../crates/digitext-sign-verify/) | 2827 | **Ký số PAdES-LTV.** `cms.rs` (CMS/PKCS#7), `tsa.rs` (timestamp), `ocsp.rs` (kiểm thu hồi), `chain.rs` (chuỗi tin cậy), `dss.rs` (Document Security Store, 30KB — file to nhất), `pdf.rs` (nhúng chữ ký vào PDF) |
| [digitext-ocr](../crates/digitext-ocr/) | 5191 | **Crate phức tạp nhất.** Xem mục 4 |
| [digitext-token-signpdf](../crates/digitext-token-signpdf/) | 1169 | Ký PDF bằng USB token / thẻ chữ ký số |
| [digitext-ingest](../crates/digitext-ingest/) | 280 | Nạp file theo manifest JSON, **đối chiếu BLAKE3 + kích thước** trước khi nhận |
| [digitext-dispatcher](../crates/digitext-dispatcher/) | 157 | Xếp hàng job OCR. Thuật toán cân bằng theo *fair-share*: ưu tiên project bị tụt so với trọng số, rồi tới deadline, rồi FIFO |
| [digitext-wipe](../crates/digitext-wipe/) | 781 | Xoá sạch dữ liệu. Có hằng `NEVER_TRUNCATE = "_sqlx_migrations"` — chốt chặn an toàn |

### Tầng giao tiếp

| Crate | LOC | Việc |
| --- | --- | --- |
| [digitext-api](../crates/digitext-api/) | 9815 | **Crate lớn nhất.** HTTP server trên axum 0.8. Xem mục 5 |
| [digitext-cli](../crates/digitext-cli/) | 514 | CLI vận hành, phụ thuộc gần như mọi crate khác |

---

## 4. digitext-ocr — đường ống nhận dạng

Crate này vừa là thư viện vừa là **worker chạy nền** (`worker.rs` 34KB,
`package_worker.rs` 22KB — hai file to nhất crate).

| File | Việc |
| --- | --- |
| `entry.rs` (23KB) | Điểm vào xử lý một tài liệu |
| `worker.rs` (34KB) | Vòng lặp worker: nhận job từ dispatcher, chạy, ghi kết quả |
| `package_worker.rs` (22KB) | Worker riêng cho khâu đóng gói |
| `triton.rs`, `grpc.rs`, `infer.rs` | Gọi Triton Inference Server qua gRPC |
| `vlm.rs` (13KB), `vllm.rs`, `vlm_fields.rs`, `vlm_prompt_v2.rs` | **Nhánh VLM**: dùng vision-language model qua vLLM để trích xuất trường, thay/bổ sung cho OCR truyền thống |
| `fields.rs`, `fields_v2.rs` | Trích xuất trường (KIE) — có 2 thế hệ, v2 song song v1 |
| `corrector.rs` | Hậu xử lý sửa lỗi nhận dạng |
| `tiling.rs`, `assemble.rs`, `render.rs` | Cắt ảnh thành tile, ghép kết quả, render ra PDF |
| `health.rs`, `readiness.rs`, `metrics.rs` | Health check + đo đạc |
| `config.rs` (15KB) | Cấu hình — khá đồ sộ |

### Model phục vụ trên Triton

[triton/models/](../triton/models/) có 7 model, ghép thành pipeline:

| Model | Vai trò |
| --- | --- |
| `det_dbnet` | Phát hiện vùng chữ (DBNet) |
| `cls_angle` | Phân loại góc xoay |
| `pre_rec` / `post_rec` | Tiền/hậu xử lý cho nhận dạng |
| `rec_svtr_vi` | Nhận dạng chữ — SVTR bản tiếng Việt |
| `rec_vietocr` | Nhận dạng chữ — VietOCR (nhánh dự phòng) |
| `ocr_pipeline` | Model ensemble ghép các bước |

Logic thuần Python ở [triton/models/ocr_pipeline/1/ocr_logic.py](../triton/models/ocr_pipeline/1/ocr_logic.py):
IoU, NMS, giải mã CTC greedy, phân nhóm chiều rộng, **sắp xếp thứ tự đọc**
(`sort_reading_order`), và `needs_fallback()` — quyết định khi nào chuyển từ model
chính sang model dự phòng dựa trên điểm tin cậy.

### ml/ — huấn luyện và đo đạc

| Thư mục | Việc |
| --- | --- |
| `ml/finetune/svtr/`, `ml/finetune/vietocr/` | Fine-tune hai model nhận dạng |
| `ml/finetune/kie/` | Nhánh KIE lớn nhất: `distill_vlm_rec.py` (chưng cất từ VLM), `extract_vlm.py`, `build_bio.py` (nhãn BIO), `eval_kie.py`, `gate_kie.py`, `gold_review_queue.py`, `doc_type_rules.py` |
| `ml/data-engine/` | Sinh dữ liệu tổng hợp (`gen_synth_rec.py`, `synth_text.py`) |
| `ml/bench/harness/` | Đo: `run_bench.py`, `score.py`, `metrics.py`, `gate.py`, `loadtest.py` |
| `ml/bench/golden-set/` | Bộ đánh giá chuẩn — **dữ liệu bị gitignore** |

Mẫu hình đáng chú ý: có `gate.py` / `gate_kie.py` — cổng chất lượng, có lẽ chặn
việc lên model nếu chỉ số tụt.

---

## 5. digitext-api — bề mặt HTTP

axum 0.8. Hạ tầng dùng chung: `jwt.rs`, `password.rs` (argon2), `limiter.rs`
(rate limit), `client_ip.rs`, `extract.rs`, `error.rs`, `config.rs`, `app.rs` (lắp router).

19 nhóm route:

| Nhóm | Đường dẫn tiêu biểu |
| --- | --- |
| `auth` | `/auth/login`, `/refresh`, `/logout`, `/validate`, `/pubkey` |
| `users` | `/users`, `/{id}/password`, `/{id}/revoke`, `/{id}/permissions` |
| `projects` | `/projects`, `/{id}/admins`, `/{id}/members`, `/{id}/tree` |
| `documents` | `/documents/{id}/download`, `/tree`, `/ocr-all`, `/make-pdf2-all`, `/extract-fields-all`, `/rebuild-non-pdfa`, `/progress`, `/pdfa-report`, `/report`, `/{id}/review`, `/review-bulk` |
| `submissions` | `/submissions`, `/{id}/confirm`, `/refuse`, `/reconcile`, `/receive`, `/approve`, `/complete`, `/returned`, `/report`, `/dossier` |
| `access` | `/access`, `/fees`, `/{id}/approve`, `/refuse`, `/pay`, `/items/{item_id}/review`, `/read`, `/download`, `/{id}/dip` |
| `disposal` | `/disposal`, `/sweep`, `/retention`, `/batches/{id}/council`, `/appraise`, `/decide`, `/execute` |
| `info` | `/info/{id}/approve`, `/refuse`, `/pay`, `/generate`, `/fulfill`, `/download` |
| `sign` | `/sign/jobs`, `/{id}/begin`, `/{id}/complete` |
| `packages` | `/packages`, `/{id}`, `/{id}/download` |
| `ocr_fields` | `/documents/{id}/fields`, `/ocr/fields/export` |
| `upload_batches` | `/upload-batches`, `/matrix`, `/{id}` |
| `devices` | `/devices`, `/register`, `/{id}/revoke` |
| `ops` | `/ops/integrity`, `/ops/integrity/run`, `/ops/backups` |
| còn lại | `records`, `stats`, `changes`, `notifications`, `/healthz` |

Ba mẫu hình lặp lại đáng để ý:

- **Quy trình duyệt nhiều bước** giống nhau ở `access`, `info`, `disposal`,
  `submissions`: tạo → duyệt/từ chối → (thu phí) → thực hiện → tải về.
- **Thao tác hàng loạt**: `ocr-all`, `make-pdf2-all`, `extract-fields-all`,
  `rebuild-non-pdfa`, `review-bulk`, `sweep`.
- **`/changes` + `/notifications`** phục vụ đồng bộ tăng dần cho client.

---

## 6. Desktop — Tauri v2

Hai nửa: frontend React và lõi Rust, nối bằng Tauri IPC (`invoke`).

### Lõi Rust ([desktop/src-tauri/src/](../desktop/src-tauri/src/))

~90 lệnh `#[tauri::command]`, nhiều nhất ở `docs.rs` (14), `submission.rs` (12),
`access.rs` / `collect.rs` (9 mỗi file).

Điểm quan trọng: **app desktop có CSDL cục bộ riêng.** `CREATE TABLE` chỉ xuất
hiện ở phía Tauri — [db.rs](../desktop/src-tauri/src/db.rs) (6 bảng),
`dossier/store.rs`, `dossier/history.rs`, `khaithac/store.rs`, `auth/store.rs`.
Đây là kiến trúc **offline-first**: máy trạm làm việc được khi mất mạng, rồi
đồng bộ lên server.

Module đáng chú ý: `scanner/` (điều khiển máy quét), `archive/`, `catalog/`,
`dossier/`, `khaithac/` (khai thác), `report/`, `lan.rs` + `lan_identity.rs` +
`lan_session.rs` (**đồng bộ ngang hàng qua LAN**), `sync.rs`, `live.rs`,
`data_share.rs`, `collect_progress.rs`, `forward_progress.rs`.

### Frontend ([desktop/src/](../desktop/src/))

React 18 + Vite + Tailwind v4 + Radix UI (vendored trong `components/ui/`) + lucide-react.
Xuất Excel bằng `xlsx` (cài trực tiếp từ CDN SheetJS, không qua npm registry).
Bản 0.17.1 (10/2026) làm lại toàn bộ giao diện trên một bộ thành phần chung.

- `ui/` — **bộ thành phần dùng chung, màn nghiệp vụ chỉ import từ `@/ui`**
  (eslint `no-restricted-imports` chặn `@/components/ui/*` và `confirm/message`
  của `plugin-dialog` trong `features/`, trừ `features/shell`): Button, IconButton
  (bắt buộc `label`), Field (+FormGrid, gắn id/aria qua context), Input/Select/
  Textarea/Checkbox/Switch, PageHeader, Toolbar, Card, Tabs, Stepper, KpiTile,
  EmptyState, Skeleton, Kbd, Help (khối trợ giúp gập, nơi đặt "Điều N" và tên
  chuẩn), Banner/ErrorBanner, Menu, Badge, Seg, Dialog, Table (cấu hình cột hoặc
  compound `Table.Root/Head/Body/Row/HeadCell/Cell`), Pagination, toast (sonner)
  + `notifyError`, `confirm()`/`prompt()` (store mức module, host `ConfirmHost`),
  `describeError` từ `lib/errors.ts`.
- `styles/theme.css` — nguồn token duy nhất (màu trạng thái 50–800, thang chữ
  xs 12 / sm 13 / base 14); `do365-tokens.css` đóng băng; `digitext.css` chỉ còn
  luật cho các màn chưa chuyển hẳn (Quét, Trình xử lý ảnh, Kho, đăng nhập).
- `api/` — 30 module, mỗi module bọc một nhóm lệnh Tauri
- `features/` — 22 nhóm màn theo vòng đời. Menu 2 cấp ở `shell/navTree.ts`:
  Tổng quan (`home`) · Số hoá (`drive`, `scan`, `tree`, `dossier`, `upload`,
  `server`) · Xử lý (`dossier/OcrPane`, `work`, `sign`, `package`) · Chỉnh lý
  (`chinhly/*`: bản sao đồng bộ, offline được; nhập mục lục hồ sơ có dòng nhóm
  I./1. ở `chinhly/mucLucImport.ts` + `thoiGianHoSo.ts` + `NhapMucLucDialog.tsx`,
  xem `docs/CHINH-LY-SYNC.md`) · Lưu trữ số (`submission`,
  `access`, `infoops`, `records`, `disposal`) · Báo cáo (`report`,
  `chinhly/ThongKeView`) · Quản trị (`users`, `settings`). `editor` mở từ Kho/Quét.
  `khogia/` chỉ còn hub cũ; danh mục cục bộ (`catalog`) đã bỏ, thay bằng `tree`.
- `shell/` — Topbar (breadcrumb, ô **Dự án đang làm** dùng chung qua
  `state/project.tsx`, Ctrl+K `CommandPalette`, chuông `useNotifications`),
  Sidebar (bàn phím, nhớ nhóm mở, Ctrl+B), StatusBar (chip bấm được), Login /
  LockScreen / UpdateOverlay.
- `state/` — state theo ngữ cảnh React (`session`, `project`, `uploads`,
  `collects`, `drive`, `navigation`)
- `live/` — cập nhật thời gian thực: `LiveProvider`, `bus`, `topics`, `pollCore`;
  màn danh sách dùng `V41t(loader, {topics})` (giữ dữ liệu cũ khi lỗi, không
  nhấp nháy khi nạp lại)
- `domain/` — `types.ts`, `permissions.ts` (**quyền được cài đặt cả hai phía**,
  đối chiếu với `digitext-core/permission.rs`)

Kiểm sau mỗi lần sửa: `npx tsc --noEmit -p .`, `npx eslint src` (jsx-a11y mức
lỗi), `npm run build`. Không có test.

---

## 7. Hạ tầng

Compose phát triển ([infra/compose/docker-compose.yml](../infra/compose/docker-compose.yml)):

| Dịch vụ | Việc |
| --- | --- |
| `postgres:16` | CSDL chính |
| `minio` | Object store S3 (kho WORM) |
| `meilisearch` | Tìm kiếm toàn văn |
| `rustus` | Upload resumable giao thức tus, gọi webhook `ingest:8081/webhook/tus` |
| `nginx` | Reverse proxy TLS |

Mọi port đều bind `127.0.0.1` — không phơi ra ngoài. Biến môi trường dùng cú pháp
`${VAR:?...}` nên thiếu là compose báo lỗi ngay thay vì chạy với giá trị rỗng.

Còn lại: `infra/terraform/` (modules + environments), `infra/ansible/`
(inventories/playbooks/roles), `infra/vault/policies/`, `infra/observability/`
(Prometheus, Grafana, SigNoz, DCGM cho GPU), `infra/cloudflared/`,
`infra/launchd-or-systemd/` (chạy nền trên macOS lẫn Linux),
`infra/compose/triton/` và `infra/compose/vllm/` (phục vụ model).

---

## 8. Những chỗ cần biết khi sửa code

### Thiếu: schema Postgres không có trong repo

`digitext-db` dùng sqlx truy vấn 28 nhóm bảng, `digitext-wipe` nhắc tới bảng
`_sqlx_migrations` — nhưng **không có thư mục `migrations/` nào trong repo**,
`.gitignore` cũng không chặn nó, và không có cache `.sqlx/` offline.

Nghĩa là: clone repo về **không dựng lại được CSDL**. Cần tìm lại migrations hoặc
sinh lại từ đầu.

### Truy vấn không được kiểm tại thời điểm biên dịch

Không có macro `query!`/`query_as!` nào — toàn bộ dùng API runtime. Sai tên cột
sẽ nổ lúc chạy chứ không phải lúc `cargo build`. Sửa DB phải test thật.

### Quyền cài đặt hai lần

`digitext-core/src/permission.rs` (Rust) và `desktop/src/domain/permissions.ts`
(11KB, TS) đều tính quyền. Sửa một bên mà quên bên kia là lệch. Không có test
đối chiếu chéo giữa hai bản.

### Chất lượng graph khác nhau theo ngôn ngữ

`code-review-graph` phân giải lời gọi hàm tốt với TS/Python nhưng
**không phân giải được lời gọi trong Rust** (scoped resolver trả về 0 CALLS edge
trên 210 file `.rs`). Cạnh CALLS phía Rust đến từ suy đoán theo tên. Dùng
`get_impact_radius_tool` trên `.rs` thì phải mở source đối chiếu.

### Không có test nào cho code sản phẩm

`crates/` có **0** file chứa `#[test]` hay `#[cfg(test)]`. `desktop/src` và `ml/`
cũng không có test. Ba file test duy nhất trong repo là test của chính công cụ
obfuscate ([dev/obfuscate/tests/](../dev/obfuscate/tests/)) — 112 node `Test` mà
graph đếm chính là chúng, không phải test của sản phẩm.

Nghĩa là **không có lưới an toàn nào** khi sửa, kể cả ở những chỗ sai là hỏng nặng:
`sign-verify` (mật mã), `package` (đúng chuẩn TT02/METS), `wipe` (xoá dữ liệu),
`disposal` (tiêu huỷ). Mọi thay đổi phải kiểm chứng thủ công.

Việc test bị gỡ sạch là một trong những căn cứ kết luận đây là bản đã làm sạch để
giao đi — xem mục 0.

### Không có comment

Hệ quả của `strip_comments.py`. Đừng đi tìm chú thích giải thích ý đồ — không có.
Ý đồ phải suy từ tên module (còn giữ nguyên) và từ luồng dữ liệu.

### Sự kiện "thay đổi" và thống kê nặng (1/10/2026)

- `change_log` do trigger AFTER STATEMENT `note_change(topic)` trên từng bảng ghi, **không** do code Rust.
  Từ migration `0013_change_log_throttle.sql` trigger bỏ qua sự kiện cùng topic trong 15 s (`ocr`, `ocrFields`)
  hoặc 3 s (còn lại) — worker OCR xong mỗi trang từng phát một sự kiện (≈ 7.000 / giờ) khiến mọi máy trạm nạp
  lại thống kê liên tục.
- `GET /documents/progress` và `/documents/report` quét toàn bộ tài liệu (~1 s / 160k tài liệu): kết quả giữ
  30 s trong `STATS_CACHE` (`routes/documents.rs`) theo phạm vi dự án; màn Tổng quan chỉ nạp theo chu kỳ 60 s.
  Thêm màn hỏi thống kê toàn kho thì dùng lại hai endpoint này, đừng viết truy vấn gộp mới theo sự kiện.

---

## 9. Điểm nóng theo graph

Node có bậc cao nhất — sửa vào đây là lan rộng nhất:

| Node | Bậc | Vị trí |
| --- | --- | --- |
| `Icon` | 334 | [desktop/src/components/Icon.tsx](../desktop/src/components/Icon.tsx) |
| `V4p9` (ScanView) | 236 | [desktop/src/features/scan/ScanView.tsx](../desktop/src/features/scan/ScanView.tsx) |
| `invoke` | 208 | cầu nối Tauri IPC |
| `SoHoaPane` | 201 | màn hình số hoá |
| `LapHoSoView` | 192 | lập hồ sơ |
| `KhaiThacView` | 191 | khai thác |
| `data_collect` | 188 | phía Tauri |

Thống kê graph: 6079 node (4178 Function, 1200 Class, 589 File, 112 Test),
49648 cạnh (38629 CALLS, 5500 CONTAINS, 3126 IMPORTS_FROM, 1921 REFERENCES,
322 TESTED_BY, 122 IMPLEMENTS, 28 INHERITS).

---

## 10. Tra cứu nhanh

```bash
# Graph
code-review-graph status
code-review-graph search "<từ khoá>"
code-review-graph impact --base HEAD~1
code-review-graph update                  # tăng dần, tự chạy qua hook

# Giải mã chuỗi bị obfuscate
echo '<base64>' | base64 -d

# Obfuscate
python3 dev/obfuscate/run_strip.py --check    # thử, không ghi
python3 dev/obfuscate/run_strip.py --apply    # GHI ĐÈ TẠI CHỖ

# Chạy
docker compose -f infra/compose/docker-compose.yml up -d
dev/run-api.sh
dev/test-server.sh
dev/verapdf-gate.sh                       # kiểm tra tuân thủ PDF/A
```
