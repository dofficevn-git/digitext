# Bàn giao — tiếp tục công việc trên máy khác

> **Cách dùng:** mở Claude Code ở gốc repo và bảo *"đọc docs/HANDOFF.md rồi tiếp tục"*.
> File này nêu **trạng thái đang dở**, không nêu kiến trúc — kiến trúc ở
> [PROJECT-MAP.md](PROJECT-MAP.md), quy ước bắt buộc ở [../CLAUDE.md](../CLAUDE.md).

Chốt tại commit `bbf551e`, cây làm việc sạch.

---

## 0. Mang gì sang máy mới

`git clone` **không đủ**. Những thứ dưới đây bị `.gitignore` loại nhưng thiếu là
không chạy được. Chép tay trước khi bắt đầu:

| Đường dẫn trên máy cũ | Cỡ | Thiếu thì sao |
|---|---|---|
| `D:\doc_extract_pla\models\plakie\` | 1,8 G | **Việc đang dở nằm ở đây.** Vintern-1B-v3_5, chưa kịp chuyển sang Alienware |
| `ocr/models/vietocr/` | 145 M | Dịch vụ OCR `:5002` không nạp được bộ nhận dạng |
| `ocr/best.pt` | 6,0 M | YOLO cắt header — hỏng **âm thầm**, xem mục 4 |
| `jwt-ed25519.pem` | 1 K | API không ký được token |
| `crates/digitext-api/.env` | | Cấu hình DigiText |
| `crates/digitext-ocr/.env` | | |
| `infra/compose/.env` | | |
| `infra/compose/ocr/.env` | | Chứa `OCR_API_KEY` |
| `infra/compose/docker-compose.override.yml` | | Đổi cổng Postgres 5432→5433 vì máy cũ có PostgreSQL 18 chiếm 5432. **Máy `.127` đã tắt** (đổi tên thành `.bak-may-cu`): ở máy này `5433` là của `ntm_qn_db`, một dự án khác — trỏ vào đó là ghi nhầm CSDL |
| `digitext-rescue.tar.gz` | 204 M | Bản cứu từ server Ubuntu. Không cần để chạy, nhưng là **bản sao duy nhất** — đừng bỏ lại |

`migrations/0001_baseline.sql` và `0002_documents_pdfa.sql` **có** trong git.

Ảnh huấn luyện (`images/`) **chỉ nằm trên Alienware**, không có trên máy Windows.

---

## 1. Hai máy

| | |
|---|---|
| **Máy chủ DigiText** (Windows) | `192.168.0.127`. Chạy Postgres/MinIO/Meilisearch (docker), `digitext-api` và worker `digitext-ocr`. IP máy cũ là `192.168.0.212` |
| **Alienware x16 R1** | `192.168.0.205`, user `do`, Ubuntu + GPU 16 G. Chạy toàn bộ dịch vụ |

Repo trên Alienware: `/home/do/Downloads/digitext-gpu-bundle/`

Phân vai hiện tại: **Alienware chỉ chạy dịch vụ nhận dạng** (`serve_ocr.py` `:5002`).
Hàng đợi job, CSDL, kho ảnh và worker đều ở máy Windows. Worker gọi sang Alienware
theo `DIGITEXT_OCR_HTTP_ENDPOINT` trong `crates/digitext-ocr/.env`. Sau này gộp về
một máy thì chỉ cần đổi biến đó thành `http://127.0.0.1:5002`.

Chuyển file giữa hai máy theo lối đã dùng suốt (Windows không chạy sshd):

```powershell
# Windows
python -m http.server 8899 --bind 0.0.0.0 --directory "$env:TEMP\patch-share"
```
```bash
# Alienware
curl -O http://<ip-windows>:8899/<file>
```

---

## 2. Trạng thái dịch vụ trên Alienware

