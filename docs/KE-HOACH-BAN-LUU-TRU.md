# Kế hoạch: bản "DigiText Lưu trữ" cài tại máy chủ chủ đầu tư (4/11/2026)

## Chốt với anh Hiếu

- Phương án B: một bản desktop riêng, cùng mã nguồn, kết nối **máy chủ đặt tại cơ quan chủ đầu tư**.
  Nhân viên CĐT tải bộ cài và nhận cập nhật **từ chính máy chủ đó**.
- Là "phần mềm lưu trữ" theo TT05: dữ liệu vào bằng **nhập gói AIP** (bước 7 Điều 9, "nhập Hệ thống"),
  không sao chép CSDL thi công.
- Chức năng: **trọn nhóm Lưu trữ số** = Tra cứu, Khai thác, Cung cấp thông tin, Bảo quản và sao lưu, Tiêu huỷ.
- Máy chủ CĐT: **cả Windows và Linux**.

## Hiện trạng (khảo sát 4/11)

| Phần | Có sẵn | Thiếu |
|---|---|---|
| Bản desktop riêng | menu lọc theo quyền (`gate`), vai trò `customer` tầng Read | tên/identifier/cập nhật/logo cứng (`tauri.conf.json`, `config.ts`, `Topbar`, `BrandLogo`); không có cơ chế bản |
| Cập nhật | API phát `/updates/{file}` từ `DIGITEXT_UPDATES_DIR` | endpoint cứng `api.digitext.vn`; `latest.json` chứa URL tuyệt đối; không có trang tải về |
| Máy chủ | migrations 0001–0024, `bootstrap-superadmin`, JWT tự sinh, bucket tự tạo | không có bộ cài; compose chỉ có Postgres; script chỉ hợp OCR PC; tạo quản trị phải gõ mật khẩu tay; không theo dõi migration đã áp |
| Nhập gói | pkgcheck đọc zip/thư mục, kiểm METS/EAD/PREMIS/PDF/A/chữ ký; `validate::f_2p7`; `apply_op`, `import.rs`, `find_or_create_hop`, mẫu `tu_ocr.rs` | **không có đường nào nạp gói vào CSDL**; `load`/`xml` của pkgcheck private; gói không mang số hộp; mã hồ sơ TT05 không lưu (chỉ tính trong view) |
| 4 module Lưu trữ số | API + màn đủ luồng nghiệp vụ | dựa trên `documents` + MinIO từ upload/OCR/ký; DIP dựng lại từ `packages.request_json`; Cung cấp thông tin tìm trên `ocr_results`; Tiêu huỷ dựa hạn lưu trữ nhập tay; Bảo quản không kiểm zip gói; người khai thác không chọn được tài liệu (picker cần `document.list`) |
| Tra cứu | `/cl/ho-so?q=`, `/cl/van-ban` (ILIKE, trigram trên tiêu đề/trích yếu) | phân biệt dấu; không tìm toàn văn; không có màn tra cứu cho người khai thác |
| Nhật ký | `audit_log` cho đăng nhập, sửa dữ liệu | **không ghi ai xem/tải tài liệu** |
| Việc nền | quét hết hạn bản sao, ứng viên tiêu huỷ, kiểm toàn vẹn, dựng DIP | nằm trong worker OCR (kéo theo dịch vụ OCR/vLLM/ClamAV) |

## Kiến trúc

- **Một mã nguồn, hai bản.** API cùng tệp chạy; `DIGITEXT_EDITION=luu_tru` tắt route thi công (quét, OCR, giao việc,
  đóng gói từ tài liệu…). Desktop build bằng lớp phủ `tauri.luutru.conf.json` + `VITE_EDITION=luu_tru`: tên
  "DigiText Lưu trữ", identifier riêng `vn.digitext.luutru` (cài song song bản thi công được), menu chỉ nhóm
  Lưu trữ số + Quản trị.
- **Cập nhật theo máy chủ:** lệnh Tauri dùng `updater_builder().endpoints()` từ địa chỉ máy chủ đã cấu hình; API trả
  `latest.json` với URL viết lại theo địa chỉ công khai của chính nó (`DIGITEXT_PUBLIC_URL`) và trang `GET /tai-ve`
  có nút tải bộ cài. Bộ cài vẫn ký minisign bằng khoá của mình. Bản phát hành giao CĐT là một tệp zip chép vào
  thư mục updates của máy chủ họ.
- **Dữ liệu:** gói AIP → `cl_phong/cl_hop/cl_ho_so/cl_van_ban/cl_van_ban_file` (mục lục) + `documents` (`archived`)
  + PDF tại `{p}-warm/{doc}.signed.pdf` + zip gốc tại `{p}-packages/…` (WORM) + lớp chữ PDF vào `ocr_results`
  để tìm toàn văn. Mọi module Lưu trữ số chạy trên chính các bảng này.

