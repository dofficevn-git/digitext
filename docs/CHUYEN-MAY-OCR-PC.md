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
| Đường vào API | **Trực tiếp, không tunnel** (anh chốt lại 30/10): Caddy + Let's Encrypt (DNS-01 Cloudflare) ở **cổng 443**, một tên duy nhất **`api.digitext.vn`** (đã đóng cứng trong updater desktop) → bản ghi DNS chuyển DNS-only trỏ IP tĩnh; router **NAT thêm 443 → 192.168.100.10:443** (5005 đã NAT sẵn không dùng tới; 5001 n8n, 5002 web KIE cũ, 80 IIS). Máy trạm giữ nguyên `https://api.digitext.vn`. **Bỏ cloudflared hoàn toàn** — tắt service trên .127 ở GĐ2, xoá tunnel trên Cloudflare |
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
   `VLLM_GPU_FRACTION=0.55`, `GPU_DEVICE_ID`, `VLLM_MEM_LIMIT=20g`, `VLLM_START_PERIOD=1200s`. Không có FlashAttention
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
  cloudflared hẳn**, đi thẳng `api.digitext.vn` 443 (NAT 443). Chờ: cắm F: (§2.4), anh bật OpenSSH trên OCR PC (§3.1), NAT 443.
