# Desktop Task Presence Framework (DTPF)

DTPF bridges web applications and the native desktop. Any web app can create, manage, and sync native sticky notes and always-on-top task overlays through a lightweight local desktop agent.

## Quick Start

### 1. Install the SDK

```bash
npm install @dtpf/sdk-react
# or framework-agnostic:
npm install @dtpf/sdk-vanilla
```

### 2. Wrap your app

```tsx
import { DTPFProvider, useStickyTask } from '@dtpf/sdk-react';

function TaskButton() {
  const { createTask } = useStickyTask();

  return (
    <button
      onClick={() =>
        createTask({ title: 'Fix login bug', body: 'Check OAuth redirect' })
      }
    >
      Create Sticky
    </button>
  );
}

export default function App() {
  return (
    <DTPFProvider config={{ appId: 'com.myapp', appName: 'My App' }}>
      <TaskButton />
    </DTPFProvider>
  );
}
```

### 3. Run the desktop agent

Download a release binary (see [Releases](https://github.com/Parthmh361/Desktop-Task-Presence-Framework/releases)) or build from source:

```bash
pnpm install-deps
pnpm agent:dev
```

On first connect, approve your web app in the agent dialog.

## How It Works

```
Web App (SDK) ── REST + WebSocket ──▶ Desktop Agent (Tauri)
                                           ├── SQLite persistence
                                           ├── Native sticky windows
                                           └── System tray
```

- **REST** `127.0.0.1:7842–7844` — task CRUD (agent tries fallback ports)
- **WebSocket** `ws://127.0.0.1:<port>/ws` — real-time events
- **Discovery** — SDK probes ports 7842–7844; agent writes `%LOCALAPPDATA%\dtpf\agent.lock` (Windows) or `~/.dtpf/agent.lock` (Linux)

## Monorepo Structure

| Path | Description |
|------|-------------|
| `apps/desktop-agent` | Tauri v2 desktop agent |
| `apps/docs` | Docusaurus documentation site |
| `packages/sdk-core` | Framework-agnostic TypeScript SDK |
| `packages/sdk-react` | React hooks + context |
| `packages/sdk-vanilla` | Zero-dependency re-export of sdk-core |
| `packages/shared-types` | Shared TypeScript types |
| `examples/react-basic` | React demo |
| `examples/vanilla-js` | Vanilla JS demo |
| `examples/nextjs-tasks` | Next.js App Router demo |

## Development

Use **two terminals**:

```bash
pnpm install-deps
pnpm demo:dev       # Terminal 1 — web demo (http://localhost:5173)
pnpm agent:dev      # Terminal 2 — desktop agent
```

Other demos:

```bash
pnpm demo:vanilla   # http://localhost:5174
pnpm demo:nextjs    # http://localhost:3000
```

If port 1420 is stuck after a crash:

```powershell
pnpm agent:kill
```

Build and test:

```bash
pnpm build
pnpm test
pnpm test:e2e       # Playwright (skips if agent not running)
```

Documentation site:

```bash
pnpm --filter @dtpf/docs start
```

### Windows requirements

1. **Node.js 20+** and **pnpm** (`corepack enable`)
2. **Rust** from [rustup.rs](https://rustup.rs)
3. **Visual Studio Build Tools** — Desktop development with C++
4. **WebView2** (pre-installed on most Windows 10/11)

```powershell
pnpm install-deps
# or: .\scripts\install-deps.ps1
pnpm agent:dev
```

Agent data: `%LOCALAPPDATA%\dtpf\`

### Linux requirements

```bash
sudo apt install libwebkit2gtk-4.1-dev librsvg2-dev patchelf libglib2.0-dev pkg-config libayatana-appindicator3-dev
pnpm install-deps
pnpm agent:dev:unix
```

Agent data: `~/.dtpf/`

### macOS requirements

- Xcode Command Line Tools
- `pnpm install-deps` then `pnpm agent:dev:unix`

Agent data: `~/Library/Application Support/dtpf/`

### Logging

Set `RUST_LOG=debug` for verbose agent logs.

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows  | Supported | WebView2 + MSVC build tools |
| Linux    | Supported | WebKitGTK + compositor for transparency |
| macOS    | Supported | LaunchAgent auto-start |

## Publishing

npm packages (`@dtpf/shared-types`, `@dtpf/sdk-core`, `@dtpf/sdk-react`, `@dtpf/sdk-vanilla`) use [Changesets](https://github.com/changesets/changesets). Tag `v*.*.*` triggers `.github/workflows/release.yml` for signed agent binaries.

- [RELEASE.md](RELEASE.md) — maintainer release setup
- [PUBLISH_READINESS.md](PUBLISH_READINESS.md) — shipped status and hygiene checklist
- [ARCHITECTURE.md](ARCHITECTURE.md) — full technical architecture spec

Documentation: https://parthmh361.github.io/Desktop-Task-Presence-Framework/

## Security

See [SECURITY.md](SECURITY.md). The agent binds to localhost only and requires user approval for new web apps.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT — see [LICENSE](LICENSE).
