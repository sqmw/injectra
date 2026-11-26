// Create only owned synthetic app fixtures (CJS & ESM); never launches or modifies installed apps.
import {mkdir,mkdtemp,writeFile,readdir} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {resolve} from 'node:path';
import {createRequire} from 'node:module';
import {finished} from 'node:stream/promises';
import {cache,requireTool} from './test-env.mjs';
const asar=createRequire(resolve(cache,'preparation-tools/package.json'))('@electron/asar');
await mkdir(resolve(cache,'runs'),{recursive:true});
const run=await mkdtemp(resolve(cache,'runs/preparation-fixtures-'));
const electron=requireTool('electron'),original=resolve(electron,'../../..');
const mainCjs=`#!/usr/bin/env node
'use strict';
const {app,BrowserWindow}=require('electron'),fs=require('fs'),path=require('path');
const identity={is_main:require.main===module,process_main:process.mainModule===module,filename:__filename,relative:require('./sibling.cjs'),strict:(function(){return this;})()===undefined};
fs.mkdirSync(app.getPath('userData'),{recursive:true});fs.writeFileSync(path.join(app.getPath('userData'),'entry.json'),JSON.stringify(identity));
app.whenReady().then(async()=>{const win=new BrowserWindow({show:false,width:600,height:400,webPreferences:{contextIsolation:true,sandbox:true}});await win.loadFile(path.join(__dirname,'index.html'));await win.webContents.executeJavaScript('globalThis.fixture='+JSON.stringify(identity));const peer=new BrowserWindow({show:false,webPreferences:{contextIsolation:true,sandbox:true}});await peer.loadFile(path.join(__dirname,'index.html'));setInterval(async()=>{try{const count=await peer.webContents.executeJavaScript('document.querySelectorAll(\"[data-injectra]\").length');fs.writeFileSync(path.join(app.getPath('userData'),'peer.json'),JSON.stringify({count}));}catch{}},250).unref();});
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
app.whenReady().then(async()=>{const win=new BrowserWindow({show:false,width:600,height:400,webPreferences:{contextIsolation:true,sandbox:true}});await win.loadFile(path.join(__dirname,'index.html'));await win.webContents.executeJavaScript('globalThis.fixture='+JSON.stringify(identity));const peer=new BrowserWindow({show:false,webPreferences:{contextIsolation:true,sandbox:true}});await peer.loadFile(path.join(__dirname,'index.html'));setInterval(async()=>{try{const count=await peer.webContents.executeJavaScript('document.querySelectorAll(\"[data-injectra]\").length');fs.writeFileSync(path.join(app.getPath('userData'),'peer.json'),JSON.stringify({count}));}catch{}},250).unref();});
app.on('window-all-closed',()=>app.quit());
`;
const kinds=['directory','asar','nested_asar','esm','esm_asar'];
for(const kind of kinds){
 const app=resolve(run,kind+'.app');execFileSync('/usr/bin/rsync',['-a',original+'/',app+'/']);
 const source=resolve(run,kind+'-source');await mkdir(source);
 const isEsm=kind==='esm'||kind==='esm_asar';
 if(isEsm){
  await writeFile(resolve(source,'package.json'),JSON.stringify({name:'injectra-fixture-'+kind,version:'1.0.0',type:'module',main:'main.mjs'}));
  await writeFile(resolve(source,'main.mjs'),mainEsm);
  await writeFile(resolve(source,'sibling.mjs'),'export default "relative-esm-module";');
 }else{
  await writeFile(resolve(source,'package.json'),JSON.stringify({name:'injectra-fixture-'+kind,version:'1.0.0',main:'main.cjs'}));
  await writeFile(resolve(source,'main.cjs'),mainCjs);
  await writeFile(resolve(source,'sibling.cjs'),'module.exports="relative-module";');
 }
 await writeFile(resolve(source,'index.html'),'<!doctype html><meta charset="utf-8"><title>Owned source fixture</title><div id="root">Original fixture</div>');
 if(kind==='directory'||kind==='esm')execFileSync('/usr/bin/rsync',['-a',source+'/',resolve(app,'Contents/Resources/app')+'/']);
 else {
  const directory=resolve(app,'Contents/Resources/app');
  if(kind==='nested_asar'){await mkdir(directory,{recursive:true});await writeFile(resolve(directory,'package.json'),JSON.stringify({name:'nested-fixture',main:'./nested.asar/main.cjs'}));}
  const output=await asar.createPackage(source,kind==='nested_asar'?resolve(directory,'nested.asar'):resolve(app,'Contents/Resources/app.asar'));
  if(output&&typeof output.on==='function')await finished(output);
 }
 async function bundles(directory){const out=[];for(const e of await readdir(directory,{withFileTypes:true})){if(!e.isDirectory()||e.isSymbolicLink())continue;const p=resolve(directory,e.name);out.push(...await bundles(p));if(/\.(app|framework|xpc)$/.test(e.name))out.push(p);}return out;}
 for(const bundle of [...await bundles(app),app])execFileSync('/usr/bin/codesign',['--force','--sign','-','--timestamp=none','--preserve-metadata=identifier,entitlements,flags',bundle]);
 await writeFile(resolve(app,'Contents/unsealed-account-fixture.txt'),'MUST_NOT_COPY_THIS_RUNTIME_DATA');
}
const manifest={run,directory:resolve(run,'directory.app'),asar:resolve(run,'asar.app'),nested_asar:resolve(run,'nested_asar.app'),esm:resolve(run,'esm.app'),esm_asar:resolve(run,'esm_asar.app')};
await writeFile(resolve(run,'fixtures.json'),JSON.stringify(manifest,null,2));
console.log(JSON.stringify(manifest,null,2));
