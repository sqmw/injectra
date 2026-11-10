// Owned-instance validation: observe a pure native call, never change its result.
rpc.exports = {
  identity() { return { pid: Process.id, executable: Process.mainModule.path,
    arch: Process.arch, policy: Process.codeSigningPolicy, runtime: Script.runtime }; },
  verify() {
    const address = Module.getGlobalExportByName('abs');
    const fn = new NativeFunction(address, 'int', ['int']);
    const thread = Process.getCurrentThreadId();
    const calls = [];
    let listener;
    try {
      listener = Interceptor.attach(address, {
        onEnter(args) { this.ownedCall = this.threadId === thread; if (this.ownedCall) this.input = args[0].toInt32(); },
        onLeave(retval) { if (this.ownedCall) calls.push({ input: this.input, result: retval.toInt32() }); },
      });
      Interceptor.flush();
      const first = fn(-7), second = fn(-13);
      listener.detach(); listener = null; Interceptor.flush();
      const countAfterDetach = calls.length;
      const after = fn(-19);
      return { symbol: 'abs', calls, first, second, after, countAfterDetach,
        countAfterFinalCall: calls.length, detached: true };
    } finally {
      if (listener) { listener.detach(); Interceptor.flush(); }
    }
  },
};
