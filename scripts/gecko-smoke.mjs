// Explicit, optional macOS Firefox validation; not a product backend.
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { spawn, execFileSync } from "node:child_process";
import { once } from "node:events";
import { mkdir, mkdtemp, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { root, cache, binary } from "./test-env.mjs";
import { BidiClient } from "../tests/gecko/bidi-client.mjs";

if (process.platform !== "darwin") throw new Error("This validation currently owns macOS Firefox only");
const pinned = JSON.parse(await readFile(join(root, "tests/gecko/firefox.json"), "utf8"));
const distribution = join(cache, "gecko-tests/firefox", pinned.version);
const receipt = JSON.parse(await readFile(join(distribution, "verified.json"), "utf8"));
assert.equal(receipt.sha512, pinned.sha512);
assert.equal(receipt.version, pinned.version);
const app = join(distribution, "distribution/Firefox.app");
assert.equal(receipt.app, app);
execFileSync("/usr/bin/codesign", ["--verify", "--deep", "--strict", app]);
await mkdir(join(cache, "runs"), { recursive: true });
const run = await mkdtemp(join(cache, "runs/firefox-bidi-"));
const profile = join(run, "profile");
const home = join(run, "home");
await mkdir(profile);
await mkdir(join(home, "Library/Application Support"), { recursive: true });
const preferences = {
  "browser.shell.checkDefaultBrowser": false,
  "browser.aboutwelcome.enabled": false,
  "browser.startup.homepage_override.mstone": "ignore",
  "browser.startup.firstrunSkipsHomepage": true,
  "datareporting.policy.dataSubmissionEnabled": false,
  "toolkit.telemetry.enabled": false,
  "app.update.auto": false,
  "remote.log.level": "Trace",
};
await writeFile(join(profile, "user.js"), Object.entries(preferences)
  .map(([key, value]) => "user_pref(" + JSON.stringify(key) + "," + JSON.stringify(value) + ");").join("\n"));
const runtime = await readFile(join(root, "web/runtime.js"), "utf8");
const server = createServer((request, response) => {
  const name = new URL(request.url, "http://127.0.0.1").pathname === "/other" ? "other" : "main";
  response.writeHead(200, { "Content-Type": "text/html; charset=utf-8", "Cache-Control": "no-store" });
  response.end("<!doctype html><meta charset=utf-8><title>Injectra Gecko " + name +
    "</title><h1 id=original>Owned " + name + " fixture</h1>");
});
server.listen(0, "127.0.0.1");
await once(server, "listening");
const origin = "http://127.0.0.1:" + server.address().port;
const env = { ...process.env, HOME: home, CFFIXED_USER_HOME: home,
  MOZ_NO_REMOTE: "1", MOZ_CRASHREPORTER_DISABLE: "1" };
delete env.MOZ_REMOTE_ALLOW_SYSTEM_ACCESS;
delete env.MOZ_HEADLESS;
let child, client, childExited = false, hostLog = "", spawnError;
const transcript = [];
const result = { passed: false, host: "Firefox", expectedVersion: pinned.version,
  platform: process.platform, arch: process.arch, run, checks: [], productBackend: false };
const check = async (name, action) => {
  await action();
  result.checks.push(name);
  console.log("PASS " + name);
};
const request = { operation: "apply", id: "injectra.gecko.validation",
  css: ".gecko-owned { color: rgb(12, 34, 56); }",
  html: '<button class="gecko-owned">Owned test button</button>' };
const applySource = (value, body) =>
  "(" + runtime + ")(" + JSON.stringify(value) + ", async context => {" + body + "})";
const removeSource = id => "(" + runtime + ")(" + JSON.stringify({ operation: "remove", id }) + ")";
const body = "globalThis.geckoOwnedCount=0; const button=context.root.querySelector('button');" +
  "const click=()=>{button.dataset.count=String(++globalThis.geckoOwnedCount);};" +
  "button.addEventListener('click',click);" +
  "context.onCleanup(()=>{button.removeEventListener('click',click);delete globalThis.geckoOwnedCount;" +
  "globalThis.geckoCleaned=(globalThis.geckoCleaned??0)+1;});";
try {
  child = spawn(join(app, "Contents/MacOS/firefox"), [
    "-no-remote", "-new-instance", "-profile", profile, "--remote-debugging-port", "0", "about:blank",
  ], { env, stdio: ["ignore", "pipe", "pipe"] });
  child.on("error", error => { spawnError = error; });
  child.on("exit", (code, signal) => {
    childExited = true; result.exit = { code, signal };
  });
  child.stdout.on("data", data => { hostLog += data; });
  child.stderr.on("data", data => { hostLog += data; });
  result.pid = child.pid;
  await writeFile(join(run, "owner.json"), JSON.stringify({
    pid: child.pid, executable: join(app, "Contents/MacOS/firefox"), profile,
  }, null, 2));
  const started = Date.now();
  let endpoint;
  while (!endpoint) {
    if (spawnError) throw spawnError;
    if (childExited) throw new Error("Owned Firefox exited before BiDi was ready");
    const match = hostLog.match(/WebDriver BiDi listening on (ws:\/\/127\.0\.0\.1:\d+)/);
    if (match) endpoint = match[1] + "/session";
    if (Date.now() - started > 45000) throw new Error("Firefox startup deadline");
    if (!endpoint) await new Promise(resolve => setTimeout(resolve, 50));
  }
  result.endpoint = endpoint;
  client = await BidiClient.connect(endpoint, entry => transcript.push(entry));
  result.preSessionStatus = await client.command("session.status", {});
  const sessionStarted = Date.now();
  const session = await client.command("session.new", { capabilities: {} }, 45000);
  result.sessionCreationMs = Date.now() - sessionStarted;
  result.session = { id: session.sessionId, browserName: session.capabilities.browserName,
    browserVersion: session.capabilities.browserVersion, platformName: session.capabilities.platformName,
    pid: session.capabilities["moz:processID"], profile: session.capabilities["moz:profile"],
    headless: session.capabilities["moz:headless"], buildID: session.capabilities["moz:buildID"] };
  assert.equal(session.capabilities["moz:processID"], child.pid);
  assert.equal(session.capabilities["moz:profile"], profile);
  assert.equal(session.capabilities["moz:headless"], false);
  assert.equal(session.capabilities.browserName, "firefox");
  assert.equal(session.capabilities.browserVersion, pinned.version);

  const main = (await client.command("browsingContext.create", { type: "tab" })).context;
  const other = (await client.command("browsingContext.create", { type: "tab" })).context;
  await client.command("browsingContext.navigate", { context: main, url: origin + "/main", wait: "complete" });
  await client.command("browsingContext.navigate", { context: other, url: origin + "/other", wait: "complete" });
  const target = { context: main };
  const evaluate = expression => client.evaluate(target, expression);
  await check("exact page discovery and Gecko identity", async () => {
    const tree = await client.command("browsingContext.getTree", {});
    assert(tree.contexts.some(item => item.context === main && item.url === origin + "/main"));
    assert(tree.contexts.some(item => item.context === other && item.url === origin + "/other"));
    result.userAgent = await evaluate("navigator.userAgent");
    assert(result.userAgent.includes("Gecko/") && result.userAgent.includes("Firefox/" + pinned.version));
  });
  await check("shared runtime CSS HTML JS apply", async () => {
    assert.equal((await evaluate(applySource(request, body))).status, "applied");
    const state = await evaluate("(()=>{const b=document.querySelector('.gecko-owned');" +
      "b.click();return {color:getComputedStyle(b).color,count:b.dataset.count," +
      "nodes:document.querySelectorAll('[data-injectra]').length};})()");
    assert.deepEqual(state, { color: "rgb(12, 34, 56)", count: "1", nodes: 2 });
  });
  await check("second page remains unchanged", async () => {
    assert.deepEqual(await client.evaluate({ context: other },
      "({nodes:document.querySelectorAll('[data-injectra]').length,count:globalThis.geckoOwnedCount??null," +
      "original:document.querySelector('#original').textContent})"),
    { nodes: 0, count: null, original: "Owned other fixture" });
  });
  await check("real page screenshot", async () => {
    const screenshot = await client.command("browsingContext.captureScreenshot", { context: main });
    const image = Buffer.from(screenshot.data, "base64");
    assert.equal(image.subarray(0, 8).toString("hex"), "89504e470d0a1a0a");
    await writeFile(join(run, "applied.png"), image);
  });
  await check("same id update without duplicates", async () => {
    assert.equal((await evaluate(applySource({ ...request,
      html: '<button class="gecko-owned">Updated test button</button>' }, body))).status, "updated");
    assert.deepEqual(await evaluate("({text:document.querySelector('.gecko-owned').textContent," +
      "nodes:document.querySelectorAll('[data-injectra]').length,cleaned:globalThis.geckoCleaned})"),
    { text: "Updated test button", nodes: 2, cleaned: 1 });
  });
  await check("remove owned content and cleanup", async () => {
    assert.equal((await evaluate(removeSource(request.id))).status, "removed");
    assert.deepEqual(await evaluate("({nodes:document.querySelectorAll('[data-injectra]').length," +
      "count:globalThis.geckoOwnedCount??null,cleaned:globalThis.geckoCleaned," +
      "original:document.querySelector('#original').textContent})"),
    { nodes: 0, count: null, cleaned: 2, original: "Owned main fixture" });
  });
  await check("absent removal", async () => {
    assert.equal((await evaluate(removeSource(request.id))).status, "absent");
  });
  await check("await Promise result", async () => {
    assert.equal(await evaluate("new Promise(resolve=>setTimeout(()=>resolve('promise-ok'),40))"), "promise-ok");
  });
  await check("page exception is observable", async () => {
    await assert.rejects(evaluate("(()=>{throw new Error('gecko-expected-error')})()"),
      error => error.response?.type === "exception" && /gecko-expected-error/.test(error.message));
  });
  await check("failed injection cleans nodes and callbacks", async () => {
    await assert.rejects(evaluate(applySource({ ...request, id: "injectra.gecko.failure" },
      "context.onCleanup(()=>{globalThis.failedCleanup=true;});throw new Error('expected-apply-failure');")),
      error => error.response?.type === "exception" && /expected-apply-failure/.test(error.message));
    assert.deepEqual(await evaluate("({nodes:document.querySelectorAll('[data-injectra]').length," +
      "cleaned:globalThis.failedCleanup})"), { nodes: 0, cleaned: true });
  });
  await check("missing context rejected before execution", async () => {
    await assert.rejects(client.evaluate({ context: "00000000-0000-0000-0000-000000000000" },
      "(globalThis.wrongTarget=true)"), error => error.response?.error === "no such frame");
    assert.equal(await evaluate("globalThis.wrongTarget??null"), null);
  });
  await check("closed context rejected", async () => {
    await client.command("browsingContext.close", { context: other });
    await assert.rejects(client.evaluate({ context: other }, "(globalThis.wrongTarget=true)"),
      error => error.response?.error === "no such frame");
  });
  await check("navigation invalidates previous realm", async () => {
    const realms = await client.command("script.getRealms", { context: main, type: "window" });
    const old = realms.realms.find(item => !item.sandbox);
    assert(old?.realm);
    await client.command("browsingContext.navigate", { context: main, url: origin + "/main?next=1", wait: "complete" });
    await assert.rejects(client.evaluate({ realm: old.realm }, "(globalThis.staleRealm=true)"),
      error => error.response?.error === "no such frame" && error.message.includes(old.realm) && /Realm/.test(error.message));
    const current = await client.command("script.getRealms", { context: main, type: "window" });
    assert(!current.realms.some(item => item.realm === old.realm));
    assert.equal(await evaluate("globalThis.staleRealm??null"), null);
  });
  await check("current product CDP route does not claim Firefox support", async () => {
    let output;
    try {
      execFileSync(binary, ["targets", "--port", String(new URL(endpoint).port)],
        { timeout: 10000, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
      throw new Error("Unexpected success from current CDP backend");
    } catch (error) {
      if (error.status !== 1) throw error;
      output = { status: error.status, stdout: error.stdout, stderr: error.stderr };
    }
    await writeFile(join(run, "product-cdp-result.json"), JSON.stringify(output, null, 2));
  });
  await check("BiDi session ends", async () => {
    await client.command("session.end", {});
  });
  result.passed = true;
} catch (error) {
  result.failure = { message: error.message, stack: error.stack, response: error.response };
} finally {
  client?.close();
  if (child && !childExited && child.pid) {
    child.kill("SIGTERM");
    await Promise.race([once(child, "exit").catch(() => {}), new Promise(resolve => setTimeout(resolve, 5000))]);
    if (!childExited) {
      child.kill("SIGKILL");
      await Promise.race([once(child, "exit").catch(() => {}), new Promise(resolve => setTimeout(resolve, 3000))]);
    }
  }
  result.ownedProcessExited = childExited;
  if (child?.pid && !childExited) {
    result.passed = false;
    result.cleanupFailure = "Owned Firefox exit not confirmed";
  }
  await new Promise(resolve => server.close(resolve));
  await writeFile(join(run, "host.log"), hostLog);
  await writeFile(join(run, "protocol.jsonl"), transcript.map(entry => JSON.stringify(entry)).join("\n") + "\n");
  await writeFile(join(run, "result.json"), JSON.stringify(result, null, 2) + "\n");
}
console.log(JSON.stringify(result));
if (!result.passed) process.exitCode = 1;
