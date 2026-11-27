import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {appendRewrite} from './append-asar.mjs';
import {builtinAsar as asar,encodePickleHeader} from '../../crates/preparation/helper/archive.mjs';

test('append rewrite retains opaque and missing unpacked records, with existing payload unchanged',async()=>{
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'injectra-append-')),source=path.join(root,'before.asar'),destination=path.join(root,'after.asar');
 const header={files:{'main.js':{size:3,offset:'0'},virtual:{size:-1000,offset:'0'},missing:{size:10,unpacked:true}}};
 fs.writeFileSync(source,Buffer.concat([encodePickleHeader(JSON.stringify(header)),Buffer.from('abc')]));
 await appendRewrite({source,destination,replacements:new Map([['main.js',Buffer.from('updated')],['.injectra/test.cjs',Buffer.from('owned')]])});
 const raw=asar.getRawHeader(destination);
 assert.deepEqual(raw.header.files.virtual,header.files.virtual);
 assert.deepEqual(raw.header.files.missing,header.files.missing);
 assert.equal(asar.extractFile(destination,'main.js').toString(),'updated');
 assert.equal(asar.extractFile(destination,path.normalize('.injectra/test.cjs')).toString(),'owned');
 assert.equal(fs.readFileSync(destination).subarray(8+raw.headerSize,8+raw.headerSize+3).toString(),'abc');
 assert.equal(asar.extractFile(source,'main.js').toString(),'abc');
});
