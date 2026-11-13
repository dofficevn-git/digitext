# Chuyển máy chủ DigiText sang OCR PC — runbook + nhật ký

Máy chủ hiện tại `192.168.0.127` (máy làm việc, Wi-Fi, HDD SMR, Docker chung 3 dự án) + GPU ở Alienware
`192.168.0.205` → **OCR PC** `192.168.100.10` (Ryzen 9 9950X, 64 GB, RTX 5070 Ti 16 GB Blackwell, Tailscale
`100.122.226.63`) chạy tất cả: API, worker, Postgres, kho MinIO, OCR GPU, Qwen. Kế hoạch gốc (27/10/2026) nằm
ngoài repo; tài liệu này là bản rút gọn đủ để làm, kèm nhật ký những gì đã làm.

## 1. Quyết định đã chốt (27/10/2026)

| Câu hỏi | Chốt |
|---|---|
| Ổ trên OCR PC | Kho MinIO trên **HDD** (kiểm CMR/SMR trước: `Get-PhysicalDisk \| ft FriendlyName,MediaType`; SMR thì giữ `MINIO_SCANNER_SPEED=slowest`), Docker/Postgres/image trên **SSD** |
| Qwen | **Gộp lên OCR PC** (Alienware chạy OCR + Qwen trong 10,5/16 GB → 5070 Ti 16 GB vừa); Alienware giữ dự phòng 30 ngày |
| Chuyển kho | **Bằng ổ F: cầm tay**: F: đã có 204.981 bản ký đúng kích thước → `dev/nap-ban-ky.py` nạp từ thư mục; phần còn lại (~1 GB) qua `dev/dong-goi-kho.py xuat --chi-khong-phai-ban-ky` |
| Đường vào API | **Trực tiếp, không tunnel** (anh chốt lại 30/10): Caddy + Let's Encrypt ở **cổng 443**, một tên duy nhất **`api.digitext.vn`** (đã đóng cứng trong updater desktop); router **NAT thêm 443 → 192.168.100.10:443** (5005 đã NAT sẵn không dùng tới; 5001 n8n, 5002 web KIE cũ, 80 IIS). Máy trạm giữ nguyên `https://api.digitext.vn`. **DNS quản lý THẲNG ở iNET, KHÔNG Cloudflare gì nữa** (anh chốt rõ 2/11 — em từng ghi nhầm "Cloudflare chỉ còn làm DNS"): đổi nameserver về iNET, bản ghi A → 113.190.254.219, chứng chỉ **TLS-ALPN-01 qua 443**, không token. Tắt cloudflared trên .127 ở GĐ2; tên miền **hết hạn 24/11/2026**, phải gia hạn |
| IP công cộng | **Đã tĩnh** `113.190.254.219` |
| Vào OCR PC | **OpenSSH** anh bật, firewall chỉ mở từ Tailscale `100.64.0.0/10` (lệnh ở §4.1) |
| Bỏ hẳn | Meilisearch, rustus, nginx trong compose; Vintern; SQL Server + web KIE cũ; Triton (chỉ giữ ONNX làm dự phòng) |

## 2. GĐ0 — đã làm trên máy .127 + Alienware (30/10/2026)

### 2.1 Repo (commit `10ecc10` + commit kèm tài liệu này)

| Tệp | Việc |
|---|---|
| `infra/compose/docker-compose.yml` | Chỉ còn `postgres` (+ tuning qua env `PG_SHARED_BUFFERS` `PG_EFFECTIVE_CACHE` `PG_WORK_MEM` `PG_MAINT_MEM` `PG_PARALLEL` `PG_MAX_WAL`, mặc định = máy .127) và `clamav` (3310). Bỏ minio/meilisearch/rustus/nginx |
| `infra/compose/vllm/docker-compose.yml` | Khớp lệnh sống Alienware: bỏ `--enforce-eager`, thêm `--kv-cache-dtype fp8`, `--mm-processor-cache-gb 0`, mặc định 16 lượt / 8192; `device_ids: ["${GPU_DEVICE_ID:-0}"]`; `VLLM_START_PERIOD` |
| `infra/compose/ocr/docker-compose.gpu.yml` | Base/torch/requirements chọn bằng env (`OCR_CUDA_IMAGE` `OCR_TORCH_INDEX` `OCR_TORCH_VERSION` `OCR_TORCHVISION_VERSION` `OCR_REQS`), mặc định = Alienware; `GPU_DEVICE_ID` |
| `infra/compose/ocr/.env.rtx5070.example` | Bộ giá trị Blackwell: CUDA 12.9 + cuDNN 9, torch `2.7.1+cu128`, `requirements-ocr-gpu-cu128.txt` (Paddle 3.2.0 cu129 + paddleocr 2.10.0 — **phải kiểm lúc build**), cổng **5012**, ngân sách VRAM khởi điểm |
| `infra/compose/ocr/Dockerfile` | Kiểm phiên bản torch ngay sau pip (chặn lấy nhầm bản PyPI không có sm_120); symlink `libcudnn.so.9`, `libnvJitLink.so.12`; healthcheck start 600 s |
| `ocr/ocr_fulltext.py` | **Warm-up** DBNet + VietOCR ngay khi nạp (lỗi kernel → health 503, không còn "xanh mà 500"); `_is_oom` thêm mẫu Paddle 3; `_release_cuda` dọn cả Paddle; health thêm `paddle_version`, `paddle_reserved_mb`, `det_backend`, `warmup`; **`OCR_DET_BACKEND=onnx`** → `det_onnx.py` |
| `ocr/det_onnx.py` (mới) | Plan-B: DBNet qua ONNX Runtime, tái tạo đúng tiền/hậu xử lý PaddleOCR 2.x. **Parity 7/7 trang thật, IoU 1,000, cùng số hộp** (`ml/bench/harness/det_parity.py`, chạy trong container trên Alienware 30/10). Model: `ocr/models/det/en_PP-OCRv3_det.onnx` (SHA-256 `072ce94a…44eff`, gitignore — có trong bộ mang sang) |
| `infra/caddy/Caddyfile` (mới) | `api.digitext.vn` 443, DNS-01 Cloudflare, body 2 GB, chờ 15 phút, log |
| `dev/watchdog.ps1` (mới) | Mỗi phút: API tại chỗ / qua tên miền, container unhealthy, worker `/readyz`, MinIO; hỏng 3 lần liền mới khởi động lại; `logs/watchdog.log` |
| `dev/backup-csdl.ps1` (mới) | `pg_dump -Fc` trong container → tệp, kiểm `pg_restore --list`, giữ 14 bản. Đo 30/10: **1,6 GB / 348 s** |
| `dev/nap-ban-ky.py` (mới) | `xuat` bảng `document_id,rel_path,filename,signed_key,size` (LTLSBN: 204.981 dòng, 0 thiếu trong kho); `nap` từ thư mục vào MinIO (ETag = MD5, bỏ qua khoá đã đúng cỡ); `kiem` (đếm + 300 mẫu MD5) |
| `dev/dong-goi-kho.py` | cờ `--chi-khong-phai-ban-ky` |
| `dev/minio-win*.ps1` | exe mặc định `infra/minio-win/minio.exe` (gitignore) |
| `dev/start-all.ps1`, `thu-nho-docker.ps1`, `theo-doi-ocr.ps1` (`-GpuHost ""` = GPU tại chỗ), `release-desktop.ps1` (`-UpdatesRemote`) | gỡ đường dẫn cứng máy .127 |
| `dev/kiem-du-o-ngoai.py`, `tai-ban-ky.py`, `xoa-pdfa-da-tai.py`, `purge-sources.py` | đưa vào repo (chuyển LTLSBN ra F:, 10/2026) |

### 2.2 Số đo baseline trên Alienware (so sánh khi dựng OCR PC)

Cùng golden set `ml/bench/golden-set/vlm_pseudo` (520 dòng), cùng trọng số `vgg_transformer_ft_v2b.pth`, dịch vụ
đang chạy thật (Paddle 2.6.2, torch 2.4.1 cu121, VLM chữ tay Qwen3-VL bật, inklines bật):

| Đo | Kết quả | Tệp |
|---|---|---|
| CER / WER / Δdấu / ECE | **0,1756** / 0,2324 / 0,0078 / 0,268 | `report_baseline_alienware_20261030.json` |
| loadtest c4 n100 | **37,6 trang/s**, p50 100 ms, p95 197 ms, p99 278 ms | `loadtest_baseline_alienware_20261030_c4.txt` |
| loadtest c8 n300 | 17,8 trang/s, p50 291 ms, p95 791 ms, p99 1.754 ms | `…_c8.txt` |
| VRAM lúc tải | 11,3 GB / 16 GB (OCR + Qwen) | |

**Chấp nhận trên OCR PC:** `gate.py --baseline report_baseline_alienware_20261030.json --tol 0.01` PASS (cùng detector thì
CER phải trùng ±0,002); `pages_per_sec` ≥ 0,7× và p95 ≤ 1,4× số trên; c8 n300 không có 500/507; VRAM ổn định < 14,5 GB.

> ⚠ Phát hiện ngoài lề (chưa sửa): CER hôm nay 0,1756 **xấu hơn** bản đo 17/9 cùng trọng số (0,1437). Soi 520 dòng:
> 70 dòng tốt lên (−0,015) nhưng **15 dòng** ra chữ dài gấp nhiều lần ("a a a a a…", "Nhà Nhà Nhà…", số bịa) ở
> conf 0,5–0,6 → +0,043. Đó là lớp dự phòng chữ tay (`ocr/handwriting.py`, Qwen3-VL từ 26/9) **bịa** trên dòng khó.
> Việc riêng: chặn kết quả VLM dài hơn ~2× ước lượng theo bề rộng dòng hoặc lặp token, trả về bản VietOCR.

### 2.3 Bộ mang sang — `D:\DigiText-ChuyenMay` (chép sang `F:\DigiText-ChuyenMay` khi cắm F:)

| Thư mục | Nội dung |
|---|---|
| `bi-mat/` | `jwt-ed25519.pem`, `crates/digitext-api/.env`, `crates/digitext-ocr/.env` (+`.env.che-do-nhanh`, `.env.bak-20261021`), `infra/compose/.env`. Không mang token cloudflared (bỏ tunnel). Cần thêm **token Cloudflare quyền Zone.DNS:Edit** cho Caddy — tạo mới trên dash.cloudflare.com |
| `alienware/` | `compose/ocr/.env` + compose, `compose/vllm/.env` + compose (bản sống), `lenh-song.txt` (docker inspect), `vietocr/*.pth` (4 tệp, 580 MB, SHA-256 khớp), `onnx/en_PP-OCRv3_det.onnx`, `golden/golden-vlm_pseudo-520.tar.gz` (520 ảnh + manifest đường dẫn tương đối + gold), baseline reports |
| `csdl/digitext-nhap-20261030-0859.dump` | dump nháp 1,6 GB (diễn tập restore ở GĐ1) |
| `kho/ban-ky-LTLSBN.csv` | bảng 204.981 bản ký cho `nap-ban-ky.py` |
| `trust/`, `data/` (C:\DigiText-data), `updates/` (bản phát hành desktop, 1,1 GB), `minio-win/minio.exe`, `rescue/digitext-rescue.tar.gz` (bản sao duy nhất), `logs/` (danh sách PDF/A đã xoá, bản trùng đã xoá) | |

### 2.4 Còn lại trên .127 khi cắm F:

```powershell
robocopy D:\DigiText-ChuyenMay F:\DigiText-ChuyenMay /E
python -I dev\dong-goi-kho.py xuat --nguon http://127.0.0.1:9000 --thu-muc F:\DigiText-KhoTar --chi-khong-phai-ban-ky   # ~1 GB
python -I dev\dong-goi-kho.py kiem --thu-muc F:\DigiText-KhoTar
```

## 3. GĐ1 — dựng OCR PC song song (máy cũ vẫn phục vụ)

### 3.1 Anh làm (RDP vào OCR PC, PowerShell admin)

```powershell
Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0; Set-Service sshd -StartupType Automatic; Start-Service sshd
New-NetFirewallRule -Name sshd-tailscale -DisplayName "OpenSSH (Tailscale)" -Direction Inbound -Protocol TCP -LocalPort 22 -RemoteAddress 100.64.0.0/10 -Action Allow
# khoá công khai máy .127 → C:\ProgramData\ssh\administrators_authorized_keys, rồi:
icacls C:\ProgramData\ssh\administrators_authorized_keys /inheritance:r /grant "Administrators:F" /grant "SYSTEM:F"
powercfg /change standby-timeout-ac 0; powercfg /change hibernate-timeout-ac 0
```
Cắm dây LAN. Cắm F:. Xác nhận ổ HDD cho kho (CMR?).

### 3.2 Em làm qua SSH

1. **Nền:** driver NVIDIA ≥ 576; Docker Desktop (WSL2; `.wslconfig` `memory=40GB swap=8GB`; bật "Start when you log in"
   hoặc auto-logon tài khoản dịch vụ — kiểm Docker lên sau reboot không cần đăng nhập); Rust + VS Build Tools + Node;
   Poppler (`winget install oschwartz10612.Poppler`); Git clone repo; chép `bi-mat/` vào đúng chỗ; `ocr/models/vietocr/`,
   `ocr/models/det/` từ bộ mang sang; `infra/minio-win/minio.exe`.
   Kiểm GPU trong Docker: `docker run --rm --gpus all nvidia/cuda:12.9.1-base-ubuntu22.04 nvidia-smi -L` → UUID.
