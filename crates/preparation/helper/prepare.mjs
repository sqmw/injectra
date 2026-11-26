import fs from 'node:fs';
import fsp from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import {createRequire} from 'node:module';
import {execFileSync} from 'node:child_process';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {inspectMain,delegateSource,esmDelegateSource} from './entry.mjs';
import {rebuild,appendRewrite} from './archive.mjs';
import {copyTree} from './copy_tree.mjs';
import {updateIntegrity,hasIntegrityResource} from './pe_integrity.mjs';

const here=path.dirname(fileURLToPath(import.meta.url));
const tools=createRequire(path.join(process.env.INJECTRA_PREPARATION_TOOLS,'package.json'));
const asar=tools('@electron/asar'),acorn=tools('acorn');
const input=JSON.parse(fs.readFileSync(0,'utf8'));
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const fileHash=p=>hash(fs.readFileSync(p));
const relative=p=>typeof p==='string'&&p.length>0&&!path.posix.isAbsolute(p)&&!p.includes('\\')&&!p.includes('\0')&&path.posix.normalize(p)===p&&!p.startsWith('../');
const samePath=(a,b)=>process.platform==='win32'?path.resolve(a).toLowerCase()===path.resolve(b).toLowerCase():path.resolve(a)===path.resolve(b);

function platformMac(op,app,payload={}){
 try{
  const r=JSON.parse(execFileSync('/usr/bin/python3',[path.join(here,'platform.py'),op,app],{input:JSON.stringify(payload),encoding:'utf8',maxBuffer:8*1024*1024}));
  if(!r.ok)throw Error(r.message);
  return r.value;
 }catch(e){
  if(e.stdout){try{throw Error(JSON.parse(String(e.stdout)).message);}catch(parsed){if(parsed instanceof SyntaxError)throw e;throw parsed;}}
  throw e;
 }
}
function entries(node,prefix=''){
 return Object.entries(node.files??{}).flatMap(([name,n])=>{
  const p=prefix?prefix+'/'+name:name;
  if(!relative(p))throw Error('Unsupported: unsafe archive path');
  if(n.link&&!relative(n.link))throw Error('Unsupported: unsafe archive symlink');
  return [{path:p,node:n},...entries(n,p)];
 });
}
function entryData(base,layout,file){
 if(!relative(file))throw Error('Unsupported: unsafe package entry');
 if(layout==='asar')return asar.extractFile(base,file);
 const target=fs.realpathSync(path.join(base,file));
 if(!target.startsWith(fs.realpathSync(base)+path.sep))throw Error('Unsupported: entry resolves outside package');
 return fs.readFileSync(target);
}

function buildReplacements(p,config){
 const bootstrap=fs.readFileSync(path.join(here,'bootstrap.cjs'));
 const evaluation=fs.readFileSync(path.join(here,'evaluation.cjs'));
 const configBytes=Buffer.from(JSON.stringify(config));
 const entry=p.public.main_entry;
 const replacements=new Map([['.injectra/bootstrap.cjs',bootstrap],['.injectra/evaluation.cjs',evaluation],['.injectra/config.json',configBytes]]);
 if(p.strategy==='main-delegate'){
  replacements.set('package.json',Buffer.from(JSON.stringify({...p.pkg,main:'./.injectra/entry.cjs'},null,2)+'\n'));
  replacements.set('.injectra/entry.cjs',Buffer.from(delegateSource(entry)));
 }else if(p.strategy==='esm-delegate'){
  replacements.set('package.json',Buffer.from(JSON.stringify({...p.pkg,main:'./.injectra/entry.mjs'},null,2)+'\n'));
  replacements.set('.injectra/entry.mjs',Buffer.from(esmDelegateSource(entry)));
 }else{
  const requirePath=path.posix.relative(path.posix.dirname(entry),'.injectra/bootstrap.cjs');
  const addition='\n;module.require('+JSON.stringify(requirePath.startsWith('./')||requirePath.startsWith('../')?requirePath:'./'+requirePath)+');\n';
  replacements.set(entry,Buffer.from(p.sourceText.slice(0,p.insert)+addition+p.sourceText.slice(p.insert)));
 }
 return replacements;
}

