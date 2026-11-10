// Concrete macOS AppKit host adapter. No UIKit/mobile implementation is registered.
export function createMacOSHost(ObjC) {
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
  return {
    available() { return Process.platform === 'darwin' && !!ObjC.classes.NSApplication; },
    // A synchronous throw means no callback was accepted; accepted callbacks must drain.
    schedule(callback) { ObjC.schedule(ObjC.mainQueue, callback); },
    views,
  };
}
