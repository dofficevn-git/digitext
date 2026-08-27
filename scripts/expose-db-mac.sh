#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
info()  { echo -e "${GREEN}[INFO]${NC}  $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; }

if ! command -v psql &>/dev/null; then
    info "Cài postgresql@16 qua Homebrew…"
    brew install postgresql@16
fi

BREW_PREFIX=$(brew --prefix)

PG_DATA_DIR=$(find "$BREW_PREFIX/var" -maxdepth 1 -type d -name "postgresql*" 2>/dev/null | head -1)
if [[ -z "$PG_DATA_DIR" ]]; then
    error "Không tìm thấy thư mục data Postgres — đã brew install chưa?"
    info "Thử: brew install postgresql@16 && brew services start postgresql@16"
    exit 1
fi
info "Postgres data dir: $PG_DATA_DIR"

PG_CONF="$PG_DATA_DIR/postgresql.conf"
PG_HBA="$PG_DATA_DIR/pg_hba.conf"

if [[ ! -f "$PG_CONF" ]]; then
    error "Không tìm thấy $PG_CONF"
    exit 1
fi

if [[ ! -f "$PG_HBA" ]]; then
    info "Khởi tạo database cluster…"
    initdb "$PG_DATA_DIR" 2>/dev/null || true
fi

if grep -q "^listen_addresses" "$PG_CONF" 2>/dev/null; then
    sed -i '' 's/^listen_addresses.*/listen_addresses = '\''*'\''/' "$PG_CONF"
else
    echo "listen_addresses = '*'" >> "$PG_CONF"
fi
info "postgresql.conf: listen_addresses = '*'"

if ! grep -q "expose-db-mac" "$PG_HBA" 2>/dev/null; then
    cat <<EOF >> "$PG_HBA"

# --- TẠM THỜI: remote access (scripts/expose-db-mac.sh) ---
host    all             all             0.0.0.0/0               scram-sha-256
host    all             all             ::/0                    scram-sha-256
EOF
    info "pg_hba.conf: thêm remote access"
fi

if brew services list 2>/dev/null | grep -q "postgresql.*started"; then
    info "Restart Postgres…"
    brew services restart postgresql@16
else
    info "Start Postgres…"
    brew services start postgresql@16
fi
sleep 2

FW_STATUS=$(/usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate 2>/dev/null || echo "unknown")
if echo "$FW_STATUS" | grep -q "enabled"; then
    warn "macOS Application Firewall đang BẬT."
    warn "Cần cho phép postgres qua firewall hoặc tắt tạm:"
    warn "  sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /opt/homebrew/opt/postgresql@16/bin/postgres"
    warn "  sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblockapp /opt/homebrew/opt/postgresql@16/bin/postgres"
fi

REMOTE_USER="${REMOTE_USER:-digitext_remote}"
REMOTE_PASS="${REMOTE_PASS:-$(openssl rand -base64 24)}"

psql -d postgres -c "CREATE USER ${REMOTE_USER} WITH PASSWORD '${REMOTE_PASS}';" 2>/dev/null || \
    warn "User ${REMOTE_USER} có thể đã tồn tại"
psql -d postgres -c "GRANT CONNECT ON DATABASE digitext TO ${REMOTE_USER};" 2>/dev/null || true

IP_LOCAL=$(ipconfig getifaddr en0 2>/dev/null || echo "<IP_LAN>")
IP_PUBLIC=$(curl -s https://api.ipify.org 2>/dev/null || echo "<IP_PUBLIC>")

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║       ⚠  CẢNH BÁO BẢO MẬT  ⚠                               ║"
echo "║  Postgres 5432 đang MỞ trên tất cả interface.               ║"
echo "║  - KHÔNG có TLS — dữ liệu gửi plaintext                     ║"
echo "║  - KHÔNG có WAF/rate-limit                                  ║"
echo "║  - Dùng xong TẮT ngay:  scripts/close-db-mac.sh             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "LAN (trong nhà):"
echo "  psql -h ${IP_LOCAL} -p 5432 -U ${REMOTE_USER} -d postgres"
echo "  postgresql://${REMOTE_USER}:${REMOTE_PASS}@${IP_LOCAL}:5432/postgres"
echo ""
echo "WAN (qua Internet — nếu router mở port):"
echo "  psql -h ${IP_PUBLIC} -p 5432 -U ${REMOTE_USER} -d postgres"
echo "  postgresql://${REMOTE_USER}:${REMOTE_PASS}@${IP_PUBLIC}:5432/postgres"
echo ""
echo "Kiểm tra Postgres đang listen:"
echo "  lsof -i :5432"
