# Đối chiếu 94 hạng mục Digifile TT05 với DigiText bản Lưu trữ (27/11/2026)

> **Cập nhật sau đợt 6 (0.25.26, 0040):** các dòng đánh dấu **→ ✓ Đ6** đã làm; bảng tổng hợp dưới là trước đợt 6. Sau đợt 6: 62 đủ, 9 một phần, 10 chưa (chưa: A08 license, B07, B08, B13, D12 OAuth/OpenAPI, F01, F02, F13, nhóm I).

Nguồn Digifile: `D:\Digifile\KeHoachNangCap_TT05\03_Danh_muc_chuc_nang_bao_gia.md` (94 hạng mục A–I) và hai báo cáo nghiệm thu
`05_*`, `06_*` (mô tả cái đã làm thật). Nguồn DigiText: mã trên nhánh `feat/ban-luu-tru` tại desktop 0.25.25, migration 0039.

Ký hiệu: **✓** có tương đương · **◐** có một phần (ghi rõ thiếu gì) · **✗** chưa có · **—** không áp dụng cho DigiText
(hạng mục chỉ để vá kiến trúc .NET cũ của Digifile).

## Tổng hợp

| Nhóm | Số mục | ✓ | ◐ | ✗ | — |
|---|---:|---:|---:|---:|---:|
| A. Nền tảng & an toàn | 13 | 4 | 2 | 1 | 6 |
| B. Mô hình dữ liệu & danh mục TT05 | 14 | 7 | 4 | 3 | 0 |
| C. Lưu trữ tệp, số hoá, ký số, đóng gói | 11 | 9 | 2 | 0 | 0 |
| D. Thu nộp | 12 | 5 | 5 | 2 | 0 |
| E. Bảo quản & huỷ | 8 | 8 | 0 | 0 | 0 |
| F. Sử dụng trực tuyến (cổng độc giả) | 14 | 6 | 5 | 3 | 0 |
| G. Chuyển đổi, in, báo cáo | 4 | 3 | 1 | 0 | 0 |
| H. Trải nghiệm & chất lượng | 12 | 5 | 1 | 0 | 6 |
| I. Tuỳ chọn | 6 | 0 | 0 | 5 | 1 |
| **Tổng** | **94** | **47** | **20** | **14** | **13** |

Trong 81 mục áp dụng được: 47 đủ, 20 một phần, 14 chưa có. Nhóm **E (bảo quản, huỷ) và G (chuyển đổi, in, báo cáo) đủ**;
nhóm **F (cổng độc giả) và D (thu nộp từ cơ quan ngoài) thiếu nhiều nhất**.

## A. Nền tảng & an toàn

| Mã | Digifile | DigiText | Ghi chú |
|---|---|---|---|
| A01 .NET 8 | — | — | Rust / Tauri |
| A02 EF Migrations | ✓ | ✓ | `digitext-api migrate`, 0001–0039, bảng `_digitext_migrations` |
| A03 Mật khẩu, lockout, 2FA | ✓ | ◐ | băm Argon2 (`password.rs`), JWT Ed25519, thu hồi phiên; **chưa thấy khoá sau N lần sai, chưa có 2FA** |
| A04 Secret ngoài mã | ✓ | ✓ | `conf\*.env`, JWT tự sinh, mật khẩu ngẫu nhiên lúc cài |
| A05 Phân quyền toàn cục, chống IDOR | ✓ | ✓ | `require_perm` + `project_access` + phạm vi phông (0038) trên mọi route |
| A06 CSRF/XSS/headers | ✓ | — | API Bearer token, desktop Tauri (không cookie) |
| A07 Kho tệp an toàn, quét virus | ✓ | ✓ | kho ngoài web, ClamAV (0027), kiểm BLAKE3 |
| A08 License ký RSA | ✓ | ✗ | DigiText không có cơ chế bản quyền; bản Lưu trữ phát hành theo máy chủ |
| A09 Log, health check | ✓ | ✓ | tracing, `/healthz`, `giam-sat.ps1` |
| A10 Git, CI, test | ✓ | ◐ | git có; **test sản phẩm gần như không có** (chỉ core/pdf/extimport) |
| A11–A13 | — | — | vấn đề riêng mã .NET cũ |

