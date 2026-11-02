# Chỉnh lý tài liệu giấy — mô hình lai máy chủ / máy trạm (30/9/2026)

Nghiệp vụ chỉnh lý (danh mục → lập hồ sơ tạm → biên mục văn bản → hệ thống hoá → in mục lục / nhãn hộp /
sản phẩm / phụ lục → kho, giá → khai thác → mượn trả → thống kê) được đưa từ `D:\Digitext.2025` (ASP.NET) vào
DigiText. **Máy chủ Postgres là nguồn sự thật; mỗi desktop giữ bản sao SQLite, sửa offline rồi đồng bộ.**

## Ở đâu

| Lớp | Vị trí |
|---|---|
| Logic thuần dùng chung (parse ngày, mã hồ sơ, trạng thái THBQ, 6 thứ tự hệ thống hoá, bảng mã TT05, số → chữ, mô tả 15 bảng) | `crates/digitext-core/src/chinh_ly.rs`, `chinh_ly/spec.rs` |
| Lược đồ máy chủ | `migrations/0012_chinh_ly.sql` (17 bảng `cl_*` + view `cl_ho_so_v`, seed 14 THBQ / 11 ngôn ngữ / 32 loại VB) |
| Tầng DB máy chủ | `crates/digitext-db/src/chinh_ly/` — `rows` (đọc/ghi dòng JSON), `validate`, `merge` (gộp 3 chiều), `sync` (`apply_op`, `pull`, `prune`), `lich_su`, `ho_so`, `danh_muc`, `he_thong_hoa`, `thong_ke` |
| HTTP | `crates/digitext-api/src/routes/chinh_ly/` — `/cl/*` (xem đầu `mod.rs`) |
| Desktop Rust | `desktop/src-tauri/src/chinhly/` — `store` (mirror SQLite), `sync` (engine nền), `commands` (`cl_*`, proxy `cl_remote`) |
| Desktop UI | `desktop/src/features/chinhly/` — Danh mục, Lập hồ sơ, Văn bản, Hệ thống hoá, Khai thác (+ in/xuất), Kho/Giá, Mượn trả, Thống kê, Đồng bộ & xung đột; `desktop/src/api/chinhly.ts` |

## Dòng dữ liệu = JSON snake_case đúng tên cột `cl_*`

Mọi API và lệnh Tauri trả nguyên dòng bảng (`to_jsonb`) cộng trường làm giàu (`ten_phong`, `ten_thbq`,
`thbq_status`, `ma_ho_so`, `hop_so`, `ten_gia`, `so_van_ban`…). Cột hệ thống: `id` (uuid, client sinh được),
`project_id`, `ver` (chuỗi toàn cục `cl_ver_seq`, trigger gán khi INSERT/UPDATE), `created_*`, `updated_*`,
`deleted_at` (xoá mềm = tombstone).

## Dự án đang làm (`desktop/src-tauri/src/chinhly/project.rs`)

Mọi dữ liệu `cl_*` thuộc một dự án. Tài khoản thường lấy dự án từ phiên đăng nhập; tài khoản **super_admin không gắn
dự án** nên phải chọn "Dự án đang làm" (thanh ở đầu mỗi màn Chỉnh lý, lệnh `cl_project_get` / `cl_project_set`, lưu
`settings.cl_project_id`). Chưa chọn → mọi lệnh `cl_*` trả `no_project`, engine đồng bộ báo "Chưa chọn dự án" và
không kéo gì (kể cả bảng mã THBQ / ngôn ngữ / loại VB). Đổi dự án → engine kéo dữ liệu dự án mới (bản sao giữ theo
`project_id`, con trỏ đồng bộ riêng từng dự án).

## Bảng nào sửa được offline

| Sửa offline (push) | Chỉ online (REST) | Chỉ đọc trên máy |
|---|---|---|
| khối phông, phông, phòng ban, mảng, THBQ/ngôn ngữ/loại VB (dòng của dự án), hồ sơ, văn bản, tệp văn bản | hộp, bó/tập, kho, giá, mượn trả; cột `ho_so_so`, `thu_tu_sap_xep`, `hop_id`, `bo_id`, `tap_id`, `gia_id` của hồ sơ | lịch sử (máy chủ sinh) |

Lý do: đánh số 1..n, dải từ/đến hồ sơ của hộp, "một phiếu mượn mở / hồ sơ" là bất biến trên *tập*; hai máy offline
cùng làm sẽ không gộp được.

## Đồng bộ

