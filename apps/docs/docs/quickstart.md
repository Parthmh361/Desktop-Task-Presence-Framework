---
sidebar_position: 2
---

# Quick Start

Get a sticky note on your desktop from a web app in three steps.

## 1. Install the desktop agent

Download the DTPF Agent for your platform from [GitHub Releases](https://github.com/dtpf/desktop-task-presence-framework/releases), or build from source:

```bash
git clone https://github.com/dtpf/desktop-task-presence-framework.git
cd desktop-task-presence-framework
pnpm install-deps   # Node packages + Rust toolchain
pnpm agent:dev      # Windows
# pnpm agent:dev:unix  # Linux / macOS
```

Start the agent and leave it running in the system tray. On first connect from a web app, approve the registration dialog.

Platform-specific prerequisites: [Windows](./platforms/windows), [Linux](./platforms/linux), [macOS](./platforms/macos).

## 2. Install the React SDK

In your web application:

```bash
npm install @dtpf/sdk-react
# or
pnpm add @dtpf/sdk-react
```

## 3. Wrap your app and create a sticky

```tsx
import { DTPFProvider, useStickyTask } from '@dtpf/sdk-react';

function TaskButton() {
  const { createTask, loading } = useStickyTask();

  return (
    <button
      disabled={loading}
      onClick={() =>
        createTask({
          title: 'Fix login bug',
          body: 'Check OAuth redirect on Safari',
          priority: 2,
          color: '#FFE066',
        })
      }
    >
      Create Sticky
    </button>
  );
}

export default function App() {
  return (
    <DTPFProvider config={{ appId: 'com.myapp.tasks', appName: 'My App' }}>
      <TaskButton />
    </DTPFProvider>
  );
}
```

`createTask` calls `createStickyTask` on the underlying `DTPFClient`. When the agent is connected, a native sticky note appears on your desktop immediately.

## Connection status

Use `useAgentStatus` and `useDTPFContext` to show connection state:

```tsx
import { useAgentStatus, useDTPFContext } from '@dtpf/sdk-react';

function StatusBar() {
  const status = useAgentStatus();
  const { isConnected, isConnecting } = useDTPFContext();

  if (isConnecting) return <p>Connecting to agent…</p>;
  if (!isConnected) return <p>Agent not running — install and start DTPF Agent.</p>;
  return <p>Agent v{status?.version} · {status?.taskCount} stickies</p>;
}
```

## Next steps

- Full demo: [`examples/react-basic`](https://github.com/dtpf/desktop-task-presence-framework/tree/main/examples/react-basic)
- Troubleshooting: [common issues](./troubleshooting)