function planPayload(p,dataMode,signingPolicy){
 const isDelegate=p.strategy==='main-delegate'||p.strategy==='esm-delegate';
 const reason=p.strategy==='esm-delegate'
  ?'Delegate original ESM main through .injectra/entry.mjs; preserve original entry bytes'
  :p.strategy==='main-delegate'
  ?'Delegate original main through host loader; preserve original entry bytes'
  :'Load bootstrap after directives, preserving original CJS module identity';
 return {
  probe:p.public,
  data_mode:dataMode,
  changes:[
   {path:p.public.package_root+'/'+(isDelegate?'package.json':p.public.main_entry),origin:'injectra',reason,before:isDelegate?p.packageHash:p.public.baseline['entry:'+p.public.main_entry],after:null},
   {path:p.public.package_root+'/.injectra',origin:'injectra',reason:'Owned generic bootstrap and configuration',before:null,after:null}
  ],
  signing_policy:signingPolicy
 };
}

// --- macOS .app path ---
function probeMac(source){
 const app=fs.realpathSync(source),description=platformMac('describe',app),resources=path.join(app,'Contents/Resources');
 const hasAsar=fs.existsSync(path.join(resources,'app.asar')),hasDirectory=fs.existsSync(path.join(resources,'app/package.json'));
 if(hasAsar===hasDirectory)throw Error('Unsupported: expected exactly one app.asar or directory package');
 const layout=hasAsar?'asar':'directory',packageRoot=hasAsar?'Contents/Resources/app.asar':'Contents/Resources/app',base=path.join(app,packageRoot);
 if(hasAsar){const header=asar.getRawHeader(base).header;entries(header);if(header.files?.['.injectra'])throw Error('Unsupported: package already contains reserved .injectra namespace');}
 else if(fs.existsSync(path.join(base,'.injectra')))throw Error('Unsupported: package already contains reserved .injectra namespace');
 const packageBytes=entryData(base,layout,'package.json'),pkg=JSON.parse(packageBytes.toString('utf8').replace(/^\uFEFF/,'')),packageHash=hash(packageBytes);
 const entry=inspectMain(base,layout,pkg,{asar,acorn}),main=entry.main,bytes=entry.bytes;
 const critical=['Contents/Info.plist','Contents/MacOS/'+description.info.CFBundleExecutable,'Contents/_CodeSignature/CodeResources',layout==='asar'?packageRoot:packageRoot+'/package.json'];
 const baseline=Object.fromEntries(critical.map(p=>[p,fileHash(path.join(app,p))]));
 baseline['entry:'+main]=hash(bytes);
 if(entry.archive)baseline[path.relative(app,entry.archive)]=fileHash(entry.archive);
 const framework=path.join(app,description.electron_framework.executable);
 if(!fs.existsSync(framework))throw Error('Unsupported: Electron Framework was not found');
 const digest=fs.readFileSync(framework).includes(Buffer.from('AGbevlPCksUGKNL8TSn7wGmJEuJsXb2A'));
 return {public:{source_app:app,executable:path.join(app,'Contents/MacOS',description.info.CFBundleExecutable),bundle_id:description.info.CFBundleIdentifier,version:description.info.CFBundleShortVersionString,layout,package_root:packageRoot,main_entry:main,sandboxed:description.sandboxed,baseline,integrity_digest:digest,entry_strategy:entry.strategy},description,...entry,pkg,packageHash};
}

async function copySourceMac(probed,dest){
 const source=probed.public.source_app;
 const allowed=new Set(probed.description.sealed.map(p=>'Contents/'+p));
 for(const p of ['Contents/Info.plist','Contents/_CodeSignature/CodeResources','Contents/PkgInfo','Contents/MacOS/'+probed.description.info.CFBundleExecutable])allowed.add(p);
 const include=rel=>!rel||allowed.has(rel)||rel.startsWith('Contents/_CodeSignature/')||[...allowed].some(p=>p.startsWith(rel+'/'));
 const excluded=[];
 await fsp.cp(source,dest,{recursive:true,dereference:false,verbatimSymlinks:true,preserveTimestamps:true,filter:(from)=>{
  const rel=path.relative(source,from);const keep=include(rel);
  if(!keep){excluded.push(rel);return false;}
  if(fs.lstatSync(from).isSymbolicLink()&&!fs.realpathSync(from).startsWith(source+path.sep))throw Error('Unsupported: source symlink escapes application');
  return true;
 }});
 return excluded;
}

