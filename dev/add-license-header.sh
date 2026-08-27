#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

MARKER="Copyright (c) 2026 DigiText"
HEADER="// Copyright (c) 2026 DigiText. All rights reserved."

added=0
while IFS= read -r f; do
  if head -n 3 "$f" | grep -qF "$MARKER"; then
    continue
  fi
  { echo "$HEADER"; cat "$f"; } >"$f.tmp" && mv "$f.tmp" "$f"
  added=$((added + 1))
done < <(git ls-files '*.rs' '*.ts' '*.tsx' |
  grep -v '^desktop/src-tauri/gen/' |
  grep -v '^desktop/src/components/ui/')

echo "added license header to $added file(s)"
