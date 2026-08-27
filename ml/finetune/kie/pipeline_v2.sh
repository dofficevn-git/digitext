#!/usr/bin/env bash
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")" || exit 2
ROOT=/home/do/DigiText
N="${N:-500}"

echo "=== [1/7] BÓC $N trang bằng prompt mới $(date +%H:%M) ==="
curl -s -o /dev/null -m 5 http://127.0.0.1:8003/v1/models || { echo "LỖI: vLLM chưa sẵn sàng"; exit 1; }
python3 extract_vlm.py ../../data --file-list data/sample.txt --only-header-pages \
  --limit "$N" --out data/kie_silver_v2.jsonl --img-dir data/img
echo "silver_v2: $(wc -l < data/kie_silver_v2.jsonl) dòng"

echo; echo "=== [2/7] TẮT vLLM (nhường VRAM cho train) $(date +%H:%M) ==="
docker compose -f "$ROOT/infra/compose/vllm/docker-compose.yml" down 2>&1 | tail -1
sleep 8; nvidia-smi --query-gpu=memory.free --format=csv,noheader

echo; echo "=== [3/7] LỌC + BIO (prompt mới) $(date +%H:%M) ==="
python3 filter_rules.py data/kie_silver_v2.jsonl ../../data --out data/kie_filtered_v2.jsonl 2>&1 | tail -2
python3 build_bio.py data/kie_filtered_v2.jsonl ../../data --out data/bio_v2.jsonl \
  --report data/bio_v2_report.json --concurrency 6 2>&1 | tail -3
[ -s data/bio_v2.jsonl ] || { echo "LỖI: bio_v2 rỗng"; exit 1; }
echo "bio_v2: $(wc -l < data/bio_v2.jsonl) trang"

echo; echo "=== [4/7] DỰNG ĐỐI CHỨNG: bio CŨ, ĐÚNG các trang của v2 $(date +%H:%M) ==="
python3 - <<'PY'
import json
v2_ids = {json.loads(l)["id"] for l in open("data/bio_v2.jsonl")}
kept = [l for l in open("data/bio.jsonl") if json.loads(l)["id"] in v2_ids]
open("data/bio_ctrl.jsonl", "w").writelines(kept)
print(f"bio_ctrl: {len(kept)} trang (giao của v2 ∩ bio cũ)")
PY

echo; echo "=== [5/7] TRAIN v2 (prompt mới) $(date +%H:%M) ==="
BIO_JSONL=data/bio_v2.jsonl OUT_DIR=output/kie_v2 EPOCHS="${EPOCHS:-10}" bash train_kie.sh 2>&1 | tail -4

echo; echo "=== [6/7] TRAIN ctrl (prompt cũ, cùng trang) $(date +%H:%M) ==="
BIO_JSONL=data/bio_ctrl.jsonl OUT_DIR=output/kie_ctrl EPOCHS="${EPOCHS:-10}" bash train_kie.sh 2>&1 | tail -4

echo; echo "=== [7/7] EVAL cả hai trên GOLD $(date +%H:%M) ==="
MODEL_DIR=output/kie_v2  OUT=data/kie_gold_report_v2.json  bash eval_kie.sh 2>&1 | tail -2
MODEL_DIR=output/kie_ctrl OUT=data/kie_gold_report_ctrl.json bash eval_kie.sh 2>&1 | tail -2

echo; echo "=== KẾT QUẢ $(date +%H:%M) ==="
python3 - <<'PY'
import json
def load(p):
    d = json.load(open(p)); o = d["overall"]; bf = d["by_field"]
    return o, bf
for tag, path in [("PROMPT CŨ (ctrl)", "data/kie_gold_report_ctrl.json"),
                  ("PROMPT MỚI (v2)", "data/kie_gold_report_v2.json")]:
    o, bf = load(path)
    print(f"\n{tag}: macro={o['f1_macro']:.4f} định danh={o['f1_identifier']:.4f} thường={o['f1_non_identifier']:.4f}")
    for k in ("coQuanBanHanh","trichYeu","loaiVanBan"):
        if k in bf:
            v=bf[k]; print(f"   {k:16} F1={v['f1']:.3f} P={v['precision']:.3f} R={v['recall']:.3f}")
print("\n(hiệu v2−ctrl trên nhóm THƯỜNG = tác dụng prompt thuần, cùng cỡ dữ liệu)")
PY
echo "=== PIPELINE_V2 XONG $(date +%H:%M) ==="