2. **Postgres:** `infra/compose/.env` thêm `PG_SHARED_BUFFERS=6GB PG_EFFECTIVE_CACHE=16GB PG_WORK_MEM=64MB PG_MAINT_MEM=1GB PG_PARALLEL=4 PG_MAX_WAL=4GB`;
   `docker compose -f infra/compose/docker-compose.yml up -d`; diễn tập
   `docker exec -i digitext-dev-postgres-1 pg_restore -U digitext -d digitext -j 8 --no-owner < digitext-nhap-*.dump`; `ANALYZE`;
   áp `migrations/0006` (CONCURRENTLY). Đo thời gian.
3. **MinIO:** `dev\minio-win-task.ps1 -DataDir <HDD>:\DigiText-KhoMayChu -Port 9000`; Defender exclusion tay cho thư mục kho + exe.
   `python -I dev\dong-goi-kho.py nap --dich http://127.0.0.1:9000 --thu-muc F:\DigiText-KhoTar` rồi
   `python -I dev\nap-ban-ky.py nap --bang F:\DigiText-ChuyenMay\kho\ban-ky-LTLSBN.csv --thu-muc "F:\2026\PDFa\LTLSBN · LTLSBNinh" --dich http://127.0.0.1:9000`
   (ước 3–4 giờ) → `nap-ban-ky.py kiem`.
4. **OCR GPU:** `infra/compose/ocr/.env` = bản Alienware + các dòng `.env.rtx5070.example` (+ `GPU_DEVICE_ID`). Build; kiểm
   theo trình tự trong file example. Paddle không chạy sm_120 → `OCR_DET_BACKEND=onnx` + thêm `onnxruntime-gpu==1.22.0`
   vào requirements cu128 (plan-B, parity đã chứng minh).
5. **vLLM:** `infra/compose/vllm/.env` = bản Alienware + `VLLM_IMAGE=vllm/vllm-openai:v0.15.1` (`ocr/.env.vllm.rtx5070.example`),
   `VLLM_GPU_FRACTION=0.70` (0.55 như Alienware **không đủ** trên 5070 Ti: mô hình 7,37 GB + đo đỉnh → KV cache âm, engine sập — 2/11), `GPU_DEVICE_ID`, `VLLM_MEM_LIMIT=20g`, `VLLM_START_PERIOD=1200s`. Không có FlashAttention
   cho sm_120 → `VLLM_ATTENTION_BACKEND=FLASHINFER`. Kiểm `host.docker.internal:8003/v1/models` từ container OCR.
6. **Chấp nhận:** giải nén `golden-vlm_pseudo-520.tar.gz`, `run_bench.py --endpoint http://127.0.0.1:5012` → `score.py` →
   `gate.py --baseline report_baseline_alienware_20261030.json --tol 0.01`; `loadtest.py` c4/c8 so §2.2.
7. **API/worker:** `crates/digitext-ocr/.env`: `DIGITEXT_OCR_HTTP_ENDPOINT=http://127.0.0.1:5012`, VLM `127.0.0.1:8003`,
   `DIGITEXT_PDFTOPPM_BIN`, `DIGITEXT_CLAMD_ADDR=127.0.0.1:3310`; `crates/digitext-api/.env`: `DIGITEXT_BEHIND_PROXY=true`,
   `DIGITEXT_UPDATES_DIR`, `DIGITEXT_DATA_DIR`, `DIGITEXT_SIGN_TRUST_ROOTS` theo ổ mới. `cargo build --release`;
   `dev\install-services.ps1`; chạy thử với `DIGITEXT_WORKER_ENABLED=false` rồi OCR 1 dự án TEST.
8. **Caddy + watchdog + backup:** theo chú thích đầu `infra/caddy/Caddyfile`, `dev/watchdog.ps1`, `dev/backup-csdl.ps1`.
   Anh: router NAT **443 → 192.168.100.10:443**; firewall OCR PC mở 443. Chứng chỉ lấy trước qua DNS-01 (không cần DNS
   trỏ về) → kiểm `curl --resolve api.digitext.vn:443:192.168.100.10 https://api.digitext.vn/healthz` từ máy trong LAN.

## 4. GĐ2 — cắt chuyển (ngoài giờ, 1–2 giờ gián đoạn)

Trên .127: tắt task `DigiText-API`, `DigiText-Worker`; `dev\backup-csdl.ps1 -Dir F:\DigiText-ChuyenMay\csdl -Giu 0` (dump chốt);
`dong-goi-kho.py xuat --chi-khong-phai-ban-ky` phần chênh; `nap-ban-ky.py xuat` lại nếu có ký thêm. Sang OCR PC: drop/restore CSDL
chốt, `nap` phần chênh, áp 0006, bật API/Worker/Caddy; **đổi DNS `api.digitext.vn`**: bỏ bản ghi CNAME tunnel, thêm A →
`113.190.254.219` đám mây xám (TTL ngắn trước đó 1 ngày); kiểm §5; tắt + gỡ service `cloudflared-digitext` trên .127, xoá tunnel
`digitext` và bản ghi rác `api.digitext.vn.khodulieu.vn` trên Cloudflare.

## 5. Kiểm chứng sau chuyển

- CSDL: `SELECT count(*) FROM documents` = 205.008 (LTLSBN) + các dự án khác như dump; `sign_sessions status='signed'` = 204.981;
  `ix_ocr_jobs_doc_kind_created` có; `roles` 5 dòng hệ thống.
- Kho: `nap-ban-ky.py kiem` 0 sai; tổng lớp = 204.981 bản ký / 799,0 GB + 3 PDF/A + 43 khác; `GET /documents/{id}/signed` mở được.
- API: `https://api.digitext.vn/healthz` 200 chứng chỉ Let's Encrypt hợp lệ, IP phân giải = 113.190.254.219 (không còn IP Cloudflare);
  `https://api.digitext.vn/updates/latest.json` đọc được; đăng nhập desktop từ ngoài văn phòng; tải tệp 150 MB (vượt giới hạn tunnel cũ).
- Worker: `127.0.0.1:9095/readyz` 200; OCR 1 tài liệu TEST → PDF/A đạt veraPDF; bóc trường v2 chạy; gate CER PASS.
- Ký số 1 tài liệu thật từ máy trạm; đóng gói 1 hồ sơ (`virus_scan` ≠ `not_configured`).
- Khởi động lại OCR PC: mọi task/service tự lên; rút mạng 2 phút → watchdog ghi log và tự hồi. Dump đêm xuất hiện trên F:.

## 6. Rollback (14 ngày)

Trên .127 bật lại `DigiText-API` / `Worker` / `MinIO` / `cloudflared-digitext` và trả DNS `api.digitext.vn` về CNAME tunnel
(vì vậy **chưa xoá tunnel trên Cloudflare trong 14 ngày**). Dữ liệu phát sinh trên
OCR PC phải dump ngược + `dong-goi-kho.py xuat` phần chênh — **tuần đầu không xoá gì trên .127**.

## 7. Nhật ký

- 27/10/2026: kế hoạch + quyết định; LTLSBN đủ trên F: (204.981 bản ký + 204.981 PDF/A + 27 bản gốc), xoá 572 bản trùng.
- 30/10/2026: GĐ0 xong phần repo, baseline Alienware, ONNX + parity, dump nháp, bộ mang sang trên D:. Anh chốt lại: **bỏ
  cloudflared hẳn**, đi thẳng `api.digitext.vn` 443 (NAT 443).
- 30/10/2026 chiều — **GĐ1 trên OCR PC** (SSH `ADMIN@100.122.226.63`, chỉ qua Tailscale relay ~350 ms; LAN hai dải chưa route,
  máy .127 không được mở dịch vụ chia sẻ → chuyển lớn bằng F:). Hiện trạng: Win11 Pro 26300, 62 GB, **HDD HGST Ultrastar 8 TB CMR**
  (D:, kho), NVMe 1,8 TB (C:), driver 610.88, Docker 29.6.1. **Máy đang chạy hệ thống khác** (`ct5d_*` 3020/3021/5444/9030,
  `vllm-vnkie` chiếm 15,7/16 GB VRAM, mssql 1433, web KIE 5002, n8n 5001, IIS 80) — anh: *chưa dừng gì*, chỉ chuẩn bị.
  Đã làm: cài Poppler/Rust/MSVC/Caddy (có module cloudflare); clone repo `C:\DigiText`; `.wslconfig` 40 GB; Postgres (tuning) +
  ClamAV lên; **restore dump 123 s** (205.008 tài liệu, 204.981 đã ký, 63 bảng như .127, 0006 có); `cargo build --release`
  2 phút (lần đầu chết vì build script bị quét thời gian thực — chạy lại là được); **image OCR Blackwell build 20 phút, kiểm đạt:
  torch 2.7.1+cu128 + Paddle 3.2.0 đều chạy GPU trên 5070 Ti** (sửa Dockerfile: ldconfig `nvidia/*/lib` vì Paddle đè wheel 12.9);
  MinIO tạm trong phiên SSH (không task) đang nạp 204.981 bản ký từ F: ~40 MB/s (~5,5 giờ). Bẫy: Docker CLI dưới SSH/SYSTEM
  lỗi "error getting credentials" → bỏ thư mục Docker khỏi PATH + `DOCKER_CONFIG=C:\DigiText-docker` (script bật đã có sẵn mẹo này).
  Script bật một lần: `dev/ocr-pc-bat-dich-vu.ps1`. Còn chờ anh: dừng `vllm-vnkie` (mới chạy được OCR/Qwen + bench), NAT 443,
  token Cloudflare DNS.
- 30/10 15:20: **nạp kho xong** 204.981 bản ký / 799,0 GB, 0 lỗi, 309 phút (43,5 MB/s từ USB); `nap-ban-ky.py kiem`: khớp
  204.981 khoá, 300 mẫu MD5 lệch 0. Qwen3-VL-8B AWQ (7,1 GB) đã tải vào volume `digitext-vllm_hf_cache`.
- 2/11/2026: anh đồng ý **bật nội bộ** (chạy thử, chưa cắt chuyển): task `DigiText-MinIO`, `DigiText-API`, `DigiText-Worker`
  đã đăng ký và chạy (MinIO/API/worker healthz 200; worker readyz chưa 200 vì chưa có OCR). Bẫy: `$PSScriptRoot` rỗng lúc
  đánh giá tham số mặc định khi gọi `-File` → 3 script đổi sang tính trong thân. CSDL trên OCR PC vẫn là bản chụp 30/10 09:00 —
  lúc cắt chuyển phải dump lại từ .127. Chưa bật: OCR/vLLM (VRAM bị vnkie), Caddy (token + NAT 443), watchdog, backup.
- 2/11/2026 (chiều): nhánh `feat/mlhs-tt05-mot-luong` (xem `docs/CHINH-LY-SYNC.md` mục 2/11): **áp 0023** lên Postgres nội bộ
  OCR PC, thay `digitext-api.exe` bằng bản build `target-mlhs/release` (máy .127 đang chạy `target/release` nên khoá tệp —
  build release phải đổi `CARGO_TARGET_DIR`); bản thử desktop (chưa ký) ở `C:\DigiText-ChuyenMay\DigiText_0.19.8-mlhs-thu_x64-setup.exe`.
  Bẫy: lần đầu chép nhầm exe cũ 22/10 (build chết vì AV mà vẫn exit 0 qua pipe) → đã khôi phục từ `.bak-20261102-1031` rồi
  mới thay đúng. Script dùng: `C:\DigiText-ChuyenMay\{ap-0023,thay-api,khoi-phuc-api}.ps1`.
- 2/11/2026 (chiều, tiếp): **sao lưu + tắt nền tảng cũ `doc_extract_pla`** (vllm-vnkie + mssql_container, compose
  `D:\doc_extract_pla`) theo yêu cầu anh → `D:\doc_extract_pla-backup\2026-11-02` (anh đổi ý từ F: sang D:; bản F: bị xoá):
  46 `.bak` COPY_ONLY + VERIFYONLY (7,6 GB, 43 CSDL người dùng, `pla_kie` là CSDL ứng dụng), `app\` (exe/.env/compose/models,
  11,1 GB; `uploads` 1,03 TB + `static` 443 GB giữ nguyên tại D:), `docker save` 2 ảnh (lớp nén OCI vì containerd store:
  vllm 8,7 GB, mssql 505 MB), `cau-hinh\` inspect + compose resolved, `volumes\` tar 3 volume, `README-KHOI-PHUC.md`.
  Rồi `docker stop` + `--restart=no` hai container (14:26). **GPU rảnh** → bật `digitext-vllm` + `digitext-ocr-fulltext` (OCR 5012
  healthy, worker readyz 200, vLLM 8003 healthy sau khi nâng `VLLM_GPU_FRACTION` 0.55→0.70; VRAM 11,3/16 GB). Chưa bench §3.2.6.
  Bẫy: (1) `sqlcmd` tua lại stdin khi chạy trong `while read` → vòng lặp lặp mãi `master`; thêm `< /dev/null`. (2) Tiến trình
  con của phiên SSH (kể cả `Start-Process`) chết khi phiên đóng → chạy nền bằng `schtasks /SC ONCE` hoặc
  `Invoke-WmiMethod Win32_Process Create` (schtasks có lúc "Access is denied"). (3) Ghi tar qua bind mount D: của Docker Desktop
  treo → stream `docker run … tar cf - > tệp` qua `cmd /c`. (4) **`/var/opt/mssql/data/pla_kie_log.ldf` (11,6 GB) treo khi đọc
  ở offset ~1,3 GB** trong đĩa ảo WSL (D state `folio_wait_bit_common`, lặp lại 3 lần cùng chỗ) → tar volume mssql **trừ tệp này**
  (`…mssql_data.tru-pla_kie_log.tar`); `pla_kie` khôi phục từ `.bak`. Chỉ `wsl --shutdown`/khởi động lại Docker Desktop mới gỡ
  được trang treo — làm vào cửa sổ cắt chuyển, không làm lúc `ct5d_*` đang chạy. Script: `C:\DigiText-ChuyenMay\{bk-docext-D,
  tar-volume-3,bat-ocr-vllm,sua-vllm}.ps1`, `mssql-bak.sh`.
- 2/11/2026 15:40: anh muốn **chạy song song hệ cũ** để người dùng vẫn xác nhận trên OCR cũ → bật lại `mssql_container`
  (phục hồi 30 s, `pla_kie` ONLINE, không chạm vùng ldf treo), web KIE `doc_extract_pla.exe` :5002 (bật qua WMI, trước đó không
  chạy), `vllm-vnkie` hạ `--gpu-memory-utilization` 0.90→**0.75** (KV 8,75 GiB, 186× ở 4096); DigiText trên OCR PC: `digitext-vllm`
  **stop** (16 GB không đủ hai vLLM), OCR 5012 + worker giữ. Alienware ở tailnet khác (`dangdat234678@`) nên không trỏ qua Tailscale
  được → anh **cắm Alienware vào LAN OCR PC: 192.168.100.11** (quét LAN tìm ra, không phải .16 như báo) → worker
  `DIGITEXT_FIELDS_ENGINE=v2`, `DIGITEXT_VLM_ENDPOINT=http://192.168.100.11:8003/…`, OCR `OCR_HW_VLM_URL` cùng chỗ; readyz 200.
  VRAM 14,0/16,3 GB. **Hệ quả: máy .127 mất Alienware (192.168.0.205 không còn)** → bản chính hết OCR/Qwen cho tới cắt chuyển
  (hàng đợi lúc đó trống). Lúc cắt chuyển hoặc hệ cũ nghỉ: `docker compose … vllm up -d` tại chỗ (0.70) và trỏ lại 127.0.0.1:8003.
  Script: `C:\DigiText-ChuyenMay\{bat-lai-C,bat-lai-A2,quet-lan}.ps1`; `.env` gốc có bản `.bak-20261102`.