- **Pull** `GET /cl/sync?projectId&since=<ver>&limit=2000` → `{since, next, hasMore, full, rows:[{table, ver, row}]}`.
  `since=0` bỏ tombstone; `full=true` khi `since` cũ hơn mốc dọn (`cl_sync_meta.pruned_ver`, tombstone xoá cứng
  sau 90 ngày) → máy trạm xoá bản sao của dự án và kéo lại. Pull lấy khoá tư vấn chia sẻ, ghi lấy khoá độc quyền →
  không có "lỗ" ver.
- **Push** `POST /cl/sync/push` `{projectId, deviceId?, ops:[{opId, table, id, op: upsert|delete, base, new, editedAt}]}`.
  `base` = dòng lúc kéo về (null = tạo offline); `new` = các cột client ghi. Máy chủ sắp cha → con, upsert trước
  delete, **mỗi op một giao dịch**, idempotent theo `opId` (`cl_sync_ops`). Kết quả từng op:
  `applied | merged(mergedFields) | no_change | conflict(row, conflictFields) | rejected(reason, message, errors[]) | duplicate`.
  `reason`: `forbidden, unknown_table, unknown_field, online_only_table, online_only_field, validation, unique:<ràng buộc>,
  fk:<cột>, missing, deleted_on_server, loan_open, in_use`.
- **Gộp 3 chiều** (`merge::three_way`): trường client không đổi → giữ máy chủ; máy chủ không đổi → lấy client; hai
  bên cùng giá trị → không xung đột; khác → `conflict`, người dùng chọn ở màn *Đồng bộ & xung đột* (theo trường).
- **Đường online dùng chung mã**: `PUT /cl/rows/{table}/{id}` `{projectId, ifVer | base, new}` = `apply_op`; lệch
  `ifVer` → 409 kèm dòng máy chủ.
- **Lịch sử** (`cl_lich_su`): máy chủ tính diff từng trường khi áp op (online, sync, import) → `diffs [{f,o,n}]`,
  `n_new`, `n_edited` — nguồn cho *Tổng kết nhân viên*.
- Máy trạm: `cl_*` mirror lưu `row_json` + cột sinh `json_extract` có chỉ mục; `dirty` (0 sạch / 1 sửa / 2 tạo),
  `base_json`; `cl_sync_outbox` gộp nhiều lần sửa thành 1 op (giữ base lần đầu); engine chạy khi đăng nhập, mỗi 60 s,
  và 1,5 s sau mỗi lần ghi; sự kiện `chinhly://sync`; ô "Chỉnh lý" ở thanh trạng thái.

## Cầu nối với bảng SQLite cũ trên máy (`desktop/src-tauri/src/chinhly/legacy.rs`)

Các màn Số hoá (Tạo lập CSDL, Đóng gói, lọc OCR theo phòng ban / THBQ, ghép PDF quét) vẫn đọc bảng cũ
`dossier` / `van_ban` / `phong_ban` / `mang_cong_viec` / `thoi_han_bq` / `kho` / `gia`. Mỗi bảng cũ có thêm cột `cl_id`:

- **Chiếu** cl_* → bảng cũ (`project_row`, `project_rows`, `prune_legacy`): chạy sau mỗi trang kéo về, sau mỗi kết quả
  đẩy, và ngay sau `cl_save` / `cl_delete`. `pdf_path` của văn bản là dữ liệu máy, không bị ghi đè.
- **Chuyển tiếp** bảng cũ → cl_* (`forward_dossier`, `forward_van_ban`, `push_unmigrated`, `reconcile_deleted`): mọi
  lệnh ghi cũ (`dossier_*`, `van_ban_*`, nhập Excel, đồng bộ từ Kho quét) sau khi ghi bảng cũ thì đẩy sang cl_* → outbox
  → máy chủ. `write_local` bỏ qua khi không có gì đổi nên chạy lại không sinh op thừa.
- **Wizard**: màn *Đồng bộ & xung đột* → "Đưa dữ liệu cũ lên máy chủ" = `push_unmigrated` cho mọi dòng chưa có `cl_id`
  (phông theo `ma_du_an`, THBQ khớp tên với bảng mã, ngôn ngữ / loại VB đổi tên → mã TT05). Chạy lại an toàn.

## Quy tắc nghiệp vụ (đã sửa lỗi bản cũ)

- **Hai loại bảo quản** (migration 0015, 5/10/2026): vĩnh viễn (`cl_thbq.trang_thai = 1`) và có thời hạn (còn lại) đánh
  số riêng — mục lục thật mỗi loại đánh lại hộp 01 / hồ sơ 1. `cl_ho_so.vinh_vien` do trigger `cl_ho_so_loai` đặt theo
  THBQ (bản đẩy lên có cột này thì bỏ qua — `spec::is_derived`); `cl_hop.vinh_vien` là danh tính của hộp.
