---
sidebar_position: 3
---

# macOS

Prerequisites for building and running the DTPF desktop agent on macOS.

## Required

| Tool | Notes |
|------|-------|
| **Node.js 20+** | Install via nvm, fnm, or Homebrew |
| **pnpm 10+** | `corepack enable` |
| **Rust stable** | [rustup.rs](https://rustup.rs) |
| **Xcode Command Line Tools** | `xcode-select --install` |

Tauri on macOS uses the system WebKit (WKWebView). No separate WebView runtime install is needed.

## One-shot setup

```bash
pnpm install-deps
# or
bash scripts/install-deps.sh
```

If `cargo check` fails with linker errors, ensure Xcode Command Line Tools are installed and selected:

```bash
xcode-select -p
```

## Run the agent

```bash
pnpm agent:dev:unix
```

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

Outputs `.dmg` and `.app.tar.gz` under `apps/desktop-agent/src-tauri/target/release/bundle/`.

## Code signing (optional)

Release builds can be signed and notarized for distribution outside the App Store. Configure `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, and `APPLE_SIGNING_IDENTITY` in CI — see the release workflow comments for placeholder secrets.