- **2/11/2026 16:00–17:05 — GĐ2 CẮT CHUYỂN XONG, OCR PC là máy chủ bản chính.** Anh: "bản ở máy này coi như đã tắt hẳn".
  Dump chốt `.127` 1,6 GB (SHA ghi ở nhật ký) → chép 9 khúc 200 MB qua Tailscale relay (~2 MB/s, 33 phút; http.server tạm bị
  bộ lọc chặn) → `phuc-hoi-csdl-chot.ps1`: drop/restore 382 s, số khớp (205.008/204.981/1.910 hồ sơ/6 user/5 vai trò, 0023 + 0006).
  Kho MinIO hai bên đã giống hệt (205.027 đối tượng, 799,9 GB) → không chuyển. **DNS: iNET** (anh đổi NS Cloudflare → ns1/ns2.inet.vn,
  bản ghi A `@`/`api` → 113.190.254.219, `www` CNAME `@`; lan xong 16:33). **FortiGate 300D** (`pla-fw`, 192.168.100.1): VIP
  `113.190.254.219-192.168.100.10_httpss` (443) + policy `Public web OCR` đã có sẵn; chỉ phải mở Windows Firewall 443 trên OCR PC.
  **Caddy** service `DigiText-Caddy` chạy, chứng chỉ Let's Encrypt TLS-ALPN-01 cho `api`, `digitext.vn`, `www` (hết hạn 31/1/2027,
  tự gia hạn); `www`/gốc → IIS :80 trên OCR PC (thay tunnel `ocr-pc`). Watchdog + backup 01:00 (`C:\DigiText-Backup\csdl`) đã đăng ký.
  Kiểm từ ngoài: healthz 200, `updates/latest.json` 200, tải bộ cài 13 MB 2,6 s (5 MB/s). Chưa kiểm: đăng nhập desktop, OCR 1
  tài liệu, ký số, đóng gói, khởi động lại máy (§5). Bẫy: (1) `ocr-pc-bat-dich-vu.ps1` chết im lặng sau "dừng minio tạm" vì
  `2>&1` dưới `Stop` → đã đổi sang Continue; nó cũng kill MinIO của task → MinIO treo (6 MB RSS, không mở 9000) phải kill lại cho
  supervisor chạy lại; API thoát 2 lần vì chưa có bucket. (2) 5 phút đầu sau khởi động API tải tệp `updates` chỉ 4–25 KB/s
  (dashboard_rollup + việc nền), sau đó 200 MB/s — đừng kết luận mạng. (3) Máy có **360 Total Security** cạnh Defender —
  nghi là nguồn "Access is denied" của schtasks/cargo. Máy .127: anh tự chạy `dev/dung-dich-vu-127.ps1`; cloudflared trên .127
  không còn tác dụng (NS đã rời Cloudflare) — rollback giờ = trỏ A về .127 không được (không NAT), nên rollback thực tế chỉ còn
  dump ngược từ OCR PC; .127 giữ Postgres/MinIO 14 ngày.
- 3/11/2026 16:00 — **áp 0024** (`records` kind thi công TT05 + cột `ref`, `project_contracts` hộp/hồ sơ/tờ) qua
  `docker exec -i digitext-dev-postgres-1 psql` như `ap-0023.ps1`; thay `C:\DigiText\target\release\digitext-api.exe` bằng bản
  build trên .127 (nhánh `feat/mlhs-tt05-mot-luong`, SHA-256 `9E630EE18A7AD5A1…`; bản cũ giữ `.bak-20261103-1601`); khởi động lại
  bằng `dev\restart-api.ps1` (pid 129224, healthz 200; `GET /api/v1/records/{id}` trả 401 thay vì 404 = route mới đã có).
  Tệp mang sang: `C:\DigiText-ChuyenMay\{digitext-api-0024.exe,0024_records_tt05_thi_cong.sql}`. Repo `C:\DigiText` trên OCR PC
  vẫn ở `main` (39fc34a), chưa checkout nhánh mới — khi build tại chỗ phải nhớ.
- 3/11/2026 16:52 — **API R3** (`GET /cl/doi-soat`, không migration): `C:\DigiText-ChuyenMay\digitext-api-r3.exe`
  (SHA-256 `7B3B5A0EAFCC8FD1…`) thay vào `target\release\digitext-api.exe`; bản 0024 giữ `.bak-20261103-1652`. Lần thay đầu
  (16:46) tưởng treo: healthz không lên sau 15 s, rollback về 0024 cũng "không lên" → thật ra máy đang nặng (ai đó bật
  `vllm-vnkie` + `mssql_container` lúc ~16:32) nên API mất ~45–50 s mới nghe cổng; tiến trình r3 đầu tiên tự thoát sau 184 s
  (chưa rõ mã thoát, `api.err.log` rỗng). Thay lại lần hai + chờ bằng vòng lặp: healthz 200 sau 48 s, `/cl/doi-soat` và
  `/records/{id}` đều 401 (route có). **Bài học:** sau `restart-api.ps1` phải chờ ≥ 60 s hoặc poll healthz, đừng kết luận
  sau 15 s; script mẫu `C:\DigiText-ChuyenMay\trien-khai-r3.ps1` (thay exe → restart → poll 240 s → thử route).
- 3/11/2026 17:26 — **API R4** (hợp đồng thêm hộp/hồ sơ/tờ trong `ContractDto`, cột đã có từ 0024; không migration):
  `C:\DigiText-ChuyenMay\digitext-api-r4.exe` (SHA-256 `26F4977FBE8F04B7…`), bản R3 giữ `.bak-20261103-1725`; chạy
  `trien-khai-r4.ps1`: healthz 200 sau 28 s, `/cl/doi-soat` 401, `information_schema` có đủ `contract_boxes/dossiers/sheets`.
  Chép thêm bộ cài desktop 0.20.0–0.22.0 vào `C:\DigiText\updates` (trước đó `latest.json` công khai kẹt ở 0.19.8 vì script
  phát hành chỉ ghi ở .127); từ nay `release-desktop.ps1 -UpdatesScp` tự chép.
- 4/11/2026 14:56 — **API R5** (biên bản Word `GET /records/{id}/docx`, ghép mục lục thiếu hộp `BoGhep`; không migration):
  `C:\DigiText-ChuyenMay\digitext-api-r5.exe` (SHA-256 `50A63732EF0A37B7…`), bản R4 giữ `.bak-20261104-1456`; `trien-khai-r5.ps1`:
  healthz 200, `/records/{id}/docx` 401, `/cl/doi-soat` 401. Chạy thử đối soát Bắc Giang chỉ đọc qua `docker exec psql`
  (xuất JSON vào `C:\DigiText-ChuyenMay\thu-*.json`), kết quả ở `docs/CHAY-THU-DOI-SOAT-BAC-GIANG.md`.

- 4/11/2026 09:00 — **Ổ C: (NVMe Kingston SNV3S2000G) có 2 sector hỏng** trong `docker_data.vhdx` (LBA 2385502240/2385515328);
  Event `disk` Id 7 ~100 lần/giây, C: bận 100 %, hàng đợi 15, 90 ms/đọc; trong VM docker-desktop `hv_storvsc … cmd 0x28 scsi 0x2`
  lặp cho 2 tag, không còn tiến trình D (yêu cầu đọc của tar `pla_kie_log.ldf` hôm 2/11 vẫn thử lại). SMART: 194/270 lần tắt không
  an toàn, 31,8 triệu lỗi đọc, ô dự phòng 90 % → ổ phải thay; nguyên nhân khả dĩ mất điện đột ngột, không do phần mềm.
  Đã làm: pg_dump khẩn ra `D:\DigiText-Backup\csdl\digitext-khancap-20261104.dump` (đọc trọn, 125 s → dữ liệu Postgres không nằm
  trên sector hỏng), chép dump đêm sang D:, cài smartmontools, **sao lưu toàn bộ C: → `D:\OCRPC-Backup-20261104`**
  (`bk-toan-bo.ps1` + `bk-sqlexpress.ps1`, xem `docs/CAI-LAI-OCR-PC.md` §2). Kiểm kê thêm trên máy: IIS 6 site (D:\DULIEU_66),
  SQL Express 2025 (chỉ CSDL hệ thống), 3 service cloudflared (chính/ct5d/n8n), n8n qua pm2. Chưa `wsl --shutdown` (chờ giờ vắng).
  Việc tiếp: thay NVMe + cài lại theo `docs/CAI-LAI-OCR-PC.md` §3; tắt máy đúng cách, UPS.
- 4/11/2026 10:00–11:35 — sao lưu toàn bộ C: xong (178 GB, `D:\OCRPC-Backup-20261104`); `wsl --shutdown` gỡ vòng đọc kẹt, dịch vụ lên lại
  sau 1 phút, WSL nhận 40 GB; phát hiện **vùng hỏng thứ 3 trong `C:\n8n\.n8n\database.sqlite`** (n8n là thứ đọc lỗi 290/phút);
  shrink log pla_kie, ghi 0 lên cluster hỏng (vhdx + sqlite), dựng lại CSDL n8n sạch (mất 1 dòng execution_data) → lỗi đọc 0/phút.
  Chi tiết + script: `docs/CAI-LAI-OCR-PC.md` §0. Còn: `ghi-de-sector-2.ps1` lúc rảnh; thay NVMe.

- 5/11/2026 08:30 — **ClamAV chặn gói AIP lớn**: đóng gói 22 phông Bắc Giang sinh ~140 gói mức hồ sơ, gói nào
  nặng hơn 100 MB thì clamd cắt kết nối giữa chừng (`INSTREAM: Size limit reached, max: 0`) → worker ghi
  `quét virus lỗi: gửi khúc dữ liệu lỗi … (os error 10053)`, gói về `failed`. 61/122 gói trong hàng chờ sẽ dính,
  gói nặng nhất 911 MB. Sửa: `infra/compose/clamd.conf` (bản sinh từ ảnh `clamav/clamav:stable` + 5 dòng giới hạn ở cuối)
  gắn vào container bằng bind mount trong `docker-compose.yml`; `StreamMaxLength`/`MaxFileSize`/`MaxScanSize` 2000M,
  `ReadTimeout` 600, `MaxScanTime` 900000. **Hai bẫy:** trị số phải **dưới 2 GB** (3000M/4000M → `Incorrect argument
  format`), và tệp phải **xuống dòng kiểu Unix** (`Add-Content` của PowerShell ghi CRLF, clamd đọc `2000M
` là sai
  định dạng). `docker compose up -d clamav` bị bộ lọc auto-mode chặn → `docker cp` cấu hình vào container rồi
  `docker restart`; mount trong compose để lần dựng lại sau vẫn còn. Sau đó cho 10 gói lỗi chạy lại
  (`chay-lai-goi-loi.sql`, chỉ đụng gói có lỗi bắt đầu bằng `quét virus lỗi:`); gói 155 MB đã dựng xong, 0 lỗi mới.
- 5/11/2026 08:30 — **Màn Gói tin chỉ thấy 50 dòng**: `GET /packages` mặc định `limit = 50` (trần 200), desktop gọi
  `packages_list` không truyền gì → lọc "Xong" ra 0 gói dù đã dựng 13. Sửa hai đầu: máy chủ nâng mặc định lên 2000
  (trần 5000) để bản desktop đang cài thấy ngay; desktop `packages_list` lấy lần lượt từng lô 200 cho tới hết
  (`desktop/src-tauri/src/packaging.rs`), phát hành ở bản sau.