- Số tạm duy nhất trong (phông, loại) (không phân biệt hoa/thường) ở cả 3 đường; (hồ sơ, STT văn bản) duy nhất; số hộp
  duy nhất trong (phòng ban, loại); hồ sơ không được nằm trong hộp khác loại (trigger báo `ck_cl_ho_so_hop_loai`);
  bó/tập là số nguyên dương. Mã hồ sơ đuôi mục lục theo loại: `cl_phong.muc_luc_so` (vĩnh viễn) /
  `muc_luc_so_cth` (có thời hạn, trống thì dùng `muc_luc_so`). Giá theo dải hộp có `cl_gia.vinh_vien` (NULL = cả hai).
  Hệ thống hoá đánh số riêng từng loại, mỗi loại từ "Bắt đầu từ số".
- Ngày nhận `d/M/yyyy`, `dd-MM-yyyy`, `d.M.yyyy`, `M/yyyy`, `yyyy`, ISO, serial Excel → chuẩn `dd/MM/yyyy | MM/yyyy | yyyy`;
  kiểm ngày thật (31/02 lỗi), năm 1800..2100, bắt đầu ≤ kết thúc.
- Trạng thái THBQ tính động: loại 1 → VV; loại 2: `(nămNay − nămKT + 1) > giá trị` → HET, không thì GL; 3, 4 → GL.
- Mã hồ sơ `{MaDuAn}.{PhongSo}.{nămKT}.{ho_so_so}[.{MucLucSo}]` (bỏ dấu chấm kép khi thiếu); mã văn bản TT05 = mã HS +
  STT 7 chữ số; TT05 cột THBQ ghi mã 01–07 (không ghi số năm).
- Xoá mềm + lan xuống văn bản / tệp; khôi phục cả cây; chặn xoá danh mục còn tham chiếu; chặn xoá hồ sơ đang mượn.
- Nhập Excel không nuốt lỗi: lỗi trả theo dòng, dòng hợp lệ vẫn ghi.

## Mẫu Excel / in (giữ bố cục bản cũ)

`desktop/src/features/chinhly/excel.ts` (mục lục tạm A–P + P1 = fileType; mẫu lập HS: A1 mã phông, dữ liệu từ dòng 5;
mẫu MLVB: dòng 1 "ID Hồ sơ: …", 16 cột), `excelXuat.ts` (mục lục hoàn thiện 6 kiểu có dòng nhóm, MLVB mỗi hồ sơ
một sheet, phụ lục, TT02 32 cột, TT05 51 cột, lịch sử), `print.ts` (MLHS, MLVB, nhãn hộp A4 2×2, sản phẩm có 2 bìa +
"Viết bằng chữ", phụ lục hướng dẫn tra tìm).

## Nhập mục lục hồ sơ (1/10/2026)

Màn Lập hồ sơ › "…" › **Nhập mục lục hồ sơ**: tệp mục lục 7 cột `Hộp số · Hồ sơ số · Tiêu đề hồ sơ · Thời gian BĐ - KT ·
Số tờ · THBQ · Ghi chú` (một tệp / THBQ của phông), dòng nhóm nằm trong cột Tiêu đề. Phân tích chạy trên máy
(`desktop/src/features/chinhly/mucLucImport.ts`, `thoiGianHoSo.ts`), hộp thoại 3 bước `NhapMucLucDialog.tsx`.

- **Dòng nhóm**: `I.`/`II.`… = phòng ban, `1.`/`2.`… = mảng (thuộc phòng ban hiện tại). Dòng nhóm khác (không có
  `I.`/`1.`, vd. "Sổ chứng thực") = mục nhỏ trong mảng: bỏ qua, không báo lỗi, hồ sơ dưới nó vẫn thuộc mảng hiện tại
  (5/10/2026; trước đó là lỗi chặn nhập). Mã phòng ban / mảng = số thứ tự 2 chữ số (`I.` → `01`, `1.` → `01`), tên = phần sau dấu chấm.
- **Số tạm = Hồ sơ số** trong tệp (như DigiText 2025, 5/10/2026 — thay cho tiền tố `VV.001` cũ); **Hồ sơ số** và **Hộp số**
  ghi luôn vào hồ sơ (Hộp số chỉ ghi ở dòng đầu hộp → mang xuống). Máy chủ upsert theo (phông, loại, số tạm) rồi xếp hộp
  bằng `he_thong_hoa::boxing` (cần quyền Hệ thống hoá; thiếu quyền thì bỏ qua Hồ sơ số / Hộp và ghi `notes`). Lỗi chặn:
  hồ sơ số trùng trong cùng loại, hộp lẫn cả hai loại, thiếu Hộp số khi tệp có cột Hộp số.
