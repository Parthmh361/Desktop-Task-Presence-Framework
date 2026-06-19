# @dtpf/sdk-vanilla

Vanilla JavaScript SDK for the [Desktop Task Presence Framework (DTPF)](https://github.com/Parthmh361/Desktop-Task-Presence-Framework). Re-exports `@dtpf/sdk-core` for non-React apps.

## Install

```bash
npm install @dtpf/sdk-vanilla
```

Requires the [DTPF Agent](https://github.com/Parthmh361/Desktop-Task-Presence-Framework/releases) running locally.

## Usage

```javascript
import { DTPFClient } from '@dtpf/sdk-vanilla';

const client = new DTPFClient({
  appId: 'com.myapp',
  appName: 'My App',
});

await client.connect();
await client.createStickyTask({ title: 'Review PR' });
```

## Documentation

- [Quick start](https://parthmh361.github.io/Desktop-Task-Presence-Framework/quickstart)
- [Full docs](https://github.com/Parthmh361/Desktop-Task-Presence-Framework/tree/main/apps/docs)
