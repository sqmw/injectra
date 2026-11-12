// Optional native Windows Gecko verification; reuses the Mac test protocol/host contracts.
import assert from 'node:assert/strict';
import {parseArgs} from 'node:util';
import {readFile} from 'node:fs/promises';
import {createServer} from 'node:http';
import {once} from 'node:events';
import {resolve} from 'node:path';
import {BidiClient} from '../tests/gecko/bidi-client.mjs';
import {MarionetteClient} from '../tests/gecko/marionette-client.mjs';
import {ThunderbirdContentHost} from '../tests/gecko/thunderbird-host.mjs';
import {verifyRuntime} from '../tests/gecko/runtime-checks.mjs';
import {root} from './test-env.mjs';
const {values}=parseArgs({options:{port:{type:'string'},protocol:{type:'string'}}});
assert.equal(process.platform,'win32');assert(['bidi','marionette'].includes(values.protocol));
const port=Number(values.port),runtime=await readFile(resolve(root,'web/runtime.js'),'utf8');
const result={passed:false,platform:process.platform,protocol:values.protocol,productBackend:false,checks:[]};
const check=async(name,action)=>{await action();result.checks.push(name);};
const server=createServer((req,res)=>{res.writeHead(200,{'Content-Type':'text/html; charset=utf-8'});res.end('<!doctype html><title>Injectra owned '+req.url+'</title><h1>Owned fixture</h1>');});
server.listen(0,'127.0.0.1');await once(server,'listening');const origin='http://127.0.0.1:'+server.address().port;
let client,closeContent;
try {
 const start=Date.now();
 while(!client){try{client=values.protocol==='bidi'?await BidiClient.connect(`ws://127.0.0.1:${port}/session`,()=>{}):await MarionetteClient.connect(port,()=>{});}catch(error){if(Date.now()-start>40000)throw error;await new Promise(r=>setTimeout(r,500));}}
 let evaluate,other;
 if(values.protocol==='bidi') {
  result.session=await client.command('session.new',{capabilities:{}},45000);
  const a=await client.command('browsingContext.create',{type:'tab'}),b=await client.command('browsingContext.create',{type:'tab'});
  for(const [context,path]of [[a.context,'main'],[b.context,'other']])await client.command('browsingContext.navigate',{context,url:origin+'/'+path,wait:'complete'});
  evaluate=source=>client.evaluate({context:a.context},source);other=source=>client.evaluate({context:b.context},source);
  closeContent=async()=>{await client.command('browsingContext.close',{context:a.context});await client.command('browsingContext.close',{context:b.context});};
 } else {
  result.session=await client.command('WebDriver:NewSession',{webSocketUrl:true},45000);
  await client.command('Marionette:SetContext',{value:'chrome'});
  assert.equal(await client.evaluate("!!document.getElementById('tabmail')"),true);
  const host=new ThunderbirdContentHost(client,result.session.sessionId);
  await client.evaluate("(()=>{const t=document.getElementById('tabmail');t.openTab('contentTab',{url:"+JSON.stringify(origin+'/main')+"});t.openTab('contentTab',{url:"+JSON.stringify(origin+'/other')+"});return true;})()");
  let a,b;const deadline=Date.now()+30000;
  while(!a||!b){const pages=await host.list();a=pages.find(p=>p.url===origin+'/main'&&!p.loading);b=pages.find(p=>p.url===origin+'/other'&&!p.loading);if(Date.now()>deadline)throw Error('Content target deadline');if(!a||!b)await new Promise(r=>setTimeout(r,100));}
  evaluate=source=>host.evaluate(a,source);other=source=>host.evaluate(b,source);
  closeContent=async()=>{await host.close(a);await host.close(b);};
  result.chrome={};
  for(const mode of ['html','parsed-html']){
   try{await verifyRuntime({evaluate:source=>client.evaluatePage(source),other:evaluate,runtime,id:'injectra.win.chrome.'+mode,check,mountMode:mode});result.chrome[mode]={passed:true};}
   catch(error){result.chrome[mode]={passed:false,error:error.message};}
  }
  assert.equal(result.chrome['parsed-html'].passed,true);
 }
 await verifyRuntime({evaluate,other,runtime,id:'injectra.win.content',check});result.passed=true;
} catch(error){result.error=String(error);process.exitCode=1;}
finally{try{await closeContent?.();}catch(error){result.cleanupError=String(error);result.passed=false;process.exitCode=1;}client?.close();await new Promise(r=>server.close(r));console.log(JSON.stringify(result));}
