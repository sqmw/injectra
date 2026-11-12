import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {copyTree} from './copy-tree.mjs';

test('internal directory links target the copy; external links reject before mutation', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'injectra-copy-'));
  try {
    const source = path.join(root, 'source'), output = path.join(root, 'copy');
    await fs.mkdir(path.join(source, 'engine', 'native'), {recursive:true});
    await fs.writeFile(path.join(source, 'engine', 'native', 'marker'), 'original');
    const kind = process.platform === 'win32' ? 'junction' : 'dir';
    await fs.symlink(path.join(source, 'engine', 'native'), path.join(source, 'native'), kind);
    assert.deepEqual(await copyTree(source, output), [{path:'native', target:path.join('engine', 'native')}]);
    await fs.writeFile(path.join(output, 'native', 'marker'), 'changed');
    assert.equal(await fs.readFile(path.join(source, 'native', 'marker'), 'utf8'), 'original');
    await fs.symlink(root, path.join(source, 'outside'), kind);
    const rejected = path.join(root, 'rejected');
    await assert.rejects(copyTree(source, rejected), /leaves copied package/);
    await assert.rejects(fs.stat(rejected), {code:'ENOENT'});
  } finally { await fs.rm(root, {recursive:true, force:true}); }
});
