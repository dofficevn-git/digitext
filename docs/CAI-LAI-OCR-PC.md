# Cài lại OCR PC từ đầu — lịch sử cài đặt + quy trình dựng lại (4/11/2026)

Viết để **tự cài lại toàn bộ máy chủ OCR PC** trên ổ NVMe mới mà không cần nhớ gì thêm. Bổ sung cho
`docs/CHUYEN-MAY-OCR-PC.md` (runbook chuyển máy, nhật ký 27/10–3/11); không lặp lại nội dung đó, chỉ dẫn chiếu.

## 0. Vì sao phải cài lại

4/11/2026: ổ hệ thống **C: NVMe Kingston SNV3S2000G 2 TB** có 2 sector hỏng (LBA 2385502240 và 2385515328, vị trí ~1,22 TB),
cả hai nằm trong `C:\Users\ADMIN\AppData\Local\Docker\wsl\disk\docker_data.vhdx` (đĩa ảo Docker 285 GB). SMART (`smartctl -a /dev/sdb`):
tắt không an toàn **194/270** lần bật; 31,8 triệu lỗi đọc không cứu được; ô dự phòng còn 90 %; đã ghi 42 TB/1.280 TB;
8.675 giờ chạy; chip điều khiển 67 °C. Windows vẫn báo "Healthy". Máy ảo Docker thử đọc lại hai sector đó vô hạn →
C: bận 100 %, 90 ms/lần đọc (từ ~2/11, lúc tar `pla_kie_log.ldf` treo). **Nguyên nhân khả dĩ: mất điện đột ngột lặp lại**,
không phải do phần mềm. Ổ còn bảo hành (lắp 5/2026) — mang kèm `cau-hinh\smart-nvme-C.txt` trong bộ sao lưu.

Gỡ tạm vòng đọc lặp (không sửa được ổ): `wsl --shutdown` rồi mở lại Docker Desktop (DigiText ngắt ~3 phút; `mssql_container`
có `--restart=no` phải `docker start` tay, rồi chạy lại `doc_extract_pla.exe`).

## 1. Máy đang có gì (kiểm kê 4/11/2026)

**Phần cứng:** ASUS TUF GAMING X870-PLUS WIFI, AMD Ryzen 9 9950X, 62 GB RAM, RTX 5070 Ti 16 GB (driver 610.88),
C: NVMe Kingston 2 TB (hỏng), D: HDD HGST HUS728T8TALE600 8 TB CMR (kho + sao lưu, 5,3 TB đã dùng), F: Seagate One Touch 1 TB USB (cầm tay).
Mạng: LAN 192.168.100.10/24 sau FortiGate 300D `pla-fw` 192.168.100.1; IP công cộng 113.190.254.219; Tailscale 100.122.226.63.

**Windows 11 Pro 26300** cài 16/5/2026, tài khoản `ADMIN` (không auto-logon). Feature bật: Hyper-V, VirtualMachinePlatform,
WSL, IIS-WebServer. Phần mềm (winget): Docker Desktop 29.6.1 (WSL 2.7.10, kernel 6.18), Git 2.55, Node 22.23.2 + npm 10.9.8 + pm2 7.0.3,
Python 3.14.7, Rustup (cargo/rustc 1.99.1), VS Build Tools 2022 + 2026, Poppler 25.07 (`pdftoppm`), Tailscale 1.104.4,
smartmontools 7.5, 7-Zip, RustDesk, WPS, Zoom, 360 Total Security (nghi gây "Access is denied" với schtasks/cargo — nên gỡ),
OpenSSH Server (capability), Caddy 2.11.7 tại `C:\DigiText-Caddy\caddy.exe` (service `DigiText-Caddy`),
cloudflared tại `C:\Program Files (x86)\cloudflared`, SQL Server 2025 Express (`MSSQL17.SQLEXPRESS`, chỉ có CSDL hệ thống — không hệ nào dùng).

**Các hệ thống đang chạy trên máy** (thứ tự cài theo thời gian):