## B. Mô hình dữ liệu & danh mục TT05

| Mã | DigiText | Ghi chú |
|---|---|---|
| B01 Cơ quan lưu trữ | ✓ | 0039, màn Quản trị › Cơ quan lưu trữ (0.25.25) |
| B02 Phông: trạng thái, lịch sử ĐVHTP có phiên bản | ✓ | `phong_lich_su`; khối phông / mã phông / mục lục số có sẵn |
| B03 Danh mục mã hoá TT05 | ◐ → ✓ Đ6 | có THBQ (mã 01–07), ngôn ngữ, loại văn bản 32 mã; **chế độ sử dụng, mức độ tin cậy, tình trạng vật lý, tình trạng dự phòng là ô chữ tự do**, không có bảng mã + màn quản trị |
| B04 18 trường EAD hồ sơ | ◐ → ✓ Đ6 | có từ khoá, ký hiệu thông tin, mức độ tin cậy, mã hồ sơ giấy, ngày không đầy đủ, ngôn ngữ; **thiếu tình trạng dự phòng (riskRecovery/Status)** |
| B05 EAD_doc | ◐ → ✓ Đ6 | có loại VB mã TT05, tách số / ký hiệu, độ tin cậy; **thiếu `process` (quy trình xử lý kèm tệp luồng) và dự phòng** |
| B06 Bộ sinh mã | ◐ → ✓ Đ6 (ma_ho_so_tt05, ma_ho_so_giay_tt05) | mã hồ sơ `mãphông.phôngsố.năm.số[.mụclục]`, arcDocCode STT 7 chữ số, OBJID urn; **chưa có mã cơ quan lưu trữ ở đầu và paperFileCode theo kho/giá/hộp** — chờ câu (20) |
| B07 Tài liệu ảnh | ✗ | EAD_pic chỉ có tên schema |
| B08 Tài liệu ghi âm / ghi hình | ✗ | EAD_media chỉ có tên schema |
| B09 Gói tài liệu rời lẻ | ◐ → ✓ Đ6 | đóng gói `AIP_tailieu` được (package); **không có đối tượng "gói tài liệu" trên màn hình để lập / quản lý** |
| B10 Nhiều tệp / tài liệu, SHA, PDF/A, chữ ký | ✓ | `documents` + `cl_van_ban_file` |
| B11 Form lập hồ sơ / biên mục | ✓ | HoSoForm, lưới văn bản, kiểm tra TT05 |
| B12 Excel chuẩn TT05 | ✓ | MLHS / MLVB / 51 cột, lỗi từng dòng |
| B13 Chuyển đổi TT02 → TT05 | ✗ | DigiText sinh ra đã theo TT05; chỉ có "Chuyển dữ liệu cũ" từ SQLite máy trạm. Cơ quan có dữ liệu TT02 cũ thì nhập qua Excel 51 cột |
| B14 Tìm kiếm full-text | ✓ | tra cứu không dấu trên mục lục + lớp chữ |

## C. Lưu trữ tệp, số hoá, ký số, đóng gói

