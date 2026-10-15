# Kế hoạch: đưa kho MinIO ra khỏi ổ ảo Docker + chức năng "Xoá dữ liệu dự án"

Lập 15/10/2026 · triển khai ngay sau khi ký số xong dự án LTLSBN.

## 1. Vì sao làm

| Vấn đề hiện tại | Nguyên nhân |
|---|---|
| Xoá dữ liệu xong ổ E: không trống ra | Kho MinIO nằm **trong ổ ảo** `E:\Docker\wsl\disk\docker_data.vhdx` (531 GB). Ổ ảo chỉ phình, không tự co. Thu gọn phải dừng Docker 1–3 giờ, kéo theo dừng cả KPI, do360, DOP-KHCN. |
| DigiText chưa có cách xoá hẳn một dự án | Chỉ có tắt dự án. Tệp trong kho vẫn còn, phải dọn tay. |
| Ký số / OCR khựng | Kho nằm trên HDD Seagate ST2000DM008 (SMR). Thêm lớp ổ ảo ext4 trong NTFS làm ghi càng chậm. |

**Mục tiêu sau khi làm xong:** kho tài liệu là một thư mục bình thường trên ổ Windows. Xoá dự án → ổ trống ngay. Không còn phải thu gọn ổ ảo, không phải dừng các hệ thống khác.

## 2. Hiện trạng (đo 14–15/10/2026)

- **MinIO** (container `digitext-dev-minio-1`, image `RELEASE.2025-09-07`) giữ một bucket `pages`, khoảng **430 GB**:
  - `{dự án}-warm/{tài liệu}.pdf`: PDF/A
  - `{dự án}-warm/{tài liệu}.signed.pdf`: bản ký số
  - `{tài liệu}/{trang}.jpg`: ảnh trang dùng cho OCR (**không có mã dự án trong đường dẫn**)
  - `{dự án}-packages/{gói}.zip`: gói AIP/SIP
- Phía dùng MinIO (đều qua `http://127.0.0.1:9000`, bucket `pages`):
  - API: `crates/digitext-api/.env`
  - Worker: `crates/digitext-ocr/.env`
- Postgres (khoảng 3 GB) và Meilisearch **giữ nguyên trong Docker**, không đụng tới.
- Từ cuối 2025, **MinIO không còn phát hành bản chạy sẵn cho Windows** (`dl.min.io/.../windows-amd64/minio.exe` trả 410). Bản phát hành mới nhất là `RELEASE.2025-10-15T17-29-55Z`, chỉ còn mã nguồn.
- Mã DigiText **đã có kho dạng thư mục** (`FsObjectStore`, biến `DIGITEXT_PDF_STORE_DIR`) cho API và worker, nhưng luồng tải lên và luồng lấy ảnh trang của worker vẫn gọi thẳng MinIO.

## 3. Quyết định cần anh chốt trước khi làm

| # | Câu hỏi | Đề xuất |
|---|---|---|
| Q1 | Ổ đích đặt kho | **SSD 2 TB mới** (chép 2–4 giờ, hết khựng). Nếu chưa có SSD thì dùng **D:** (HDD, còn trống 484 GB): làm được nhưng chép 10–20 giờ, tốc độ gần như giữ nguyên. |
| Q2 | Phương án | **A: MinIO chạy thẳng trên Windows.** Không sửa mã, rủi ro thấp. Xem mục 4. |
| Q3 | Giữ bản cũ trong ổ ảo bao lâu để quay lui | **14 ngày**, sau đó xoá volume và thu gọn ổ ảo một lần cuối. |
| Q4 | Ai được xoá dữ liệu dự án | Chỉ **quản trị hệ thống (superadmin)**, kèm xác nhận hai bước. |

## 4. Hai phương án

