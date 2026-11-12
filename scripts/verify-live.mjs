// Explicit opt-in validation against a running, user-authorized CDP page.
// Does not launch/quit apps, read conversation text, or use account features.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { once } from "node:events";
import { mkdir, mkdtemp, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { parseArgs } from "node:util";
import { binary, cache } from "./test-env.mjs";

const { values } = parseArgs({ options: { port: { type: "string" }, target: { type: "string" }, dom: { type: "boolean", default: false }, "no-screenshot": {type:"boolean",default:false} } });
const port = Number(values.port);
assert.ok(Number.isInteger(port) && port > 0 && port <= 65535 && values.target,
  "Usage: node scripts/verify-live.mjs --port PORT --target EXACT_PAGE_ID");
await mkdir(resolve(cache, "runs"), { recursive: true });
const run = await mkdtemp(resolve(cache, "runs", "live-"));
const id = `injectra.verify.${Date.now()}`;
const flags = ["--port", String(port), "--target", values.target];
function cli(command, tail) {
  const result = spawnSync(binary, [command, ...flags, ...tail], { encoding: "utf8", timeout: 12000 });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}
const pages = await fetch(`http://127.0.0.1:${port}/json/list`, { signal: AbortSignal.timeout(3000) }).then(r => r.json());
const page = pages.find(p => p.id === values.target && p.type === "page");
assert.ok(page, "Exact page not found");
const endpoint = new URL(page.webSocketDebuggerUrl);
assert.equal(endpoint.hostname, "127.0.0.1");
assert.equal(endpoint.port, String(port));
assert.equal(endpoint.protocol, "ws:");
const socket = new WebSocket(endpoint);
await once(socket, "open");
let next = 0;
function call(method, params) {
  const id = ++next;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => { socket.removeEventListener("message", receive); reject(Error("CDP verification timeout")); }, 6000);
    function receive(event) {
      const reply = JSON.parse(event.data);
      if (reply.id !== id) return;
      clearTimeout(timer);
      socket.removeEventListener("message", receive);
      if (reply.error || reply.result?.exceptionDetails) reject(Error(JSON.stringify(reply)));
      else resolve(reply.result);
    }
    socket.addEventListener("message", receive);
    socket.send(JSON.stringify({ id, method, params }));
  });
}
const evaluate = async expression => (await call("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true })).result.value;
const selector = JSON.stringify(`[data-injectra="${id}"]`);
const inspect = `(() => { const nodes = document.querySelectorAll(${selector}); const root = [...nodes].find(n => n.tagName === 'DIV'); const button = root?.querySelector('button'); return {nodes:nodes.length, clicks:root?.dataset.clicks ?? null, background:button ? getComputedStyle(button).backgroundColor : null}; })()`;

await writeFile(resolve(run, "manifest.json"), JSON.stringify({ version: 1, id, css: "style.css", html: "panel.html", js: "behavior.js" }));
await writeFile(resolve(run, "style.css"), `[data-injectra="${id}"] section {position:fixed;top:110px;right:28px;z-index:2147483647;background:#17233b;color:white;padding:20px;border-radius:14px;font:15px/1.5 system-ui;box-shadow:0 8px 28px #0004;width:240px} [data-injectra="${id}"] button {background:rgb(76, 85, 213);color:white;border:0;padding:8px 14px;border-radius:8px;cursor:pointer;font:inherit}`);
await writeFile(resolve(run, "panel.html"), '<section aria-label="Injectra兼容性验证"><strong>Injectra · 实机验证</strong><p>CSS / HTML / JS 已挂载</p><button type="button">验证交互 · 0</button></section>');
await writeFile(resolve(run, "behavior.js"), 'const root=context.root; const button=root.querySelector("button"); root.dataset.clicks="0"; const click=()=>{root.dataset.clicks=String(Number(root.dataset.clicks)+1);button.textContent="验证交互 · "+root.dataset.clicks;};context.onCleanup(()=>button.removeEventListener("click",click));button.addEventListener("click",click);');
if (values.dom) {
  await writeFile(resolve(run, "panel.html"), "");
  await writeFile(resolve(run, "behavior.js"), `const root=document.createElement('div');root.dataset.injectra=${JSON.stringify(id)};
const section=document.createElement('section'),heading=document.createElement('strong'),text=document.createElement('p'),button=document.createElement('button');
heading.textContent='Injectra';text.textContent='JS DOM / CSS';button.type='button';button.textContent='Verify';section.append(heading,text,button);root.append(section);document.body.append(root);
root.dataset.clicks='0';const click=()=>{root.dataset.clicks=String(Number(root.dataset.clicks)+1);};button.addEventListener('click',click);context.onCleanup(()=>{button.removeEventListener('click',click);root.remove();});`);
}
const pageUrl = new URL(page.url);
const report = { id, platform: process.platform, route: `${pageUrl.protocol}//${pageUrl.host}`, mode: values.dom ? "js-dom" : "html-string", checks: {}, cleaned: false };
try {
  report.checks.before = await evaluate(inspect);
  assert.equal(report.checks.before.nodes, 0);
  report.checks.apply = cli("apply", ["--bundle", resolve(run, "manifest.json")]);
  report.checks.mounted = await evaluate(inspect);
  assert.equal(report.checks.mounted.nodes, 2);
  assert.equal(report.checks.mounted.background, "rgb(76, 85, 213)");
  await evaluate(`document.querySelector(${selector} + ' button').click(); true`);
  report.checks.interaction = await evaluate(inspect);
  assert.equal(report.checks.interaction.clicks, "1");
  if (!values["no-screenshot"]) {
  const clip = await evaluate(`(() => {const r=document.querySelector(${selector} + ' section').getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height,scale:1};})()`);
  const screenshot = await call("Page.captureScreenshot", { format: "png", clip });
  await writeFile(resolve(run, "injected-panel.png"), Buffer.from(screenshot.data, "base64"));
  } else report.screenshot = "not-requested";
  report.checks.update = cli("apply", ["--bundle", resolve(run, "manifest.json")]);
  assert.equal(report.checks.update.status, "updated");
  report.checks.updated = await evaluate(inspect);
  assert.equal(report.checks.updated.nodes, 2);
  assert.equal(report.checks.updated.clicks, "0");
  report.checks.remove = cli("remove", ["--id", id]);
  report.checks.after = await evaluate(inspect);
  assert.equal(report.checks.after.nodes, 0);
  assert.equal(cli("remove", ["--id", id]).status, "absent");
  report.cleaned = true;
  report.passed = true;
} catch (error) {
  report.error = String(error);
  throw error;
} finally {
  if (!report.cleaned) {
    try { cli("remove", ["--id", id]); report.cleaned = (await evaluate(inspect)).nodes === 0; }
    catch (error) { report.cleanupError = String(error); }
  }
  await writeFile(resolve(run, "result.json"), JSON.stringify(report, null, 2));
  socket.close();
  console.log(JSON.stringify({ ...report, evidenceDirectory: run }, null, 2));
}
