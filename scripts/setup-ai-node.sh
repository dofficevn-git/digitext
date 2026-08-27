#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
info()  { echo -e "${GREEN}[INFO]${NC}  $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
error() { echo -e "${RED}[ERR ]${NC} $*" >&2; }
step()  { echo -e "\n${BLUE}══ $* ══${NC}"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_SRC="${REPO_ROOT}/target/release/digitext-ocr"
BIN_DST="/usr/local/bin/digitext-ocr"
UNIT_SRC="${REPO_ROOT}/infra/launchd-or-systemd/systemd/digitext@.service"
ENV_EXAMPLE="${REPO_ROOT}/infra/launchd-or-systemd/systemd/ocr.env.example"
ENV_DST="/etc/digitext/ocr.env"
COMMON_ENV="/etc/digitext/common.env"
DATA_DIR="/var/lib/digitext"
PAGES_DIR="${DATA_DIR}/pages"
PDF_DIR="${DATA_DIR}/pdf"

DRIVER_PACKAGE="${DRIVER_PACKAGE:-}"

require_root() {
    if [[ $EUID -ne 0 ]]; then
        error "Cần quyền root. Chạy: sudo bash scripts/setup-ai-node.sh $*"
        exit 1
    fi
}

do_check() {
    step "Preflight (chỉ đọc)"
    . /etc/os-release 2>/dev/null || true
    info "OS: ${PRETTY_NAME:-unknown}"
    case "${VERSION_ID:-}" in
        22.04|24.04) : ;;
        *) warn "OS ${VERSION_ID:-?} ngoài mục tiêu §P0 (22.04/24.04). Vẫn chạy được nhưng để ý gói driver/repo." ;;
    esac

    if lspci 2>/dev/null | grep -qi nvidia; then
        info "GPU NVIDIA: $(lspci | grep -i nvidia | grep -i -m1 'vga\|3d' | sed 's/.*: //')"
    else
        warn "Không thấy GPU NVIDIA qua lspci."
    fi

    if command -v nvidia-smi &>/dev/null; then
        info "Driver: đã cài — $(nvidia-smi --query-gpu=driver_version,memory.total --format=csv,noheader 2>/dev/null | head -1)"
    else
        warn "nvidia-smi CHƯA có → cần bước 'driver' (+ reboot)."
    fi

    if lsmod 2>/dev/null | grep -q '^nouveau'; then
        warn "nouveau đang nạp → cài driver NVIDIA sẽ blacklist nouveau, cần REBOOT."
    fi

    if command -v mokutil &>/dev/null && mokutil --sb-state 2>/dev/null | grep -qi enabled; then
        warn "Secure Boot BẬT → module NVIDIA cần ký/enroll MOK (có thể hỏi mật khẩu lúc reboot)."
    fi

    command -v docker &>/dev/null && info "Docker: $(docker --version)" || warn "Docker CHƯA có → bước 'docker'."
    command -v nvidia-ctk &>/dev/null && info "nvidia-container-toolkit: có" || warn "toolkit CHƯA có → bước 'toolkit'."

    if [[ -x "$BIN_SRC" ]]; then
        info "Binary build sẵn: $BIN_SRC"
    else
        warn "Chưa thấy $BIN_SRC — build trước: (cd $REPO_ROOT && cargo build --release -p digitext-ocr)"
    fi
    [[ -f "$ENV_DST" ]] && info "Env: $ENV_DST đã có" || warn "Env $ENV_DST chưa có → bước 'deploy' tạo từ mẫu."
    echo
}

do_driver() {
    step "NVIDIA driver"
    if command -v nvidia-smi &>/dev/null && nvidia-smi &>/dev/null; then
        info "Driver đã hoạt động — bỏ qua."
        return
    fi
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -y
    apt-get install -y ubuntu-drivers-common pciutils mokutil

    if command -v mokutil &>/dev/null && mokutil --sb-state 2>/dev/null | grep -qi enabled; then
        warn "Secure Boot BẬT: sau reboot có thể phải enroll MOK để module nạp được."
    fi

    if [[ -n "$DRIVER_PACKAGE" ]]; then
        info "Cài driver chỉ định: $DRIVER_PACKAGE"
        apt-get install -y "$DRIVER_PACKAGE"
    else
        info "Dùng ubuntu-drivers tự chọn bản khuyến nghị…"
        ubuntu-drivers install || {
            warn "ubuntu-drivers thất bại — thử bản open mới nhất trong repo."
            local cand
            cand=$(apt-cache search -n 'nvidia-driver-[0-9]*-open' | awk '{print $1}' | sort -V | tail -1)
            [[ -n "$cand" ]] && apt-get install -y "$cand" || { error "Không tìm được gói driver. Đặt DRIVER_PACKAGE=…"; return 1; }
        }
    fi
    warn "⚠ DRIVER đã cài — PHẢI REBOOT để nạp module: sudo reboot"
    warn "  Sau reboot kiểm tra: nvidia-smi"
    NEED_REBOOT=1
}