## Giai đoạn

### GĐ1 — Nền bản riêng (3–4 ngày) — XONG 4/11/2026 (nhánh `feat/ban-luu-tru`)
Còn để sau: icon/logo riêng (đang dùng logo DigiText); tên ứng dụng trong bộ cài vẫn có dấu ("DigiText Lưu trữ"),
tệp bộ cài đổi sang tên ASCII khi phát hành.
1. `desktop/src-tauri/tauri.luutru.conf.json`, icon/logo riêng; `config.ts` đọc `VITE_EDITION`; thay các chỗ tên cứng.
2. `navTreeFor(edition)` dùng ở Sidebar, CommandPalette, Topbar, AppShell; Tổng quan riêng cho bản lưu trữ
   (số hồ sơ/văn bản/trang, yêu cầu khai thác chờ duyệt, cảnh báo bảo quản).
3. Lệnh `update_check`/`update_install` theo `server_url()`; `updater.ts` chuyển sang (dùng cho cả bản thi công).
4. `dev/release-desktop.ps1 -Edition luutru`: build lớp phủ, ra `phat-hanh-luutru-<ver>.zip`.
5. API: `DIGITEXT_PUBLIC_URL`, `latest.json` viết lại URL, `GET /tai-ve`; `DIGITEXT_EDITION` chặn route thi công.

### GĐ2 — Bộ cài máy chủ (4–5 ngày)
1. Lệnh `digitext-api migrate` (áp `migrations/*.sql` theo thứ tự, bảng `_digitext_migrations`, 0006 ngoài giao dịch);
   dùng luôn cho OCR PC thay áp tay.
2. `bootstrap-superadmin --password-file`.
3. Việc nền lưu trữ chạy không cần OCR: chế độ worker `--chi-luu-tru` (hoặc chuyển vào API).
4. **Windows:** `luutru-server-win.zip` = api.exe, worker.exe, minio.exe, caddy.exe, migrations, mẫu `.env`/Caddyfile,
   `cai-dat.ps1` (đăng ký dịch vụ, bucket, migrate, tạo quản trị, sao lưu hằng ngày, watchdog). Postgres 16 dùng bộ cài
   chuẩn của PostgreSQL (script kiểm và hướng dẫn).
5. **Linux:** `Dockerfile` API/worker + `infra/compose/luutru/docker-compose.yml` (postgres, minio, api, worker, caddy) + `cai-dat.sh`.
6. `docs/CAI-DAT-MAY-CHU-LUU-TRU.md` cho kỹ thuật viên CĐT.

### GĐ3 — Nhập gói AIP vào Hệ thống (5–7 ngày) — XONG 4/11/2026 (nhánh `feat/ban-luu-tru`)

Đã làm và chạy thử trên Postgres trống (migration 0001–0025 áp sạch) với 3 gói Diễn Xá (`test/dienxa/goi-thu`, bộ dựng đời
cũ, kiểm KHÔNG ĐẠT nên dùng `--nhan-ca-goi-loi`): 3 hồ sơ, 150 văn bản, 150 tài liệu `archived` chữ ký hợp lệ, 3 zip gốc,
lớp chữ 114 tài liệu — 16 giây. Nhập lại → "đã có"; chế độ chặt → từ chối; dừng giữa chừng rồi chạy lại → không trùng.
Đường HTTP (lô → khúc 40 MB + 26 MB, lệch offset trả 409 → nhập → kết thúc) chạy với API bản lưu trữ.

Lệnh máy chủ:
```
digitext-api nhap-goi --du-an <mã dự án> --nguoi <tên đăng nhập> [--nhan-ca-goi-loi] [--ghi-chu "<chữ>"] <thư mục|gói.zip>…
```

Quy tắc đã chốt: chữ ký KHÔNG toàn vẹn → luôn từ chối; tệp chưa ký → nhận, `documents.ky_so = 'chua_ky'`, đếm trong báo cáo;
kiểm gói có lỗi → từ chối trừ khi bật nhận cả gói lỗi; cùng mã hồ sơ từ gói khác → từ chối (xoá hồ sơ cũ trước). Hộp không
suy ra từ gói (gói không mang số hộp; `paperFileCode` thường trùng mã hồ sơ) — để GĐ4 nếu cần. Lớp chữ gói Diễn Xá không dấu
(nghi lớp chữ OCR cũ của chính gói) — kiểm lại trên gói DigiText đóng.
1. pkgcheck thành thư viện: `pub mod load/xml`, `load_bytes`, hàm trả cây EAD + bản đồ METS DMDID→FILEID.
2. Migration 0025: `packages.nguon` (`dung`/`nhap`), `request_json` được rỗng với gói nhập; `cl_ho_so.ma_ho_so_goc`,
   `cl_ho_so.objid` (nhập lại không trùng); bảng `ingest_batches`/`ingest_items` (lô, từng gói, kết quả kiểm, lỗi).
