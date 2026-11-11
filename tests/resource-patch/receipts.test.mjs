import test from "node:test";
import assert from "node:assert/strict";
import { evaluateProof } from "./receipts.mjs";

const expected = { path: "/owned/aDrive-Test.app/Contents/Resources/app.asar/dist/index.html", nonce: "unique-run" };
const base = { ...expected, peer: "127.0.0.1" };
const events = [
  { ...base, phase: "loaded" },
  { ...base, phase: "applied", status: "applied", connected: "true", appRootPresent: "true", color: "rgb(53, 32, 107)" },
  { ...base, phase: "clicked", connected: "true" },
];
test("proof requires ordered code execution, visible style and interaction from the same run/page", () => {
  assert.equal(evaluateProof(events, expected).passed, true);
  assert.equal(evaluateProof(events.slice(1), expected).passed, false);
  assert.equal(evaluateProof([events[2], events[0], events[1]], expected).passed, false);
  for (const changed of [{ color: "transparent" }, { nonce: "other-run" }, { path: "/other-page" }, { peer: "external" }, { connected: "false" }]) {
    assert.equal(evaluateProof([events[0], { ...events[1], ...changed }, events[2]], expected).passed, false);
  }
});
