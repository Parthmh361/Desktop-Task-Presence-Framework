#!/usr/bin/env bash
set -euo pipefail

echo "Setting up DTPF development environment..."

# Node dependencies
pnpm install

# Build SDK packages
pnpm --filter @dtpf/shared-types build
pnpm --filter @dtpf/sdk-core build
pnpm --filter @dtpf/sdk-react build

# Build sticky note UI
pnpm --filter sticky-note-ui build

# Rust (optional — requires system deps on Linux)
if command -v cargo >/dev/null 2>&1; then
  if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "On Linux, install Tauri deps if not already present:"
    echo "  sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf"
  fi
  (cd apps/desktop-agent/src-tauri && cargo check) || echo "Rust check skipped (install system deps)"
fi

echo "Done. Run 'pnpm --filter react-basic dev' and 'cd apps/desktop-agent && pnpm tauri dev'"
