#!/usr/bin/env node
/**
 * Cross-platform entry point for installing all DTPF dev dependencies.
 * Windows: runs install-deps.ps1 (installs Rust, checks MSVC, builds packages)
 * Linux/macOS: runs install-deps.sh
 */
import { spawnSync } from 'node:child_process';
import { platform } from 'node:os';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const os = platform();

if (os === 'win32') {
  const ps1 = join(scriptsDir, 'install-deps.ps1');
  const result = spawnSync(
    'powershell',
    ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', ps1],
    { stdio: 'inherit' },
  );
  process.exit(result.status ?? 1);
}

const sh = join(scriptsDir, 'install-deps.sh');
const result = spawnSync('bash', [sh], { stdio: 'inherit' });
process.exit(result.status ?? 1);
