// Owned Electron directory/ASAR/ESM acceptance; all payloads and output stay in external cache.
import assert from 'node:assert/strict';
import {execFileSync,spawn} from 'node:child_process';
import {mkdir,readFile,writeFile,unlink,access} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {binary,root} from './test-env.mjs';
const fixtures=JSON.parse(process.env.INJECTRA_FIXTURES_FILE?await readFile(process.env.INJECTRA_FIXTURES_FILE,'utf8'):execFileSync(process.execPath,[resolve(root,'scripts/preparation-fixtures.mjs')],{encoding:'utf8'}));
const sha=async p=>createHash('sha256').update(await readFile(p)).digest('hex');
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
for(const layout of ['directory','asar','nested_asar','esm','esm_asar']) {
 const isEsm=layout==='esm'||layout==='esm_asar';
 const original=fixtures[layout];
 const originalEntry=(layout==='asar'||layout==='esm_asar')
  ?resolve(original,'Contents/Resources/app.asar')
  :resolve(original,layout==='nested_asar'?'Contents/Resources/app/nested.asar':isEsm?'Contents/Resources/app/main.mjs':'Contents/Resources/app/main.cjs');
 const before=await sha(originalEntry);
 const artifact=JSON.parse(execFileSync(binary,['prepare','--app',original,'--output',resolve(fixtures.run,'cli-artifacts'),'--data-mode','isolated'],{encoding:'utf8'}));
 assert.equal(await sha(originalEntry),before);
 if(isEsm)assert.equal(artifact.entry_strategy,'esm-delegate');
 const prepared=resolve(artifact.app,(layout==='asar'||layout==='esm_asar')?'Contents/Resources/app.asar':layout==='nested_asar'?'Contents/Resources/app/nested.asar':isEsm?'Contents/Resources/app/main.mjs':'Contents/Resources/app/main.cjs');
 const preparedHash=await sha(prepared);
 if(layout==='nested_asar'||layout==='esm')assert.equal(preparedHash,before,'delegated original entry must remain byte-for-byte unchanged');
 await assert.rejects(access(resolve(artifact.artifact_file,'../preflight-home/profile/entry.json')));
 const source=resolve(fixtures.run,'increments-'+layout);await mkdir(source,{recursive:true});
 const manifest=resolve(source,'manifest.json');
 await writeFile(manifest,JSON.stringify({version:1,id:'injectra.test.dev',js:'page.js',css:'page.css',html:'page.html'}));
 await writeFile(resolve(source,'page.css'),'[data-proof] { color: rgb(1, 2, 3); }');
 await writeFile(resolve(source,'page.html'),'<span data-proof>first</span>');
 const expectedSuffix=isEsm?'/main.mjs':'/main.cjs';
 const expectedRel=isEsm?'relative-esm-module':'relative-module';
 const js=stage=>`if (!globalThis.fixture?.is_main || !globalThis.fixture?.process_main || !globalThis.fixture?.filename?.endsWith(${JSON.stringify(expectedSuffix)}) || globalThis.fixture?.relative !== ${JSON.stringify(expectedRel)} || !globalThis.fixture?.strict) throw Error('entry semantics');\nif(context.root.textContent!==${JSON.stringify(stage)})throw Error('HTML update');\nif(getComputedStyle(context.root.firstElementChild).color!=='rgb(1, 2, 3)')throw Error('CSS update');\nconsole.log('INJECTRA_PROOF_${stage}');`;
 await writeFile(resolve(source,'page.js'),js('first'));
 let events=[],stderr='',partial='';
 const child=spawn(binary,['dev','--artifact',artifact.artifact_file,'--bundle',manifest,'--target','1'],{stdio:['ignore','pipe','pipe']});
 child.stdout.on('data',b=>{partial+=b;let at;while((at=partial.indexOf('\n'))>=0){const line=partial.slice(0,at);partial=partial.slice(at+1);try{events.push(JSON.parse(line));}catch{}}});child.stderr.on('data',b=>stderr+=b);
 const wait=async predicate=>{for(let i=0;i<200;i++){const event=events.find(predicate);if(event)return event;if(child.exitCode!==null)throw Error(stderr);await sleep(100);}throw Error('Timeout: '+JSON.stringify(events)+' '+stderr);};
 let pid;
 try {
  pid=(await wait(e=>e.event==='dev-ready')).pid;
  const first=await wait(e=>e.event==='execution');assert.equal(first.certainty,'Executed');assert.equal(first.error,null);
  await wait(e=>e.event==='log'&&e.data.message==='INJECTRA_PROOF_first');
  await writeFile(resolve(source,'page.html'),'<span data-proof>second</span>');await writeFile(resolve(source,'page.js'),js('second'));
  await wait(e=>e.event==='log'&&e.data.message==='INJECTRA_PROOF_second');
  assert.equal(await sha(prepared),preparedHash);assert.equal(await sha(originalEntry),before);
  await sleep(500);assert.equal(JSON.parse(await readFile(resolve(artifact.artifact_file,'../home/profile/peer.json'),'utf8')).count,0);
  const executions=events.filter(e=>e.event==='execution');assert.equal(executions.length,2);assert.notEqual(executions[0].version,executions[1].version);
  await writeFile(resolve(source,'page.js'),"throw new Error('source-location-proof');");
  const error=await wait(e=>e.event==='execution'&&e.error?.includes('source-location-proof'));assert.equal(error.certainty,'Executed');assert.equal(error.location?.line,1);
  await writeFile(resolve(source,'page.js'),js('second'));
  await wait(e=>e.event==='execution'&&e.request_id!==executions[1].request_id&&e.version===executions[1].version);
  // Navigation invalidates the old target; no automatic reinjection follows.
  await writeFile(resolve(source,'page.js'),'setTimeout(()=>location.reload(),300);');
  for(let i=0;i<100&&child.exitCode===null;i++)await sleep(100);
  assert.equal(child.exitCode,1,stderr);assert.match(stderr,/page closed or changed/);
  assert.equal(await sha(prepared),preparedHash);
 } finally {
  if(child.exitCode===null){child.kill('SIGINT');for(let i=0;i<30&&child.exitCode===null;i++)await sleep(100);}
  if(pid){const command=execFileSync('/bin/ps',['-p',String(pid),'-o','comm='],{encoding:'utf8'}).trim();assert.equal(command,artifact.executable);process.kill(pid,'SIGTERM');}
  await writeFile(resolve(source,'acceptance.json'),JSON.stringify({layout,artifact:artifact.artifact_file,events,stderr,preparedHash},null,2));
 }
 console.log(JSON.stringify({layout,status:'passed',artifact:artifact.artifact_file}));
}

// Explicitly unsupported bytecode (.jsc) is rejected before creating an output artifact.
const packageFile=resolve(fixtures.directory,'Contents/Resources/app/package.json');
const originalPackage=await readFile(packageFile);const jsc=resolve(fixtures.directory,'Contents/Resources/app/main.jsc');
const rejected=resolve(fixtures.run,'rejected-output');
try {
 await writeFile(packageFile,JSON.stringify({name:'unsupported-fixture',main:'main.jsc'}));await writeFile(jsc,Buffer.from([0xde,0xad,0xbe,0xef]));
 let error;try{execFileSync(binary,['prepare','--app',fixtures.directory,'--output',rejected,'--data-mode','isolated'],{encoding:'utf8',stdio:'pipe'});}catch(e){error=e;}
 assert.ok(error);assert.match(String(error.stderr),/Unsupported.*(bytecode|non-JavaScript)/);await assert.rejects(access(rejected));
}finally{await writeFile(packageFile,originalPackage);await unlink(jsc);}
console.log(JSON.stringify({unsupported:'jsc-bytecode',status:'rejected-before-mutation'}));