| Cổng | Dịch vụ | Trạng thái (cập nhật chiều 17/9) |
|---|---|---|
| 5002 | OCR toàn văn (`serve_ocr.py`) | **chạy trên GPU** (`docker-compose.yml` + `docker-compose.gpu.yml`), health trả `gpu:true`, ~550 MB VRAM lúc rảnh. Ảnh CPU cũ giữ ở tag `digitext-ocr-fulltext-ocr:cpu-20260917` |
| 8003 | vLLM Qwen2.5-VL — DigiText bóc 16 trường | **chạy**, dựng lại từ `bundle/infra/compose/vllm` (`.env`: `VLLM_BIND=0.0.0.0`, fraction 0.55), ~9 GB VRAM. Trước đó chỉ bind loopback nên từ Windows tưởng chết |
| 5001 | Web + KIE (`app.py`) | chạy, `admin`/`admin123` |
| 1433 | SQL Server | chạy (loopback) |
| 3000 | vLLM `vnkie` (Vintern) | **đã nạp được** sau khi dời 14 file vào `ocr/models/plakie/` (trước đó giải nén nhầm ra `ocr/models/`); **đang stop** để nhường VRAM. Bật khi A/B: `docker compose --env-file .env up -d vllm-vnkie` |
| — | `digitext@ocr.service` (systemd) | binary cũ 20/8, crash-loop 257+ lần vì trỏ Postgres cũ. **Tắt đi:** `sudo systemctl disable --now digitext@ocr` |

SSH: `ssh do@192.168.0.205` bằng khoá của máy Windows (`~/.ssh/id_ed25519`, đã
add vào `authorized_keys` 17/9). Bundle trên đó **không phải git** — deploy bằng
`scp` từng file vào `/home/do/Downloads/digitext-gpu-bundle/`.

`:5002` và `:5001` cố ý tách rời: SQL Server chết thì OCR của DigiText vẫn chạy.
Đừng gộp. `app.py:184` cũng đăng ký blueprint OCR nên `:5001` có
`/api/ocr/tile`, nhưng **không được trỏ DigiText vào đó** — image web không cài
PaddleOCR nên trả 501, và trỏ vào đó là vứt bỏ đúng tính chất tách rời trên.

Mật khẩu SQL Server nằm ở `infra/compose/ocr-web/.env` **trên Alienware** — cố ý
không chép vào repo. Nó bị SQL Server chốt vào volume `sqlserver-data` lúc khởi
tạo; sửa file `.env` về sau không đổi được, phải `ALTER LOGIN`.

Lệnh vận hành:
```bash
cd /home/do/Downloads/digitext-gpu-bundle/infra/compose/ocr-web
docker compose ps
docker compose --env-file .env up -d <service>
docker compose logs -f <service>
```

---

## 2b. Máy chủ DigiText `192.168.0.127` — đã dựng 2026-09-17

| Cổng | Dịch vụ | Ghi chú |
|---|---|---|
| 5432 | Postgres 16 (`digitext-dev-postgres-1`) | CSDL mới, đã chạy `0001` + `0002`, 34 bảng |
| 9000/9001 | MinIO | bucket `pages` đã tạo tay — **code không tự tạo bucket** |
| 7700 | Meilisearch | |
| 8080 | `digitext-api` | chưa có tài khoản nào, xem dưới |
| 9095 | worker `digitext-ocr` | `/readyz` gọi thật sang `:5002` của Alienware (các mục kiểm mang tên "triton" là tên cũ) |

- Máy này **đã cài Rust** (rustup stable 1.98 + VS Build Tools 2022, chiều 17/9).
  `cargo` chưa nằm trong PATH của shell mới → `dev/run-*.ps1` tự chạy
  `target\release\*.exe` (build 17/9 15:03, có đủ Phase 1–2). Smart App Control
  đã **tắt** — bật lại thì cargo không chạy được (chặn file build-script).
- App desktop chạy bằng `cd desktop; npm run tauri dev` (lần đầu build ~5 phút).
- `engine_versions`: worker **tự đăng ký** dòng recognition theo
  `DIGITEXT_RECOGNITION_ENGINE_NAME` (mặc định `vietocr-vgg-transformer-http`).
  Bucket MinIO cũng tự tạo khi API/worker khởi động.
- Khoá `X-API-Key` đã được kiểm giữa hai máy: không kèm khoá trả 401, kèm khoá trả 200.
- Máy này còn chạy dự án khác (`plx-*`, `kpi-*`, `ntm_qn_*`, chiếm 1433/1434/3030/5433/6379/8081/8088).
  Kiểm cổng trước khi thêm dịch vụ.
- Tài khoản `admin` đã tạo (17/9). Có 1 dự án `TEST`, 110 tài liệu đã OCR + PDF 2 lớp
  (`to_sign`).
- Chưa bật: bóc trường v2 (cần vLLM `:8003` trên Alienware, đang tắt), ký số
  (`DIGITEXT_SIGN_TRUST_ROOTS`, `DIGITEXT_TSA_TRUST_ROOTS` chưa đặt).

## 2c. Nâng cấp OCR — làm tới đâu (kế hoạch 10 phase, 17/9)

