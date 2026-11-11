import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, readFile, mkdir, writeFile, symlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { asar, rewriteAsar, sha256 } from "./archive.mjs";

test("rebuild preserves ordinary, unpacked and linked content while replacing the page", async () => {
  const directory = await mkdtemp(resolve(tmpdir(), "injectra-asar-"));
  const source = resolve(directory, "source");
  await mkdir(resolve(source, "dist"), { recursive: true });
  await writeFile(resolve(source, "dist/index.html"), "<body>original</body>");
  await writeFile(resolve(source, "dist/native.node"), "unpacked fixture");
  await writeFile(resolve(source, "dist/z-last.bin"), Buffer.alloc(3 * 1024 * 1024, 0x61));
  await symlink("index.html", resolve(source, "dist/link.html"));
  const input = resolve(directory, "input.asar"), output = resolve(directory, "output.asar");
  await asar.createPackageWithOptions(source, input, { unpack: "*.node" });
  const hash = sha256(await readFile(input));
  const report = await rewriteAsar(input, output, new Map([["dist/index.html", Buffer.from("<body>patched</body>")], ["dist/proof.js", Buffer.from("proof()")]]));
  assert.equal(sha256(await readFile(input)), hash);
  assert.equal(asar.extractFile(output, "dist/link.html").toString(), "<body>patched</body>");
  assert.equal(asar.statFile(output, "dist/native.node").unpacked, true);
  assert.equal(asar.extractFile(output, "dist/native.node").toString(), "unpacked fixture");
  assert.equal(report.headerHash, sha256(asar.getRawHeader(output).headerString));
});

test("archive edits reject source overwrite and escaping paths", async () => {
  await assert.rejects(rewriteAsar("same.asar", "same.asar", new Map()), /preserved/);
  await assert.rejects(rewriteAsar("input.asar", "output.asar", new Map([["../escape.js", Buffer.from("x")]])), /Unsafe/);
});
