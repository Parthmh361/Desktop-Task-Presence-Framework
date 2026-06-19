# @dtpf/sdk-core

Framework-agnostic TypeScript SDK for the [Desktop Task Presence Framework (DTPF)](https://github.com/Parthmh361/Desktop-Task-Presence-Framework).

## Install

```bash
npm install @dtpf/sdk-core
```

Requires the [DTPF Agent](https://github.com/Parthmh361/Desktop-Task-Presence-Framework/releases) running locally.

## Usage

```typescript
import { DTPFClient } from '@dtpf/sdk-core';

const client = new DTPFClient({
  appId: 'com.myapp',
  appName: 'My App',
});

await client.connect();
const task = await client.createStickyTask({ title: 'Fix login bug' });
```

## Documentation

- [Quick start](https://parthmh361.github.io/Desktop-Task-Presence-Framework/quickstart)
- [Full docs](https://github.com/Parthmh361/Desktop-Task-Presence-Framework/tree/main/apps/docs)
