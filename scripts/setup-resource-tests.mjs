import { mkdir, copyFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { cache, root } from "./test-env.mjs";

const destination = resolve(cache, "resource-patch-tools");
await mkdir(destination, { recursive: true });
for (const name of ["package.json", "package-lock.json"]) {
  await copyFile(resolve(root, "tests/resource-patch", name), resolve(destination, name));
}
const result = spawnSync("npm", ["ci", "--prefix", destination, "--no-audit", "--no-fund"], { stdio: "inherit" });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
