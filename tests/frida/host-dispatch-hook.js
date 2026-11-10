// Only the owned app's real NSApplication receiver; no event contents are read.
const native = (name, result, args) => new NativeFunction(
  Module.getGlobalExportByName(name), result, args);
const selector = native('sel_registerName', 'pointer', ['pointer']);
const getClass = native('object_getClass', 'pointer', ['pointer']);
const superClass = native('class_getSuperclass', 'pointer', ['pointer']);
const className = native('class_getName', 'pointer', ['pointer']);
const getMethod = native('class_getInstanceMethod', 'pointer', ['pointer', 'pointer']);
const getImp = native('method_getImplementation', 'pointer', ['pointer']);
const setImp = native('method_setImplementation', 'pointer', ['pointer', 'pointer']);
const getTypes = native('method_getTypeEncoding', 'pointer', ['pointer']);
const isMain = native('pthread_main_np', 'int', []);
const sendBool = native('objc_msgSend', 'bool', ['pointer', 'pointer']);
const dispatch = native('dispatch_async_f', 'void', ['pointer', 'pointer', 'pointer']);
const mainQueue = Module.getGlobalExportByName('_dispatch_main_q');
const appSlot = Module.getGlobalExportByName('NSApp');
const eventSelector = selector(Memory.allocUtf8String('sendEvent:'));
const runningSelector = selector(Memory.allocUtf8String('isRunning'));
const queued = [];
let state = null, callback = null, installed = false, pinned = false, attempted = false;
let calls = 0, completed = 0, inFlight = 0, wrongReceiver = 0, wrongThread = 0;

function onMain(action) {
  return new Promise((resolve, reject) => {
    const fn = new NativeCallback(() => {
      try { resolve(action()); } catch (error) { reject(error); }
      finally { Script.unpin(); }
    }, 'void', ['pointer']);
    queued.push(fn); // Retain until script unload, including native return.
    Script.pin();
    try { dispatch(mainQueue, ptr(0), fn); }
    catch (error) { Script.unpin(); reject(error); }
  });
}

function status() {
  return { calls, completed, inFlight, wrongReceiver, wrongThread, installed,
    restored: state !== null && getImp(state.method).equals(state.original),
    running: state !== null && sendBool(state.app, runningSelector),
    mainThread: isMain() === 1 };
}

rpc.exports = {
  identity() {
    return { pid: Process.id, executable: Process.mainModule.path,
      arch: Process.arch, policy: Process.codeSigningPolicy, runtime: Script.runtime };
  },
  install() {
    return onMain(() => {
      if (attempted) throw new Error('Do not replay a Hook installation');
      const app = appSlot.readPointer();
      if (app.isNull()) return { ready: false };
      if (!sendBool(app, runningSelector)) return { ready: false };
      attempted = true;
      const actualClass = getClass(app);
      const method = getMethod(actualClass, eventSelector);
      if (method.isNull()) throw new Error('No actual host sendEvent: method');
      const types = getTypes(method).readUtf8String();
      if (!/^v\d*@\d*:\d*@\d*$/.test(types))
        throw new Error('Unexpected sendEvent: ABI: ' + types);
      let owner = actualClass;
      for (;;) {
        const parent = superClass(owner);
        if (parent.isNull() || !getMethod(parent, eventSelector).equals(method)) break;
        owner = parent;
      }
      const original = getImp(method);
      const image = Process.findModuleByAddress(original);
      const originalCall = new NativeFunction(original, 'void', ['pointer', 'pointer', 'pointer'],
        { exceptions: 'propagate' });
      state = { app, method, original };
      callback = new NativeCallback((self, command, event) => {
        if (!self.equals(app)) wrongReceiver++;
        if (isMain() !== 1) wrongThread++;
        calls++; inFlight++;
        try { originalCall(self, command, event); completed++; }
        finally { inFlight--; }
      }, 'void', ['pointer', 'pointer', 'pointer']);
      Script.pin(); pinned = true;
      installed = true; // An exception cannot prove setImp did not execute.
      const previous = setImp(method, callback);
      if (!previous.equals(original)) throw new Error('Host IMP changed concurrently');
      return { ready: true, receiver: app.toString(), selector: 'sendEvent:', types,
        actualClass: className(actualClass).readUtf8String(),
        declaringClass: className(owner).readUtf8String(),
        originalModule: image === null ? null : image.name,
        originalOffset: image === null ? null : original.sub(image.base).toString(),
        mechanism: 'objc-imp-swap', mainThread: isMain() === 1 };
    });
  },
  snapshot() { return onMain(status); },
  restore() {
    return onMain(() => {
      if (state === null) throw new Error('No Hook state');
      if (installed) {
        if (!getImp(state.method).equals(callback))
          throw new Error('IMP ownership lost; terminate owned process without overwriting it');
        setImp(state.method, state.original);
        if (!getImp(state.method).equals(state.original)) throw new Error('Restore not confirmed');
        installed = false;
      }
      if (inFlight !== 0) throw new Error('Native callback has not drained');
      if (pinned) { Script.unpin(); pinned = false; }
      return status();
    });
  }
};
