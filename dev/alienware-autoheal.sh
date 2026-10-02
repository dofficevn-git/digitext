#!/usr/bin/env bash
# Tự chữa trên Alienware: container Docker nào health = unhealthy thì khởi động lại (Docker chỉ đánh dấu,
# không tự restart). Cài bằng cron của user `do`, chạy mỗi 2 phút:
#   */2 * * * * /home/do/Downloads/digitext-gpu-bundle/dev/alienware-autoheal.sh
# Nhật ký: ~/autoheal.log. Sự cố 2/10/2026: OCR :5002 treo 3 giờ vì vLLM ngốn RAM, không ai biết.
set -u
LOG="$HOME/autoheal.log"
for c in $(docker ps --filter health=unhealthy --format '{{.Names}}'); do
  mem=$(free -m | awk '/^Mem:/{print $7}')
  echo "$(date '+%F %T') unhealthy: $c (RAM trống ${mem} MB) → restart" >> "$LOG"
  docker restart "$c" >> "$LOG" 2>&1
done
# Gọn nhật ký
if [ -f "$LOG" ] && [ "$(wc -l < "$LOG")" -gt 2000 ]; then tail -n 1000 "$LOG" > "$LOG.tmp" && mv "$LOG.tmp" "$LOG"; fi
