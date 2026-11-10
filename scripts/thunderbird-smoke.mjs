// Optional Thunderbird validation on an owned empty profile, separate from product backends.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createServer } from "node:http";
import { once } from "node:events";
import { appendFileSync } from "node:fs";
import { mkdir, mkdtemp, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { root, cache } from "./test-env.mjs";
import { BidiClient } from "../tests/gecko/bidi-client.mjs";
import { MarionetteClient } from "../tests/gecko/marionette-client.mjs";
import { verifyRuntime } from "../tests/gecko/runtime-checks.mjs";
import { ThunderbirdContentHost } from "../tests/gecko/thunderbird-host.mjs";

assert.equal(process.platform, "darwin");
const inspectOnly = process.argv.includes("--inspect");
const parsedHtml = process.argv.includes("--parsed-html");
assert(!(inspectOnly && parsedHtml), "Use --inspect or --parsed-html, not both");
const pinned = JSON.parse(await readFile(join(root, "tests/gecko/thunderbird.json"), "utf8"));
const receipt = JSON.parse(await readFile(join(cache, "gecko-tests/thunderbird", pinned.version, "verified.json"), "utf8"));
assert.equal(receipt.sha512, pinned.sha512);
execFileSync("/usr/bin/codesign", ["--verify", "--deep", "--strict", receipt.app]);
await mkdir(join(cache, "runs"), { recursive: true });
const run = await mkdtemp(join(cache, "runs/thunderbird-gecko-"));
const home = join(run, "home"), profile = join(run, "profile");
await mkdir(join(home, "Library/Application Support"), { recursive: true });
await mkdir(profile);
const prefs = {
  "marionette.port": 0,
  "remote.log.level": "Trace",
  "mail.shell.checkDefaultClient": false,
  "mail.provider.suppress_dialog_on_startup": true,
  "mailnews.start_page.enabled": false,
  "mailnews.start_page.url": "about:blank",
  "datareporting.policy.dataSubmissionEnabled": false,
  "toolkit.telemetry.enabled": false,
  "app.update.auto": false,
};
await writeFile(join(profile, "user.js"), Object.entries(prefs)
  .map(([key, value]) => "user_pref(" + JSON.stringify(key) + "," + JSON.stringify(value) + ");").join("\n"));
const server = createServer((request, response) => {
  const name = new URL(request.url, "http://127.0.0.1").pathname === "/other" ? "other" : "main";
  response.writeHead(200, { "Content-Type": "text/html; charset=utf-8", "Cache-Control": "no-store" });
  response.end("<!doctype html><meta charset=utf-8><title>Injectra Thunderbird " + name +
    "</title><h1 id=original>Thunderbird owned " + name + " fixture</h1>");
});
server.listen(0, "127.0.0.1"); await once(server, "listening");
const origin = "http://127.0.0.1:" + server.address().port;
const env = { ...process.env, HOME: home, CFFIXED_USER_HOME: home, MOZ_NO_REMOTE: "1",
  MOZ_CRASHREPORTER_DISABLE: "1", MOZ_REMOTE_ALLOW_SYSTEM_ACCESS: "1" };
delete env.MOZ_HEADLESS;
const transcript = [];
const result = { passed: false, mode: inspectOnly ? "inspection" : parsedHtml ? "parsed-html-contract-v2" : "contract",
  htmlRoute: parsedHtml ? "test-only DOMParser + importNode preparation" : "shared runtime innerHTML",
  host: "Thunderbird", version: pinned.version, run, checks: [], productBackend: false };
let child, exited = false, spawnError, marionette, bidi;
const check = async (name, action) => {
  await action(); result.checks.push(name); console.log("PASS " + name);
};
try {
  child = spawn(join(receipt.app, "Contents/MacOS/thunderbird"),
    ["--foreground", "--new-instance", "--profile", profile, "--marionette", "--remote-debugging-port", "0"],
    { env, stdio: ["ignore", "pipe", "pipe"] });
  child.on("error", error => { spawnError = error; });
  child.on("exit", (code, signal) => { exited = true; result.exit = { code, signal }; });
  for (const stream of [child.stdout, child.stderr]) stream.on("data", data => appendFileSync(join(run, "host.log"), data));
  result.pid = child.pid;
  await writeFile(join(run, "owner.json"), JSON.stringify({ pid: child.pid, app: receipt.app, profile }, null, 2));
  let port;
  const started = Date.now();
  while (!port) {
    if (spawnError) throw spawnError;
    if (exited) throw new Error("Owned Thunderbird exited during startup");
    try { port = Number((await readFile(join(profile, "MarionetteActivePort"), "utf8")).trim()); }
    catch (error) { if (error.code !== "ENOENT") throw error; }
    if (Date.now() - started > 45000) throw new Error("Marionette startup deadline");
    if (!port) await new Promise(resolve => setTimeout(resolve, 50));
  }
  marionette = await MarionetteClient.connect(port, entry => transcript.push(entry));
  result.greeting = marionette.greeting;
  const sessionStarted = Date.now();
  const session = await marionette.command("WebDriver:NewSession", { webSocketUrl: true }, 45000);
  result.sessionCreationMs = Date.now() - sessionStarted;
  result.foreground = true;
  result.suites = {};
  result.session = session;
  assert.equal(session.capabilities.browserVersion, pinned.version);
  assert.equal(session.capabilities["moz:processID"], child.pid);
  assert.equal(session.capabilities["moz:profile"], profile);
  assert.equal(session.capabilities["moz:headless"], false);
  result.windows = await marionette.command("WebDriver:GetWindowHandles");
  await marionette.command("Marionette:SetContext", { value: "chrome" });
  result.chrome = await marionette.evaluate("({url:location.href,title:document.title,head:!!document.head," +
    "body:!!document.body,windowType:document.documentElement.getAttribute('windowtype')," +
    "tabmail:!!document.getElementById('tabmail'),accounts:Services.prefs.getStringPref('mail.accountmanager.accounts','')," +
    "contentType:document.contentType,rootNamespace:document.documentElement.namespaceURI," +
    "createdDivNamespace:document.createElement('div').namespaceURI," +
    "markupProbe:(()=>{const d=document.createElement('div');d.innerHTML='<button>probe</button>';" +
    "return {markup:d.innerHTML,children:d.children.length,button:!!d.querySelector('button')};})()})");
  if (session.capabilities.webSocketUrl) {
    bidi = await BidiClient.connect(session.capabilities.webSocketUrl, entry =>
      transcript.push({ ...entry, protocol: "bidi" }));
    try { result.bidiTree = await bidi.command("browsingContext.getTree", {}); }
    catch (error) { result.bidiInspectionFailure = error.response ?? { message: error.message }; }
  }
if (!inspectOnly) {
    assert.equal(result.chrome.url, "chrome://messenger/content/messenger.xhtml");
    assert.equal(result.chrome.windowType, "mail:3pane");
    assert.equal(result.chrome.accounts, "");
    assert(bidi, "Expected the advertised BiDi channel for protocol comparison");
    const contentHost = new ThunderbirdContentHost(marionette, session.sessionId);
    result.contentRoute = "Marionette -> Thunderbird tabmail -> Gecko execution actor";
    const runtime = await readFile(join(root, "web/runtime.js"), "utf8");
    result.createdTabs = await marionette.evaluate("(()=>{const t=document.getElementById('tabmail');" +
      "t.openTab('contentTab',{url:" + JSON.stringify(origin + "/main") + "});" +
      "t.openTab('contentTab',{url:" + JSON.stringify(origin + "/other") + "});" +
      "return t.tabInfo.map(x=>({mode:x.mode.name,url:x.browser?.currentURI?.spec??null}));})()");
    let main, other;
    const pageDeadline = Date.now() + 30000;
    while (!main || !other) {
      const pages = await contentHost.list();
      main = pages.find(item => item.url === origin + "/main" && !item.loading);
      other = pages.find(item => item.url === origin + "/other" && !item.loading);
      if (Date.now() > pageDeadline) { result.lastPages = pages; throw new Error("Thunderbird host target discovery deadline"); }
      if (!main || !other) await new Promise(resolve => setTimeout(resolve, 100));
    }
    result.targets = { main, other };
    result.bidiTreeAfterHostDiscovery = await bidi.command("browsingContext.getTree", {});
    const mainEvaluate = expression => contentHost.evaluate(main, expression);
    const otherEvaluate = expression => contentHost.evaluate(other, expression);
    const savePng = async (name, data) => {
      const bytes = Buffer.from(data, "base64");
      assert.equal(bytes.subarray(0, 8).toString("hex"), "89504e470d0a1a0a");
      await writeFile(join(run, name), bytes);
    };
    await check("exact Thunderbird session and two content targets", async () => {
      assert.equal(session.capabilities.browserName, "thunderbird");
      assert.equal(await mainEvaluate("document.title"), "Injectra Thunderbird main");
      assert.equal(await otherEvaluate("document.title"), "Injectra Thunderbird other");
    });
    const suite = async (name, action) => {
      try { await action(); result.suites[name] = { passed: true }; }
      catch (error) {
        result.suites[name] = { passed: false, message: error.message, response: error.response, cleanupFailure: error.cleanupFailure };
        console.log("FAIL " + name + ": " + error.message);
      }
    };
    await suite("applicationUIHtml", () => verifyRuntime({
      evaluate: expression => marionette.evaluatePage(expression), other: otherEvaluate, runtime,
      id: "injectra.thunderbird.chrome", check: (name, action) => check("application UI: " + name, action),
      screenshot: async () => {
        const image = await marionette.command("WebDriver:TakeScreenshot", { full: false });
        await savePng("application-ui.png", image.value);
      },
    }));
    if (parsedHtml) await suite("applicationUIParsedHtml", () => verifyRuntime({
      evaluate: expression => marionette.evaluatePage(expression), other: otherEvaluate, runtime,
      id: "injectra.thunderbird.parsed-html", mountMode: "parsed-html",
      check: (name, action) => check("application UI parsed HTML: " + name, action),
      screenshot: async () => {
        const image = await marionette.command("WebDriver:TakeScreenshot", { full: false });
        await savePng("application-ui-parsed-html.png", image.value);
      },
    }));
    await suite("applicationUIDom", () => verifyRuntime({
      evaluate: expression => marionette.evaluatePage(expression), other: otherEvaluate, runtime,
      id: "injectra.thunderbird.chrome-dom", mountMode: "dom",
      check: (name, action) => check("application UI DOM route: " + name, action),
      screenshot: async () => {
        const image = await marionette.command("WebDriver:TakeScreenshot", { full: false });
        await savePng("application-ui-dom.png", image.value);
      },
    }));
    await suite("contentPage", () => verifyRuntime({
      evaluate: mainEvaluate, other: otherEvaluate, runtime,
      id: "injectra.thunderbird.content", check: (name, action) => check("content page: " + name, action),
      screenshot: async () => {
        await savePng("content-page.png", await contentHost.screenshot(main));
      },
    }));
    await check("foreign session and missing target rejected", async () => {
      await assert.rejects(contentHost.evaluate({ ...main, sessionId: "foreign" }, "(globalThis.tbWrongTarget=true)"), /another test session/);
      await assert.rejects(contentHost.evaluate({ ...main, context: "0" }, "(globalThis.tbWrongTarget=true)"), /target not found/);
      assert.equal(await mainEvaluate("globalThis.tbWrongTarget??null"), null);
    });
    await check("navigation rejects old document generation", async () => {
      const old = main;
      await contentHost.navigate(old, origin + "/main?next=1");
      const deadline = Date.now() + 15000;
      while (true) {
        const pages = await contentHost.list();
        const next = pages.find(item => item.url === origin + "/main?next=1" && !item.loading && item.generation !== old.generation);
        if (next) { main = next; break; }
        if (Date.now() > deadline) throw new Error("Owned navigation deadline");
        await new Promise(resolve => setTimeout(resolve, 100));
      }
      await assert.rejects(contentHost.evaluate(old, "(globalThis.tbStaleRealm=true)"), /target generation changed|target not found/);
      assert.equal(await mainEvaluate("globalThis.tbStaleRealm??null"), null);
    });
    await check("closed target rejected", async () => {
      await contentHost.close(other);
      await assert.rejects(otherEvaluate("(globalThis.tbWrongTarget=true)"), /target not found|target generation changed/);
    });
    await check("application UI clean and still no accounts", async () => {
      assert.deepEqual(await marionette.evaluate("({url:location.href,nodes:document.querySelectorAll('[data-injectra]').length," +
        "accounts:Services.prefs.getStringPref('mail.accountmanager.accounts','')})"),
        { url: "chrome://messenger/content/messenger.xhtml", nodes: 0, accounts: "" });
    });
  }
  await check("session deletion", () => marionette.command("WebDriver:DeleteSession", {}));
  result.baselinePassed = Object.entries(result.suites).filter(([name]) => name !== "applicationUIParsedHtml").every(([, suite]) => suite.passed);
  result.baselineOutcome = result.baselinePassed ? "passed" : "partial";
  if (parsedHtml) {
    // V2 certifies the alternate fixture preparation route. V1 is still measured above.
    result.requiredSuites = ["applicationUIParsedHtml", "applicationUIDom", "contentPage"];
    result.passed = result.requiredSuites.every(name => result.suites[name]?.passed === true);
    result.outcome = result.passed ? "parsed-html-route-passed" : "partial";
  } else {
    result.passed = result.baselinePassed;
    result.outcome = result.passed ? "passed" : "partial";
  }
} catch (error) {
  result.failure = { message: error.message, stack: error.stack, response: error.response };
} finally {
  bidi?.close(); marionette?.close();
  if (child?.pid && !exited) {
    child.kill("SIGTERM");
    await Promise.race([once(child, "exit").catch(() => {}), new Promise(resolve => setTimeout(resolve, 5000))]);
    if (!exited) {
      child.kill("SIGKILL");
      await Promise.race([once(child, "exit").catch(() => {}), new Promise(resolve => setTimeout(resolve, 3000))]);
    }
  }
  result.ownedProcessExited = exited;
  if (child?.pid && !exited) { result.passed = false; result.cleanupFailure = "Owned process exit unconfirmed"; }
  await new Promise(resolve => server.close(resolve));
  await writeFile(join(run, "protocol.jsonl"), transcript.map(entry => JSON.stringify(entry)).join("\n") + "\n");
  await writeFile(join(run, "result.json"), JSON.stringify(result, null, 2) + "\n");
}
console.log(JSON.stringify(result));
if (!result.passed) process.exitCode = 1;