Kế hoạch đầy đủ nằm ở plan đã duyệt; runbook máy GPU mới:
[DEPLOY-GPU-NODE.md](DEPLOY-GPU-NODE.md).

| Phase | Việc | Trạng thái |
|---|---|---|
| 0 | Kiểm dữ liệu sau sự cố 35 trang | xong, 0 tài liệu sai |
| 1 | Worker: backoff trong DB (`ocr_jobs.not_before`, migration 0003), health gate, đếm đúng trang lỗi, log, metrics, tự tạo bucket/engine, 10 test | **xong, đã kiểm bằng proxy giả lập mất kết nối** |
| 2 | `POST /documents/ocr-retry-failed`, `progress.pagesOcrFailed`, nút "Thử lại N trang OCR lỗi" | xong (nút chỉ hiện khi có trang lỗi) |
| 3.1 | `run_bench.py`/`loadtest.py --backend http` | xong |
| 3.2 | Bộ đồng thuận VLM + baseline | **xong**: 520 dòng `val.txt` (nhãn Qwen) → `ml/bench/golden-set/vlm_pseudo/` (trên Alienware + bản sao local). `make_golden_from_pairs.py` mới. GPU CER **0,2281**, CPU **0,2283**, gate PASS. ECE 0,54 → confidence VietOCR gần như không hiệu chuẩn |
| 3.3 | Gold KIE từ nút Duyệt: `ml/finetune/kie/gold_from_db.py export-gold` | công cụ xong; **chưa có trường nào verified** — cần bạn duyệt ~40 tài liệu trong app |
| 4 | OCR GPU | **xong, đang chạy**. Hai bẫy đã trả giá: index wheel paddle cu120 và symlink `libcudnn.so` (§4). 12 trang: 24 s (CPU 240 s); ~1,2 s/trang 44 dòng |
| 5 | VRAM, Qwen, Vintern | **xong**: Qwen bind LAN; Vintern nạp được (stop chờ A/B). Bảng VRAM: OCR 0,5–3 GB + Qwen 9 GB; Vintern 4 GB chỉ chạy khi Qwen stop |
| 6.1 | `DIGITEXT_VLM_PROFILE` | xong, có test |
| 6.2 | A/B Qwen vs Vintern | **v2 với Qwen đã chạy trên cả 110 tài liệu, 0 lỗi**: 8 trường chính phủ 98–100%, `thuTu/toSo/trangSo` thấp (không có trên trang). Thăm dò 1 ảnh với Vintern + `guided_json`: JSON hợp lệ nhưng lẫn trường — cần A/B đúng cách khi có gold (3.3) |
| 7 | Fine-tune VietOCR | dữ liệu `data/rec_vi` xong, CER thật **0,278 > 0,25 → đáng làm**; script xong; **chưa huấn luyện** (xem 200 dòng `train.txt` trước) |
| 8 | Runbook máy mới, metrics | xong phần tài liệu; cảnh báo SigNoz chưa đặt |
| 9 | Chạy lại lô với bước phá | **xong** (16:16 17/9): OCR lại 227 trang trên GPU, 2 lane — **~254 s** chạy thực (sáng cùng ngày trên CPU: ~75 phút). Dừng container OCR 109 s ở ~30%: `attempts` tối đa 1, 0 `failed_transient`, gate tự mở lại, tài liệu giữ `to_sign`. Conf trung bình 0,891 |

Worker Windows hiện: `DIGITEXT_WORKER_CONCURRENCY=2`, `DIGITEXT_FIELDS_ENGINE=v2`
(Qwen, profile `qwen`, 2 trang đầu). Engine v2 trong CSDL: `qwen2.5-vl-7b-instruct-awq-kie`.

Điều đã đo được khi kiểm Phase 1 (proxy `127.0.0.1:5999` → Alienware, tắt/bật):
gate đóng sau lỗi đầu tiên, các lane khác không claim, `attempts` đứng yên; mở
lại → `dịch vụ OCR đã trở lại` trong ≤ 30 s và job chạy tiếp. Khi dịch vụ
"nửa sống" (health 200 nhưng tile lỗi) thì backoff giãn 2→60 s: 8 lượt trong
6 phút thay vì 5 lượt trong vài giây.

---

## 3. Việc đang dở — làm tiếp từ đây

### 3.1 Đưa Vintern lên Alienware (dừng giữa chừng ở đúng đây)

Model đã kiểm đủ trên máy Windows: `InternVLChatModel`, 14 file, `config.json` ở
gốc thư mục.

