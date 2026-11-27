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

### GĐ2 — Bộ cài máy chủ (4–5 ngày) — XONG 9/11/2026 (nhánh `feat/ban-luu-tru`), chưa thử trên máy sạch (GĐ5)

Đã làm: `digitext-api migrate` (`crates/digitext-api/src/migrate.rs`, bảng `_digitext_migrations`, tách câu lệnh chạy
autocommit trên một kết nối nên `CREATE INDEX CONCURRENTLY` chạy được; máy chủ đã áp tay: `migrate --da-ap-den <NNNN>`);
worker `DIGITEXT_CHI_LUU_TRU=true` (không OCR, không cần Triton, vẫn đóng gói DIP / kiểm toàn vẹn / quét huỷ / hết hạn
khai thác / quét virus); gói Windows `dev/luutru/` (`dong-goi-may-chu.ps1` → `dist/luutru-server-win-*.zip`, trong gói
`cai-dat.ps1` / `giam-sat.ps1` / `sao-luu.ps1` / `nhap-goi.ps1`); Linux `infra/compose/luutru/` (Dockerfile, compose,
`cai-dat.sh`, `sao-luu.sh`; `dong-goi-may-chu.ps1 -Linux` → tar.gz kèm image `docker save`, không kèm mã nguồn); TLS LAN:
Caddy `tls internal`, chứng chỉ gốc + `cai-chung-chi-goc.cmd` (certutil -user) phát ở `/tai-ve` (cả cổng 80), desktop
reqwest thêm `rustls-tls-native-roots` (updater reqwest 0.13 vốn dùng platform-verifier), màn đăng nhập nhận ra lỗi
UnknownIssuer; `docs/CAI-DAT-MAY-CHU-LUU-TRU.md`. CSDL tạo với ICU `vi-VN` (Windows cần PostgreSQL ≥ 15).
Kho tài liệu là THƯ MỤC (`DIGITEXT_PDF_STORE_DIR`, FsObjectStore) ở cả hai bộ cài — bỏ MinIO (image MinIO đã rời Docker
Hub / quay.io 11/2026; máy chủ một máy không cần; MinIO Windows từng chết theo stderr).

Đã chạy thử: migrate trên Postgres trống (27 tệp) và đường đánh dấu; worker chế độ lưu trữ; parser PowerShell cho 5
script; phần sinh Caddyfile + `caddy validate` (nội bộ và tên miền); compose Linux trọn luồng trên Docker Desktop (cài, chạy lại,
HTTPS bằng chứng chỉ gốc phát ra, cổng 80 chỉ phát chứng chỉ, nhập gói, tra cứu, bản đọc watermark, sao lưu CSDL + ghi sổ).
CHƯA thử: `cai-dat.ps1` trọn vẹn (đăng ký dịch vụ thật — làm trên máy Windows sạch ở GĐ5), máy trạm tin chứng chỉ tự cấp.
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

### GĐ4 — Tra cứu và 4 module trên dữ liệu nhập (8–10 ngày) — XONG 5/11/2026 (nhánh `feat/ban-luu-tru`)

Chạy thử trên Postgres trống (0001–0026) với 3 gói Diễn Xá đã nhập: tìm không dấu nhiều từ trên mục lục và lớp chữ (có
trích đoạn, số trang), lọc năm / chế độ; bản đọc watermark + nhật ký; khách bị chặn đọc tài liệu hạn chế và khi phí đọc > 0;
DIP 2 tài liệu nhập → worker dựng xong (pkgcheck chỉ còn lỗi PDF/A của chính tệp gốc Diễn Xá); kiểm toàn vẹn bắt được PDF
sửa 1 byte (blake3) và zip sửa 1 byte (sha256); tiêu huỷ hồ sơ THBQ 5 năm → 6 tài liệu huỷ, zip gói xoá theo quyết định,
tra cứu ẩn; huỷ 1 tài liệu của gói còn 67 → giữ zip, ghi nhật ký `giu_goi_aip`.

Chưa làm: vai trò mặc định riêng cho bản Lưu trữ (dùng vai trò gốc: khách = customer, cán bộ = admin); gán hộp.

