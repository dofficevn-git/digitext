#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

MARKER="Copyright (c) 2026 DigiText"

missing=0
while IFS= read -r f; do
  if ! head -n 3 "$f" | grep -qF "$MARKER"; then
    echo "::error file=$f::missing license header"
    missing=$((missing + 1))
  fi
done < <(git ls-files '*.rs' '*.ts' '*.tsx' |
  grep -v '^desktop/src-tauri/gen/' |
  grep -v '^desktop/src/components/ui/')

if [ "$missing" -gt 0 ]; then
  echo "❌ $missing file(s) missing the license header — run: dev/add-license-header.sh"
  exit 1
fi
echo "✓ all source files carry the license header"
