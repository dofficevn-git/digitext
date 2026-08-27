#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
info()  { echo -e "${GREEN}[INFO]${NC}  $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; }

check_prereqs() {
    if [[ $EUID -ne 0 ]]; then
        error "Script này cần chạy với quyền root (sudo)."
        exit 1
    fi

    local cmds=(openssl curl systemctl ufw)
    for cmd in "${cmds[@]}"; do
        if ! command -v "$cmd" &>/dev/null; then
            error "Thiếu lệnh '$cmd' — cài đặt trước khi chạy."
            exit 1
        fi
    done

    if [[ ! -f /usr/local/bin/digitext-api ]]; then
        warn "Chưa thấy binary /usr/local/bin/digitext-api — sẽ deploy sau?"
    fi
}

setup_jwt_key() {
    local key_path="$1"
    if [[ -f "$key_path" ]]; then
        info "Khoá JWT đã tồn tại: $key_path"
        return
    fi

    info "Tạo khoá JWT Ed25519…"
    install -d -o digitext -g digitext -m 0750 "$(dirname "$key_path")" 2>/dev/null || true
    local tmp_key
    tmp_key=$(mktemp)
    openssl genpkey -algorithm ed25519 -out "$tmp_key"
    install -o digitext -g digitext -m 0600 "$tmp_key" "$key_path"
    shred -u "$tmp_key"

    info "Khoá JWT đã tạo: $key_path"
    warn "⚠ SAO LƯU $key_path ở nơi an toàn! Mất khoá = mất mọi phiên đăng nhập."
    read -rp "  Nhấn Enter sau khi đã sao lưu… "
}

setup_systemd() {
    local domain="$1"
    local db_url="$2"
    local jwt_path="$3"

    info "Tạo /etc/digitext/api.env…"
    cat > /etc/digitext/api.env <<ENVEOF
DATABASE_URL=${db_url}
DIGITEXT_BIND=127.0.0.1:8080
DIGITEXT_JWT_KEY_PATH=${jwt_path}
DIGITEXT_DATA_DIR=/var/lib/digitext
DIGITEXT_BEHIND_PROXY=true
RUST_LOG=info
ENVEOF
    chown root:digitext /etc/digitext/api.env
    chmod 0640 /etc/digitext/api.env

    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local repo_root
    repo_root="$(cd "$script_dir/.." && pwd)"
    local unit_src="${repo_root}/infra/launchd-or-systemd/systemd/digitext@.service"

    if [[ ! -f "$unit_src" ]]; then
        error "Không tìm thấy systemd unit template: $unit_src"
        exit 1
    fi

    info "Cài systemd unit digitext@.service…"
    cp "$unit_src" /etc/systemd/system/digitext@.service

    id -u digitext &>/dev/null || useradd --system --no-create-home --shell /usr/sbin/nologin digitext
    id -g digitext &>/dev/null || groupadd --system digitext
    usermod -aG digitext digitext 2>/dev/null || true

    install -d -o digitext -g digitext -m 0750 /var/lib/digitext

    systemctl daemon-reload
    systemctl enable --now digitext@api

    info "Đợi service khởi động…"
    sleep 2
    if curl -sf http://127.0.0.1:8080/healthz >/dev/null 2>&1; then
        info "digitext@api: OK (healthz trả ok)"
    else
        warn "digitext@api chưa phản hồi — kiểm tra: journalctl -u digitext@api -n 20 --no-pager"
    fi
}

setup_tunnel() {
    info "Cài cloudflared (kho Cloudflare)…"
    if ! command -v cloudflared &>/dev/null; then
        curl -fsSL https://pkg.cloudflare.com/cloudflare-main.gpg \
            | tee /usr/share/keyrings/cloudflare-main.gpg >/dev/null
        echo "deb [signed-by=/usr/share/keyrings/cloudflare-main.gpg] https://pkg.cloudflare.com/cloudflared any main" \
            | tee /etc/apt/sources.list.d/cloudflared.list >/dev/null
        apt update && apt install -y cloudflared
    else
        info "cloudflared đã cài."
    fi

    info ""
    info "========== Cloudflare Tunnel Login =========="
    info "Trình duyệt sẽ mở — đăng nhập Cloudflare và chọn domain."
    cloudflared tunnel login

    info "Tạo tunnel 'digitext-api'…"
    cloudflared tunnel create digitext-api

    local tunnel_json tunnel_id
    tunnel_json=$(ls -t ~/.cloudflared/*.json 2>/dev/null | head -1)
    if [[ -z "$tunnel_json" ]]; then
        error "Không tìm thấy tunnel credentials JSON."
        exit 1
    fi
    tunnel_id=$(basename "$tunnel_json" .json)

    info "Tunnel ID: $tunnel_id"

    install -d -m 0700 /etc/cloudflared
    cp "$tunnel_json" "/etc/cloudflared/${tunnel_id}.json"

    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local repo_root
    repo_root="$(cd "$script_dir/.." && pwd)"
    local config_src="${repo_root}/infra/cloudflared/config.yml"
    local config_dst="/etc/cloudflared/config.yml"

    if [[ -f "$config_src" ]]; then
        cp "$config_src" "$config_dst"
    else
        warn "Không tìm thấy config mẫu — tạo mới."
        cat > "$config_dst" <<'YAMLEOF'
tunnel: <TUNNEL_ID>
credentials-file: /etc/cloudflared/<TUNNEL_ID>.json

ingress:
  - hostname: api.example.vn
    service: http://127.0.0.1:8080
  - service: http_status:404
YAMLEOF
    fi

    sed -i "s/<TUNNEL_ID>/${tunnel_id}/g" "$config_dst"
    sed -i "s/api\.example\.vn/${domain}/g" "$config_dst"

    info "Cấu hình /etc/cloudflared/config.yml hoàn tất."
    info "Tạo route DNS…"
    cloudflared tunnel route dns digitext-api "$domain"

    info "Cài cloudflared service (systemd)…"
    cloudflared service install

    info "Đợi tunnel kết nối…"
    sleep 3
    if curl -sf "https://${domain}/healthz" >/dev/null 2>&1; then
        info "https://${domain}/healthz → OK"
    else
        warn "Tunnel chưa hoạt động — kiểm tra: sudo journalctl -u cloudflared -n 20 --no-pager"
    fi
}

setup_firewall() {
    info "Cấu hình UFW (đóng mọi cổng vào)…"
    ufw --force reset
    ufw default deny incoming
    ufw default allow outgoing
    ufw allow OpenSSH
    ufw --force enable
    info "ufw status:"
    ufw status verbose
}

main() {
    echo "========================================================"
    echo "  Triển khai DigiText Remote Access (Cloudflare Tunnel)"
    echo "========================================================"
    echo ""

    check_prereqs

    read -rp "Domain (vd: api.digitext.vn): " DOMAIN
    [[ -z "$DOMAIN" ]] && { error "Cần domain."; exit 1; }

    read -rp "DATABASE_URL (postgres://user:pass@127.0.0.1:5432/digitext): " DB_URL
    [[ -z "$DB_URL" ]] && { error "Cần DATABASE_URL."; exit 1; }

    local JWT_PATH="${JWT_PATH:-/etc/digitext/jwt-ed25519.pem}"

    echo ""
    echo "BẮT ĐẦU TRIỂN KHAI"
    echo "=================="

    echo ""
    info "[Bước 1/5] Khoá JWT cố định…"
    setup_jwt_key "$JWT_PATH"

    echo ""
    info "[Bước 2/5] Systemd service digitext@api…"
    setup_systemd "$DOMAIN" "$DB_URL" "$JWT_PATH"

    echo ""
    info "[Bước 3/5] Cloudflare Tunnel…"
    setup_tunnel "$DOMAIN"

    echo ""
    info "[Bước 4/5] Tường lửa UFW…"
    setup_firewall

    echo ""
    info "[Bước 5/5] Cloudflare Dashboard — làm thủ công:"
    cat <<MANUAL
  • SSL/TLS → Full (strict), Min TLS 1.3, bật HSTS
  • WAF → Managed rules: bật; Bot Fight Mode: bật
  • Rate limiting:
      /api/v1/auth/login   → 10 req/phút/IP
      /api/v1/auth/refresh → 30 req/phút/IP
  • (Tuỳ chọn) WAF geo rule: chỉ VN
MANUAL

    echo ""
    echo "========================================================"
    info "Triển khai hoàn tất!"
    echo ""
    echo "Kiểm tra nhanh:"
    echo "  1. curl https://${DOMAIN}/healthz"
    echo "  2. Desktop → Cài đặt → Máy chủ → nhập https://${DOMAIN}"
    echo ""
    echo "Ghi chú bảo mật:"
    echo "  • DIGITEXT_BEHIND_PROXY=true — app chỉ tin CF-Connecting-IP"
    echo "    khi peer là loopback (chống header giả mạo)."
    echo "  • Sao lưu khoá JWT + Postgres định kỳ."
    echo "========================================================"
}

main "$@"
