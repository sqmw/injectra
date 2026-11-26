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

test("list reports managed bundles across empty, active, concurrent and cleaned states", async () => {
  const { dom, runtime } = setup();
  const list = async () => JSON.parse(JSON.stringify(await runtime({ operation: "list" }, null)));
  try {
    assert.deepEqual(await list(), { bundles: [] });
    await runtime(request, async context => { context.onCleanup(() => {}); });
    await runtime({ operation: "apply", id: "second", css: "a{}", html: "" }, async () => {});
    assert.deepEqual(await list(), {
      bundles: [
        { id: "test", nodes: 2, cleanups: 1 },
        { id: "second", nodes: 1, cleanups: 0 },
      ],
    });
    let release;
    const inflight = runtime(
      { operation: "apply", id: "busy-one", css: "b{}", html: "" },
      () => new Promise(resolve => { release = resolve; }),
    );
    assert.deepEqual(await list(), {
      bundles: [
        { id: "test", nodes: 2, cleanups: 1 },
        { id: "second", nodes: 1, cleanups: 0 },
      ],
    });
    release();
    await inflight;
    await runtime({ operation: "remove", id: "test" });
    await runtime({ operation: "remove", id: "second" });
    await runtime({ operation: "remove", id: "busy-one" });
    await assert.rejects(
      runtime(request, async () => { throw new Error("boom"); }),
      /boom/,
    );
    assert.deepEqual(await list(), { bundles: [] });
  } finally { dom.window.close(); }
});

test("declarative dom mount works when innerHTML is blocked by Trusted Types and policy errors separate from script errors", async () => {
  const { dom, window, runtime } = setup();
  const blockedDesc = Object.getOwnPropertyDescriptor(window.Element.prototype, "innerHTML");
  Object.defineProperty(window.Element.prototype, "innerHTML", {
    configurable: true,
    get: blockedDesc.get,
    set() {
      throw new window.TypeError(
        "Failed to set the 'innerHTML' property on 'Element': This document requires 'TrustedHTML' assignment.",
      );
    },
  });
  try {
    // 1. innerHTML path reports structured policy block and cleans up mounted <style>.
    await assert.rejects(
      runtime(request, async () => {}),
      /\[injectra:policy:html-mount\].*TrustedHTML/,
    );
    assert.equal(window.document.querySelectorAll("[data-injectra]").length, 0);

    // 2. Declarative DOM path mounts HTML controls, text, and SVG without touching innerHTML.
    const domReq = {
      operation: "apply",
      id: "tt.safe",
      css: '[data-injectra="tt.safe"] button { color: rgb(1, 2, 3); }',
      html: "",
      dom: [
        { t: "button", a: [["type", "button"], ["data-v", "a&b"]], c: ["<Safe>"] },
        { t: "svg", ns: "svg", a: [["viewBox", "0 0 10 10"]], c: [{ t: "circle", ns: "svg", a: [["r", "4"]] }] },
      ],
    };
    let cleaned = 0;
    const applied = await runtime(domReq, async context => {
      assert.ok(context.root);
      context.onCleanup(() => { cleaned++; });
    });
    assert.equal(applied.status, "applied");
    assert.equal(window.document.querySelector('[data-injectra="tt.safe"] button').textContent, "<Safe>");
    assert.equal(
      window.document.querySelector('[data-injectra="tt.safe"] circle').namespaceURI,
      "http://www.w3.org/2000/svg",
    );

    const updated = await runtime({ ...domReq, dom: [{ t: "span", c: ["v2"] }] }, async context => {
      context.onCleanup(() => { cleaned++; });
    });
    assert.equal(updated.status, "updated");
    assert.equal(cleaned, 1);
    assert.equal(window.document.querySelectorAll('[data-injectra="tt.safe"]').length, 2);

    // 3. User script failure is NOT mislabeled as a policy block.
    let scriptErr;
    try {
      await runtime(domReq, async () => { throw new Error("user-logic-error"); });
    } catch (e) {
      scriptErr = e;
    }
    assert.match(String(scriptErr), /user-logic-error/);
    assert.doesNotMatch(String(scriptErr), /\[injectra:policy:/);
    assert.equal(window.document.querySelectorAll('[data-injectra="tt.safe"]').length, 0);
  } finally {
    Object.defineProperty(window.Element.prototype, "innerHTML", blockedDesc);
    dom.window.close();
  }
});
