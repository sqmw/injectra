// Shared runtime checks for Thunderbird application UI and embedded content.
import assert from "node:assert/strict";
import { parsedHtmlProgram } from "./parsed-html.mjs";
export async function verifyRuntime({ evaluate, other, runtime, id, check, screenshot, mountMode = "html" }) {
  assert(["html", "dom", "parsed-html"].includes(mountMode), "Unknown fixture mount mode");
  const request = { operation: "apply", id,
    css: '[data-injectra="' + id + '"] button { color: rgb(12, 34, 56); }' +
      'div[data-injectra="' + id + '"] { position:fixed;right:20px;top:80px;z-index:2147483647;' +
      'padding:16px;border:2px solid #2474a5;background:white;color:black; }',
    html: mountMode !== "dom" ? '<button>Injectra Thunderbird test</button>' : "" };
  const program = (value, body = "") => mountMode === "parsed-html"
    ? parsedHtmlProgram(runtime, value, body)
    : "(" + runtime + ")(" + JSON.stringify(value) + ",async context=>{" + body + "})";
  const scriptFor = label => {
    const setup = mountMode === "dom"
      ? "const root=document.createElement('div');root.dataset.injectra=" + JSON.stringify(id) +
        ";const button=document.createElement('button');button.textContent=" + JSON.stringify(label) +
        ";root.append(button);document.body.append(root);context.onCleanup(()=>root.remove());"
      : "const root=context.root;";
    return setup + "globalThis.tbOwnedCount=0;const b=root.querySelector('button');" +
      "const click=()=>{b.dataset.count=String(++globalThis.tbOwnedCount);};b.addEventListener('click',click);" +
      "context.onCleanup(()=>{b.removeEventListener('click',click);delete globalThis.tbOwnedCount;" +
      "globalThis.tbCleaned=(globalThis.tbCleaned??0)+1;});";
  };
  const count = "document.querySelectorAll('[data-injectra]').length";
  let primaryError;
  try {
    await check("page-global execution and DOM", async () => {
      assert.deepEqual(await evaluate("({same:globalThis===window,head:!!document.head,body:!!document.body})"),
        { same: true, head: true, body: true });
    });
    await check(mountMode !== "dom" ? "JS CSS HTML apply and interaction" : "JS CSS apply with DOM-created controls", async () => {
      assert.equal((await evaluate(program(request, scriptFor("Injectra Thunderbird test")))).status, "applied");
      assert.deepEqual(await evaluate("(()=>{const b=document.querySelector('[data-injectra] button');b.click();" +
        "return {count:b.dataset.count,color:getComputedStyle(b).color,nodes:" + count + "};})()"),
        { count: "1", color: "rgb(12, 34, 56)", nodes: 2 });
    });
    await check("other target remains unchanged", async () => {
      assert.deepEqual(await other("({nodes:" + count + ",count:globalThis.tbOwnedCount??null})"),
        { nodes: 0, count: null });
    });
    if (screenshot) await check("visible injected panel screenshot", screenshot);
    await check("same id update without duplicate nodes", async () => {
      assert.equal((await evaluate(program({ ...request, html: mountMode !== "dom" ? "<button>Updated Thunderbird test</button>" : "" }, scriptFor("Updated Thunderbird test")))).status, "updated");
      assert.deepEqual(await evaluate("({nodes:" + count + ",cleaned:globalThis.tbCleaned," +
        "text:document.querySelector('[data-injectra] button').textContent})"),
        { nodes: 2, cleaned: 1, text: "Updated Thunderbird test" });
    });
    await check("remove and registered cleanup", async () => {
      assert.equal((await evaluate(program({ operation: "remove", id }))).status, "removed");
      assert.deepEqual(await evaluate("({nodes:" + count + ",count:globalThis.tbOwnedCount??null,cleaned:globalThis.tbCleaned})"),
        { nodes: 0, count: null, cleaned: 2 });
    });
    await check("absent removal", async () => {
      assert.equal((await evaluate(program({ operation: "remove", id }))).status, "absent");
    });
    await check("Promise result and page exception", async () => {
      assert.equal(await evaluate("new Promise(resolve=>setTimeout(()=>resolve('tb-promise'),30))"), "tb-promise");
      await assert.rejects(evaluate("(()=>{throw Error('tb-expected-error')})()"), /tb-expected-error/);
    });
    await check("failed injection cleanup", async () => {
      await assert.rejects(evaluate(program({ ...request, id: id + ".failure" },
        "context.onCleanup(()=>{globalThis.tbFailedCleanup=true;});throw Error('tb-apply-failure');")), /tb-apply-failure/);
      assert.deepEqual(await evaluate("({nodes:" + count + ",cleaned:globalThis.tbFailedCleanup})"),
        { nodes: 0, cleaned: true });
    });
    await check("cleanup failure remains visible", async () => {
      const failedId = id + ".cleanup";
      await evaluate(program({ ...request, id: failedId }, "context.onCleanup(()=>{throw Error('tb-cleanup-failure');});"));
      await assert.rejects(evaluate(program({ operation: "remove", id: failedId })), /tb-cleanup-failure/);
      assert.equal(await evaluate(count), 0);
    });
  } catch (error) {
    primaryError = error;
    throw error;
  } finally {
    try {
      for (const ownedId of [id, id + ".failure", id + ".cleanup"]) {
        await evaluate(program({ operation: "remove", id: ownedId }));
      }
      await evaluate("(()=>{delete globalThis.tbCleaned;delete globalThis.tbFailedCleanup;delete globalThis.tbOwnedCount;return true;})()");
    } catch (cleanupError) {
      if (primaryError) primaryError.cleanupFailure = cleanupError.message;
      else throw cleanupError;
    }
  }
}
