// Test-only: change an owned subclass's IMP, never NSObject or an app class.
let activeCallback = null;
let verified = false;
const native = (name, result, args) => new NativeFunction(
  Module.getGlobalExportByName(name), result, args);

rpc.exports = {
  identity() {
    return { pid: Process.id, executable: Process.mainModule.path,
      arch: Process.arch, policy: Process.codeSigningPolicy, runtime: Script.runtime };
  },
  verify() {
    if (verified) throw new Error('One verification per script; do not replay unknown results');
    verified = true;
    if (Process.platform !== 'darwin' || Process.pointerSize !== 8)
      throw new Error('This fixture requires 64-bit Darwin');
    const cls = native('objc_getClass', 'pointer', ['pointer']);
    const sel = native('sel_registerName', 'pointer', ['pointer']);
    const method = native('class_getInstanceMethod', 'pointer', ['pointer', 'pointer']);
    const getImp = native('method_getImplementation', 'pointer', ['pointer']);
    const setImp = native('method_setImplementation', 'pointer', ['pointer', 'pointer']);
    const encoding = native('method_getTypeEncoding', 'pointer', ['pointer']);
    const allocate = native('objc_allocateClassPair', 'pointer', ['pointer', 'pointer', 'ulong']);
    const add = native('class_addMethod', 'bool', ['pointer', 'pointer', 'pointer', 'pointer']);
    const register = native('objc_registerClassPair', 'void', ['pointer']);
    const sendPointer = native('objc_msgSend', 'pointer', ['pointer', 'pointer']);
    const sendHash = native('objc_msgSend', 'ulong', ['pointer', 'pointer']);
    const sendVoid = native('objc_msgSend', 'void', ['pointer', 'pointer']);
    const selector = name => sel(Memory.allocUtf8String(name));
    const root = cls(Memory.allocUtf8String('NSObject'));
    if (root.isNull()) throw new Error('NSObject unavailable');
    const hash = selector('hash');
    const rootMethod = method(root, hash);
    const original = getImp(rootMethod);
    const originalCall = new NativeFunction(original, 'ulong', ['pointer', 'pointer']);
    const name = 'InjectraDispatchProbe_' + Process.id + '_' + Date.now();
    const ownedClass = allocate(root, Memory.allocUtf8String(name), 0);
    if (ownedClass.isNull()) throw new Error('Owned class allocation failed');
    if (!add(ownedClass, hash, original, encoding(rootMethod)))
      throw new Error('Owned class method installation failed');
    register(ownedClass);
    const ownMethod = method(ownedClass, hash);
    if (ownMethod.isNull() || ownMethod.equals(rootMethod))
      throw new Error('Refusing to replace inherited/root method');
    let object = ptr(0), installed = false, pinned = false, calls = 0;
    try {
      object = sendPointer(sendPointer(ownedClass, selector('alloc')), selector('init'));
      if (object.isNull()) throw new Error('Owned object allocation failed');
      const baseline = sendHash(object, hash).toString();
      activeCallback = new NativeCallback((self, command) => {
        if (self.equals(object)) calls++;
        return originalCall(self, command);
      }, 'ulong', ['pointer', 'pointer']);
      Script.pin(); pinned = true;
      // Mark before the native call: an exception does not prove non-execution.
      installed = true;
      const previous = setImp(ownMethod, activeCallback);
      if (!previous.equals(original)) throw new Error('Unexpected previous IMP');
      const first = sendHash(object, hash).toString();
      const second = sendHash(object, hash).toString();
      const hookedCalls = calls;
      setImp(ownMethod, original);
      if (!getImp(ownMethod).equals(original)) throw new Error('Restoration not confirmed');
      installed = false;
      const after = sendHash(object, hash).toString();
      return { className: name, selector: 'hash', baseline, first, second, after,
        hookedCalls, callsAfterRestore: calls, restored: true,
        rootMethodUnchanged: getImp(rootMethod).equals(original), mechanism: 'objc-imp-swap' };
    } finally {
      if (installed) {
        setImp(ownMethod, original);
        if (!getImp(ownMethod).equals(original))
          throw new Error('Unsafe to unload: restoration failed; terminate owned process');
      }
      if (pinned) Script.unpin();
      activeCallback = null;
      if (!object.isNull()) sendVoid(object, selector('release'));
      // Registered class metadata remains until this owned process exits.
    }
  }
};
