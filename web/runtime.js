async function injectraRuntime(request, execute) {
  const key = Symbol.for("injectra.runtime.v1");
  const registry = globalThis[key] ??= new Map();
  const { id } = request;
  const busy = registry.busy ??= new Set();
  if (busy.has(id)) throw new Error(`Bundle operation already in progress: ${id}`);
  busy.add(id);
  try {

    async function dispose(entry) {
      const errors = [];
      for (const cleanup of entry.cleanups.slice().reverse()) {
        try { await cleanup(); } catch (error) { errors.push(String(error)); }
      }
      for (const node of entry.nodes) node.remove();
      if (errors.length) throw new Error(`Cleanup incomplete: ${errors.join("; ")}`);
    }

    const previous = registry.get(id);
    if (previous) {
      // Remove ownership first: a failing callback must not be silently retried.
      registry.delete(id);
      await dispose(previous);
    }
    if (request.operation === "remove") {
      return { id, status: previous ? "removed" : "absent" };
    }
    if (request.operation !== "apply") throw new Error("Unknown operation");
    if (!document.head || !document.body) throw new Error("Page not ready: document head/body missing");

    const entry = { nodes: [], cleanups: [] };
    try {
      if (request.css) {
        const style = document.createElement("style");
        style.dataset.injectra = id;
        style.textContent = request.css;
        entry.nodes.push(style);
        document.head.append(style);
      }
      let root = null;
      if (request.html) {
        root = document.createElement("div");
        root.dataset.injectra = id;
        root.innerHTML = request.html;
        entry.nodes.push(root);
        document.body.append(root);
      }
      await execute(Object.freeze({
        root,
        onCleanup(callback) {
          if (typeof callback !== "function") throw new TypeError("cleanup must be a function");
          entry.cleanups.push(callback);
        },
      }));
      registry.set(id, entry);
      return { id, status: previous ? "updated" : "applied" };
    } catch (error) {
      try { await dispose(entry); }
      catch (cleanupError) { throw new Error(`${error}; ${cleanupError}`); }
      throw error;
    }
  } finally {
    busy.delete(id);
  }
}
