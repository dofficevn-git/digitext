# Cài máy chủ DigiText Lưu trữ

Tài liệu dành cho kỹ thuật viên của cơ quan chủ đầu tư. Sau khi cài xong, máy chủ:
- nhận gói AIP do đơn vị thi công số hoá bàn giao,
- cho cán bộ tra cứu, khai thác, bảo quản và tiêu huỷ tài liệu theo TT 05/2025/TT-BNV,
- phát bộ cài DigiText Lưu trữ cho các máy trạm trong cơ quan, và các máy trạm tự cập nhật từ chính máy chủ này.

Có hai cách cài, chọn theo hệ điều hành của máy chủ: **Windows** hoặc **Linux (Docker)**.

## 1. Chuẩn bị

| Hạng mục | Yêu cầu |
|---|---|
| Máy chủ | 4 nhân CPU, 16 GB RAM, Windows Server 2019+ / Windows 10–11 Pro, hoặc Linux 64-bit có Docker |
| Ổ kho tài liệu | Dung lượng ≈ 1,2 × tổng dung lượng gói AIP sẽ nhập (vd 2 TB gói → ổ 2,5 TB) |
| Ổ sao lưu | Ổ **khác** ổ kho, dung lượng tương đương ổ kho |
| Địa chỉ | **Tên miền** (vd `luutru.co-quan.vn`, cần mở cổng 80/443 từ Internet), hoặc **tên máy / IP cố định trong mạng nội bộ** (vd `192.168.1.20`) |
| Cổng | 443 (HTTPS), 80 (cấp chứng chỉ / phát chứng chỉ gốc) |

**Chỉ dùng trong mạng nội bộ:** máy chủ tự cấp chứng chỉ HTTPS. Mỗi máy trạm cần cài chứng chỉ gốc của máy chủ một lần (mục 4).

## 2. Cài trên Windows

1. Cài **PostgreSQL 16** bằng bộ cài chuẩn tại <https://www.postgresql.org/download/windows/>. Giữ các tuỳ chọn mặc định (cổng 5432) và **ghi lại mật khẩu tài khoản `postgres`**.
2. Giải nén `luutru-server-win-<phiên bản>.zip` vào một thư mục cố định, vd `C:\DigiText-LuuTru`.
3. Mở **PowerShell bằng quyền quản trị** (chuột phải → Run as administrator), rồi chạy:

   ```powershell
   cd C:\DigiText-LuuTru
   powershell -ExecutionPolicy Bypass -File .\cai-dat.ps1 -DiaChi 192.168.1.20 -KhoDir D:\DigiText-Kho -SaoLuuDir E:\DigiText-SaoLuu -TenCoQuan "Lưu trữ lịch sử tỉnh ..."
   ```

   Script sẽ hỏi hai việc: mật khẩu tài khoản `postgres`, và mật khẩu cho tài khoản quản trị đầu tiên (`quantri`).

   Tuỳ chọn thêm:
   - `-CongApi 8090` khi máy đã có phần mềm khác chiếm cổng 8080 (API chỉ nghe ở `127.0.0.1`, máy trạm không thấy cổng này).
     Script kiểm cổng trước khi đụng tới CSDL và báo tên tiến trình đang chiếm.
   - `-MatKhauPgTep <tệp>` và `-MatKhauQuanTriTep <tệp>`: đọc hai mật khẩu từ tệp thay vì hỏi, để cài tự động (mỗi tệp một dòng).
     Xoá hai tệp này sau khi cài xong.
4. Cuối cùng script báo **XONG** kèm địa chỉ tải bộ cài cho máy trạm.

