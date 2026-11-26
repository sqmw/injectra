import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { once } from "node:events";
import { copyFile, mkdir, mkdtemp, writeFile } from "node:fs/promises";
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

function cliMachine(args, expected = 0) {
  const result = spawnSync(binary, ["--machine", ...args], { encoding: "utf8", timeout: 12000 });
  if (result.error) throw result.error;
  assert.equal(result.status, expected, `${result.stdout}\n${result.stderr}`);
  assert.equal(result.stderr, "", `expected empty stderr in --machine mode, got: ${result.stderr}`);
  const lines = result.stdout.trimEnd().split("\n");
  assert.equal(lines.length, 1, `expected single-line JSON in --machine mode, got ${lines.length} lines`);
  return JSON.parse(lines[0]);
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
  let ttPage;
  for (let attempt = 0; attempt < 100; attempt++) {
    if (child.exitCode !== null) throw Error(`Electron exited: ${log}`);
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/list`, { signal: AbortSignal.timeout(300) });
      const list = await response.json();
      page = list.find(p => p.type === "page" && p.title === "Injectra · Electron fixture");
      ttPage = list.find(p => p.type === "page" && p.title === "Injectra · Trusted Types fixture");
      if (page && ttPage) break;
    } catch {}
    await delay(100);
  }
  assert.ok(page && ttPage, `fixtures not ready: ${log}`);
  const discovered = cli(["targets", "--port", String(port)]);
  assert.ok(discovered.some(target => target.id === page.id));
  for (const target of discovered) assert.deepEqual(Object.keys(target).sort(), ["id", "title", "url"]);
  assert.match(cli(["targets", "--port", "0"], 1), /^injectra: /);
  assert.match(cli(["apply"], 2), /--port/);

  const machineTargets = cliMachine(["targets", "--port", String(port)], 0);
  assert.equal(machineTargets.ok, true);
  assert.ok(Array.isArray(machineTargets.data));
  const machineCliErr = cliMachine(["apply"], 2);
  assert.equal(machineCliErr.ok, false);
  assert.equal(machineCliErr.error.kind, "InvalidInput");
  assert.equal(machineCliErr.error.phase, "Prepare");
  assert.equal(machineCliErr.error.certainty, "NotExecuted");

  socket = new WebSocket(page.webSocketDebuggerUrl);
  await once(socket, "open");
  const flags = ["--port", String(port), "--target", page.id];
  assert.deepEqual(cli(["list", ...flags]), { bundles: [] });
  // Minimal, test-owned input; no dependency on the separate plugin library.
  const manifest = resolve(run, "fixture.json");
  await writeFile(manifest, JSON.stringify({version:1,id:"injectra.test.fixture",css:"fixture.css",html:"fixture.html",js:"fixture.js"}));
  await writeFile(resolve(run,"fixture.css"),'[data-injectra] button { background:rgb(91,75,219); }');
  await writeFile(resolve(run,"fixture.html"),'<button type="button">Fixture</button>');
  await writeFile(resolve(run,"fixture.js"),'const button=context.root.querySelector("button");const click=()=>document.documentElement.dataset.injectraSmoke="1";button.addEventListener("click",click);context.onCleanup(()=>{button.removeEventListener("click",click);delete document.documentElement.dataset.injectraSmoke;});');
  assert.equal(cli(["apply", ...flags, "--bundle", manifest]).status, "applied");
  assert.deepEqual(cli(["list", ...flags]), {
    bundles: [{ id: "injectra.test.fixture", nodes: 2, cleanups: 1 }],
  });
  assert.equal(await evaluate('document.querySelectorAll("[data-injectra]").length'), 2);
  assert.equal(await evaluate('getComputedStyle(document.querySelector("[data-injectra] button")).backgroundColor'), "rgb(91, 75, 219)");
  assert.equal(await evaluate('document.querySelector("[data-injectra] button").click(); document.documentElement.dataset.injectraSmoke'), "1");
  assert.equal(cli(["apply", ...flags, "--bundle", manifest]).status, "updated");
  assert.equal(await evaluate('document.querySelectorAll("[data-injectra]").length'), 2);
  assert.equal(await evaluate('document.documentElement.dataset.injectraSmoke ?? null'), null);
  assert.equal(cli(["remove", ...flags, "--id", "injectra.test.fixture"]).status, "removed");
  assert.deepEqual(cli(["list", ...flags]), { bundles: [] });
  assert.equal(await evaluate('document.querySelectorAll("[data-injectra]").length'), 0);
  assert.equal(cli(["remove", ...flags, "--id", "injectra.test.fixture"]).status, "absent");
  assert.match(cli(["apply", "--port", String(port), "--target", "missing-target", "--bundle", manifest], 1), /exactly one/);

  const machineMissing = cliMachine(["apply", "--port", String(port), "--target", "missing-target", "--bundle", manifest], 3);
  assert.equal(machineMissing.ok, false);
  assert.equal(machineMissing.error.kind, "TargetNotFound");
  assert.equal(machineMissing.error.certainty, "NotExecuted");

  assert.equal(cli(["eval", ...flags, "--expression", "document.title"]), "Injectra · Electron fixture");
  assert.equal(cli(["eval", ...flags, "--expression", "Promise.resolve(42)"]), 42);
  assert.equal(cli(["eval", ...flags, "--expression", "void 0"]), null);
  const probeFile = resolve(run, "probe.js");
  await writeFile(probeFile, '({ title: document.title, count: document.querySelectorAll("[data-injectra]").length }) // comment\n');
  assert.deepEqual(cli(["eval", ...flags, "--file", probeFile]), { title: "Injectra · Electron fixture", count: 0 });
  const machineEvalBlank = cliMachine(["eval", ...flags, "--expression", "   "], 3);
  assert.equal(machineEvalBlank.error.kind, "InvalidInput");
  assert.equal(machineEvalBlank.error.certainty, "NotExecuted");
  const machineEvalErr = cliMachine(["eval", ...flags, "--expression", "(() => { throw new Error('eval-boom'); })()"], 4);
  assert.equal(machineEvalErr.error.kind, "Execution");
  assert.equal(machineEvalErr.error.certainty, "Executed");

  await writeFile(resolve(run, "bad.js"), 'context.onCleanup(() => { document.body.removeAttribute("data-test"); }); document.body.dataset.test = "partial"; throw new Error("intentional-script-failure");');
  await writeFile(resolve(run, "bad.css"), "body { color: red; }");
  await writeFile(resolve(run, "bad.json"), JSON.stringify({ version: 1, id: "bad", js: "bad.js", css: "bad.css" }));
  assert.match(cli(["apply", ...flags, "--bundle", resolve(run, "bad.json")], 1), /intentional-script-failure/);
  const machineBad = cliMachine(["apply", ...flags, "--bundle", resolve(run, "bad.json")], 4);
  assert.equal(machineBad.ok, false);
  assert.equal(machineBad.error.kind, "Execution");
  assert.equal(machineBad.error.phase, "Execute");
  assert.equal(machineBad.error.certainty, "Executed");
  assert.deepEqual(cli(["list", ...flags]), { bundles: [] });
  assert.equal(await evaluate('document.querySelectorAll("[data-injectra]").length'), 0);
  assert.equal(await evaluate('document.body.hasAttribute("data-test")'), false);

  // Real Chromium Trusted Types ("require-trusted-types-for 'script'; trusted-types 'none'") verification:
  const ttFlags = ["--port", String(port), "--target", ttPage.id];
  const ttInnerBlocked = cliMachine(["apply", ...ttFlags, "--bundle", manifest], 4);
  assert.equal(ttInnerBlocked.ok, false);
  assert.equal(ttInnerBlocked.error.kind, "PolicyBlocked");
  assert.equal(ttInnerBlocked.error.phase, "Execute");
  assert.equal(ttInnerBlocked.error.certainty, "Executed");
  assert.deepEqual(cli(["list", ...ttFlags]), { bundles: [] });

  const ttDomManifest = resolve(run, "tt-dom.json");
  await writeFile(resolve(run, "tt-dom.css"), '[data-injectra="injectra.test.tt.dom"] button { color: rgb(11, 22, 33); }');
  await writeFile(ttDomManifest, JSON.stringify({
    version: 1,
    id: "injectra.test.tt.dom",
    html_mode: "dom",
    css: "tt-dom.css",
    html: "tt-dom.html",
    js: "tt-dom.js",
  }));
  await writeFile(resolve(run, "tt-dom.html"), '<button type="button" data-safe="yes">&lt;Safe&gt;</button><svg viewBox="0 0 10 10"><circle r="4"/></svg>');
  await writeFile(resolve(run, "tt-dom.js"), 'const b=context.root.querySelector("button");const click=()=>document.documentElement.dataset.ttClicked="1";b.addEventListener("click",click);context.onCleanup(()=>{b.removeEventListener("click",click);delete document.documentElement.dataset.ttClicked;});');
  assert.equal(cli(["apply", ...ttFlags, "--bundle", ttDomManifest]).status, "applied");
  assert.equal(cli(["eval", ...ttFlags, "--expression", 'document.querySelector("[data-injectra] button").textContent']), "<Safe>");
  assert.equal(cli(["eval", ...ttFlags, "--expression", 'getComputedStyle(document.querySelector("[data-injectra] button")).color']), "rgb(11, 22, 33)");
  assert.equal(cli(["eval", ...ttFlags, "--expression", 'document.querySelector("[data-injectra] circle").namespaceURI']), "http://www.w3.org/2000/svg");
  assert.equal(cli(["eval", ...ttFlags, "--expression", '(() => { document.querySelector("[data-injectra] button").click(); return document.documentElement.dataset.ttClicked; })()']), "1");
  assert.equal(cli(["apply", ...ttFlags, "--bundle", ttDomManifest]).status, "updated");
  assert.equal(cli(["eval", ...ttFlags, "--expression", 'document.querySelectorAll("[data-injectra]").length']), 2);
  assert.equal(cli(["remove", ...ttFlags, "--id", "injectra.test.tt.dom"]).status, "removed");
  assert.deepEqual(cli(["list", ...ttFlags]), { bundles: [] });
  assert.equal(cli(["eval", ...ttFlags, "--expression", 'document.querySelectorAll("[data-injectra]").length']), 0);

  const appPath = resolve(electron, "../../..");
  const plan = cli(["launch-plan", "--app", appPath, "--port", String(port)]);
  assert.equal(plan.executable, electron);
  assert.deepEqual(Object.keys(plan).sort(), ["arguments", "executable", "note"]);
  assert.deepEqual(plan.arguments, [`--remote-debugging-port=${port}`, "--remote-debugging-address=127.0.0.1"]);

  const inspected = cli(["inspect-app", "--app", appPath, "--port", String(port)]);
  assert.equal(inspected.host_family, "electron");
  assert.equal(inspected.engine, "chromium");
  assert.equal(inspected.routes.find(r => r.route === "cdp")?.supported, true);
  assert.equal(inspected.routes.find(r => r.route === "electron-gadget")?.supported, false);
  assert.equal(inspected.launch_plan?.executable, electron);

  const dummyApp = resolve(run, "DummyNative.app");
  await mkdir(resolve(dummyApp, "Contents/MacOS"), { recursive: true });
  await copyFile("/usr/bin/true", resolve(dummyApp, "Contents/MacOS/DummyNative"));
  await writeFile(
    resolve(dummyApp, "Contents/Info.plist"),
    '<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>CFBundleExecutable</key><string>DummyNative</string><key>CFBundleIdentifier</key><string>io.injectra.dummy.native</string></dict></plist>',
  );
  const unsupported = cliMachine(["inspect-app", "--app", dummyApp], 3);
  assert.equal(unsupported.ok, false);
  assert.equal(unsupported.error.kind, "Unsupported");
  assert.equal(unsupported.error.certainty, "NotExecuted");

  const version = await fetch(`http://127.0.0.1:${port}/json/version`).then(r => r.json());
  const evidence = { passed: true, platform: process.platform, arch: process.arch, electron: requireTool("electron/package.json").version,
    browser: version.Browser, checks: ["discovery", "CSS computed style", "HTML mount", "JS interaction", "replace without duplicates", "cleanup", "absent removal", "missing target rejection", "script failure cleanup", "macOS launch plan", "bundle list", "page eval", "machine contract", "inspect-app", "Trusted Types PolicyBlocked separation", "Trusted Types declarative DOM mount"] };
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