**Tìm theo tên loại văn bản — XONG 26/11/2026.** Mục lục gói AIP ghi MÃ TT05 (gói Diễn Xá: 149 văn bản mã `32` = "Khác",
1 văn bản mã `22` = "Công văn"), mà `cl_vb_tim` nhận chính cột `loai_vb` nên gõ "công văn" ra 0 kết quả dù cột hiển thị
vẫn hiện đúng tên qua `cl_loai_van_ban`. Nay `GET /tra-cuu` dịch từ tìm sang danh sách mã trước (bảng loại chỉ vài chục
dòng) rồi so bằng `=` trong câu chính, nên câu chính vẫn dùng được chỉ mục trigram trên `cl_vb_tim` — không phải LIKE
trên tên loại của từng dòng. Đo trên 3 gói Diễn Xá đã nhập: "cong van" và "công văn" đều ra 1 (trước ra 0), "khac" ra
149, "quyet dinh" ra 0 (đúng — không có loại đó), không có từ tìm ra 150. Dòng khớp được đánh `khop = "muc_luc"`. Lớp chữ gói Diễn Xá
là OCR cũ sai bảng mã ("Di6n X6") — tìm nội dung chỉ tốt với gói có lớp chữ chuẩn.
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

Phần đã chạy 9/11/2026 (không cần máy sạch): `dong-goi-may-chu.ps1 -Linux` ra đủ hai gói (Windows 42 MB, Linux 58 MB);
gói Windows giải nén chạy TẠM trên máy dev (không đăng ký dịch vụ): migrate 28 tệp, `giam-sat.ps1` giữ API / worker /
Caddy.exe, Caddy tự cấp chứng chỉ + xuất `chung-chi-goc.crt` / `cai-chung-chi-goc.cmd`, `nhap-goi.ps1` nhập 3 gói Diễn Xá,
`/tai-ve` qua cổng 80, HTTPS bằng chứng chỉ gốc. Sửa: chữ Việt gửi `psql.exe` đi qua tệp UTF-8 (`-f`) thay `-c`; tên gói
theo phiên bản desktop. CÒN LẠI (cần máy Windows sạch có quyền quản trị — Windows Sandbox chưa bật trên máy dev — và một
máy trạm có người thao tác): `cai-dat.ps1` thật với PostgreSQL bộ cài chuẩn, `sao-luu.ps1` với pg_dump.exe, máy trạm cài
chứng chỉ gốc + đăng nhập + tự cập nhật (updater dùng platform-verifier: kiểm có vướng thu hồi chứng chỉ như curl/schannel).
Máy ảo Windows sạch + máy Linux Docker: cài máy chủ theo tài liệu, tải bộ cài từ `/tai-ve`, nhập gói Đầm Hà / Bắc Giang,
tra cứu, yêu cầu khai thác → duyệt → tải bản sao, kiểm toàn vẹn, tiêu huỷ thử, phát hành bản cập nhật và xem máy trạm tự cập nhật.

**Dựng lại và soát lần hai, 26/11/2026** (vẫn trên máy dev — gói đứng yên từ 9/11 trong khi code đi thêm 3 migration và
cả bộ giao diện 0.25). Làm trước để buổi dựng máy ảo sạch không bị đốt vào lỗi dựng gói:

- Gói máy chủ dựng lại: `dist\luutru-server-win-0.25.15-20261126.zip` **43,4 MB** (api 25,7 MB + worker 14,6 MB +
  caddy 43,2 MB, 29 migration). `cargo build --release -p digitext-api -p digitext-ocr` sạch.
- Bộ cài máy trạm bản Lưu trữ **mắc ở 0.24.0 từ 4/11** → dựng lại 0.25.15 (`release-desktop.ps1 -Edition luutru
  -Version 0.25.15 -SkipBump`, không scp đi đâu). Trước khi lên máy sạch thì dựng lại theo phiên bản mới nhất, kẻo
  cơ quan nhận bộ cài lạc hậu hàng chục bản.
