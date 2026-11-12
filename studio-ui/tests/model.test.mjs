import test from 'node:test';
import assert from 'node:assert/strict';
import { load, state } from './load.mjs';
const { StudioModel, targetIdentity, acknowledgeSave, canEdit } = await load('model');
test('target generations invalidate selection and stop automatic application', () => {
  const model = new StudioModel(), original = state(); model.accept(original); model.select(targetIdentity(original.targets[0])); model.enableAuto();
  assert.equal(model.target().id, 'page-1');
  model.accept({ ...original, targets: [{ ...original.targets[0], generation: 'generation-2' }] });
  assert.equal(model.target(), undefined); assert.equal(model.autoApply, false); assert.match(model.autoPausedReason, /改变/);
});
test('unknown results pause once and are never treated as successful execution', () => {
  const model = new StudioModel(); model.accept(state()); model.enableAuto();
  const unknown = { ...state(), runs: [{ request_id: 'r1', target: 'page-1', revision: 'revision-1', status: 'failed', error: { certainty: 'Unknown' } }] };
  model.accept(unknown); assert.equal(model.autoApply, false); assert.match(model.autoPausedReason, /未知/);
  model.enableAuto(); model.accept(unknown); assert.equal(model.autoApply, true, 'same historical event does not defeat explicit operator choice');
});
test('save acknowledgement preserves edits made while the request was pending', () => {
  const draft = { path: 'page.js', content: 'newer unsaved', baseHash: 'old', dirty: true, conflict: false, editSequence: 3 };
  const ack = acknowledgeSave(draft, 2, 'saved-hash'); assert.equal(ack.dirty, true); assert.equal(ack.content, draft.content); assert.equal(ack.baseHash, 'saved-hash');
  assert.equal(canEdit({ origin: 'application' }), false); assert.equal(canEdit({ origin: 'injectra' }), false); assert.equal(canEdit({ origin: 'user' }), true);
});
