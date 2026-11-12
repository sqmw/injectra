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
  let eventHandler, editor, requestHook;
  let version = 1;
  const api = {
    async request(path, body, method) {
      calls.push({ path, body, method });
      if (requestHook) { const result = await requestHook({ path, body, method }); if (result !== undefined) return result; }
      if (path === '/api/state') return structuredClone(current);
      if (path.startsWith('/api/files?')) { const name = new URL(path, 'http://local').searchParams.get('path'); return { ...current.files.find(file => file.path === name), content: contents[name] }; }
      if (path === '/api/files') {
        let file = current.files.find(file => file.path === body.path);
        if (body.expected_hash === null && file) throw Error('File already exists');
        if (!file) { const relative = body.path.replace(/^user:/, ''); file = { path: body.path, relative_path: relative, origin: 'user', stage: relative === 'startup.cjs' ? 'startup' : 'page', hash: '' }; current.files.push(file); }
        contents[body.path] = body.content; version++; current.workspace.revision = 'revision-' + version; file.hash = 'hash-' + version;
        return { revision: current.workspace.revision };
      }
      if (path === '/api/run') return { request_id: 'run-1' };
      return {};
    },
    events(handler, status) { eventHandler = handler; status(true); return () => {}; },
  };
  const editorFactory = (parent, change) => { editor = document.createElement('textarea'); parent.append(editor); editor.addEventListener('input', () => change(editor.value)); return { load(path, content, editable) { editor.value = content; editor.readOnly = !editable; }, focusLine() {}, destroy() {} }; };
  const app = new StudioApp(document.getElementById('root'), api, editorFactory);
  const findButton = text => [...document.querySelectorAll('button')].find(button => button.textContent === text);
  return { app, dom, calls, current, get editor() { return editor; }, findButton, event: data => eventHandler(data), hook: handler => { requestHook = handler; }, close() { dom.window.close(); delete globalThis.window; delete globalThis.document; } };
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

test('manual acknowledgement identifies the exact unknown request and queued does not clear the pause', async () => {
  const f = fixture(); try {
    const unknown = { request_id: 'unknown-1', target: 'page-1', generation: 'generation-1', revision: 'revision-1', status: 'unknown' };
    f.current.runs = [unknown]; await f.app.start();
    const select = document.querySelector('select[aria-label="目标页面"]'); select.selectedIndex = 1; select.dispatchEvent(new window.Event('change'));
    assert.match(document.querySelector('.automatic-notice').textContent, /可能已经执行/);
    f.hook(({ path }) => { if (path === '/api/run') { f.current.runs.push({ ...unknown, request_id: 'manual-2', status: 'queued' }); return { request_id: 'manual-2' }; } });
    f.findButton('确认未知状态并运行').click(); await tick(); await tick();
    const request = f.calls.find(call => call.path === '/api/run'); assert.equal(request.body.acknowledge_unknown, 'unknown-1');
    const auto = document.querySelector('input[type=checkbox]'); assert.equal(auto.disabled, true); assert.equal(auto.checked, false);
    assert.equal(f.findButton('运行已保存版本').disabled, true); assert.match(document.querySelector('.results-list').textContent, /排队中/);
    f.current.runs[1].status = 'succeeded'; f.event({ type: 'state', state: structuredClone(f.current) });
    assert.equal(auto.disabled, false); assert.equal(auto.checked, false, 'explicit re-enable is still required');
  } finally { f.close(); }
});

test('automatic save application never acknowledges a previous request', async () => {
  const f = fixture(); try {
    await f.app.start(); const select = document.querySelector('select[aria-label="目标页面"]'); select.selectedIndex = 1; select.dispatchEvent(new window.Event('change'));
    const auto = document.querySelector('input[type=checkbox]'); auto.checked = true; auto.dispatchEvent(new window.Event('change'));
    f.findButton('page.js').click(); await tick(); f.editor.value = 'new increment'; f.editor.dispatchEvent(new window.Event('input')); f.findButton('保存').click(); await tick(); await tick(); await tick();
    const run = f.calls.find(call => call.path === '/api/run'); assert.ok(run); assert.equal(Object.hasOwn(run.body, 'acknowledge_unknown'), false);
    assert.equal(run.body.revision, 'revision-2');
  } finally { f.close(); }
});