| Hệ | Cài khi | Ở đâu | Chạy bằng | Cổng / tên miền |
|---|---|---|---|---|
| IIS: `DGT_Base` (digitext.vn), `DGT_Dmaster` (dmaster.doffice.vn), `DGF_PhuongDongTrieu` :8081, `DGF_PhongDKKTCOTO` :8082, `DGF_UBNDTinhQuangNinh` :8084 | trước 8/2026 | `D:\DULIEU_66\DigiApp\<site>` (app pool không managed runtime = .NET Core/ASP.NET Core) | IIS W3SVC | 80 (host header), 8081/8082/8084 |
| Cloudflared tunnel chính (`Cloudflared`) | 8/2026 | token `C:\ProgramData\cloudflared\token`; config cũ `C:\backup-cloudflared`, `systemprofile\.cloudflared` | service | đưa IIS/web ra ngoài (trước 2/11 cả `digitext.vn`, `www`) |
| n8n | 20/8/2026 | `C:\n8n\app` (node_modules\n8n), dữ liệu `C:\n8n\.n8n`, `ecosystem.config.js`; tunnel `C:\n8n\cloudflared\config.yml` | pm2 (`pm2 resurrect` từ `C:\Users\ADMIN\.pm2\dump.pm2`) + service `cloudflared-n8n` | 5001 |
| ct5d (smart-apartment) | 9/2026 | mã `C:\srv\smart-apartment-ct5d`; Docker `ct5d_admin_prod`, `ct5d_api_prod`, `ct5d_postgres_prod` (pgvector:pg15), `ct5d_minio_prod`; volume `ct5d_prod_*`; tunnel `C:\Users\ADMIN\.cloudflared\ct5d.yml` | docker compose + service `cloudflared-ct5d` | 3020/3021/5444/9030 |
| doc_extract_pla (web KIE cũ) | 8/2026 | `D:\doc_extract_pla` (Flask `doc_extract_pla.exe`, uploads 1 TB, static 443 GB, models); Docker `vllm-vnkie` (vllm v0.15.1, model `D:\doc_extract_pla\models\plakie`, GPU 0.75), `mssql_container` (mssql 2019, volume `doc_extract_pla_mssql_data`, 47 CSDL, `pla_kie`) | compose `docker-compose.vllm.yml` + `docker-compose.sqlserver.yml`; exe chạy tay (WMI Create) | 5002 web, 3000 vllm, 1433 |
| **DigiText** (bản chính từ 2/11) | 30/10–2/11/2026 | `C:\DigiText` (repo nhánh `feat/mlhs-tt05-mot-luong` + `target\release`), kho `D:\DigiText-KhoMayChu` (MinIO), Docker `digitext-dev-postgres-1` (postgres:16, volume `digitext-dev_pgdata`), `digitext-dev-clamav-1`, `digitext-ocr-fulltext-ocr-1` (ảnh tự build 33 GB, cổng **5012**), `digitext-vllm-vllm-1` (đang STOP, Qwen trong volume `digitext-vllm_hf_cache`); task SYSTEM `DigiText-MinIO`, `DigiText-API`, `DigiText-Worker`, `DigiText-Watchdog` (mỗi phút), `DigiText-Backup-CSDL` (01:00 → `C:\DigiText-Backup\csdl`, giữ 14); service `DigiText-Caddy` (443, TLS-ALPN-01, chứng chỉ `C:\DigiText-Caddy\data`) | xem §3 | 8080 API, 9000 MinIO, 9095 worker, 5012 OCR, 8003 vLLM, 443 `api.digitext.vn` + `digitext.vn`/`www` → IIS :80 |

Ngoài máy: DNS tại iNET (A `@`/`api` → 113.190.254.219, CNAME `www`), FortiGate VIP 443/80/5001/5002/5005 → 192.168.100.10 +
policy "Public web OCR"; Alienware 192.168.100.11 chạy Qwen (`:8003`) cho worker/OCR của OCR PC khi `digitext-vllm` tắt.

## 2. Bộ sao lưu để cài lại

