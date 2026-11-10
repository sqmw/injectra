import test from 'node:test';
import assert from 'node:assert/strict';
import { createAgent } from '../../crates/webkit/helper/webkit_agent.js';

const settle=()=>new Promise(resolve=>setImmediate(resolve));
function fixture({holdNative=false,holdQueue=false,rejectSchedule=false,rejectBlock=false,rejectCleanup=false}={}) {
  let onMain=false,pins=0,retains=0,calls=0,schedules=0,callback;
  const queued=[];
  globalThis.Process={id:123,arch:'test-arch',mainModule:{path:'/test-only'}};
  globalThis.Script={pin(){pins++;},unpin(){pins--;}};
  globalThis.ptr=()=>0;
  const classes={
    WKWebView:{},
    WKContentWorld:{pageWorld:()=> 'page-world'},
    NSDictionary:{dictionary:()=>({})},
  };
  Object.defineProperty(classes,'NSApplication',{get(){throw Error('Desktop API must stay in the host adapter');}});
  const ObjC={
    available:true,classes,selector:name=>name,
    Object:class {constructor(value){return value;}},
    Block:class {constructor(description){if(rejectBlock)throw Error("callback allocation failed");Object.assign(this,description);}},
  };
  const view={
    respondsToSelector_(){assert.equal(onMain,true);return true;},
    retain(){assert.equal(onMain,true);retains++;},
    release(){assert.equal(onMain,true);retains--;},
    callAsyncJavaScript_arguments_inFrame_inContentWorld_completionHandler_(source,args,frame,world,block){
      assert.equal(onMain,true);assert.equal(world,'page-world');calls++;callback=block;
      if(!holdNative){
        const AsyncFunction=Object.getPrototypeOf(async function(){}).constructor;
        new AsyncFunction(source)().then(value=>block.implementation({toString:()=>value},null));
      }
    },
  };
  const invoke=fn=>{onMain=true;try{fn();}finally{onMain=false;}};
  const host={
    available:()=>true,
    schedule(fn){
      schedules++;
      if(rejectSchedule || (rejectCleanup&&schedules>1))throw Error('host did not accept callback');
      if(holdQueue)queued.push(()=>invoke(fn));else queueMicrotask(()=>invoke(fn));
    },
    views(){
      assert.equal(onMain,true);
      return [{view,native_id:'test-view',url:'test://page',title:'Test host'}];
    },
  };
  const api=createAgent(ObjC,host);
  return {api,counts:()=>({pins,retains,calls}),queued,
    complete(){callback.implementation({toString:()=>JSON.stringify({ok:true,value:2})},null);}};
}

test('WK engine discovers pages using a host port without desktop window APIs',async()=>{
  const f=fixture();
  assert.equal(f.api.probe().available,true);
  assert.deepEqual(await f.api.targets(),{ok:true,value:[{native_id:'test-view',url:'test://page',title:'Test host'}]});
  await settle();
  assert.deepEqual(f.counts(),{pins:0,retains:0,calls:0});
});

test('page execution and release use the host main queue and page content world',async()=>{
  const f=fixture();
  assert.deepEqual(await f.api.evaluate('test-view','test://page','Promise.resolve(2)'),{ok:true,value:2});
  await settle();
  assert.deepEqual(f.counts(),{pins:0,retains:0,calls:1});
  assert.equal(f.api.shutdown().closed,true);
});

test('stale view identity is rejected before native execution',async()=>{
  const f=fixture();
  const value=await f.api.evaluate('foreign-view','test://page','1+1');
  assert.equal(value.kind,'target_not_found');assert.equal(value.certainty,'NotExecuted');
  await settle();assert.equal(f.counts().calls,0);assert.equal(f.counts().pins,0);
});

test('a host that rejects scheduling does not leave a pinned callback',async()=>{
  const f=fixture({rejectSchedule:true});
  const value=await f.api.targets();
  assert.equal(value.certainty,'NotExecuted');
  assert.equal(f.api.shutdown().closed,true);
  assert.deepEqual(f.counts(),{pins:0,retains:0,calls:0});
});

test('deadline does not free a native callback that still owns a retained view',async t=>{
  const timers=[];
  t.mock.method(globalThis,'setTimeout',fn=>{timers.push(fn);return timers.length;});
  t.mock.method(globalThis,'clearTimeout',()=>{});
  const f=fixture({holdNative:true});
  const pending=f.api.evaluate('test-view','test://page','1+1');
  await settle();
  assert.deepEqual(f.counts(),{pins:1,retains:1,calls:1});
  timers[0]();
  assert.equal((await pending).certainty,'Unknown');
  assert.equal(f.api.shutdown().closed,false);
  f.complete();await settle();
  assert.equal(f.api.status().closed,true);
  assert.deepEqual(f.counts(),{pins:0,retains:0,calls:1});
});

test('a callback queued past its deadline drains without starting a late payload',async t=>{
  const timers=[];
  t.mock.method(globalThis,'setTimeout',fn=>{timers.push(fn);return timers.length;});
  t.mock.method(globalThis,'clearTimeout',()=>{});
  const f=fixture({holdQueue:true});
  const pending=f.api.evaluate('test-view','test://page','1+1');
  timers[0]();
  assert.equal((await pending).certainty,'Unknown');
  assert.equal(f.api.shutdown().closed,false);
  f.queued.shift()();await settle();
  assert.equal(f.api.status().closed,true);
  assert.deepEqual(f.counts(),{pins:0,retains:0,calls:0});
});

test("native callback allocation failure releases its retained view before returning",async()=>{
  const f=fixture({rejectBlock:true});
  const result=await f.api.evaluate("test-view","test://page","1+1");
  assert.equal(result.certainty,"NotExecuted");
  await settle();
  assert.deepEqual(f.counts(),{pins:0,retains:0,calls:0});
  assert.equal(f.api.shutdown().closed,true);
});

test("failed cleanup scheduling never reports retained native resources as closed",async()=>{
  const f=fixture({rejectBlock:true,rejectCleanup:true});
  const result=await f.api.evaluate("test-view","test://page","1+1");
  assert.equal(result.certainty,"Unknown");
  await settle();
  assert.equal(f.api.shutdown().closed,false);
  assert.equal(f.counts().retains,1);
});