- **Thời gian** (chuyển từ hai công cụ CheckThoiGian / DienThoiGian): tách mốc theo gạch / xuống dòng; điền `2020` → cả năm,
  `04/2020` → cả tháng, một ngày → lặp lại, mốc sai → giữ mốc đúng + suy từ năm trong ô, ô trống → năm trong tiêu đề
  (cụm năm sau chữ "năm", không có thì cụm cuối; bỏ "nhiệm kỳ"); đầu > cuối → đảo. Kiểm: sai định dạng, năm ngoài dải năm
  tiêu đề, tiêu đề không có năm → cảnh báo (vẫn nhập); không suy được năm → ngày để trống.
- **Hộp số** và **Hồ sơ số** gửi lên máy chủ; có quyền Hệ thống hoá thì máy chủ xếp hộp ngay (`hth::boxing`, hộp theo
  (phòng ban, loại, số hộp)); không có quyền thì bỏ qua và ghi `notes`. (Trước 2/11/2026 tài liệu ghi "không nhập" — sai.)
- **2/11/2026 — mục lục thật của Bắc Giang** (23 tệp ở `test/MLHS`, 10.171 hồ sơ, kiểm bằng `npm run kiem:mucluc -- <thư mục> [THBQ mặc định]`
  trong `desktop/`): tên cột so sau khi bỏ dấu + bỏ mọi khoảng trắng (`chuanTenCot`), bí danh ngắn so bằng đúng (`BĐ`, `KT`,
  `SỐ`, `Năm`); thời gian không bắt buộc có cột (suy từ năm trong tiêu đề); tự chọn sheet có dòng tiêu đề (sheet 1 thường
  là bìa), đổi sheet trong hộp thoại; tệp không có dòng `I.` → phòng ban "Chung" (00); số tờ không phải số / thiếu số tờ /
  hộp lẫn hai loại / chưa có hộp số → **cảnh báo** (vẫn nhập; số tờ gốc ghi vào Ghi chú); ô THBQ trống → "THBQ khi ô trống"
  chọn cho cả tệp. Vẫn chặn: hồ sơ số trùng cùng loại, thiếu tiêu đề, thời gian không suy được (quyết định 12/10).
  Kết quả: 16/23 tệp nhập ngay; 7 tệp còn 96 dòng phải sửa tay (92 thiếu thời gian, 3 trùng số, 1 THBQ "Xóa").
- **Nhập**: `POST /cl/ho-so/import` với `createMissing: true` (+ `ten_phong_ban`, `ten_mang`): máy chủ khớp phòng ban / mảng
  **chỉ theo tên** (mã I/II…, 1/2… là số thứ tự riêng của từng tệp — hai tệp cùng phông có thể cùng ghi "II." cho hai
  phòng ban khác nhau), chưa có thì tạo qua `apply_op` (`so_tt` = mã) rồi dùng cho các dòng sau; mã đã bị mục khác
  dùng → lấy mã số lớn nhất + 1 và trả ghi chú trong `notes`. Trả thêm `createdPhongBan`, `createdMang`. Mẫu 14 cột cũ không đổi (`createMissing` mặc định false). Sau nhập desktop gọi
  `cl_sync_now` để kéo danh mục / hồ sơ mới về bản sao.
- **Tệp đã chuyển**: sheet 1 đúng mẫu lập hồ sơ (đọc lại được bằng "Nhập hồ sơ từ Excel"), sheet 2 "Kiểm tra" (dòng gốc,
  hộp số, số tạm, thời gian gốc → mới, kết luận, ghi chú), sheet 3 danh mục THBQ. "Tải tệp gốc đã đánh dấu" (`xuatTepDanhDau`, ExcelJS) trả lại chính tệp gốc
  `<tên>_DADIEN.xlsx` đã điền thời gian, tô màu + ghi chú như DienThoiGian (thay cho CSV danh sách kiểm tra cũ).
- Tệp thử: `test/chinhly/` (gốc, báo lỗi dòng nhóm) và `test/chinhly/da-sua/` (đã thêm `I.`, xoá dòng con): 104 / 352 hồ sơ.

## Kiểm thử

`cargo test -p digitext-core chinh_ly` (thuần), `DIGITEXT_TEST_DATABASE_URL=… cargo test -p digitext-db chinh_ly`
(CSDL: số tạm trùng, push idempotent/gộp/xung đột, pull tombstone/phân trang/full, hệ thống hoá + gán hộp + giá suy
ra, chặn xoá danh mục), `cd desktop/src-tauri && cargo test chinhly` (mirror: outbox gộp, xoá lan, bỏ thay đổi).
