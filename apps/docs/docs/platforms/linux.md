---
sidebar_position: 2
---

# Linux

Prerequisites for building and running the DTPF desktop agent on Linux.

## Required packages (Debian / Ubuntu)

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  librsvg2-dev \
  patchelf \
  libglib2.0-dev \
  pkg-config \
  libayatana-appindicator3-dev
```

On older Ubuntu systems without Ayatana, use `libappindicator3-dev` instead of `libayatana-appindicator3-dev`.

## Rust and Node

| Tool | Notes |
|------|-------|
| **Node.js 20+** | Install via nvm, fnm, or your distro |
| **pnpm 10+** | `corepack enable && corepack prepare pnpm@latest --activate` |
| **Rust stable** | [rustup.rs](https://rustup.rs) |

## One-shot setup

```bash
pnpm install-deps
# or
bash scripts/install-deps.sh
```

## Run the agent

```bash
pnpm agent:dev:unix
```

## Compositor note

Sticky note transparency on Linux may require a compositing window manager (e.g. Picom, Mutter, KWin).

## Data locations

```
~/.dtpf/
```

## Build a release bundle

```bash
pnpm install
pnpm --filter sticky-note-ui build
pnpm --filter desktop-agent build
```

Outputs `.AppImage` and `.deb` under `apps/desktop-agent/src-tauri/target/release/bundle/`.
