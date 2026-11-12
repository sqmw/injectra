import {mkdir,copyFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import {cache,root} from './test-env.mjs';
const destination=process.env.INJECTRA_PREPARATION_TOOLS??resolve(cache,'preparation-tools');
await mkdir(destination,{recursive:true});
for(const name of ['package.json','package-lock.json'])await copyFile(resolve(root,'crates/preparation/tools',name),resolve(destination,name));
const result=spawnSync('npm',['ci','--prefix',destination,'--no-audit','--no-fund'],{stdio:'inherit'});
if(result.error)throw result.error;process.exitCode=result.status??1;