// --- Windows directory / executable path ---
function isAuxiliaryWinExe(name){
 const lower=name.toLowerCase();
 return lower.startsWith('unins')||lower.startsWith('uninstall')||lower==='update.exe'||lower==='squirrel.exe'||lower.endsWith('_executionstub.exe')||lower.includes('crashpad')||lower.includes('elevate');
}

function findWindowsPackages(rootDir){
 const found=[];
 const skipTop=new Set(['Files','User Data','userdata','Crashpad','Cache','logs']);
 function walk(dir,depth){
  for(const ent of fs.readdirSync(dir,{withFileTypes:true})){
   if(ent.isSymbolicLink())continue;
   if(depth===0&&skipTop.has(ent.name))continue;
   const full=path.join(dir,ent.name);
   if(ent.isFile()&&ent.name==='app.asar'){
    found.push({layout:'asar',base:full,packageRoot:path.relative(rootDir,full).split(path.sep).join('/')});
   }else if(ent.isDirectory()&&ent.name==='app'&&fs.existsSync(path.join(full,'package.json'))){
    found.push({layout:'directory',base:full,packageRoot:path.relative(rootDir,full).split(path.sep).join('/')});
   }else if(ent.isDirectory()&&depth<3){
    walk(full,depth+1);
   }
  }
 }
 walk(rootDir,0);
 return found;
}

function selectWindowsExe(rootDir,componentDir,explicitExe){
 if(explicitExe)return explicitExe;
 for(const dir of [componentDir,rootDir]){
  const exes=fs.readdirSync(dir,{withFileTypes:true})
   .filter(e=>e.isFile()&&/\.exe$/i.test(e.name))
   .map(e=>e.name).sort();
  const primary=exes.filter(n=>!isAuxiliaryWinExe(n));
  if(primary.length)return path.join(dir,primary[0]);
  if(exes.length)return path.join(dir,exes[0]);
 }
 throw Error('Unsupported: no Windows host executable found');
}

async function probeWindows(sourceInput){
 const resolved=fs.realpathSync(sourceInput);
 const stat=fs.statSync(resolved);
 const sourceRoot=stat.isFile()?path.dirname(resolved):resolved;
 const explicitExe=stat.isFile()?resolved:null;
 const packages=findWindowsPackages(sourceRoot);
 if(packages.length!==1)throw Error('Unsupported: expected exactly one app.asar or directory package under Windows target');
 const {layout,base,packageRoot}=packages[0];
 if(layout==='asar'){
  const header=asar.getRawHeader(base).header;
  entries(header);
  if(header.files?.['.injectra'])throw Error('Unsupported: package already contains reserved .injectra namespace');
 }else if(fs.existsSync(path.join(base,'.injectra'))){
  throw Error('Unsupported: package already contains reserved .injectra namespace');
 }
 const componentDir=path.dirname(path.dirname(base));
 const executable=selectWindowsExe(sourceRoot,componentDir,explicitExe);
 const relExe=path.relative(sourceRoot,executable).split(path.sep).join('/');
 if(!relExe||relExe.startsWith('..')||path.isAbsolute(relExe))throw Error('Unsupported: executable outside target root');
 const packageBytes=entryData(base,layout,'package.json');
 const pkg=JSON.parse(packageBytes.toString('utf8').replace(/^\uFEFF/,''));
 const packageHash=hash(packageBytes);
 const entry=inspectMain(base,layout,pkg,{asar,acorn});
 const main=entry.main,bytes=entry.bytes;
 const baseline={};
 baseline[relExe]=fileHash(executable);
 baseline[layout==='asar'?packageRoot:packageRoot+'/package.json']=fileHash(layout==='asar'?base:path.join(base,'package.json'));
 baseline['entry:'+main]=hash(bytes);
 if(entry.archive)baseline[path.relative(sourceRoot,entry.archive).split(path.sep).join('/')]=fileHash(entry.archive);
 const pe=await import(pathToFileURL(tools.resolve('resedit')).href);
 let digest=hasIntegrityResource({library:pe,binary:executable});
 if(!digest){
  for(const name of fs.readdirSync(componentDir).filter(n=>/\.(exe|dll)$/i.test(n)).slice(0,32)){
   if(hasIntegrityResource({library:pe,binary:path.join(componentDir,name)})){digest=true;break;}
  }
 }
 return {
  public:{
   source_app:sourceRoot,
   executable,
   bundle_id:typeof pkg.name==='string'?pkg.name:'',
   version:typeof pkg.version==='string'?pkg.version:'unknown',
   layout,
   package_root:packageRoot,
   main_entry:main,
   sandboxed:false,
   baseline,
   integrity_digest:digest,
   entry_strategy:entry.strategy
  },
  ...entry,
  pkg,
  packageHash,
  componentDir,
  relExe
 };
}

