// Validation only: rebuild with Electron's ASAR writer, preserving pack/link state.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { statSync } from "node:fs";
import { Readable } from "node:stream";
import { finished } from "node:stream/promises";
import { resolve, posix } from "node:path";
import { cache } from "../../scripts/test-env.mjs";

const require = createRequire(resolve(cache, "resource-patch-tools/package.json"));
export const asar = require("@electron/asar");
export const sha256 = bytes => createHash("sha256").update(bytes).digest("hex");

export function entries(header, prefix = "") {
  return Object.entries(header.files ?? {}).flatMap(([name, node]) => {
    const path = prefix ? `${prefix}/${name}` : name;
    return [{ path, node }, ...entries(node, path)];
  });
}

export async function rewriteAsar(source, destination, replacements) {
  assert.notEqual(resolve(source), resolve(destination), "Source archive must be preserved");
  for (const path of replacements.keys()) {
    assert(path && !path.startsWith("/") && posix.normalize(path) === path && !path.startsWith("../"), "Unsafe archive path");
  }
  const original = entries(asar.getRawHeader(source).header);
  const pending = new Map(replacements);
  const streams = original.map(({ path, node }) => {
    if (node.files) return { path, type: "directory", unpacked: node.unpacked };
    if (node.link) return { path, type: "link", stat: { mode: 0o120777 }, symlink: posix.relative(posix.dirname(path), node.link), unpacked: node.unpacked };
    const replacement = pending.get(path);
    pending.delete(path);
    const bytes = replacement ?? asar.extractFile(source, path);
    return { path, type: "file", unpacked: node.unpacked,
      stat: { size: bytes.length, mode: node.unpacked ? statSync(resolve(source + ".unpacked", path)).mode : node.executable ? 0o100755 : 0o100644 },
      streamGenerator: () => Readable.from(bytes) };
  });
  for (const [path, bytes] of pending) {
    assert(original.some(item => item.path === posix.dirname(path) && item.node.files), "New file must use an existing directory");
    streams.push({ path, type: "file", stat: { size: bytes.length, mode: 0o100644 }, streamGenerator: () => Readable.from(bytes) });
  }
  const output = await asar.createPackageFromStreams(destination, streams);
  // ASAR 3 returns its writable before the final buffered writes finish.
  await finished(output);
  asar.uncache(destination);
  const rebuilt = new Map(entries(asar.getRawHeader(destination).header).map(item => [item.path, item.node]));
  for (const { path, node } of original) {
    const result = rebuilt.get(path);
    for (const flag of ["unpacked", "link", "executable"]) assert.equal(result[flag], node[flag], `${path}: ${flag}`);
    if (!node.files && !node.link) {
      const wanted = replacements.get(path) ?? asar.extractFile(source, path);
      assert.equal(sha256(asar.extractFile(destination, path)), sha256(wanted), path);
    }
  }
  for (const [path, bytes] of replacements) assert.equal(sha256(asar.extractFile(destination, path)), sha256(bytes), path);
  return { originalEntries: original.length, rebuiltEntries: rebuilt.size,
    changedPaths: [...replacements.keys()], headerHash: sha256(asar.getRawHeader(destination).headerString) };
}
