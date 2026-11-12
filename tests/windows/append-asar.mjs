// Preserve vendor archive payload and opaque metadata; replace only declared owned entries.
import fs from 'node:fs';
import fsp from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {pipeline,finished} from 'node:stream/promises';
const digest=b=>createHash('sha256').update(b).digest('hex');
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
  for(let i=0;i<bytes.length;i+=blockSize)blocks.push(digest(bytes.subarray(i,i+blockSize)));
  const node={...old,size:bytes.length,integrity:{algorithm:'SHA256',hash:digest(bytes),blockSize,blocks}};
  if(old.unpacked){const output=path.join(destination+'.unpacked',...parts);await fsp.mkdir(path.dirname(output),{recursive:true});await fsp.writeFile(output,bytes);}
  else {node.offset=String(offset);offset+=bytes.length;appended.push(bytes);}
  Object.defineProperty(parent.files,leaf,{value:node,enumerable:true,writable:true,configurable:true});
 }
 const text=JSON.stringify(header),head=pickle.createEmpty();head.writeString(text);const headBytes=head.toBuffer(),size=pickle.createEmpty();size.writeUInt32(headBytes.length);
 const out=fs.createWriteStream(destination);out.write(size.toBuffer());out.write(headBytes);
 await pipeline(fs.createReadStream(source,{start:8+raw.headerSize}),out,{end:false});for(const bytes of appended)out.write(bytes);out.end();await finished(out);asar.uncache(destination);
 for(const [name,bytes]of replacements){if(digest(asar.extractFile(destination,path.normalize(name)))!==digest(bytes))throw Error('Appended replacement mismatch');}
 return digest(text);
}