**`D:\OCRPC-Backup-20261104\`** (script `C:\DigiText-ChuyenMay\bk-toan-bo.ps1` + `bk-sqlexpress.ps1`, nhật ký `nhat-ky*.log`,
`README-KHOI-PHUC.md` bên trong): `cau-hinh\` (docker inspect từng container/volume/ảnh, task Windows XML, services, firewall, IP,
IIS site/apppool XML, wslconfig, Docker settings, systeminfo, SMART); `csdl\` (pg_dump DigiText `-Fc`, `pg_dumpall` ct5d);
`mssql-bak\` (mọi CSDL SQL Server cũ, COPY_ONLY + VERIFYONLY); `sqlexpress-bak\` (master/model/msdb của SQL Express);
`volumes\` (tar: ct5d minio/uploads/postgres-raw, Qwen, cache OCR, cache vllm cũ, clamav, sqlserver-data cũ); `images\`
(`docker save` OCR Blackwell, ct5d admin/api); `tep\` (toàn bộ `C:\DigiText*`, `C:\n8n`, `C:\srv`, `C:\SQL2025`, `C:\inetpub`,
`inetsrv\config`, hồ sơ ADMIN gồm `.pm2 .ssh .docker .cloudflared Documents Desktop Downloads`, cloudflared ProgramData/systemprofile/exe).

Có sẵn ở D: từ trước: `D:\doc_extract_pla-backup\2026-11-02\` (46 .bak, app, ảnh vllm v0.15.1 + mssql 2019, tar volume trừ
`pla_kie_log.ldf`), `D:\DigiText-Backup\csdl\` (dump đêm + dump khẩn 4/11), `D:\DigiText-KhoMayChu` (kho MinIO 800 GB),
`D:\doc_extract_pla` (ứng dụng cũ + uploads/static), `D:\DULIEU_66\DigiApp` (các site IIS). Máy .127 còn dump chốt 2/11 + bộ mang sang.

Không có trong sao lưu (kéo lại được): ảnh Docker Hub `postgres:16`, `clamav/clamav:stable`, `minio/minio`, `pgvector/pgvector:pg15`,
`vllm/vllm-openai:v0.8.5`, `mcr.microsoft.com/mssql/server:2022-latest`; bộ cài phần mềm (winget); OneDrive (đám mây).

## 3. Quy trình cài lại trên ổ mới (thứ tự bắt buộc)

Ước 1 ngày làm việc, trong đó chép dữ liệu ~2 giờ, build/tải ảnh ~1 giờ. Giữ nguyên D: và F:. Mọi lệnh PowerShell **admin**.

1. **Windows 11 Pro** lên NVMe mới (có tản nhiệt). Tài khoản `ADMIN`, đặt **IP tĩnh 192.168.100.10/24**, GW 192.168.100.1, DNS 8.8.8.8.
   Tắt ngủ: `powercfg /change standby-timeout-ac 0; powercfg /change hibernate-timeout-ac 0`. Múi giờ VN.
   Bật feature: `Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V,VirtualMachinePlatform,Microsoft-Windows-Subsystem-Linux,IIS-WebServer -All`.
   Driver chipset AMD + **NVIDIA ≥ 610** (Blackwell). Cài `winget install` các gói ở §1 (Docker.DockerDesktop, Git.Git, OpenJS.NodeJS.22,
   Python.Python.3.14, Rustlang.Rustup, Microsoft.VisualStudio.2022.BuildTools (C++), oschwartz10612.Poppler, Tailscale.Tailscale,
   smartmontools.smartmontools, 7zip.7zip, RustDesk); `npm i -g pm2`; SQL Express chỉ khi thật cần (hiện không hệ nào dùng).
   OpenSSH: `Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0`, khoá công khai .127 → `C:\ProgramData\ssh\administrators_authorized_keys`
   (bản sao ở `tep\ProgramData-ssh`), firewall 22 chỉ từ 100.64.0.0/10. Tailscale đăng nhập lại (cùng tailnet máy .127).
   **Từ nay tắt máy bằng Shut down, lắp UPS.** Không cài 360 Total Security.
2. **Docker Desktop**: WSL2, "Start when you log in"; chép `cau-hinh\wslconfig` → `C:\Users\ADMIN\.wslconfig` (memory 40 GB, swap 8 GB), `wsl --shutdown`.
   Chép `tep\DigiText-docker` → `C:\DigiText-docker` (config rỗng, dùng với `DOCKER_CONFIG` khi chạy dưới SSH/task).
   Kiểm GPU: `docker run --rm --gpus all nvidia/cuda:12.9.1-base-ubuntu22.04 nvidia-smi -L`.
3. **DigiText** (theo runbook §3.2, rút gọn):
   - `tep\DigiText` → `C:\DigiText` (đã có `.env`, `jwt-ed25519.pem`, `target\release`, `updates\`, `ocr\models`). Nếu cần build lại:
     `cargo build --release` (lần đầu có thể chết vì AV quét build script — chạy lại). `tep\DigiText-Caddy` → `C:\DigiText-Caddy` (giữ `data\` chứng chỉ).
   - Postgres + ClamAV: `docker compose -f C:\DigiText\infra\compose\docker-compose.yml up -d`; khôi phục:
     `docker exec -i digitext-dev-postgres-1 pg_restore -U digitext -d digitext -j 8 --no-owner < D:\OCRPC-Backup-20261104\csdl\digitext-20261104-toanbo.dump`
     (mẫu đầy đủ drop/create/ANALYZE/đếm: `C:\DigiText-ChuyenMay\phuc-hoi-csdl-chot.ps1`). Migrations đã nằm trong dump (đến 0024).
   - Volume: `docker volume create digitext-vllm_hf_cache` rồi `docker run --rm -v digitext-vllm_hf_cache:/v -v D:\OCRPC-Backup-20261104\volumes:/b alpine tar xf /b/digitext-vllm_hf_cache.tar -C /v`;
     tương tự `digitext-ocr-fulltext_paddlecache`. Ảnh OCR: `docker load -i D:\OCRPC-Backup-20261104\images\digitext-ocr-fulltext-ocr.tar`
     (hoặc build lại 20 phút theo `infra/compose/ocr/.env.rtx5070.example`). vLLM: `VLLM_GPU_FRACTION=0.70` (đã trong `.env`).
   - Bật tất cả: `powershell -ExecutionPolicy Bypass -File C:\DigiText\dev\ocr-pc-bat-dich-vu.ps1 -KhoDir D:\DigiText-KhoMayChu`
     (MinIO task → API/worker task → OCR + vLLM compose → Caddy service + firewall 443 → watchdog → backup 01:00). Nếu GPU còn bận
     bởi `vllm-vnkie` thì `-KhongGpu` và để worker dùng Alienware (`.env` đang trỏ 192.168.100.11:8003 cho VLM; OCR vẫn cần container 5012).
   - Task Windows khác (nếu script không tạo): nhập từ `cau-hinh\task-*.xml` bằng `Register-ScheduledTask -Xml`.
   - Kiểm §5 runbook: `https://api.digitext.vn/healthz` 200, đăng nhập desktop, OCR 1 tài liệu, ký 1 tài liệu, đóng gói 1 hồ sơ, reboot.