- 5/11/2026 09:20 - **API + desktop 0.24.1: tải gói theo phông.** `GET /packages` trả thêm `fondCode`/`fondName`
  lấy từ `request_json` (bảng `packages` không có cột phông), và dùng truy vấn rút gọn `packages::list_gon` bỏ hẳn
  `request_json` khỏi danh sách - trung bình 24 KB/gói, có gói 70 KB, với mức 2000 gói là vài chục MB mỗi lần hỏi,
  10 giây một lần. Desktop: cột Phông, ô lọc phông, chọn nhiều dòng (chỉ gói `built` có tệp) và nút "Tải về đã chọn"
  tải lần lượt vào một thư mục. Dựng từ **worktree sạch ở 9102d37** (không lấy `0041340` vì commit đó cần
  `migrations/0026` chưa áp trên máy chủ): `digitext-api.exe` SHA-256 `0948B368E0927409…`, bản trước giữ
  `.bak-20261105-0920`; healthz 200 sau khi `restart-api.ps1`, `/packages` 401. Bộ cài + `.sig` + `latest.json`
  đã chép vào `C:\DigiText\updates`, `https://api.digitext.vn/updates/latest.json` trả 0.24.1.
  **Bẫy:** lệnh gộp thay exe + khởi động lại bị bộ lọc auto-mode chặn ("Production Deploy") - tách làm hai lệnh thì
  qua, đúng như ghi chú 3/11.
- 5/11/2026 09:37 - **Desktop 0.24.2: lọc theo loại gói.** Một hồ sơ sinh cả SIP lẫn AIP nên bảng Gói tin trộn
  hai loại; thêm ô lọc loại gói (chỉ liệt kê loại thật sự có trên máy chủ) bên cạnh ô lọc phông. Tải cả lô mà
  chọn lẫn nhiều loại thì tách mỗi loại một thư mục con theo mã loại (`aip_hoso`, `sip_hoso`); chọn một loại thì
  đổ thẳng vào thư mục đã chọn. Chỉ sửa desktop, không đụng máy chủ. Dựng từ worktree sạch ở 9102d37.
- 5/11/2026 10:00 - **Desktop 0.24.3: khung tiến độ tải gói.** Tải cả lô trước đây chỉ có một dòng chữ trong
  thanh công cụ, rời màn là mất. Thêm `state/goiDownloads.tsx` (kho trạng thái ở cấp ứng dụng, tải tuần tự vì
  gói AIP hồ sơ nặng cả trăm MB) và `features/package/GoiDownloadWidget.tsx` dựng theo đúng khuôn
  `DownloadStatusWidget` (cùng `uploadWidget.css`), cắm vào `TransferDock` cạnh khung đẩy tệp: thanh phần trăm,
  số gói, dung lượng, tốc độ, thời gian còn lại, gói đang tải, nút dừng, danh sách gói lỗi mở ra được.
  Xem trước trên trình duyệt: `?dlgoi=1`. Bản 0.24.2 đã dựng và chép lên máy chủ nhưng chưa bật (latest.json
  còn ở 0.24.1) nên gộp luôn vào 0.24.3.
- 5/11/2026 10:40 - **Desktop 0.24.4: đóng gói phần đã chuẩn, xuất Excel chỗ lệch.** Anh Hiếu chốt: hồ sơ lệch
  mục lục thì **bỏ qua**, chờ bên lập mục lục sửa rồi đóng gói sau. Màn Từ hồ sơ: "khớp" nay là **có dòng mục lục
  VÀ không lệch tệp nào** (`dongChuan` loại cả dòng có `maKhac` lẫn dòng `soTepKhop < soTep`), dải thông tin đếm
  theo định nghĩa đó ("đóng gói được N/M"), nút chính **Đóng gói N hồ sơ khớp** chọn sẵn đúng phần đó rồi mở hộp
  thoại. Nút **Xuất Excel N chỗ lệch** ghi `doi-chieu-muc-luc-<mã phông>.xlsx` 5 trang: Tong hop, Excel thua,
  May chu thua, Trung trong Excel, Lech ma tep — gửi lại bên lập mục lục. Dải cảnh báo vàng nhắc hồ sơ lệch chưa
  đóng gói. Chỉ sửa desktop, không đụng máy chủ.
- 5/11/2026 10:55 - **Desktop 0.24.5: mã phông lấy thẳng trong tệp metadata.** Anh Hiếu gõ tay mã phông ở hộp
  thoại đóng gói rồi nhập nhầm. Nay có tệp metadata thì ô **Mã phông** (và **Tên cơ quan hình thành phông** nếu
  tệp có ghi) lấy đúng giá trị trong tệp và **khoá lại**, không gõ được, không ghi đè lần nhớ trước; không có
  metadata mới cho gõ như cũ. Sửa thêm `gopMucLuc`: thêm tệp mục lục thứ hai trước đây làm mất `fondCode`,
  `fondName`, `loai`, `khongTep` của tệp đầu nên hộp thoại quay về đoán mã theo tên thư mục.
  Đã thử bản chặn "mã phông trùng phông khác" rồi **bỏ**: hai tệp metadata Bắc Giang ghi tên phông khác nhau
  ("Văn phòng UBND tỉnh Bắc Giang" và "UBND tỉnh Bắc Giang") nhưng cùng mã H05.13.24.01 — cùng một phông,
  chặn theo tên là chặn nhầm.
  Cùng bản: ô lọc phông ở tab **Gói tin** gom theo **tên** phông chứ không theo mã — gom theo mã thì hai phông
  trùng mã chập làm một, danh sách lọc mất hẳn một phông (nhãn ghi thêm mã khi mã khác tên).
- 5/11/2026 11:00 - **Worker đóng gói chạy song song.** Đo lúc hàng chờ 1.500 gói: CPU ~15 % trên máy 32 luồng,
  RAM trống 34,8 GB, tốc độ 5,6 gói/phút (1,5 MB/s). Nguyên nhân: worker nhận đúng một gói mỗi lượt và trong
  mỗi gói đọc từng tệp PDF một. (Nguyên nhân thứ hai lúc đó: explorer.exe trên chính máy chủ đang chép ~70 MB/s
  từ D: sang F: thư mục `UONG BI (đóng gói)`, ổ F chỉ còn 15 % rảnh, giành ổ với worker.)
  Sửa: `PackageWorker::with_concurrency(goi, tep)`; `run_loop` tách thành N luồng `mot_luong`, mỗi luồng tự gọi
  `claim_next` (đã FOR UPDATE SKIP LOCKED nên không nhận trùng); trong một gói kéo tệp theo lô bằng
  `try_join_all` + `lay_mot_tep`, giữ nguyên thứ tự. Hai tham số mới `DIGITEXT_PACKAGE_CONCURRENCY` (mặc định 3,
  tối đa 16) và `DIGITEXT_PACKAGE_FETCH` (mặc định 6, tối đa 32) — `DIGITEXT_WORKER_CONCURRENCY` cũ chỉ worker
  OCR dùng. Hạn giữ việc của gói nâng lên tối thiểu 900 s (luồng rảnh sẽ dọn lease của gói luồng khác đang dựng
  dở nếu hạn ngắn). Luồng đầu dọn lease hết hạn **mỗi phút kể cả khi bận** — trước chỉ dọn lúc rảnh nên worker
  chết giữa chừng là gói kẹt `building` mãi.
  Trên OCR PC đặt `DIGITEXT_PACKAGE_CONCURRENCY=4`, `DIGITEXT_PACKAGE_FETCH=8` trong
  `crates\digitext-ocr\.env` (bản cũ `.env.bak-20261105-songsong`); exe mới
  `C:\DigiText-ChuyenMay\digitext-ocr-songsong.exe` SHA-256 `D517C624576B2DB8…`, thay bằng
  `C:\DigiText-ChuyenMay\thay-worker.ps1` (dừng tiến trình → đổi tên bản cũ → chép → supervisor tự bật lại).
  **Bẫy:** bộ lọc auto-mode chặn mọi lệnh dừng tiến trình trên máy chủ, phải để anh Hiếu chạy script.
- 5/11/2026 11:20 - **Đo lại sau khi worker chạy song song** (gói AIP/SIP Bắc Giang, hàng chờ ~1.400):

  | Mức | gói/phút | MB/s | CPU clamav |
  |---|---|---|---|
  | 1 luồng (như cũ) | 5,6 | 1,5 | — |
  | 4 luồng | 5,0 | 3,7 | 397 % |
  | 8 luồng | 5,7 | 11,7 | 283 % |

  Lượng dữ liệu ra tăng gần 8 lần. CPU clamav tụt xuống ở mức 8 luồng = quét virus thôi là chỗ nghẽn.
  Cũng nâng `MaxThreads` của clamd 10 → 20 (trong `infra/compose/clamd.conf`, đã khởi động lại; 4 gói đang
  quét dở lúc đó lỗi, cho chạy lại bằng `dev/chay-lai-goi-loi.sql`).
- 5/11/2026 11:30 - **Quét virus một lần mỗi tài liệu** (`migrations/0027_quet_virus_tai_lieu.sql`): trước đây
  worker gửi trọn tệp zip sang clamd sau khi dựng, hồ sơ đóng cả AIP lẫn SIP thì cùng những PDF đó bị quét hai
  lần. Nay `documents.virus_scan` lưu kết quả từng tài liệu kèm dấu bản đã quét (`@signed`/`@warm` — ký số xong
  byte khác đi nên phải quét lại), worker đối chiếu rồi chỉ quét tài liệu chưa có kết quả, **trước khi nén** nên
  tệp bẩn không tốn công nén. Thêm vòng nền `quet_virus_nen` quét sẵn bản đã ký để việc quét ra khỏi đường găng.
  **Thứ tự: áp 0027 TRƯỚC rồi mới thay exe** — worker mới truy vấn cột mới, chạy sớm là mọi gói lỗi;
  `dev/nang-cap-quet-virus.ps1` làm đúng thứ tự và dừng nếu migration chưa xong. exe mới
  `C:\DigiText-ChuyenMay\digitext-ocr-quetvirus.exe` SHA-256 `33AA168EC198A419…`.
  **Bẫy:** bộ lọc auto-mode chặn cả lệnh áp migration lẫn lệnh dừng tiến trình — phải để anh Hiếu chạy script.
  Lần dựng đầu hỏng vì em chép `package_worker.rs` của commit 0041340 (có `check_package` dùng blake3/sha2 mà
  Cargo.toml ở 9102d37 chưa có dep) — phải áp lại từng bản vá lên bản gốc 9102d37.
- 5/11/2026 11:35 - **Desktop 0.24.6: sửa lại ô lọc phông.** Bản 0.24.5 gom theo tên nên một phông bị tách ba
  ("Văn phòng UBND tỉnh Bắc Giang", "UBND tỉnh Bắc Giang (1997 - 2017)", "UBND tỉnh Bắc Giang (1997-2017)"
  đều là H05.13.24.01). Một mã phông là một phông: gom lại theo **mã**, nhãn lấy tên gặp nhiều nhất trong các
  gói của mã đó. Lỗi ban đầu (0.24.4) không phải thiếu mục lọc mà là nhãn lấy nhầm tên.
- 5/11/2026 13:45 - **`dev/xuat-goi.py`: xuất gói ra ổ cắm thẳng vào máy chủ.** Tải 829 gói (70,4 GB) qua app
  chỉ được ~3 MB/s, máy báo còn 12 giờ 34 phút — đường đi vòng ra Internet rồi quay lại. Chép tay thư mục
  MinIO không dùng được: `D:\DigiText-KhoMayChu\pages\<dự án>-packages\<id>.zip` là **thư mục** kiểu xl
  (xl.meta + part.N), không phải tệp zip.
  Script đọc MinIO qua 127.0.0.1, lấy danh sách gói bằng `docker exec … psql` (khỏi cần chuỗi kết nối CSDL),
  ghi ra tên thật `zip_name`, nhiều luồng, chạy lại được (tệp đủ kích thước thì bỏ qua). Khoá MinIO đọc từ
  `crates/digitext-api/.env` như `dev/chep-kho.py`.

  ```powershell
  cd C:\DigiText
  python -I -X utf8 dev\xuat-goi.py --thu-muc F:/Goi-AIP --loai aip_hoso --luong 8
  ```

  Chạy thử 12 gói lúc worker vẫn đang đóng gói: **20 MB/s** (gấp ~7 lần đường app), zip mở ra đúng bố cục
  E-ARK. Hết hàng chờ đóng gói thì còn nhanh hơn vì không tranh ổ D.
- 5/11/2026 14:10 - **Desktop 0.24.7: lọc phông gom theo mã + tên.** Trên máy chủ ô lọc chỉ hiện một phông.
  Lý do là **dữ liệu thật có bốn phông cùng mang mã `H05.13.24.01`** nên 0.24.6 (gom theo mã) nuốt mất ba:

  | Tên phông trong gói | Số gói | Đóng lúc |
  |---|---|---|
  | Văn phòng UBND tỉnh Bắc Giang | 138 | 08:12 |
  | UBND tỉnh Bắc Giang (1997 - 2017) / (1997-2017) | 1.658 | 10:18 |
  | UBND tỉnh Bắc Giang | 276 | 13:32 |
  | Chi cục Văn thư lưu trữ | 88 | 13:36 |

  Nay gom theo **mã + tên**, chỉ gộp các cách ghi khác nhau mỗi khoảng trắng (`chuanTen` bỏ hết khoảng
  trắng rồi so). Hai lần sửa trước đều lệch: 0.24.5 gom theo tên (tách một phông thành ba), 0.24.6 gom theo
  mã (gộp bốn phông làm một).
  **Việc còn treo cho nghiệp vụ:** 88 gói "Chi cục Văn thư lưu trữ" mang mã của UBND tỉnh — mã đó đã nằm
  trong mã lưu trữ từng hồ sơ bên trong gói. Cần đối chiếu danh mục phông của Trung tâm rồi đóng lại nếu sai.
