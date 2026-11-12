// Windows MSVC V8 ABI experiment. Validate against an owned Node fixture first.
let hook,closed=false,busy=false,queue=[];
const retained=[];
rpc.exports={
 start(modulePath){
  if(Process.platform!=='windows'||!['ia32','x64'].includes(Process.arch))throw Error('Windows x86/x64 only');
  const x64=Process.arch==='x64';
  const m=Process.enumerateModules().find(m=>m.path.toLowerCase()===modulePath.toLowerCase());
  if(!m)throw Error('Explicit engine module is not loaded');
  const fn=(n,args,member=false)=>new NativeFunction(m.getExportByName(n),'pointer',args,x64?'win64':member?'thiscall':'mscdecl');
  const current=fn(x64?'?GetCurrent@Isolate@v8@@SAPEAV12@XZ':'?GetCurrent@Isolate@v8@@SAPAV12@XZ',[]);
  const context=fn(x64?'?GetCurrentContext@Isolate@v8@@QEAA?AV?$Local@VContext@v8@@@2@XZ':'?GetCurrentContext@Isolate@v8@@QAE?AV?$Local@VContext@v8@@@2@XZ',['pointer','pointer'],true);
  const scope=fn(x64?'??0HandleScope@v8@@QEAA@PEAVIsolate@1@@Z':'??0HandleScope@v8@@QAE@PAVIsolate@1@@Z',['pointer','pointer'],true);
  const unscope=fn(x64?'??1HandleScope@v8@@QEAA@XZ':'??1HandleScope@v8@@QAE@XZ',['pointer'],true);
  const string=fn(x64?'?NewFromUtf8@String@v8@@SA?AV?$MaybeLocal@VString@v8@@@2@PEAVIsolate@2@PEBDW4NewStringType@2@H@Z':'?NewFromUtf8@String@v8@@SA?AV?$MaybeLocal@VString@v8@@@2@PAVIsolate@2@PBDW4NewStringType@2@H@Z',['pointer','pointer','pointer','int','int']);
  const compile=fn(x64?'?Compile@Script@v8@@SA?AV?$MaybeLocal@VScript@v8@@@2@V?$Local@VContext@v8@@@2@V?$Local@VString@v8@@@2@PEAVScriptOrigin@2@@Z':'?Compile@Script@v8@@SA?AV?$MaybeLocal@VScript@v8@@@2@V?$Local@VContext@v8@@@2@V?$Local@VString@v8@@@2@PAVScriptOrigin@2@@Z',['pointer','pointer','pointer','pointer']);
  const run=fn(x64?'?Run@Script@v8@@QEAA?AV?$MaybeLocal@VValue@v8@@@2@V?$Local@VContext@v8@@@2@@Z':'?Run@Script@v8@@QAE?AV?$MaybeLocal@VValue@v8@@@2@V?$Local@VContext@v8@@@2@@Z',['pointer','pointer','pointer'],true);
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