test('new startup files use create-only user paths and retain another file draft', async () => {
  const f = fixture(); try {
    f.current.files = f.current.files.filter(file => file.path !== 'startup.cjs');
    await f.app.start(); f.findButton('page.js').click(); await tick(); f.editor.value = 'unsaved existing draft'; f.editor.dispatchEvent(new window.Event('input'));
    const name = document.querySelector('input[placeholder^="startup.cjs"]'); name.value = 'startup.cjs'; f.findButton('新增增量文件').click(); await tick(); await tick();
    const request = f.calls.find(call => call.path === '/api/files'); assert.deepEqual(request.body, { path: 'user:startup.cjs', content: '', expected_hash: null });
    assert.equal(document.querySelector('.file-header strong').textContent, 'startup.cjs'); assert.equal(document.querySelector('.file-header .badge').textContent, '启动增量');
    assert.equal(f.editor.readOnly, false); assert.equal(f.editor.value, ''); assert.equal(f.app.hasDirtyDrafts(), true);
    f.findButton('page.js').click(); await tick(); assert.equal(f.editor.value, 'unsaved existing draft');
  } finally { f.close(); }
});

test('file creation conflicts preserve both the draft and actionable error', async () => {
  const f = fixture(); try {
    await f.app.start(); f.findButton('page.js').click(); await tick(); f.editor.value = 'do not overwrite'; f.editor.dispatchEvent(new window.Event('input'));
    f.hook(({ path, body }) => { if (path === '/api/files' && body.expected_hash === null) throw Error('Creation conflict: file already exists'); });
    const name = document.querySelector('input[placeholder^="startup.cjs"]'); name.value = 'helpers/setup.cjs'; f.findButton('新增增量文件').click(); await tick(); await tick();
    assert.equal(f.editor.value, 'do not overwrite'); assert.equal(f.app.hasDirtyDrafts(), true); assert.equal(name.value, 'helpers/setup.cjs');
    assert.match(document.querySelector('.notice').textContent, /Creation conflict/);
    name.value = '../application.js'; f.findButton('新增增量文件').click(); await tick();
    assert.equal(f.calls.filter(call => call.path === '/api/files').length, 1);
  } finally { f.close(); }
});

test('export requires and submits a selected target generation with a visible scope', async () => {
  const f = fixture(); try {
    await f.app.start(); assert.equal(f.findButton('导出固定版本').disabled, true);
    const select = document.querySelector('select[aria-label="目标页面"]'); select.selectedIndex = 1; select.dispatchEvent(new window.Event('change'));
    assert.equal(f.findButton('导出固定版本').disabled, false);
    document.querySelector('input[placeholder="选择一个新的输出目录"]').value = '/new/export';
    assert.match(document.querySelector('.deployment').textContent, /app:\/\/real/); assert.match(document.querySelector('.deployment').textContent, /当前路径：\/page/);
    f.findButton('导出固定版本').click(); await tick(); await tick();
    assert.deepEqual(f.calls.find(call => call.path === '/api/export').body, { destination: '/new/export', target: 'page-1', generation: 'generation-1' });
    f.current.targets[0].generation = 'generation-2'; f.event({ type: 'state', state: structuredClone(f.current) });
    assert.equal(f.findButton('导出固定版本').disabled, true);
  } finally { f.close(); }
});

test('pending startup snapshot does not prohibit independent page execution', async () => {
  const f = fixture(); try {
    await f.app.start(); const select = document.querySelector('select[aria-label="目标页面"]'); select.selectedIndex = 1; select.dispatchEvent(new window.Event('change'));
    assert.match(document.querySelector('.startup-card').textContent, /页面增量可单独更新/);
    assert.equal(f.findButton('运行已保存版本').disabled, false);
  } finally { f.close(); }
});
