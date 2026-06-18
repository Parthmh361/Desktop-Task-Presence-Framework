# Desktop Task Presence Framework (DTPF)

DTPF bridges web applications and the native desktop. Any web app can create, manage, and sync native sticky notes and always-on-top task overlays through a lightweight local desktop agent.

## Quick Start

### 1. Install the SDK

```bash
npm install @dtpf/sdk-react
# or
pnpm add @dtpf/sdk-react
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

Download the DTPF Agent for your platform, or build from source:

```bash
pnpm install
pnpm --filter sticky-note-ui build
cd apps/desktop-agent && pnpm tauri dev
```

On first connect, approve your web app in the agent dialog.

## How It Works

```
Web App (SDK) ── REST + WebSocket ──▶ Desktop Agent (Tauri)
                                           ├── SQLite persistence
                                           ├── Native sticky windows
                                           └── System tray
```

- **REST** `127.0.0.1:7842` — task CRUD
- **WebSocket** `ws://127.0.0.1:7842/ws` — real-time events
- **Discovery** — probes ports 7842–7844

## Monorepo Structure

| Path | Description |
|------|-------------|
| `apps/desktop-agent` | Tauri v2 desktop agent |
| `packages/sdk-core` | Framework-agnostic TypeScript SDK |
| `packages/sdk-react` | React hooks + context |
| `packages/shared-types` | Shared TypeScript types |
| `examples/react-basic` | Demo integration |

## Development

```bash
pnpm install
pnpm build                    # Build all packages
pnpm --filter react-basic dev # Run demo web app
pnpm --filter sticky-note-ui build
cd apps/desktop-agent && pnpm tauri dev
```

### Linux requirements

```bash
sudo apt install libwebkit2gtk-4.1-dev librsvg2-dev patchelf libglib2.0-dev pkg-config libayatana-appindicator3-dev
```

On Ubuntu systems with Ayatana indicators, use `libayatana-appindicator3-dev` instead of `libappindicator3-dev`.

Transparency on Linux may require a compositor (e.g. Picom).

## Platform Support

| Platform | MVP Status |
|----------|------------|
| Windows  | Supported |
| Linux    | Supported |
| macOS    | Phase 2 |

## License

MIT