try{
 if(process.platform==='darwin'&&process.arch==='arm64'){
  const p=probeMac(input.source_app);
  if(input.operation==='probe'){console.log(JSON.stringify({ok:true,value:p.public}));process.exit(0);}
  if(input.data_mode==='isolated'&&p.public.sandboxed)throw Error('Unsupported: sandbox container isolation requires a supported target profile; select explicitly approved shared data');
  if(input.operation==='plan'){
   console.log(JSON.stringify({ok:true,value:planPayload(p,input.data_mode,'local ad-hoc with local-library entitlement; existing runtime and ASAR/fuses retained')}));
   process.exit(0);
  }
  const dest=input.app,owner=JSON.parse(fs.readFileSync(path.join(path.dirname(dest),'ownership.json'),'utf8'));
  if(owner.owner!=='injectra.preparation.v1'||!samePath(owner.app,dest)||!samePath(owner.source_app,p.public.source_app)||samePath(dest,p.public.source_app))throw Error('Invalid owned destination');
  if(fs.existsSync(dest))throw Error('Destination already exists');
  const excluded=await copySourceMac(p,dest);
  const signing=platformMac('sign-nested',dest);
  const replacements=buildReplacements(p,input.config);
  let asarHash=null;
  if(p.public.layout==='asar'){
   const src=path.join(dest,p.public.package_root),rebuilt=path.join(path.dirname(dest),'rebuilt.asar');
   asarHash=await rebuild(asar,src,rebuilt,replacements);await fsp.copyFile(rebuilt,src);
   if(fs.existsSync(rebuilt+'.unpacked'))await fsp.cp(rebuilt+'.unpacked',src+'.unpacked',{recursive:true,force:true,dereference:false,verbatimSymlinks:true});
  }else{
   for(const [file,bytes]of replacements){const out=path.join(dest,p.public.package_root,file);await fsp.mkdir(path.dirname(out),{recursive:true});await fsp.writeFile(out,bytes);}
  }
  const signed=platformMac('finish',dest,{asar_hash:asarHash});
  const again=probeMac(input.source_app).public;
  if(JSON.stringify(again.baseline)!==JSON.stringify(p.public.baseline))throw Error('Source application changed during preparation');
  const entry=p.public.main_entry;
  const changes=[...replacements].map(([file,bytes])=>({path:p.public.package_root+'/'+file,origin:'injectra',reason:file===entry?'Bootstrap insertion after CJS directives':'Owned preparation resource',before:file==='package.json'?p.packageHash:file===entry?p.public.baseline['entry:'+entry]:null,after:hash(bytes)}));
  for(const signedPath of signing.signed)changes.push({path:signedPath,origin:'injectra',reason:'Local test signing of nested native code',before:null,after:null});
  changes.push({path:'Contents/MacOS/'+p.description.info.CFBundleExecutable,origin:'injectra',reason:'Local test signing of app executable',before:p.public.baseline['Contents/MacOS/'+p.description.info.CFBundleExecutable],after:fileHash(path.join(dest,'Contents/MacOS',p.description.info.CFBundleExecutable))});
  changes.push({path:'Contents/Info.plist',origin:'injectra',reason:'Derived identity and integrity metadata',before:p.public.baseline['Contents/Info.plist'],after:fileHash(path.join(dest,'Contents/Info.plist'))});
  changes.push({path:'Contents/_CodeSignature',origin:'injectra',reason:'Local signing of outer and nested code; does not preserve vendor identity',before:null,after:null});
  console.log(JSON.stringify({ok:true,value:{probe:p.public,changes,signing:signed,excluded_unsealed:excluded}}));
 }else if(process.platform==='win32'&&['x64','ia32'].includes(process.arch)){
  const p=await probeWindows(input.source_app);
  if(input.operation==='probe'){console.log(JSON.stringify({ok:true,value:p.public}));process.exit(0);}
  if(input.operation==='plan'){
   console.log(JSON.stringify({ok:true,value:planPayload(p,input.data_mode,'Windows derived copy with PE INTEGRITY/ELECTRONASAR update when present; Authenticode signature on mutated binaries is not preserved')}));
   process.exit(0);
  }
  const dest=input.app,owner=JSON.parse(fs.readFileSync(path.join(path.dirname(dest),'ownership.json'),'utf8'));
  if(owner.owner!=='injectra.preparation.v1'||!samePath(owner.app,dest)||!samePath(owner.source_app,p.public.source_app)||samePath(dest,p.public.source_app))throw Error('Invalid owned destination');
  if(fs.existsSync(dest))throw Error('Destination already exists');
  const excluded=[];
  await copyTree(p.public.source_app,dest,item=>{
   const rel=path.relative(p.public.source_app,item);
   if(!rel)return true;
   const top=rel.split(path.sep)[0];
   if(['Files','User Data','userdata','Crashpad','Cache','logs'].includes(top)&&!p.public.package_root.startsWith(top+'/')){excluded.push(rel);return false;}
   if(rel.toLowerCase().endsWith('.log')){excluded.push(rel);return false;}
   return true;
  });
  const replacements=buildReplacements(p,input.config);
  const pickle=tools('chromium-pickle-js');
  const pe=await import(pathToFileURL(tools.resolve('resedit')).href);
  const nativeChanges=[];
  if(p.public.layout==='asar'){
   const archive=path.join(dest,p.public.package_root);
   const oldHash=hash(asar.getRawHeader(archive).headerString);
   const rebuilt=path.join(path.dirname(dest),'rebuilt.asar');
   const nextHash=await appendRewrite({asar,pickle,source:archive,destination:rebuilt,replacements});
   await fsp.copyFile(rebuilt,archive);
   if(fs.existsSync(rebuilt+'.unpacked'))await fsp.cp(rebuilt+'.unpacked',archive+'.unpacked',{recursive:true,force:true,dereference:false});
   const destComponent=path.dirname(path.dirname(archive));
   for(const name of fs.readdirSync(destComponent).filter(n=>/\.(exe|dll)$/i.test(n))){
    const binary=path.join(destComponent,name);
    let result;
    try{result=updateIntegrity({library:pe,binary,archive,before:oldHash,after:nextHash,ownedRoot:dest});}
    catch(error){if(/baseline|Unsupported/.test(error.message))throw error;continue;}
    if(result.updates)nativeChanges.push({path:path.relative(dest,binary).split(path.sep).join('/'),...result});
   }
  }else{
   for(const [file,bytes]of replacements){const out=path.join(dest,p.public.package_root,file);await fsp.mkdir(path.dirname(out),{recursive:true});await fsp.writeFile(out,bytes);}
   if(p.archive){
    const relArch=path.relative(p.public.source_app,p.archive).split(path.sep).join('/');
    if(fileHash(path.join(dest,relArch))!==p.public.baseline[relArch])throw Error('Original nested archive changed');
   }
  }
  const again=(await probeWindows(input.source_app)).public;
  if(JSON.stringify(again.baseline)!==JSON.stringify(p.public.baseline))throw Error('Source application changed during preparation');
  const entry=p.public.main_entry;
  const changes=[...replacements].map(([file,bytes])=>({path:p.public.package_root+'/'+file,origin:'injectra',reason:file===entry?'Bootstrap insertion after CJS directives':'Owned preparation resource',before:file==='package.json'?p.packageHash:file===entry?p.public.baseline['entry:'+entry]:null,after:hash(bytes)}));
  for(const nc of nativeChanges){
   changes.push({path:nc.path,origin:'injectra',reason:'Updated Windows PE INTEGRITY/ELECTRONASAR header digest',before:p.public.baseline[nc.path]??null,after:fileHash(path.join(dest,nc.path))});
  }
  console.log(JSON.stringify({ok:true,value:{probe:p.public,changes,signing:{signature_preserved:nativeChanges.length===0,pe_updates:nativeChanges.length},excluded_unsealed:excluded}}));
 }else{
  throw Error('Unsupported: preparation requires macOS arm64 or Windows x64/x86');
 }
}catch(error){
 console.log(JSON.stringify({ok:false,kind:String(error.message).startsWith('Unsupported:')?'Unsupported':'Prepare',message:String(error.message)}));
 process.exitCode=1;
}
