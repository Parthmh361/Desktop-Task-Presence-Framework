#!/usr/bin/env node
/**
 * Regenerate icon.ico from PNG sources (fixes Windows RC2175 build error).
 */
import { writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const iconsDir = join(
  dirname(fileURLToPath(import.meta.url)),
  '..',
  'apps',
  'desktop-agent',
  'src-tauri',
  'icons',
);

const pngs = ['32x32.png', '128x128.png'].map((f) => join(iconsDir, f));

console.log('Regenerating icon.ico for Windows...');

const result = spawnSync(
  'pnpm',
  ['dlx', 'png-to-ico', ...pngs],
  { encoding: 'buffer', shell: true },
);

if (result.status !== 0) {
  console.error('Failed to generate icon.ico');
  process.stderr.write(result.stderr ?? '');
  process.exit(result.status ?? 1);
}

writeFileSync(join(iconsDir, 'icon.ico'), result.stdout);
console.log('Wrote', join(iconsDir, 'icon.ico'));
