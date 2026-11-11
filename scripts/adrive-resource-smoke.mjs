// Optional real-app experiment. Never edits the installed application.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createServer } from "node:http";
import { mkdir, mkdtemp, readFile, writeFile, copyFile, chmod, access } from "node:fs/promises";
import { randomBytes } from "node:crypto";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { once } from "node:events";
import { cache, root } from "./test-env.mjs";
import { asar, rewriteAsar, sha256 } from "../tests/resource-patch/archive.mjs";
import { pageProof } from "../tests/resource-patch/page-proof.mjs";
import { evaluateProof } from "../tests/resource-patch/receipts.mjs";

assert.equal(process.platform, "darwin");
assert.equal(process.arch, "arm64");
const [mode, argument] = process.argv.slice(2);
const json = value => JSON.stringify(value, null, 2) + "\n";
const hash = async path => sha256(await readFile(path));
const localLibraries = mode === "--prepare-local-signing";
const sign = (app, outer = false) => JSON.parse(execFileSync("/usr/bin/python3", ["-B", resolve(root, "tests/resource-patch/sign-copy.py"), app, ...(outer ? ["--outer"] : []), ...(localLibraries ? ["--local-libraries"] : [])], { encoding: "utf8" }));
async function snapshot(run, label) {
  const sources = {};
  for (const path of ["scripts/adrive-resource-smoke.mjs", "tests/resource-patch/archive.mjs", "tests/resource-patch/page-proof.mjs", "tests/resource-patch/receipts.mjs", "tests/resource-patch/sign-copy.py", "tests/resource-patch/package-lock.json", "web/runtime.js"]) {
    const bytes = await readFile(resolve(root, path));
    const destination = resolve(run, label, path);
    await mkdir(resolve(destination, ".."), { recursive: true });
    await writeFile(destination, bytes, { flag: "wx" });
    sources[path] = sha256(bytes);
  }
  await writeFile(resolve(run, `${label}.json`), json(sources), { flag: "wx" });
  return sources;
}

