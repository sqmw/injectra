// Build a test-only Gecko Debugger operation in a separate system sandbox.
// Execute in the original window global; detach the temporary debugger in all outcomes.
export function pageWorldScript(expression) {
  const source = "(async()=>JSON.stringify(await (" + expression + ")))()";
  return "return (async()=>{" +
    "const {addDebuggerToGlobal}=ChromeUtils.importESModule('moz-src:///devtools/platform/jsdebugger.sys.mjs');" +
    "addDebuggerToGlobal(globalThis);const debuggerInstance=new Debugger();" +
    "try{const debuggee=debuggerInstance.addDebuggee(window);" +
    "const completion=debuggee.executeInGlobal(" + JSON.stringify(source) + ",{bypassCSP:true});" +
    "if(!completion)throw Error('Gecko debugger execution terminated');" +
    "if('throw' in completion){const value=completion.throw;" +
    "throw Error(String(value instanceof Debugger.Object?value.unsafeDereference():value));}" +
    "const value=completion.return;" +
    "return await (value instanceof Debugger.Object?value.unsafeDereference():value);" +
    "}finally{debuggerInstance.removeAllDebuggees();}})();";
}
