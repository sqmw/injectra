import fs from 'node:fs';
import path from 'node:path';
import {Readable} from 'node:stream';
import {finished} from 'node:stream/promises';
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
