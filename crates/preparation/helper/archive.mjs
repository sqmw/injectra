// Zero-dependency Node built-in ASAR reader, directory packer, and append rewriter.
// Formal CLI `prepare` delegates macOS rebuild and Windows appendRewrite directly to pure-Rust `crates/preparation/src/asar.rs`.
import fs from 'node:fs';
import fsp from 'node:fs/promises';
import path from 'node:path';
import {finished,pipeline} from 'node:stream/promises';
import {createHash} from 'node:crypto';

const DEFAULT_BLOCK_SIZE=4*1024*1024;
const MAX_HEADER_BYTES=16*1024*1024;
const hash=b=>createHash('sha256').update(b).digest('hex');
const normRel=p=>String(p??'').replaceAll('\\','/').replace(/^\.\//,'');
const relative=p=>p&&!path.posix.isAbsolute(p)&&!p.includes('\\')&&!p.includes('\0')&&path.posix.normalize(p)===p&&!p.startsWith('../');

export function encodePickleHeader(text){
 const textBuf=Buffer.from(text,'utf8');
 const pad=(4-(textBuf.length%4))%4;
 const stringObjSize=4+textBuf.length+pad;
 const headerSize=stringObjSize+4;
 const out=Buffer.alloc(8+headerSize);
 out.writeUInt32LE(4,0);
 out.writeUInt32LE(headerSize,4);
 out.writeUInt32LE(stringObjSize,8);
 out.writeUInt32LE(textBuf.length,12);
 textBuf.copy(out,16);
 return out;
}

export function getRawHeader(asarPath){
 const fd=fs.openSync(asarPath,'r');
 try{
  const prefix=Buffer.alloc(16);
  if(fs.readSync(fd,prefix,0,16,0)!==16)throw Error('Invalid ASAR header prefix');
  const headerSize=prefix.readUInt32LE(4);
  const jsonLen=prefix.readUInt32LE(12);
  if(jsonLen===0||jsonLen>MAX_HEADER_BYTES||headerSize<jsonLen+8)throw Error('Invalid ASAR pickle header boundary');
  const jsonBuf=Buffer.alloc(jsonLen);
  if(fs.readSync(fd,jsonBuf,0,jsonLen,16)!==jsonLen)throw Error('Truncated ASAR header JSON');
  const headerString=jsonBuf.toString('utf8');
  return {headerSize,headerString,header:JSON.parse(headerString)};
 }finally{
  fs.closeSync(fd);
 }
}

export function statFile(asarPath,relPath){
 const rel=normRel(relPath);
 if(!relative(rel))throw Error('Unsafe archive path');
 const {header}=getRawHeader(asarPath);
 let node=header;
 for(const part of rel.split('/')){
  if(!node?.files||!Object.hasOwn(node.files,part))return null;
  node=node.files[part];
 }
 return node;
}

export function extractFile(asarPath,relPath){
 const rel=normRel(relPath);
 if(!relative(rel))throw Error('Unsafe archive path');
 const raw=getRawHeader(asarPath);
 let node=raw.header;
 for(const part of rel.split('/')){
  if(!node?.files||!Object.hasOwn(node.files,part))throw Error('ASAR entry not found: '+rel);
  node=node.files[part];
 }
 if(node.files||node.link)throw Error('ASAR entry is not a regular file: '+rel);
 if(node.unpacked){
  const rootReal=fs.realpathSync(asarPath+'.unpacked');
  const candidate=fs.realpathSync(path.join(asarPath+'.unpacked',...rel.split('/')));
  if(!candidate.startsWith(rootReal+path.sep)&&candidate!==rootReal)throw Error('Unsupported: unpacked entry escapes directory');
  return fs.readFileSync(candidate);
 }
 const size=Number(node.size),offset=Number(node.offset);
 if(!Number.isSafeInteger(size)||size<0||!Number.isSafeInteger(offset)||offset<0)throw Error('Invalid ASAR entry bounds');
 const out=Buffer.alloc(size);
 if(size===0)return out;
 const fd=fs.openSync(asarPath,'r');
 try{
  const read=fs.readSync(fd,out,0,size,8+raw.headerSize+offset);
  if(read!==size)throw Error('Truncated ASAR entry payload');
  return out;
 }finally{
  fs.closeSync(fd);
 }
}

function fileIntegrity(bytes,blockSize=DEFAULT_BLOCK_SIZE){
 const blocks=[];
 let offset=0;
 while(offset+blockSize<=bytes.length){
  blocks.push(hash(bytes.subarray(offset,offset+blockSize)));
  offset+=blockSize;
 }
 blocks.push(hash(bytes.subarray(offset)));
 return {algorithm:'SHA256',hash:hash(bytes),blockSize,blocks};
}

export async function createPackage(srcDir,destAsar){
 const rootReal=fs.realpathSync(srcDir);
 const chunks=[];
 let offset=0;
 function walk(dir){
  const files={};
  const entries=fs.readdirSync(dir,{withFileTypes:true}).sort((a,b)=>a.name.localeCompare(b.name));
  for(const ent of entries){
   const full=path.join(dir,ent.name);
   const lstat=fs.lstatSync(full);
   if(lstat.isSymbolicLink()){
    const target=fs.realpathSync(full);
    if(!target.startsWith(rootReal+path.sep)&&target!==rootReal)throw Error('Unsupported: symlink escapes source root');
    const rel=path.relative(rootReal,target).split(path.sep).join('/');
    files[ent.name]={link:rel};
   }else if(lstat.isDirectory()){
    files[ent.name]=walk(full);
   }else if(lstat.isFile()){
    const bytes=fs.readFileSync(full);
    const node={size:bytes.length,offset:String(offset),integrity:fileIntegrity(bytes)};
    if(process.platform!=='win32'&&(lstat.mode&0o111)!==0)node.executable=true;
    files[ent.name]=node;
    chunks.push(bytes);
    offset+=bytes.length;
   }
  }
  return {files};
 }
 const header=walk(rootReal);
 const headerString=JSON.stringify(header);
 await fsp.mkdir(path.dirname(destAsar),{recursive:true});
 const out=fs.createWriteStream(destAsar);
 out.write(encodePickleHeader(headerString));
 for(const chunk of chunks)out.write(chunk);
 out.end();
 await finished(out);
 return hash(headerString);
}

export const builtinAsar={getRawHeader,statFile,extractFile,createPackage,uncache(){}};

// Preserve vendor archive payload and opaque metadata; replace only declared owned entries.
export async function appendRewrite({asar=builtinAsar,source,destination,replacements}){
 const raw=asar.getRawHeader(source),header=JSON.parse(raw.headerString),packedLength=fs.statSync(source).size-8-raw.headerSize;
 if(packedLength<0)throw Error('Invalid ASAR header boundary');
 const appended=[];let offset=packedLength;
 if(fs.existsSync(source+'.unpacked'))await fsp.cp(source+'.unpacked',destination+'.unpacked',{recursive:true,dereference:false});
 for(const [name,bytes]of replacements){
  const parts=name.split('/');if(parts.some(p=>!p||p==='.'||p==='..'||p.includes('\\')||p.includes('\0')))throw Error('Unsafe replacement path');
  let parent=header;
  for(const part of parts.slice(0,-1)){
   parent.files??={};if(!Object.hasOwn(parent.files,part))Object.defineProperty(parent.files,part,{value:{files:{}},enumerable:true,writable:true,configurable:true});
   parent=parent.files[part];if(!parent.files||parent.link)throw Error('Replacement parent is not a directory');
  }
  parent.files??={};const leaf=parts.at(-1),old=Object.hasOwn(parent.files,leaf)?parent.files[leaf]:{};
  if(old.files||old.link)throw Error('Replacement is not a regular file');
  const blockSize=old.integrity?.blockSize??DEFAULT_BLOCK_SIZE;if(!Number.isSafeInteger(blockSize)||blockSize<1)throw Error('Invalid integrity block size');
  const node={...old,size:bytes.length,integrity:fileIntegrity(bytes,blockSize)};
  if(old.unpacked){const output=path.join(destination+'.unpacked',...parts);await fsp.mkdir(path.dirname(output),{recursive:true});await fsp.writeFile(output,bytes);}
  else {node.offset=String(offset);offset+=bytes.length;appended.push(bytes);}
  Object.defineProperty(parent.files,leaf,{value:node,enumerable:true,writable:true,configurable:true});
 }
 const text=JSON.stringify(header);
 const out=fs.createWriteStream(destination);out.write(encodePickleHeader(text));
 await pipeline(fs.createReadStream(source,{start:8+raw.headerSize}),out,{end:false});for(const bytes of appended)out.write(bytes);out.end();await finished(out);
 for(const [name,bytes]of replacements){if(hash(asar.extractFile(destination,path.normalize(name)))!==hash(bytes))throw Error('Appended replacement mismatch');}
 return hash(text);
}
