#!/usr/bin/env bash
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WATCHDOG_LIB=1 source "$here/unattended_watchdog.sh"

fails=0
check(){
  if [ "$2" = "$3" ]; then
    echo "ok  - $1"
  else
    echo "FAIL- $1"; echo "     want: [$2]"; echo "     got : [$3]"; fails=$((fails+1))
  fi
}

tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
cd "$tmp"; mkdir -p data

check "chua co gi -> dau" \
  "[2-3/5] tat vLLM / bat dau loc" "$(derive_stage 000 0 5000)"

check "vLLM bat -> boc" \
  "[1/5] BOC VLM: 1200/5000 trang" "$(derive_stage 200 1200 5000)"

printf 'a\nb\n' > data/kie_filtered_v2.jsonl
check "loc xong, bio chua chay" \
  "[3/5] LOC xong, BIO chuan bi chay" "$(derive_stage 000 5000 5000)"

printf 'x\n' > data/bio_v2_full.jsonl
check "bio dang chay hien pct" \
  "[3/5] BIO dang chay: 1/2 trang (50%)" "$(derive_stage 000 5000 5000)"

echo '{}' > data/bio_v2_full_report.json
check "bio xong co report" \
  "[3/5] LOC+BIO xong (1 trang), chuan bi train" "$(derive_stage 000 5000 5000)"

mkdir -p output/kie_v2_full
check "train co output dir" \
  "[4/5] TRAIN LayoutLMv3 (vLLM da tat)" "$(derive_stage 000 5000 5000)"

check "vLLM bat uu tien cao nhat" \
  "[1/5] BOC VLM: 9/5000 trang" "$(derive_stage 200 9 5000)"

echo
if [ "$fails" -eq 0 ]; then
  echo "TAT CA PASS (7/7)"
else
  echo "$fails TEST HONG"; exit 1
fi
