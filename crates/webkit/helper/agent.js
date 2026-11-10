import ObjC from 'frida-objc-bridge';

let closing=false, active=0, next=0;
const blocks=new Map();
function failure(kind,message,certainty='NotExecuted'){return {ok:false,kind,message,certainty};}
function main(operation) {
  if(closing)return Promise.resolve(failure('connection','Session is closing'));
  active++;Script.pin();
  return new Promise(resolve=>{
    let delivered=false;
    const timer=setTimeout(()=>{delivered=true;resolve(failure('timeout','Native main-thread deadline','Unknown'));},6500);
    ObjC.schedule(ObjC.mainQueue,()=>{
      let value;
      try { value=operation(); } catch(error) { value=failure("connection",String(error),"Unknown"); }
      Promise.resolve(value).then(result=>{
        if(!delivered){clearTimeout(timer);resolve(result);}
      },error=>{
        if(!delivered){clearTimeout(timer);resolve(failure('connection',String(error),'Unknown'));}
      }).finally(()=>{active--;Script.unpin();});
    });
  });
}
function text(object){return object===null ? '' : object.toString();}
function views(){
  const result=[],seen=new Set();
  function visit(view,window){
    if(!view||seen.has(view.handle.toString())||seen.size>=4096)return;
    seen.add(view.handle.toString());
    if(view.isKindOfClass_(ObjC.classes.WKWebView)){
      result.push({view,native_id:view.handle.toString(),url:text(view.URL()),title:text(view.title())||text(window.title())});
    }
    const children=view.subviews();
    for(let i=0;i<Number(children.count());i++)visit(children.objectAtIndex_(i),window);
  }
  const windows=ObjC.classes.NSApplication.sharedApplication().windows();
  for(let i=0;i<Number(windows.count());i++){const window=windows.objectAtIndex_(i);visit(window.contentView(),window);}
  return result;
}
function envelope(source){
  return 'return await (async()=>{let timer;try{return await Promise.race([(async()=>{try{const value=await ('+source+');try{return JSON.stringify({ok:true,value:value===undefined?null:value});}catch(e){return JSON.stringify({ok:false,kind:"result_encoding",message:String(e),certainty:"Executed"});}}catch(e){return JSON.stringify({ok:false,kind:"execution",message:String(e),stack:String(e?.stack??""),certainty:"Executed"});}})(),new Promise(resolve=>{timer=setTimeout(()=>resolve(JSON.stringify({ok:false,kind:"timeout",message:"Page Promise deadline; no replay",certainty:"Unknown"})),4500);})]);}finally{clearTimeout(timer);}})();';
}
rpc.exports={
  identity(){return {pid:Process.id,arch:Process.arch,executable:Process.mainModule.path};},
  probe(){return {available:ObjC.available&&!!ObjC.classes.WKWebView&&!!ObjC.classes.WKContentWorld};},
  targets(){return main(()=>({ok:true,value:views().map(({view,...page})=>page)}));},
  evaluate(nativeId,url,source){
    return main(()=>{
      const entry=views().find(v=>v.native_id===nativeId&&v.url===url);
      if(!entry)return failure('target_not_found','WKWebView identity changed before execution');
      const view=entry.view;
      if(!view.respondsToSelector_(ObjC.selector('callAsyncJavaScript:arguments:inFrame:inContentWorld:completionHandler:')))
        return failure('unsupported','WKWebView asynchronous page API unavailable');
      return new Promise(resolve=>{
        const token=++next;view.retain();
        const finish=result=>ObjC.schedule(ObjC.mainQueue,()=>{
          blocks.delete(token);view.release();resolve(result);
        });
        const block=new ObjC.Block({
          retType:'void',argTypes:['object','object'],
          implementation(value,error){
            let result;
            try{
              if(error!==null)result=failure('connection',new ObjC.Object(error.handle??error).localizedDescription().toString(),'Unknown');
              else if(value===null)result=failure('result_encoding','Missing WebKit result','Executed');
              else result=JSON.parse(new ObjC.Object(value.handle??value).toString());
            }catch(e){result=failure('result_encoding',String(e),'Executed');}
            finish(result);
          }
        });
        blocks.set(token,block);
        try{
          view.callAsyncJavaScript_arguments_inFrame_inContentWorld_completionHandler_(
            envelope(source),ObjC.classes.NSDictionary.dictionary(),ptr(0),
            ObjC.classes.WKContentWorld.pageWorld(),block);
        }catch(error){finish(failure('connection',String(error),'Unknown'));}
      });
    });
  },
  shutdown(){closing=true;return {closed:active===0&&blocks.size===0};},
  status(){return {closed:closing&&active===0&&blocks.size===0,active,callbacks:blocks.size};}
};
