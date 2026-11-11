// Explicit, optional test-copy recipes. No installed app or user session is modified.
import assert from "node:assert/strict";
import { readFile, writeFile, mkdir, mkdtemp, copyFile, chmod, access } from "node:fs/promises";
import { openSync, closeSync } from "node:fs";
import { execFileSync, spawn } from "node:child_process";
import { resolve, dirname, basename } from "node:path";
import { randomBytes } from "node:crypto";
import { createServer } from "node:http";
import { once } from "node:events";
import { cache, root } from "./test-env.mjs";
import { asar, rewriteAsar, sha256 } from "../tests/resource-patch/archive.mjs";
import { pageProof } from "../tests/resource-patch/page-proof.mjs";
import { evaluateProof } from "../tests/resource-patch/receipts.mjs";

assert.equal(process.platform, "darwin"); assert.equal(process.arch, "arm64");
const [mode, arg] = process.argv.slice(2);
const sharedData = arg === "qq" && process.argv.includes("--shared-data");
const recipes = {
  qq: { source: "/Applications/QQ.app", executable: "QQ", kind: "main-wrapper" },
  tencentvideo: { source: "/Applications/QQLive.app", executable: "QQLive", kind: "loose-html", html: "Contents/Resources/hd_lottie/demo.html" },
  claude: { source: "/Applications/Claude.app", executable: "Claude", kind: "asar-html", html: ".vite/renderer/main_window/index.html" },
};
const json = value => JSON.stringify(value, null, 2) + "\n";
const digest = async path => sha256(await readFile(path));
const sign = (app, outer = false) => JSON.parse(execFileSync("/usr/bin/python3", ["-B", resolve(root,"tests/resource-patch/sign-copy.py"),app,"--local-libraries",...(outer?["--outer"]:[])],{encoding:"utf8"}));
if (mode === "--prepare") {
  const recipe = recipes[arg]; assert(recipe, "Unknown test recipe");
  const run = await mkdtemp(resolve(cache,"runs/resource-patch-")); await chmod(run,0o700);
  const app = resolve(run,"Injectra-Test.app");
  const critical = ["Contents/MacOS/"+recipe.executable,"Contents/Info.plist","Contents/_CodeSignature/CodeResources",...(recipe.kind === "asar-html"?["Contents/Resources/app.asar"]:recipe.kind === "main-wrapper"?["Contents/Resources/app/package.json","Contents/Resources/app/application.asar"]:[])];
  const originalHashes = Object.fromEntries(await Promise.all(critical.map(async path=>[path,await digest(resolve(recipe.source,path))])));
  await writeFile(resolve(run,"ownership.json"),json({source:recipe.source,app,originalHashes,createdBy:process.env.INJECTRA_EXECUTION_ID??"T-038/E-001",dataPolicy:sharedData?"shared":"isolated"}));
  for(const path of ["scripts/resource-patch-smoke.mjs","tests/resource-patch/page-proof.mjs","tests/resource-patch/sign-copy.py","tests/resource-patch/archive.mjs","tests/resource-patch/integrity-digest.py","web/runtime.js"]){await mkdir(dirname(resolve(run,"source",path)),{recursive:true});await copyFile(resolve(root,path),resolve(run,"source",path));}
  execFileSync("/usr/bin/rsync",["-a",recipe.source+"/",app+"/"]);
  const bundleId = "org.injectra.validation." + arg + "." + basename(run).split("-").at(-1).toLowerCase();
  if(!sharedData) execFileSync("/usr/bin/python3",["-c","import plistlib,sys; p=sys.argv[1]; d=plistlib.load(open(p,'rb')); d['CFBundleIdentifier']=sys.argv[2]; plistlib.dump(d,open(p,'wb'))",resolve(app,"Contents/Info.plist"),bundleId]);
  await writeFile(resolve(run,"nested-signing.json"),json(sign(app)));
  const reservation=createServer();reservation.listen(0,"127.0.0.1");await once(reservation,"listening");const port=reservation.address().port;await new Promise(r=>reservation.close(r));
  const nonce=randomBytes(16).toString("hex"),endpoint=`http://127.0.0.1:${port}/proof/${nonce}`;
  const proof=pageProof({runtime:await readFile(resolve(root,"web/runtime.js"),"utf8"),nonce,endpoint,pageSuffixes:[],rootSelector:"body"});
  await writeFile(resolve(run,"proof.js"),proof);
  let archive=null;
  if(recipe.kind === "main-wrapper") {
    const directory=resolve(app,"Contents/Resources/app"),path=resolve(directory,"package.json");const pkg=JSON.parse(await readFile(path,"utf8"));const previous=pkg.main;pkg.main="./injectra-start.cjs";
    await writeFile(path,json(pkg));
    const wrapper=`const {app}=require('electron'); const source=${JSON.stringify(proof)}; app.on('browser-window-created',(_event,win)=>{win.webContents.once('dom-ready',()=>win.webContents.executeJavaScript(source).catch(e=>console.error('injectra-proof:',String(e))));}); require(${JSON.stringify(previous)});\n`;
    await writeFile(resolve(directory,"injectra-start.cjs"),wrapper);
  } else if(recipe.kind === "asar-html") {
    const input=resolve(app,"Contents/Resources/app.asar"),output=resolve(run,"patched.asar");const html=asar.extractFile(input,recipe.html).toString("utf8");assert(/<\/head>/i.test(html));
    archive=await rewriteAsar(input,output,new Map([[recipe.html,Buffer.from(html.replace(/<\/head>/i,'<script src="./injectra-proof.js"></script></head>'))],[dirname(recipe.html)+"/injectra-proof.js",Buffer.from(proof)]]));
    await copyFile(output,input);execFileSync("/usr/bin/rsync",["-a",output+".unpacked/",input+".unpacked/"]);
  } else {
    const path=resolve(app,recipe.html),html=await readFile(path,"utf8");assert(/<\/head>/i.test(html));await writeFile(path,html.replace(/<\/head>/i,'<script src="./injectra-proof.js"></script></head>'));await writeFile(resolve(dirname(path),"injectra-proof.js"),proof);
  }
  const updatePlist = "import plistlib,sys\np=sys.argv[1]\nd=plistlib.load(open(p,'rb'))\nh=sys.argv[2]\nd['CFBundleDisplayName']='Injectra Test'\nif h: d['ElectronAsarIntegrity']['Resources/app.asar']['hash']=h\nplistlib.dump(d,open(p,'wb'))";
  execFileSync("/usr/bin/python3",["-c",updatePlist,resolve(app,"Contents/Info.plist"),archive?.headerHash??""]);
  if(arg === "claude") {
    const digest = JSON.parse(execFileSync("/usr/bin/python3",["-B",resolve(root,"tests/resource-patch/integrity-digest.py"),app],{encoding:"utf8"}));
    await writeFile(resolve(run,"integrity-digest.json"),json(digest));
    execFileSync("/usr/bin/codesign",["--force","--sign","-","--timestamp=none","--preserve-metadata=identifier,entitlements,flags",resolve(app,"Contents/Frameworks/Electron Framework.framework")]);
  }
  sign(app,true);execFileSync("/usr/bin/codesign",["--verify","--deep","--strict",app]);
  const plan={run,app,recipe:arg,...recipe,bundleId:sharedData?null:bundleId,dataPolicy:sharedData?"shared":"isolated",nonce,port,originalHashes,archive,proofHash:sha256(proof),localLibraryEntitlement:true,removedSharedAccountEntitlements:!sharedData};await writeFile(resolve(run,"plan.json"),json(plan));console.log(json({prepared:true,run,recipe:arg,noncePrefix:nonce.slice(0,10)}));
} else if(mode === "--run") {
  const run=resolve(arg),plan=JSON.parse(await readFile(resolve(run,"plan.json"),"utf8"));assert.equal(plan.run,run);assert.equal(plan.app,resolve(run,"Injectra-Test.app"));
  await access(resolve(run,"launch.json")).then(()=>{throw Error("Prepare a new run for retries");},e=>{if(e.code!=="ENOENT")throw e;});
  const processes=execFileSync("/bin/ps",["-axo","comm="],{encoding:"utf8"}).split("\n").map(line=>line.trim());
  assert(!processes.includes(resolve(plan.app,"Contents/MacOS",plan.executable)),"Test copy already running; preserve it and prepare a new run");
  const home=resolve(run,"home");await mkdir(resolve(home,"Library/Application Support"),{recursive:true});const events=[];
  const server=createServer((req,res)=>{const u=new URL(req.url,`http://127.0.0.1:${plan.port}`);if(u.pathname!==`/proof/${plan.nonce}`||u.searchParams.get("nonce")!==plan.nonce){res.writeHead(404).end();return;}const event={...Object.fromEntries(u.searchParams),peer:req.socket.remoteAddress};events.push(event);console.log(json({phase:event.phase,path:event.path}));res.writeHead(204).end();});server.listen(plan.port,"127.0.0.1");await once(server,"listening");
  const env={...process.env,HOME:home,CFFIXED_USER_HOME:home,XDG_CACHE_HOME:resolve(home,".cache"),XDG_CONFIG_HOME:resolve(home,".config"),XDG_DATA_HOME:resolve(home,".local/share")};for(const k of ["ELECTRON_RUN_AS_NODE","NODE_OPTIONS","DYLD_INSERT_LIBRARIES"])delete env[k];
  const command=[resolve(plan.app,"Contents/MacOS",plan.executable),`--user-data-dir=${resolve(home,"profile")}`,`--disk-cache-dir=${resolve(home,"chromium-cache")}`,"--use-mock-keychain","--no-first-run"];
  const fd=openSync(resolve(run,"startup.log"),"a",0o600),child=spawn(command[0],command.slice(1),{env,detached:true,stdio:["ignore",fd,fd]});child.unref();closeSync(fd);let exit=null;child.on("exit",(code,signal)=>{exit={code,signal};});await once(child,"spawn");await writeFile(resolve(run,"launch.json"),json({pid:child.pid,command,configuredHome:home,startedAt:new Date().toISOString()}));console.log(json({run,pid:child.pid,recipe:plan.recipe,noncePrefix:plan.nonce.slice(0,10)}));
  const start=Date.now();while(Date.now()-start<180000&&!events.some(e=>e.phase==="clicked")){if(exit&&Date.now()-start>4000)break;await new Promise(r=>setTimeout(r,250));}await new Promise(r=>server.close(r));
  const paths=[...new Set(events.map(e=>e.path))],passed=paths.some(path=>evaluateProof(events,{path,nonce:plan.nonce}).passed);for(const [path,h]of Object.entries(plan.originalHashes))assert.equal(await digest(resolve(plan.source,path)),h);
  await writeFile(resolve(run,"result.json"),json({passed,exit,events,pid:child.pid,criticalOriginalFilesUnchanged:true,automaticChildCleanup:false}));console.log(json({passed,exit,events:events.length,run}));process.exitCode=passed?0:1;
} else throw Error("Use --prepare <qq|tencentvideo|claude> or --run <prepared-run>");