do_docker() {
    step "Docker CE"
    if command -v docker &>/dev/null && docker compose version &>/dev/null; then
        info "Docker + compose plugin đã có — bỏ qua cài."
    else
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -y
        apt-get install -y ca-certificates curl gnupg
        install -m 0755 -d /etc/apt/keyrings
        if [[ ! -f /etc/apt/keyrings/docker.gpg ]]; then
            curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
            chmod a+r /etc/apt/keyrings/docker.gpg
        fi
        . /etc/os-release
        local codename="${VERSION_CODENAME:-noble}"
        if ! curl -fsI "https://download.docker.com/linux/ubuntu/dists/${codename}/Release" >/dev/null 2>&1; then
            warn "Kho Docker chưa có dist '${codename}' → dùng 'noble' (24.04 LTS)."
            codename="noble"
        fi
        echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu ${codename} stable" \
            > /etc/apt/sources.list.d/docker.list
        apt-get update -y
        apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
    fi
    systemctl enable --now docker
    local owner
    owner="$(stat -c '%U' "$REPO_ROOT" 2>/dev/null || echo '')"
    if [[ -n "$owner" && "$owner" != "root" ]]; then
        usermod -aG docker "$owner" 2>/dev/null && info "Đã thêm '$owner' vào group docker (đăng nhập lại để có hiệu lực)."
    fi
    info "Docker: $(docker --version)"
}

do_toolkit() {
    step "nvidia-container-toolkit"
    if command -v nvidia-ctk &>/dev/null; then
        info "nvidia-container-toolkit đã có — cấu hình lại runtime cho chắc."
    else
        export DEBIAN_FRONTEND=noninteractive
        local kr=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg
        curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | gpg --dearmor -o "$kr"
        curl -fsSL https://nvidia.github.io/libnvidia-container/stable/deb/nvidia-container-toolkit.list \
            | sed "s#deb https://#deb [signed-by=${kr}] https://#g" \
            > /etc/apt/sources.list.d/nvidia-container-toolkit.list
        apt-get update -y
        apt-get install -y nvidia-container-toolkit
    fi
    nvidia-ctk runtime configure --runtime=docker
    systemctl restart docker
    info "Đã cấu hình Docker dùng nvidia runtime."
}

do_deploy() {
    step "Triển khai binary + env digitext-ocr"
    if [[ ! -x "$BIN_SRC" ]]; then
        error "Chưa có binary $BIN_SRC. Build trước: (cd $REPO_ROOT && cargo build --release -p digitext-ocr)"
        return 1
    fi
    getent group digitext >/dev/null || groupadd --system digitext
    id -u digitext &>/dev/null || useradd --system --no-create-home --shell /usr/sbin/nologin -g digitext digitext

    install -o root -g root -m 0755 "$BIN_SRC" "$BIN_DST"
    info "Binary → $BIN_DST ($($BIN_DST --version))"

    install -d -o digitext -g digitext -m 0750 "$DATA_DIR" "$PAGES_DIR" "$PDF_DIR"
    install -d -o root -g digitext -m 0750 /etc/digitext

    if [[ ! -f "$ENV_DST" ]]; then
        install -o root -g digitext -m 0640 "$ENV_EXAMPLE" "$ENV_DST"
        warn "Đã tạo $ENV_DST từ mẫu — EDIT: điền DIGITEXT_DATABASE_URL (DB server khác)."
    else
        info "$ENV_DST đã tồn tại — giữ nguyên (không đè cấu hình của bạn)."
    fi
    [[ -f "$COMMON_ENV" ]] || { : > "$COMMON_ENV"; chown root:digitext "$COMMON_ENV"; chmod 0640 "$COMMON_ENV"; }
}

