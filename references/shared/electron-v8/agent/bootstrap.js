const getenv = new NativeFunction(Module.getGlobalExportByName('getenv'), 'pointer', ['pointer']);
const unsetenv = new NativeFunction(Module.getGlobalExportByName('unsetenv'), 'int', ['pointer']);
function env(name) { const p=getenv(Memory.allocUtf8String(name)); return p.isNull()?null:p.readUtf8String(); }
function foundationHome() {
 const f=Module.findGlobalExportByName('NSHomeDirectory');
 if (f===null) return null;
 const home=new NativeFunction(f,'pointer',[])();
 const sel=new NativeFunction(Module.getGlobalExportByName('sel_registerName'),'pointer',['pointer'])(Memory.allocUtf8String('UTF8String'));
 const chars=new NativeFunction(Module.getGlobalExportByName('objc_msgSend'),'pointer',['pointer','pointer'])(home,sel);
 return chars.isNull()?null:chars.readUtf8String();
}
rpc.exports = {
 bootstrap() {
  const result={pid:Process.id,arch:Process.arch,executable:Process.mainModule.path,
    gadget:Process.enumerateModules().filter(m=>/FridaGadget/.test(m.name)).map(m=>m.path),
    home:env('HOME'),fixedHome:env('CFFIXED_USER_HOME'),foundationHome:foundationHome(),
    beforeDyld:env('DYLD_INSERT_LIBRARIES')};
  result.unsetDyld=unsetenv(Memory.allocUtf8String('DYLD_INSERT_LIBRARIES'));
  result.afterDyld=env('DYLD_INSERT_LIBRARIES');
  return result;
 }
};
