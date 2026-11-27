// Owned Windows formal CLI acceptance: inspect-app, launch-plan, session attach-v8, prepare (directory/asar+PE/nested_asar/esm) and dev.
import assert from 'node:assert/strict';
import {execFileSync,spawn,spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import fs from 'node:fs';
import {access,cp,mkdir,mkdtemp,readFile,symlink,unlink,writeFile} from 'node:fs/promises';
import {createRequire} from 'node:module';
import path from 'node:path';
import {finished} from 'node:stream/promises';
import {pathToFileURL} from 'node:url';
import {binary,cache,requireTool} from './test-env.mjs';
import {builtinAsar as asar} from '../crates/preparation/helper/archive.mjs';

if(process.platform!=='win32')throw Error('Windows acceptance script runs on win32 only');

const toolsDir=process.env.INJECTRA_PREPARATION_TOOLS??path.resolve(cache,'preparation-tools');
const tools=createRequire(path.join(toolsDir,'package.json'));
const pe=await import(pathToFileURL(tools.resolve('resedit')).href);
const sha=async p=>createHash('sha256').update(await readFile(p)).digest('hex');
const sleep=ms=>new Promise(r=>setTimeout(r,ms));

function cli(args,expected=0,envExtra={}){
 const r=spawnSync(binary,args,{encoding:'utf8',timeout:60000,env:{...process.env,...envExtra}});
 if(r.error)throw r.error;
 assert.equal(r.status,expected,`CLI ${args.join(' ')} failed (${r.status}):\nstdout=${r.stdout}\nstderr=${r.stderr}`);
 return expected===0?JSON.parse(r.stdout):r.stderr;
}

function cliMachine(args,expected=0,envExtra={}){
 const r=spawnSync(binary,['--machine',...args],{encoding:'utf8',timeout:60000,env:{...process.env,...envExtra}});
 if(r.error)throw r.error;
 assert.equal(r.status,expected,`CLI --machine ${args.join(' ')} failed (${r.status}):\nstdout=${r.stdout}\nstderr=${r.stderr}`);
 const line=r.stdout.trimEnd().split(/\r?\n/).at(-1);
 return JSON.parse(line);
}

await mkdir(path.resolve(cache,'runs'),{recursive:true});
const run=await mkdtemp(path.resolve(cache,'runs','win-formal-'));
const evidence={platform:process.platform,arch:process.arch,run,checks:{}};

// 1. Read-only inspect-app & launch-plan checks
const sampleElectron=process.env.INJECTRA_WIN_SAMPLE_ELECTRON;
if(sampleElectron&&fs.existsSync(sampleElectron)){
 const rep=cli(['inspect-app','--app',sampleElectron,'--port','9222']);
 assert.equal(rep.host_family,'electron');
 assert.equal(rep.routes.find(r=>r.route==='cdp')?.supported,true);
 assert.equal(rep.routes.find(r=>r.route==='prepare-dev')?.supported,true);
 assert.equal(rep.routes.find(r=>r.route==='windows-v8')?.supported,true);
 assert.ok(rep.launch_plan?.arguments?.includes('--remote-debugging-port=9222'));
 evidence.checks.inspectSampleElectron={passed:true,path:sampleElectron,detected:rep.detected_components,package:rep.package};
}

const wv2Dir=path.join(run,'synthetic-wv2');
await mkdir(wv2Dir,{recursive:true});
await writeFile(path.join(wv2Dir,'Sample.exe'),Buffer.from('MZ'));
await writeFile(path.join(wv2Dir,'WebView2Loader.dll'),Buffer.from('MZ'));
const wv2Report=cli(['inspect-app','--app',wv2Dir,'--port','9333']);
assert.equal(wv2Report.host_family,'system-webview');
assert.equal(wv2Report.engine,'webview2');
assert.match(wv2Report.launch_plan.environment.WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS,/--remote-debugging-port=9333/);
const nativeDir=path.join(run,'synthetic-native');
await mkdir(nativeDir,{recursive:true});
await writeFile(path.join(nativeDir,'Plain.exe'),Buffer.from('MZ'));
const nativeErr=cliMachine(['inspect-app','--app',nativeDir],3);
assert.equal(nativeErr.ok,false);
assert.equal(nativeErr.error.kind,'Unsupported');
assert.equal(nativeErr.error.certainty,'NotExecuted');
evidence.checks.inspectSynthetic={passed:true};

// 2. Formal Windows CLI session attach-v8 against owned Node.exe process
const v8Dir=path.join(run,'v8-node-fixture');
await mkdir(v8Dir,{recursive:true});
const keepAlive=path.join(v8Dir,'keepalive.cjs');
const markerFile=path.join(v8Dir,'marker.json');
const dispatchSrc=path.join(v8Dir,'dispatch.js');
await writeFile(keepAlive,'setInterval(()=>{},100);');
await writeFile(dispatchSrc,`process.mainModule.require('fs').writeFileSync(${JSON.stringify(markerFile)},JSON.stringify({answer:6*7,pid:process.pid,platform:process.platform}));`);
const nodeChild=spawn(process.execPath,[keepAlive],{stdio:'ignore'});
try{
 await sleep(800);
 const mismatch=cliMachine(['session','attach-v8','--pid',String(nodeChild.pid),'--exe',path.join(nativeDir,'Plain.exe'),'--source-file',dispatchSrc],3);
 assert.equal(mismatch.ok,false);
 assert.equal(mismatch.error.kind,'TargetNotFound');
 assert.equal(mismatch.error.certainty,'NotExecuted');

 const attached=cliMachine(['session','attach-v8','--pid',String(nodeChild.pid),'--exe',process.execPath,'--source-file',dispatchSrc],0);
 assert.equal(attached.ok,true);
 assert.equal(attached.data.dispatched,true);
 assert.equal(attached.data.certainty,'Executed');
 const marker=JSON.parse(await readFile(markerFile,'utf8'));
 assert.deepEqual(marker,{answer:42,pid:nodeChild.pid,platform:'win32'});
 evidence.checks.sessionAttachV8={passed:true,pid:nodeChild.pid,arch:attached.data.agent?.arch,hooks:attached.data.agent?.hooks};
}finally{
 if(nodeChild.exitCode===null)nodeChild.kill('SIGTERM');
}

// 3. Formal Windows CLI prepare + dev across directory (+junction), asar (+PE INTEGRITY), nested_asar, and esm
const electronExe=requireTool('electron');
const electronRoot=path.dirname(electronExe);
const mainCjs=`#!/usr/bin/env node
'use strict';
const {app,BrowserWindow}=require('electron'),fs=require('fs'),path=require('path');
const identity={is_main:require.main===module,process_main:process.mainModule===module,filename:__filename,relative:require('./sibling.cjs'),strict:(function(){return this;})()===undefined};
fs.mkdirSync(app.getPath('userData'),{recursive:true});fs.writeFileSync(path.join(app.getPath('userData'),'entry.json'),JSON.stringify(identity));
app.whenReady().then(async()=>{const win=new BrowserWindow({show:false,width:600,height:400,webPreferences:{contextIsolation:true,sandbox:true}});await win.loadFile(path.join(__dirname,'index.html'));await win.webContents.executeJavaScript('globalThis.fixture='+JSON.stringify(identity));const peer=new BrowserWindow({show:false,webPreferences:{contextIsolation:true,sandbox:true}});await peer.loadFile(path.join(__dirname,'index.html'));setInterval(async()=>{try{const count=await peer.webContents.executeJavaScript('document.querySelectorAll("[data-injectra]").length');fs.writeFileSync(path.join(app.getPath('userData'),'peer.json'),JSON.stringify({count}));}catch{}},250).unref();});
app.on('window-all-closed',()=>app.quit());
`;
const mainEsm=`import {app,BrowserWindow} from 'electron';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import sibling from './sibling.mjs';
const __filename=fileURLToPath(import.meta.url);
const __dirname=path.dirname(__filename);
const identity={is_main:true,process_main:true,filename:__filename,relative:sibling,strict:true,esm:true};
fs.mkdirSync(app.getPath('userData'),{recursive:true});fs.writeFileSync(path.join(app.getPath('userData'),'entry.json'),JSON.stringify(identity));
app.whenReady().then(async()=>{const win=new BrowserWindow({show:false,width:600,height:400,webPreferences:{contextIsolation:true,sandbox:true}});await win.loadFile(path.join(__dirname,'index.html'));await win.webContents.executeJavaScript('globalThis.fixture='+JSON.stringify(identity));const peer=new BrowserWindow({show:false,webPreferences:{contextIsolation:true,sandbox:true}});await peer.loadFile(path.join(__dirname,'index.html'));setInterval(async()=>{try{const count=await peer.webContents.executeJavaScript('document.querySelectorAll("[data-injectra]").length');fs.writeFileSync(path.join(app.getPath('userData'),'peer.json'),JSON.stringify({count}));}catch{}},250).unref();});
app.on('window-all-closed',()=>app.quit());
`;

function injectPeIntegrity(exePath,asarPath){
 const headerHash=createHash('sha256').update(asar.getRawHeader(asarPath).headerString).digest('hex');
 const bytes=fs.readFileSync(exePath);
 const exe=pe.NtExecutable.from(bytes,{ignoreCert:true});
 const resources=pe.NtExecutableResource.from(exe);
 const payload=[{file:'resources\\app.asar',alg:'sha256',value:headerHash}];
 resources.entries.push({type:'INTEGRITY',id:'ELECTRONASAR',lang:1033,codepage:1200,bin:new TextEncoder().encode(JSON.stringify(payload)).buffer});
 resources.outputResource(exe);
 fs.writeFileSync(exePath,Buffer.from(exe.generate()));
 return headerHash;
}

evidence.checks.prepareDev={};
const layouts=['directory','asar','nested_asar','esm'];
for(const layout of layouts){
 const isEsm=layout==='esm';
 const fixtureDir=path.join(run,'fixture-'+layout);
 await cp(electronRoot,fixtureDir,{recursive:true,dereference:false});
 const resDir=path.join(fixtureDir,'resources');
 await mkdir(resDir,{recursive:true});
 for(const leftover of ['default_app.asar','app.asar','app']){
  const p=path.join(resDir,leftover);
  if(fs.existsSync(p))fs.rmSync(p,{recursive:true,force:true});
 }
 const srcDir=path.join(run,'src-'+layout);
 await mkdir(srcDir,{recursive:true});
 if(isEsm){
  await writeFile(path.join(srcDir,'package.json'),JSON.stringify({name:'win-fixture-'+layout,version:'1.0.0',type:'module',main:'main.mjs'}));
  await writeFile(path.join(srcDir,'main.mjs'),mainEsm);
  await writeFile(path.join(srcDir,'sibling.mjs'),'export default "relative-esm-module";');
 }else{
  await writeFile(path.join(srcDir,'package.json'),JSON.stringify({name:'win-fixture-'+layout,version:'1.0.0',main:'main.cjs'}));
  await writeFile(path.join(srcDir,'main.cjs'),mainCjs);
  await writeFile(path.join(srcDir,'sibling.cjs'),'module.exports="relative-module";');
 }
 await writeFile(path.join(srcDir,'index.html'),'<!doctype html><meta charset="utf-8"><title>Owned Windows fixture</title><div id="root">Original Windows fixture</div>');

 let peBeforeHash=null;
 if(layout==='directory'||layout==='esm'){
  await cp(srcDir,path.join(resDir,'app'),{recursive:true});
  if(layout==='directory'){
   await mkdir(path.join(fixtureDir,'vendor','assets'),{recursive:true});
   await writeFile(path.join(fixtureDir,'vendor','assets','marker.txt'),'ok');
   await symlink(path.join(fixtureDir,'vendor','assets'),path.join(fixtureDir,'linked-assets'),'junction');
  }
 }else if(layout==='asar'){
  const archivePath=path.join(resDir,'app.asar');
  const out=await asar.createPackage(srcDir,archivePath);
  if(out&&typeof out.on==='function')await finished(out);
  peBeforeHash=injectPeIntegrity(path.join(fixtureDir,'electron.exe'),archivePath);
 }else if(layout==='nested_asar'){
  const appDir=path.join(resDir,'app');
  await mkdir(appDir,{recursive:true});
  await writeFile(path.join(appDir,'package.json'),JSON.stringify({name:'win-nested',version:'1.0.0',main:'./nested.asar/main.cjs'}));
  const out=await asar.createPackage(srcDir,path.join(appDir,'nested.asar'));
  if(out&&typeof out.on==='function')await finished(out);
 }

 const originalExe=path.join(fixtureDir,'electron.exe');
 const originalExeHash=await sha(originalExe);
 const originalEntry=layout==='asar'?path.join(resDir,'app.asar'):layout==='nested_asar'?path.join(resDir,'app','nested.asar'):isEsm?path.join(resDir,'app','main.mjs'):path.join(resDir,'app','main.cjs');
 const beforeEntryHash=await sha(originalEntry);

 const inspectRes=cli(['inspect-app','--app',fixtureDir]);
 assert.equal(inspectRes.host_family,'electron');
 if(layout==='asar')assert.ok(inspectRes.detected_components.includes('pe-asar-integrity'),'pure-Rust PE reader must detect INTEGRITY/ELECTRONASAR');

 const artifact=cli(['prepare','--app',fixtureDir,'--output',path.join(run,'cli-artifacts'),'--data-mode','isolated']);
 assert.equal(artifact.platform,'windows');
 assert.equal(await sha(originalExe),originalExeHash,'source exe must remain unchanged');
 assert.equal(await sha(originalEntry),beforeEntryHash,'source entry must remain unchanged');
 if(isEsm)assert.equal(artifact.entry_strategy,'esm-delegate');
 if(layout==='nested_asar')assert.equal(artifact.entry_strategy,'main-delegate');
 if(layout==='asar'){
  const newHeaderHash=createHash('sha256').update(asar.getRawHeader(path.join(artifact.app,'resources','app.asar')).headerString).digest('hex');
  assert.notEqual(newHeaderHash,peBeforeHash);
  const updatedPe=pe.NtExecutableResource.from(pe.NtExecutable.from(fs.readFileSync(artifact.executable),{ignoreCert:true}));
  const integ=JSON.parse(Buffer.from(updatedPe.entries.find(e=>String(e.type).toUpperCase()==='INTEGRITY').bin).toString('utf8').replace(/\0+$/,''));
  assert.equal(integ[0].value,newHeaderHash,'PE INTEGRITY resource must match updated ASAR header');
 }
 if(layout==='directory'){
  const linkTarget=fs.realpathSync(path.join(artifact.app,'linked-assets'));
  assert.ok(linkTarget.toLowerCase().startsWith(artifact.app.toLowerCase()),'junction must resolve inside derived copy');
 }
 await assert.rejects(access(path.resolve(artifact.artifact_file,'../preflight-home/profile/entry.json')));

 const sourceDir=path.join(run,'increments-'+layout);
 await mkdir(sourceDir,{recursive:true});
 const manifest=path.join(sourceDir,'manifest.json');
 await writeFile(manifest,JSON.stringify({version:1,id:'injectra.test.win.dev',js:'page.js',css:'page.css',html:'page.html'}));
 await writeFile(path.join(sourceDir,'page.css'),'[data-proof] { color: rgb(1, 2, 3); }');
 await writeFile(path.join(sourceDir,'page.html'),'<span data-proof>first</span>');
 const expectedSuffix=isEsm?'main.mjs':'main.cjs';
 const expectedRel=isEsm?'relative-esm-module':'relative-module';
 const js=stage=>`if (!globalThis.fixture?.is_main || !globalThis.fixture?.process_main || !globalThis.fixture?.filename?.endsWith(${JSON.stringify(expectedSuffix)}) || globalThis.fixture?.relative !== ${JSON.stringify(expectedRel)} || !globalThis.fixture?.strict) throw Error('entry semantics');\nif(context.root.textContent!==${JSON.stringify(stage)})throw Error('HTML update');\nif(getComputedStyle(context.root.firstElementChild).color!=='rgb(1, 2, 3)')throw Error('CSS update');\nconsole.log('INJECTRA_WIN_PROOF_${stage}');`;
 await writeFile(path.join(sourceDir,'page.js'),js('first'));

 const events=[];let stderr='',partial='';
 const child=spawn(binary,['dev','--artifact',artifact.artifact_file,'--bundle',manifest,'--target','1'],{stdio:['ignore','pipe','pipe']});
 child.stdout.on('data',b=>{partial+=b;let at;while((at=partial.indexOf('\n'))>=0){const line=partial.slice(0,at).trim();partial=partial.slice(at+1);if(!line)continue;try{events.push(JSON.parse(line));}catch{}}});
 child.stderr.on('data',b=>stderr+=b);
 const wait=async predicate=>{for(let i=0;i<240;i++){const ev=events.find(predicate);if(ev)return ev;if(child.exitCode!==null)throw Error(`dev exited early (${child.exitCode}): ${stderr}\nevents=${JSON.stringify(events)}`);await sleep(100);}throw Error(`Timeout waiting for dev event: ${JSON.stringify(events)} stderr=${stderr}`);};
 let pid=null;
 try{
  pid=(await wait(e=>e.event==='dev-ready')).pid;
  const first=await wait(e=>e.event==='execution');
  assert.equal(first.certainty,'Executed');assert.equal(first.error,null);
  await wait(e=>e.event==='log'&&e.data.message==='INJECTRA_WIN_PROOF_first');
  await writeFile(path.join(sourceDir,'page.html'),'<span data-proof>second</span>');
  await writeFile(path.join(sourceDir,'page.js'),js('second'));
  await wait(e=>e.event==='log'&&e.data.message==='INJECTRA_WIN_PROOF_second');
  await writeFile(path.join(sourceDir,'page.js'),"throw new Error('win-source-location-proof');");
  const errEv=await wait(e=>e.event==='execution'&&e.error?.includes('win-source-location-proof'));
  assert.equal(errEv.certainty,'Executed');assert.equal(errEv.location?.line,1);
  evidence.checks.prepareDev[layout]={passed:true,strategy:artifact.entry_strategy,pid,artifact:artifact.artifact_file};
 }finally{
  if(child.exitCode===null){child.kill('SIGTERM');for(let i=0;i<30&&child.exitCode===null;i++)await sleep(100);}
  if(pid){
   try{
    const verifyExe=execFileSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',`(Get-CimInstance Win32_Process -Filter "ProcessId=${pid}").ExecutablePath`],{encoding:'utf8'}).trim();
    if(verifyExe.toLowerCase()===artifact.executable.toLowerCase()){
     execFileSync('taskkill.exe',['/PID',String(pid),'/T','/F'],{stdio:'ignore'});
    }
   }catch{}
  }
 }
}

// 4. Bytecode (.jsc) rejected before creating output directory
const jscDir=path.join(run,'fixture-directory');
const pkgPath=path.join(jscDir,'resources','app','package.json');
const origPkg=await readFile(pkgPath);
const jscPath=path.join(jscDir,'resources','app','main.jsc');
const rejected=path.join(run,'rejected-win-output');
try{
 await writeFile(pkgPath,JSON.stringify({name:'win-jsc',main:'main.jsc'}));
 await writeFile(jscPath,Buffer.from([0xde,0xad,0xbe,0xef]));
 const err=cliMachine(['prepare','--app',jscDir,'--output',rejected,'--data-mode','isolated'],3);
 assert.equal(err.ok,false);
 assert.equal(err.error.kind,'Unsupported');
 assert.equal(err.error.certainty,'NotExecuted');
 await assert.rejects(access(rejected));
 evidence.checks.rejectJsc={passed:true};
}finally{
 await writeFile(pkgPath,origPkg);
 await unlink(jscPath);
}

await writeFile(path.join(run,'result.json'),JSON.stringify(evidence,null,2));
console.log(JSON.stringify(evidence,null,2));
