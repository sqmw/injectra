import { mkdir, copyFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { root, tools, output, requireTool } from './env.mjs';
await mkdir(output, { recursive: true });
await requireTool('esbuild').build({ entryPoints: [resolve(root, 'src/main.ts')], outfile: resolve(output, 'app.js'), bundle: true, nodePaths: [resolve(tools, 'node_modules')], format: 'esm', target: 'es2022', sourcemap: false, minify: true, legalComments: 'none' });
await copyFile(resolve(root, 'index.html'), resolve(output, 'index.html'));
console.log(output);
