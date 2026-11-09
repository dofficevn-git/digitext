#!/usr/bin/env bash
# CÀI MÁY CHỦ DIGITEXT LƯU TRỮ TRÊN LINUX (Docker Compose) — bản Lưu trữ GĐ2, 9/11/2026, docs/CAI-DAT-MAY-CHU-LUU-TRU.md.
#   sudo ./cai-dat.sh luutru.co-quan.vn            có tên miền → Let's Encrypt
#   sudo ./cai-dat.sh 192.168.1.20                 mạng nội bộ → chứng chỉ tự cấp, phát chứng chỉ gốc cho máy trạm
# Biến tuỳ chọn: KHO_DIR (mặc định /srv/digitext-kho), SAO_LUU_DIR (/srv/digitext-saoluu), MA_DU_AN (LUUTRU),
#   TEN_CO_QUAN, QUAN_TRI (quantri), QUAN_TRI_MAT_KHAU (cài không người gõ), TLS_NOI_BO=1 (ép chứng chỉ tự cấp).
# Chạy lại được: .env (mật khẩu) giữ nguyên, dữ liệu giữ nguyên. Cần: Docker Engine + plugin compose, mạng tải image.
set -euo pipefail
cd "$(dirname "$0")"
GOC=$(pwd)
ghi() { printf '%s %s\n' "$(date '+%F %T')" "$*" | tee -a cai-dat.log; }
dung() { ghi "LỖI: $*"; exit 1; }
# Không dùng `tr </dev/urandom | head`: head đóng ống trước → tr chết SIGPIPE → pipefail + set -e thoát âm thầm.
ngau_nhien() { local s; s=$(head -c 512 /dev/urandom | LC_ALL=C tr -dc 'A-Za-z0-9'); printf '%s' "${s:0:${1:-24}}"; }
dc() { docker compose "$@"; }

command -v docker >/dev/null || dung "chưa cài Docker Engine"
docker compose version >/dev/null 2>&1 || dung "chưa có plugin docker compose"
command -v rsync >/dev/null || echo "CẢNH BÁO: chưa cài rsync — sao lưu kho hằng đêm sẽ lỗi (apt install rsync)"

# --- 0. Cấu hình (giữ giá trị cũ)
[ -f .env ] && set -a && . ./.env && set +a
DIA_CHI=${1:-${DIA_CHI:-}}
[ -n "$DIA_CHI" ] || dung "thiếu địa chỉ: ./cai-dat.sh <tên miền | tên máy | IP>"
DIA_CHI=$(echo "$DIA_CHI" | tr 'A-Z' 'a-z' | sed -E 's#^https?://##; s#/.*$##')
KHO_DIR=${KHO_DIR:-/srv/digitext-kho}
SAO_LUU_DIR=${SAO_LUU_DIR:-/srv/digitext-saoluu}
MA_DU_AN=${MA_DU_AN:-LUUTRU}
QUAN_TRI=${QUAN_TRI:-quantri}
PG_PASSWORD=${PG_PASSWORD:-$(ngau_nhien 24)}
NOI_BO=0
if [ "${TLS_NOI_BO:-0}" = 1 ] || [[ "$DIA_CHI" =~ ^[0-9]+(\.[0-9]+){3}$ ]] || [[ "$DIA_CHI" != *.* ]] || [[ "$DIA_CHI" =~ \.(local|lan|internal|home|corp)$ ]]; then NOI_BO=1; fi
umask 077
cat > .env <<EOF
# Sinh bởi cai-dat.sh $(date '+%d/%m/%Y %H:%M') — chứa mật khẩu, không chép đi nơi khác.
DIA_CHI=$DIA_CHI
KHO_DIR=$KHO_DIR
SAO_LUU_DIR=$SAO_LUU_DIR
MA_DU_AN=$MA_DU_AN
QUAN_TRI=$QUAN_TRI
PG_PASSWORD=$PG_PASSWORD
EOF
umask 022
mkdir -p "$KHO_DIR" "$SAO_LUU_DIR" du-lieu/api du-lieu/caddy du-lieu/postgres updates goi-nhap
chown 10001 du-lieu/api "$KHO_DIR" 2>/dev/null || true
ghi "=== CÀI DIGITEXT LƯU TRỮ · địa chỉ $DIA_CHI · chứng chỉ $([ $NOI_BO = 1 ] && echo 'tự cấp (mạng nội bộ)' || echo "Let's Encrypt") · kho $KHO_DIR"

# --- 1. Caddyfile
{
  echo "# Sinh bởi cai-dat.sh — Caddy trước API DigiText Lưu trữ."
  echo "$DIA_CHI {"
  [ $NOI_BO = 1 ] && printf '\ttls internal\n'
  printf '\treverse_proxy api:8080 {\n\t\ttransport http {\n\t\t\tread_timeout 30m\n\t\t\twrite_timeout 30m\n\t\t}\n\t}\n}\n'
  if [ $NOI_BO = 1 ]; then
    echo "# Mạng nội bộ: cổng 80 chỉ phát chứng chỉ gốc + trang tải về cho máy trạm chưa tin HTTPS."
    echo "http://$DIA_CHI {"
    printf '\t@chungchi path /updates/chung-chi-goc.crt /updates/cai-chung-chi-goc.cmd /tai-ve\n'
    printf '\thandle @chungchi {\n\t\treverse_proxy api:8080\n\t}\n'
    # redir đứng trước handle trong thứ tự directive của Caddy → bọc vào handle dự phòng.
    printf '\thandle {\n\t\tredir https://{host}{uri}\n\t}\n}\n'
  fi
} > Caddyfile

