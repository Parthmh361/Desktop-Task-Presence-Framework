#!/usr/bin/env bash
set -euo pipefail

echo "Setting up DTPF development environment..."

# Prefer cross-platform Node setup when available
if command -v node >/dev/null 2>&1; then
  node "$(dirname "$0")/setup.mjs"
  exit 0
fi

# Fallback without Node
pnpm install
pnpm --filter @dtpf/shared-types build
pnpm --filter @dtpf/sdk-core build
pnpm --filter @dtpf/sdk-react build
pnpm --filter sticky-note-ui build

if command -v cargo >/dev/null 2>&1; then
  if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "On Linux, install Tauri deps if not already present:"
    echo "  sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf"
  fi
  (cd apps/desktop-agent/src-tauri && cargo check) || echo "Rust check skipped (install system deps)"
fi

echo "Done. Run 'pnpm demo:dev' and 'pnpm agent:dev'"
