#!/usr/bin/env bash
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")" || exit 2
ROOT=/home/do/DigiText
LIMIT="${LIMIT:-5000}"

echo "=== [1/5] BÓC lên $LIMIT trang bằng prompt mới $(date +'%m-%d %H:%M') ==="
docker compose -f "$ROOT/infra/compose/vllm/docker-compose.yml" up -d 2>&1 | tail -1
until curl -s -o /dev/null -m 3 http://127.0.0.1:8003/v1/models; do sleep 10; done
echo "vLLM sẵn sàng | silver_v2 trước: $(wc -l < data/kie_silver_v2.jsonl 2>/dev/null || echo 0)"
python3 extract_vlm.py ../../data --file-list data/sample.txt --only-header-pages \
  --limit "$LIMIT" --out data/kie_silver_v2.jsonl --img-dir data/img
echo "silver_v2 sau: $(wc -l < data/kie_silver_v2.jsonl) dòng"

echo; echo "=== [2/5] TẮT vLLM $(date +%H:%M) ==="
docker compose -f "$ROOT/infra/compose/vllm/docker-compose.yml" down 2>&1 | tail -1
sleep 8; nvidia-smi --query-gpu=memory.free --format=csv,noheader

echo; echo "=== [3/5] LỌC + BIO $(date +%H:%M) — CPU, ~10s/trang (chạy lâu) ==="
python3 filter_rules.py data/kie_silver_v2.jsonl ../../data --out data/kie_filtered_v2.jsonl 2>&1 | tail -2
python3 build_bio.py data/kie_filtered_v2.jsonl ../../data --out data/bio_v2_full.jsonl \
  --report data/bio_v2_full_report.json --concurrency 6
[ -s data/bio_v2_full.jsonl ] || { echo "LỖI: bio_v2_full rỗng"; exit 1; }
echo "bio_v2_full: $(wc -l < data/bio_v2_full.jsonl) trang, match_rate $(python3 -c "import json;print(json.load(open('data/bio_v2_full_report.json'))['match_rate'])")"

echo; echo "=== [4/5] TRAIN model đầy đủ (prompt mới) $(date +%H:%M) ==="
BIO_JSONL=data/bio_v2_full.jsonl OUT_DIR=output/kie_v2_full EPOCHS="${EPOCHS:-10}" bash train_kie.sh 2>&1 | tail -5

echo; echo "=== [5/5] EVAL trên GOLD $(date +%H:%M) ==="
MODEL_DIR=output/kie_v2_full OUT=data/kie_gold_report_v2_full.json bash eval_kie.sh 2>&1 | tail -2

echo; echo "=== KẾT QUẢ $(date +%H:%M) — prompt MỚI full vs model CŨ 4894-trang ==="
python3 - <<'PY'
import json, os
def load(p):
    if not os.path.exists(p): return None
    d = json.load(open(p)); return d["overall"], d["by_field"]
old = load("data/kie_gold_report.json")       # model 4894-trang, prompt CŨ
new = load("data/kie_gold_report_v2_full.json")# model ~5000-trang, prompt MỚI
for tag, r in [("CŨ (4894tr, prompt cũ)", old), ("MỚI (~5000tr, prompt mới)", new)]:
    if r is None: print(f"{tag}: (chưa có)"); continue
    o, bf = r
    print(f"\n{tag}: macro={o['f1_macro']:.4f} định danh={o['f1_identifier']:.4f} thường={o['f1_non_identifier']:.4f}")
    for k in ("coQuanBanHanh","trichYeu","loaiVanBan","soKyHieu","ngayThang"):
        if k in bf:
            v=bf[k]; print(f"   {k:16} F1={v['f1']:.3f} P={v['precision']:.3f} R={v['recall']:.3f}")
if old and new:
    print(f"\nΔ nhóm thường: {new[0]['f1_non_identifier']-old[0]['f1_non_identifier']:+.4f} "
          "(prompt sạch + đủ data; kỳ vọng dương rõ)")
PY
echo "=== PIPELINE_FULL XONG $(date +'%m-%d %H:%M') ==="
