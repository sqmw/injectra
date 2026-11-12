'use strict';
let listener=null, calls=0,returned=0,address,original;
rpc.exports={
 start(){
  if(listener)throw Error('Hook already installed');
  const ui=Process.getModuleByName('user32.dll');address=ui.getExportByName('DispatchMessageW');
  original=Array.from(new Uint8Array(address.readByteArray(16)));
  listener=Interceptor.attach(address,{onEnter(){calls++;},onLeave(){returned++;}});
  return {pid:Process.id,platform:Process.platform,arch:Process.arch,method:'user32.DispatchMessageW',engine_modules:Process.enumerateModules().filter(m=>/^(libcef|chrome|msedge|node|electron)/i.test(m.name)).map(m=>m.name)};
 },
 snapshot(){return{calls,returned};},
 stop(){listener?.detach();listener=null;Interceptor.flush();return{calls,returned,restored:JSON.stringify(original)===JSON.stringify(Array.from(new Uint8Array(address.readByteArray(16))))};}
};
