// V8 ABI bridge derived from verified probe 622fe2e; no Interceptor or CDP.
let callbacks=[],queue=[],uv=null,handle=null,closed=false,pendingClose=false,scheduled=false;
let setupError=null,initialized=false,runs=0;
function fn(module,name,ret,args){return new NativeFunction(module.getExportByName(name),ret,args);}
function initialize(){
 const electron=Process.enumerateModules().find(m=>m.name==='Electron Framework');
 if(!electron)throw Error('Electron Framework unavailable');
 uv={loop:fn(electron,'uv_default_loop','pointer',[]),init:fn(electron,'uv_async_init','int',['pointer','pointer','pointer']),
 send:fn(electron,'uv_async_send','int',['pointer']),unref:fn(electron,'uv_unref','void',['pointer']),
 close:fn(electron,'uv_close','void',['pointer','pointer']),size:fn(electron,'uv_handle_size','ulong',['int'])};
 const v={isolate:fn(electron,'_ZN2v87Isolate10GetCurrentEv','pointer',[]),
 context:fn(electron,'_ZN2v87Isolate17GetCurrentContextEv','pointer',['pointer']),
 scope:fn(electron,'_ZN2v811HandleScopeC1EPNS_7IsolateE','void',['pointer','pointer']),
 unscope:fn(electron,'_ZN2v811HandleScopeD1Ev','void',['pointer']),
 string:fn(electron,'_ZN2v86String11NewFromUtf8EPNS_7IsolateEPKcNS_13NewStringTypeEi','pointer',['pointer','pointer','int','int']),
 compile:fn(electron,'_ZN2v86Script7CompileENS_5LocalINS_7ContextEEENS1_INS_6StringEEEPNS_12ScriptOriginE','pointer',['pointer','pointer','pointer']),
 run:fn(electron,'_ZN2v86Script3RunENS_5LocalINS_7ContextEEE','pointer',['pointer','pointer'])};
 const onClose=new NativeCallback(()=>{closed=true;send({event:'uv-closed'});Script.unpin();},'void',['pointer']);
 const processQueue=new NativeCallback(()=>{
  if(pendingClose){uv.close(handle,onClose);return;}
  while(queue.length){
   const {id,source}=queue.shift();let scope=null;
   try{
    const isolate=v.isolate();if(isolate.isNull())throw Error('No current isolate');
    scope=Memory.alloc(4096);v.scope(scope,isolate);
    const context=v.context(isolate);if(context.isNull())throw Error('No current context');
    const code=v.string(isolate,Memory.allocUtf8String(source),0,-1);if(code.isNull())throw Error('V8 string creation returned empty');
    const compiled=v.compile(context,code,ptr(0));if(compiled.isNull())throw Error('V8 compile returned empty');
    runs++;const result=v.run(compiled,context);send({event:'v8-dispatched',id,returned:!result.isNull(),runs});
   }catch(error){send({event:'v8-error',id,message:String(error),runs});}
   finally{if(scope!==null)v.unscope(scope);}
  }
 },'void',['pointer']);
 const onMain=new NativeCallback(()=>{
  if(pendingClose){closed=true;Script.unpin();return;}
  try{
   const size=Number(uv.size(1));if(size<32||size>4096)throw Error('Unexpected uv async handle size');
   handle=Memory.alloc(size);const rc=uv.init(uv.loop(),handle,processQueue);if(rc!==0)throw Error('uv_async_init='+rc);
   uv.unref(handle);initialized=true;
  }catch(error){setupError=String(error);closed=true;Script.unpin();}
 },'void',['pointer']);
 callbacks=[onClose,processQueue,onMain];
 const dispatch=new NativeFunction(Module.getGlobalExportByName('dispatch_async_f'),'void',['pointer','pointer','pointer']);
 Script.pin();scheduled=true;dispatch(Module.getGlobalExportByName('_dispatch_main_q'),ptr(0),onMain);
 return {scheduled:true,policy:Process.codeSigningPolicy};
}
rpc.exports={
 identity(){return {pid:Process.id,arch:Process.arch,executable:Process.mainModule.path};},
 setup(){try{return initialize();}catch(e){setupError=String(e);return {error:setupError};}},
 status(){return {initialized,closed,setupError,runs,pending:queue.length};},
 execute(id,source){if(!initialized||closed||pendingClose)return {accepted:false};queue.push({id,source});uv.send(handle);return {accepted:true};},
 shutdown(){if(closed||pendingClose)return {closing:!closed,closed};pendingClose=true;if(initialized)uv.send(handle);else if(!scheduled)closed=true;return {closing:!closed,closed};}
};
