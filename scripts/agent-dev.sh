#!/usr/bin/env bash
# Start the Tauri desktop agent (Linux/macOS).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
AGENT_DIR="$ROOT/apps/desktop-agent"

export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v node >/dev/null 2>&1; then
  echo "Node.js not found. Install Node.js 20+ first."
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust (cargo) not found. Run: pnpm install-deps"
  exit 1
fi

cd "$AGENT_DIR"
exec pnpm exec tauri dev
