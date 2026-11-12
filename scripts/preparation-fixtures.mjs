// Create only owned synthetic app fixtures; never launches or modifies installed apps.
import {mkdir,mkdtemp,writeFile,copyFile,readdir} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {resolve} from 'node:path';
import {createRequire} from 'node:module';
import {finished} from 'node:stream/promises';
import {cache,requireTool} from './test-env.mjs';
const asar=createRequire(resolve(cache,'preparation-tools/package.json'))('@electron/asar');
await mkdir(resolve(cache,'runs'),{recursive:true});
const run=await mkdtemp(resolve(cache,'runs/preparation-fixtures-'));
const electron=requireTool('electron'),original=resolve(electron,'../../..');
const main=`#!/usr/bin/env node
'use strict';
const {app,BrowserWindow}=require('electron'),fs=require('fs'),path=require('path');
const identity={is_main:require.main===module,process_main:process.mainModule===module,filename:__filename,relative:require('./sibling.cjs'),strict:(function(){return this;})()===undefined};
fs.mkdirSync(app.getPath('userData'),{recursive:true});fs.writeFileSync(path.join(app.getPath('userData'),'entry.json'),JSON.stringify(identity));
app.whenReady().then(async()=>{const win=new BrowserWindow({show:false,width:600,height:400,webPreferences:{contextIsolation:true,sandbox:true}});await win.loadFile(path.join(__dirname,'index.html'));await win.webContents.executeJavaScript('globalThis.fixture='+JSON.stringify(identity));const peer=new BrowserWindow({show:false,webPreferences:{contextIsolation:true,sandbox:true}});await peer.loadFile(path.join(__dirname,'index.html'));setInterval(async()=>{try{const count=await peer.webContents.executeJavaScript('document.querySelectorAll(\"[data-injectra]\").length');fs.writeFileSync(path.join(app.getPath('userData'),'peer.json'),JSON.stringify({count}));}catch{}},250).unref();});
app.on('window-all-closed',()=>app.quit());
`;
for(const kind of ['directory','asar']){
 const app=resolve(run,kind+'.app');execFileSync('/usr/bin/rsync',['-a',original+'/',app+'/']);
 const source=resolve(run,kind+'-source');await mkdir(source);
 await writeFile(resolve(source,'package.json'),JSON.stringify({name:'injectra-fixture-'+kind,version:'1.0.0',main:'main.cjs'}));
 await writeFile(resolve(source,'main.cjs'),main);await writeFile(resolve(source,'sibling.cjs'),'module.exports="relative-module";');
 await writeFile(resolve(source,'index.html'),'<!doctype html><meta charset="utf-8"><title>Owned source fixture</title><div id="root">Original fixture</div>');
 if(kind==='directory')execFileSync('/usr/bin/rsync',['-a',source+'/',resolve(app,'Contents/Resources/app')+'/']);
 else {const output=await asar.createPackage(source,resolve(app,'Contents/Resources/app.asar'));if(output&&typeof output.on==='function')await finished(output);}
 async function bundles(directory){const out=[];for(const e of await readdir(directory,{withFileTypes:true})){if(!e.isDirectory()||e.isSymbolicLink())continue;const p=resolve(directory,e.name);out.push(...await bundles(p));if(/\.(app|framework|xpc)$/.test(e.name))out.push(p);}return out;}
 for(const bundle of [...await bundles(app),app])execFileSync('/usr/bin/codesign',['--force','--sign','-','--timestamp=none','--preserve-metadata=identifier,entitlements,flags',bundle]);
 await writeFile(resolve(app,'Contents/unsealed-account-fixture.txt'),'MUST_NOT_COPY_THIS_RUNTIME_DATA');
}
await writeFile(resolve(run,'fixtures.json'),JSON.stringify({run,directory:resolve(run,'directory.app'),asar:resolve(run,'asar.app')},null,2));
console.log(JSON.stringify({run,directory:resolve(run,'directory.app'),asar:resolve(run,'asar.app')},null,2));
