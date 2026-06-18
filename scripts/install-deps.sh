#!/usr/bin/env bash
# Installs DTPF development dependencies on Linux/macOS.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "DTPF dependency installer ($(uname -s))"

if ! command -v node >/dev/null 2>&1; then
  echo "Node.js not found. Install Node.js 20+ first."
  exit 1
fi

if ! command -v pnpm >/dev/null 2>&1; then
  echo "Enabling pnpm via corepack..."
  corepack enable
  corepack prepare pnpm@10.20.0 --activate
fi

if [[ "$(uname -s)" == "Linux" ]]; then
  echo ""
  echo "Installing Linux Tauri system dependencies (requires sudo)..."
  if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update
    sudo apt-get install -y \
      libwebkit2gtk-4.1-dev \
      libappindicator3-dev \
      librsvg2-dev \
      patchelf \
      libglib2.0-dev \
      pkg-config \
      build-essential \
      curl
  else
    echo "Non-Debian Linux: install WebKitGTK and appindicator dev packages manually."
  fi

  if ! command -v cargo >/dev/null 2>&1; then
    echo "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  fi
elif [[ "$(uname -s)" == "Darwin" ]]; then
  if ! command -v cargo >/dev/null 2>&1; then
    echo "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  fi
  if ! xcode-select -p >/dev/null 2>&1; then
    echo "Install Xcode Command Line Tools: xcode-select --install"
  fi
fi

node scripts/setup.mjs

echo ""
echo "Done. Run: pnpm demo:dev  and  pnpm agent:dev"
