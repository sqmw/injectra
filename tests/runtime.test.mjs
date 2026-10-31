import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { requireTool, root } from "../scripts/test-env.mjs";

const { JSDOM } = requireTool("jsdom");
const source = readFileSync(resolve(root, "web/runtime.js"), "utf8");
const setup = () => {
  const dom = new JSDOM("<!doctype html><html><head></head><body><p>Original</p></body></html>", { runScripts: "outside-only" });
  return { dom, window: dom.window, runtime: dom.window.eval(`(${source})`) };
};
const request = { operation: "apply", id: "test", css: "body { color: red }", html: "<button>Test</button>" };

test("overlapping operations on the same id are rejected", async () => {
  const { dom, runtime } = setup();
  let release;
  const pending = runtime(request, () => new Promise(resolve => { release = resolve; }));
  try {
    await assert.rejects(runtime(request, async () => {}), /already in progress/);
    release();
    await pending;
    assert.equal((await runtime({ operation: "remove", id: "test" })).status, "removed");
  } finally { dom.window.close(); }
});

test("same id updates without accumulating nodes and removal runs cleanup", async () => {
  const { dom, window, runtime } = setup();
  let balance = 0;
  const script = async context => { balance++; context.onCleanup(() => balance--); };
  try {
    await runtime(request, script);
    const updated = await runtime(request, script);
    assert.equal(updated.status, "updated");
    assert.equal(balance, 1);
    assert.equal(window.document.querySelectorAll("[data-injectra]").length, 2);
    await runtime({ operation: "remove", id: "test" });
    assert.equal(balance, 0);
    assert.equal(window.document.querySelectorAll("[data-injectra]").length, 0);
    assert.equal(window.document.body.textContent, "Original");
    assert.equal((await runtime({ operation: "remove", id: "test" })).status, "absent");
  } finally { dom.window.close(); }
});

test("script failure cleans owned nodes and registered effects", async () => {
  const { dom, window, runtime } = setup();
  let cleaned = false;
  try {
    await assert.rejects(runtime(request, async context => {
      context.onCleanup(() => { cleaned = true; });
      throw new Error("script failed");
    }), /script failed/);
    assert.equal(cleaned, true);
    assert.equal(window.document.querySelectorAll("[data-injectra]").length, 0);
  } finally { dom.window.close(); }
});

test("cleanup failures remain visible while owned nodes are removed", async () => {
  const { dom, window, runtime } = setup();
  try {
    await runtime(request, async context => context.onCleanup(() => { throw Error("cannot undo"); }));
    await assert.rejects(runtime({ operation: "remove", id: "test" }), /Cleanup incomplete/);
    assert.equal(window.document.querySelectorAll("[data-injectra]").length, 0);
  } finally { dom.window.close(); }
});

test("independent bundle IDs do not remove one another", async () => {
  const { dom, window, runtime } = setup();
  try {
    await runtime(request, async () => {});
    await runtime({ ...request, id: "second" }, async () => {});
    await runtime({ operation: "remove", id: "test" });
    assert.equal(window.document.querySelectorAll('[data-injectra="second"]').length, 2);
  } finally { dom.window.close(); }
});