Script làm những việc sau:
- Tạo cơ sở dữ liệu `digitext`, dùng sắp xếp tiếng Việt.
- Sinh cấu hình vào `conf\`, mật khẩu tạo ngẫu nhiên.
- Đăng ký 3 dịch vụ tự chạy khi mở máy: `DigiText-LuuTru-API`, `-Worker`, `-Caddy`. Kho tài liệu là thư mục
  `-KhoDir` (không dùng MinIO).
- Cập nhật cấu trúc cơ sở dữ liệu.
- Tạo tài khoản quản trị và kho lưu trữ đầu tiên.
- Mở tường lửa cổng 80/443.
- Đặt lịch sao lưu lúc 01:00 hằng đêm.

Chạy lại script nhiều lần cũng không sao: cấu hình và dữ liệu đã có được giữ nguyên.
Thông số đã chọn (địa chỉ, thư mục, cổng API) ghi ở `conf\may-chu.env`; lần chạy lại không cần nhập lại.

**Gỡ dịch vụ** (dữ liệu vẫn giữ): `.\cai-dat.ps1 -Go`.

**Nhật ký:** thư mục `logs\`, gồm `api.err.log`, `worker.err.log`, `caddy.err.log` và `sao-luu.log`.

## 3. Cài trên Linux (Docker)

1. Cài Docker Engine kèm plugin compose (<https://docs.docker.com/engine/install/>).
2. Giải nén `luutru-server-linux-<phiên bản>.tar.gz` lên máy chủ (gồm `cai-dat.sh`, `docker-compose.yml`, `sao-luu.sh`
   và image dựng sẵn `digitext-luutru-image.tar.gz`), vào thư mục vừa giải nén.
3. Chạy:

   ```bash
   sudo KHO_DIR=/srv/digitext-kho SAO_LUU_DIR=/mnt/saoluu TEN_CO_QUAN="Lưu trữ lịch sử tỉnh ..." ./cai-dat.sh 192.168.1.20
   ```

   Script nạp image, tải postgres / caddy từ Docker Hub, hỏi mật khẩu quản trị đầu tiên.

Các dịch vụ chạy trong `docker compose` gồm: postgres, api, worker, caddy (kho tài liệu là thư mục `KHO_DIR`). Cần cài `rsync` cho sao lưu kho. Chỉ Caddy mở cổng 80/443. Sao lưu chạy lúc 01:00 qua `/etc/cron.d/digitext-luutru`.

## 4. Máy trạm

1. Mở trình duyệt, vào `https://<địa chỉ>/tai-ve`. Nếu dùng mạng nội bộ mà trình duyệt cảnh báo chứng chỉ, vào `http://<địa chỉ>/tai-ve`.
2. **Chỉ áp dụng cho mạng nội bộ:** bấm **Tải tệp cài chứng chỉ**, bấm đúp tệp vừa tải, chọn **Yes** khi Windows hỏi. Không cần quyền quản trị máy. Nếu cơ quan có domain, có thể đẩy tệp `chung-chi-goc.crt` xuống mọi máy bằng Group Policy (Trusted Root Certification Authorities).
3. Tải và chạy bộ cài **DigiText Lưu trữ**. Ở màn đăng nhập, bấm **Máy chủ** và nhập `https://<địa chỉ>`.
4. Từ đây, các bản cập nhật tự đến từ máy chủ.

