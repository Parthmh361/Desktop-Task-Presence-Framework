#!/usr/bin/env node
/**
 * Cross-platform DTPF dev environment setup (Windows, Linux, macOS).
 */
import { spawnSync } from 'node:child_process';
import { platform } from 'node:os';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: root,
    stdio: 'inherit',
    shell: platform() === 'win32',
    ...options,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function pnpm(args) {
  run('pnpm', args);
}

console.log('Setting up DTPF development environment...\n');

pnpm(['install']);
pnpm(['--filter', '@dtpf/shared-types', 'build']);
pnpm(['--filter', '@dtpf/sdk-core', 'build']);
pnpm(['--filter', '@dtpf/sdk-react', 'build']);
pnpm(['--filter', 'sticky-note-ui', 'build']);

const os = platform();
if (os === 'linux') {
  console.log('\nLinux: install Tauri system deps if cargo check fails:');
  console.log(
    '  sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf',
  );
} else if (os === 'win32') {
  console.log('\nWindows requirements:');
  console.log('  - Node.js 20+ and pnpm (corepack enable && corepack prepare pnpm@latest --activate)');
  console.log('  - Rust stable (https://rustup.rs)');
  console.log('  - Visual Studio Build Tools with C++ workload');
  console.log('  - WebView2 Runtime (included on Windows 10/11; https://developer.microsoft.com/microsoft-edge/webview2/)');
} else if (os === 'darwin') {
  console.log('\nmacOS: install Xcode Command Line Tools if cargo check fails.');
}

const cargo = spawnSync('cargo', ['--version'], { stdio: 'pipe', shell: os === 'win32' });
if (cargo.status === 0) {
  const check = spawnSync('cargo', ['check'], {
    cwd: join(root, 'apps/desktop-agent/src-tauri'),
    stdio: 'inherit',
    shell: os === 'win32',
  });
  if (check.status !== 0) {
    console.log('\nRust check skipped (install platform-specific Tauri dependencies).');
  }
} else {
  console.log('\nRust not found — install from https://rustup.rs to build the desktop agent.');
}

console.log('\nDone.');
console.log('  Demo web app:  pnpm demo:dev');
console.log('  Desktop agent: pnpm agent:dev');
