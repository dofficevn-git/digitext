#!/usr/bin/env bash
set -uo pipefail

derive_stage(){
  local vllm_up="$1" silver="$2" limit="$3"
  local bio filt pct
  bio=$([ -f data/bio_v2_full.jsonl ] && wc -l < data/bio_v2_full.jsonl || echo 0)
  filt=$([ -f data/kie_filtered_v2.jsonl ] && wc -l < data/kie_filtered_v2.jsonl || echo 0)
  if [ "$vllm_up" = "200" ]; then
    echo "[1/5] BOC VLM: $silver/$limit trang"
  elif [ -d output/kie_v2_full ]; then
    echo "[4/5] TRAIN LayoutLMv3 (vLLM da tat)"
  elif [ -s data/bio_v2_full_report.json ]; then
    echo "[3/5] LOC+BIO xong ($bio trang), chuan bi train"
  elif [ -s data/bio_v2_full.jsonl ]; then
    pct=$(python3 -c "print(f'{100*$bio/max(1,$filt):.0f}')" 2>/dev/null || echo '?')
    echo "[3/5] BIO dang chay: $bio/$filt trang (${pct}%)"
  elif [ -s data/kie_filtered_v2.jsonl ]; then
    echo "[3/5] LOC xong, BIO chuan bi chay"
  else
    echo "[2-3/5] tat vLLM / bat dau loc"
  fi
}

[ "${WATCHDOG_LIB:-0}" = 1 ] && return 0

export PATH="/home/do/.local/bin:/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin"
KIE=/home/do/DigiText/ml/finetune/kie
cd "$KIE" || exit 2

STATE="$KIE/data/unattended"
mkdir -p "$STATE"
LOG="$STATE/watchdog.log"
STATUS="$STATE/status.txt"
DONE="$STATE/DONE"
ATTEMPTS="$STATE/attempts"
LOCK="$STATE/watchdog.lock"
REPORT="$KIE/data/kie_gold_report_v2_full.json"
OLD_REPORT="$KIE/data/kie_gold_report.json"
LIMIT="${LIMIT:-5000}"
MAX_RESUMES=3

exec 9>"$LOCK"
flock -n 9 || exit 0

log(){ echo "[$(date '+%m-%d %H:%M:%S')] $*" >> "$LOG"; }

[ -f "$DONE" ] && exit 0

render_summary(){
  python3 - "$OLD_REPORT" "$REPORT" <<'PY'
import json, os, sys
old_p, new_p = sys.argv[1], sys.argv[2]
def load(p):
    if not os.path.exists(p): return None
    d = json.load(open(p)); return d["overall"], d["by_field"]
old, new = load(old_p), load(new_p)
for tag, r in [("CU (4894tr, prompt cu)", old), ("MOI (~5000tr, prompt moi)", new)]:
    if r is None: print(f"{tag}: (chua co)"); continue
    o, bf = r
    print(f"{tag}: macro={o['f1_macro']:.4f} dinh_danh={o['f1_identifier']:.4f} thuong={o['f1_non_identifier']:.4f}")
    for k in ("coQuanBanHanh","trichYeu","loaiVanBan","soKyHieu","ngayThang"):
        if k in bf:
            v=bf[k]; print(f"   {k:16} F1={v['f1']:.3f} P={v['precision']:.3f} R={v['recall']:.3f}")
if old and new:
    d = new[0]['f1_non_identifier'] - old[0]['f1_non_identifier']
    print(f"DELTA nhom thuong (prompt moi full vs cu): {d:+.4f}")
PY
}

if [ -s "$REPORT" ]; then
  { echo "=== PIPELINE DONE @ $(date '+%F %H:%M') ==="; echo; render_summary; } > "$STATUS"
  touch "$DONE"
  log "DONE: co bao cao final $REPORT — watchdog tu tat."
  exit 0
fi

if pgrep -f 'bash .*pipeline_full\.sh' >/dev/null || pgrep -f 'pipeline_full\.sh' >/dev/null; then
  silver=$(wc -l < data/kie_silver_v2.jsonl 2>/dev/null || echo 0)
  vllm_up=$(curl -s -o /dev/null -m 3 -w '%{http_code}' http://127.0.0.1:8003/v1/models 2>/dev/null || echo 000)
  stage=$(derive_stage "$vllm_up" "$silver" "$LIMIT")
  gpu=$(nvidia-smi --query-gpu=memory.used,memory.free --format=csv,noheader 2>/dev/null | head -1)
  { echo "=== CHAY @ $(date '+%F %H:%M') ==="
    echo "buoc: $stage"
    echo "silver_v2: $silver trang | vLLM http=$vllm_up | GPU used/free: $gpu"
    echo "resume da dung: $(cat "$ATTEMPTS" 2>/dev/null || echo 0)/$MAX_RESUMES"
  } > "$STATUS"
  log "alive | $stage | silver=$silver"
  exit 0
fi

n=$(cat "$ATTEMPTS" 2>/dev/null || echo 0)
if [ "$n" -ge "$MAX_RESUMES" ]; then
  { echo "=== GIVE_UP @ $(date '+%F %H:%M') ==="
    echo "pipeline chet, da resume $n/$MAX_RESUMES lan van chua ra bao cao. Can nguoi xem log."
    echo "log gan nhat: $STATE/pipeline_run_${n}.log"
  } > "$STATUS"
  log "GIVE_UP sau $n resume."
  exit 0
fi
n=$((n+1)); echo "$n" > "$ATTEMPTS"
RUNLOG="$STATE/pipeline_run_${n}.log"
log "RESUME lan $n/$MAX_RESUMES — pipeline down, chua co bao cao. Log: $RUNLOG"
{ echo "=== RESUME lan $n @ $(date '+%F %H:%M') ==="
  echo "pipeline_full.sh khong con chay va chua co bao cao final → khoi dong lai (resume theo page_id)."
} > "$STATUS"
LIMIT="$LIMIT" BIO_RESUME=1 setsid nohup bash "$KIE/pipeline_full.sh" > "$RUNLOG" 2>&1 &
log "da phong pipeline_full.sh detached (PID $!)."
exit 0
