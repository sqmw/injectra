import test from "node:test";
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtemp, mkdir, writeFile, readFile, realpath } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { createHash } from "node:crypto";
import { root } from "../../scripts/test-env.mjs";
const sentinel = Buffer.from("AGbevlPCksUGKNL8TSn7wGmJEuJsXb2A");
const digest = map => createHash("sha256").update(Object.keys(map).sort().map(key => key + map[key].algorithm + map[key].hash).join("")).digest();
async function fixture(corrupt = false) {
  const run = await realpath(await mkdtemp(resolve(tmpdir(), "resource-patch-")));
  const app = resolve(run, "Injectra-Test.app"), source = resolve(run, "Original.app");
  const framework = resolve(app, "Contents/Frameworks/Electron Framework.framework/Electron Framework");
  await mkdir(resolve(framework, ".."), { recursive: true }); await mkdir(resolve(source, "Contents"), { recursive: true });
  const old = { "Resources/b.asar": { algorithm: "SHA256", hash: "1".repeat(64) }, "Resources/a.asar": { algorithm: "SHA256", hash: "2".repeat(64) } };
  const next = { ...old, "Resources/a.asar": { algorithm: "SHA256", hash: "3".repeat(64) } };
  for (const [directory, map] of [[app, next], [source, old]]) execFileSync("/usr/bin/python3", ["-c", "import json,plistlib,sys;plistlib.dump(json.load(sys.stdin),open(sys.argv[1],'wb'))", resolve(directory, "Contents/Info.plist")], { input: JSON.stringify({ ElectronAsarIntegrity: map }) });
  await writeFile(resolve(run, "ownership.json"), JSON.stringify({ app, source, createdBy: "T-038/E-001" }));
  const slot = Buffer.concat([sentinel, Buffer.from([corrupt ? 0 : 1, 1]), digest(old)]);
  const bytes = Buffer.concat([Buffer.from("untouched-prefix"), slot, Buffer.from("between"), slot, Buffer.from("untouched-tail")]);
  await writeFile(framework, bytes);
  return { app, framework, bytes, next };
}
test("digest rebuild updates every active slot without disabling enforcement", async () => {
  const f = await fixture();
  const result = spawnSync("/usr/bin/python3", ["-B", resolve(root, "tests/resource-patch/integrity-digest.py"), f.app], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  const report = JSON.parse(result.stdout), expected = Buffer.from(f.bytes);
  assert.equal(report.slots.length, 2);
  for (const offset of report.slots) { assert.deepEqual(expected.subarray(offset, offset + 2), Buffer.from([1, 1])); digest(f.next).copy(expected, offset + 2); }
  assert.deepEqual(await readFile(f.framework), expected);
});
test("unexpected digest slot is rejected before any bytes are written", async () => {
  const f = await fixture(true);
  const result = spawnSync("/usr/bin/python3", ["-O", resolve(root, "tests/resource-patch/integrity-digest.py"), f.app], { encoding: "utf8" });
  assert.notEqual(result.status, 0);
  assert.deepEqual(await readFile(f.framework), f.bytes);
});
