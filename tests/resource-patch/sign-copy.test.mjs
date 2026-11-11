import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, mkdir, writeFile, realpath } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { root } from "../../scripts/test-env.mjs";

const invoke = app => spawnSync("/usr/bin/python3", ["-O", resolve(root, "tests/resource-patch/sign-copy.py"), app], { encoding: "utf8" });
test("optimized Python still rejects original app paths before signing", () => {
  const result = invoke("/Applications/aDrive.app");
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Not an owned test copy/);
});
test("a test-like path requires a matching ownership record", async () => {
  const run = await realpath(await mkdtemp(resolve(tmpdir(), "adrive-resource-")));
  const app = resolve(run, "aDrive-Test.app");
  await mkdir(app);
  assert.match(invoke(app).stderr, /Missing test ownership/);
  await writeFile(resolve(run, "ownership.json"), JSON.stringify({ app, source: "/Applications/aDrive.app", createdBy: "wrong-task" }));
  assert.match(invoke(app).stderr, /does not match/);
});
