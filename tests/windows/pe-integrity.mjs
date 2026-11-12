// Windows PE format helper, based on Electron Packager's documented INTEGRITY resource.
import fs from 'node:fs';
import path from 'node:path';
export function updateIntegrity({library,binary,archive,before,after,ownedRoot}) {
 const relative=path.relative(path.resolve(ownedRoot),path.resolve(binary));
 if(!relative||relative.startsWith('..')||path.isAbsolute(relative))throw Error('PE mutation outside owned copy');
 const bytes=fs.readFileSync(binary),exe=library.NtExecutable.from(bytes,{ignoreCert:true}),resources=library.NtExecutableResource.from(exe);
 let updates=0;
 for(const entry of resources.entries) {
  if(String(entry.type).toUpperCase()!=='INTEGRITY'||String(entry.id).toUpperCase()!=='ELECTRONASAR')continue;
  const metadata=JSON.parse(Buffer.from(entry.bin).toString('utf8').replace(/\0+$/,''));if(!Array.isArray(metadata))throw Error('Unsupported PE integrity format');
  let changed=false;
  for(const record of metadata) {
   if(typeof record.file!=='string'||record.file.includes(':')||/^[\\/]/.test(record.file))throw Error('Unsupported PE archive location');
   const referenced=path.resolve(path.dirname(binary),record.file.replaceAll('\\',path.sep));
   if(referenced.toLowerCase()!==path.resolve(archive).toLowerCase())continue;
   if(String(record.alg).toLowerCase()!=='sha256'||String(record.value).toLowerCase()!==before)throw Error('PE integrity baseline mismatch');
   record.value=after;changed=true;updates++;
  }
  if(changed)entry.bin=new TextEncoder().encode(JSON.stringify(metadata)).buffer;
 }
 if(updates){resources.outputResource(exe);fs.writeFileSync(binary,Buffer.from(exe.generate()));}
 return {updates,signature_preserved:updates===0};
}