- Chạy thử trọn luồng nhập gói với Postgres container tạm: migrate **29/29 tệp** sạch trên CSDL trống →
  `bootstrap-superadmin` → tạo kho `LUUTRU` → `nhap-goi` 3 gói Diễn Xá: **1 phông · 3 hồ sơ · 150 văn bản ·
  150 tài liệu · 3 gói · 123 trang lớp chữ**, nhận cả 3 gói dù kiểm gói có lỗi. Khớp lần chạy 9/11.
- Câu `CREATE DATABASE … LOCALE_PROVIDER icu ICU_LOCALE 'vi-VN' LOCALE 'C'` của `cai-dat.ps1` đo trên PostgreSQL 16:
  chạy được, `datctype` ra `C` nhưng `lower()`/`upper()` tiếng Việt **vẫn đúng** (ICU lo) và sắp xếp ra đúng thứ tự
  An · Ánh · Ăn · Âm · Bình · Dung · Đông. Không phải đổi câu.

Bốn lỗi sửa luôn trong đợt soát này — đều là loại chỉ lộ ra trên máy sạch:

1. **Ổ đĩa mặc định không có.** `-KhoDir D:\…` / `-SaoLuuDir E:\…` mà máy chủ cơ quan chỉ có C: thì `New-Item` chỉ
   báo lỗi rồi đi tiếp, `Resolve-Path` trả rỗng, `api.env` nhận `DIGITEXT_PDF_STORE_DIR=` trống và API chết tận bước 4
   với lý do không liên quan. Nay chặn ngay đầu script, nói rõ phải đổi bằng tham số nào.
2. **PostgreSQL cài rồi nhưng dịch vụ chưa chạy** thì mọi lỗi phía dưới hiện thành "sai mật khẩu postgres". Nay gọi
   `pg_isready.exe` trước và chỉ đường `services.msc`.