| Mã | DigiText | Ghi chú |
|---|---|---|
| C01 Storage service, PREMIS | ✓ | FsObjectStore / MinIO, WORM, PREMIS trong gói; sao lưu 2 bộ qua `sao-luu.ps1` |
| C02 PDF/A hai lớp + OCR | ✓ | chính là thế mạnh của DigiText (bản Lưu trữ cần máy chủ OCR nếu muốn OCR lại) |
| C03 Ký số server-side (HSM / SmartCA) | ◐ | ký bằng USB token tại máy trạm, PAdES-LTV; **chưa có HSM / SmartCA server-side** |
| C04 Ký số token trong luồng | ✓ | |
| C05 Module quản lý số hoá | ◐ → ✓ Đ6 (vào menu bản Lưu trữ) | đủ ở bản thi công (giao nhận, QC, nghiệm thu, nhật ký); **bản Lưu trữ chưa đưa vào menu** — xem câu hỏi cuối |
| C06 METS CSIP | ✓ | |
| C07 EAD / EAD_doc | ✓ | EAD_pic / EAD_media ✗ (B07, B08) |
| C08 PREMIS v3 | ✓ | |
| C09 Đóng gói + kiểm + tái đóng gói | ✓ | |
| C10 Nhập gói AIP / SIP | ✓ | `nhap-goi`, Nhập gói AIP |
| C11 Bộ kiểm thử gói mẫu | ✓ | pkgcheck, `check_tt05_package` |

## D. Thu nộp

| Mã | DigiText | Ghi chú |
|---|---|---|
| D01 Nộp lưu hiện hành | ✓ | 0031: 60 ngày, nhắc, trả lại, khoá sửa, 360 ngày, sổ, biên bản |
| D02 Đăng ký nộp LTLS 17 thông tin | ◐ → ✓ Đ6 (tệp đính kèm) | `/submissions` có mã yêu cầu, cơ quan, đã nộp giấy, số lần, tóm tắt, phông, phương án, cách thức, dự kiến, liên hệ, mục lục; **thiếu tệp đính kèm (văn bản đề nghị, lịch sử phông, hướng dẫn xác định giá trị) và hạn đăng ký 12 tháng** |
| D03 Xác nhận yêu cầu (7 ngày) | ✓ | đối soát + SLA `sla_confirm_overdue` |
| D04 Nộp cùng hệ thống | ◐ → ✓ Đ6 | hồ sơ nộp theo mục lục Excel / gói; **chưa có "chọn hồ sơ đang có → nộp vào LTLS"** |
| D05 Nộp gói SIP | ✓ | |
| D06 Tiếp nhận 10 thông tin | ◐ → ✓ Đ6 | có tổng gói / hồ sơ / dung lượng; chưa đủ 10 cột Điều 23 |
| D07 Trùng lặp + kiểm từng hồ sơ | ◐ → ✓ Đ6 | có `duplicate_in_system`, đạt / không đạt từng dòng; **chưa có màn so sánh 2 hồ sơ nghi trùng, chưa tự từ chối trùng trong cùng đợt** |
| D08 Phê duyệt, hoàn thành | ✓ | |
| D09 Biên bản | ✓ | records |
| D10 Vào kho / trả lại | ✓ | `/returned` |
| D11 Tài khoản cơ quan nộp (multi-tenant nhẹ) | ✗ → ✓ Đ6 | chỉ có vai trò customer theo dự án; **không có danh mục cơ quan nộp lưu** |
| D12 REST API + API key cho hệ thống ngoài | ✗ → ◐ Đ6 (X-Api-Key, tài liệu tay; chưa OAuth / OpenAPI) | API JWT nội bộ, webhook NCC; không có API key |

## E. Bảo quản & huỷ — đủ 8/8

E01 kế hoạch kiểm tra · E02 Mẫu 01 · E03 lịch sao lưu, Mẫu 02 / 03 · E04 phương tiện · E05 sự cố Mẫu 04 · E06 quét hết hạn + trùng ·
E07 hội đồng · E08 thẩm định, quyết định, thực hiện có lưu vết — tất cả có (0032, 0033, `/disposal/*`).

## F. Sử dụng trực tuyến (cổng độc giả)