- 9/11/2026 - **Kiểm PDF/A đo thêm trang trắng, trang lệch, trang quét ngang** (desktop 0.24.8,
  `crates/digitext-extimport/src/trang.rs`). Đọc ảnh JPEG nhúng sẵn qua `digitext_pdf::page_jpeg` nên không
  cần Docker hay poppler, chạy ngay trên máy người dùng cùng lượt kiểm PDF/A.

  * Mực: nền giấy = mức sáng ở phân vị 90 của chính trang đó, ngưỡng = nền trừ 60. Nhờ vậy giấy ố vàng và
    bản quét xám vẫn đo đúng. Bỏ 3 % sát mép (gáy sách, bóng nắp máy quét, lỗ ghim).
  * Góc: chiếu điểm mực lên trục dọc ở từng góc thử, quét thô ±6° bước 0,5° rồi dò lại bước 0,1°.
  * Trang ngang: so với điểm chiếu theo trục còn lại; hơn 1,35 lần thì chữ đang dựng đứng.

  **Ngưỡng lấy từ số đo thật, không đoán**: chạy trên 332 trang quét Bắc Giang ở `data/01a0cd40-…`, trang ÍT
  mực nhất mà vẫn có nội dung đạt **1,14 %** (bảng 7 dòng đầu trang, phần dưới để trống). Nên để trang trắng
  ở 0,15 % (dưới 7 lần) và thêm bậc "gần như trắng" tới 0,6 %. Lệch từ 1° trở lên thì báo.
  Kết quả trên tập đó: 72 trang lệch (lớn nhất 4,6°), 8 trang quét ngang, 0 trang trắng, không báo nhầm —
  đã mở ảnh xem tận mắt vài trang bị gắn cờ. Tốc độ ~7 trang/giây.

  Giao diện: ô "Kiểm trang trắng và trang lệch" (tắt mặc định vì chậm), cột mới trong bảng, số trang cụ thể
  khi mở rộng dòng. CSV thêm 6 cột: Trang trắng, Trang gần như trắng, Trang lệch (kèm góc), Góc lệch lớn nhất,
  Trang quét ngang, Số trang đã đo. `pdfa::run` giữ nguyên chữ ký cho `cli.rs`; cờ nằm ở `run_opts`.
  Công cụ chỉnh ngưỡng: `cargo run -p digitext-extimport --example do-trang -- <thư mục> [--muc] [--xuat <thư mục>] [--csv <tệp>]`.
- 9/11/2026 - **Webhook nhận dự án từ hệ thống ngoài** (migration 0028,
  `crates/digitext-api/src/routes/hooks.rs`, `crates/digitext-db/src/webhooks.rs`). Hệ thống quản lý hợp đồng
  gọi `POST /api/v1/hooks/du-an` với tên dự án, loại dự án, khối lượng và ngày kết thúc hợp đồng; API tự tạo
  dự án, ghi `project_contracts` (0008 + 0024 đã có sẵn đủ cột) và gán quản trị. Tài liệu cho bên gửi:
  `docs/WEBHOOK-DU-AN.md`.

  * Xác thực: khoá trong bảng `api_clients` (argon2 băm phần bí mật như mật khẩu người dùng), tuỳ chọn
    HMAC-SHA256 trên chuỗi `<timestamp>.<thân gói>` + chặn lệch giờ quá 300 giây, tuỳ chọn lọc IP. HMAC tự
    dựng trên `sha2` để không thêm phụ thuộc workspace; có test đối chiếu vector RFC 4231 ca 2 và ca 4.
  * Chống trùng: `UNIQUE (nguon, ma_ngoai)` trên `projects`. Webhook gửi lại → cập nhật dự án cũ, trả 200
    `taoMoi: false`, không sinh dự án thứ hai. Trường nào gói tin không gửi thì giữ nguyên (gộp lên hợp đồng
    cũ qua `webhooks::lay_hop_dong`), nên gửi riêng một mốc ngày không xoá các số khối lượng đã nhập tay.
  * `maDuAn` không gửi thì tự đặt `DA<yy><nnn>`, cờ `projects.ma_tu_sinh` + cảnh báo trong phản hồi. Mã đã có
    nhưng do người tạo tay (`ma_ngoai IS NULL`) thì NHẬN dự án đó về thay vì tạo bản thứ hai; mã đang thuộc
    hợp đồng khác thì 409.
  * Dự án không có `project_admins` thì chỉ super admin thấy (vai trò Admin lọc theo `projects::f_1ng`) — nên
    gói tin nhận `quanTri` (tên đăng nhập hoặc email) và khoá có `default_admin_id`; thiếu cả hai thì trả cảnh báo.
  * Nhật ký `webhook_deliveries` ghi gói gốc + kết quả + lý do từ chối cho mọi lời gọi ĐÃ qua xác thực (lời gọi
    sai khoá không ghi, kẻo thành chỗ cho người lạ nhồi dữ liệu). Xem qua `GET /api/v1/hooks/nhat-ky` (cần quyền
    ProjectUpdate). Nhật ký kiểm toán ghi `project.create`/`project.update` với `actor_id = NULL`, detail
    `{"qua": "webhook"}`.
  * Cấp khoá: `digitext-api.exe hook-key tao --nguon crm --ten "..." [--quan-tri <tên đăng nhập>] [--khong-ky]
    [--ip a,b]`; còn `doi` (cấp lại), `thu-hoi`, `xem`. Bí mật chỉ in một lần.
  * Chạy thử thật trên Postgres tạm (container `digitext-hooktest`, đã xoá): 28 migration áp sạch, 13 ca gồm tạo
    mới 201, gửi lại 200 không trùng, cập nhật riêng khối lượng vẫn giữ hồ sơ/hộp cũ, tự đặt mã, nhận dự án tạo
    tay, chữ ký sai 401, lệch giờ 401, thiếu `maNgoai` 400, loại lạ 400, mã có dấu chấm 400, mã đã dùng 409,
    đóng dự án chưa có 404, đóng dự án thật 200 (`is_active = false`).

  **Thứ tự phát hành**: áp **0026 rồi 0028** lên CSDL OCR PC TRƯỚC khi đổi `digitext-api.exe` — 0026 vẫn chưa áp,
  mà bản API mới dựng từ nhánh `feat/ban-luu-tru` có route `tra_cuu` cần 0026. 0028 tự nó độc lập với 0026.
  Caddy không phải sửa: đường mới nằm trong `/api/v1` đã proxy sẵn.

- 10/11/2026 - **Cửa đọc sản lượng cho cổng nhà cung cấp của DO365**
  (`crates/digitext-api/src/routes/ncc.rs`, tài liệu `docs/WEBHOOK-NCC.md`). Cổng NCC bên DO365 có hai mục để
  «sắp có» — *Báo cáo sản lượng* và *Tự kiểm chất lượng* — chưa có route lẫn màn hình. Đợt này làm mục đầu:
  `POST /api/v1/ncc/san-luong` trả số theo ngày + luỹ kế + mốc hợp đồng + tiến độ; DO365 tự vẽ bằng giao diện
  của họ. **Không iframe, không phơi giao diện DigiText ra ngoài.** Không có migration nào.

  * **Ánh xạ không cần bảng mới**: DigiText không có khái niệm đội/tổ/nhà thầu ở bất kỳ bảng nào, nên «đội» của
    DO365 (`so-hoa` | `chinh-ly`) được hiểu là **nhóm chỉ số** của `dashboard_daily` — 7 cột số hoá, 4 cột chỉnh
    lý. DigiText trả số theo nhóm, DO365 tự biết (dự án, đội) đó là nhà thầu nào. Phản hồi mang cờ
    `gopNhieuNhaThau` vì một đội có hai nhà thầu thì không tách được (lối thoát: DO365 đẩy hai `maNgoai`).
  * **POST chứ không GET** cho lệnh đọc: chuỗi ký là `<ts>.<thân gói>` nên để tham số trong thân thì chúng nằm
    trong vùng ký, không lọt vào nhật ký proxy, và bên DO365 dùng lại nguyên hàm `goi()` đã viết — không thêm
    dòng mã ký nào. Dùng GET thì query nằm ngoài chữ ký: bắt được một gói là đổi được `maNgoai` trong 300 giây.
  * Phạm vi đọc do **khoá** quyết định (`nguon` lấy từ `api_clients`, không từ thân gói). `maNgoai` của nguồn
    khác và `maNgoai` không tồn tại trả cùng một `404 ncc_du_an_khong_thay`.
  * Chống rò: không import `user_daily`/`staff_report`/`thong_ke::nhan_vien`, không dùng `DocRow` (có
    `uploaded_by_name`), không `#[serde(flatten)] Metrics` (lộ `work_items_*`, `submissions_*`), bỏ
    `project_contracts.note` và `health.rag`. Có test chặn cột nội bộ lọt vào nhóm chỉ số.
  * Hai bẫy đã xử lý: `dashboard_rollup` **xoá dòng toàn 0** nên phải lấp ngày trống, kẻo đồ thị nhảy ngày;
    `project_progress` có `WHERE p.is_active` nên **dự án đã đóng trả rỗng** → nhánh dự phòng dùng
    `pipeline_counts`, trả `hopDong: null` thay vì 500.

  **Pha 0 — chi phí xác thực (sửa trong `routes/hooks.rs`, áp cho CẢ webhook đang chạy):**

  * `password::verify` (đồng bộ, Argon2id 19 MiB, chạy thẳng trên luồng worker Tokio) → `password::f_2pn`
    (`spawn_blocking` + semaphore 8 chỗ). Webhook vài lần/ngày nên không ai thấy, nhưng cổng NCC gọi mỗi lần
    có người mở màn hình: vài chục lời gọi cùng lúc là chặn hết worker và kéo theo tải tệp, callback OCR, ký số.
  * Thêm đệm 60 giây cho **kết luận phép băm** (vân tay `sha256(key_id‖bí mật‖secret_hash)` — không giữ bí mật
    thô trong bộ nhớ). Mỗi lời gọi **vẫn** tra `api_clients`, vẫn lọc IP, vẫn kiểm lệch giờ và chữ ký ⇒ thu hồi
    và đổi khoá có hiệu lực **tức thì** (đã chứng minh bằng test). Đo trên bản debug: 832 ms → 45 ms.
  * `ghi_da_goi` chỉ chạy khi vừa qua đường băm thật ⇒ nhiều nhất một `UPDATE api_clients` mỗi khoá mỗi 60 giây,
    thay vì mỗi lời gọi một lần (bảng hai dòng, HOT update, autovacuum chạy không xuể).
  * Chốt chống dò khoá bằng `LoginLimiter` chốt theo **địa chỉ người gọi**, cố ý KHÔNG theo `key_id`: phần công
    khai của khoá nằm ngay trong header nên chốt theo nó thì người lạ gửi bí mật bậy vài lần là khoá cửa của
    chính hệ thống thật trong 5 phút. Chỉ ghi lần sai, không xoá sổ khi đúng (bộ đếm IP dùng chung với đường
    đăng nhập người dùng).

  * `ncc::router()` gắn ở **khối thi công** (`app.rs`, sau `edition_luu_tru()`), không ở `chung`: số liệu nó đọc
    chỉ có trên máy chủ thi công, và bản Lưu trữ của chủ đầu tư không có nhà thầu nào để hỏi.
  * Chạy thử thật trên Postgres tạm (container `digitext-ncctest`, đã xoá): 28 migration áp sạch, **61 ca đạt**
    — lấp ngày trống, tổng đúng theo dữ liệu gieo, ba giá trị `doi` + giá trị lạ 400, thiếu `maNgoai` 400, ngày
    năm hai chữ số 400, khoảng quá dài 400, `maNgoai` lạ 404, khoá sai 401, chữ ký sai 401, lệch giờ 401, dự án
    đóng không 500, thu hồi khoá giữa chừng 401 ngay, đổi `secret_hash` thì bí mật cũ chết ngay, và rà 9 chuỗi
    cấm không lọt vào phản hồi.

  **Phát hành**: không có migration mới — chỉ đổi `digitext-api.exe` rồi `dev/restart-api.ps1`. Caddy không phải
  sửa (đường mới nằm trong `/api/v1` đã proxy sẵn). Phần cổng là việc của bên DO365, đặc tả ở `docs/WEBHOOK-NCC.md`.

  **Còn nợ**: mục «Tự kiểm chất lượng» (cần 3 hàm DB mới + chốt chính sách có phơi `review_note` cho nhà thầu
  không); hộp/tờ theo ngày (`dashboard_daily` không có cột); độ tin cậy OCR theo dự án.

