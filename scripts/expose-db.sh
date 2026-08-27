#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
info()  { echo -e "${GREEN}[INFO]${NC}  $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; }

if [[ $EUID -ne 0 ]]; then
    error "Cần quyền root (sudo)."
    exit 1
fi

PG_VERSION=${PG_VERSION:-16}
PG_CONF_DIR=$(find /etc/postgresql -maxdepth 1 -type d -name "${PG_VERSION}*" 2>/dev/null | head -1)
if [[ -z "$PG_CONF_DIR" ]]; then
    PG_CONF_DIR=$(find /etc/postgresql -maxdepth 1 -type d | sort | tail -1 2>/dev/null)
fi
if [[ -z "$PG_CONF_DIR" ]]; then
    error "Không tìm thấy thư mục cấu hình Postgres (thử đặt PG_VERSION)."
    exit 1
fi

PG_CONF="$PG_CONF_DIR/postgresql.conf"
PG_HBA="$PG_CONF_DIR/pg_hba.conf"

if [[ ! -f "$PG_CONF" ]] || [[ ! -f "$PG_HBA" ]]; then
    error "Không tìm thấy postgresql.conf hoặc pg_hba.conf trong $PG_CONF_DIR"
    exit 1
fi

info "Cấu hình Postgres tại: $PG_CONF_DIR"

if grep -q "^listen_addresses" "$PG_CONF"; then
    sed -i "s/^listen_addresses.*/listen_addresses = '*'/" "$PG_CONF"
else
    echo "listen_addresses = '*'" >> "$PG_CONF"
fi
info "postgresql.conf: listen_addresses = '*'"

if ! grep -q "^host.*all.*all.*0\.0\.0\.0/0.*scram-sha-256" "$PG_HBA"; then
    cat <<EOF >> "$PG_HBA"

# --- TẠM THỜI: remote access (scripts/expose-db.sh) ---
host    all             all             0.0.0.0/0               scram-sha-256
host    all             all             ::/0                    scram-sha-256
EOF
    info "pg_hba.conf: thêm remote access (scram-sha-256)"
fi

systemctl restart postgresql 2>/dev/null || pg_ctlcluster "$PG_VERSION" main reload 2>/dev/null || \
    warn "Không restart được Postgres — tự restart tay"

if command -v ufw &>/dev/null; then
    ufw allow 5432/tcp comment "Postgres direct (temporary)"
    info "UFW: mở port 5432/tcp"
    ufw status | grep 5432
fi

REMOTE_USER="${REMOTE_USER:-digitext_remote}"
REMOTE_PASS="${REMOTE_PASS:-$(openssl rand -base64 24)}"

su - postgres -c "psql -c \"CREATE USER ${REMOTE_USER} WITH PASSWORD '${REMOTE_PASS}';\" 2>/dev/null" || \
    warn "User ${REMOTE_USER} có thể đã tồn tại — bỏ qua"
su - postgres -c "psql -c \"GRANT CONNECT ON DATABASE digitext TO ${REMOTE_USER};\"" 2>/dev/null || true

SERVER_IP=$(curl -s https://api.ipify.org 2>/dev/null || curl -s https://icanhazip.com 2>/dev/null || echo "<IP_CUA_SERVER>")

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║       ⚠  CẢNH BÁO BẢO MẬT  ⚠                               ║"
echo "║  Postgres 5432 đang MỞ trên tất cả interface.               ║"
echo "║  - KHÔNG có TLS — mật khẩu + dữ liệu gửi plaintext          ║"
echo "║  - KHÔNG có WAF/rate-limit — brute force qua Internet        ║"
echo "║  - Dùng xong TẮT ngay:  ufw delete allow 5432/tcp           ║"
echo "║     + xoá dòng remote trong pg_hba.conf                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Kết nối từ xa:"
echo "  psql -h ${SERVER_IP} -p 5432 -U ${REMOTE_USER} -d digitext"
echo "  Password: ${REMOTE_PASS}"
echo ""
echo "URL kết nối:"
echo "  postgresql://${REMOTE_USER}:${REMOTE_PASS}@${SERVER_IP}:5432/digitext"
echo ""
echo "Để thu hồi quyền user:"
echo "  su - postgres -c \"psql -c 'DROP USER IF EXISTS ${REMOTE_USER};'\""
