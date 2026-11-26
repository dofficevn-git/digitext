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
