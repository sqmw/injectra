import { resolve } from 'node:path';
import { mkdir, mkdtemp } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { cache, root, tools, requireTool } from '../scripts/env.mjs';
export { requireTool };
export async function load(name) {
  await mkdir(resolve(cache, 'tests'), { recursive: true });
  const directory = await mkdtemp(resolve(cache, 'tests/module-'));
  const file = resolve(directory, name + '.mjs');
  await requireTool('esbuild').build({ entryPoints: [resolve(root, 'src', name + '.ts')], outfile: file, bundle: true, nodePaths: [resolve(tools, 'node_modules')], format: 'esm', platform: 'node', target: 'node24', logLevel: 'silent' });
  return import(pathToFileURL(file).href);
}
export const state = () => ({ workspace: { id: 'workspace-1', root: '/workspace', revision: 'revision-1' }, files: [{ path: 'page.js', origin: 'user', stage: 'page', hash: 'hash-1' }, { path: 'startup.cjs', origin: 'user', stage: 'startup', hash: 'startup-1' }, { path: 'original.js', origin: 'application', stage: 'page', hash: 'original' }], targets: [{ id: 'page-1', title: 'Actual page', url: 'app://real/page', generation: 'generation-1', capabilities: ['web.page.evaluate', 'web.page.reload'] }], runs: [], capabilities: ['prepare', 'dev', 'restart', 'export', 'workspace.import'], provenance: { application: { original: true }, injectra: { bootstrap_version: '1' } }, startup: { status: 'saved', pending_restart: true, saved_revision: 'revision-1' } });
