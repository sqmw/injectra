import fs from 'node:fs';
import fsp from 'node:fs/promises';
import path from 'node:path';
import {Readable} from 'node:stream';
import {finished,pipeline} from 'node:stream/promises';
import {createHash} from 'node:crypto';
const hash=b=>createHash('sha256').update(b).digest('hex');
const relative=p=>p&&!path.posix.isAbsolute(p)&&!p.includes('\\')&&!p.includes('\0')&&path.posix.normalize(p)===p&&!p.startsWith('../');
function entries(node,prefix=''){return Object.entries(node.files??{}).flatMap(([name,n])=>{const p=prefix?prefix+'/'+name:name;if(!relative(p))throw Error('Unsafe archive path');if(n.link&&!relative(n.link))throw Error('Unsafe archive link');return [{path:p,node:n},...entries(n,p)];});}
export async function rebuild(asar,source,dest,replacements){
 const original=entries(asar.getRawHeader(source).header),remaining=new Map(replacements),streams=[];
 for(const {path:p,node:n}of original){
  if(n.files){streams.push({path:p,type:'directory',unpacked:n.unpacked});continue;}
  if(n.link){streams.push({path:p,type:'link',stat:{mode:0o120777},symlink:path.posix.relative(path.posix.dirname(p),n.link),unpacked:n.unpacked});continue;}
  const bytes=remaining.get(p);remaining.delete(p);
  const stat=n.unpacked?fs.statSync(path.join(source+'.unpacked',p)):{size:n.size,mode:n.executable?0o100755:0o100644};
  streams.push({path:p,type:'file',unpacked:n.unpacked,stat:{size:bytes?bytes.length:stat.size,mode:stat.mode},streamGenerator:()=>Readable.from(bytes??asar.extractFile(source,path.normalize(p)))});
 }
 // Explicitly add parents of the owned namespace before files.
 const known=new Set(original.filter(e=>e.node.files).map(e=>e.path));
 for(const [p,bytes]of remaining){let parent=path.posix.dirname(p);const parents=[];while(parent!=='.'&&!known.has(parent)){parents.unshift(parent);parent=path.posix.dirname(parent);}for(const d of parents){streams.push({path:d,type:'directory'});known.add(d);}streams.push({path:p,type:'file',stat:{size:bytes.length,mode:0o100644},streamGenerator:()=>Readable.from(bytes)});}
 const output=await asar.createPackageFromStreams(dest,streams);await finished(output);asar.uncache(dest);
 for(const {path:p,node:n}of original){const got=asar.statFile(dest,path.normalize(p),false);for(const flag of ['unpacked','link','executable'])if(got[flag]!==n[flag])throw Error('Archive state changed: '+p+': '+flag);if(!n.files&&!n.link){const wanted=replacements.get(p)??asar.extractFile(source,path.normalize(p));if(hash(asar.extractFile(dest,path.normalize(p)))!==hash(wanted))throw Error('Archive bytes mismatch: '+p);}}
 for(const [p,b]of replacements)if(hash(asar.extractFile(dest,path.normalize(p)))!==hash(b))throw Error('Inserted bytes mismatch: '+p);
 return hash(asar.getRawHeader(dest).headerString);
}
// Preserve vendor archive payload and opaque metadata; replace only declared owned entries.
export async function appendRewrite({asar,pickle,source,destination,replacements}) {
 const raw=asar.getRawHeader(source),header=JSON.parse(raw.headerString),packedLength=fs.statSync(source).size-8-raw.headerSize;
 if(packedLength<0)throw Error('Invalid ASAR header boundary');
 const appended=[];let offset=packedLength;
 if(fs.existsSync(source+'.unpacked'))await fsp.cp(source+'.unpacked',destination+'.unpacked',{recursive:true,dereference:false});
 for(const [name,bytes]of replacements) {
  const parts=name.split('/');if(parts.some(p=>!p||p==='.'||p==='..'||p.includes('\\')||p.includes('\0')))throw Error('Unsafe replacement path');
  let parent=header;
  for(const part of parts.slice(0,-1)){
   parent.files??={};if(!Object.hasOwn(parent.files,part))Object.defineProperty(parent.files,part,{value:{files:{}},enumerable:true,writable:true,configurable:true});
   parent=parent.files[part];if(!parent.files||parent.link)throw Error('Replacement parent is not a directory');
  }
  parent.files??={};const leaf=parts.at(-1),old=Object.hasOwn(parent.files,leaf)?parent.files[leaf]:{};
  if(old.files||old.link)throw Error('Replacement is not a regular file');
  const blockSize=old.integrity?.blockSize??4194304,blocks=[];if(!Number.isSafeInteger(blockSize)||blockSize<1)throw Error('Invalid integrity block size');
  for(let i=0;i<bytes.length;i+=blockSize)blocks.push(hash(bytes.subarray(i,i+blockSize)));
  const node={...old,size:bytes.length,integrity:{algorithm:'SHA256',hash:hash(bytes),blockSize,blocks}};
  if(old.unpacked){const output=path.join(destination+'.unpacked',...parts);await fsp.mkdir(path.dirname(output),{recursive:true});await fsp.writeFile(output,bytes);}
  else {node.offset=String(offset);offset+=bytes.length;appended.push(bytes);}
  Object.defineProperty(parent.files,leaf,{value:node,enumerable:true,writable:true,configurable:true});
 }
 const text=JSON.stringify(header),head=pickle.createEmpty();head.writeString(text);const headBytes=head.toBuffer(),size=pickle.createEmpty();size.writeUInt32(headBytes.length);
 const out=fs.createWriteStream(destination);out.write(size.toBuffer());out.write(headBytes);
 await pipeline(fs.createReadStream(source,{start:8+raw.headerSize}),out,{end:false});for(const bytes of appended)out.write(bytes);out.end();await finished(out);asar.uncache(destination);
 for(const [name,bytes]of replacements){if(hash(asar.extractFile(destination,path.normalize(name)))!==hash(bytes))throw Error('Appended replacement mismatch');}
 return hash(text);
}
