#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
info()  { echo -e "${GREEN}[INFO]${NC}  $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }

BREW_PREFIX=$(brew --prefix)
PG_DATA_DIR=$(find "$BREW_PREFIX/var" -maxdepth 1 -type d -name "postgresql*" 2>/dev/null | head -1)

if [[ -z "$PG_DATA_DIR" ]]; then
    warn "Không tìm thấy Postgres data dir — bỏ qua."
    exit 0
fi

PG_HBA="$PG_DATA_DIR/pg_hba.conf"
PG_CONF="$PG_DATA_DIR/postgresql.conf"

if [[ -f "$PG_HBA" ]]; then
    sed -i '' '/expose-db-mac/d' "$PG_HBA"
    sed -i '' '/0\.0\.0\.0\/0.*scram-sha-256/d' "$PG_HBA"
    sed -i '' '/::\/0.*scram-sha-256/d' "$PG_HBA"
    info "pg_hba.conf: đã xoá remote access"
fi

if [[ -f "$PG_CONF" ]]; then
    sed -i '' "s/^listen_addresses = '*'/listen_addresses = 'localhost'/" "$PG_CONF"
    info "postgresql.conf: listen_addresses = 'localhost'"
fi

REMOTE_USER="${REMOTE_USER:-digitext_remote}"
psql -d postgres -c "DROP USER IF EXISTS ${REMOTE_USER};" 2>/dev/null && \
    info "Đã xoá user ${REMOTE_USER}" || true

brew services restart postgresql@16 2>/dev/null
info "Postgres đã restart — chỉ listen localhost."
