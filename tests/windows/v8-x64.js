// Windows x64 MSVC V8 ABI experiment. Validate against an owned Node fixture first.
let hook,closed=false,busy=false,queue=[];
const retained=[];
rpc.exports={
 start(modulePath){
  if(Process.platform!=='windows'||Process.arch!=='x64')throw Error('Windows x64 only');
  const m=Process.enumerateModules().find(m=>m.path.toLowerCase()===modulePath.toLowerCase());
  if(!m)throw Error('Explicit engine module is not loaded');
  const fn=(n,args)=>new NativeFunction(m.getExportByName(n),'pointer',args);
  const current=fn('?GetCurrent@Isolate@v8@@SAPEAV12@XZ',[]);
  const context=fn('?GetCurrentContext@Isolate@v8@@QEAA?AV?$Local@VContext@v8@@@2@XZ',['pointer','pointer']);
  const scope=fn('??0HandleScope@v8@@QEAA@PEAVIsolate@1@@Z',['pointer','pointer']);
  const unscope=fn('??1HandleScope@v8@@QEAA@XZ',['pointer']);
  const string=fn('?NewFromUtf8@String@v8@@SA?AV?$MaybeLocal@VString@v8@@@2@PEAVIsolate@2@PEBDW4NewStringType@2@H@Z',['pointer','pointer','pointer','int','int']);
  const compile=fn('?Compile@Script@v8@@SA?AV?$MaybeLocal@VScript@v8@@@2@V?$Local@VContext@v8@@@2@V?$Local@VString@v8@@@2@PEAVScriptOrigin@2@@Z',['pointer','pointer','pointer','pointer']);
  const run=fn('?Run@Script@v8@@QEAA?AV?$MaybeLocal@VValue@v8@@@2@V?$Local@VContext@v8@@@2@@Z',['pointer','pointer','pointer']);
  hook=Interceptor.attach(Module.getGlobalExportByName('GetQueuedCompletionStatusEx'),{onLeave(){
   if(closed||busy||!queue.length)return;
   const isolate=current();if(isolate.isNull())return;
   busy=true;let handleScope=null;
   const command=queue.shift();
   try{
    handleScope=Memory.alloc(4096);scope(handleScope,isolate);
    const out=Memory.alloc(16);context(isolate,out);const ctx=out.readPointer();if(ctx.isNull())throw Error('No active context');
    string(out,isolate,Memory.allocUtf8String(command.source),0,-1);const text=out.readPointer();if(text.isNull())throw Error('String creation failed');
    out.writePointer(ptr(0));compile(out,ctx,text,ptr(0));const compiled=out.readPointer();if(compiled.isNull())throw Error('Compilation failed');
    out.writePointer(ptr(0));run(compiled,out,ctx);send({event:'v8-run',id:command.id,returned:!out.readPointer().isNull(),thread:Process.getCurrentThreadId()});
   }catch(error){send({event:'v8-error',id:command.id,message:String(error)});}
   finally{if(handleScope)unscope(handleScope);busy=false;}
  }});
  return {pid:Process.id,arch:Process.arch,module:m.path};
 },
 execute(id,source){if(closed)throw Error('Closed');queue.push({id,source});return {queued:true};},
 stop(){closed=true;queue=[];hook?.detach();return {detached:true};}
};
