import { mkdir, copyFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import { root, tools } from './env.mjs';
await mkdir(tools, { recursive: true });
for (const name of ['package.json', 'package-lock.json']) await copyFile(resolve(root, name), resolve(tools, name));
const result = spawnSync('npm', ['ci', '--prefix', tools, '--no-audit', '--no-fund'], { stdio: 'inherit' });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
