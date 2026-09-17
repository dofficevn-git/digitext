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

| Cổng | Dịch vụ | Trạng thái |
|---|---|---|
| 5002 | OCR toàn văn (`serve_ocr.py`) | **chạy**, `{"ready":true}` |
| 8003 | vLLM Qwen2.5-VL — DigiText bóc 16 trường | **chạy**, giữ 9,3/16 G VRAM |
| 5001 | Web + KIE (`app.py`) | **chạy**, `admin`/`admin123` |
| 1433 | SQL Server | **chạy**, 8 bảng đã dựng |
| 3000 | vLLM `vnkie` (Vintern) | **DỪNG** — thiếu trọng số, xem mục 3 |

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

- Máy này **không cài Rust**. Các script `dev/run-*.ps1` tự chạy `target\debug\*.exe`
  (bản build ngày 2026-09-08, mới hơn commit Rust cuối cùng ngày 2026-09-05). Sửa
  code Rust thì phải build lại ở máy có cargo.
- `engine_versions` đã có dòng `vietocr-vgg-transformer-http` / `recognition`.
  Không có dòng này thì worker thoát ngay.
- Khoá `X-API-Key` đã được kiểm giữa hai máy: không kèm khoá trả 401, kèm khoá trả 200.
- Máy này còn chạy dự án khác (`plx-*`, `kpi-*`, `ntm_qn_*`, chiếm 1433/1434/3030/5433/6379/8081/8088).
  Kiểm cổng trước khi thêm dịch vụ.
- **Chưa có tài khoản:** `bootstrap-superadmin` hỏi mật khẩu qua TTY nên phải tự gõ:
  `.\dev\run-api.ps1 bootstrap-superadmin --username <tên>`
- Chưa bật: bóc trường v2 (cần vLLM `:8003` trên Alienware, đang tắt), ký số
  (`DIGITEXT_SIGN_TRUST_ROOTS`, `DIGITEXT_TSA_TRUST_ROOTS` chưa đặt).

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
