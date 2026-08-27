#!/usr/bin/env bash
set -euo pipefail
: "${VAULT_ADDR:?export VAULT_ADDR=https://vault.control.internal:8200}"
: "${VAULT_TOKEN:?export VAULT_TOKEN=<admin token>}"
: "${VAULT_PG_ADMIN_PASS:?export VAULT_PG_ADMIN_PASS=...}"

PG_HOST="${PG_HOST:-postgres.control.internal}"
PG_PORT="${PG_PORT:-5432}"
PG_DB="${PG_DB:-digitext}"
PG_ADMIN_USER="${PG_ADMIN_USER:-vault_admin}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "==> (1) KV v2 cho secret tĩnh (khoá JWT…)"
vault secrets enable -path=secret kv-v2 2>/dev/null || echo "   (đã bật)"

echo "==> (2) Database secrets engine — dynamic Postgres creds"
vault secrets enable database 2>/dev/null || echo "   (đã bật)"
vault write database/config/digitext \
  plugin_name=postgresql-database-plugin \
  allowed_roles="digitext-api" \
  connection_url="postgresql://{{username}}:{{password}}@${PG_HOST}:${PG_PORT}/${PG_DB}?sslmode=require" \
  username="${PG_ADMIN_USER}" \
  password="${VAULT_PG_ADMIN_PASS}"

vault write database/roles/digitext-api \
  db_name=digitext \
  creation_statements="CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO \"{{name}}\"; GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO \"{{name}}\";" \
  revocation_statements="REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM \"{{name}}\"; DROP ROLE IF EXISTS \"{{name}}\";" \
  default_ttl="1h" max_ttl="24h"

echo "==> (3) Policy least-privilege"
vault policy write digitext-api "${HERE}/policies/digitext-api.hcl"

echo "==> (4) AppRole auth cho máy chủ app"
vault auth enable approle 2>/dev/null || echo "   (đã bật)"
vault write auth/approle/role/digitext-api \
  token_policies="digitext-api" \
  token_ttl=1h token_max_ttl=4h \
  secret_id_ttl=10m secret_id_num_uses=1 \
  bind_secret_id=true

echo
echo "==> RoleID (không bí mật — đặt vào group_vars: vault_role_id):"
vault read -field=role_id auth/approle/role/digitext-api/role-id
echo
echo "==> Giao SecretID cho host lúc deploy (response-wrapped, TTL ngắn, dùng 1 lần):"
echo "   WRAP=\$(vault write -wrap-ttl=120s -f auth/approle/role/digitext-api/secret-id -format=json | jq -r .wrap_info.token)"
echo "   ansible-playbook … -e vault_wrapped_secret_id=\$WRAP"
