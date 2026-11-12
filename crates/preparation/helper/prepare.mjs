import fs from 'node:fs';
import {inspectMain,delegateSource} from './entry.mjs';
import fsp from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { createRequire } from 'node:module';
import { execFileSync } from 'node:child_process';
import { Readable } from 'node:stream';
import { finished } from 'node:stream/promises';
import { fileURLToPath } from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
const require=createRequire(path.join(process.env.INJECTRA_PREPARATION_TOOLS,'package.json'));
const asar=require('@electron/asar'),acorn=require('acorn');
const input=JSON.parse(fs.readFileSync(0,'utf8'));
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const fileHash=p=>hash(fs.readFileSync(p));
const relative=p=>typeof p==='string'&&p.length>0&&!path.posix.isAbsolute(p)&&!p.includes('\\')&&!p.includes('\0')&&path.posix.normalize(p)===p&&!p.startsWith('../');
function platform(op,app,payload={}){try {const r=JSON.parse(execFileSync('/usr/bin/python3',[path.join(here,'platform.py'),op,app],{input:JSON.stringify(payload),encoding:'utf8',maxBuffer:8*1024*1024}));if(!r.ok)throw Error(r.message);return r.value;}catch(e){if(e.stdout){try{throw Error(JSON.parse(String(e.stdout)).message);}catch(parsed){if(parsed instanceof SyntaxError)throw e;throw parsed;}}throw e;}}
function entries(node,prefix=''){return Object.entries(node.files??{}).flatMap(([name,n])=>{const p=prefix?prefix+'/'+name:name;if(!relative(p))throw Error('Unsupported: unsafe archive path');if(n.link&&!relative(n.link))throw Error('Unsupported: unsafe archive symlink');return [{path:p,node:n},...entries(n,p)];});}
function entryData(base,layout,file){if(!relative(file))throw Error('Unsupported: unsafe package entry');if(layout==='asar')return asar.extractFile(base,file);const target=fs.realpathSync(path.join(base,file));if(!target.startsWith(fs.realpathSync(base)+path.sep))throw Error('Unsupported: entry resolves outside package');return fs.readFileSync(target);}
function probe(source){
 const app=fs.realpathSync(source),description=platform('describe',app),resources=path.join(app,'Contents/Resources');
 const hasAsar=fs.existsSync(path.join(resources,'app.asar')),hasDirectory=fs.existsSync(path.join(resources,'app/package.json'));
 if(hasAsar===hasDirectory)throw Error('Unsupported: expected exactly one app.asar or directory package');
 const layout=hasAsar?'asar':'directory',packageRoot=hasAsar?'Contents/Resources/app.asar':'Contents/Resources/app',base=path.join(app,packageRoot);
 if(hasAsar){const header=asar.getRawHeader(base).header;entries(header);if(header.files?.['.injectra'])throw Error('Unsupported: package already contains reserved .injectra namespace');}else if(fs.existsSync(path.join(base,'.injectra')))throw Error('Unsupported: package already contains reserved .injectra namespace');
 const packageBytes=entryData(base,layout,'package.json'),pkg=JSON.parse(packageBytes.toString('utf8')),packageHash=hash(packageBytes);
 const entry=inspectMain(base,layout,pkg,{asar,acorn}),main=entry.main,bytes=entry.bytes;
 const critical=['Contents/Info.plist','Contents/MacOS/'+description.info.CFBundleExecutable,'Contents/_CodeSignature/CodeResources',layout==='asar'?packageRoot:packageRoot+'/package.json'];
 const baseline=Object.fromEntries(critical.map(p=>[p,fileHash(path.join(app,p))]));baseline['entry:'+main]=hash(bytes);if(entry.archive)baseline[path.relative(app,entry.archive)]=fileHash(entry.archive);
 const framework=path.join(app,description.electron_framework.executable);
 if(!fs.existsSync(framework))throw Error('Unsupported: Electron Framework was not found');
 const digest=fs.readFileSync(framework).includes(Buffer.from('AGbevlPCksUGKNL8TSn7wGmJEuJsXb2A'));
 return {public:{source_app:app,executable:path.join(app,'Contents/MacOS',description.info.CFBundleExecutable),bundle_id:description.info.CFBundleIdentifier,version:description.info.CFBundleShortVersionString,layout,package_root:packageRoot,main_entry:main,sandboxed:description.sandboxed,baseline,integrity_digest:digest,entry_strategy:entry.strategy},description,...entry,pkg,packageHash};
}
async function rebuild(source,dest,replacements){
 const original=entries(asar.getRawHeader(source).header),remaining=new Map(replacements),streams=[];
 for(const {path:p,node:n}of original){
  if(n.files){streams.push({path:p,type:'directory',unpacked:n.unpacked});continue;}
  if(n.link){streams.push({path:p,type:'link',stat:{mode:0o120777},symlink:path.posix.relative(path.posix.dirname(p),n.link),unpacked:n.unpacked});continue;}
  const bytes=remaining.get(p);remaining.delete(p);
  const stat=n.unpacked?fs.statSync(path.join(source+'.unpacked',p)):{size:n.size,mode:n.executable?0o100755:0o100644};
  streams.push({path:p,type:'file',unpacked:n.unpacked,stat:{size:bytes?bytes.length:stat.size,mode:stat.mode},streamGenerator:()=>Readable.from(bytes??asar.extractFile(source,p))});
 }
 // Explicitly add parents of the owned namespace before files.
 const known=new Set(original.filter(e=>e.node.files).map(e=>e.path));
 for(const [p,bytes]of remaining){let parent=path.posix.dirname(p);const parents=[];while(parent!=='.'&&!known.has(parent)){parents.unshift(parent);parent=path.posix.dirname(parent);}for(const d of parents){streams.push({path:d,type:'directory'});known.add(d);}streams.push({path:p,type:'file',stat:{size:bytes.length,mode:0o100644},streamGenerator:()=>Readable.from(bytes)});}
 const output=await asar.createPackageFromStreams(dest,streams);await finished(output);asar.uncache(dest);
 for(const {path:p,node:n}of original){const got=asar.statFile(dest,p,false);for(const flag of ['unpacked','link','executable'])if(got[flag]!==n[flag])throw Error('Archive state changed: '+p+': '+flag);if(!n.files&&!n.link){const wanted=replacements.get(p)??asar.extractFile(source,p);if(hash(asar.extractFile(dest,p))!==hash(wanted))throw Error('Archive bytes mismatch: '+p);}}
 for(const [p,b]of replacements)if(hash(asar.extractFile(dest,p))!==hash(b))throw Error('Inserted bytes mismatch: '+p);
 return hash(asar.getRawHeader(dest).headerString);
}
async function copySource(probed,dest){
 const source=probed.public.source_app,root=path.join(source,'Contents');
 const allowed=new Set(probed.description.sealed.map(p=>'Contents/'+p));
 for(const p of ['Contents/Info.plist','Contents/_CodeSignature/CodeResources','Contents/PkgInfo','Contents/MacOS/'+probed.description.info.CFBundleExecutable])allowed.add(p);
 const nested=[]; // Nested code seals were expanded by the platform descriptor.
 const include=relative=>!relative||allowed.has(relative)||relative.startsWith('Contents/_CodeSignature/')||nested.some(n=>relative.startsWith(n))||[...allowed].some(p=>p.startsWith(relative+'/'))||nested.some(p=>p.startsWith(relative+'/'));
 const excluded=[];
 await fsp.cp(source,dest,{recursive:true,dereference:false,verbatimSymlinks:true,preserveTimestamps:true,filter:(from)=>{
  const rel=path.relative(source,from);const keep=include(rel);if(!keep){excluded.push(rel);return false;}if(fs.lstatSync(from).isSymbolicLink()&&!fs.realpathSync(from).startsWith(source+path.sep))throw Error('Unsupported: source symlink escapes application');return true;
 }});
 return excluded;
}
try{
 if(process.platform!=='darwin'||process.arch!=='arm64')throw Error('Unsupported: preparation requires macOS arm64');
 const p=probe(input.source_app);
 if(input.operation==='probe'){console.log(JSON.stringify({ok:true,value:p.public}));process.exit(0);}
 if(input.data_mode==='isolated'&&p.public.sandboxed)throw Error('Unsupported: sandbox container isolation requires a supported target profile; select explicitly approved shared data');
 if(input.operation==='plan'){console.log(JSON.stringify({ok:true,value:{probe:p.public,data_mode:input.data_mode,changes:[{path:p.public.package_root+'/'+(p.strategy==='main-delegate'?'package.json':p.public.main_entry),origin:'injectra',reason:p.strategy==='main-delegate'?'Delegate original main through host loader; preserve original entry bytes':'Load bootstrap after directives, preserving original CJS module identity',before:p.strategy==='main-delegate'?p.packageHash:p.public.baseline['entry:'+p.public.main_entry],after:null},{path:p.public.package_root+'/.injectra',origin:'injectra',reason:'Owned generic bootstrap and configuration',before:null,after:null}],signing_policy:'local ad-hoc with local-library entitlement; existing runtime and ASAR/fuses retained'}}));process.exit(0);}
 const dest=input.app,owner=JSON.parse(fs.readFileSync(path.join(path.dirname(dest),'ownership.json'),'utf8'));
 if(owner.owner!=='injectra.preparation.v1'||owner.app!==dest||owner.source_app!==p.public.source_app||path.resolve(dest)===p.public.source_app)throw Error('Invalid owned destination');
 if(fs.existsSync(dest))throw Error('Destination already exists');
 const excluded=await copySource(p,dest);
 const signing=platform('sign-nested',dest);
 const bootstrap=fs.readFileSync(path.join(here,'bootstrap.cjs'));
 const config=Buffer.from(JSON.stringify(input.config));
 const entry=p.public.main_entry;
 const requirePath=path.posix.relative(path.posix.dirname(entry),'.injectra/bootstrap.cjs');
 const addition='\n;module.require('+JSON.stringify(requirePath.startsWith('./')||requirePath.startsWith('../')?requirePath:'./'+requirePath)+');\n';
 const replacements=new Map([['.injectra/bootstrap.cjs',bootstrap],['.injectra/evaluation.cjs',fs.readFileSync(path.join(here,'evaluation.cjs'))],['.injectra/config.json',config]]);
 if(p.strategy==='main-delegate'){
  replacements.set('package.json',Buffer.from(JSON.stringify({...p.pkg,main:'./.injectra/entry.cjs'},null,2)+'\n'));
  replacements.set('.injectra/entry.cjs',Buffer.from(delegateSource(entry)));
 }else replacements.set(entry,Buffer.from(p.sourceText.slice(0,p.insert)+addition+p.sourceText.slice(p.insert)));
 let asarHash=null;
 if(p.public.layout==='asar'){
  const src=path.join(dest,p.public.package_root),rebuilt=path.join(path.dirname(dest),'rebuilt.asar');
  asarHash=await rebuild(src,rebuilt,replacements);await fsp.copyFile(rebuilt,src);
  if(fs.existsSync(rebuilt+'.unpacked'))await fsp.cp(rebuilt+'.unpacked',src+'.unpacked',{recursive:true,force:true,dereference:false,verbatimSymlinks:true});
 }else{for(const [file,bytes]of replacements){const out=path.join(dest,p.public.package_root,file);await fsp.mkdir(path.dirname(out),{recursive:true});await fsp.writeFile(out,bytes);}}
 const signed=platform('finish',dest,{asar_hash:asarHash});
 const again=probe(input.source_app).public;
 if(JSON.stringify(again.baseline)!==JSON.stringify(p.public.baseline))throw Error('Source application changed during preparation');
 const changes=[...replacements].map(([file,bytes])=>({path:p.public.package_root+'/'+file,origin:'injectra',reason:file===entry?'Bootstrap insertion after CJS directives':'Owned preparation resource',before:file==='package.json'?p.packageHash:file===entry?p.public.baseline['entry:'+entry]:null,after:hash(bytes)}));
 for(const signedPath of signing.signed)changes.push({path:signedPath,origin:'injectra',reason:'Local test signing of nested native code',before:null,after:null});
 changes.push({path:'Contents/MacOS/'+p.description.info.CFBundleExecutable,origin:'injectra',reason:'Local test signing of app executable',before:p.public.baseline['Contents/MacOS/'+p.description.info.CFBundleExecutable],after:fileHash(path.join(dest,'Contents/MacOS',p.description.info.CFBundleExecutable))});
 changes.push({path:'Contents/Info.plist',origin:'injectra',reason:'Derived identity and integrity metadata',before:p.public.baseline['Contents/Info.plist'],after:fileHash(path.join(dest,'Contents/Info.plist'))});
 changes.push({path:'Contents/_CodeSignature',origin:'injectra',reason:'Local signing of outer and nested code; does not preserve vendor identity',before:null,after:null});
 console.log(JSON.stringify({ok:true,value:{probe:p.public,changes,signing:signed,excluded_unsealed:excluded}}));
}catch(error){console.log(JSON.stringify({ok:false,kind:String(error.message).startsWith('Unsupported:')?'Unsupported':'Prepare',message:String(error.message)}));process.exitCode=1;}