```powershell
# Windows — phục vụ thẳng, khỏi chép 1,8 G qua lại
python -m http.server 8899 --bind 0.0.0.0 --directory "<duong-dan>\doc_extract_pla\models"
```
```bash
# Alienware
cd /home/do/Downloads/digitext-gpu-bundle/ocr/models
rmdir plakie 2>/dev/null
wget -r -np -nH --cut-dirs=1 -R "index.html*" http://<ip-windows>:8899/plakie/
ls plakie/config.json && du -sh plakie/     # ~1,8 G, 14 file

cd ../../infra/compose/ocr-web
sed -i 's/^VLLM_GPU_FRACTION=.*/VLLM_GPU_FRACTION=0.25/' .env
docker compose --env-file .env up -d vllm-vnkie
docker compose logs -f vllm-vnkie           # chờ "Application startup complete"
nvidia-smi --query-gpu=memory.used,memory.total --format=csv
curl -s localhost:3000/v1/models            # phải có "id": "vnkie"
```

**Hai bẫy, theo thứ tự sẽ gặp:**

1. Giải nén lồng thành `plakie/plakie/config.json` → vLLM báo
   `Invalid repository ID or local directory specified`. Đã gặp đúng lỗi này khi
   thư mục còn rỗng.
2. `config.json` khai `auto_map` dạng `5CD-AI/Vintern-1B-v3_5--configuration_…`.
   Tiền tố `repo--` khiến transformers đi tải mã từ HuggingFace **dù file `.py`
   đã nằm ngay cạnh**. Log báo không tới được hub thì:
   ```bash
   sed -i 's|5CD-AI/Vintern-1B-v3_5--||g' ocr/models/plakie/config.json
   ```
   Cứ thử nguyên trạng trước — có mạng thì không cần sửa.

**Vì sao `0.25` chứ không phải `0.90` của bản gốc:** `--gpu-memory-utilization`
là tỉ lệ trên **toàn bộ card**, không phải phần còn trống. Card 16 G đã có Qwen
giữ 9,3 G. `0.90` = xin 14,4 G ⇒ chắc chắn hỏng. `0.25` ≈ 4,1 G, đủ cho 1,8 G
trọng số cộng KV cache. Kiểm `nvidia-smi` sau khi nạp xong, đừng đo lúc container
vừa `Started` — lúc đó vLLM chưa cấp phát gì.

### 3.2 Fine-tune VietOCR

Đang chạy nền trên Alienware:
```bash
cd ~/Downloads/digitext-gpu-bundle/ml/finetune/vietocr
tail -f build.log
ls data/rec_vi/*.png | wc -l        # crop ghi dần ra đĩa, đếm được tiến độ
```
370 trang, ~2,5 giờ. Nếu log trống mà tiến trình còn sống thì là **đệm stdout** —
chạy lại bằng `python3 -u … > build.log 2>&1 &`.

Xong thì đọc phần thống kê và nhìn **`TÍNH CẢ dòng bị loại vì bất đồng`**, không
nhìn dòng `chỉ dòng GIỮ LẠI`. Bộ lọc loại đúng những dòng VietOCR đọc sai nặng
nhất, nên con số "giữ lại" là tự lừa mình — lượt thử 5 trang cho 0,133 trong khi
thật ra ≈0,22. Trên ~0,25 thì fine-tune đáng làm, dưới ~0,15 thì cải thiện ít.

**Trước khi chạy `train_vietocr.py`, hãy tự đọc `data/rec_vi/train.txt`** — nhãn
do VLM sinh, chưa ai kiểm bằng mắt. Nhãn rác thì huấn luyện xong mới biết.

---

## 4. Bẫy đã trả giá

**Ba thứ hỏng âm thầm — web vẫn trả 200:**
- `ocr/best.pt` thiếu → `main.py:90` nuốt lỗi, đặt `yolo_model = None`, mọi tài
  liệu rơi vào nhánh fallback, nhận dạng loại văn bản sai mà không báo gì.
- vLLM `vnkie` chết → `DocumentKIE.__init__` chỉ đọc biến môi trường, **không hề
  kết nối**. Chỉ đổ vỡ lúc xử lý tài liệu đầu tiên.
- Cả hai đều không bị healthcheck bắt. Kiểm riêng:
  ```bash
  docker compose logs web | grep -E "YOLO|KIE|Database initialized|Waitress"
  ```
  Phải thấy `Đã load YOLO model: best.pt`.

