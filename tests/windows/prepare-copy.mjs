// Explicit experimental Windows Electron copy. No app-name branches; no installed-file mutations.
import fs from 'node:fs';
import fsp from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {parseArgs} from 'node:util';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {inspectMain,delegateSource} from '../../crates/preparation/helper/entry.mjs';
import {appendRewrite} from './append-asar.mjs';
import {updateIntegrity} from './pe-integrity.mjs';
import {copyTree} from './copy-tree.mjs';
const {values}=parseArgs({options:{exe:{type:'string'},output:{type:'string'},config:{type:'string'},package:{type:'string'},strategy:{type:'string'}}});
if(process.platform!=='win32')throw Error('Windows proof only');
const tools=createRequire(path.join(process.env.INJECTRA_WIN_TEST_TOOLS,'package.json'));
const pickle=tools('chromium-pickle-js'),asar=tools('@electron/asar'),acorn=tools('acorn'),pe=await import(pathToFileURL(tools.resolve('resedit')).href);
const sourceExe=fs.realpathSync(values.exe),source=path.dirname(sourceExe),destination=path.resolve(values.output);
if(fs.existsSync(destination)||destination.toLowerCase().startsWith(source.toLowerCase()+path.sep))throw Error('New destination outside source required');
const hash=b=>createHash('sha256').update(b).digest('hex');
let packageRelative=values.package?.replaceAll('\\','/');
if(!packageRelative){const choices=['resources/app','resources/app.asar'].filter(p=>fs.existsSync(path.join(source,p)));if(choices.length!==1)throw Error('Unsupported: provide an unambiguous package location');packageRelative=choices[0];}
if(path.posix.isAbsolute(packageRelative)||packageRelative.includes(':')||packageRelative.split('/').includes('..'))throw Error('Unsafe package location');
const layout=packageRelative.endsWith('.asar')?'asar':'directory',base=path.join(source,packageRelative);
const packageBytes=layout==='asar'?asar.extractFile(base,'package.json'):fs.readFileSync(path.join(base,'package.json'));
if(layout==='asar'?!!asar.getRawHeader(base).header.files?.['.injectra']:fs.existsSync(path.join(base,'.injectra')))throw Error('Reserved namespace exists');
const pkg=JSON.parse(packageBytes.toString('utf8').replace(/^\uFEFF/,''));if(pkg.main!==undefined&&typeof pkg.main!=='string')throw Error('Unsupported: non-string main');pkg.main=pkg.main?.replaceAll('\\','/');const entry=inspectMain(base,layout,pkg,{asar,acorn});if(values.strategy){if(!['cjs-insert','main-delegate'].includes(values.strategy))throw Error('Unknown entry strategy');if(values.strategy==='cjs-insert'&&entry.sourceText===undefined)throw Error('Unsupported: opaque source cannot use insertion');entry.strategy=values.strategy;}
const sourceHash={exe:hash(fs.readFileSync(sourceExe)),package:hash(packageBytes)};
if(layout==='asar')sourceHash.archive=hash(fs.readFileSync(base));
else if(entry.archive)sourceHash.archive=hash(fs.readFileSync(entry.archive));
const copiedLinks=await copyTree(source,destination,input=>{
 const relative=path.relative(source,input),top=relative.split(path.sep)[0];
 if(['Files','User Data','userdata','Crashpad','Cache','logs'].includes(top)&&!packageRelative.startsWith(top+'/'))return false;
 if(relative.endsWith('.log'))return false;
 return true;
});
const repo=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const replacements=new Map([['.injectra/config.json',fs.readFileSync(values.config)]]);
for(const name of ['bootstrap.cjs','evaluation.cjs'])replacements.set('.injectra/'+name,fs.readFileSync(path.join(repo,'crates/bootstrap/helper',name)));
if(entry.strategy==='main-delegate'){
 replacements.set('package.json',Buffer.from(JSON.stringify({...pkg,main:'./.injectra/entry.cjs'},null,2)));
 replacements.set('.injectra/entry.cjs',Buffer.from(delegateSource(entry.main)));
}else{
 const rel=path.posix.relative(path.posix.dirname(entry.main),'.injectra/bootstrap.cjs');
 const added='\n;module.require('+JSON.stringify(rel.startsWith('../')?rel:'./'+rel)+');\n';
 replacements.set(entry.main,Buffer.from(entry.sourceText.slice(0,entry.insert)+added+entry.sourceText.slice(entry.insert)));
}
let nativeChanges=[];
if(layout==='asar') {
 const archive=path.join(destination,packageRelative),old=hash(asar.getRawHeader(archive).headerString),temp=destination+'.rebuilt.asar';
 const next=await appendRewrite({asar,pickle,source:archive,destination:temp,replacements});await fsp.copyFile(temp,archive);
 if(fs.existsSync(temp+'.unpacked'))await fsp.cp(temp+'.unpacked',archive+'.unpacked',{recursive:true});
 const component=path.dirname(path.dirname(archive));
 for(const name of fs.readdirSync(component).filter(n=>/\.(exe|dll)$/i.test(n))) {
  const binary=path.join(component,name);let result;
  try{result=updateIntegrity({library:pe,binary,archive,before:old,after:next,ownedRoot:destination});}catch(error){if(/baseline|Unsupported/.test(error.message))throw error;continue;}
  if(result.updates)nativeChanges.push({path:path.relative(destination,binary),...result});
 }
}else{
 for(const [file,bytes]of replacements){const output=path.join(destination,packageRelative,file);await fsp.mkdir(path.dirname(output),{recursive:true});await fsp.writeFile(output,bytes);}
 if(entry.archive&&hash(fs.readFileSync(path.join(destination,path.relative(source,entry.archive))))!==sourceHash.archive)throw Error('Original nested archive changed');
}
if(hash(fs.readFileSync(sourceExe))!==sourceHash.exe)throw Error('Original executable changed');
const recheck=layout==='asar'?asar.extractFile(base,'package.json'):fs.readFileSync(path.join(base,'package.json'));
if(hash(recheck)!==sourceHash.package)throw Error('Original package changed');
const originalArchive=layout==='asar'?base:entry.archive;if(originalArchive&&hash(fs.readFileSync(originalArchive))!==sourceHash.archive)throw Error('Original archive baseline changed');
const result={source,app:destination,exe:path.join(destination,path.basename(sourceExe)),package:packageRelative,originalMain:entry.main,sourceHash,strategy:entry.strategy,nativeChanges,copiedLinks,productBackend:false};
await fsp.writeFile(destination+'.proof.json',JSON.stringify(result,null,2));console.log(JSON.stringify(result));
