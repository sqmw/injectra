// Device-side read-only identity probe, owned by the macOS-local target provider.
rpc.exports = {
  identity() {
    const receipt = {pid:Process.id, platform:Process.platform, arch:Process.arch,
      executable:Process.mainModule.path};
    if (Process.platform !== 'darwin' || Process.pointerSize !== 8) return receipt;
    const size=Memory.alloc(8);size.writeU64(128);
    const value=Memory.alloc(128);
    const read=new NativeFunction(Module.getGlobalExportByName('sysctlbyname'),'int',
      ['pointer','pointer','pointer','pointer','ulong']);
    const rc=read(Memory.allocUtf8String('kern.bootsessionuuid'),value,size,ptr(0),0);
    if (rc===0 && size.readU64().toNumber()<=128) receipt.boot=value.readUtf8String();
    return receipt;
  }
};
