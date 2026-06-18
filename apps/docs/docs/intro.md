---
sidebar_position: 1
slug: /
---

# What is DTPF?

The **Desktop Task Presence Framework (DTPF)** is an open-source, cross-platform framework that bridges web applications and the native desktop. Any web app — React, Next.js, Vue, Angular, or vanilla JS — can create, manage, and synchronize native sticky notes, task overlays, and always-on-top reminders through a locally installed desktop agent.

## Three layers

| Layer | Role |
|-------|------|
| **Frontend SDK** | Your web app issues task commands (`createStickyTask`, `updateStickyTask`, etc.) |
| **Desktop agent** | A lightweight Tauri system-tray app that executes commands and renders native windows |
| **Local communication** | REST + WebSocket over `127.0.0.1` — no cloud required |

## Why DTPF?

- **SDK-first** — add desktop stickies to a web app in minutes, no Electron knowledge required
- **Framework agnostic** — works with any frontend stack
- **Lifecycle-aware** — stickies sync with task state (created → updated → completed → removed)
- **Survives restarts** — tasks persist across browser close and OS reboot
- **Offline-first** — runs entirely on localhost; optional cloud sync is a future add-on

## Architecture

```
Web App (SDK) ── REST + WebSocket ──▶ Desktop Agent (Tauri)
                                           ├── SQLite persistence
                                           ├── Native sticky windows
                                           └── System tray
```

See [Quick Start](./quickstart) to install the agent and integrate the SDK.