if (mode === "--prepare" || localLibraries) {
  const source = resolve(argument ?? "/Applications/aDrive.app");
  const paths = ["Contents/MacOS/aDrive", "Contents/Resources/app.asar", "Contents/Info.plist", "Contents/_CodeSignature/CodeResources"];
  const originalHashes = Object.fromEntries(await Promise.all(paths.map(async path => [path, await hash(resolve(source, path))])));
  await mkdir(resolve(cache, "runs"), { recursive: true });
  const run = await mkdtemp(resolve(cache, "runs/adrive-resource-"));
  await chmod(run, 0o700);
  const app = resolve(run, "aDrive-Test.app");
  await writeFile(resolve(run, "ownership.json"), json({ source, app, originalHashes, createdBy: "T-035/E-001" }));
  const sources = await snapshot(run, "prepare-source");
  execFileSync("/usr/bin/rsync", ["-a", "--exclude=/Contents/MacOS/sync_drive_app/data/", "--exclude=syncapp.pid", source + "/", app + "/"]);
  const nestedSigning = sign(app);
  await writeFile(resolve(run, "nested-signing.json"), json(nestedSigning));
  const server = createServer(); server.listen(0, "127.0.0.1"); await once(server, "listening");
  const port = server.address().port;
  await new Promise(resolve => server.close(resolve));
  const nonce = randomBytes(16).toString("hex");
  const endpoint = `http://127.0.0.1:${port}/proof/${nonce}`;
  const input = resolve(app, "Contents/Resources/app.asar"), output = resolve(run, "patched.asar");
  const originalHtml = asar.extractFile(input, "dist/index.html").toString("utf8");
  assert(!originalHtml.includes("injectra-proof.js"));
  assert(/<script\b/i.test(originalHtml));
  const html = originalHtml.replace(/<script\b/i, '<script src="./injectra-proof.js"></script><script');
  const runtime = await readFile(resolve(root, "web/runtime.js"), "utf8");
  const proof = pageProof({ runtime, nonce, endpoint });
  const archive = await rewriteAsar(input, output, new Map([["dist/index.html", Buffer.from(html)], ["dist/injectra-proof.js", Buffer.from(proof)]]));
  await copyFile(output, input);
  execFileSync("/usr/bin/rsync", ["-a", output + ".unpacked/", input + ".unpacked/"]);
  asar.uncache(input);
  assert.equal(sha256(asar.getRawHeader(input).headerString), archive.headerHash);
  execFileSync("/usr/bin/python3", ["-c", "import plistlib,sys; p=sys.argv[1]; d=plistlib.load(open(p,'rb')); d['ElectronAsarIntegrity']['Resources/app.asar']['hash']=sys.argv[2]; d['CFBundleDisplayName']='阿里云盘 · Injectra 测试'; plistlib.dump(d,open(p,'wb'))", resolve(app, "Contents/Info.plist"), archive.headerHash]);
  const outerSigning = sign(app, true);
  const verify = execFileSync("/usr/bin/codesign", ["--verify", "--deep", "--strict", "--verbose=2", app], { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
  const plan = { run, app, source, nonce, port, endpoint, originalHashes, archive, runtimeSha: sha256(runtime), proofSha: sha256(proof),
    archiveToolVersion: "3.4.1", sources, outerSigning, localLibraryEntitlement: localLibraries, signatureVerified: true, signatureOutput: verify, originalCriticalFilesUnchanged: true,
    excludedRuntimeData: ["Contents/MacOS/sync_drive_app/data/", "syncapp.pid"], automaticChildCleanup: false };
  for (const path of paths) assert.equal(await hash(resolve(source, path)), originalHashes[path]);
  await writeFile(resolve(run, "plan.json"), json(plan));
  await writeFile(resolve(run, "proof.js"), proof);
  console.log(json({ prepared: true, run, app, archive, signatureVerified: true }));
} else if (mode === "--run" || mode === "--run-mock-keychain") {
  const plan = JSON.parse(await readFile(resolve(argument, "plan.json"), "utf8"));
  const { run, app, source, nonce, port } = plan;
  assert.equal(resolve(argument), run);
  assert.equal(app, resolve(run, "aDrive-Test.app"));
  await access(resolve(run, "launch.json")).then(() => { throw new Error("Existing launch evidence: prepare a new run for retries"); }, error => { if (error.code !== "ENOENT") throw error; });
  const processes = execFileSync("/bin/ps", ["-axo", "comm="], { encoding: "utf8" }).split("\n").map(line => line.trim());
  for (const path of [source, app]) assert(!processes.includes(resolve(path, "Contents/MacOS/aDrive")), "Normal exit required before testing");
  const sources = await snapshot(run, "launch-source");
  const events = [], started = Date.now();
  let clicked = false;
  const server = createServer((request, response) => {
    const url = new URL(request.url, `http://127.0.0.1:${port}`);
    if (request.method !== "GET" || url.pathname !== `/proof/${nonce}` || url.searchParams.get("nonce") !== nonce) { response.writeHead(404).end(); return; }
    const data = Object.fromEntries(["nonce", "phase", "path", "ready", "status", "connected", "appRootPresent", "color", "error"].filter(key => url.searchParams.has(key)).map(key => [key, url.searchParams.get(key)]));
    const event = { elapsedMs: Date.now() - started, peer: request.socket.remoteAddress, ...data };
    events.push(event);
    console.log(json({ receipt: event.phase, connected: event.connected, appRootPresent: event.appRootPresent }));
    if (event.phase === "clicked") clicked = true;
    response.writeHead(204, { "Cache-Control": "no-store" }).end();
  });
  server.listen(port, "127.0.0.1"); await once(server, "listening");
  const home = resolve(run, "home");
  await mkdir(resolve(home, "Library/Application Support"), { recursive: true });
  const env = { ...process.env, HOME: home, CFFIXED_USER_HOME: home, XDG_CACHE_HOME: resolve(home, ".cache"), XDG_CONFIG_HOME: resolve(home, ".config"), XDG_DATA_HOME: resolve(home, ".local/share") };
  for (const key of ["ELECTRON_RUN_AS_NODE", "DYLD_INSERT_LIBRARIES", "NODE_OPTIONS"]) delete env[key];
  const { openSync, closeSync } = await import("node:fs");
  const log = openSync(resolve(run, "startup.log"), "a", 0o600);
  const command = [resolve(app, "Contents/MacOS/aDrive"), `--user-data-dir=${resolve(home, "profile")}`, `--disk-cache-dir=${resolve(home, "chromium-cache")}`, "--no-first-run"];
  if (mode === "--run-mock-keychain") command.push("--use-mock-keychain");
  const child = spawn(command[0], command.slice(1), { env, detached: true, stdio: ["ignore", log, log] });
  child.unref(); closeSync(log);
  await once(child, "spawn");
  let exit = null;
  child.on("exit", (code, signal) => { exit = { code, signal, elapsedMs: Date.now() - started }; });
  await writeFile(resolve(run, "launch.json"), json({ rootPid: child.pid, command, configuredHome: home, sources, startedAt: new Date().toISOString() }));
  console.log(json({ run, pid: child.pid, app, noncePrefix: nonce.slice(0, 10), observingSeconds: 90 }));
  while (Date.now() - started < 90000 && !clicked) await new Promise(resolve => setTimeout(resolve, 250));
  await new Promise(resolve => server.close(resolve));
  const expectedPath = fileURLToPath(new URL(`file://${app}/Contents/Resources/app.asar/dist/index.html`));
  const { passed, stages } = evaluateProof(events, { path: expectedPath, nonce });
  const originals = Object.fromEntries(await Promise.all(Object.keys(plan.originalHashes).map(async path => [path, await hash(resolve(source, path))])));
  assert.deepEqual(originals, plan.originalHashes);
  await writeFile(resolve(run, "receipts.json"), json(events));
  await writeFile(resolve(run, "result.json"), json({ passed, stages, exit, rootPid: child.pid, events, originalCriticalFilesUnchanged: true, automaticChildCleanup: false, controllerFinishedAt: new Date().toISOString() }));
  console.log(json({ passed, exit, receipts: events.length, run, childLeftForVerifiedNormalExit: true }));
  process.exitCode = passed ? 0 : 1;
} else throw new Error("Use --prepare [app], --prepare-local-signing [app] or --run <prepared-run>");
