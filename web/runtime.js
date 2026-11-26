async function injectraRuntime(request, execute) {
  const key = Symbol.for("injectra.runtime.v1");
  const registry = globalThis[key] ??= new Map();
  if (!request || typeof request !== "object") throw new Error("Invalid runtime request");
  if (request.operation === "list") {
    return {
      bundles: Array.from(registry.entries(), ([bundleId, entry]) => ({
        id: bundleId,
        nodes: entry.nodes.length,
        cleanups: entry.cleanups.length,
      })),
    };
  }
  const { id } = request;
  const busy = registry.busy ??= new Set();
  if (busy.has(id)) throw new Error(`Bundle operation already in progress: ${id}`);
  busy.add(id);

  function isPolicyError(error) {
    const text = String(error && error.message ? error.message : error);
    const name = error && error.name ? String(error.name) : "";
    return (
      name === "SecurityError" ||
      /TrustedHTML|TrustedScript|TrustedScriptURL|Trusted Types|Content Security Policy/i.test(text)
    );
  }

  function wrapMountError(stage, error, hint) {
    const text = String(error && error.message ? error.message : error);
    if (isPolicyError(error)) {
      return new Error(`[injectra:policy:${stage}] ${hint ? `${hint}: ` : ""}${text}`);
    }
    return new Error(`[injectra:mount:${stage}] ${text}`);
  }

  function mountDomNodes(parent, nodes, svgNs) {
    const SVG = "http://www.w3.org/2000/svg";
    for (const spec of nodes) {
      if (typeof spec === "string") {
        parent.appendChild(document.createTextNode(spec));
        continue;
      }
      if (!spec || typeof spec !== "object" || typeof spec.t !== "string") {
        throw new Error("Invalid declarative DOM node");
      }
      const lowerTag = spec.t.toLowerCase();
      if (lowerTag === "script") {
        throw new Error("Declarative DOM mount does not allow <script> elements; put behavior in JS");
      }
      const useSvg = svgNs || spec.ns === "svg" || lowerTag === "svg";
      const el = useSvg
        ? document.createElementNS(SVG, spec.t)
        : document.createElement(spec.t);
      if (Array.isArray(spec.a)) {
        for (const pair of spec.a) {
          if (!Array.isArray(pair) || pair.length !== 2) continue;
          const attrName = String(pair[0]);
          const attrValue = String(pair[1]);
          if (/^on/i.test(attrName)) {
            throw new Error(`Declarative DOM mount does not allow inline event attribute: ${attrName}`);
          }
          el.setAttribute(attrName, attrValue);
        }
      }
      if (Array.isArray(spec.c) && spec.c.length > 0) {
        mountDomNodes(el, spec.c, useSvg && lowerTag !== "foreignobject");
      }
      parent.appendChild(el);
    }
  }

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
        try {
          const style = document.createElement("style");
          style.dataset.injectra = id;
          style.textContent = request.css;
          entry.nodes.push(style);
          document.head.append(style);
        } catch (error) {
          throw wrapMountError("css-mount", error, "Host page policy blocked <style> mount");
        }
      }
      let root = null;
      if (Array.isArray(request.dom) && request.dom.length > 0) {
        try {
          root = document.createElement("div");
          root.dataset.injectra = id;
          mountDomNodes(root, request.dom, false);
          entry.nodes.push(root);
          document.body.append(root);
        } catch (error) {
          throw wrapMountError("dom-mount", error, "Host page policy blocked declarative DOM mount");
        }
      } else if (request.html) {
        try {
          root = document.createElement("div");
          root.dataset.injectra = id;
          root.innerHTML = request.html;
          entry.nodes.push(root);
          document.body.append(root);
        } catch (error) {
          throw wrapMountError(
            "html-mount",
            error,
            'Host page policy (Trusted Types / CSP) blocked innerHTML mount; use html_mode="dom" for declarative DOM construction',
          );
        }
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
