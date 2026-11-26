import { mkdir, copyFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { dependencies, root } from "./test-env.mjs";

await mkdir(dependencies, { recursive: true });
for (const name of ["package.json", "package-lock.json"]) {
  await copyFile(resolve(root, "tests", name), resolve(dependencies, name));
}
const result = spawnSync("npm", ["ci", "--prefix", dependencies, "--no-audit", "--no-fund"], { stdio: "inherit", shell: process.platform === "win32" });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
