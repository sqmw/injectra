import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile} from 'node:fs/promises';
import {tmpdir,homedir} from 'node:os';
import {resolve} from 'node:path';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {inspectMain,delegateSource,esmDelegateSource} from '../helper/entry.mjs';
import {builtinAsar as asar} from '../helper/archive.mjs';
const tools=createRequire(resolve(process.env.INJECTRA_PREPARATION_TOOLS??resolve(process.env.INJECTRA_CACHE_DIR??resolve(homedir(),'.cache/injectra'),'preparation-tools'),'package.json'));
const acorn=tools('acorn');
const root=()=>mkdtemp(resolve(tmpdir(),'injectra-entry-'));
test('delegated original main keeps identity, argv, sibling resolution and bootstrap order',async()=>{
 const base=await root();await mkdir(resolve(base,'.injectra'));await mkdir(resolve(base,'lib'));
 const main='lib/start.cjs';
 await writeFile(resolve(base,main),`globalThis.order.push('original');module.exports={marker:1};console.log(JSON.stringify({main:require.main===module,processMain:process.mainModule===module,id:module.id,parent:module.parent,argv:process.argv[1]===__filename,relative:require('./sibling.cjs'),order:globalThis.order}));`);
 await writeFile(resolve(base,'lib/sibling.cjs'),'module.exports="sibling";');
 await writeFile(resolve(base,'.injectra/bootstrap.cjs'),"globalThis.order=['bootstrap'];");
 await writeFile(resolve(base,'.injectra/entry.cjs'),delegateSource(main));
 const result=JSON.parse(execFileSync(process.execPath,[resolve(base,'.injectra/entry.cjs')],{encoding:'utf8'}));
 assert.deepEqual(result,{main:true,processMain:true,id:'.',parent:null,argv:true,relative:'sibling',order:['bootstrap','original']});
 execFileSync(process.execPath,['-e',`if(require(${JSON.stringify(resolve(base,'.injectra/entry.cjs'))}).marker!==1)throw Error('main exports lost')`]);
});
test('ESM entry selects esm-delegate, runs CJS bootstrap before original ESM and keeps import.meta.url & relative imports',async()=>{
 const base=await root();await mkdir(resolve(base,'.injectra'));await mkdir(resolve(base,'esm'));
 const main='esm/start.mjs';
 await writeFile(resolve(base,'package.json'),JSON.stringify({name:'esm-test',type:'module',main}));
 await writeFile(resolve(base,'esm/sibling.mjs'),'export default "esm-sibling";');
 await writeFile(resolve(base,main),`import sibling from './sibling.mjs';\nimport {fileURLToPath} from 'node:url';\nglobalThis.order.push('original-esm');\nconsole.log(JSON.stringify({urlEndsWithMain:import.meta.url.endsWith('/esm/start.mjs'),argvMatchesMain:process.argv[1]===fileURLToPath(import.meta.url),relative:sibling,order:globalThis.order}));\n`);
 const info=inspectMain(base,'directory',{main,type:'module'},{asar,acorn});
 assert.equal(info.strategy,'esm-delegate');
 assert.equal(info.main,main);
 await writeFile(resolve(base,'.injectra/bootstrap.cjs'),"globalThis.order=['bootstrap-cjs'];");
 await writeFile(resolve(base,'.injectra/entry.mjs'),esmDelegateSource(main));
 const result=JSON.parse(execFileSync(process.execPath,[resolve(base,'.injectra/entry.mjs')],{encoding:'utf8'}));
 assert.deepEqual(result,{urlEndsWithMain:true,argvMatchesMain:true,relative:'esm-sibling',order:['bootstrap-cjs','original-esm']});
});
test('nested archives select host delegation without rewriting opaque entry bytes',async()=>{
 const base=await root(),content=resolve(base,'content');await mkdir(resolve(content,'launcher'),{recursive:true});
 const opaque=Buffer.from([0xff,0x00,0x81,0x01]);await writeFile(resolve(content,'launcher/index.js'),opaque);
 await asar.createPackage(content,resolve(base,'runtime.asar'));
 const info=inspectMain(base,'directory',{main:'./runtime.asar/launcher/index.js'},{asar,acorn});
 assert.equal(info.strategy,'main-delegate');assert.equal(info.main,'runtime.asar/launcher/index.js');assert.deepEqual(info.bytes,opaque);assert.deepEqual(asar.extractFile(resolve(base,'runtime.asar'),'launcher/index.js'),opaque);
});
test('plain CJS retains directive insertion; bytecode (.jsc) and path escapes remain rejected',async()=>{
 const base=await root();await writeFile(resolve(base,'main.js'),"'use strict';\nmodule.exports=1;");
 await writeFile(resolve(base,'main.jsc'),Buffer.from([0xde,0xad,0xbe,0xef]));
 const info=inspectMain(base,'directory',{main:'main.js'},{asar,acorn});assert.equal(info.strategy,'cjs-insert');assert.equal(info.insert,"'use strict';".length);
 const esmPkg=inspectMain(base,'directory',{main:'main.js',type:'module'},{asar,acorn});assert.equal(esmPkg.strategy,'esm-delegate');
 assert.throws(()=>inspectMain(base,'directory',{main:'main.jsc'},{asar,acorn}),/bytecode|non-JavaScript/);
 assert.throws(()=>inspectMain(base,'directory',{main:'../main.js'},{asar,acorn}),/escapes/);
 assert.equal(await readFile(resolve(base,'main.js'),'utf8'),"'use strict';\nmodule.exports=1;");
});

test('multi-level ASAR entries use native API separators on every host',async()=>{
 const base=await root(),content=resolve(base,'content');await mkdir(resolve(content,'.vite/build'),{recursive:true});
 await writeFile(resolve(content,'.vite/build/index.js'),"'use strict';module.exports=1;");await writeFile(resolve(content,'package.json'),JSON.stringify({main:'.vite/build/index.js'}));
 const archive=resolve(base,'app.asar');await asar.createPackage(content,archive);
 const entry=inspectMain(archive,'asar',{main:'.vite/build/index.js'},{asar,acorn});assert.equal(entry.strategy,'cjs-insert');assert.equal(entry.main,'.vite/build/index.js');
});
