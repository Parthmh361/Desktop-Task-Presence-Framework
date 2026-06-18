#!/usr/bin/env node
/**
 * Start the Tauri desktop agent with Cargo on PATH (Windows often misses
 * ~/.cargo/bin until the terminal is restarted after rustup install).
 */
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const agentDir = join(root, 'apps', 'desktop-agent');
const cargoBin = join(homedir(), '.cargo', 'bin');
const cargoExe = join(cargoBin, process.platform === 'win32' ? 'cargo.exe' : 'cargo');

const pathSep = process.platform === 'win32' ? ';' : ':';
const env = { ...process.env };

if (!existsSync(cargoExe)) {
  console.error('');
  console.error('Rust (cargo) not found.');
  console.error('');
  console.error('Install it with:  pnpm install-deps');
  console.error('Or download from: https://rustup.rs');
  console.error('');
  process.exit(1);
}

if (!env.PATH?.split(pathSep).some((p) => p.toLowerCase() === cargoBin.toLowerCase())) {
  env.PATH = `${cargoBin}${pathSep}${env.PATH ?? ''}`;
}

function findTauriBin() {
  const candidates =
    process.platform === 'win32'
      ? [
          join(agentDir, 'node_modules', '.bin', 'tauri.cmd'),
          join(agentDir, 'node_modules', '.bin', 'tauri.CMD'),
          join(root, 'node_modules', '.bin', 'tauri.cmd'),
          join(root, 'node_modules', '.bin', 'tauri.CMD'),
        ]
      : [
          join(agentDir, 'node_modules', '.bin', 'tauri'),
          join(root, 'node_modules', '.bin', 'tauri'),
        ];

  return candidates.find((p) => existsSync(p)) ?? 'tauri';
}

const tauriCmd = findTauriBin();

const result = spawnSync(tauriCmd, ['dev'], {
  cwd: agentDir,
  stdio: 'inherit',
  env,
  shell: process.platform === 'win32',
});

process.exit(result.status ?? 1);