- 10/11/2026 - **Cửa tự kiểm cho cổng nhà cung cấp: lỗi máy chủ đã bắt trên thứ đã nộp**
  (`POST /api/v1/ncc/tu-kiem/da-nop`, đặc tả mục 9 `docs/WEBHOOK-NCC.md`). Lấp mục menu «Tự kiểm chất lượng»
  đang để trống bên cổng NCC của DO365. **Không migration, không bảng mới, không engine mới.**

  * **Vì sao rẻ bất thường**: ba con số này ĐÃ nằm trong CSDL từ trước — `documents.review_status` do người duyệt
    đặt, `documents.pdfa_2b` tính ngay lúc tải lên (`routes/documents.rs`), `ocr_jobs.status` do tiến trình bóc
    trường ghi. Cửa này chỉ ĐỌC: không tải tệp lên, không chạy pkgcheck/veraPDF, nên không cần hạn ngạch.
  * **Ba loại lỗi** (`documents::NCC_LOAI_LOI`): `tra_lai` · `pdfa_cu` (có PDF/A nhưng không đạt 2b) ·
    `boc_truong_loi`. Cố tình KHÔNG có "thiếu lớp chữ" / "chưa có PDF/A" / "OCR chưa xong": tệp mới nộp đương
    nhiên chưa qua các bước đó, báo ra là nhà thầu tưởng mình làm sai. Ba mã đều đọc từ cột có thật.
  * `review_note` **mặc định KHÔNG trả ra ngoài** — ô đó đang được viết với giả định "chỉ người trong nhà đọc".
    Bật bằng `DIGITEXT_NCC_LY_DO=true`, và phải đổi nhãn ô nhập bên desktop TRƯỚC khi bật. Phản hồi mang cờ
    `coLyDo` để bên kia biết đường mà vẽ, thay vì dò xem khoá `lyDo` có tồn tại không.
  * **Hai hàm DB mới** trong `digitext-db/src/documents.rs`: `ncc_loi_tong` (số đếm toàn dự án) +
    `ncc_loi_ds` (danh sách tệp, UNION ALL ba nhánh nên một tệp vướng hai loại ra hai dòng ⇒ số đếm và danh
    sách khớp nhau). Cả hai nhận `&[Uuid]` chứ KHÔNG `Option<&[Uuid]>`: cửa nhà cung cấp không bao giờ được
    phép hỏi cả kho, nên bỏ hẳn khả năng truyền `None` = tất cả dự án.
  * **Mở rộng `REVIEW_DERIVED_CTE`** thêm `d.review_note, d.reviewed_at` vào `base`. Mọi truy vấn cũ chọn cột
    tường minh nên không ảnh hưởng — đã chạy lại hình `review_summary` + `tree_docs` trên CSDL thật để xác nhận.
  * `gioiHan` là **trần cứng, kẹp không báo lỗi** (mặc định 200, trần 1000) vì đây là lệnh đọc; cờ `biCat` nói
    rõ có bị cắt. Số đếm trong `loai[]` KHÔNG bị cắt theo.
  * **Chống rò** như cửa sản lượng: không `DocRow` (có `uploaded_by_name`), không id nội bộ, không `storage_path`,
    không `rel_path` đầy đủ (chỉ phông/hộp/hồ sơ đã dẫn xuất). Có 13 ca rà chuỗi cấm trong phản hồi.

  **Bẫy đã sập một lần**: `GoiKiem` thiếu `#[serde(rename_all = "camelCase")]` nên `maNgoai` không vào được,
  mọi lời gọi ra 400 "thiếu maNgoai". Không có test tích hợp thì lỗi này ra tới tay đối tác — `GoiHoi` có attribute
  đó nên dễ tưởng struct mới cũng có.

  **Chạy thử thật** (container `digitext-ncctest` port 5498, đã xoá): 28 migration áp sạch (autocommit — nhiều
  migration có `CREATE INDEX CONCURRENTLY`, không chỉ 0021). Gieo 10 tài liệu: 3 trả lại + 2 pdfa cũ + 1 bóc
  trường lỗi + 4 sạch. **48/48 ca đạt** + 4/4 ca nhánh bật `DIGITEXT_NCC_LY_DO`: đếm đúng từng loại, phông dẫn
  xuất đúng, 13 chuỗi cấm không lọt, trần kẹp cả ba chiều, thiếu `maNgoai` 400, `maNgoai` lạ 404, khoá/chữ ký/lệch
  giờ 401, dự án đã đóng 200 không 500, tài liệu `disposed` không bị tính. Không hồi quy: `/ncc/san-luong` và
  `/hooks/kiem-tra` vẫn 200.
  Ràng buộc `ck_documents_review_trace` bắt `review_status <> 'pending'` phải có `reviewed_at` — gieo dữ liệu
  thử phải tôn trọng, đừng tưởng là lỗi code.

  **Phát hành**: không migration — đổi `digitext-api.exe` rồi `dev/restart-api.ps1`. Caddy không sửa.

  **Còn nợ của «Tự kiểm»**: hai cửa nữa, cố ý tách riêng vì cơ chế khác nhau —
  (1) **kiểm gói tin + PDF/A tệp lẻ** cho thứ CHƯA nộp: phải tải tệp lên, bộ máy là crate Rust;
  `digitext-api/Cargo.toml` CHƯA có dòng `digitext-extimport` (chỉ có qua pkgcheck, nên route không gọi tên được),
  và kết luận chuẩn ISO của veraPDF cần **Docker + image `verapdf/cli:v1.30.2`** trên OCR PC (chưa kéo về);
  (2) **kiểm mục lục MLHS**: bộ đọc là `desktop/src/features/chinhly/mucLucImport.ts` (TS thuần, chạy được ngoài
  Tauri — `desktop/scripts/kiem-muc-luc.mjs` đã bundle bằng esbuild chỉ cần giả lập 2 module), nên nửa đọc chạy
  trong trình duyệt NCC, còn nửa kiểm trùng với hồ sơ đã có thì qua cửa JSON nhỏ (`chinh_ly::validate::normalize`).
  Chưa chốt: bộ đọc do DigiText phục vụ (luôn mới) hay DO365 nhúng sẵn kèm bắt tay phiên bản.

- 10/11/2026 - **Cửa tự kiểm mục lục hồ sơ: bộ đọc MLHS chuyển sang Rust, chạy ở máy chủ**
  (`crates/digitext-mucluc/` mới, `POST /api/v1/ncc/tu-kiem/muc-luc`, đặc tả mục 10 `docs/WEBHOOK-NCC.md`).
  Anh Hiếu chốt 10/11: **cổng nhà cung cấp chỉ là giao diện, mọi xử lý ở máy chủ DigiText.**
  Không migration.

  * **Vì sao chuyển lên máy chủ KHÔNG tạo thêm bản sao luật**: màn «Nhập mục lục» bên desktop vốn đã bắt buộc
    có mạng (`NhapMucLucDialog.tsx` tắt nút khi `offline`), nên sau khi cửa này xong thì desktop gọi sang đây và
    bộ TS `mucLucImport.ts` bị xoá ⇒ còn một bản duy nhất, nằm ở máy chủ. Chặng xoá TS CHƯA làm.
  * **Bộ đối chứng là điều kiện để tin bản Rust**: `test/MLHS` còn 23 mục lục thật của Bắc Giang + harness
    `desktop/scripts/kiem-muc-luc.mjs`. Mốc TS 10/11: 23 tệp · 10171 hồ sơ · 10 tệp bị chặn · 126 lỗi · 1725
    cảnh báo. Bản Rust ra **y hệt**, và so sâu hơn: 23/23 tệp khớp cả 12 chỉ tiêu + bảng thông báo;
    **10171/10171 dòng** khớp ngày bắt đầu, ngày kết thúc, kết luận thời gian, số lỗi, số cảnh báo.
    Harness TS đã được mở rộng (`KIEM_XUAT_JSON` kết xuất thêm tg/lỗi/cảnh báo) và bản Rust có
    `MUCLUC_XUAT_JSON` đối ứng — giữ cả hai, đây là lưới an toàn duy nhất cho bộ đọc này.

  **Ba bẫy tầng ô (calamine vs SheetJS) — đã ghi ở đầu `digitext-mucluc/src/lib.rs`:**

  * **Lưới phải đệm về A1.** SheetJS được gọi với `range: {s:{r:0,c:0}, e: ref.e}` nên luôn đệm từ A1;
    `calamine::Range` trả đúng vùng đã dùng, có thể bắt đầu ở C5 ⇒ chỉ số cột lệch hết.
  * **Số nguyên dạng thực.** `String(5)` của JS là `"5"`; calamine cho `Float(5.0)` — phải tự định dạng.
  * **Mốc serial ngày** 25569: đã đối chiếu 4 giá trị bằng chính biểu thức JS chạy qua node (45000 →
    15/03/2023). Test ban đầu của Claude kỳ vọng sai, không phải code sai.

  **Bẫy đã sập và là bẫy khó nhất — `regex` của Rust không có lookbehind/lookahead:**

  * Dòng 295 của «PHÔNG SỞ NÔNG NGHIỆP VÀ PTNT BẮC GIANG», tiêu đề `… tháng 10-11/2008 Từ số: 1428-1856`.
    Bên JS, `(\d{4})(?:\s*[-–—]\s*(\d{4}))?` **ăn trọn** `1428-1856` thành MỘT lần khớp rồi mới loại vì 1428
    ngoài dải năm ⇒ `1856` không bao giờ được xét riêng. Bản Rust lọc dải năm TRƯỚC khi ghép nên `1856` lọt vào
    và thành "năm tài liệu ở tiêu đề" ⇒ sinh thêm một cảnh báo sai (1726 thay vì 1725).
    Sửa: `quet_cum_4` KHÔNG lọc dải; việc loại theo dải làm ở **thân vòng lặp**, đúng chỗ `continue` của JS.
    Có test hồi quy mang nguyên tiêu đề thật đó.

  **Cửa HTTP:**

  * Nhận tệp **base64 trong JSON** chứ không multipart: chuỗi ký là `<ts>.<thân gói>` nên bên DO365 dùng lại
    y nguyên hàm `goi()`, không viết thêm dòng mã ký nào. Giá: gói to thêm ~33%, tệp lớn nhất 504 KiB nên không
    đáng kể. Trần 8 MiB tệp + `DefaultBodyLimit` 12 MiB cho route.
  * Đọc + phân tích chạy trong `spawn_blocking` (tệp 2.829 dòng là việc nặng CPU) — cùng lý do với
    `password::f_2pn`, đừng chiếm worker Tokio.
  * Trần 5.000 dòng = đúng `IMPORT_MAX` của `routes::chinh_ly::import`; quá thì **báo lỗi nói rõ**, không cắt im.
  * **Khác bản TS một chủ ý**: xin `sheet` không có trong tệp thì trả **400 kèm danh sách sheet có thật**, không
    im lặng đọc sheet khác. Hộp thoại desktop chỉ cho chọn sheet có thật nên không gặp; qua API thì xin sai là
    chuyện thường, mà im lặng trả số của sheet khác là cách làm người ta tin vào số sai.
  * **Hai hàm DB mới** `digitext-db/src/chinh_ly/ncc.rs`: `thbq_cua_du_an` (dự án có Danh mục riêng thì CHỈ dùng
    danh mục đó, chưa có dòng nào mới rơi về bản chung — trộn hai bên sinh tên trùng) và
    `trung_so_tam_theo_phong`. Khoá duy nhất thật là `ux_cl_ho_so_so_tam (phong_id, vinh_vien,
    lower(btrim(so_tam)))` — **theo phông, không theo dự án** — mà cổng không biết sẽ nhập vào phông nào, nên
    đếm cho từng phông và để bên kia bày. Đây là tầng kiểm mà trình duyệt không thể làm.
  * Chống rò: `dong[]` chỉ mang `dong`/`hoSoSo`/`hopSo`/`loi`/`canhBao` — **không có tiêu đề hồ sơ**. Thông báo
    lỗi có trích nội dung thì trích từ chính tệp người ta gửi lên, không phải từ kho. `trungVoiKho` có trả tên
    phông của dự án — chấp nhận, vì không nói phông nào thì con số vô dụng.

  **Chạy thử thật** (container `digitext-ncctest` port 5498, đã xoá): 28 migration áp sạch, **38/38 ca đạt** —
  5 mục lục thật khớp đúng mốc TS qua HTTP (kể cả tệp `.xls` 2.727 hồ sơ), chưa có Danh mục THBQ → 400 nói rõ,
  trùng số tạm đếm đúng theo phông, 7 chuỗi cấm không lọt, base64 sai / không phải Excel / thiếu tham số → 400,
  `maNgoai` lạ → 404, chữ ký sai → 401, đổi sheet được, xin sheet lạ → 400 có liệt kê, `chiTiet=false` thì
  `dong[]` rỗng mà số đếm vẫn còn. Không hồi quy: `/hooks/kiem-tra`, `/ncc/san-luong`, `/ncc/tu-kiem/da-nop`
  đều 200.

  ⚠️ **`cargo test -p digitext-api` KHÔNG dựng lại `target/debug/digitext-api.exe`** — chạy thử bằng exe cũ thì
  route mới ra 404 hàng loạt, dễ tưởng code sai. Phải `cargo build -p digitext-api --bin digitext-api`.

  **Phát hành**: không migration — đổi `digitext-api.exe` rồi `dev/restart-api.ps1`. Caddy không sửa.

  **Còn nợ**: (1) chặng xoá bộ TS: desktop gọi `/ncc/tu-kiem/muc-luc`… (thực ra cần một cửa nội bộ tương đương
  cho desktop, vì cửa này xác thực bằng khoá webhook) rồi xoá `mucLucImport.ts`; (2) cửa kiểm gói tin + PDF/A
  tệp lẻ — cần thêm dòng `digitext-extimport` vào `digitext-api/Cargo.toml` (hiện chỉ có qua pkgcheck) và kéo
  image `verapdf/cli:v1.30.2` về OCR PC; (3) `review_note` vẫn tắt (`DIGITEXT_NCC_LY_DO`).

