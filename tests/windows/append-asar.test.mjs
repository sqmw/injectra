import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';import os from 'node:os';import path from 'node:path';import {createRequire}from'node:module';import{appendRewrite}from'./append-asar.mjs';
const tools=createRequire(path.join(process.env.INJECTRA_WIN_TEST_TOOLS??path.join(os.homedir(),'.cache/injectra/windows-test-tools'),'package.json'));const asar=tools('@electron/asar'),pickle=tools('chromium-pickle-js');
test('append rewrite retains opaque and missing unpacked records, with existing payload unchanged',async()=>{
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'injectra-append-')),source=path.join(root,'before.asar'),destination=path.join(root,'after.asar');
 const header={files:{'main.js':{size:3,offset:'0'},virtual:{size:-1000,offset:'0'},missing:{size:10,unpacked:true}}};
 const h=pickle.createEmpty();h.writeString(JSON.stringify(header));const s=pickle.createEmpty();s.writeUInt32(h.toBuffer().length);fs.writeFileSync(source,Buffer.concat([s.toBuffer(),h.toBuffer(),Buffer.from('abc')]));
 await appendRewrite({asar,pickle,source,destination,replacements:new Map([['main.js',Buffer.from('updated')],['.injectra/test.cjs',Buffer.from('owned')]])});
 const raw=asar.getRawHeader(destination);assert.deepEqual(raw.header.files.virtual,header.files.virtual);assert.deepEqual(raw.header.files.missing,header.files.missing);
 assert.equal(asar.extractFile(destination,'main.js').toString(),'updated');assert.equal(asar.extractFile(destination,path.normalize('.injectra/test.cjs')).toString(),'owned');assert.equal(fs.readFileSync(destination).subarray(8+raw.headerSize,8+raw.headerSize+3).toString(),'abc');
 assert.equal(asar.extractFile(source,'main.js').toString(),'abc');
});
