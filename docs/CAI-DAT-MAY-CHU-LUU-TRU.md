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
