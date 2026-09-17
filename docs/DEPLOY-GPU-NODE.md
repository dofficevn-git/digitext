# Dựng máy chủ GPU cho OCR + VLM

> Runbook cho **máy chủ GPU mới** (Ubuntu 22.04/24.04, card NVIDIA). Alienware
> `192.168.0.205` là nơi thử; máy đích chưa tồn tại nên mọi cổng, đường dẫn và
> tỉ lệ VRAM đều đi qua `.env`, không sửa file compose.
>
> Kiến trúc và việc đang dở: [PROJECT-MAP.md](PROJECT-MAP.md), [HANDOFF.md](HANDOFF.md).

## 0. Máy này chạy gì

| Dịch vụ | Compose | Cổng | VRAM (card 16 GB) |
|---|---|---|---|
| OCR toàn văn (PaddleOCR det + VietOCR rec) | `infra/compose/ocr` + overlay `docker-compose.gpu.yml` | 5002 | ≤ 3 GB |
| vLLM Qwen2.5-VL-7B-AWQ — bóc 16 trường | `infra/compose/vllm` | 8003 | ~8,8 GB (`VLLM_GPU_FRACTION=0.55`) |
| vLLM Vintern-1B (`vnkie`) — ứng viên thay Qwen | `infra/compose/ocr-web` service `vllm-vnkie` | 3000 | ~4 GB (`VLLM_GPU_FRACTION=0.25`) |

Card 16 GB: **OCR + một VLM**. Qwen và Vintern không chạy cùng lúc — A/B thì
`stop` cái này, `up` cái kia. Card ≥ 24 GB thì chạy cả ba được.

Worker `digitext-ocr` (Rust) và API/Postgres/MinIO có thể ở máy này hoặc máy
khác; chúng chỉ biết máy GPU qua `DIGITEXT_OCR_HTTP_ENDPOINT` và
`DIGITEXT_VLM_ENDPOINT`.

## 1. Chuẩn bị hệ điều hành

```bash
git clone <repo> ~/digitext && cd ~/digitext
sudo scripts/setup-ai-node.sh check      # đọc: GPU, driver, docker, toolkit
sudo scripts/setup-ai-node.sh driver     # có thể cần reboot
sudo scripts/setup-ai-node.sh docker
sudo scripts/setup-ai-node.sh toolkit    # nvidia-container-toolkit
docker run --rm --gpus all nvidia/cuda:12.1.1-base-ubuntu22.04 nvidia-smi -L
```

Dòng cuối phải in tên card. Không in thì dừng — mọi bước sau đều cần nó.

## 2. Chép model (không có trong git)

| Từ | Đến trên máy GPU | Cỡ |
|---|---|---|
| `ocr/models/vietocr/vgg_transformer.pth` (+ bản fine-tune nếu có) | `~/digitext/ocr/models/vietocr/` | 145 MB |
| `D:\doc_extract_pla\models\plakie\` (Windows) | `~/digitext/ocr/models/plakie/` | 1,8 GB |

Cách chuyển đã dùng: Windows `python -m http.server 8899 --bind 0.0.0.0
--directory <thư mục>`, máy GPU `wget -r -np -nH --cut-dirs=1 -R "index.html*"
http://<ip-windows>:8899/plakie/`. Kiểm `ls plakie/config.json` — phải ở gốc,
không lồng `plakie/plakie/`. Trọng số PaddleOCR (~100 MB) và Qwen (~6 GB) tự tải
lần đầu vào volume.

## 3. OCR toàn văn trên GPU

```bash
cd infra/compose/ocr && cp .env.example .env
# sửa .env: OCR_API_KEY (bắt buộc, khớp DIGITEXT_OCR_HTTP_API_KEY bên worker),
#           OCR_BIND=0.0.0.0 nếu worker ở máy khác, OCR_USE_GPU=true, OCR_REC_BATCH=32
docker compose -f docker-compose.yml -f docker-compose.gpu.yml --env-file .env up -d --build
docker compose -f docker-compose.yml -f docker-compose.gpu.yml logs -f ocr
```

Chờ thấy `Đã nạp PaddleOCR (… GPU)` và `Đã nạp VietOCR (…, cuda:0, …)`. Nếu
`OCR_USE_GPU=true` mà torch không thấy CUDA, dịch vụ **cố ý dừng** (health 503)
thay vì rơi về CPU âm thầm.

Kiểm:
```bash
curl -s localhost:5002/api/ocr/health      # {"ready":true,"gpu":true,"device":"cuda:0",...}
nvidia-smi --query-compute-apps=pid,used_memory --format=csv   # sau ~20 request, ≤ 3 GB
```

