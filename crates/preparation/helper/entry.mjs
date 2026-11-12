// Electron/Node package-entry semantics. No application names or user payloads.
import fs from 'node:fs';
import path from 'node:path';
export function inspectMain(base,layout,pkg,{asar,acorn}) {
 let main=pkg.main??'index.js';if(typeof main!=='string')throw Error('Unsupported: main must be a string');main=main.replace(/^\.\//,'');
 const safe=p=>p&&!path.posix.isAbsolute(p)&&!p.includes('\\')&&!p.includes('\0')&&path.posix.normalize(p)===p&&!p.startsWith('../');
 if(!safe(main))throw Error('Unsupported: main escapes package');
 const read=file=>{
  if(layout==='asar')return {bytes:asar.extractFile(base,file)};
  const parts=file.split('/');
  for(let i=0;i<parts.length-1;i++) {
   const archive=path.join(base,...parts.slice(0,i+1));
   if(parts[i].endsWith('.asar')&&fs.existsSync(archive)&&fs.statSync(archive).isFile()) {
    const real=fs.realpathSync(archive);if(!real.startsWith(fs.realpathSync(base)+path.sep))throw Error('Unsupported: nested archive leaves package');
    const inside=parts.slice(i+1).join('/');const node=asar.statFile(real,inside,false);
    if(!node||node.files||node.link||node.size>16*1024*1024)throw Error('Unsupported: invalid nested main');
    return {bytes:asar.extractFile(real,inside),archive:real};
   }
  }
  const real=fs.realpathSync(path.join(base,file));if(!real.startsWith(fs.realpathSync(base)+path.sep))throw Error('Unsupported: entry resolves outside package');
  if(fs.statSync(real).size>16*1024*1024)throw Error('Unsupported: entry exceeds parser bound');
  return {bytes:fs.readFileSync(real)};
 };
 let found;
 for(const candidate of path.posix.extname(main)?[main]:[main,main+'.js',main+'.cjs',main+'/index.js']) {
  try {found=read(candidate);main=candidate;break;}catch(error){if(String(error.message).startsWith('Unsupported:'))throw error;}
 }
 if(!found)throw Error('Unsupported: main is not readable in its package layout');
 if(!['.js','.cjs'].includes(path.posix.extname(main))||(pkg.type==='module'&&!main.endsWith('.cjs')))throw Error('Unsupported: ESM or non-JavaScript main');
 if(found.bytes.length>16*1024*1024)throw Error('Unsupported: entry exceeds parser bound');
 // Embedded archives are deliberately kept byte-for-byte; the host owns their loader.
 if(found.archive)return {main,...found,strategy:'main-delegate'};
 let sourceText;try {sourceText=new TextDecoder('utf8',{fatal:true,ignoreBOM:true}).decode(found.bytes);}catch{return {main,...found,strategy:'main-delegate'};}
 const bom=sourceText.charCodeAt(0)===0xfeff?1:0,parseText=sourceText.slice(bom);
 let ast;try {ast=acorn.parse(parseText,{ecmaVersion:'latest',sourceType:'script',allowHashBang:true,allowReturnOutsideFunction:true});}catch(error){throw Error('Unsupported: main is not parseable CommonJS: '+error.message);}
 if(ast.body.some(n=>n.type==='FunctionDeclaration'&&n.id?.name==='module'))throw Error('Unsupported: main shadows the module entry API');
 let insert=parseText.startsWith('#!')?bom+parseText.indexOf('\n')+1:bom;
 for(const statement of ast.body){if(!statement.directive)break;insert=bom+statement.end;}
 return {main,...found,strategy:'cjs-insert',sourceText,insert};
}
export function delegateSource(main) {
 // isMain=true makes Node establish the ORIGINAL module's id, require.main and
 // process.mainModule before its body runs. Plain require(main) would lose these.
 return "'use strict';\nrequire('./bootstrap.cjs');\nconst Module=require('node:module'),path=require('node:path'),fs=require('node:fs');\nconst filename=path.resolve(__dirname,'..',"+JSON.stringify(main)+");if(process.argv[1]){try{if(fs.realpathSync(process.argv[1])===__filename)process.argv[1]=filename;}catch{}}module.exports=Module._load(filename,null,true);\n";
}
