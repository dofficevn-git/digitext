#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
cd "$root"

compose="docker compose -f infra/compose/docker-compose.yml"
$compose up -d --wait postgres 2>/dev/null || $compose up -d postgres

for _ in $(seq 1 30); do
  if $compose exec -T postgres pg_isready -U "${PG_USER:-digitext}" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

export DATABASE_URL="postgres://${PG_USER:-digitext}:${PG_PASSWORD:-changeme}@localhost:5432/${PG_DB:-digitext}"
exec cargo test --workspace "$@"