3. Nhập: kiểm pkgcheck (lỗi → từ chối gói, ghi lý do) → phông theo mã phông trong OBJID/arcFileCode (tạo nếu thiếu) →
   hồ sơ từ EAD hồ sơ (hộp lấy từ `paperFileCode` dạng hộp.hs nếu có) → văn bản từ EAD văn bản, ghép PDF theo structMap →
   `documents` + MinIO → lớp chữ vào `ocr_results` → zip gốc vào WORM. Ghi `cl_lich_su` nguồn `import`.
4. Hai đường vào: **lệnh máy chủ** `digitext-api nhap-goi <thư mục>` (bộ hồ sơ nghiệm thu hàng trăm GB giao bằng ổ cứng)
   và màn desktop **Nhập gói vào Hệ thống** (lô nhỏ, tiến độ, báo cáo kết quả nhập PDF/Excel).
5. WORM thật: chặn ghi đè khoá `*-packages/`, không chỉ chặn xoá.

### GĐ4 — Tra cứu và 4 module trên dữ liệu nhập (8–10 ngày)
1. **Tra cứu:** migration 0026 `unaccent` + chỉ mục trigram không dấu (tiêu đề, trích yếu, số ký hiệu, cơ quan ban hành,
   lớp chữ); `GET /tra-cuu` (lọc phông, năm, THBQ, loại văn bản, chế độ sử dụng); màn Tra cứu: ô tìm chung + bộ lọc +
   xem PDF trong app. Tài liệu hạn chế sử dụng chỉ hiện mục lục, không mở bản số.
2. **Khai thác:** chọn tài liệu từ kết quả tra cứu; DIP dựng từ gói AIP gốc thay `request_json`.
3. **Cung cấp thông tin:** tìm trên mục lục + lớp chữ thay vì tên tệp.
4. **Bảo quản:** kiểm toàn vẹn zip gói (`zip_sha256`) và từng PDF (blake3) theo lịch; báo cáo; sao lưu giữ như hiện có.
5. **Tiêu huỷ:** ứng viên tính từ THBQ + năm kết thúc của hồ sơ; thực hiện = ẩn khỏi tra cứu, xoá bản làm việc,
   ghi biên bản; zip WORM theo quyết định bên dưới.
6. **Nhật ký khai thác:** ghi `audit_log` mọi lần xem/tải/in; màn tra nhật ký cho cán bộ lưu trữ.
7. Vai trò mặc định: Người khai thác (`customer`), Cán bộ lưu trữ (`admin`), Quản trị (`super_admin`).

### GĐ5 — Chạy thử trọn vẹn (2–3 ngày)
Máy ảo Windows sạch + máy Linux Docker: cài máy chủ theo tài liệu, tải bộ cài từ `/tai-ve`, nhập gói Đầm Hà / Bắc Giang,
tra cứu, yêu cầu khai thác → duyệt → tải bản sao, kiểm toàn vẹn, tiêu huỷ thử, phát hành bản cập nhật và xem máy trạm tự cập nhật.

**Tổng ước: khoảng 5 tuần.** GĐ1 và GĐ3 cũng có ích cho bản thi công (cập nhật theo máy chủ, lệnh migrate, nhập gói).

## Cần anh quyết

1. **Kết nối máy trạm ↔ máy chủ CĐT.** App hiện chỉ nhận `https` có chứng chỉ công khai (rustls không đọc kho chứng chỉ
   Windows); `http` chỉ cho localhost. Đề xuất: bản lưu trữ dùng kho chứng chỉ Windows (`rustls-platform-verifier`) +
   Caddy tự cấp chứng chỉ nội bộ, bộ cài máy trạm kèm chứng chỉ gốc → chạy trong LAN không cần tên miền. CĐT có tên miền
   thì dùng Let's Encrypt như OCR PC.
2. **Tiêu huỷ và WORM.** Gói AIP không xoá được, nhưng tiêu huỷ phải huỷ cả bản số. Đề xuất: chỉ đường "thực hiện quyết
   định tiêu huỷ" (có số quyết định, ghi nhật ký) được xoá zip; mọi đường khác vẫn chặn.
3. **Gói chưa ký số** (worker rơi về bản chưa ký khi thiếu): đề xuất nhận, đánh dấu "chưa ký" trên hồ sơ và báo cáo nhập.
4. **Thứ tự:** đề xuất GĐ1 → GĐ3 → GĐ4 → GĐ2 → GĐ5 — dựng và nhập thử trên máy chủ dev trước, đóng bộ cài sau cùng.
