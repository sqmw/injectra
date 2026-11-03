import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { once } from "node:events";
import { mkdir, mkdtemp, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { binary, cache, root, requireTool } from "./test-env.mjs";

if (process.platform !== "darwin") throw Error("This smoke test certifies macOS only");
await mkdir(resolve(cache, "runs"), { recursive: true });
const run = await mkdtemp(resolve(cache, "runs", "electron-"));
const reservation = createServer();
reservation.listen(0, "127.0.0.1");
await once(reservation, "listening");
const port = reservation.address().port;
await new Promise(resolve => reservation.close(resolve));
const env = { ...process.env, INJECTRA_FIXTURE_PORT: String(port), INJECTRA_FIXTURE_PROFILE: resolve(run, "profile") };
delete env.ELECTRON_RUN_AS_NODE;
const electron = requireTool("electron");
const child = spawn(electron, [resolve(root, "tests/electron/main.cjs")], { env, stdio: ["ignore", "pipe", "pipe"] });
let log = "";
child.stdout.on("data", data => { log += data; });
child.stderr.on("data", data => { log += data; });
const exited = once(child, "exit");
let socket;

function cli(args, expected = 0) {
  const result = spawnSync(binary, args, { encoding: "utf8", timeout: 12000 });
  if (result.error) throw result.error;
  assert.equal(result.status, expected, result.stderr);
  return expected === 0 ? JSON.parse(result.stdout) : result.stderr;
}

let commandId = 0;
async function evaluate(expression) {
  const id = ++commandId;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => { socket.removeEventListener("message", receive); reject(Error("fixture evaluation timed out")); }, 5000);
    function receive(event) {
      const reply = JSON.parse(event.data);
      if (reply.id !== id) return;
      clearTimeout(timer);
      socket.removeEventListener("message", receive);
      if (reply.error || reply.result?.exceptionDetails) reject(Error(JSON.stringify(reply)));
      else resolve(reply.result.result.value);
    }
    socket.addEventListener("message", receive);
    socket.send(JSON.stringify({ id, method: "Runtime.evaluate", params: { expression, returnByValue: true, awaitPromise: true } }));
  });
}

try {
  let page;
  for (let attempt = 0; attempt < 100; attempt++) {
    if (child.exitCode !== null) throw Error(`Electron exited: ${log}`);
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/list`, { signal: AbortSignal.timeout(300) });
      page = (await response.json()).find(page => page.type === "page" && page.title === "Injectra · Electron fixture");
      if (page) break;
    } catch {}
    await delay(100);
  }
  assert.ok(page, `fixture not ready: ${log}`);
  const discovered = cli(["targets", "--port", String(port)]);
  assert.ok(discovered.some(target => target.id === page.id));
  for (const target of discovered) assert.deepEqual(Object.keys(target).sort(), ["id", "title", "url"]);
  assert.match(cli(["targets", "--port", "0"], 1), /^injectra: /);
  assert.match(cli(["apply"], 2), /--port/);
  socket = new WebSocket(page.webSocketDebuggerUrl);
  await once(socket, "open");
  const flags = ["--port", String(port), "--target", page.id];
  const manifest = resolve(root, "examples/accent/manifest.json");
  assert.equal(cli(["apply", ...flags, "--bundle", manifest]).status, "applied");
  assert.equal(await evaluate('document.querySelectorAll("[data-injectra]").length'), 2);
  assert.equal(await evaluate('getComputedStyle(document.querySelector("[data-injectra] button")).backgroundColor'), "rgb(91, 75, 219)");
  assert.equal(await evaluate('document.querySelector("[data-injectra] button").click(); document.documentElement.style.zoom'), "0.9");
  assert.equal(cli(["apply", ...flags, "--bundle", manifest]).status, "updated");
  assert.equal(await evaluate('document.querySelectorAll("[data-injectra]").length'), 2);
  assert.equal(await evaluate('document.documentElement.style.zoom'), "");
  assert.equal(cli(["remove", ...flags, "--id", "injectra.accent"]).status, "removed");
  assert.equal(await evaluate('document.querySelectorAll("[data-injectra]").length'), 0);
  assert.equal(cli(["remove", ...flags, "--id", "injectra.accent"]).status, "absent");
  assert.match(cli(["apply", "--port", String(port), "--target", "missing-target", "--bundle", manifest], 1), /exactly one/);

  await writeFile(resolve(run, "bad.js"), 'context.onCleanup(() => { document.body.removeAttribute("data-test"); }); document.body.dataset.test = "partial"; throw new Error("intentional-script-failure");');
  await writeFile(resolve(run, "bad.css"), "body { color: red; }");
  await writeFile(resolve(run, "bad.json"), JSON.stringify({ version: 1, id: "bad", js: "bad.js", css: "bad.css" }));
  assert.match(cli(["apply", ...flags, "--bundle", resolve(run, "bad.json")], 1), /intentional-script-failure/);
  assert.equal(await evaluate('document.querySelectorAll("[data-injectra]").length'), 0);
  assert.equal(await evaluate('document.body.hasAttribute("data-test")'), false);

  const appPath = resolve(electron, "../../..");
  const plan = cli(["launch-plan", "--app", appPath, "--port", String(port)]);
  assert.equal(plan.executable, electron);
  assert.deepEqual(Object.keys(plan).sort(), ["arguments", "executable", "note"]);
  assert.deepEqual(plan.arguments, [`--remote-debugging-port=${port}`, "--remote-debugging-address=127.0.0.1"]);
  const version = await fetch(`http://127.0.0.1:${port}/json/version`).then(r => r.json());
  const evidence = { passed: true, platform: process.platform, arch: process.arch, electron: requireTool("electron/package.json").version,
    browser: version.Browser, checks: ["discovery", "CSS computed style", "HTML mount", "JS interaction", "replace without duplicates", "cleanup", "absent removal", "missing target rejection", "script failure cleanup", "macOS launch plan"] };
  await writeFile(resolve(run, "result.json"), JSON.stringify(evidence, null, 2));
  console.log(JSON.stringify({ ...evidence, evidenceDirectory: run }, null, 2));
} finally {
  socket?.close();
  if (child.exitCode === null) child.kill("SIGTERM");
  await Promise.race([exited, delay(5000)]);
  if (child.exitCode === null && child.signalCode === null) {
    child.kill("SIGKILL"); // Only the fixture process spawned by this script.
    await exited;
  }
  await writeFile(resolve(run, "electron.log"), log);
}
