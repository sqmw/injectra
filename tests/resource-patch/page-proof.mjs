// Minimal test input, not a reusable plugin. The runtime is read from web/runtime.js.
export function pageProof({ runtime, nonce, endpoint }) {
  return `(() => {
    const nonce = ${JSON.stringify(nonce)};
    const endpoint = ${JSON.stringify(endpoint)};
    const id = "injectra.test.resource-proof";
    const receipts = [];
    function report(phase, extra = {}) {
      const url = new URL(endpoint);
      for (const [key, value] of Object.entries({ nonce, phase, path: location.pathname, ready: document.readyState, ...extra })) url.searchParams.set(key, String(value));
      const image = new Image(); receipts.push(image); image.src = url.href;
    }
    globalThis.__injectraResourceProof = { nonce, receipts };
    report("loaded");
    ${runtime}
    async function apply() {
      try {
        if (location.protocol !== "file:" || !location.pathname.endsWith("/dist/index.html")) throw new Error("Unexpected page");
        const request = { operation: "apply", id,
          css: '[data-injectra="' + id + '"] { position:fixed!important;top:20px!important;right:20px!important;z-index:2147483647!important;background:#35206b!important;color:#fff!important;padding:16px!important;border:2px solid #ba9aff!important;border-radius:12px!important;font:14px/1.7 system-ui!important;box-shadow:0 4px 24px #0005!important } [data-injectra="' + id + '"] button { background:#fff!important;color:#35206b!important;border:0!important;border-radius:6px!important;padding:6px 12px!important;cursor:pointer!important }',
          html: '<section aria-label="Injectra 页面执行验证"><strong>Injectra 页面执行成功</strong><div>测试标识：' + nonce.slice(0, 10) + '</div><button type="button">验证页面交互</button><div role="status">等待点击验证</div></section>' };
        const result = await injectraRuntime(request, async context => {
          const button = context.root.querySelector("button");
          const click = () => { context.root.querySelector('[role="status"]').textContent = "交互验证通过"; report("clicked", { connected: context.root.isConnected }); };
          button.addEventListener("click", click);
          context.onCleanup(() => button.removeEventListener("click", click));
          context.root.dataset.proofNonce = nonce;
        });
        const node = document.querySelector('[data-injectra="' + id + '"][data-proof-nonce="' + nonce + '"]');
        report("applied", { status: result.status, connected: !!node?.isConnected,
          appRootPresent: !!document.getElementById("root"), color: node ? getComputedStyle(node).backgroundColor : "" });
      } catch (error) { report("error", { error: String(error).slice(0, 200) }); }
    }
    if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", apply, { once: true });
    else apply();
  })();\n`;
}
