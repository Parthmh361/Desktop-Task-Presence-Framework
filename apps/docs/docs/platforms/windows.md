---
sidebar_position: 1
---

# Windows

Prerequisites for building and running the DTPF desktop agent on Windows.

## Required

| Tool | Notes |
|------|-------|
| **Node.js 20+** | LTS recommended; enable pnpm via `corepack enable` |
| **pnpm 10+** | Monorepo package manager |
| **Rust stable** | Install from [rustup.rs](https://rustup.rs) |
| **Visual Studio Build Tools** | **Desktop development with C++** workload (MSVC linker) |
| **WebView2 Runtime** | Pre-installed on most Windows 10/11 systems; required by Tauri |

## One-shot setup

```powershell
pnpm install-deps
# or
.\scripts\install-deps.ps1
```

## Run the agent

```powershell
pnpm agent:dev
```

## Data locations

Agent state is stored under:

```
%LOCALAPPDATA%\dtpf\
```

Includes the SQLite database, auth secrets, and agent lock file (port discovery).

## Build a release installer

```powershell
pnpm install
pnpm --filter sticky-note-ui build
pnpm --filter desktop-agent build
```

Outputs `.msi` and `.exe` installers under `apps/desktop-agent/src-tauri/target/release/bundle/`.
