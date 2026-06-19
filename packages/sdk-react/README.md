# @dtpf/sdk-react

React hooks and context for the [Desktop Task Presence Framework (DTPF)](https://github.com/Parthmh361/Desktop-Task-Presence-Framework).

## Install

```bash
npm install @dtpf/sdk-react
```

Requires the [DTPF Agent](https://github.com/Parthmh361/Desktop-Task-Presence-Framework/releases) running locally.

## Usage

```tsx
import { DTPFProvider, useStickyTask } from '@dtpf/sdk-react';

function TaskButton() {
  const { createTask } = useStickyTask();
  return (
    <button onClick={() => createTask({ title: 'Fix login bug' })}>
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

## Documentation

- [Quick start](https://parthmh361.github.io/Desktop-Task-Presence-Framework/quickstart)
- [Full docs](https://github.com/Parthmh361/Desktop-Task-Presence-Framework/tree/main/apps/docs)