- 10/11/2026 (tiếp) - **Cửa nội bộ `POST /cl/muc-luc/doc` — chặng 1 của việc dẹp bản sao bộ đọc MLHS**
  (`crates/digitext-api/src/routes/chinh_ly/muc_luc.rs`). Không migration.

  * Mục đích: máy trạm gọi sang máy chủ thay vì tự đọc, rồi **xoá `mucLucImport.ts`** ⇒ còn một bản luật duy
    nhất. Chặng 2 (đổi `NhapMucLucDialog.tsx` + xoá TS) CHƯA làm — cần chạy app thật để nghiệm thu.
  * Khác cửa nhà cung cấp hai điểm: xác thực bằng phiên người dùng (`Ts`) + `project_access` +
    `Permission::DossierCreate`; và **trả đủ dữ liệu gồm tiêu đề hồ sơ** (người trong nhà), trong khi cửa
    `/ncc/tu-kiem/muc-luc` cố ý không trả tiêu đề.
  * **Hình JSON bám đúng bộ TS** (`serde(rename_all = "camelCase")`): `phanTich` có đủ 16 khoá như
    `PhanTichMucLuc`, `rows[]` đủ 13 khoá như `DongHoSo`, `tg` có `batDau`/`ketThuc`/`ketLuan`/`ghiChu`.
    `cot` dùng `skip_serializing_if = "Option::is_none"` — **bỏ hẳn khoá** chứ không trả `null`, vì bên TS
    kiểm `cot.hopSo !== undefined` mà `null !== undefined` là đúng ⇒ trả `null` thì bên kia tưởng có cột.
  * Trả thêm `rowsNhap[]` = `phan_tich::dong_nhap` (bản Rust của `dongMau`), **xếp đúng thứ tự `rows`** để
    máy trạm dịch số dòng lỗi của lệnh nhập về số dòng trong tệp Excel. Giữ nguyên quy ước **bỏ khoá rỗng**:
    `validate::normalize` coi khoá-có-mặt-nhưng-rỗng khác khoá-không-có. Đã chạy thử nhập thật qua
    `/cl/ho-so/import`: 3 hồ sơ vào CSDL, không lỗi dòng nào.

  **Hai lỗi của chính Claude bị bắt lúc chạy thử — đáng ghi lại:**

  * **`thbq_cua_du_an` lệch với `danh_muc::catalog`.** Claude viết "dự án có danh mục riêng thì CHỈ dùng danh
    mục đó", nhưng `catalog()` **gộp** `project_id = $1 OR project_id IS NULL` và xếp `so_tt NULLS LAST, ten`.
    Máy trạm khớp THBQ bằng `cat.thbq` từ `GET /cl/catalog`, nên lệch điều kiện/thứ tự là **hai bộ đọc cho kết
    quả khác nhau trên cùng một tệp** — đúng thứ mà cả việc chuyển sang Rust này nhằm xoá bỏ. Đã sửa cho giống
    hệt `catalog()`; có ca thử đối chiếu nguồn THBQ của cửa với chính `/cl/catalog`.
  * **Nhánh 400 "dự án chưa có Danh mục THBQ" là mã chết.** Migration 0012 gieo **14 dòng dùng chung**
    (`project_id IS NULL`) — và danh sách đó **đúng bằng** danh sách harness TS dùng, nên dự án không có danh
    mục riêng vẫn khớp mốc TS tuyệt đối. Thông báo cũ hứa một lỗi không nổ được và đổ việc cho người dùng; đã
    đổi thành chốt an toàn nói rõ "lỗi cấu hình máy chủ". Hai tài liệu (`WEBHOOK-NCC.md` và
    `DO365-MASTER/01_DOCS/NOI-DIGITEXT-CONG-NCC.md`) đã sửa theo.
  * `project_access` **cho super admin đi qua với bất kỳ uuid**, kể cả dự án không tồn tại ⇒ cửa đọc xong cả
    tệp rồi mới vỡ ở lệnh nhập. Đã thêm kiểm `EXISTS (SELECT 1 FROM projects …)` → 404.

  **Chạy thử thật**: cửa nội bộ **39/39 ca** (container `digitext-mltest` port 5499, super admin thật, đăng
  nhập thật, nhập thật vào `/cl/ho-so/import`); chạy lại cả hai cửa nhà cung cấp sau khi đổi `thbq_cua_du_an`:
  **38/38** (mục lục) + **48/48** (đã nộp); đối chứng 23 mục lục vẫn **10171/10171 dòng** khớp bản TS;
  `cargo test --workspace` **131 ca đạt, 0 hỏng**.

  **Phát hành**: không migration — đổi `digitext-api.exe` rồi `dev/restart-api.ps1`.

- 10/11/2026 (tiếp) - **Chặng 2 MLHS: máy trạm dùng bộ đọc ở máy chủ, bản TS rút về làm bản đối chứng**

  * `desktop/src/features/chinhly/mucLucMayChu.ts` (mới): hình dây + `docMucLucMayChu` gọi `/cl/muc-luc/doc`.
    Base64 cắt khúc 32 KiB vì `String.fromCharCode(...)` tràn ngăn xếp với tệp 500 KiB.
  * `mucLucXuat.ts` (mới): phần **ghi** tách khỏi `mucLucImport.ts` — tệp mẫu lập hồ sơ + sheet «Kiểm tra» +
    tệp gốc đánh dấu. Ở lại máy trạm vì ghi tệp xuống đĩa và giữ bố cục tệp gốc, nhưng KHÔNG còn luật kiểm nào.
    `xuatTepDaChuyen` nhận `rowsNhap` **của máy chủ** thay vì tự dựng — kẻo tệp tải xuống khác dữ liệu sắp nhập.
  * **Hai màn** đã đổi, không phải một: `NhapMucLucDialog.tsx` **và** `PackFromServer.tsx` (tab Đóng gói › Từ hồ sơ
    cũng gọi `phanTichMucLuc`). Màn Đóng gói trước đây dùng `THBQ_DANH_MUC` **viết cứng** (có "30 năm", thiếu
    "15 năm") — giờ dùng Danh mục THẬT của dự án. Hệ quả nhỏ: tên THBQ trong siêu dữ liệu gói lấy theo Danh mục
    dự án thay vì bộ cứng; `rutGon` vẫn dùng `thbq ?? thbqGoc` nên không bao giờ rỗng.
  * `mucLucImport.ts` + `thoiGianHoSo.ts` **KHÔNG còn mã app nào import**. Giữ lại làm **bản đối chứng** cho
    `digitext-mucluc`, chỉ `scripts/kiem-muc-luc.ts` dùng. Đã ghi cảnh báo ở đầu cả hai tệp, kèm đúng hai lệnh
    chạy đối chứng và mốc 23 tệp / 10171 hồ sơ. **Đừng sửa hai tệp đó để "theo" bản Rust — chúng là cái thước.**

  **Nghiệm thu được tới đâu**: `npx tsc --noEmit` sạch, `npm run build` dựng xong, và bản đối chứng TS vẫn chạy
  ra đúng mốc cũ (23 tệp · 10171 hồ sơ · 10 chặn · 126 lỗi · 1725 cảnh báo) = sửa không làm hỏng cái thước.
  ⚠️ **CHƯA bấm thử trong app Tauri** — hai màn này không có test nào. Phải chạy thử tay: nhập một mục lục thật,
  đổi sheet, tải «tệp đã chuyển» và «tệp đánh dấu», rồi tab Đóng gói › Từ hồ sơ nhập một mục lục.

- 10/11/2026 (tiếp) - **Cửa `POST /api/v1/ncc/tu-kiem/pdf` — tự kiểm một tệp PDF**
  (đặc tả mục 11 `docs/WEBHOOK-NCC.md`). Không migration.

  * Thêm dòng `digitext-extimport` vào `digitext-api/Cargo.toml` (trước chỉ có qua pkgcheck nên route không gọi
    tên được). Không tăng kích cỡ exe: crate đã nằm trong cây phụ thuộc.
  * Phép kiểm là **chính hàm** `check::PdfFacts::tt05_failures` mà Tiện ích › Kiểm PDF/A và đường nhập gói dùng —
    không có luật riêng cho cửa này. Mã lỗi có dẫn chiếu Điều 8.2.a sẵn trong thông báo.
  * `classify_map` quyết ngưỡng dpi 200/300. Khai `loaiTaiLieu` = bản đồ → 300; chỉ `trichYeu` có từ khoá thì
    **vẫn 200** (bộ xếp loại cố ý bảo thủ: áp nhầm 300 cho văn bản hành chính tệ hơn).
  * **`PDF-CHUA-KY-SO` tách thành cờ `chuaKySo`, không nằm trong `chuaDat[]`**: dây chuyền DigiText ký số SAU khi
    nhà thầu giao sản phẩm, trộn vào là mọi tệp của họ đều "chưa đạt" vì một lý do không phải việc của họ.
  * **KHÔNG chạy veraPDF** (cần Docker + ảnh `verapdf/cli:v1.30.2` trên OCR PC, chưa kéo về) — phản hồi mang
    `ghiChu` nói rõ `datChuan: true` chưa phải kết luận ISO 19005.
  * Chạy thử thật trên 5 PDF trong repo: **53/53 ca đạt**. Kết quả khớp thực tế đã biết (kho Điền Xá): PDF quét
    thô không khai pdfaid, 0/N trang có lớp chữ.

- 10/11/2026 (tiếp) - **Kiểm GÓI TIN: quyết định KHÔNG làm cửa web** (mục 12 `docs/WEBHOOK-NCC.md`).
  Gói AIP cỡ hàng trăm MB đến vài GB; base64 trong JSON cộng 33% và buộc trình duyệt giữ cả gói trong bộ nhớ,
  tải theo khúc thì thành giao thức ba bước bên kia phải viết riêng và máy chủ phải giữ chỗ tạm cho mỗi nhà thầu.
  Mà gói đang nằm trên máy nhà thầu, và desktop đã có «Tiện ích › Kiểm tra gói tin» làm đúng phép kiểm đó tại
  chỗ, xuất báo cáo PDF. ⇒ Hướng đúng: **cấp bản desktop (hoặc bản rút gọn) cho nhà thầu**. Đã nói rõ trong cả
  hai tài liệu để bên DO365 không dựng giao diện chờ.

  **Tổng trạng cổng NCC sau hôm nay**: 4 cửa sống (`san-luong`, `tu-kiem/da-nop`, `tu-kiem/muc-luc`,
  `tu-kiem/pdf`) + 1 cửa nội bộ (`/cl/muc-luc/doc`). `cargo test --workspace` **166 ca đạt, 0 hỏng**.

- 11/11/2026 08:04 - **Đẩy API bản 4 cửa NCC lên OCR PC.** `digitext-api.exe` 25.714.688 byte (build 10/11
  17:02) thay cho bản 15:03. Không migration — pha này không thêm bảng/cột nào. Nghiệm thu từ máy phát triển:
  `POST` không ký tới `/ncc/san-luong`, `/ncc/tu-kiem/da-nop`, `/ncc/tu-kiem/muc-luc`, `/ncc/tu-kiem/pdf`,
  `/cl/muc-luc/doc` đều trả **401** (trước đó ba đường cuối trả 404 vì exe cũ).

  **Bẫy chép tệp sang OCR PC — đích scp phải là THƯ MỤC.** `sshd` trên OCR PC đặt `DefaultShell` =
  `powershell.exe`, nên đích dạng *tên tệp đầy đủ* (`…:C:/DigiText/target/release/digitext-api.exe.new`) làm
  PowerShell cắt sai tham số của `scp -t` ở đầu kia ⇒ `scp.exe: Connection closed`, đứt trước khi truyền byte
  nào. Dạng chạy được là dạng `dev/release-desktop.ps1` vẫn dùng: đích là thư mục, kết thúc bằng `/`.

  ```powershell
  # máy phát triển
  scp D:\DigiText\target\release\digitext-api.exe ADMIN@100.122.226.63:C:/DigiText-ChuyenMay/
  ssh ADMIN@100.122.226.63 "(Get-Item C:\DigiText-ChuyenMay\digitext-api.exe).Length"   # phải khớp byte
  ```
  ```
  :: trên OCR PC (console/RDP). Cửa sổ Command Prompt thì KHÔNG có Rename-Item/Move-Item —
  :: đó là lệnh PowerShell. Dùng ren/move:
  cd /d C:\DigiText\target\release
  ren digitext-api.exe digitext-api.exe.old
  move C:\DigiText-ChuyenMay\digitext-api.exe digitext-api.exe
  powershell -ExecutionPolicy Bypass -File C:\DigiText\dev\restart-api.ps1
  ```

  Chép **không** được vào `C:\DigiText\updates` cho tiện: thư mục đó phục vụ qua HTTP công khai
  (`https://api.digitext.vn/updates/…`). Dùng `C:\DigiText-ChuyenMay` làm chỗ tạm.

- 11/11/2026 (tiếp) - **Tệp mẫu MLHS + hướng dẫn điền, sinh ở máy chủ** (mục 10.9
  `docs/WEBHOOK-NCC.md`). Cửa mới `POST /ncc/tu-kiem/muc-luc/mau` trả `.xlsx` base64 + bảng cột +
  hướng dẫn + danh mục THBQ của dự án + danh sách phông. Không migration.

  * **Mẫu sinh từ chính bảng nhận cột của bộ đọc.** `digitext-mucluc` có thêm `MO_TA_COT` — bản
    mặt người của `NHAN_COT` (nhãn, các cách viết, bắt buộc hay không, kiểu, giải thích) — và
    `mau.rs` dựng nội dung 3 sheet từ nó. Test `moi_cach_viet_deu_vao_dung_cot` bắt từng cách
    viết trong bảng phải được `tim_tieu_de_cot` xếp đúng chỉ số ⇒ sửa bí danh mà quên mẫu là
    test đỏ chứ không phải nhà thầu phát hiện hộ.
  * **Không thêm phụ thuộc ghi Excel.** `digitext-api/src/xlsx.rs` đóng gói `.xlsx` bằng `zip`
    đã có: chuỗi inline (`t="inlineStr"`), `styles.xml` đúng hai kiểu ô (thường / đậm), bỏ
    `sharedStrings` và `docProps`. Test đóng gói xong **đọc lại bằng chính bộ đọc MLHS**.
  * **Hai lỗi thật trong mẫu do test bắt, không phải do đọc lại code:**
    (1) mẫu ban đầu xếp hồ sơ vĩnh viễn và có thời hạn **chung một hộp** → hệ thống cảnh báo
    "tách thành hai hộp cùng số"; mẫu không được dạy cái mà bộ kiểm sẽ kêu.
    (2) câu dặn "Giữ nguyên dòng \"Phông:\" …" đặt phía trên dòng tiêu đề **bị nhận làm tên
    phông** — `re_phong_header` lấy lần khớp CUỐI nên câu dặn đè mất tên phông thật.
  * Cửa `tu-kiem/muc-luc` trả thêm `phong` (tên phông trong tệp, phông khớp, số trùng của đúng
    phông đó, số phông trong dự án) và `thbqDanhMuc`. **Nhà thầu không tạo và không chọn phông** —
    phông do cán bộ lưu trữ mở, tên trong tệp chỉ để đối chiếu, đúng như desktop đang làm.
  * `digitext_db::chinh_ly::ncc::phong_cua_du_an` — chỉ trả `ten`, không trả `id`.

