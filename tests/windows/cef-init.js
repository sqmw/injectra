// Experimental CEF C ABI probe, Windows only. No application-name branches.
// Layouts: chromiumembedded/cef branches 4240 (86), 4472 (91), 5304 (107),
// include/internal/cef_types.h. Only an exact version/size combination is used.
const layouts = {
  86: {ia32:{size:240,port:200}, x64:{size:440,port:372}},
  91: {ia32:{size:256,port:200}, x64:{size:472,port:372}},
  107:{ia32:{size:240,port:200}, x64:{size:440,port:372}},
};
let observer, listener, port;
rpc.exports = {
  start(value) {
    if (Process.platform !== 'windows' || !Number.isInteger(value) || value < 1024 || value > 65535) throw Error('Invalid Windows probe configuration');
    port = value;
    observer = Process.attachModuleObserver({onAdded(module) {
      if (module.name.toLowerCase() !== 'libcef.dll' || listener) return;
      try {
        const version = new NativeFunction(module.getExportByName('cef_version_info'), 'int', ['int'])(0);
        const layout = layouts[version]?.[Process.arch];
        send({event:'cef-module',version,arch:Process.arch,supported:!!layout,path:module.path});
        if (!layout) return;
        listener = Interceptor.attach(module.getExportByName('cef_initialize'), {
          onEnter(args) {
            const size = Number(args[1].readPointer().toString());
            if (size !== layout.size) {send({event:'cef-layout-rejected',size,expected:layout.size});return;}
            // Substitute a copy for this call; do not modify caller-owned settings.
            this.settings = Memory.alloc(size);
            Memory.copy(this.settings,args[1],size);
            const oldPort = this.settings.add(layout.port).readS32();
            if (oldPort !== 0 && oldPort !== port) {send({event:'cef-existing-debug-port',port:oldPort});return;}
            this.settings.add(layout.port).writeS32(port);
            args[1] = this.settings;
            this.modified = true;
            send({event:'cef-initialize',version,size,port});
          },
          onLeave(result) {if(this.modified)send({event:'cef-initialized',success:result.toInt32() === 1});},
        });
      } catch(error) {send({event:'cef-error',message:String(error)});}
    }});
    return {pid:Process.id,arch:Process.arch};
  },
  stop() {observer?.detach();listener?.detach();return {detached:true};},
};
