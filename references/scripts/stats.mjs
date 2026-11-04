import {readFileSync,readdirSync} from 'node:fs';
import {dirname,resolve,relative,extname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
export const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const extensions=new Set(['.py','.js','.mjs','.rs','.sh','.css','.html','.mk']);
const isCode=p=>extensions.has(extname(p))||p.split('/').at(-1)==='Makefile';
export function walkCode(dir){return readdirSync(dir,{withFileTypes:true}).flatMap(e=>{
 if(e.name.startsWith('.')||e.name==='node_modules'||e.name==='__pycache__'||e.isSymbolicLink())return [];
 const p=resolve(dir,e.name);return e.isDirectory()?walkCode(p):e.isFile()&&isCode(p)?[p]:[];
});}
export function collect(){
 const catalog=JSON.parse(readFileSync(resolve(root,'catalog.json')));
 const owned=new Set(),files=[];
 function add(path,category,application=null){
  const p=resolve(root,path),rel=relative(root,p);
  if(rel.startsWith('..')||owned.has(p))throw Error(`Invalid/duplicate ownership: ${path}`);
  if(!isCode(rel))throw Error(`Metadata is not source code: ${path}`);
  const data=readFileSync(p);owned.add(p);
  files.push({path:rel,category,application,lines:data.toString().trimEnd().split('\n').length,bytes:data.length,sha256:createHash('sha256').update(data).digest('hex')});
 }
 for(const p of walkCode(resolve(root,'shared')))add(p,'shared');
 const applications=[];
 for(const manifest of catalog.cases){
  const c=JSON.parse(readFileSync(resolve(root,manifest))),dir=dirname(manifest);
  for(const category of ['application','validation'])for(const p of c.source_groups[category])add(resolve(root,dir,p),category,c.id);
  for(const dependency of c.shared_dependencies)if(!walkCode(resolve(root,dependency)).length)throw Error(`Missing shared dependency ${dependency}`);
  applications.push({id:c.id,name:c.application,version:c.recorded_version,shared_dependencies:c.shared_dependencies});
 }
 for(const p of walkCode(resolve(root,'scripts')))add(p,'maintenance');
 add('Makefile','maintenance');
 add('config.mk','maintenance');
 const unowned=walkCode(root).filter(p=>!owned.has(p));if(unowned.length)throw Error(`Unclassified sources: ${unowned.map(p=>relative(root,p)).join(', ')}`);
 const sum=predicate=>files.filter(predicate).reduce((n,p)=>n+p.lines,0);
 return {schema_version:1,policy:catalog.statistics,totals:{shared:sum(p=>p.category==='shared'),application:sum(p=>p.category==='application'),validation:sum(p=>p.category==='validation'),maintenance:sum(p=>p.category==='maintenance'),all:sum(()=>true)},applications:applications.map(c=>({...c,application_lines:sum(p=>p.application===c.id&&p.category==='application'),validation_lines:sum(p=>p.application===c.id&&p.category==='validation')})),files};
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const result=collect();
 if(process.argv.includes('--json'))console.log(JSON.stringify(result,null,2));
 else{
  console.log('Application\tVersion\tApplication lines\tValidation lines');
  for(const a of result.applications)console.log(`${a.name}\t${a.version}\t${a.application_lines}\t${a.validation_lines}`);
  console.log(`Shared (once): ${result.totals.shared}; maintenance: ${result.totals.maintenance}; total: ${result.totals.all}`);
 }
}
