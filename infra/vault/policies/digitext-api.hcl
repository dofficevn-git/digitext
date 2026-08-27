# infra/vault/policies/digitext-api.hcl
# Policy least-privilege gắn với AppRole `digitext-api`. Chỉ đọc đúng thứ app cần.

# Dynamic Postgres creds — Vault sinh user/password sống ngắn theo yêu cầu.
path "database/creds/digitext-api" {
  capabilities = ["read"]
}

# Static secrets (KV v2): khoá ký JWT Ed25519, v.v.
path "secret/data/digitext/api" {
  capabilities = ["read"]
}
path "secret/metadata/digitext/api" {
  capabilities = ["read"]
}

# Agent tự gia hạn / huỷ token của chính nó.
path "auth/token/renew-self" {
  capabilities = ["update"]
}
path "auth/token/revoke-self" {
  capabilities = ["update"]
}
