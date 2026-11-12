import test from 'node:test';
import assert from 'node:assert/strict';
import { load, state, requireTool } from './load.mjs';
const { JSDOM } = requireTool('jsdom');
const { StudioApp } = await load('studio');
const tick = () => new Promise(resolve => setImmediate(resolve));
function fixture() {
  const dom = new JSDOM('<div id="root"></div>', { url: 'http://127.0.0.1:1234' });
  globalThis.window = dom.window; globalThis.document = dom.window.document;
  const current = state(), calls = [], contents = { 'page.js': 'original increment', 'startup.cjs': 'startup increment', 'original.js': '<script>not executable here</script>' };
  let eventHandler, editor;
  const api = { async request(path, body, method) { calls.push({ path, body, method }); if (path === '/api/state') return structuredClone(current); if (path.startsWith('/api/files?')) { const name = new URL(path, 'http://local').searchParams.get('path'); return { ...current.files.find(file => file.path === name), content: contents[name] }; } if (path === '/api/files') { contents[body.path] = body.content; current.workspace.revision = 'revision-2'; current.files.find(file => file.path === body.path).hash = 'hash-2'; return { revision: 'revision-2' }; } if (path === '/api/run') return { request_id: 'run-1' }; return {}; }, events(handler, status) { eventHandler = handler; status(true); return () => {}; } };
  const editorFactory = (parent, change) => { editor = document.createElement('textarea'); parent.append(editor); editor.addEventListener('input', () => change(editor.value)); return { load(path, content, editable) { editor.value = content; editor.readOnly = !editable; }, focusLine() {}, destroy() {} }; };
  const app = new StudioApp(document.getElementById('root'), api, editorFactory);
  const findButton = text => [...document.querySelectorAll('button')].find(button => button.textContent === text);
  return { app, dom, calls, current, get editor() { return editor; }, findButton, event: data => eventHandler(data), close() { dom.window.close(); delete globalThis.window; delete globalThis.document; } };
}
test('origin views are read-only and incoming state never overwrites a dirty draft', async () => {
  const f = fixture(); try {
    await f.app.start(); f.findButton('original.js').click(); await tick(); assert.equal(f.editor.readOnly, true); assert.match(f.editor.value, /<script>/); assert.equal(document.querySelector('.code-editor script'), null);
    f.findButton('page.js').click(); await tick(); f.editor.value = 'unsaved work'; f.editor.dispatchEvent(new window.Event('input'));
    f.current.files[0].hash = 'changed-elsewhere'; f.event({ type: 'state', state: structuredClone(f.current) });
    assert.equal(f.editor.value, 'unsaved work'); assert.equal(f.findButton('保存').disabled, true); assert.match(document.querySelector('.file-header').textContent, /冲突/);
  } finally { f.close(); }
});
test('save is distinct from run and submits exact revision and selected generation', async () => {
  const f = fixture(); try {
    await f.app.start(); f.findButton('page.js').click(); await tick();
    f.editor.value = 'edited'; f.editor.dispatchEvent(new window.Event('input')); f.findButton('保存').click(); await tick(); await tick();
    assert.equal(f.calls.some(call => call.path === '/api/run'), false);
    assert.equal(f.calls.find(call => call.path === '/api/files').body.expected_hash, 'hash-1');
    const select = document.querySelector('select[aria-label="目标页面"]'); select.selectedIndex = 1; select.dispatchEvent(new window.Event('change'));
    f.findButton('运行已保存版本').click(); await tick(); await tick();
    assert.deepEqual(f.calls.find(call => call.path === '/api/run').body, { target: 'page-1', generation: 'generation-1', revision: 'revision-2' });
  } finally { f.close(); }
});
test('unknown events disable automatic apply without submitting a replay', async () => {
  const f = fixture(); try {
    await f.app.start(); const select = document.querySelector('select[aria-label="目标页面"]'); select.selectedIndex = 1; select.dispatchEvent(new window.Event('change'));
    const auto = document.querySelector('input[type=checkbox]'); auto.checked = true; auto.dispatchEvent(new window.Event('change'));
    f.event({ type: 'run', run: { request_id: 'unknown-1', target: 'page-1', revision: 'revision-1', status: 'unknown' } });
    assert.equal(auto.checked, false); assert.equal(auto.disabled, true); assert.equal(f.calls.some(call => call.path === '/api/run'), false);
  } finally { f.close(); }
});
test('startup save displays pending restart and never auto-runs a page', async () => {
  const f = fixture(); try {
    await f.app.start(); const select = document.querySelector('select[aria-label="目标页面"]'); select.selectedIndex = 1; select.dispatchEvent(new window.Event('change'));
    const auto = document.querySelector('input[type=checkbox]'); auto.checked = true; auto.dispatchEvent(new window.Event('change'));
    f.findButton('startup.cjs').click(); await tick(); f.editor.value = 'changed startup'; f.editor.dispatchEvent(new window.Event('input')); f.findButton('保存').click(); await tick(); await tick();
    assert.equal(f.calls.some(call => call.path === '/api/run'), false); assert.match(document.querySelector('.startup-card').textContent, /等待重启/);
  } finally { f.close(); }
});