3. **`[int]$banPg`** chết bằng dòng .NET nếu `psql --version` không ra số (psql.exe thiếu thư viện VC++). Nay kiểm regex.
4. **Gói mang theo cả bộ cài cũ.** `dong-goi-may-chu.ps1` chép mọi `*.exe` trong `updates-luutru\` → gói 26/11 có cả
   0.24.0 nặng 12,8 MB mà `latest.json` không trỏ tới; kỹ thuật viên mở `/tai-ve` cài nhầm được. Nay chỉ chép đúng
   bản `latest.json` nêu, thiếu tệp thì dừng.

Thêm hai cảnh báo: kho và sao lưu **cùng ổ** (Điều 30 TT05 muốn phương tiện riêng), và sao lưu đặt ở **thư mục mạng**
`\\máy\...` — việc 01:00 chạy bằng SYSTEM nên sẽ không vào được.

**Tổng ước: khoảng 5 tuần.** GĐ1 và GĐ3 cũng có ích cho bản thi công (cập nhật theo máy chủ, lệnh migrate, nhập gói).

### GĐ6 — Lưu trữ hiện hành, Điều 19 TT 05/2025 (26/11/2026, nhánh `feat/ban-luu-tru`)

Bối cảnh: so với Digifile 2023 bản TT05 (`D:\Digifile`, khảo sát 26/11), DigiText Lưu trữ thiếu nghiệp vụ **lưu trữ của cơ
quan** — trước hết là nộp hồ sơ vào lưu trữ cơ quan và danh mục hồ sơ. Anh Hiếu chốt: bản Lưu trữ phục vụ cả lưu trữ
hiện hành lẫn lưu trữ lịch sử. Kế hoạch đầy đủ 4 đợt (Lưu trữ hiện hành → Bảo quản Điều 29–31 → Báo cáo TT05 + mượn trả +
kho → PL VI + biểu mẫu + quyền theo phông) ở tệp kế hoạch phiên làm việc; đợt 1 đã xong.

Chốt nghiệp vụ (26/11): hạn nộp **60 ngày lịch kể từ ngày kết thúc hồ sơ** (chỉ ghi năm → 31/12, ghi tháng → cuối tháng);
**khoá ngay khi nộp** (rút phiếu được khi chưa tiếp nhận); **một danh mục / phông / năm, mỗi phòng ban soạn phần mình**;
**một người một phòng ban, một phông**; nhắc 15/7/1 ngày cho người lập và cả phòng, quá hạn nhắc theo tuần, chỉ từ năm
`DIGITEXT_LT_NHAC_TU_NAM` (mặc định năm hiện tại − 1); đơn vị nộp còn đọc hồ sơ 360 ngày sau tiếp nhận (gồm PDF); chỉ bản
Lưu trữ có menu, bản thi công vẫn có route.

Đã làm:
- **Dữ liệu** `migrations/0031_luu_tru_hien_hanh.sql`: `lt_nop_luu`, `lt_nop_luu_ho_so`, `lt_danh_muc`, `lt_danh_muc_dong`;
  `cl_ho_so` + `nop_luu_trang_thai | nop_luu_id | khoa_luc` (máy chủ sinh) + `dm_dong_id` (chỉ trực tuyến); `users.phong_ban_id`;
  `notifications.ref` + chỉ mục duy nhất; hàm `cl_date_end_of()`; view `cl_ho_so_v` dựng lại (có `ngay_kt_cuoi`, `dm_ma_dong`,
  `nop_luu_ma_phieu`). 6 quyền mới `luutru.submit | receive | unlock`, `danhmuc.edit | submit | approve` (`Permission::ALL` 70,
  `permissions.ts` cùng lúc); `user` không có `receive` / `danhmuc.submit`. Vai trò mặc định bản Lưu trữ `roles::ensure_luu_tru`:
  `nhan_vien_phong_ban`, `can_bo_luu_tru`, `lanh_dao` (sửa được ở Vai trò và quyền) — đóng nợ GĐ4.
- **Máy chủ**: `digitext-db/nop_luu.rs`, `danh_muc_ho_so.rs`; khoá sửa trong `apply_inner`; phạm vi phòng ban `PhamVi` ở
  `/cl/ho-so`, `/tra-cuu`, `/cl/sync` pull; `routes/nop_luu.rs`, `routes/danh_muc_ho_so.rs`; biên bản generic GNL / SNL / DMH
  (bảng kẻ viền thật trong PDF — `FormBlock::Table`, DOCX theo); worker `nhac_nop_luu`. Test `chinh_ly/tests_luu_tru.rs` 3/3
  (khoá + rút + tiếp nhận; phạm vi + 360 ngày + nhắc không lặp + pull theo phòng; danh mục trọn luồng). Kịch bản API trọn luồng
  (super admin · nhân viên phòng A · cán bộ LT) chạy xanh trên Postgres container tạm.
- **Desktop 0.25.18**: nhóm menu Lưu trữ hiện hành, ba màn `features/luutru/`, ô Dòng danh mục + khoá ở `HoSoForm`, nộp lưu
  từ `LapHoSoView`, ô Phòng ban ở Người dùng, KPI + lối tắt ở Tổng quan, 3 mẫu biên bản ở sổ; Tauri `lt_get`/`lt_call`,
  `store::khoa_nop_luu` báo ngay khi sửa hồ sơ đã nộp (kể cả offline), session mang phòng ban (SQLite 26).

Chưa làm / cần anh: ảnh màn Digifile để so bố cục; mẫu biên bản giao nhận và sổ nộp lưu theo phụ lục TT05 (hiện là mẫu
generic); chạy thử trên máy sạch cùng GĐ5 (nộp lưu trên hồ sơ **lập mới** — hồ sơ nhập gói là `archived`, không đụng).
Nâng cấp máy chủ đang chạy: `digitext-api migrate` áp 0031 lúc bảo trì (DROP/CREATE view, API lỗi vài giây), rồi gán phòng ban
cho nhân viên ở Quản trị › Người dùng.

### GĐ7 — Bảo quản đủ Điều 29–31 + quét trùng Điều 48 (26/11/2026, nhánh `feat/ban-luu-tru`)

Đợt 2 của kế hoạch đối chiếu Digifile. Anh chưa trả lời câu (9)–(14) nên làm theo đề xuất: không có PDF Mẫu 01–04 chính thức → suy từ
Điều 29–31; "đơn vị" trong kế hoạch = phông của dự án; quyền duyệt riêng `bk.approve` (người duyệt khác người lập, super admin ngoại
lệ); trùng mục lục chỉ vào danh mục "xem xét", không tự đánh `trung_thua`; xoá bản sao lưu cũ chỉ nhắc + ghi nhận; bản thi công có đủ 5 tab.

Đã làm:
- **0032**: `bk_ke_hoach` + `bk_ke_hoach_tep` (ảnh chụp tệp chốt lúc duyệt), `bk_loi`, `bk_lich_sao_luu` (3 dòng seed), `bk_phuong_tien`,
  `bk_su_co`; `integrity_checks.nguon/ke_hoach_id`; `backup_runs` thêm 10 cột (lịch, phương tiện, đường dẫn, dung lượng, mã băm, thời gian,
  người, đã xoá). **0033**: `disposal_entries.phat_hien/dup_ho_so_id`, chỉ mục blake3 và mục lục.
- **Máy chủ**: `digitext-core::bao_quan` (toán + test), `digitext-db::bao_quan`, `routes/bao_quan.rs`, `disposal::f_trung_blake3/f_trung_muc_luc`
  (vào sweep API và worker 24 h), worker `bao_quan_worker.rs` (kế hoạch 1 h, nhắc lịch 1 h thay cảnh báo 26 h cứng, nhắc phương tiện 24 h),
  Mẫu 01–04 mở rộng, `sao-luu.ps1/.sh` ghi đủ cột và đánh dấu bản cũ đã xoá (tự nhận máy chủ chưa áp 0032).
- **Desktop 0.25.19**: Bảo quản 5 tab, `api/baoquan.ts`, Tauri `bq_get/bq_call`, cột Phát hiện ở Tiêu huỷ, KPI ở Tổng quan, 4 mẫu biên bản
  thêm ô tuỳ chọn.
- **Kiểm chứng**: `cargo test -p digitext-core` (36), kịch bản `dev/luutru/thu-bao-quan.ps1` xanh trên Postgres tạm: lập → xem trước → trình →
  duyệt chốt 3 tệp → chạy (3 lỗi vì kho trống) → hoàn thành → khắc phục → kiểm lại lỗi → bỏ qua → kết luận điền sẵn; `/ops/integrity/run`
  không mở thêm dòng lỗi; lịch chưa có / đúng hạn / trễ 22 h, 16 bản giữ 14 → cần xoá 2 → đánh dấu; phương tiện sắp hết 100 ngày;
  sự cố `SC-2026-001` gắn bản sao lưu, prefill Mẫu 04, báo quản trị; sweep ra 1 trùng blake3 + 1 trùng mục lục, lần hai 0.

Chưa làm / cần anh: ảnh màn Digifile (Kế hoạch kiểm tra, Phương tiện, Sự cố) để so bố cục; PDF Mẫu 01–04 chính thức nếu có (hiện suy từ
điều luật); worker chưa chạy thử thật (cùng hàm băm với API, đã kiểm qua API). Nâng cấp máy chủ: `migrate` áp 0032 + 0033 (0033 cần 0026
trước), rồi vào Bảo quản › Sao lưu sửa lịch cho khớp giờ chạy script, Phương tiện khai ổ sao lưu kèm hạn độ bền.

### GĐ8 — Báo cáo thống kê TT05, mượn – trả có duyệt, kho / giá sức chứa (27/11/2026, nhánh `feat/ban-luu-tru`)

Đợt 3 của kế hoạch đối chiếu Digifile (nhóm 3A → 4A → 4B). Anh chưa trả lời câu (15)–(16) nên làm theo đề xuất: không có mẫu Excel / PDF
18 báo cáo của Digifile → cột suy từ dữ liệu và điều luật, PDF có đầu văn bản hành chính + khối ký "Người lập báo cáo / Lãnh đạo cơ quan";
bản xuất là tệp tải về, KHÔNG lưu vào sổ biên bản; quyền duyệt mượn riêng `loan.approve` (tầng quản trị, như `bk.approve`).

Đã làm:
- **0034** mượn – trả có duyệt (giữ bảng `cl_muon_tra`, thêm `nhom_id`, `so_phieu`, duyệt / giao / gia hạn / nhắc; phiếu cũ = đã giao,
  không số), **0035** sức chứa giá / kho, **0036** chỉ mục `audit_log` theo dự án.
- **Máy chủ**: 19 báo cáo (`digitext-db::bao_cao`, `routes/bao_cao.rs` — bản Lưu trữ thấy 18, `thu_nop` chỉ thi công), Excel / PDF máy
  chủ dựng; mượn – trả: `digitext-core::chinh_ly::muon_tra` (bảng chuyển + test), `routes/chinh_ly/muon_tra.rs` (lập nhiều hồ sơ, duyệt
  lẻ / cả phiếu, từ chối, giao, gia hạn ≤ 2 lần giữ hạn gốc, trả, huỷ, phiếu PDF), `sync::muon_tra_chan` chặn đường generic / offline,
  nhắc hạn trong API mỗi giờ; kho / giá: `routes/chinh_ly/kho_gia.rs` (tổng hợp, hồ sơ theo kho / giá + Excel, hồ sơ chưa hộp, gán hộp
  cho hồ sơ nhập gói), nhắc `kho_day`.
- **Desktop 0.25.20**: Báo cáo › Thống kê TT05 (cả hai bản), Mượn – trả viết lại, Kho và giá có sức chứa + tab Gán hộp, KPI mượn trả ở
  Tổng quan Lưu trữ.
- **Kiểm chứng**: `cargo test` core 34 / api 46 / db 27; kịch bản `dev/luutru/thu-dot-3.ps1` xanh trên Postgres tạm (nhân viên lập phiếu
  2 hồ sơ → không duyệt / không giao ngay được → cán bộ duyệt 1, từ chối 1 có lý do → giao → gia hạn 2 lần, lần 3 bị chặn → lùi hạn,
  job nhắc gửi `loan_overdue` cho người lập + quản trị → trả; phiếu 2 duyệt cả phiếu, huỷ 1 dòng; giá 1–3 chứa 2 hộp = 66,7 % ≥ 60 %
  → `kho_day`; gán hộp 2 hồ sơ (1 chưa phòng ban) + 1 lỗi; 18 báo cáo JSON chạy, Excel / PDF phông và sổ mượn trả xem bằng mắt).
  Bẫy vá nhân tiện: `validate::normalize` ép `trang_thai` thành số với mọi bảng → mọi phiếu mượn tạo qua `/cl/rows` bị từ chối.

Chưa làm / cần anh: (15) mẫu Excel / PDF 18 báo cáo và ảnh màn Báo cáo của Digifile nếu có để chỉnh cột, khối ký; có cần lưu bản xuất
vào sổ biên bản không; (16) giữ `loan.approve` riêng hay gộp vào `loan.manage`. Còn lại của kế hoạch: Đợt 4 = 3B trang thể thức PL VI
+ sổ chuyển đổi (migration 0037 `chuyen_doi_giay`), 3C biểu mẫu PDF tại máy trạm, 4C quyền theo phông (0038 `user_phong`) — cần câu (17)–(19).

### GĐ9 — Chuyển đổi số → giấy (PL VI), biểu mẫu PDF tại máy trạm, phân quyền theo phông (27/11/2026, nhánh `feat/ban-luu-tru`)

Đợt 4 (cuối) của kế hoạch đối chiếu Digifile (3B → 3C → 4C). Anh chưa trả lời câu (17)–(19) nên làm theo đề xuất: trang thể thức suy từ
Mục 1 Phụ lục VI (Digifile cũng tự dựng), đặt SAU nội dung; bản in ký tay + đóng dấu, KHÔNG ký số lại bản ghép; số chuyển đổi theo cơ quan
(dự án) / năm `CD-{năm}-{stt:06}`; in HTML giữ làm "In nhanh" song song PDF một vài bản phát hành; phạm vi phông áp CẢ hai bản, người bị
giới hạn vẫn làm ngoại tuyến (bản sao chỉ còn phông được cấp — không chặn 403 như plan), tài khoản khách cũng gán được.

Đã làm:
- **0037** `chuyen_doi_giay`, **0038** `user_phong`.
- **Máy chủ**: `digitext-pdf::ghep` (ghép PDF + chân trang, có test), `digitext-db::{chuyen_doi, user_phong}`, `routes/chuyen_doi.rs`
  (kiểm chữ ký số lúc chuyển đổi bằng `digitext_sign_verify`, trang thể thức `manual`, lưu `{project}-chuyen-doi/{id}.pdf`, nhật ký
  `khai_thac.chuyen_doi`, sổ JSON / Excel, tải PDF), báo cáo #19 sổ chuyển đổi; phạm vi phông cắm vào mọi đường đọc mục lục / tra cứu /
  đọc tài liệu / pull offline / mượn trả / kho / báo cáo / thống kê / hệ thống hoá / nhập mục lục / đối soát; route `GET/PUT /users/{id}/phong`,
  `GET /cl/pham-vi-phong`; `ManualBlock` có serde + `PageBreak` + `Centered`.
- **Desktop 0.25.21**: Tra cứu › "In ra giấy" + hộp thoại chuyển đổi; màn Sổ chuyển đổi (cả hai bản); Người dùng › "Phạm vi phông";
  Kết xuất mục lục có bản PDF dựng tại máy trạm (mục lục hồ sơ, sản phẩm có bìa, mục lục văn bản, phụ lục tra tìm); Tauri `bc_call`, `pdf_manual`.
- **Kiểm chứng**: `cargo test` pdf (ghép 1 + 3 trang → 4, chân trang), kịch bản `dev/luutru/thu-dot-4.ps1` xanh trên Postgres tạm: giới hạn
  nvc ở phông A → /cl/ho-so, /cl/van-ban, catalog, tra cứu, bộ lọc, đọc tài liệu, pull offline, mượn trả, báo cáo, thống kê, lịch sử, gán
  hộp, nhập mục lục đều chỉ còn / chặn đúng phông; bỏ giới hạn thấy lại hết; chuyển đổi tài liệu A → `CD-2026-000001`, PDF = gốc + 1
  trang, lần hai 000002, sổ + Excel + nhật ký + báo cáo #19, người bị giới hạn không chuyển đổi tài liệu phông khác. Trang thể thức đã
  xem bằng mắt.

Chưa làm / cần anh: (17) mẫu PL VI chính thức để so câu chữ, có cần ký số lại bản ghép không (hiện ký tay + dấu); (18) bao giờ bỏ hẳn
in HTML (hiện song song); (19) phạm vi phông có cần áp cho kho tệp thi công (`documents` không gắn phông) và `/cl/rows/{table}/{id}` không.
Ngoài kế hoạch còn: nhãn hộp 2×2 ra PDF, cổng độc giả web, tài liệu ảnh / ghi âm, email / SMS (đã ghi "để sau" trong plan).

### GĐ10 — Đợt 5: dọn phần còn sót (cơ quan lưu trữ, lịch sử phông, bịt phạm vi phông, nhãn hộp PDF, nợ Digifile) (27/11/2026, nhánh `feat/ban-luu-tru`)

Anh bảo "làm đợt tiếp theo" sau khi 4 đợt xong → làm phần còn sót TRONG phạm vi đã chốt, không đụng nhóm "để sau" (cổng độc giả web,
ảnh / ghi âm, email / SMS).

Đã làm:
- **0039** `co_quan_luu_tru` (một dòng / dự án), `phong_lich_su` (phiên bản theo phông).
- **Cấu hình cơ quan lưu trữ** (Digifile B01): `GET/PUT /projects/{id}/co-quan`; dùng ở hạn nộp lưu / mốc nhắc / quyền sử dụng (SQL
  `COALESCE` theo dự án, worker nhắc theo mốc từng dự án), khối ký + đầu văn bản báo cáo TT05, trang thể thức chuyển đổi. Chưa khai = như
  trước (tên dự án, 60 / 15-7-1 / 360, "LÃNH ĐẠO CƠ QUAN"). **Mẫu sinh mã hồ sơ (B06) chưa đổi** — mã nằm ở ba nơi (view SQL, hệ thống
  hoá, desktop offline) và dữ liệu cũ đã mang mã; đổi cần anh chốt công thức (câu (20) dưới).
- **Lịch sử đơn vị hình thành phông có phiên bản** (B02): `GET/POST /cl/phong/{id}/lich-su`; desktop Danh mục › tab Phông › "Lịch sử phông".
- **Bịt phạm vi phông** (nợ 4C): `/cl/rows/*` (đọc / ghi / xoá / khôi phục, kể cả dời hồ sơ sang phông khác), `POST /cl/sync/push` (op ngoài
  phông bị từ chối `phong_scope`, không áp), `/access` (lập / nạp / danh sách), `/info/{id}/generate`, `/nop-luu/*`, `/danh-muc-ho-so`.
- **Nhãn hộp 2×2 ra PDF** (`ManualBlock::Labels2x2`), menu Kết xuất "Nhãn hộp (PDF)"; HTML giữ "In nhanh".
- **Nợ Digifile**: cột "Cơ quan ban hành" trong Excel 51 cột đọc được; ngôn ngữ `01;02` tách đúng; `digitext sign` báo ngay khi slot không
  có token (không chờ driver ~40 s). Mức độ tin cậy mặc định: TT02 không có trường này (chỉ `cl_van_ban.do_tin_cay` TT05) — không sửa.
- **Desktop 0.25.22** cả hai bản; Tauri `bc_call` kiểm quyền theo đường.
- **Kiểm chứng**: `cargo test` core / pdf / extimport / api / db; kịch bản `dev/luutru/thu-dot-5.ps1` (tự áp 0039): cấu hình cơ quan →
  hạn nộp 90 ngày phản ánh ở `ho-so-can-nop`, PDF báo cáo có cấp trên + chức vụ + người ký; lịch sử phông 2 phiên bản; nvc giới hạn
  phông A bị chặn ở `/cl/rows` (GET / PUT / POST / DELETE), push, khai thác, danh mục hồ sơ; PDF chuyển đổi mang tên cơ quan mới.

Chưa làm / cần anh: (20) công thức mã hồ sơ theo Phụ lục I (có thêm mã cơ quan lưu trữ ở đầu không, áp cho hồ sơ cũ thế nào); (21) lịch
sử phông có đưa vào EAD `bioghist` khi đóng gói không (gói dựng ở máy trạm từ SQLite, cần kéo thêm bảng). Vẫn còn ngoài kế hoạch: cổng
độc giả web, ảnh / ghi âm, email / SMS; GĐ5 máy sạch.

## Cần anh quyết

1. **Kết nối máy trạm ↔ máy chủ CĐT.** App hiện chỉ nhận `https` có chứng chỉ công khai (rustls không đọc kho chứng chỉ
   Windows); `http` chỉ cho localhost. Đề xuất: bản lưu trữ dùng kho chứng chỉ Windows (`rustls-platform-verifier`) +
   Caddy tự cấp chứng chỉ nội bộ, bộ cài máy trạm kèm chứng chỉ gốc → chạy trong LAN không cần tên miền. CĐT có tên miền
   thì dùng Let's Encrypt như OCR PC.
2. **Tiêu huỷ và WORM.** Gói AIP không xoá được, nhưng tiêu huỷ phải huỷ cả bản số. ĐÃ CHỐT 5/11/2026: chỉ đường "thực
   hiện quyết định tiêu huỷ" (có số quyết định, ghi nhật ký) được xoá zip, và chỉ khi đã huỷ hết tài liệu của gói; mọi
   đường khác vẫn chặn.
3. **Gói chưa ký số** (worker rơi về bản chưa ký khi thiếu): đề xuất nhận, đánh dấu "chưa ký" trên hồ sơ và báo cáo nhập.
4. **Thứ tự:** đề xuất GĐ1 → GĐ3 → GĐ4 → GĐ2 → GĐ5 — dựng và nhập thử trên máy chủ dev trước, đóng bộ cài sau cùng.