| Mã | DigiText | Ghi chú |
|---|---|---|
| F01 Cổng độc giả web | ✗ | anh đã bảo để sau |
| F02 Đăng ký độc giả CCCD + OTP | ✗ | như trên |
| F03 Yêu cầu đọc | ◐ | vai trò độc giả làm trên desktop, không có web / giỏ |
| F04 Duyệt 2 cấp | ◐ → ✓ Đ6 | duyệt từng dòng có cờ "có điều kiện"; **không có cấp 2 riêng** |
| F05 Phí, thanh toán | ✓ | bảng giá, xác nhận thanh toán |
| F06 Bản đọc watermark, hết hạn, tự xoá | ✓ | |
| F07 Bản sao không xác thực | ✓ | watermark + "BẢN SAO" |
| F08 Bản sao có xác thực | ✓ | ký bằng khoá cơ quan (`copy_sign`) |
| F09 DIP có xác-thuc.pdf | ✓ | |
| F10 Bản sao giấy | ◐ → ✓ Đ6 | in qua chuyển đổi PL VI; **chưa theo dõi giao nhận bản giấy** |
| F11 Cung cấp thông tin 3 hình thức | ✓ | danh mục / trích xuất / tổng hợp |
| F12 Backoffice SLA, phân công | ◐ → ✓ Đ6 | SLA nhắc có; **chưa phân công người xử lý, chưa thống kê % đúng hạn** |
| F13 Email / SMS | ✗ | chỉ in-app; để sau |
| F14 Mượn – trả nâng cấp | ✓ | 0034 |

## G. Chuyển đổi, in, báo cáo

| Mã | DigiText | Ghi chú |
|---|---|---|
| G01 Chuyển đổi PL VI | ✓ | 0037 |
| G02 PDF máy chủ / máy trạm cho biểu mẫu | ✓ | |
| G03 Báo cáo + dashboard | ✓ | 19 báo cáo, Tổng quan Lưu trữ |
| G04 Tài liệu, đào tạo | ◐ → ◐ Đ6 (HDSD theo vai trò có; video chưa) | có hướng dẫn cài; **chưa có HDSD theo vai trò, video** |

## H. Trải nghiệm & chất lượng

H01–H07, H11 — (vá giao diện .NET cũ; DigiText có bộ `ui/*` riêng) · H08 audit log ✓ (`audit_log`, nhật ký khai thác; **chuỗi băm chống sửa có từ Đ6** — `/ops/audit/kiem-chuoi`) · H09 hiệu năng ✓ · H10 ma trận quyền ✓ (0022) · H12 danh mục hồ sơ có duyệt ✓ (0031).

## I. Tuỳ chọn — ✗ tất cả (cổng thanh toán, Elasticsearch, LGSP, song ngữ, ASR); I06 hạ tầng —.

## Việc đề nghị làm tiếp (theo thứ tự tác động)

1. **Cơ quan nộp lưu + tài khoản cơ quan nộp (D11) và hoàn thiện đăng ký nộp (D02, D04, D06, D07)** — bản Lưu trữ lịch sử nhận từ
   nhiều cơ quan thì đây là chỗ hụt rõ nhất so với Digifile.
2. **Bảng mã TT05 còn thiếu (B03) + trường dự phòng (B04, B05)** — nhỏ, nhưng là "Bắt buộc (TT05)" trong kế hoạch Digifile.
3. **Mã hồ sơ có mã cơ quan lưu trữ, paperFileCode (B06)** — chờ anh chốt câu (20).
4. **Gói tài liệu rời lẻ trên màn hình (B09)**.
5. **Đưa module số hoá vào bản Lưu trữ (C05)** nếu cơ quan tự số hoá — kéo theo máy chủ OCR.
6. Khoá sau N lần đăng nhập sai (A03), phân công + % đúng hạn ở khai thác (F12), theo dõi bản sao giấy (F10), HDSD theo vai trò (G04).
7. Để sau theo quyết định đã có: cổng độc giả (F01–F03), email / SMS (F13), ảnh / ghi âm (B07, B08), nhóm I.
