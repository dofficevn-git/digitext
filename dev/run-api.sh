#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="$ROOT/crates/digitext-api/.env"
EXAMPLE="$ROOT/crates/digitext-api/.env.example"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "⚠  Chưa có $ENV_FILE — copy từ .env.example." >&2
  cp "$EXAMPLE" "$ENV_FILE"
  echo "   Đã tạo. Sửa DATABASE_URL cho khớp infra/compose/.env rồi chạy lại." >&2
  exit 1
fi

set -a
source "$ENV_FILE"
set +a

: "${DATABASE_URL:?DATABASE_URL chưa được đặt trong $ENV_FILE}"
echo "→ bind=${DIGITEXT_BIND:-127.0.0.1:8080}  db=${DATABASE_URL%%:*}://…@${DATABASE_URL##*@}"
exec cargo run -p digitext-api -- "$@"
