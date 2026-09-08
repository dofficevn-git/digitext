# Giao diện web + KIE (doc_extract_pla) trên node GPU

Dựng lại ứng dụng gốc của `doc_extract_pla` — giao diện người dùng, quản lý lô,
gói việc, bóc 8 trường bằng VLM — **song song** với dịch vụ OCR toàn văn mà
DigiText gọi vào.

Hai thứ này **không thay thế nhau**. Chúng khác cổng, khác vòng đời, khác đối
tượng phục vụ, và cố ý không phụ thuộc nhau:

| | `:5002` OCR toàn văn | `:5001` Giao diện web |
|---|---|---|
| Nguồn | `serve_ocr.py` | `app.py` |
| Ai gọi | DigiText (máy gọi máy) | người dùng mở trình duyệt |
| Xác thực | `X-API-Key` | đăng nhập phiên |
| Cần SQL Server | **không** | **có** — không có là không khởi động |
| Cần GPU | không (chạy CPU) | có, qua vLLM |
| Compose | `infra/compose/ocr/` | thư mục này |

Tách như vậy là có chủ đích: SQL Server chết thì OCR của DigiText vẫn chạy.
Gộp chung sẽ mất tính chất đó.

## Bản đồ cổng trên node GPU

```
 5001  giao diện web            ← dựng bằng compose ở đây
 5002  OCR toàn văn             ← đang chạy
 3000  vLLM  vnkie   (Vintern)  ← dựng bằng compose ở đây
 8003  vLLM  Qwen2.5-VL         ← đang chạy, DigiText bóc 16 trường
 1433  SQL Server               ← dựng bằng compose ở đây
```

## Hai file phải tự mang sang

Không có trong repo (`.gitignore`), phải chép tay vào đúng vị trí **trước khi**
dựng:

| Đích | Cái gì | Hậu quả nếu thiếu |
|---|---|---|
| `ocr/best.pt` | YOLOv8n cắt vùng header (5,9 MB) | Web **vẫn lên**. `main.py:90` nuốt lỗi, đặt `yolo_model = None`, rồi mọi tài liệu rơi vào nhánh fallback — nhận dạng loại văn bản sai lệch mà không báo gì |
| `ocr/models/plakie/` | Vintern-1B đã fine-tune | vLLM không khởi động. Web vẫn lên (`DocumentKIE.__init__` không hề kết nối), chỉ đổ vỡ lúc xử lý tài liệu đầu tiên |

Cả hai đều hỏng **âm thầm**. Kiểm riêng từng cái sau khi dựng, đừng chờ web báo lỗi.

## Dựng

```bash
cp infra/compose/ocr-web/.env.example infra/compose/ocr-web/.env
# Đặt DB_PASSWORD và SECRET_KEY — cả hai đều bắt buộc, compose từ chối chạy nếu trống.
#   openssl rand -hex 32

cd infra/compose/ocr-web
docker compose --env-file .env up -d --build
```

Lần đầu mất khá lâu: SQL Server khởi tạo ~90 giây, và image web phải kéo driver
ODBC của Microsoft cùng torch.

Theo dõi:

```bash
docker compose logs -f web
```

## Kiểm sau khi dựng

```bash
# 1. SQL Server đã tạo CSDL chưa
docker compose logs sqlserver-init | tail -5

# 2. vLLM đã nạp model chưa — phải thấy "vnkie"
curl -s localhost:3000/v1/models | python3 -m json.tool

# 3. VRAM: Qwen + Vintern phải cùng nằm gọn trong 16GB
nvidia-smi --query-gpu=memory.used,memory.total --format=csv

# 4. Web
curl -s -o /dev/null -w '%{http_code}\n' localhost:5001/login    # 200

# 5. Dịch vụ OCR cũ KHÔNG được ảnh hưởng
curl -s localhost:5002/api/ocr/health                            # {"ready":true}
```

Mở `http://<ip-node-gpu>:5001` từ máy khác trong mạng.

## VRAM — chỗ dễ làm sập nhất

Card 16 GB, Qwen2.5-VL đang giữ ~9,3 GB. Còn khoảng 6,7 GB.

`--gpu-memory-utilization` của vLLM là tỉ lệ trên **toàn bộ card**, không phải
trên phần còn trống. Bản gốc `docker-compose.vllm.yml` để `0.90`, nghĩa là xin
14,4 GB — vượt xa chỗ trống, vLLM sẽ chết lúc khởi động hoặc tệ hơn là đẩy tiến
trình Qwen vào OOM.

Ở đây mặc định `VLLM_GPU_FRACTION=0.20` ≈ 3,2 GB. Vintern-1B ở bfloat16 chỉ tốn
~2 GB trọng số, phần dư dành cho KV cache. Nâng lên thì kiểm `nvidia-smi` trước,
và nhớ trừ phần Qwen đang giữ.

## Vài điều đã biết trước

**`app.py` cũng đăng ký blueprint OCR** (`app.py:184`), nên `:5001` cũng có
`/api/ocr/tile`. **Đừng trỏ DigiText vào đó.** Image web cố ý không cài
PaddleOCR/VietOCR (xem `ocr/requirements-web.txt`), nên endpoint đó ở `:5001` sẽ
trả 501. Kể cả có cài, trỏ vào đó cũng buộc OCR của DigiText phải sống nhờ SQL
Server — đúng thứ mà việc tách hai dịch vụ nhằm tránh.

**Truy cập file qua UNC không dùng được trên Linux.** `external_routes.py:10`
mặc định `\103.154.62.17\UploadFile`, đường dẫn Windows. Trên Ubuntu phải mount
CIFS rồi trỏ `EXTERNAL_FILE_SERVER_PATH` vào điểm mount. Chưa mount thì chỉ các
route `/api/external/*` hỏng, phần còn lại của web chạy bình thường.

**Mật khẩu `sa` bị chốt vào volume lúc khởi tạo.** Đổi `DB_PASSWORD` trong `.env`
về sau không có tác dụng — SQL Server đã lưu nó trong `sqlserver-data`. Muốn đổi
thật thì `ALTER LOGIN`, hoặc xoá volume và mất toàn bộ dữ liệu.

**Không có test nào.** Cả DigiText lẫn `doc_extract_pla` đều 0 test cho mã sản
phẩm. Mọi kiểm chứng ở trên là thủ công, và đó là tất cả những gì có.