- 11/11/2026 (tiếp) - **Migration 0029: `projects.chu_dau_tu` + `project_contracts.contract_meters`**
  (thư DO365 ngày 11/11). **PHẢI ÁP TRÊN OCR PC TRƯỚC KHI ĐẨY exe MỚI.**

  ```powershell
  :: trên OCR PC, console/RDP (script cần biến môi trường từ .env)
  powershell -ExecutionPolicy Bypass -File C:\DigiText\dev\api-lenh.ps1 migrate --chi-xem
  powershell -ExecutionPolicy Bypass -File C:\DigiText\dev\api-lenh.ps1 migrate
  ```

  * `chu_dau_tu`: bên gửi trước đó phải nhồi tên cơ quan vào `tenDuAn` vì không có chỗ khác, làm
    danh sách dự án toàn dòng na ná. Thêm ô `chuDauTu` vào webhook; `tenDuAn` quay về đúng nghĩa.
  * **Nội dung công việc thì KHÔNG cần ô mới** — `loaiDuAn` đã có `chinh_ly_so_hoa` từ 0028.
    Đã ghi rõ trong `WEBHOOK-DU-AN.md` kèm bảng 5 giá trị, và `POST /hooks/kiem-tra` nay trả
    `khoiLuongKhoa` / `khoiLuongSoLe` / `coChuDauTu` để bên gửi dò được bằng máy.
  * `contract_meters numeric(12,2)`: mét giá có số lẻ. Đọc/ghi qua `::float8` vì sqlx của dự án
    không bật feature decimal — không thêm phụ thuộc chỉ để đựng một con số.
  * **Bẫy đã xử:** desktop lưu hợp đồng bằng cách gửi TRỌN `ContractDto`, nên thêm cột ở máy chủ
    mà quên thêm ô ở desktop thì mỗi lần cán bộ sửa hợp đồng sẽ **xoá mất số mét webhook vừa
    ghi**. Đã thêm ô "Số mét hợp đồng" vào `ProjectProgressView` kèm hàm `soLe` riêng (hàm `so`
    cũ làm tròn xuống, 150,5 m thành 150 m).
  * `cargo test --workspace`: **185 ca đạt, 0 hỏng**.

- 11/11/2026 10:12-10:45 - **SỰ CỐ: webhook `du_an.*` trả 500 cho mọi dự án ĐÃ CÓ hợp đồng.**
  Nguyên nhân là lỗi của bản 0029 ở trên, không phải lỗi gói tin bên gửi.

  **Chuyện gì xảy ra.** `webhooks::lay_hop_dong` liệt kê cột tường minh rồi giải mã vào
  `progress::Contract`. Thêm trường `contract_meters` vào struct mà quên thêm cột vào câu
  `SELECT` ⇒ sqlx `FromRow` báo `no column found for name: contract_meters`.

  **Vì sao trốn được mọi lưới.** Lỗi chỉ nổ lúc **giải mã một dòng có thật**:
  * `cargo test --workspace` xanh — không có test chạm CSDL.
  * `cargo build` xanh — không có macro `query!`, SQL chỉ nổ lúc chạy.
  * Chạy thử tay lần đầu cũng xanh: dự án vừa tạo **chưa có** dòng `project_contracts` nên
    `fetch_optional` trả `None` và **không giải mã gì**. Phải có dự án đã gửi khối lượng
    trước đó mới nổ.
  * Và nó nổ với **bất kỳ** gói tin chạm hợp đồng — chỉ cần một trong `khoiLuong`,
    `ngayKyHopDong`, `ngayKetThucHopDong`, `canBoPhuTrach`, `ghiChu` ⇒ nhìn rời rạc tưởng
    lỗi do hai ô mới.

  **Lần theo thế nào.** Scheduled Task `DigiText-API` **không hứng stdout** (`api.out.log`,
  `api.err.log` đều 0 byte; `api.log` là của supervisor) nên không đọc được lỗi trên máy chủ.
  Phải dựng lại ở máy phát triển: CSDL mới `thu0029` trong `digitext-dev-postgres-1`, áp cả
  29 migration, chạy `digitext-api serve`, phát lại đúng gói tin của bên gửi. Lần một **không
  tái hiện được** vì thiếu đúng điều kiện trên — chỉ khi cho `du_an.tao` kèm `khoiLuong` để
  sinh dòng hợp đồng trước thì mới ra 500 và log in ra tên cột thiếu.
  ⇒ **Việc còn nợ: cho Scheduled Task ghi stdout ra tệp.** Không có log thì mỗi sự cố đều
  phải dựng lại môi trường ở máy khác.

  **Sửa.** Danh sách cột gom về một hằng `progress::CONTRACT_COLS`, dùng chung. Kèm test
  `contract_cols_phu_het_truong`: nó destructure `Contract` **đủ trường, không có `..`**, nên
  thêm trường mà quên cột thì test **không biên dịch được** — bắt ngay lúc `cargo build`, không
  đợi tới lúc gặp dòng thật.

  **Bài học xếp thứ tự áp.** 0029 áp lúc 03:30:43Z, exe mới chạy từ 02:31Z — tức là có **59
  phút** máy chủ chạy mã mới trên lược đồ cũ, mọi `du_an.*` và cả 5 cửa NCC đều 500. Thứ tự
  đúng vẫn là **migration trước, exe sau**; lần này làm ngược.

- 11/11/2026 11:06 - **Sửa sự cố + trả lại `khoiLuong` trong phản hồi webhook** (đề nghị DO365).
  exe `25.850.880` byte. Không migration.

  * `webhooks::lay_hop_dong` dùng hằng `progress::CONTRACT_COLS`; test `contract_cols_phu_het_truong`
    destructure `Contract` đủ trường không có `..` ⇒ thêm trường mà quên cột là **không biên dịch
    được**.
  * Phản hồi `du_an.*` thêm `khoiLuong` — khối lượng **đã lưu sau khi ghi**, đúng tên khoá của gói
    tin gửi lên, khoá rỗng thì vắng mặt chứ không `null`. Không tốn truy vấn: dùng chính bản ghi
    vừa viết. Chỉ có mặt khi gói tin chạm tới hợp đồng.
  * Nghiệm thu trọn vòng trên CSDL mới `thu0029b` (áp đủ 29 migration), 4 ca:
    `tao` có khối lượng → 201 · `capnhat` đúng gói tin 10:32 → 200 kèm `{met:175, trang:465000}` ·
    `capnhat` **chỉ gửi mét** → phản hồi vẫn trả 465.000 trang giữ lại từ lần trước (chỗ hay lệch
    nhất) · `du_an.dong` → **không** có khoá `khoiLuong`.
  * `cargo test --workspace`: 186 ca đạt.

- 13/11/2026 - **`POST /ncc/tu-kiem/muc-luc/mau` trả 400 "thiếu maNgoai" dù gói tin có maNgoai.**
  `struct GoiMau` thiếu `#[serde(rename_all = "camelCase")]` — trường chỉ nhận khoá `ma_ngoai`,
  không nhận `maNgoai` như tài liệu §10.9 công bố. Bốn struct gói tin kia của `ncc.rs` đều có.
  Bên DO365 tự đọc nguồn tìm ra và chỉ đúng dòng.

  **Đây là lần THỨ HAI cùng một lỗi** (lần đầu: `GoiKiem`, cửa `tu-kiem/da-nop`). Đặc điểm làm
  nó sống dai: biên dịch sạch, mọi test khác xanh, và ở đầu kia nó hiện ra là *"400 thiếu
  maNgoai"* trên một gói tin **có** maNgoai ⇒ bên gọi đi tìm lỗi trong mã của họ trước.

  **Đã đóng lớp lỗi**, không chỉ sửa một chỗ: test `moi_cua_deu_nhan_ma_ngoai_camel_case` giải
  mã `{"maNgoai":"…"}` vào **cả năm** struct gói tin, kèm
  `khoa_nhieu_chu_khac_cung_nhan_camel_case` cho các khoá nhiều chữ khác. Đã kiểm ngược: bỏ
  thuộc tính ra thì test đỏ và chỉ đích danh cửa sai. Thêm cửa mới thì thêm một dòng vào test đó.

- 13/11/2026 (tiếp) - **«Cảnh báo 59» mà bảng rỗng** — thẻ đếm và danh sách dùng hai phép lọc
  khác nhau. Lỗi của cửa `tu-kiem/muc-luc`, không phải của giao diện bên DO365.

  `so_canh_bao` đếm dòng có `canh_bao` **hoặc** có **kết luận thời gian** là cảnh báo/lỗi. Danh
  sách `dong[]` thì chỉ lấy dòng có `loi`/`canh_bao`. Mà lời giải thích của loại thứ hai nằm
  trong `tg.ghi_chu` ("năm 1700 ngoài dải 1800–2100"…), không nằm trong `canh_bao` ⇒ 59 cảnh báo
  đếm được nhưng không có dòng nào để bày, người dùng không có cách nào biết chúng là gì.

  * Tách phép liệt kê thành hàm thuần `dong_co_van_de(&pt)`, lọc **đúng** tập dòng mà hai bộ đếm
    đếm. `canhBao` trả ra = `canh_bao` **cộng** `tg.ghi_chu` khi kết luận thời gian không sạch,
    nên hai con số không thể lệch nhau nữa. Thêm `thoiGian` `{batDau, ketThuc, ketLuan, ghiChu}`
    vào mỗi dòng cho bên nào muốn bày riêng.
  * Test `so_dem_va_danh_sach_khong_duoc_lech_nhau` dựng lưới ô thật, chạy qua bộ phân tích, rồi
    so `soCanhBao` với số dòng có `canhBao`. Bộ dữ liệu thử có **khẳng định rằng nó chứa đủ ba
    loại** (lỗi chặn · cảnh báo thường · cảnh báo chỉ ở thời gian) — không có khẳng định đó thì
    test xanh mà chẳng kiểm gì. Đã kiểm ngược: quay về phép lọc cũ thì test đỏ với đúng câu
    *"thẻ báo 3 cảnh báo nhưng bảng chỉ có 2 dòng cảnh báo"*.
  * `calamine` thêm vào `[dev-dependencies]` của `digitext-api` chỉ để dựng lưới ô trong test.
  * Đặc tả: thêm mục 10.3b `docs/WEBHOOK-NCC.md`, nói rõ bất biến "số dòng có `canhBao` **bằng
    đúng** `soCanhBao`" để bên kia cứ bày thẳng, khỏi tự đếm lại.

- 13/11/2026 (tiếp) - **Cửa thứ năm: «Tự kiểm gói tin» — đảo quyết định cũ.** Mục 12
  `docs/WEBHOOK-NCC.md` trước ghi "cố ý KHÔNG làm cửa web" vì gói AIP cỡ GB. Lý do đó chỉ đúng
  nếu phải tải cả gói lên. Đo trên `test/AIP.zip`: **131 MB nội dung / 326 KB siêu dữ liệu —
  0,247%**. Cửa mới `POST /ncc/tu-kiem/goi` nhận **bản kê** (đường dẫn + cỡ mọi tệp, kèm nội dung
  riêng các tệp `.xml`/`.xsd`), không nhận gói. Gói 1 GB gửi lên chừng 2,5 MB. Không migration.

  * `digitext-pkgcheck` thêm chế độ rút gọn: `Loaded.thieu_noi_dung` + `Loaded.co_tep`,
    `tu_ban_ke()`, `check_loaded()`. Ba phép kiểm cần byte thật (PDF/A từng tệp, mã băm METS, mã
    băm PREMIS) bị bỏ qua **và đếm lại** vào `chua_kiem_noi_dung`.
  * **`verdict` thêm hai giá trị** `CHƯA KIỂM ĐỦ` / `CHƯA KIỂM ĐỦ (có cảnh báo)`. Còn phép kiểm
    chưa chạy thì tuyệt đối không được nói "ĐẠT" — người đọc sẽ hiểu là gói nộp được.
  * **Phép kiểm `SIZE` của METS vẫn chạy**, vì bản kê có cỡ tệp. Chỗ này là do **test đối chiếu
    bắt ra**: bản đầu băm/đo trên mảng rỗng nên báo 7 lỗi giả "SIZE=… khác thực tế 0". Không có
    test đó thì nhà thầu là người đầu tiên thấy.
  * Nghiệm thu: `rut_gon_khop_goi_that` (`#[ignore]`, cần `DIGITEXT_PKG_SAMPLE`) dựng bản rút gọn
    từ **chính gói thật** rồi so. Kết quả trên `test/AIP.zip`: đầy đủ *KHÔNG ĐẠT · 1 lỗi · 14
    cảnh báo*; rút gọn *CHƯA KIỂM ĐỦ (có cảnh báo) · 0 lỗi · 11 cảnh báo · bỏ qua 21*. Lỗi cấu
    trúc **giống hệt**; khác đúng phần cần nội dung.
  * ⚠️ **Gói qua cửa này vẫn có thể bị từ chối lúc giao** — đúng gói mẫu trên là ví dụ: bản đầy
    đủ KHÔNG ĐẠT vì 7/7 tệp không phải PDF/A, bản rút gọn không thấy. Đã viết thẳng điều đó vào
    cả `ghiChu` của phản hồi lẫn mục 12.2 và bản dặn bên DO365.
  * Cổng lấy bản kê bằng cách đọc **thư mục trung tâm** của zip (~64 KB cuối tệp) — dưới 1 MB đọc
    từ đĩa dù gói 10 GB. Gói lồng gói thì lớp trong lưu **không nén** nên đọc được tại chỗ.
  * `cargo test --workspace`: **198 ca đạt, 0 hỏng**.
