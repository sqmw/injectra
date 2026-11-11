// Local development preview. Payloads remain in the external plugin library.
import { execFileSync, spawn } from "node:child_process";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { openSync, closeSync } from "node:fs";
import { resolve } from "node:path";
import { root } from "./test-env.mjs";
import { createServer } from "node:http";
import { randomBytes, createHash } from "node:crypto";
import { once } from "node:events";
import { existsSync } from "node:fs";
const [mode, argument] = process.argv.slice(2);
if (mode !== "--prepare") throw Error("Use --prepare [compiled-program-file]; omit file for DOM metadata inspection");
const output = execFileSync(process.execPath,[resolve(root,"scripts/resource-patch-smoke.mjs"),"--prepare","qq","--shared-data"],{encoding:"utf8",env:{...process.env,INJECTRA_EXECUTION_ID:"T-039/E-001"}});
const {run}=JSON.parse(output), plan=JSON.parse(await readFile(resolve(run,"plan.json"),"utf8"));
const {app}=plan;
const programFile=resolve(run,"program.js");await writeFile(programFile,argument?await readFile(resolve(argument),"utf8"):"null");
const inspect=`(() => {
 const doc=document, root=doc.documentElement;
 const safeClass=n=>/^[a-zA-Z_][a-zA-Z_0-9-]{0,70}$/.test(n)&&!/[0-9]{6}/.test(n);
 const counts={}; for(const el of doc.querySelectorAll('body *')) for(const n of el.classList) if(safeClass(n)) counts[n]=(counts[n]||0)+1;
 const vars={}; for(const el of [root,doc.body,doc.querySelector('#app'),doc.querySelector('#root')].filter(Boolean)) {const s=getComputedStyle(el);for(let i=0;i<s.length;i++){const n=s.item(i),v=s.getPropertyValue(n).trim();if(n.startsWith('--')&&/^(#[0-9a-f]{3,8}$|rgba?\\(|transparent$)/i.test(v))vars[n]=v;}}
 const layout=Array.from(doc.querySelectorAll('body *')).filter(el=>{const r=el.getBoundingClientRect();return r.width>150&&r.height>200;}).slice(0,32).map(el=>({tag:el.tagName,classes:Array.from(el.classList).filter(safeClass),width:Math.round(el.getBoundingClientRect().width),height:Math.round(el.getBoundingClientRect().height),background:getComputedStyle(el).backgroundColor,color:getComputedStyle(el).color}));
 return {path:location.pathname,classes:counts,colors:vars,layout,theme:globalThis.QQPastelThemes?.status?.()??null};
})()`;
const previous=JSON.parse(await readFile(resolve(plan.source,"Contents/Resources/app/package.json"),"utf8")).main;
const token=randomBytes(16).toString("hex");
const server=createServer(async(req,res)=>{try{if(req.url===`/${token}/program`){if(existsSync(resolve(run,"finish-preview"))){res.end(JSON.stringify({stop:true}));setTimeout(()=>server.close(),1500);return;}const program=await readFile(programFile,"utf8");res.setHeader("Content-Type","application/json");res.end(JSON.stringify({version:createHash("sha256").update(program).digest("hex"),program}));return;}
if(req.url===`/${token}/report`&&req.method==="POST"){let data="";for await(const chunk of req){data+=chunk;if(data.length>131072)throw Error("Report too large");}const report=JSON.parse(data);if(!Number.isSafeInteger(report.webContentsId))throw Error("Invalid target");await writeFile(resolve(run,`page-${report.webContentsId}.json`),JSON.stringify(report,null,2));res.end("ok");return;}res.writeHead(404).end();}catch(e){res.writeHead(400).end(String(e));}});
server.listen(0,"127.0.0.1");await once(server,"listening");const endpoint=`http://127.0.0.1:${server.address().port}/${token}`;
const wrapper=`const {app}=require('electron'),http=require('http');const targets=new Map();let last='';
const endpoint=${JSON.stringify(endpoint)},inspect=${JSON.stringify(inspect)};
function report(data){const req=http.request(endpoint+'/report',{method:'POST',headers:{'Content-Type':'application/json'}},r=>r.resume());req.on('error',()=>{});req.end(JSON.stringify(data));}
async function sample(wc){if(wc.isDestroyed())return;try{const data=await wc.executeJavaScript(inspect);report({pid:process.pid,webContentsId:wc.id,...data});}catch(e){console.error('injectra-preview',String(e));}}
async function apply(wc,program){if(wc.isDestroyed()||!wc.getURL().includes('/renderer/index.html'))return;try{await wc.executeJavaScript(program);await sample(wc);}catch(e){console.error('injectra-preview',String(e));}}
app.on('browser-window-created',(_,win)=>{const wc=win.webContents;wc.once('dom-ready',()=>{if(!wc.getURL().includes('/renderer/index.html'))return;targets.set(wc.id,wc);win.setTitle('QQ · 粉紫主题预览');win.show();win.focus();sample(wc);});});
const monitor=setInterval(()=>{http.get(endpoint+'/program',r=>{let text='';r.on('data',b=>text+=b);r.on('end',()=>{try{const data=JSON.parse(text);if(data.stop){clearInterval(monitor);return;}if(data.version!==last){last=data.version;for(const wc of targets.values())apply(wc,data.program);}else for(const wc of targets.values())sample(wc);}catch{}});}).on('error',()=>{});},1000);monitor.unref();
require(${JSON.stringify(previous)});`;
await writeFile(resolve(app,"Contents/Resources/app/injectra-start.cjs"),wrapper);
execFileSync("/usr/bin/python3",["-B",resolve(root,"tests/resource-patch/sign-copy.py"),app,"--local-libraries","--outer"]);
execFileSync("/usr/bin/codesign",["--verify","--deep","--strict",app]);
const home=resolve(run,"home"),env={...process.env,HOME:home,CFFIXED_USER_HOME:home};
await mkdir(resolve(home,"Library/Application Support"),{recursive:true});
for(const key of ["ELECTRON_RUN_AS_NODE","NODE_OPTIONS","DYLD_INSERT_LIBRARIES"])delete env[key];
const log=openSync(resolve(run,"preview.log"),"a",0o600),command=[resolve(app,"Contents/MacOS/QQ"),"--use-mock-keychain",`--user-data-dir=${resolve(home,"profile")}`];
const child=spawn(command[0],command.slice(1),{env,detached:true,stdio:["ignore",log,log]});child.unref();closeSync(log);
await writeFile(resolve(run,"theme-preview.json"),JSON.stringify({pid:child.pid,run,app,programFile,sharedData:true,command},null,2));
console.log(JSON.stringify({pid:child.pid,run,app,programFile},null,2));
