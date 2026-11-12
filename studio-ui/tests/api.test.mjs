import test from 'node:test';
import assert from 'node:assert/strict';
import { load } from './load.mjs';
const { StudioApi, takeToken, StudioError, startupFailure } = await load('api');
test('fetch is invoked with the global receiver required by native browser APIs', async () => {
  let invoked = false;
  const nativeLikeFetch = async function (url) {
    assert.equal(this, globalThis, 'Illegal invocation: receiver must be the browser global');
    invoked = true;
    assert.equal(url, 'http://127.0.0.1:1234/api/state');
    return new Response('{"ok":true}', { status: 200 });
  };
  const api = new StudioApi('fixture-token', 'http://127.0.0.1:1234', nativeLikeFetch);
  assert.deepEqual(await api.request('/api/state'), { ok: true }); assert.equal(invoked, true);
});
test('startup failure keeps the actual diagnostic message while redacting credentials', () => {
  assert.equal(startupFailure(new TypeError('Illegal invocation'), 'fixture-token').message, 'Illegal invocation');
  const token = 'fixture+/secret';
  const failure = startupFailure(new StudioError(`Rejected ${token} and ${encodeURIComponent(token)}`, 401, 'NotExecuted'), token);
  assert.equal(failure.message, 'Rejected [redacted] and [redacted]');
  assert.equal(failure.status, 401); assert.equal(failure.certainty, 'NotExecuted');
});
test('fragment credentials are cleared and used only in same-origin headers', async () => {
  const histories = [];
  assert.equal(takeToken({ hash: '#token=private-token', pathname: '/', search: '?workspace=id' }, { replaceState: (...args) => histories.push(args) }), 'private-token');
  assert.equal(histories[0][2], '/?workspace=id');
  const requests = []; const api = new StudioApi('private-token', 'http://127.0.0.1:1234', async (...args) => { requests.push(args); return new Response('{}', { status: 200 }); });
  await api.request('/api/state'); assert.equal(requests[0][0], 'http://127.0.0.1:1234/api/state'); assert.equal(requests[0][1].headers.Authorization, 'Bearer private-token');
  await assert.rejects(api.request('https://outside.invalid/api/state'), /same-origin/); await assert.rejects(api.request('/not-an-api'), /same-origin/); assert.equal(requests.length, 1);
});
test('unknown certainty and file conflicts survive the HTTP boundary', async () => {
  const api = new StudioApi('token', 'http://127.0.0.1:1234', async () => new Response(JSON.stringify({ message: 'result unconfirmed', certainty: 'Unknown' }), { status: 409 }));
  await assert.rejects(api.request('/api/run', {}), error => error instanceof StudioError && error.status === 409 && error.certainty === 'Unknown');
});
test('websocket uses first-frame authentication and ignores state before acknowledgement', () => {
  const previous = globalThis.WebSocket;
  let socket;
  globalThis.WebSocket = class extends EventTarget {
    constructor(url) { super(); this.url = url; this.sent = []; socket = this; }
    send(value) { this.sent.push(JSON.parse(value)); }
    close() { this.dispatchEvent(new Event('close')); }
  };
  try {
    const api = new StudioApi('private-token', 'http://127.0.0.1:1234');
    const events = [], statuses = [];
    const close = api.events(value => events.push(value), value => statuses.push(value));
    assert.equal(socket.url, 'ws://127.0.0.1:1234/api/events');
    socket.dispatchEvent(new Event('open'));
    assert.deepEqual(socket.sent, [{ type: 'authenticate', token: 'private-token' }]);
    socket.dispatchEvent(new MessageEvent('message', { data: JSON.stringify({ type: 'state', state: {} }) }));
    assert.equal(events.length, 0);
    socket.dispatchEvent(new MessageEvent('message', { data: JSON.stringify({ type: 'authenticated' }) }));
    socket.dispatchEvent(new MessageEvent('message', { data: JSON.stringify({ type: 'state', state: {} }) }));
    assert.equal(events.length, 1); close(); assert.deepEqual(statuses, [true, false]);
  } finally { globalThis.WebSocket = previous; }
});