**A — MinIO chạy thẳng trên Windows (đề xuất)**
- Build `minio.exe` từ mã nguồn bản `RELEASE.2025-10-15T17-29-55Z` (cần Go ≥ 1.24). Chạy bằng Scheduled Task `DigiText-MinIO` (tài khoản SYSTEM, tự chạy khi khởi động), giống `DigiText-API` / `DigiText-Worker`.
- Dữ liệu nằm thẳng trên NTFS, ví dụ `X:\DigiText-KhoMayChu\`. MinIO xoá tệp là ổ trống ngay.
- **Không sửa một dòng mã DigiText nào.** API và worker vẫn gọi `127.0.0.1:9000` như cũ.
- Chi phí: 0 đồng (giấy phép AGPL, dùng nội bộ, không sửa, không phân phối).

**B — Bỏ hẳn MinIO, kho là thư mục thường**
- Dùng `FsObjectStore` sẵn có cho mọi chỗ. Phải sửa thêm luồng tải lên (tus/rustus → MinIO) và worker lấy ảnh trang.
- Lợi: một dịch vụ ít hơn, tệp nhìn thấy bằng Explorer.
- Hại: phải sửa và kiểm lại 6 crate (`api`, `ocr`, `ingest`, `storage`, `wipe`, `cli`). Rủi ro cao hơn hẳn, mất khoảng 3–5 ngày.
- → **Để sau.** Chỉ cân nhắc nếu A gặp trục trặc khi build hoặc chạy.

## 4b. Đã làm trước (15/10/2026)

- [x] Go 1.24.8 (bản giải nén, kiểm SHA-256) ở `C:\Users\HieuPC\tools\go`.
- [x] Build `minio.exe` từ thẻ `RELEASE.2025-10-15T17-29-55Z` (commit `9e49d5e7`) → `C:\Users\HieuPC\tools\minio-build\minio.exe`, SHA-256 `a7ee0d09…44e2f1`.
- [x] `dev/minio-win.ps1` (chạy MinIO) và `dev/minio-win-task.ps1` (đăng ký Scheduled Task, tự xin UAC).
- [x] Chạy thử cổng 9100, thư mục thử trên C::
  - `mc` trong container cũ gọi được qua `host.docker.internal:9100`.
  - Chép 2 tệp thật (PDF/A + bản ký) khớp md5.
  - **Xoá xong, ổ trống lại sau khoảng 1 phút.**
- [x] Mã chức năng **Xoá dữ liệu dự án** (mục 6): API `/projects/{id}/purge` + hộp thoại desktop (commit `01e1432`).
- [x] **Vai trò rõ ràng** (commit `d08d56e`): thêm "Quản lý" (`manager`); quản trị dự án chỉ một dự án; migration 0018 đã áp.
- [ ] **Khi ký xong:** build + khởi động lại API, phát hành desktop (hai mục trên mới có hiệu lực).
- [ ] Chọn ổ đích (Q1), thêm loại trừ Defender, đăng ký task ở cổng 9100 → bắt đầu bước 5.3.

## 5. Các bước — Phương án A

Ước tính trên SSD. Trên HDD thì bước 5.3 dài 10–20 giờ.

### 5.0 Chuẩn bị (làm trước, không ảnh hưởng hệ thống, khoảng 1 giờ)
1. Lắp SSD, định dạng NTFS, đặt nhãn `DIGITEXT-KHO`. Nếu dùng D: thì bỏ qua bước này.
2. Cài Go, build `minio.exe` theo đúng thẻ phiên bản, ghi lại SHA-256. Đặt ở `D:\DigiText\infra\minio-win\minio.exe`.
3. **Làm tay:** thêm loại trừ Windows Defender cho thư mục kho và `C:\Users\HieuPC\tools\minio-build\minio.exe` (Windows Security → Virus & threat protection → Exclusions). Quét thời gian thực làm ghi chậm đi rất nhiều. Script tự thêm loại trừ thì bị chính Defender chặn, coi là phần mềm độc hại (15/10).
4. Viết `dev/minio-win.ps1`:
   - Đọc tài khoản gốc từ `infra/compose/.env` (dùng lại đúng `MINIO_USER` / `MINIO_PASSWORD`, để API/worker không phải đổi khoá).
   - Đặt `MINIO_SCANNER_SPEED=slowest` và `MINIO_DRIVE_MAX_TIMEOUT=2m`.
   - Chạy `minio.exe server <thư mục kho> --address 127.0.0.1:<cổng> --console-address 127.0.0.1:<cổng+1>`, ghi log ra `logs/minio.log`.
5. Tạo Scheduled Task `DigiText-MinIO` (SYSTEM, khi khởi động, tự chạy lại khi lỗi). **Tạm dùng cổng 9100** để chạy song song với MinIO cũ.

### 5.1 Chạy thử MinIO mới (khoảng 15 phút, không ảnh hưởng)
- Tạo bucket `pages`, ghi, đọc, xoá một tệp thử.
- Kiểm `mc admin info`, log không có lỗi.

### 5.2 Điều kiện bắt đầu chép
- Ký số LTLSBN **đã xong**. Không có phiên ký mở, không có việc OCR/PDF/A đang chạy. Kiểm bằng cửa sổ "Theo doi OCR".
- **Chưa đẩy** phông mới (STNHB, SO THUY LOI HB, STCVG HB).

### 5.3 Chép lần 1 trong lúc hệ thống vẫn chạy (2–4 giờ trên SSD)
- Dùng `mc` có sẵn trong container MinIO cũ, chép sang MinIO mới qua `host.docker.internal:9100`:
  `mc mirror --preserve old/pages new/pages`
- Hệ thống vẫn dùng bình thường. Dữ liệu phát sinh trong lúc chép sẽ được chép ở bước 5.4.

### 5.4 Chuyển hẳn (dừng khoảng 15–30 phút; báo trước cho người dùng)
1. Dừng `DigiText-API` và `DigiText-Worker`.
2. Chép lần 2 (chỉ phần chênh): `mc mirror --preserve --overwrite old/pages new/pages`.
3. Đối chiếu:
   - `mc diff old/pages new/pages` phải **rỗng**.
   - So số tệp và tổng dung lượng hai bên.
   - Lấy mẫu ngẫu nhiên 200 tệp, so SHA-256.
4. Dừng container MinIO cũ: `docker compose -p digitext-dev stop minio`. **Không xoá volume.**
5. Chuyển MinIO mới sang cổng **9000** (sửa tham số trong Scheduled Task rồi chạy lại). API/worker không phải đổi `.env`.
6. Bật lại `DigiText-API` và `DigiText-Worker`.
7. Sửa `infra/compose/docker-compose.yml`: đưa dịch vụ `minio` vào `profiles: ["old-minio"]`, để `docker compose up` không tự bật lại nó.

### 5.5 Kiểm tra sau khi chuyển (30 phút)
- `/healthz` = 200; cửa sổ "Theo doi OCR" đọc được hàng đợi.
- Desktop: mở PDF/A một tài liệu, tải bản đã ký, xem ảnh trang.
- Ký số thử 3 tệp; đóng gói thử một hồ sơ.
- Đẩy thử một tệp mới, chờ OCR → PDF/A xong; kiểm bản gốc được xoá đúng (`DIGITEXT_PURGE_SOURCE_AFTER_PDFA`).
- Xoá thử một tệp → dung lượng ổ đích giảm ngay.

### 5.6 Quay lui (nếu bước 5.5 hỏng)
1. Dừng `DigiText-MinIO`.
2. `docker compose -p digitext-dev start minio`. Dữ liệu cũ còn nguyên trong volume.
3. Bật lại API/worker. Mất khoảng 5 phút.
4. Dữ liệu phát sinh sau giờ chuyển (nếu có) chép ngược bằng `mc mirror new/pages old/pages`.

### 5.7 Dọn sau 14 ngày chạy ổn
1. `docker volume rm digitext-dev_miniodata`.
2. Chạy `dev/thu-nho-docker.ps1` **một lần cuối**, ngoài giờ: ổ ảo co từ khoảng 530 GB xuống còn khoảng 10 GB.
3. Cập nhật `docs/PROJECT-MAP.md` và `docs/HANDOFF.md`: kho nằm ở đâu, cách khởi động lại.

## 6. Chức năng "Xoá dữ liệu dự án"

Làm song song với mục 5, phát hành cùng đợt. Thiết kế:

**Máy chủ — `POST /projects/{id}/purge`** (chỉ superadmin)
1. **Chạy thử** (`dryRun: true`, mặc định): trả về số tài liệu, số trang, số tệp và dung lượng trong kho, số gói, số phiên ký, số bản ghi chỉnh lý. Không xoá gì.
2. **Điều kiện chặn:**
   - Còn việc OCR/PDF/A/đóng gói đang chờ hoặc đang chạy.
   - Còn phiên ký đang mở.
   - Dự án **chưa có gói bàn giao** (chặn, trừ khi đánh dấu "xoá dù chưa bàn giao" kèm lý do).
3. **Thi hành:** phải gửi đúng **mã dự án** gõ tay (vd `LTLSBN`). Thứ tự:
   1. Ghi nhật ký `project_purge_log`: ai, lúc nào, lý do, số liệu lúc chạy thử.
   2. Xoá trong kho:
      - `{dự án}-warm/`, `{dự án}-packages/`
      - ảnh trang `{tài liệu}/` của từng tài liệu. Ảnh trang không mang mã dự án, nên phải lấy danh sách tài liệu từ CSDL trước khi xoá CSDL.
   3. Xoá CSDL trong một giao dịch:
      - `documents` và các bảng phụ thuộc (`pages`, `ocr_jobs`, `ocr_results`, `ocr_fields`, `sign_sessions`, `review_queue`, `work_items`…)
      - các bảng `cl_*`, `packages`, `submissions`, `work_*` theo `project_id`.
   4. Tắt dự án và đổi tên thành `[ĐÃ XOÁ] …`. Giữ lại tài khoản và nhật ký.
   - Chạy nền, có tiến độ. Đứt giữa chừng thì chạy lại được: bước kho xoá theo danh sách, bước CSDL chạy sau cùng.
4. Không có "thùng rác": xoá là mất. Màn hình phải nói rõ điều này, và gợi ý **xuất gói / sao lưu trước**.

**Desktop — Quản trị → Dự án → "Xoá dữ liệu dự án"**
- Bước 1: bảng số liệu chạy thử và cảnh báo.
- Bước 2: gõ lại mã dự án, nhập lý do.
- Bước 3: thanh tiến độ, kết quả "đã giải phóng X GB".
- Tuỳ chọn **"xoá luôn bản trong Kho máy này"** (`D:\DigiText-Kho\<dự án>`).

**Kiểm thử:** tạo một dự án thử, nạp 20 tệp, OCR, ký 5 tệp, đóng gói 1 hồ sơ → chạy thử → xoá. Kiểm kho không còn tệp nào của dự án đó, CSDL không còn dòng nào, các dự án khác còn nguyên.

## 7. Lịch đề xuất

| Khi nào | Việc | Ảnh hưởng người dùng |
|---|---|---|
| Trong lúc chờ ký xong | 5.0, 5.1; viết mã chức năng xoá dự án | Không |
| Ký xong (ngày N) | 5.3 chép lần 1 | Không (máy chậm hơn chút) |
| Tối ngày N | 5.4 chuyển hẳn, 5.5 kiểm tra | **Dừng 15–30 phút** |
| N+1 | Phát hành desktop có "Xoá dữ liệu dự án"; đẩy phông STNHB, SO THUY LOI HB | Không |
| Sau khi có SSD 2 TB và chuyển xong | Đẩy STCVG HB (273 GB) | Không |
| N+14 | 5.7 xoá volume cũ, thu gọn ổ ảo lần cuối | Dừng Docker 1–3 giờ (ngoài giờ) |

## 8. Rủi ro

| Rủi ro | Cách xử lý |
|---|---|
| Build `minio.exe` lỗi / Go không cài được | Dùng bản build sẵn từ máy khác; cùng lắm chuyển sang phương án B. |
| Chép thiếu hoặc lỗi tệp | Bước 5.4 đối chiếu `mc diff` + mẫu SHA-256; dữ liệu cũ giữ 14 ngày. |
| Ổ đích đầy (nếu dùng D: HDD) | Trước khi chép kiểm D: trống ≥ dung lượng kho + 50 GB; nếu không đủ thì chờ SSD. |
| Windows Defender / Windows Update khởi động lại máy giữa lúc chép | Thêm loại trừ Defender; tạm dừng Windows Update trong ngày chuyển (như lần thu gọn Docker 9/10). |
| Xoá nhầm dự án | Chỉ superadmin; chạy thử bắt buộc; gõ lại mã dự án; chặn dự án chưa bàn giao; nhật ký. |
