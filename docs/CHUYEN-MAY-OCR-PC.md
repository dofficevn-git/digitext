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