Mỗi request in một dòng `⏱ tile WxH · dòng=N · det=…ms · tổng=…ms · chờ=…ms`
(`OCR_LOG_TIMING=false` để tắt). `chờ` lớn = `OCR_GPU_CONCURRENCY` đang bó.

Rollback về CPU: bỏ `-f docker-compose.gpu.yml`, `OCR_USE_GPU=false`, `up -d --build`.

## 4. VLM

Qwen (mặc định của v2):
```bash
cd infra/compose/vllm && docker compose up -d && docker compose logs -f   # "Application startup complete"
curl -s localhost:8003/v1/models
```
Đổi VRAM: `VLLM_GPU_FRACTION=0.55` / `VLLM_MAX_NUM_SEQS=4` trong env của compose.

Vintern (khi A/B, sau khi đã `docker compose -f infra/compose/vllm/docker-compose.yml stop`):
```bash
cd infra/compose/ocr-web && cp .env.example .env   # đặt VLLM_GPU_FRACTION=0.25
docker compose --env-file .env up -d vllm-vnkie && docker compose logs -f vllm-vnkie
curl -s localhost:3000/v1/models                    # "id": "vnkie"
```
Không tới được HuggingFace → `sed -i 's|5CD-AI/Vintern-1B-v3_5--||g' ocr/models/plakie/config.json`.

## 5. Trỏ worker DigiText sang máy này

Trong `.env` của worker (`crates/digitext-ocr/.env` hoặc `/etc/digitext/ocr.env`):
```
DIGITEXT_OCR_ENGINE=http
DIGITEXT_OCR_HTTP_ENDPOINT=http://<ip-gpu>:5002
DIGITEXT_OCR_HTTP_API_KEY=<OCR_API_KEY>
DIGITEXT_OCR_HTTP_TIMEOUT_MS=90000
DIGITEXT_MAX_RETRIES=20
DIGITEXT_RETRY_BASE_DELAY_MS=2000
DIGITEXT_RETRY_MAX_DELAY_MS=60000
DIGITEXT_RECOGNITION_ENGINE_NAME=vietocr-vgg-transformer-http   # đổi khi đổi trọng số
# bóc 16 trường v2
DIGITEXT_FIELDS_ENGINE=v2
DIGITEXT_VLM_ENDPOINT=http://<ip-gpu>:8003/v1/chat/completions
DIGITEXT_VLM_MODEL=Qwen2.5-VL-7B-Instruct-AWQ
DIGITEXT_VLM_PROFILE=qwen            # internvl khi trỏ sang vnkie
DIGITEXT_FIELDS_V2_MAX_PAGES=2
```
Sau khi bật worker: `curl -s localhost:9095/readyz` phải 200 (nó gọi thật
`/api/ocr/health` của máy GPU). Khi máy GPU chết, worker ngừng nhận job và tự
nối lại — không cần khởi động lại worker.

## 6. Đo trước khi tin

```bash
# tốc độ
python ml/bench/harness/loadtest.py --manifest <manifest.jsonl> --backend http \
    --endpoint http://<ip-gpu>:5002 --api-key $OCR_API_KEY --concurrency 4 --requests 100
# chất lượng (bộ đồng thuận VLM, xem HANDOFF §3.3)
python ml/bench/harness/run_bench.py --manifest <manifest.jsonl> --backend http \
    --endpoint http://<ip-gpu>:5002 --api-key $OCR_API_KEY --out pred_gpu.jsonl
python ml/bench/harness/score.py --gold gold.jsonl --pred pred_gpu.jsonl --out report_gpu.json
python ml/bench/harness/gate.py --report report_gpu.json --baseline report_cpu_base.json --tol 0.005
```
Cùng trọng số thì CER CPU và GPU phải trùng — lệch là sai cấu hình, không phải "GPU kém hơn".

## 7. Đổi trọng số VietOCR (bản fine-tune)

1. Chép `vgg_transformer_ft_v1.pth` vào `ocr/models/vietocr/`.
2. Container thử: `OCR_REC_WEIGHTS=/models/vietocr/vgg_transformer_ft_v1.pth OCR_PORT=5003
   docker compose -p ocr-ft -f docker-compose.yml -f docker-compose.gpu.yml --env-file .env up -d`
   (lúc VLM đang tắt — thêm ~3 GB VRAM).
3. `run_bench.py --endpoint http://localhost:5003` → `gate.py --baseline report_gpu.json`.
4. Đạt thì đổi `OCR_REC_WEIGHTS` của dịch vụ chính, `up -d`; worker đặt
   `DIGITEXT_RECOGNITION_ENGINE_NAME=vietocr-vgg-transformer-ft-v1` +
   `DIGITEXT_RECOGNITION_ENGINE_ACTIVATE=true` một lần để kết quả mới mang id engine mới.
5. Rollback: đổi lại hai biến trên.