**Đệm stdout.** Python đệm theo khối khi đầu ra không phải terminal. Đã cắn hai
lần: container (vá bằng `PYTHONUNBUFFERED=1` trong Dockerfile) và `nohup` (vá
bằng `python3 -u`). Log trống **không** có nghĩa là không có gì chạy.

**Import cấp module ≠ có được dùng.** Đã loại `google-genai` vì không dòng nào gọi
`genai.*` — nhưng `main.py:9` import ở cấp module nên `app.py` sập vòng lặp. Muốn
tỉa phụ thuộc thì quét đồ thị import cấp module bằng `ast`, đừng grep chỗ gọi.

**Mật khẩu SQL Server.** Đừng sinh bằng `openssl rand -hex` — hex chỉ có chữ
thường và số, SQL Server đòi đủ 3 trong 4 nhóm, container sẽ chết với log mơ hồ.
Dùng `openssl rand -base64 24 | tr -d '=\n' | tr '+/' 'Xy'`.

**Cargo trên Windows.** Chạy từ PowerShell, không từ Git Bash — `link` của Git
Bash che `link.exe` của MSVC và làm hỏng cả `target/`.

**Bộ nhớ.** Hai lần sập cả máy Alienware đều do `predict_batch` nhận trọn số dòng
của một trang. Đã vá bằng `OCR_REC_BATCH=8` + `mem_limit`. Đừng bỏ trần bộ nhớ.

**OCR chạy CPU trên máy có GPU.** Image `:5002` là `python:3.10-slim` + paddle
CPU, và `ocr_fulltext.py` không truyền `use_gpu` cho PaddleOCR — nên 20 s/trang
suốt tháng 9 dù card 16 GB rảnh. Đã sửa (Phase 4) nhưng **phải up bằng overlay
`docker-compose.gpu.yml`**; file compose gốc vẫn là CPU. Health giờ trả `gpu:true/false`
— nhìn vào đó, đừng tin `OCR_USE_GPU` trong `.env`.

**Paddle GPU: hai bẫy lúc build/chạy.** (1) `paddlepaddle-gpu==2.6.2.post120`
chỉ có ở index `packages/stable/cu120/`, không có ở `stable.html`. (2) Model nạp
xong, health OK, nhưng **mọi request 500** "Cannot load cudnn shared library":
Paddle dlopen `libcudnn.so` không phiên bản, image runtime chỉ có `.so.8`.
Dockerfile giờ tự tạo symlink; nếu thấy lại lỗi này thì kiểm
`docker exec … ls -la /usr/lib/x86_64-linux-gnu/libcudnn.so`.

**Cổng bind loopback ≠ dịch vụ chết.** Qwen `:8003` và SQL Server `:1433` bind
`127.0.0.1` trên Alienware; quét cổng từ máy khác thấy "đóng" là bình thường.
Kiểm bằng `ssh … curl localhost:8003/v1/models`.

**"OCR toàn bộ" không chạy lại trang lỗi.** Nó chỉ lấy tài liệu chưa có job.
Trang `failed_transient`/`page_failed` phải qua nút "Thử lại N trang OCR lỗi"
(hoặc `POST /documents/ocr-retry-failed`). Tài liệu đã `to_sign` bị bỏ qua — muốn
OCR lại thì phải xử lý chữ ký trước.

**Bộ đồng thuận VLM không phải golden set.** Nhãn do Qwen sinh; `gate.py --tol 0.01`
chỉ để bắt hồi quy. Có người gõ thì thay bằng `golden.py build` hai người.

---

## 5. Chưa làm, đã nêu nhưng chưa được yêu cầu

- Viết lại lịch sử git để xoá `dangdat1111` khỏi các commit cũ. Mã hiện tại đã
  sạch, chỉ còn lịch sử.
- Xoá hoặc khoá repo công khai `dangdat1111/DigiText`.
- `external_routes.py:10` mặc định UNC `\\103.154.62.17\UploadFile` — đường dẫn
  Windows, trên Ubuntu phải mount CIFS rồi trỏ `EXTERNAL_FILE_SERVER_PATH` vào
  điểm mount. Chưa mount thì chỉ `/api/external/*` hỏng.
- `database.py:211` băm mật khẩu bằng SHA-256 trần, không salt. `:5001` đang mở
  ra LAN với `admin`/`admin123`. Đổi mật khẩu ngay; muốn chắc thì thay bằng
  bcrypt/argon2 — nhưng đó là sửa mã nguồn `doc_extract_pla`.
- **Không có test nào** cho mã sản phẩm ở cả hai dự án. Mọi kiểm chứng đều thủ
  công, và đó là tất cả những gì có.
