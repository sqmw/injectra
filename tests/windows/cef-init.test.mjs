import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
const source=fs.readFileSync(new URL('./cef-init.js',import.meta.url),'utf8');
function fixture({major=107,size=440,oldPort=0}={}) {
 const bytes=Buffer.alloc(size);bytes.writeBigUInt64LE(BigInt(size));if(size>=376)bytes.writeInt32LE(oldPort,372);
 class P {constructor(b,offset=0){this.b=b;this.offset=offset;}add(n){return new P(this.b,this.offset+n);}readPointer(){return {toString:()=>String(this.b.readBigUInt64LE(this.offset))};}readS32(){return this.b.readInt32LE(this.offset);}writeS32(n){this.b.writeInt32LE(n,this.offset);}}
 let hooks,observer;const messages=[],allocated=[];const rpc={exports:{}};
 vm.runInNewContext(source,{rpc,Process:{platform:'windows',arch:'x64',id:42,attachModuleObserver(o){observer=o;return {detach(){}};}},NativeFunction:function(){return ()=>major;},Memory:{alloc(n){const p=new P(Buffer.alloc(n));allocated.push(p);return p;},copy(to,from,n){from.b.copy(to.b,0,from.offset,from.offset+n);}},Interceptor:{attach(_address,o){hooks=o;return {detach(){}};}},send:x=>messages.push(x)});
 rpc.exports.start(50001);observer.onAdded({name:'libcef.dll',path:'fixture.dll',getExportByName:n=>n});
 const original=new P(bytes),args=[null,original];if(hooks)hooks.onEnter.call({},args);
 return {args,original,messages,allocated,hooks};
}
test('exact CEF layout changes only a temporary settings copy',()=>{const f=fixture();assert.notEqual(f.args[1],f.original);assert.equal(f.original.add(372).readS32(),0);assert.equal(f.args[1].add(372).readS32(),50001);assert.equal(f.messages.at(-1).event,'cef-initialize');});
test('unknown version and structure size never substitute settings',()=>{for(const input of [{major:999},{size:432}]){const f=fixture(input);assert.equal(f.args[1],f.original);assert.equal(f.allocated.length,0);}});
test('preserve an existing inspector port',()=>{const f=fixture({oldPort:9222});assert.equal(f.args[1],f.original);assert.equal(f.messages.at(-1).event,'cef-existing-debug-port');});
