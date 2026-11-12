import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { homedir } from 'node:os';
import { createRequire } from 'node:module';
export const root = fileURLToPath(new URL('../', import.meta.url));
export const cache = resolve(process.env.INJECTRA_CACHE_DIR ?? resolve(process.env.XDG_CACHE_HOME ?? resolve(homedir(), '.cache'), 'injectra'), 'studio-ui');
export const tools = resolve(cache, 'tools');
export const output = process.env.INJECTRA_STUDIO_ASSETS ?? resolve(cache, 'dist');
export const requireTool = createRequire(resolve(tools, 'package.json'));
