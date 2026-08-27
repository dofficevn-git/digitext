#!/usr/bin/env bash
set -euo pipefail

TARGET_USER="${SUDO_USER:-$(id -un)}"

echo "→ apt update + cài docker.io, docker-compose-v2"
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y docker.io docker-compose-v2

echo "→ bật & khởi động docker service"
systemctl enable --now docker

echo "→ thêm '$TARGET_USER' vào group docker (chạy docker không cần sudo)"
usermod -aG docker "$TARGET_USER"

echo
echo "✅ Xong. Kiểm tra: docker --version && docker compose version"
echo "⚠  '$TARGET_USER' cần ĐĂNG XUẤT/ĐĂNG NHẬP LẠI (hoặc: newgrp docker) để dùng docker không sudo."