4. **Hệ cũ doc_extract_pla**: `docker compose --project-directory D:\doc_extract_pla -f docker-compose.vllm.yml -f docker-compose.sqlserver.yml --env-file .env.vllm up -d`
   (ảnh: `docker load` từ `D:\doc_extract_pla-backup\2026-11-02\images`; compose đã giải: `…\cau-hinh\compose-resolved.yml`).
   RESTORE từng `.bak` ở `mssql-bak\` (`docker cp` vào `/var/opt/mssql/backup`, `RESTORE DATABASE [x] FROM DISK=… WITH MOVE …`).
   Chạy web: `D:\doc_extract_pla\doc_extract_pla.exe` (cách chạy tách phiên: `C:\DigiText-ChuyenMay\bat-lai-C.ps1`).
   Cảnh báo VRAM: vnkie 0.75 + DigiText vLLM 0.70 không cùng lúc được trên 16 GB.
5. **ct5d**: `tep\srv` → `C:\srv`; `docker load` 2 ảnh ct5d; tạo 3 volume `ct5d_prod_*` và `tar xf` từ `volumes\`; **hoặc** dựng Postgres
   rỗng rồi `psql -U <user> -f csdl\ct5d-postgres-20261104.sql`; `docker compose up -d` trong `C:\srv\smart-apartment-ct5d`.
   Service tunnel: `sc.exe create cloudflared-ct5d binPath= "\"C:\Program Files (x86)\cloudflared\cloudflared.exe\" --config \"C:\Users\ADMIN\.cloudflared\ct5d.yml\" --no-autoupdate tunnel run" start= auto`
   (`tep\ADMIN\.cloudflared` → `C:\Users\ADMIN\.cloudflared`).
6. **n8n**: `tep\n8n` → `C:\n8n`; `tep\ADMIN\.pm2` → `C:\Users\ADMIN\.pm2`; `pm2 resurrect`; `pm2 startup` (Windows: dùng task tự chạy
   `pm2 resurrect` khi đăng nhập, hoặc `pm2-installer`). Tunnel: service `cloudflared-n8n` với `C:\n8n\cloudflared\config.yml`
   (lệnh đầy đủ trong `tep\cloudflared\services.txt`).
7. **IIS**: bật feature (bước 1) + ASP.NET Core Hosting Bundle (các app pool không dùng managed runtime); nhập site từ
   `cau-hinh\iis-sites.xml` / `iis-apppools.xml` (hoặc `appcmd add site /name:DGT_Base /bindings:http/*:80:digitext.vn /physicalPath:D:\DULIEU_66\DigiApp\DGT_Base`
   cho từng dòng ở §1) — hoặc chép `tep\inetsrv-config\applicationHost.config` đè sau khi cài IIS.
   Tunnel chính: `sc.exe create Cloudflared binPath= "\"C:\Program Files (x86)\cloudflared\cloudflared.exe\" tunnel run --token-file C:\ProgramData\cloudflared\token" start= auto`
   (`tep\cloudflared\ProgramData-cloudflared` → `C:\ProgramData\cloudflared`).
8. **Firewall + FortiGate**: firewall Windows mở 443 (`DigiText-Caddy-443`), 22 (Tailscale), 80, 5001, 5002, 8081/8082/8084 như
   `cau-hinh\firewall-inbound.txt`; FortiGate giữ nguyên (VIP trỏ IP máy, không đổi).
9. **Kiểm cuối**: từ ngoài `https://api.digitext.vn/healthz`, `https://digitext.vn`, web cũ `:5002`, n8n qua tunnel, ct5d qua tunnel,
   các site IIS; `wsl --shutdown` thử rồi reboot máy — mọi task/service/container tự lên; dump đêm xuất hiện lúc 01:02.

## 4. Bẫy đã gặp (đọc trước khi làm)

- Docker CLI dưới SSH/task: bỏ thư mục Docker khỏi `PATH` **hoặc** giữ nhưng đặt `DOCKER_CONFIG=C:\DigiText-docker` (credential helper cần phiên đăng nhập).
  `cmd /c "docker …"` cần `docker.exe` trong PATH hoặc đường dẫn đầy đủ.
- Tiến trình con chết theo phiên SSH → chạy tách bằng `Invoke-WmiMethod Win32_Process Create` hoặc task ONCE (`schtasks` đôi khi "Access is denied").
- PowerShell 5.1 + `$ErrorActionPreference=Stop` + `2>&1`: dòng stderr của docker/robocopy làm script chết im lặng → dùng `Continue`.
- Bind mount ghi thẳng D: từ container từng treo → stream `docker run … tar cf - > tệp` qua `cmd /c`.
- Heredoc/`printf`/`sed` trong Bash phía máy .127 làm `\\` thành `\` (đã hỏng đường dẫn `\resources\bin` một lần) → ghi script bằng
  công cụ Write, chép bằng scp, thêm BOM UTF-8 để PowerShell đọc tiếng Việt.
- `VLLM_GPU_FRACTION` phải 0.70 trên 5070 Ti (0.55 sập). Paddle 3.2 + torch cu128 chạy được sm_120, không cần plan-B ONNX.
- `sqlcmd` ODBC 18 cần `-C` (tin chứng chỉ tự ký); trong `while read` phải `< /dev/null`.
- Nhật ký System của Windows quay vòng nhanh khi có bão lỗi đĩa — thời điểm "đầu tiên" trong log không phải lúc bắt đầu thật.
- Máy từng có cloudflared tunnel `ocr-pc` cho `digitext.vn`/`www` — từ 2/11 hai tên này đi qua Caddy :443 → IIS :80; tunnel chính
  (`Cloudflared`) vẫn chạy cho tên miền khác, đừng gỡ.
