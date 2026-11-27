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
| [digitext-sign-verify](../crates/digitext-sign-verify/) | 2827 | **Ký số PAdES-LTV.** `cms.rs` (CMS/PKCS#7), `tsa.rs` (timestamp), `ocsp.rs` (kiểm thu hồi), `chain.rs` (chuỗi tin cậy), `dss.rs` (Document Security Store, 30KB — file to nhất), `pdf.rs` (nhúng chữ ký vào PDF), `nhieu.rs` (**kiểm MỌI chữ ký trong một PDF** + kết luận từng tệp — `verify()` chỉ kiểm một chữ ký và đòi nó phủ trọn tệp, nên tệp ký nhiều lần bị kết luận "đã bị sửa sau khi ký"; đọc ghi chú đầu file trước khi sửa gì ở đây) |
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

25 nhóm route:

| Nhóm | Đường dẫn tiêu biểu |
| --- | --- |
| `auth` | `/auth/login`, `/refresh`, `/logout`, `/validate`, `/pubkey` |
| `users` | `/users` (tạo nhận `roleCode`), `/{id}` (PATCH đổi `roleCode`), `/{id}/password`, `/{id}/revoke`, `/{id}/permissions` |
| `roles` | `/roles` GET/POST, `/roles/{code}` GET/PUT/DELETE, `/roles/{code}/reset` — bộ quyền vai trò (migration 0022, 22/10/2026) |
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
| `dashboard` | `/dashboard/overview` (lãnh đạo), `/project`, `/staff`, `/me` (nhân viên: chỉ số của mình + trung bình nhóm ẩn danh), `/kpi-settings` GET/PUT, `POST /rollup` (super) — 22/10/2026 |
| `nop_luu` | `/nop-luu/tong-quan`, `/ho-so-can-nop?loc=den_han\|qua_han`, `POST /nop-luu {phongBanId, hoSoIds[]}`, `/{id}` + `/rut`, `/ho-so/{hoSoId}/ket-qua`, `/tiep-nhan` (tự lập biên bản `giao_nhan_nop_luu`), `/tra-lai`, `/bien-ban`, `/mo-khoa` (audit), `/so-nop-luu[/bien-ban]` — Lưu trữ hiện hành Điều 19, 26/11/2026 (khối `chung`, cả hai bản) |
| `bao_quan` | `/bao-quan/tong-quan` · kế hoạch kiểm tra `ke-hoach` GET/POST, `/xem-truoc`, `/{id}` GET/PUT, `/{id}/trinh \| duyet (bk.approve, không tự duyệt) \| tu-choi \| huy \| chay?limit \| bien-ban`, `/{id}/tep`, `/{id}/ket-luan` (điền sẵn Mẫu 01) · lỗi `loi` GET, `/{id}/khac-phuc \| kiem-lai \| bo-qua` · sao lưu `lich` GET/POST, `lich/{id}` PUT, `sao-luu` GET/POST (đủ cột 0032), `sao-luu/tinh-trang`, `sao-luu/{id}/da-xoa` · `phuong-tien` GET/POST/PUT · `su-co` GET/POST, `/{id}` GET/PUT, `/{id}/prefill` (Mẫu 04), `/{id}/bien-ban` — Bảo quản Điều 29–31, 0032 (26/11/2026, khối `chung`); `/ops/*` cũ giữ nguyên; `POST /disposal/sweep` trả thêm `trungTenTep/trungBlake3/trungMucLuc` (0033) |
| `bao_cao` | `GET /bao-cao` (danh mục 19 báo cáo Điều 44.3b: mã, tên, căn cứ, nhóm, bản, bộ lọc, cột — desktop dựng màn từ đây), `GET /bao-cao/{ma}?projectId&phongId&namTu&namDen&from&to&format=json\|xlsx\|pdf` (Excel `xlsx::sheet_bang`; PDF `digitext_pdf::manual` A4 ngang khi > 7 cột, đầu văn bản hành chính + khối ký) — 27/11/2026, khối `chung`, quyền `stats.view` / `stats.view_project`; `thu_nop` chỉ bản thi công |
| `chinh_ly/muon_tra` | `GET/POST /cl/muon-tra` (phiếu nhiều hồ sơ, `so_phieu` MT-năm-stt, `giaoNgay` cần `loan.approve`), `/tong-quan`, `/{id}` GET, `/{id}/duyet \| tu-choi \| giao \| gia-han \| tra \| huy` (POST không cần thân), `/nhom/{nhomId}` GET + `/duyet \| tu-choi \| huy` (cả phiếu), `/{id}/phieu.pdf` — mượn – trả có duyệt 0034 (27/11/2026) |
| `chinh_ly/kho_gia` | `GET /cl/kho-gia/tong-hop?projectId&khoId&phongId` (sức chứa, hộp / hồ sơ đang chứa, % đầy, gần đầy), `GET /cl/kho-gia/ho-so?khoId\|giaId&format=xlsx`, `GET /cl/kho-gia/chua-hop?phongId`, `POST /cl/ho-so/gan-hop {phongBanId, rows[{hoSoId, hopSo}]}` (đặt phòng ban cho hồ sơ nhập gói rồi `he_thong_hoa::boxing`) — 0035 (27/11/2026) |
| `chuyen_doi` | `POST /chuyen-doi {projectId, documentId, mucDich?, nguoiNhan?, canCu?, chucVu?}` (`access.review`; kiểm chữ ký số lúc chuyển đổi, ghép trang thể thức PL VI vào CUỐI, chân trang i/n, cấp số `CD-{năm}-{stt:06}`, ghi `chuyen_doi_giay` + `khai_thac.chuyen_doi`), `GET /chuyen-doi?projectId&from&to[&format=xlsx]`, `GET /chuyen-doi/{id}/download` — Điều 13–16, 0037 (27/11/2026) |
| `users` (0038) | `GET/PUT /users/{id}/phong {projectId, phongIds[]}` — phạm vi phông (`user.grant_permissions`; rỗng = không giới hạn; super_admin / manager không giới hạn được); `GET /cl/pham-vi-phong?projectId` — phạm vi của chính mình |
| `co_quan` (0039) | `GET /projects/{id}/co-quan` (người thuộc dự án; trả dòng đã khai + giá trị hiệu dụng) · `PUT` (`project.update` trong phạm vi) — cấu hình cơ quan lưu trữ: mã CQ lưu trữ, tên / cấp trên / địa chỉ, loại hình, người ký + chức vụ, hạn nộp lưu / mốc nhắc / quyền sử dụng; không khai = tên dự án + 60 / 15-7-1 / 360 (27/11/2026) |
| `chinh_ly/phong_lich_su` (0039) | `GET /cl/phong/{id}/lich-su` (đọc mục lục + trong phạm vi phông) · `POST` (`catalog.update`) — lịch sử đơn vị hình thành phông / lịch sử phông có phiên bản, số phiên bản = max + 1 dưới khoá tư vấn (27/11/2026) |
| `danh_muc_ho_so` | `/danh-muc-ho-so` GET/POST, `/hieu-luc?phongId&nam`, `/{id}`, `/{id}/dong[/{dongId}]` POST/PUT/DELETE, `/dong/nhap`, `/trinh`, `/duyet` (không tự duyệt), `/tra-lai`, `/ban-hanh`, `/phien-ban-moi`, `/doi-chieu`, `/bien-ban` — danh mục hồ sơ dự kiến Điều 19.4 |
| còn lại | `records`, `stats`, `changes`, `notifications`, `/healthz` |

Ba mẫu hình lặp lại đáng để ý:

- **Quy trình duyệt nhiều bước** giống nhau ở `access`, `info`, `disposal`,
  `submissions`: tạo → duyệt/từ chối → (thu phí) → thực hiện → tải về.
- **Thao tác hàng loạt**: `ocr-all`, `make-pdf2-all`, `extract-fields-all`,
  `rebuild-non-pdfa`, `review-bulk`, `sweep`.
- **`/changes` + `/notifications`** phục vụ đồng bộ tăng dần cho client.
- **Lưu trữ hiện hành (Điều 19, migration 0031, 26/11/2026)** — bảng `lt_nop_luu` / `lt_nop_luu_ho_so` / `lt_danh_muc` /
  `lt_danh_muc_dong` (tiền tố `lt_`: chỉ trực tuyến, không vào bộ máy generic `cl_*`); `cl_ho_so` thêm cột máy chủ sinh
  `nop_luu_trang_thai | nop_luu_id | khoa_luc` (`is_derived`) và `dm_dong_id` (`online_only`). **Khoá sửa** cắm ở
  `digitext-db::chinh_ly::sync::apply_inner` (hồ sơ `cho_tiep_nhan | da_nop` → mọi op trên `cl_ho_so` cột người dùng,
  `cl_van_ban`, `cl_van_ban_file` → `Rejected{locked}`); desktop kiểm trước ở `chinhly/store.rs::khoa_nop_luu`. **Phạm vi
  phòng ban** `ho_so::PhamVi` (`routes/chinh_ly/mod.rs::pham_vi_cua`): người có `users.phong_ban_id` chỉ thấy hồ sơ phòng
  mình + hồ sơ mình lập (+360 ngày sau khi nộp) ở `/cl/ho-so`, `/tra-cuu`, và cả `/cl/sync` pull (bản sao offline chỉ còn
  phần của phòng). Hạn nộp 60 ngày lịch từ `cl_date_end_of(ngay_ket_thuc)` (`digitext-core::luu_tru`); worker
  `package_worker::nhac_nop_luu` mỗi giờ (`DIGITEXT_LT_NHAC_GIAY`), nhắc 15/7/1 ngày + quá hạn theo tuần cho người lập và cả
  phòng, từ năm `DIGITEXT_LT_NHAC_TU_NAM` (mặc định năm−1), khử lặp bằng `notifications.ref`. Vai trò mặc định bản Lưu trữ
  `roles::ensure_luu_tru` (`nhan_vien_phong_ban`, `can_bo_luu_tru`, `lanh_dao`). Test tích hợp `chinh_ly/tests_luu_tru.rs`
  (cần `DIGITEXT_TEST_DATABASE_URL`). Kịch bản API thử trọn luồng: `dev/luutru/thu-nop-luu.ps1 -MatKhauDb <mk>` (Postgres tạm + API debug, máy dev).
- **Bảo quản đủ Điều 29–31 (migration 0032, 26/11/2026)** — `bk_ke_hoach` (kế hoạch kiểm tra theo dự án: du_thao → cho_duyet →
  da_duyet [chốt ảnh chụp tệp vào `bk_ke_hoach_tep`] → dang_thuc_hien → hoan_thanh), `bk_loi` (một dòng ĐANG MỞ cho mỗi tệp / gói,
  mở từ MỌI lần kiểm lỗi qua `bao_quan::ghi_loi_tu_kiem` — gọi trong `ops::f_26a`, `ops::ghi_kiem_goi`, `kh_ghi_ket_qua`; khắc phục →
  kiểm lại đạt / lỗi), `integrity_checks.nguon` (tu_dong | ke_hoach | tay | kiem_lai) + `ke_hoach_id`; toàn máy chủ: `bk_lich_sao_luu`
  (3 dòng seed khớp sao-luu.ps1/.sh), `backup_runs` thêm lich_id / phương tiện / đường dẫn / dung lượng / mã băm / `da_xoa_at`,
  `bk_phuong_tien` (hạn độ bền), `bk_su_co` (gắn `backup_run_id`). Toán hạn / trễ / bản cần xoá / sắp hết độ bền thuần ở
  `digitext-core/src/bao_quan.rs` (có test). Worker `crates/digitext-ocr/src/bao_quan_worker.rs` (cả hai chế độ, chạy ngay lượt đầu,
  `DIGITEXT_BQ_GIAY` rút chu kỳ): `thuc_hien_ke_hoach` (1 h, lô 200), `nhac_lich_sao_luu` (thay cảnh báo 26 h cứng của `f_2b5`; nhắc
  trễ theo ngày, "chưa có bản" theo ngày / tháng, bản cũ cần xoá), `nhac_phuong_tien` (24 h, mỗi tháng một lần). Mẫu 01–04 trong
  `bienban.rs` thêm trường `#[serde(default)]` (kế hoạch / kỳ / phương pháp / thời gian, tệp lỗi có mã hồ sơ + khắc phục; nhật ký
  sao lưu in bảng thật có dung lượng / số hiệu PT / nơi bảo quản; sao lưu có thời gian / dung lượng / mã băm / hạn lưu; sự cố có mã /
  mức độ / phạm vi / dữ liệu mất / tệp kiểm sau phục hồi). Biên bản lập từ desktop (sổ biên bản, điền sẵn) rồi gắn lại qua
  `/bien-ban {recordId}`. **0033** quét trùng Điều 48.2: `disposal::f_trung_blake3` (cùng dự án, cùng blake3) và `f_trung_muc_luc`
  (hai hồ sơ cùng phông + `khong_dau(tieu_de)` + số ký hiệu + năm → tài liệu của hồ sơ lập sau, `dup_ho_so_id`), cột
  `disposal_entries.phat_hien`; chỉ `xem_xet`, không đánh `cl_ho_so.trung_thua`. Kịch bản thử: `dev/luutru/thu-bao-quan.ps1 -MatKhauDb <mk>`.
- **Đợt 3 đối chiếu Digifile (27/11/2026, migration 0034–0036)** — (a) **Báo cáo TT05** `digitext-db/src/bao_cao.rs` (19 hàm, mỗi
  báo cáo một câu `to_jsonb`, bộ lọc `BoLoc{phong, năm kết thúc, từ – đến}`), định nghĩa cột / nhóm / bản ở `routes/bao_cao.rs::DEFS`
  (một nơi duy nhất: desktop `BaoCaoView` dựng bộ lọc + bảng từ JSON danh mục). `luot_khai_thac` đọc `audit_log khai_thac.*` theo
  `detail->>'projectId'` → chỉ mục 0036. (b) **Mượn – trả có duyệt**: máy trạng thái thuần `digitext-core::chinh_ly::muon_tra`
  (cho_duyet → da_duyet \| tu_choi \| huy; da_duyet → dang_muon \| huy; dang_muon → da_tra; gia hạn ≤ 2 lần; có test), ghi ở
  `digitext-db::chinh_ly::muon_tra` (SQL thẳng vì cột duyệt / giao / gia hạn là `is_derived`), đường generic `apply_op` bị chặn bởi
  `sync::muon_tra_chan` (offline không đổi trạng thái; tạo thẳng `dang_muon` cần `loan.approve`); quyền mới `loan.approve` (ALL = 72);
  nhắc sắp hạn 3 ngày / quá hạn mỗi tuần trong API `cli.rs::nhac_muon_tra_kho` (mỗi giờ, `DIGITEXT_MT_NHAC_GIAY` để thử) cho người lập +
  quản trị dự án, khử lặp `notifications.ref`. Bẫy đã vá: `validate::normalize` ép `trang_thai` thành số cho mọi bảng (cl_thbq) → phiếu
  mượn (chữ) bị từ chối; nay bỏ qua với `cl_muon_tra`. (c) **Kho / giá sức chứa**: `cl_gia.suc_chua_hop / canh_bao_pct`, `cl_kho.suc_chua_hop`
  (vào `spec.cols`), `digitext-db::chinh_ly::kho_gia::tong_hop` gom `cl_ho_so_v.gia_eff_id` một lượt theo dự án (không view),
  `dat_phong_ban` + `boxing` cho hồ sơ nhập gói, nhắc `kho_day` mỗi tuần. Kịch bản thử trọn ba phần: `dev/luutru/thu-dot-3.ps1 -MatKhauDb <mk>`.
- **Đợt 4 đối chiếu Digifile (27/11/2026, migration 0037–0038)** — (a) **Chuyển đổi tài liệu số ra giấy** (Điều 13–16, PL VI):
  `digitext-pdf/src/ghep.rs` (`noi_truoc` nối hai PDF: đánh số lại đối tượng, một cây Pages phẳng, kéo Resources / MediaBox / Rotate kế
  thừa xuống trang, bỏ AcroForm / Outlines; `chan_trang` in dòng nhỏ đáy mỗi trang cùng bộ hàm với watermark), `digitext-db::chuyen_doi`
  (số theo dự án / năm với khoá tư vấn), `routes/chuyen_doi.rs`; trang thể thức dựng bằng `manual::render_manual` và đặt SAU nội dung
  (Mục 1 PL VI). Không ký số lại bản ghép; chữ ký của bản gốc kiểm bằng `digitext_sign_verify::nhieu::{kiem_tat_ca, ket_luan_tep}` và
  ghi kết luận lên trang thể thức. Báo cáo #19 `chuyen_doi` thêm vào `DEFS`. (b) **Phân quyền theo phông** `user_phong` (0038):
  `routes/chinh_ly/mod.rs::{pham_vi_phong, kiem_phong, phong_hieu_dung}` — không có dòng = không giới hạn; cắm vào `ho_so::Filter.phong_ids`,
  `VanBanFilter`, `lich_su::Filter`, `sync::pull` (bản sao offline chỉ còn phông được cấp — thay vì 403), `/cl/catalog` (lọc JSON: khoá
  `phong`, `phong_ban`, `mang`, `hop`, `bo_tap`, `gia` — KHÔNG có tiền tố `cl_`), `/tra-cuu` + `bo-loc` + `tai-lieu/{id}/doc`, mượn trả
  (list, lập, từng bước), kho / giá, báo cáo (`BoLoc.phong_ids`; báo cáo không có chiều phông vẫn là số toàn dự án), thống kê, hệ thống
  hoá, nhập mục lục, đối soát, chuyển đổi. Chưa áp: `/cl/rows/{table}/{id}` đọc một dòng, `access` / `info` (tài liệu theo dự án).
  (c) **Biểu mẫu PDF tại máy trạm**: `ManualBlock` có `serde` (tag `t`, content `v`) + `PageBreak` + `Centered`; Tauri `in_pdf::pdf_manual`
  nhận mảng khối JSON, dựng bằng font sans hệ thống, hộp thoại Lưu. Kịch bản thử: `dev/luutru/thu-dot-4.ps1 -MatKhauDb <mk>`.
- **Đợt 5 đối chiếu Digifile (27/11/2026, migration 0039)** — dọn phần còn sót trong phạm vi đã chốt. (a) **Bịt phạm vi phông**:
  `digitext-db::user_phong::{phong_cua_dong, phong_cua_tai_lieu, tai_lieu_trong_pham_vi}` suy phông của một dòng `cl_*` (theo
  `phong_id` / phòng ban / hồ sơ / văn bản) hoặc của tài liệu (qua `cl_van_ban_file`); `routes/chinh_ly/mod.rs::kiem_phong_dong` cắm vào
  `/cl/rows/{table}[/{id}]` GET / POST / PUT (kiểm cả dòng đang có lẫn dòng sắp ghi — không "dời" hồ sơ sang phông khác) / DELETE / restore;
  `POST /cl/sync/push` từ chối tại chỗ op ngoài phông (`Rejected{reason: "phong_scope"}`, không áp); `/access` (lập, nạp, danh sách — yêu
  cầu có tài liệu ngoài phông ẩn / 403; tài liệu chưa nối mục lục không ràng), `/info/{id}/generate` tìm theo chủ đề chỉ trong tài liệu
  đã nối mục lục thuộc phông được cấp, `/nop-luu/*` (hồ sơ cần nộp, phiếu, sổ, tổng quan), `/danh-muc-ho-so` (list, hiệu lực, tạo, nạp).
  (b) **Cấu hình cơ quan lưu trữ** `co_quan_luu_tru` (Digifile B01): `digitext-db::co_quan::{get, hieu_dung, dat, sql_han_nop,
  sql_quyen_su_dung}`; hạn nộp / quyền sử dụng trong SQL đổi thành `COALESCE((SELECT … FROM co_quan_luu_tru …), $n)` giữ nguyên bind
  cũ làm mặc định (nop_luu, ho_so::Filter, tra_cuu); worker `quet_nhac` dùng `luu_tru::{tinh_trang_han_voi, moc_nhac_voi}` theo mốc từng
  dự án; `bao_cao` PDF (đầu văn bản có cấp trên, khối ký = chức vụ + người ký cấu hình) và `chuyen_doi` (tên cơ quan, chức vụ mặc định,
  người ký) đọc `hieu_dung`. Mẫu sinh mã (B06) KHÔNG đổi: mã hồ sơ vẫn `ma_du_an.phong_so.nam.so[.muc_luc]` ở 3 nơi (view SQL,
  `he_thong_hoa`, desktop `store.rs`) — đổi phải đổi cả ba + dữ liệu cũ, chờ anh chốt. (c) **Lịch sử ĐVHTP có phiên bản** `phong_lich_su`
  (B02): `digitext-db::phong_lich_su`, route `/cl/phong/{id}/lich-su`; chưa đưa vào EAD (`bioghist`) vì gói dựng ở máy trạm từ SQLite.
  (d) **Nhãn hộp 2×2 PDF**: `ManualBlock::Labels2x2(Vec<Label{top, big, bottom}>)` 4 ô một trang A4, viền cắt, test `nhan_hop_2x2_hai_trang`.
  (e) **Nợ Digifile**: `digitext-extimport` nhận tiêu đề cột "Cơ quan ban hành" / "Tác giả" cho `doc_organ`, ô ngôn ngữ `01;02` tách thành
  nhiều mã (`pack::tach_ngon_ngu`); `digitext-token-signpdf` báo ngay "slot không có token cắm" trước khi mở phiên (E2E 11:241 chờ ~40 s;
  tắt kiểm bằng `DIGITEXT_SIGN_KHONG_KIEM_TOKEN=1`). Kịch bản thử: `dev/luutru/thu-dot-5.ps1 -MatKhauDb <mk>` (tự `migrate`).
- **Dashboard theo ngày** (`routes/dashboard.rs`, migration `0021_dashboard_daily.sql`): bảng `dashboard_daily`
  (ngày VN × dự án × người; dòng tổng dự án `user_id = uuid 0`) do hàm SQL `dashboard_rollup(từ, đến)` tính;
  API tự roll hôm qua + hôm nay mỗi 2 phút và 35 ngày lúc 00:15 (`spawn_rollup` gọi từ `cli.rs`), phát topic
  `dashboard`. Toán sức khoẻ dự án (RAG, dự kiến xong, bước nghẽn) ở `digitext-core::dashboard` — desktop không tự
  tính. Máy mới: áp 0021 rồi backfill theo khối 30 ngày (lệnh trong header migration).

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

**0.25 "ít chữ" (19/11/2026):** quy ước giao diện ở `docs/GIAO-DIEN.md` — mô tả màn ≤ 70 ký tự, giải thích vào nút (i)
(`PageHeader info`, `ui/InfoButton`), `ui/CanDuAn`, `ui/FilterBar` + `ui/MultiSelect`, `Button count`, `Field hintAs`; logo chữ
`ui/Wordmark` (phông `--font-display` Russo One đóng kèm ở `src/assets/fonts/`); menu trái một dòng (số bước trong nút tròn),
thanh trạng thái chỉ hiện chữ khi có vấn đề. Đợt 1/4 xong.

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
  `describeError` từ `lib/errors.ts`. **Biểu đồ** `ui/charts/` (22/10/2026, SVG tự viết, không thư viện):
  ChartFrame (đo bề rộng, legend ≥ 2 chuỗi, "Xem bảng", ←/→), Sparkline, BarChart, LineChart (≤ 4 chuỗi),
  StackedBar, Ring; màu qua token `--color-chart-*` (phân loại thứ tự cố định đã kiểm CVD, tuần tự = dải brand;
  không dùng màu trạng thái cho chuỗi số liệu).
- `styles/theme.css` — nguồn token duy nhất (màu trạng thái 50–800, thang chữ
  xs 12 / sm 13 / base 14). **0.19 (23/10/2026) đồng bộ DO365 / MS Teams:** phông Segoe UI
  (không gói font), brand = tím DOFFICE #5A58A6, gray = xám Fluent, trạng thái theo Teams,
  `run-*` cam #E8772E chỉ cho "đang chạy", chart-1..4 đã kiểm CVD — giá trị lấy từ tệp
  "DO365 Shared Design Tokens v1.0" (do365-tokens.css dùng chung hệ sinh thái DOFFICE; bản
  app ở `D:\DO365-MASTER_EXECUTIONrontend\web\src\styles.css`). Nút/huy hiệu/thẻ theo
  `.do-btn-*`, `.do-badge`, `.do-card` của tệp đó. Chưa có chế độ tối. `do365-tokens.css` đóng băng;
  `digitext.css`: `--side-*` (menu trái nền trắng), `.topbar` tím chạy suốt + ghi đè token
  trong phạm vi, luật cho các màn chưa chuyển hẳn (Quét, Trình xử lý ảnh, Kho, đăng nhập).
- `api/` — 30 module, mỗi module bọc một nhóm lệnh Tauri
- `features/` — 22 nhóm màn theo vòng đời. Menu 2 cấp ở `shell/navTree.ts`. **Từ 3/11/2026 (desktop 0.20.0) menu
  xếp theo công đoạn thi công Điều 9 TT 05/2025/TT-BNV:** Tổng quan · Dữ liệu chủ (Bước 3: Danh mục, Lập hồ sơ, Hệ thống
  hoá, Mục lục và kết xuất, Đồng bộ) · Số hoá (Bước 4: Cây thư mục, Quét, Kho tài liệu, Tài liệu đã quét, Đưa lên máy
  chủ, Kho máy chủ, OCR và bóc tách, Giao việc, Việc của tôi, **Kết nối dữ liệu chủ** = `KhaiThacView` chế độ `ketnoi`
  chỉ còn Tạo văn bản từ OCR + Metadata TT05/TT02) · Ký số, đóng gói (Bước 6–7: Ký số, Đóng gói 3 tab, Thu nộp) ·
  Nghiệm thu (Bước 8: Sổ biên bản) — rồi dưới đường chia: Lưu trữ số (giữ cho bản khách hàng) · Báo cáo · **Tiện ích**
  (`view:tienich`, hub `shell/TienIchHub.tsx`, tab `V3xTienIch`: Kiểm PDF/A, **Kiểm chữ ký số**, Kiểm tra gói tin, Từ kho ngoài,
  Đóng gói nhanh, Ví dụ dạy máy, Chuyển dữ liệu cũ; cộng Tách TIFF, Kho và giá, Mượn – trả) · Quản trị. Id mục con giữ nguyên;
  `Sidebar.QUY_TRINH` định nghĩa trục; xem trước `?view=tienich&tab=…`, `?view=khogia&kg=ketnoi`. Các nhóm Giao nhận
  tài liệu (Bước 1–2), Kiểm tra sản phẩm (Bước 5) và màn Nhật ký, Nghiệm thu khối lượng sẽ thêm ở các bản sau (kế hoạch
  4 bản, migration 0024). **R2 (3/11):** nhóm Giao nhận tài liệu (`view:handover` → `records/HandoverView.tsx`: biên bản
  giao nhận / trả tài liệu giấy, điền sẵn từ mục lục chỉnh lý) và Số hoá › Nhật ký số hoá (`view:nhatky` →
  `report/NhatKyView.tsx`: dự án × người × ngày từ `/dashboard/project`, chốt ngày = biên bản `nhat_ky_so_hoa`). Biên bản
  "generic" (Điều 9): mẫu ở `records/recordForms.ts` (`generic`, `taoPayloadGeneric`), máy chủ `bienban.rs::render_generic`
  chỉ in PDF; `GET /records?kind=&ref=&projectId=`, `GET /records/{id}` (payload). **R3 (3/11):** nhóm Kiểm tra sản phẩm
  (Bước 5, `view:qc` → `qc/QcView.tsx`): `GET /cl/doi-soat?projectId&phongId[&thuMuc]` (`routes/chinh_ly/doi_soat.rs`, chỉ đọc,
  không migration) ghép mục lục `cl_ho_so_v` với tài liệu máy chủ qua `khoa(hộp, hồ sơ)` như `tu_ocr.rs`, kết luận từng hồ sơ
  (`dat | chua_quet | thieu_trang | thua_trang | khong_pdfa | pdfa_cu | bi_tu_choi`; trang < số tờ = thiếu, trang > 2×số tờ =
  nghi thừa), tệp không khớp mục lục, `thuMucCo` để lọc khi dự án nhiều phông; màn thêm bảng Quét lại (`docs_pdfa_rejected`),
  Xuất Excel, biên bản `kiem_tra_san_pham` điền sẵn. Chỉ cảnh báo, không chặn ký số/đóng gói. **R4 (3/11):** Nghiệm thu ›
  Nghiệm thu, bàn giao (`view:acceptance` → `acceptance/NghiemThuView.tsx`): khối lượng theo phông = mục lục chỉnh lý (hộp,
  hồ sơ, Σ số tờ) ⨝ kho máy chủ (`docs_review_summary` gộp theo thư mục phông, ghép tên bỏ dấu, sửa tay) so với hợp đồng
  (`project_contracts.contract_boxes/dossiers/sheets`, nhập ở Tiến độ dự án › Hợp đồng); biên bản `nghiem_thu` /
  `ban_giao_lan` điền sẵn; "Tạo bộ hồ sơ nghiệm thu" ghi vào thư mục chọn: Excel khối lượng, PDF mọi biên bản của dự án,
  MLHS TT05 từng phông (Tauri `save_export`/`record_download` nhận `dir`; `writeXlsx`/`xuatTT05` nhận thư mục đích).
  **R5 (4/11):** biên bản Word `GET /records/{id}/docx` (`digitext-api/src/docx.rs` tự viết WordprocessingML + zip, dựng lại
  từ payload mỗi lần, mẫu generic giữ bảng thật; nút Word ở sổ biên bản, Tauri `record_download` `format: "docx"`); bộ hồ sơ
  nghiệm thu có hộp tuỳ chọn: thêm `02-Bien-ban/Word`, `04-Goi-AIP` (Tauri `package_download` nhận `dir`) + `05-Bao-cao-kiem-tra-goi.pdf`
  (`pkgcheckRun` trên thư mục gói, trang "Kiểm tra gói" trong Excel), nén `<thư mục>.zip` (Tauri `bundle::zip_dir`); sổ biên bản
  lật trang `recordsAll` (máy chủ trả ≤ 200 dòng/lần). Ghép mục lục ↔ tài liệu dùng chung `tu_ocr::BoGhep`: khớp (hộp, hồ sơ)
  trước, rồi theo riêng số hồ sơ khi mục lục không kê hộp (cả Tạo văn bản từ OCR lẫn đối soát); đối soát tách `tinh` (đọc CSDL)
  + `ghep` (hàm thuần) — test `chay_that_json` (bỏ qua mặc định) chạy trên JSON xuất từ máy chủ + `kiem:mucluc` `KIEM_XUAT_JSON`.
  **Bản Lưu trữ GĐ1 (4/11, nhánh `feat/ban-luu-tru`, docs/KE-HOACH-BAN-LUU-TRU.md):** `config.ts` `LUU_TRU`
  (`VITE_EDITION=luu_tru`, DEV `?edition=luu_tru`), `navTree.ts` `NAV_LUU_TRU` + `VIEW_CUA_BAN` (AppShell chặn màn ngoài
  bản), `home/LuuTruHome.tsx`; Tauri `edition.rs` (`cfg(luu_tru)` từ `build.rs`), `update.rs` (cập nhật theo máy chủ đang
  kết nối); lớp phủ `tauri.luutru.conf.json`; API `config::edition_luu_tru()` (bỏ route thi công trong `app.rs`),
  `config::public_url()`, `routes/updates.rs` viết lại `latest.json` + `GET /tai-ve`. Phát hành: DEPLOY-DESKTOP §3b.
  **GĐ3 nhập gói AIP (4/11):** `digitext-api/src/nhap_goi.rs` (dùng chung lệnh `digitext-api nhap-goi` và `routes/ingest.rs`
  tải lên theo khúc) — `digitext_pkgcheck::check_and_extract` (EAD hồ sơ/văn bản + ghép tệp theo structMap) → zip gốc
  `{p}-packages/` (`packages.nguon='nhap'`) → `cl_phong` (mã phông trong OBJID) / `cl_ho_so` / `cl_van_ban` / `cl_van_ban_file`
  qua `apply_op` → `documents` `archived` + `goi_id` + `ky_so` → lớp chữ (lopdf) vào `ocr_results` (engine cố định
  `lop-chu-goi-nhap`) → `cl_ho_so_goi` (dấu xong, nhập lại không trùng). Migration 0025 + `ingest_batches/items`.
  `WormGuard.put` chặn ghi đè `*-packages/` khác nội dung. `bootstrap-superadmin --password-file`. Desktop
  `features/ingest/NhapGoiView.tsx` (`view:nhapgoi`, quyền submission.approve), Tauri `ingest.rs`.
  **GĐ4 tra cứu + 4 module (5/11):** migration 0026 — `khong_dau()` (unaccent, IMMUTABLE), `cl_vb_tim()` / `cl_hs_tim()`
  + chỉ mục trigram (mục lục, `ocr_results.content`), `integrity_checks.package_id`, `packages.huy_at/huy_quyet_dinh`.
  `routes/tra_cuu.rs`: `GET /tra-cuu` (mẫu `%từ1%từ2%`, khớp mục lục hoặc lớp chữ, trích đoạn), `/tra-cuu/bo-loc`,
  `/tra-cuu/tai-lieu/{id}/doc` (watermark; khách chỉ đọc thẳng tài liệu công khai khi phí đọc = 0), `/tra-cuu/nhat-ky`
  (`audit_log` `khai_thac.*` — ghi cả ở đọc/tải/DIP của `access.rs`). DIP tài liệu nhập: `nhap_goi::dip_tu_goi` dựng EAD
  từ mục lục. Cung cấp thông tin (`digitext-db/info.rs`) tìm không dấu trên mục lục + lớp chữ. Bảo quản: blake3 tài liệu
  nhập + sha256 zip gói (API `/ops/integrity/run` và worker `f_2b5`). Tiêu huỷ: `disposal::f_1ah` tính hạn từ THBQ cho
  tài liệu nhập; `execute` xoá zip qua `ObjectStore::xoa_theo_quyet_dinh` (đường DUY NHẤT qua WormGuard) khi hết tài liệu
  của gói. Desktop `features/tracuu/TraCuuView.tsx` (`view:tracuu`, quyền access.request), Tauri `tra_cuu.rs`.
  **GĐ2 bộ cài máy chủ (9/11):** `digitext-api migrate [--da-ap-den NNNN] [--chi-xem]` (`migrate.rs`, bảng
  `_digitext_migrations`) — dùng được cho cả OCR PC thay psql tay; worker `DIGITEXT_CHI_LUU_TRU=true` (config.rs, entry.rs,
  `OcrProbe::KhongCo`); gói Windows `dev/luutru/` + Linux `infra/compose/luutru/` (+ `.dockerignore` danh sách trắng ở gốc);
  `/tai-ve` phát `chung-chi-goc.crt` / `cai-chung-chi-goc.cmd`; desktop reqwest `rustls-tls-native-roots`,
  `auth/client.rs::chuoi_loi`. Hướng dẫn: `docs/CAI-DAT-MAY-CHU-LUU-TRU.md`.
  **GĐ6 Lưu trữ hiện hành Điều 19 (26/11):** nhóm menu `luutruhienhanh` chỉ ở `NAV_LUU_TRU` (Phông, phòng ban · Danh mục hồ
  sơ `view:danhmuc` · Lập hồ sơ · Nộp lưu `view:nopluu` · Tiếp nhận nộp lưu `view:tiepnhan`, badge `nopLuu`); màn ở
  `features/luutru/` (`NopLuuView`, `TiepNhanView`, `DanhMucHoSoView`, `luuTruFlow.ts`), API `api/luutru.ts` qua hai lệnh Tauri
  `lt_get` (chỉ đọc) / `lt_call` (ghi, gắn cứng `/nop-luu` + `/danh-muc-ho-so`, `src-tauri/src/luu_tru.rs`); topic `nopLuu`,
  `danhMuc`. `HoSoForm` có ô Dòng danh mục (ghi `dm_dong_id` qua `clRemoteSave`, tắt khi offline) + banner khoá; `LapHoSoView`
  cột Nộp lưu, menu "Nộp lưu … đã chọn", bộ lọc phòng ban khoá theo `session.phongBanId`; `UsersView` ô Phòng ban; session /
  `auth/store.rs` (schema SQLite 26) mang `phongBanId`, `tenPhongBan`. Cổng màn: `nopluu → luutru.submit`,
  `tiepnhan → luutru.receive`, `danhmuc → danhmuc.edit` (6 quyền mới, `Permission::ALL` = 70).
  **GĐ7 Bảo quản Điều 29–31 (26/11, 0032/0033):** `infoops/BkView.tsx` thành hub 5 tab (Tổng quan · Kế hoạch kiểm tra · Sao lưu ·
  Phương tiện · Sự cố; dev `?view=bk&tab=kehoach`), panel ở `infoops/baoquan/` (`KeHoachPanel` lập / trình / duyệt / kiểm ngay /
  kết luận Mẫu 01 + danh mục lỗi khắc phục – kiểm lại; `SaoLuuPanel` thẻ lịch đúng hạn / trễ, bản cũ cần xoá, sổ đủ cột, ghi nhận tay,
  `SoBienBan` Mẫu 02/03 điền từ sổ; `PhuongTienPanel`; `SuCoPanel` + Mẫu 04 điền sẵn; `baoQuanFlow.ts` nhãn + `ganBienBanMoiNhat`),
  `api/baoquan.ts` qua Tauri `bq_get` / `bq_call` (`src-tauri/src/bao_quan.rs`, gắn cứng `/bao-quan`, topic `ops`); `DisposalView` cột
  "Phát hiện"; `LuuTruHome` ô sao lưu trễ · lỗi mở · sự cố; quyền `bk.approve` (71 quyền). Mục menu `baoquan` giữ ở cả hai bản.
  **GĐ8 Đợt 3 (27/11, 0034–0036):** `report/BaoCaoView.tsx` (tab `report:tt05`, nhóm Báo cáo có ở CẢ `NAV_LUU_TRU`): cột trái danh
  mục báo cáo theo nhóm, bộ lọc sinh từ `boLoc`, bảng theo `cot` + dòng tổng, nút Excel / PDF tải qua Tauri `bc_download` (hộp thoại Lưu;
  `src-tauri/src/bao_cao.rs`, nhận `/bao-cao`, `/cl/muon-tra/*.pdf`, `/cl/kho-gia/ho-so`), `api/baoCao.ts`. `chinhly/MuonTraView.tsx`
  viết lại: KPI chờ duyệt / chờ giao / đang mượn / quá hạn, Seg 6 trạng thái, lập phiếu nhiều hồ sơ (`LapPhieuDialog`, ô "giao luôn" khi
  có `loan.approve`), menu hành động theo `muonTraFlow::hanhDongMuon` (duyệt / từ chối / duyệt cả phiếu / giao / gia hạn / trả / huỷ),
  phiếu PDF, sổ Excel (= báo cáo `muon_tra`), `api/muonTra.ts`. `chinhly/KhoGiaView.tsx`: tab Giá đọc `kgTongHop` (sức chứa, hộp,
  % đầy, tô gần đầy / vượt), ô sức chứa + ngưỡng cảnh báo, xuất hồ sơ theo kho / giá, tab **Gán hộp** (`GanHopPanel`: hồ sơ chưa hộp của
  phông, phòng ban chủ hộp, nhập từng dòng hoặc dán 2 cột), `api/khoGia.ts`. `LuuTruHome` thêm hàng KPI mượn trả. Quyền `loan.approve`
  (72 quyền) ở `permissions.ts`.
  **GĐ9 Đợt 4 (27/11, 0037–0038):** `tracuu/TraCuuView` nút "In ra giấy" (cán bộ) → `ChuyenDoiDialog` (mục đích, người nhận, căn cứ, chức vụ
  người ký) → tải PDF đã ghép; màn mới `luutru/SoChuyenDoiView` (`view:chuyendoi`, mục "Sổ chuyển đổi" trong nhóm Lưu trữ số của cả hai
  bản, gate `access.view`), `api/chuyenDoi.ts`. `users/UsersView` menu "Phạm vi phông" → `PhongDialog` (MultiSelect phông của dự án, trống =
  không giới hạn), `api/users.ts::userPhongGet/Set` qua Tauri `bc_call` (nhận `/chuyen-doi`, `/users/{id}/phong`). Biểu mẫu PDF tại máy trạm:
  `chinhly/printPdf.ts` (`pdfMucLucHoSo`, `pdfSanPham` hai bìa + bảng gom mảng / năm, `pdfMucLucVanBan` sang trang từng hồ sơ, `pdfPhuLuc`)
  trên `api/inPdf.ts::{kb, pdfManual}`; menu Kết xuất của `KhaiThacView` có mục "(PDF)" đứng trước, HTML giữ là "In nhanh … (HTML)".
  **GĐ10 Đợt 5 (27/11, 0039):** hộp thoại `settings/CoQuanDialog` (mở từ nút "Cơ quan lưu trữ" ở `home/LuuTruHome` — bản Lưu trữ — và cạnh
  "Định mức" ở `home/ProjectDashboard` — bản thi công; sửa cần `project.update`), `api/coQuan.ts`; `chinhly/PhongLichSuDialog` từ menu dòng
  tab Phông của `DanhMucView` ("Lịch sử phông", chỉ khi trực tuyến; thêm phiên bản cần `catalog.update`), `api/phongLichSu.ts`; nhãn hộp
  `printPdf.ts::pdfNhanHop` trên dữ liệu `print.ts::nhanHopData` dùng chung với HTML, `inPdf.ts::kb.labels`; Tauri `bc_call` kiểm quyền
  theo đường (`quyen_goi`: `/projects/{id}/co-quan` project.view / project.update, `/cl/phong/{id}/lich-su` catalog.read / catalog.update);
  `topics.ts` `bc_call` += `projects`, `chinhLy`.
  Mô tả cũ bên dưới giữ để tra cứu vị trí tệp:
  Quản trị có thêm `users/RolesView` (vai trò và bộ quyền, 22/10/2026) ·
  Tổng quan (`home`: một màn, ba góc nhìn theo vai trò — `StaffDashboard` nhân viên, `ProjectDashboard` một dự án,
  `LeaderDashboard` mọi dự án; khối dùng chung ở `home/blocks/`; định mức KPI `report/KpiSettingsDialog`;
  xem trước dev `?view=home&role=user|admin|manager&project=1`) · Số hoá (`drive`, `scan`, `tree`, `dossier`, `upload`,
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

### Schema Postgres: `migrations/0001`–`0039` (sửa 27/11/2026)

`digitext-db` dùng sqlx truy vấn 28 nhóm bảng. Thư mục `migrations/` **có** trong repo
(0001 baseline … 0022 roles, 0023 cl_van_ban nguồn OCR, 0024 records thi công TT05, 0025 nhập gói AIP, 0026 tra cứu không dấu /
kiểm gói / huỷ gói, 0027 quét virus, 0028–0029 webhook dự án, 0030 `permissions_seen` — API tự bổ sung quyền MỚI vào vai trò hệ
thống lúc khởi động, `roles::bo_sung_quyen_moi`; 0031 lưu trữ hiện hành Điều 19: `lt_*`, cột `cl_ho_so`, `users.phong_ban_id`,
`notifications.ref`, **DROP/CREATE view `cl_ho_so_v`** → áp lúc bảo trì; 0032 bảo quản Điều 29–31: `bk_*`, cột `integrity_checks` /
`backup_runs`; 0033 quét trùng: `disposal_entries.phat_hien`, chỉ mục blake3 + mục lục, cần 0026 trước; 0034 mượn – trả có duyệt: CHECK
`cl_muon_tra.trang_thai` 6 giá trị + 12 cột duyệt / giao / gia hạn / nhắc; 0035 `cl_gia.suc_chua_hop / canh_bao_pct`, `cl_kho.suc_chua_hop`;
0036 chỉ mục `audit_log` theo dự án cho báo cáo lượt khai thác, CONCURRENTLY; 0037 `chuyen_doi_giay` sổ chuyển đổi ra giấy; 0038 `user_phong`
phạm vi phông; 0039 `co_quan_luu_tru` cấu hình cơ quan lưu trữ + `phong_lich_su` lịch sử phông có phiên bản) — ghi chú cũ "không có migrations" là sai. Không có cache `.sqlx/` offline.
Áp bằng `digitext-api migrate` (bảng `_digitext_migrations`, 9/11/2026; máy chủ đã áp tay thì `migrate --da-ap-den <NNNN>` một lần)
hoặc tay theo thứ tự bằng `psql -f` (xem `docs/CHUYEN-MAY-OCR-PC.md`).

### Truy vấn không được kiểm tại thời điểm biên dịch

Không có macro `query!`/`query_as!` nào — toàn bộ dùng API runtime. Sai tên cột
sẽ nổ lúc chạy chứ không phải lúc `cargo build`. Sửa DB phải test thật.

### Quyền: vai trò gốc trong code, bộ quyền trong CSDL (22/10/2026)

- 5 vai trò gốc (`digitext-core::Role`) quyết định tầng `Perm` và phạm vi dự án; bộ quyền của từng vai trò (kể cả
  5 vai trò hệ thống) nằm ở bảng `roles` + `role_permissions`, sửa ở Quản trị › Vai trò và quyền. `users.role_code`
  trỏ tới vai trò tuỳ chỉnh (NULL = vai trò gốc). Quyền hiệu lực = `effective_with(preset, role, overrides, p)` —
  bộ quyền không vượt tầng của vai trò gốc (`allowed_for_base`). API tự chèn vai trò hệ thống khi khởi động
  (`roles::ensure_system`); `Ts` nạp bộ quyền qua cache 20 s (`routes/roles.rs::preset_cached`). Đăng nhập / lease trả
  thẳng danh sách quyền hiệu lực nên desktop không tự tính nữa (máy chủ cũ thì vẫn tự tính).

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
| `package/PackFromServer` | 385 | Đóng gói › Từ hồ sơ (26/10/2026): phông/hộp/hồ sơ từ `docs_review_summary` của dự án đang chọn — mỗi thư mục đẩy lên là một phông; thay SoHoaPane(donggoi) vốn đọc bảng `phong` cục bộ không phân dự án |
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