do_systemd() {
    step "systemd digitext@ocr"
    [[ -f "$UNIT_SRC" ]] || { error "Không thấy unit template $UNIT_SRC"; return 1; }
    install -m 0644 "$UNIT_SRC" /etc/systemd/system/digitext@.service
    systemctl daemon-reload
    systemctl enable digitext@ocr
    systemctl restart digitext@ocr || true
    sleep 1
    if systemctl is-active --quiet digitext@ocr; then
        info "digitext@ocr: active (chế độ chờ). Health:"
        curl -fsS http://127.0.0.1:9090/healthz 2>/dev/null && echo || warn "healthz chưa phản hồi (đợi vài giây rồi thử lại)."
    else
        warn "digitext@ocr chưa active — xem: journalctl -u digitext@ocr -n 30 --no-pager"
    fi
}

do_firewall() {
    step "Tường lửa UFW (§P8 deny-incoming)"
    if ! command -v ufw &>/dev/null; then
        export DEBIAN_FRONTEND=noninteractive; apt-get install -y ufw
    fi
    ufw --force default deny incoming
    ufw --force default allow outgoing
    ufw allow OpenSSH || true
    ufw --force enable
    info "UFW bật: deny-incoming, allow OpenSSH. Triton(8000/8001)+worker(9090) chỉ loopback."
    ufw status verbose | head -12 || true
}

do_verify() {
    step "Kiểm chứng"
    if command -v nvidia-smi &>/dev/null && nvidia-smi &>/dev/null; then
        nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader || true
        if command -v docker &>/dev/null && command -v nvidia-ctk &>/dev/null; then
            info "Thử container thấy GPU…"
            if docker run --rm --gpus all nvidia/cuda:12.4.1-base-ubuntu22.04 nvidia-smi -L 2>/dev/null; then
                info "✔ Container nhìn thấy GPU — sẵn sàng chạy Triton."
            else
                warn "Container chưa thấy GPU (có thể cần reboot sau khi cài driver, hoặc kéo image lần đầu)."
            fi
        fi
    else
        warn "nvidia-smi chưa chạy → chạy bước 'driver' rồi REBOOT trước khi verify."
    fi
}

NEED_REBOOT=0
main() {
    local args=("$@")
    [[ ${#args[@]} -eq 0 ]] && args=(all)

    if [[ " ${args[*]} " == *" check "* && ${#args[@]} -eq 1 ]]; then
        do_check; exit 0
    fi
    require_root "${args[@]}"
    do_check

    for a in "${args[@]}"; do
        case "$a" in
            all)      do_docker; do_toolkit; do_deploy; do_systemd; do_firewall; do_verify ;;
            driver)   do_driver ;;
            docker)   do_docker ;;
            toolkit)  do_toolkit ;;
            deploy)   do_deploy ;;
            systemd)  do_systemd ;;
            firewall) do_firewall ;;
            verify)   do_verify ;;
            check)    : ;;
            *) error "Bước không rõ: '$a' (all|driver|docker|toolkit|deploy|systemd|firewall|verify|check)"; exit 2 ;;
        esac
    done

    step "Xong"
    if [[ "$NEED_REBOOT" == "1" ]]; then
        warn "⚠ CẦN REBOOT để nạp driver NVIDIA:  sudo reboot"
        warn "  Sau reboot chạy:  sudo bash scripts/setup-ai-node.sh verify"
    fi
    cat <<NEXT

Bước tiếp theo:
  1) Điền DB server khác vào /etc/digitext/ocr.env → DIGITEXT_DATABASE_URL=postgres://…@<db-host>:5432/digitext
  2) Khởi Triton (khi đã có engine model):  cd infra/compose/triton && cp .env.example .env && docker compose up -d
  3) Bật worker khi Triton READY + DB tới được:
        sudoedit /etc/digitext/ocr.env   # DIGITEXT_WORKER_ENABLED=true
        sudo systemctl restart digitext@ocr
  4) Theo dõi:  journalctl -u digitext@ocr -f   |   curl 127.0.0.1:9090/readyz
NEXT
}
main "$@"
