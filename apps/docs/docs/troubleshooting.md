---
sidebar_position: 4
---

# Troubleshooting

Common issues when developing or integrating with DTPF.

## Port 1420 already in use

The sticky-note dev UI (Vite) runs on port **1420** during `pnpm agent:dev`. Tauri proxies this in development.

**Symptoms:** `agent:dev` fails with `EADDRINUSE` or Tauri cannot load the dev URL.

**Fix:**

1. Find what is using the port:
   ```bash
   # Linux / macOS
   lsof -i :1420

   # Windows (PowerShell)
   netstat -ano | findstr :1420
   ```
2. Stop the conflicting process, or change the port in `apps/desktop-agent/ui/sticky-note/vite.config.ts` and update `devUrl` in `tauri.conf.json` to match.

The agent **API** uses port **7842** (with fallbacks 7843, 7844) — a separate issue from the Vite dev port.

## Agent API port 7842 unavailable

The desktop agent binds REST/WebSocket to `127.0.0.1:7842` (then 7843, 7844).

**Symptoms:** SDK shows "Agent not connected"; health check fails.

**Fix:**

- Ensure the agent is running (system tray icon visible).
- Check nothing else is bound to 7842–7844.
- Inspect the lock file for the active port:
  - Windows: `%LOCALAPPDATA%\dtpf\agent.lock`
  - Linux/macOS: `~/.dtpf/agent.lock`

## `cargo` not found / not in PATH

**Symptoms:** `pnpm agent:dev` or `cargo test` fails with `cargo: command not found`.

**Fix:**

1. Install Rust: https://rustup.rs
2. Restart your terminal (or IDE) so PATH includes `~/.cargo/bin` (Linux/macOS) or `%USERPROFILE%\.cargo\bin` (Windows).
3. Verify: `cargo --version`

On Windows, also confirm **Desktop development with C++** is installed in Visual Studio Build Tools.

## WebView2 not found (Windows)

Tauri on Windows requires the **Microsoft Edge WebView2 Runtime**.

**Symptoms:** Agent fails to start; build errors referencing WebView2.

**Fix:**

1. Download and install the [WebView2 Evergreen Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).
2. On Windows 10/11, WebView2 is usually pre-installed — if missing, the bootstrapper installs it.
3. For CI, verify the registry key exists (see `.github/workflows/ci.yml`).

## SDK cannot connect / auth rejected

**Symptoms:** Banner shows "DTPF Agent not connected" or auth dialog never appears.

**Fix:**

1. Confirm the agent is running.
2. Use a stable `appId` in `DTPFProvider` config (e.g. `com.mycompany.myapp`).
3. Approve the registration dialog on first connect.
4. Check browser console and agent logs for CORS or token errors.

## Linux: sticky transparency not working

Requires a compositing window manager. Install and enable Picom (or use GNOME/KDE defaults).

## macOS: gatekeeper blocks unsigned builds

Locally built `.app` bundles may be quarantined. Right-click → Open, or remove quarantine:

```bash
xattr -cr /path/to/DTPF\ Agent.app
```

For production distribution, use signed releases from [GitHub Releases](https://github.com/Parthmh361/Desktop-Task-Presence-Framework/releases).

## macOS: sticky notes are opaque

Sticky windows on macOS render without transparency (opaque background). Linux and Windows support transparent sticky backgrounds. Full macOS transparency requires `macos-private-api` and is planned for a future release.

## Preferences UI not yet available

The tray **Preferences** menu item shows a placeholder dialog. Agent settings (auto-start, data directory) are not configurable from the UI yet — planned for v1.1.

## Auto-updater fails / "Failed to check for updates"

The updater fetches `latest.json` from GitHub Releases. This file only exists after a successful tagged release with signed Tauri builds. Install the latest release manually first; subsequent updates work automatically once `latest.json` is published.

## End-to-end tests in CI

Playwright e2e tests (`pnpm test:e2e`) require a running agent and are not wired into CI yet. Run them locally after starting `pnpm agent:dev`.