# --- 2. Image: gói phát hành kèm digitext-luutru-image.tar.gz (docker save) → nạp; không có thì dựng từ mã nguồn.
if [ -f digitext-luutru-image.tar.gz ]; then
  ghi "nạp image từ digitext-luutru-image.tar.gz…"
  gunzip -c digitext-luutru-image.tar.gz | docker load
elif ! docker image inspect digitext-luutru:local >/dev/null 2>&1; then
  [ -f ../../../Cargo.toml ] || dung "không có digitext-luutru-image.tar.gz và không có mã nguồn để dựng image"
  ghi "dựng image digitext-luutru:local từ mã nguồn (lần đầu mất 10–20 phút)…"
  dc build migrate
fi
dc up -d postgres
dc up migrate || dung "áp migrations lỗi — xem 'docker compose logs migrate'"
dc up -d api worker caddy
for i in $(seq 1 60); do dc exec -T api bash -c 'exec 3<>/dev/tcp/127.0.0.1/8080' 2>/dev/null && break; sleep 2; done

psql() { dc exec -T postgres psql -X -q -t -A -U digitext -d digitext "$@"; }
# --- 3. Quản trị đầu tiên + kho lưu trữ đầu tiên
if [ "$(psql -c 'SELECT count(*) FROM users')" = 0 ]; then
  MK=${QUAN_TRI_MAT_KHAU:-}
  while [ ${#MK} -lt 8 ]; do
    read -r -s -p "Mật khẩu cho tài khoản quản trị '$QUAN_TRI' (tối thiểu 8 ký tự): " MK; echo
    read -r -s -p "Nhập lại: " MK2; echo
    [ "$MK" = "$MK2" ] || { echo "Hai lần nhập không khớp."; MK=; }
  done
  # Mật khẩu đi qua stdin vào tệp tạm TRONG container, không nằm trên dòng lệnh / đĩa máy chủ.
  printf '%s\n' "$MK" | dc exec -T api sh -c 'umask 077; cat > /tmp/mk-quantri'
  KQ=0
  dc exec -T api digitext-api bootstrap-superadmin --username "$QUAN_TRI" --display-name "Quản trị hệ thống" --password-file /tmp/mk-quantri || KQ=1
  dc exec -T api rm -f /tmp/mk-quantri
  MK=; MK2=
  [ $KQ = 0 ] || dung "không tạo được quản trị"
  ghi "đã tạo tài khoản quản trị '$QUAN_TRI'"
fi
if [ "$(psql -c 'SELECT count(*) FROM projects')" = 0 ]; then
  TEN_CO_QUAN=${TEN_CO_QUAN:-}
  [ -n "$TEN_CO_QUAN" ] || read -r -p "Tên cơ quan / kho lưu trữ: " TEN_CO_QUAN
  psql -v ma="$MA_DU_AN" -v ten="${TEN_CO_QUAN:-Kho lưu trữ}" <<'SQL'
INSERT INTO projects (id, code, name) VALUES (gen_random_uuid(), upper(:'ma'), :'ten');
SQL
  ghi "đã tạo kho lưu trữ $MA_DU_AN"
fi

# --- 4. Chứng chỉ gốc cho máy trạm (mạng nội bộ)
if [ $NOI_BO = 1 ]; then
  ROOT=du-lieu/caddy/caddy/pki/authorities/local/root.crt
  for i in $(seq 1 30); do [ -f "$ROOT" ] && break; sleep 2; done
  if [ -f "$ROOT" ]; then
    cp "$ROOT" updates/chung-chi-goc.crt
    {
      printf '@echo off\r\nset "F=%%TEMP%%\\digitext-chung-chi-goc.crt"\r\nmore +5 "%%~f0" > "%%F%%"\r\n'
      printf 'certutil -user -addstore -f Root "%%F%%" && (echo Da cai chung chi goc may chu DigiText Luu tru.) || (echo Khong cai duoc chung chi.)\r\n'
      printf 'del "%%F%%" & pause & exit /b\r\n'
      sed 's/$/\r/' "$ROOT"
    } > updates/cai-chung-chi-goc.cmd
    ghi "chứng chỉ gốc: updates/chung-chi-goc.crt + cai-chung-chi-goc.cmd (máy trạm tải ở http://$DIA_CHI/tai-ve)"
  else
    ghi "CẢNH BÁO: Caddy chưa sinh chứng chỉ gốc — xem 'docker compose logs caddy', chạy lại cai-dat.sh"
  fi
fi

# --- 5. Sao lưu 01:00 hằng đêm
if [ -d /etc/cron.d ] && [ -w /etc/cron.d ]; then
  echo "0 1 * * * root $GOC/sao-luu.sh >> $GOC/sao-luu.log 2>&1" > /etc/cron.d/digitext-luutru
  ghi "sao lưu 01:00 hằng đêm → $SAO_LUU_DIR (/etc/cron.d/digitext-luutru)"
else
  ghi "CẢNH BÁO: không ghi được /etc/cron.d — tự đặt lịch chạy $GOC/sao-luu.sh hằng đêm"
fi
[ -f updates/latest.json ] || ghi "CHƯA có bộ cài máy trạm trong updates/ — chép gói phát hành DigiText Lưu trữ vào."
ghi "=== XONG. Máy trạm mở https://$DIA_CHI/tai-ve. Nhập gói từ ổ cứng: chép vào goi-nhap/ rồi"
ghi "    docker compose exec api digitext-api nhap-goi --du-an $MA_DU_AN --nguoi $QUAN_TRI /goi"
