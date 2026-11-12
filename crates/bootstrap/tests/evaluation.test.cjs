const test=require('node:test'),assert=require('node:assert/strict'),vm=require('node:vm');
const {guardedSource}=require('../helper/evaluation.cjs');
const command={request_id:'request',revision:'r1',page:'1',generation:'old-document',source:'(globalThis.userEffect = true)'};
test('navigation between controller check and actual execution never runs user code in the new document',async()=>{
 const prepared=guardedSource(command,'key');const nextDocument=vm.createContext({});
 vm.runInContext("globalThis[Symbol.for('key')]='new-document'",nextDocument);
 const result=await vm.runInContext(prepared.source,nextDocument);
 assert.equal(result.certainty,'NotExecuted');assert.equal(nextDocument.userEffect,undefined);
});
test('matching document executes and injected wrapper does not shift user source lines',async()=>{
 const page=vm.createContext({});vm.runInContext("globalThis[Symbol.for('key')]='old-document'",page);
 const ok=await vm.runInContext(guardedSource(command,'key').source,page);assert.equal(ok.ok,true);assert.equal(page.userEffect,true);
 const source="(()=>{\nconst x=1;\nthrow new Error('user-failure');\n})()\n//# sourceURL=injectra://workspace/r1/page.js\n";
 const result=await vm.runInContext(guardedSource({...command,source},'key').source,page);
 assert.equal(result.certainty,'Executed');assert.match(result.stack,/:3:\d+/);
});