Bộ cài máy trạm nằm trong thư mục `updates\` (Linux: `updates/`) của máy chủ. Khi có bản mới, chép gói phát hành DigiText Lưu trữ vào thư mục đó, đè lên tệp cũ.

## 5. Nhập gói AIP

- **Gói ít, qua mạng:** trên máy trạm, vào **Nhập Hệ thống › Nhập gói AIP**.
- **Ổ cứng chứa bộ hồ sơ nghiệm thu, hàng trăm GB:** cắm ổ vào máy chủ, rồi chạy:
  - Windows: `.\nhap-goi.ps1 -ThuMuc F:\BoHoSoNghiemThu`
  - Linux: chép gói vào `goi-nhap/`, rồi chạy `docker compose exec api digitext-api nhap-goi --du-an LUUTRU --nguoi quantri /goi`

Có thể nhập lại nhiều lần: gói nào đã nhập sẽ được báo "đã có". Gói kiểm KHÔNG ĐẠT bị từ chối, trừ khi thêm `-NhanCaGoiLoi` (Linux: `--nhan-ca-goi-loi`). Gói có chữ ký số bị sửa luôn bị từ chối.

## 6. Vận hành

| Việc | Windows | Linux |
|---|---|---|
| Khởi động lại một dịch vụ | `Restart-ScheduledTask DigiText-LuuTru-API` | `docker compose restart api` |
| Sao lưu ngay | `.\sao-luu.ps1` | `./sao-luu.sh` |
| Cập nhật máy chủ | giải nén bản mới đè lên `bin\` và `migrations\`, chạy lại `cai-dat.ps1` | giải nén bản mới đè lên, chạy lại `./cai-dat.sh` |
| Cập nhật cấu trúc CSDL tay | nạp `conf\api.env`, chạy `bin\digitext-api.exe migrate` | `docker compose up migrate` |

**Khôi phục cơ sở dữ liệu** từ bản sao lưu:
- Windows: `pg_restore -d <DATABASE_URL> --clean <tệp .dump>`
- Linux: `docker compose exec -T postgres pg_restore -U digitext -d digitext --clean < tệp.dump`

Kho tài liệu khôi phục bằng cách chép ngược thư mục `kho` trong ổ sao lưu.

Màn **Lưu trữ số › Bảo quản và sao lưu** trên máy trạm hiện kết quả các lần sao lưu đêm và kết quả kiểm tra toàn vẹn tài liệu cùng gói AIP.

## 7. Lưu trữ hiện hành (từ bản 0.25.18)

Bản này thêm nhóm **Lưu trữ hiện hành** (Điều 19 TT 05/2025/TT-BNV): phòng ban lập hồ sơ theo **Danh mục hồ sơ** của năm,
**nộp lưu** vào lưu trữ cơ quan trong 60 ngày kể từ ngày kết thúc hồ sơ; lưu trữ cơ quan **tiếp nhận**, biên bản giao nhận
và sổ nộp lưu lập tự động.

Khi nâng cấp máy chủ đang chạy:
1. Chạy lại `cai-dat.ps1` (Windows) hoặc `./cai-dat.sh` (Linux) như mục 6. Bước cập nhật cấu trúc CSDL áp thêm tệp `0031`
   (dựng lại một bảng tra cứu — máy trạm có thể mất kết nối vài giây), nên làm ngoài giờ.
2. API tự tạo ba vai trò mẫu nếu chưa có: **Nhân viên phòng ban**, **Cán bộ lưu trữ**, **Lãnh đạo**. Sửa bộ quyền ở
   Quản trị › Vai trò và quyền.
3. Ở Quản trị › Người dùng, **gắn phòng ban** cho nhân viên các phòng (trước đó tạo phòng ban ở Lưu trữ hiện hành › Phông,
   phòng ban). Người có phòng ban chỉ thấy hồ sơ của phòng mình; cán bộ lưu trữ và lãnh đạo để trống.
4. Tuỳ chọn trong `conf\worker.env` (Linux: `worker.env`): `DIGITEXT_LT_NHAC_TU_NAM=2026` — chỉ nhắc hạn nộp cho hồ sơ kết thúc
   từ năm này (mặc định: năm hiện tại trừ 1), tránh dội nhắc lên hàng vạn hồ sơ cũ mới nhập.

## 8. Bảo quản đủ Điều 29–31 (từ bản 0.25.19)

Màn **Lưu trữ số › Bảo quản và sao lưu** có 5 tab: Tổng quan · Kế hoạch kiểm tra (Điều 29, lập – trình – duyệt – máy chủ tự kiểm, danh mục lỗi
và khắc phục, biên bản Mẫu 01) · Sao lưu (lịch ngày / tháng / 3 năm, sổ sao lưu, bản cũ cần xoá, Mẫu 02 / 03) · Phương tiện (hạn độ bền,
nhắc chuyển đổi trước 1 năm — Điều 30.4) · Sự cố (Điều 31, Mẫu 04).

Khi nâng cấp máy chủ đang chạy:
1. Chạy lại `cai-dat.ps1` / `./cai-dat.sh` như mục 6: cập nhật CSDL áp thêm `0032`, `0033` (không dựng lại bảng tra cứu, máy trạm không mất kết nối).
2. Vào **Bảo quản › Sao lưu**: ba lịch mặc định đã có (gia tăng kho hằng ngày, đầy đủ CSDL hằng ngày, đầy đủ 3 năm làm tay). Sửa chu kỳ /
   độ trễ cho phép / số bản giữ cho khớp cách cơ quan chạy sao lưu. Script `sao-luu.ps1` / `sao-luu.sh` đêm tự ghi vào lịch cùng loại và
   đánh dấu bản cũ đã xoá; sao lưu 3 năm ra phương tiện riêng thì bấm **Ghi nhận sao lưu**.
3. Vào **Bảo quản › Phương tiện**: khai ổ sao lưu, băng từ… kèm **hạn độ bền** nhà sản xuất công bố. Hệ thống nhắc quản trị mỗi tháng khi còn
   dưới 1 năm.
4. Quyền mới **Duyệt kế hoạch kiểm tra** (`bk.approve`) tự thêm cho vai trò quản trị, lãnh đạo; người lập kế hoạch không tự duyệt được.
5. Tuỳ chọn thử nhanh: `DIGITEXT_BQ_GIAY=60` trong `worker.env` rút chu kỳ việc nền bảo quản xuống 60 giây (chỉ dùng khi thử).

## 9. Báo cáo TT05, mượn – trả có duyệt, kho / giá (từ bản 0.25.20)

- **Báo cáo › Thống kê TT05**: 18 báo cáo theo Điều 44.3b (mục lục, tài liệu số, bảo quản, khai thác, kho), lọc theo phông / năm / thời
  gian, xuất Excel hoặc PDF (có đầu văn bản và khối ký). Cần quyền xem số liệu (`stats.view`).
- **Kho lưu trữ › Mượn – trả**: phiếu mượn nhiều hồ sơ, **chờ duyệt → duyệt (từng hồ sơ hoặc cả phiếu) → giao → trả**; từ chối / huỷ có lý
  do; gia hạn tối đa 2 lần; máy chủ nhắc người lập và quản trị dự án khi còn 3 ngày đến hạn và mỗi tuần khi quá hạn. Phiếu cũ (một hồ sơ,
  giao luôn) vẫn hiện, không có số phiếu.
- **Kho lưu trữ › Kho và giá**: giá có **sức chứa** (trống = độ dài dải hộp) và **ngưỡng cảnh báo** (mặc định 90 %); đạt ngưỡng máy chủ báo
  quản trị dự án mỗi tuần. Tab **Gán hộp** xếp hộp cho hồ sơ nhập từ gói AIP (chọn phông, phòng ban chủ hộp, nhập số hộp hoặc dán từ bảng tính).

Khi nâng cấp máy chủ đang chạy:
1. Chạy lại `cai-dat.ps1` / `./cai-dat.sh` như mục 6: cập nhật CSDL áp thêm `0034`, `0035`, `0036` (không dựng lại bảng tra cứu; `0036` tạo chỉ
   mục không khoá bảng).
2. Quyền mới **Duyệt phiếu mượn** (`loan.approve`) tự thêm cho vai trò quản trị, lãnh đạo (cán bộ lưu trữ gốc quản trị cũng có). Muốn nhân viên
   quầy chỉ lập và giao mà không duyệt thì bỏ quyền này ở vai trò của họ.
3. Vào **Kho và giá › Giá**, sửa từng giá: khai sức chứa và ngưỡng cảnh báo nếu khác mặc định.
4. Tuỳ chọn thử nhanh: `DIGITEXT_MT_NHAC_GIAY=60` trong `conf\api.env` rút chu kỳ nhắc mượn trả / giá đầy xuống 60 giây (chỉ dùng khi thử).

## 10. Chuyển đổi số → giấy, phạm vi phông, biểu mẫu PDF (từ bản 0.25.21)

- **Tra cứu › In ra giấy** (cán bộ lưu trữ): chuyển đổi một tài liệu số sang giấy theo Điều 13–16 TT 05/2025: hệ thống kiểm chữ ký số của
  bản gốc, ghép trang thể thức "TÀI LIỆU LƯU TRỮ CHUYỂN ĐỔI" (Phụ lục VI) vào cuối, đánh chân trang từng trang, cấp số `CD-năm-số` và ghi
  **Lưu trữ số › Sổ chuyển đổi** + nhật ký khai thác. Bản in có giá trị khi người có thẩm quyền ký và đóng dấu lên trang thể thức.
- **Quản trị › Người dùng › Phạm vi phông**: giới hạn một tài khoản (quản trị dự án, nhân viên, khách) chỉ thấy một số phông của dự án;
  không chọn gì = thấy mọi phông như trước. Áp cho mục lục, tra cứu, đọc tài liệu, mượn trả, kho, báo cáo và bản sao ngoại tuyến.
- **Kết xuất mục lục**: các bản in mục lục hồ sơ, sản phẩm có bìa, mục lục văn bản, phụ lục tra tìm nay ra **PDF dựng tại máy trạm** (chạy
  cả ngoại tuyến); bản HTML cũ vẫn còn dưới tên "In nhanh … (HTML)".

Khi nâng cấp máy chủ đang chạy:
1. Chạy lại `cai-dat.ps1` / `./cai-dat.sh` như mục 6: cập nhật CSDL áp thêm `0037`, `0038` (nhanh, không khoá bảng).
2. Máy chủ cần font TTF (`DIGITEXT_CERT_FONT`, bộ cài đã đặt) để dựng trang thể thức; thiếu font API trả lỗi 503 khi chuyển đổi.
3. Không có việc nền mới.

## 11. Cơ quan lưu trữ, lịch sử phông, nhãn hộp PDF (từ bản 0.25.22)

- **Tổng quan › Cơ quan lưu trữ** (quản trị dự án): khai mã cơ quan lưu trữ (Phụ lục I), tên / cơ quan cấp trên / địa chỉ, loại hình
  (lưu trữ cơ quan hay lưu trữ lịch sử), người ký và chức vụ, hạn nộp lưu, mốc nhắc, quyền sử dụng sau nộp. Giá trị này in lên đầu và khối
  ký của báo cáo TT05, trang thể thức chuyển đổi, và quyết định hạn nộp lưu / nhắc hạn. Chưa khai thì phần mềm dùng tên dự án và 60 ngày /
  nhắc 15-7-1 / 360 ngày như trước.
- **Danh mục › Phông › Lịch sử phông**: lịch sử đơn vị hình thành phông và lịch sử phông theo Điều 4, 22; mỗi lần nộp bổ sung lập một phiên
  bản mới, bản cũ giữ nguyên. Chỉ khi trực tuyến.
- **Phạm vi phông** nay chặn cả sửa / xoá từng dòng, đẩy thay đổi ngoại tuyến, yêu cầu khai thác, cung cấp thông tin, nộp lưu và danh mục hồ sơ.
- **Kết xuất mục lục › Nhãn hộp (PDF)**: 4 nhãn một trang A4, có viền để cắt.

Khi nâng cấp máy chủ đang chạy:
1. Chạy lại `cai-dat.ps1` / `./cai-dat.sh` như mục 6: cập nhật CSDL áp thêm `0039` (hai bảng mới, không khoá bảng cũ).
2. Không có việc nền mới; worker nhắc nộp lưu tự đọc mốc nhắc mới của từng cơ quan.
