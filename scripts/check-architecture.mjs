// Validate the dependency boundaries that keep execution domains replaceable.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { root } from "./test-env.mjs";

const command = spawnSync("cargo", ["metadata", "--no-deps", "--format-version", "1", "--locked", "--offline"], { cwd: root, encoding: "utf8" });
assert.equal(command.error, undefined, command.error?.message);
assert.equal(command.status, 0, command.stderr);
const packages = new Map(JSON.parse(command.stdout).packages.map(p => [p.name, p]));
const allowed = {
  "injectra-core": [],
  "injectra-web": ["injectra-core"],
  "injectra-native": ["injectra-core"],
  "injectra-application": ["injectra-core", "injectra-web"],
  "injectra-cdp": ["injectra-core", "injectra-web"],
  "injectra-platform-macos": ["injectra-core"],
  "injectra-hosts": ["injectra-core", "injectra-platform-macos"],
  "injectra-cli": ["injectra-core", "injectra-web", "injectra-application", "injectra-cdp", "injectra-hosts"],
};
for (const [name, dependencies] of Object.entries(allowed)) {
  const pkg = packages.get(name);
  assert.ok(pkg, `missing architectural role: ${name}`);
  for (const dep of pkg.dependencies.filter(d => d.kind !== "dev")) {
    if (dep.path) assert.ok(dependencies.includes(dep.name), `${name} must not depend on ${dep.name}`);
    if (name === "injectra-core") assert.fail(`core must remain dependency-free: ${dep.name}`);
    if (name === "injectra-native") assert.equal(dep.name, "injectra-core", `native contracts must build without external runtimes: ${dep.name}`);
  }
}
console.log(`Architecture OK: ${Object.keys(allowed).length} crate roles; core/native remain independent of Web and concrete backends.`);
