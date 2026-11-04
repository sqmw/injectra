import ObjC from 'frida-objc-bridge';
rpc.exports={inspect(){return new Promise((resolve,reject)=>ObjC.schedule(ObjC.mainQueue,()=>{try{
 const app=ObjC.classes.NSApplication.sharedApplication();const a=app.windows();const windows=[];
 for(let i=0;i<Number(a.count());i++){const w=a.objectAtIndex_(i);windows.push({class:w.$className,title:w.title().toString(),visible:!!w.isVisible()});}
 resolve({pid:Process.id,policy:Process.codeSigningPolicy,windows,wk:!!ObjC.classes.WKWebView});
 }catch(e){reject(e)}}));}};
