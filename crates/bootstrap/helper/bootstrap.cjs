'use strict';
// Installed in a package-owned directory, required by the unchanged CJS main module.
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const crypto = require('node:crypto');
const { app, webContents } = require('electron');
const { guardedSource } = require('./evaluation.cjs');
const config = JSON.parse(fs.readFileSync(path.join(__dirname, 'config.json'), 'utf8'));
const VERSION = 1;
if (process.env.INJECTRA_PREFLIGHT_NONCE) {
  try { JSON.parse(fs.readFileSync(path.join(config.stage_root,'current.json'),'utf8')); fs.writeSync(1,'INJECTRA_PREFLIGHT:'+JSON.stringify({artifact:config.artifact,nonce:process.env.INJECTRA_PREFLIGHT_NONCE})+'\n'); process.exit(0); }
  catch(error) { fs.writeSync(2,'INJECTRA_PREFLIGHT_ERROR:'+String(error)+'\n'); process.exit(1); }
  throw Error('Preflight must not continue into original entry');
}
const documentKey = 'injectra.document.' + config.artifact;
const executionSources = new Map();
const instance = crypto.randomBytes(32).toString('hex');
const pages = new Map();
let session = null, closing = false, polling = false, reporting = false;
const receipts = [], logs = [];
let startup = { status: 'not_configured', revision: null, origin: 'user', stage: 'startup' };
let exportProgram = null;
const nonce = () => crypto.randomBytes(32).toString('hex');
function mac(role, client, server) {
  return crypto.createHmac('sha256', Buffer.from(config.key, 'hex'))
    .update(JSON.stringify(['injectra-bootstrap-v1', role, config.artifact, instance, process.pid, client, server])).digest('hex');
}
function request(route, data, bearer = session) {
  return new Promise((resolve, reject) => {
    const base = new URL(config.controller_origin);
    if (base.protocol !== 'http:' || !['127.0.0.1', '[::1]'].includes(base.hostname)) return reject(Error('Controller must be loopback HTTP'));
    const body = data === undefined ? null : JSON.stringify(data);
    const req = http.request(new URL(route, base), { method: body === null ? 'GET' : 'POST', headers: {
      ...(bearer ? { Authorization: 'Bearer ' + bearer } : {}),
      ...(body ? { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(body) } : {}),
    } }, response => {
      let text = '', length = 0;
      response.on('data', chunk => { length += chunk.length; if (length > 5 * 1024 * 1024) response.destroy(Error('Bridge response too large')); else text += chunk; });
      response.on('error', reject);
      response.on('end', () => { try { if (response.statusCode !== 200) throw Error('Bridge HTTP ' + response.statusCode); resolve(JSON.parse(text)); } catch (e) { reject(e); } });
    });
    req.setTimeout(20000, () => req.destroy(Error('Bridge deadline')));
    req.on('socket', socket => socket.unref());
    req.on('error', reject); req.end(body);
  });
}
function frame(wc) { try { return { routing: wc.mainFrame.routingId, process: wc.mainFrame.processId }; } catch { return null; } }
function sameFrame(page) { const f = frame(page.wc); return f && page.routing === f.routing && page.process === f.process; }
function invalidate(page) { page.generation = nonce(); page.ready = false; page.routing = null; page.process = null; }
function visiblePages() {
  return [...pages.values()].filter(p => !p.wc.isDestroyed() && p.ready && sameFrame(p)).map(p => ({
    id: String(p.wc.id), generation: p.generation, title: p.wc.getTitle().slice(0,4096), url: p.wc.getURL().slice(0,16384),
  }));
}
function matchesSelector(url) {
  try { const u = new URL(url); const s = config.selector;
    return s && (!s.path_suffix || u.pathname.endsWith(s.path_suffix)) && (!s.origin || u.origin === s.origin || (u.protocol + '//' + u.host) === s.origin);
  } catch { return false; }
}
function track(wc) {
  if (pages.has(wc.id) || !['window','browserView','webview','offscreen'].includes(wc.getType())) return;
  const p = { wc, generation: nonce(), ready: false, exported: false }; pages.set(wc.id, p);
  async function ready() {
    if (wc.isDestroyed()) return;
    const f = frame(wc); if (!f || !wc.getURL() || wc.getURL() === 'about:blank') return;
    p.generation = nonce(); p.ready = false; p.routing = f.routing; p.process = f.process;
    const generation = p.generation;
    try { await wc.executeJavaScript('globalThis[Symbol.for('+JSON.stringify(documentKey)+')]='+JSON.stringify(generation)); }
    catch { return; }
    if (wc.isDestroyed() || p.generation !== generation || !sameFrame(p)) return;
    p.ready = true;
    if (config.mode === 'export' && !p.exported && exportProgram && matchesSelector(wc.getURL())) {
      p.exported = true;
      wc.executeJavaScript(exportProgram).catch(error => process.stderr.write('INJECTRA_EXPORT_ERROR:' + String(error) + '\n'));
    }
    flush().catch(() => {});
  }
  wc.on('did-start-navigation', (_event, url, inPlace, mainFrame) => {
    // New Electron event-detail forms are handled conservatively.
    if (typeof url === 'object') { if (url.isMainFrame !== false && !url.isSameDocument) invalidate(p); }
    else if (mainFrame !== false && !inPlace) invalidate(p);
  });
  wc.on('render-process-gone', () => invalidate(p));
  wc.on('dom-ready', ready);
  wc.once('destroyed', () => { pages.delete(wc.id); flush().catch(() => {}); });
  wc.on('console-message', (_event, level, message, line, source) => {
    if (level && typeof level === 'object') { const d = level; level = d.level; message = d.message; line = d.lineNumber; source = d.sourceId; }
    const context = executionSources.get(source);
    if (!context || context.page !== String(wc.id)) return;
    if (logs.length < 100) logs.push({...context,level:String(level),message:String(message).slice(0,8192),source,line:Number(line)||0});
  });
  if (app.isReady() && !wc.isLoadingMainFrame()) ready();
}
async function flush() {
  if (!session || reporting || closing) return;
  reporting = true; const current = session;
  const sentReceipts = receipts.splice(0,64), sentLogs = logs.splice(0,100);
  try { await request('/bridge/report', { pages: visiblePages(), receipts: sentReceipts, logs: sentLogs, startup }, current); }
  catch (e) {
    // Receipt retry is allowed; command execution is never replayed.
    if (session === current) { receipts.unshift(...sentReceipts); logs.unshift(...sentLogs); }
    throw e;
  } finally { reporting = false; }
}
async function execute(command) {
  const current = session, p = pages.get(Number(command.page));
  let receipt = { request_id:command.request_id, page:command.page, generation:command.generation, revision:command.revision, ok:false, value:null, certainty:'NotExecuted' };
  if (!p || p.wc.isDestroyed() || !p.ready || p.generation !== command.generation || !sameFrame(p)) {
    receipt.error = 'Page generation changed before execution';
  } else if (command.operation === 'reload') {
    try { p.wc.reload(); receipt = {...receipt,ok:true,value:{reload_requested:true},certainty:'Executed'}; }
    catch(e) {receipt.error=String(e);receipt.certainty='Unknown';}
  } else if (command.operation === 'evaluate') {
    try {
      const evaluation = guardedSource(command, documentKey);
      executionSources.set(evaluation.url,evaluation.context);
      while(executionSources.size>256)executionSources.delete(executionSources.keys().next().value);
      const result = await p.wc.executeJavaScript(evaluation.source);
      receipt.certainty = result.certainty;
      if(result.ok){
        if(typeof result.encoded!=='string'||Buffer.byteLength(result.encoded)>512*1024)throw Error('Result encoding exceeds bridge limits');
        receipt.value=JSON.parse(result.encoded);receipt.ok=true;
      }else receipt.error=result.error+(result.stack?'\n'+result.stack:'');
    } catch(e) { receipt.certainty = p.wc.isDestroyed() || p.generation !== command.generation || !sameFrame(p) ? 'Unknown' : 'Executed'; receipt.error = String(e); }
  } else receipt.error = 'Unsupported operation';
  if (session === current) { receipts.push(receipt); await flush(); }
}
async function connect() {
  const client = nonce();
  const challenge = await request('/bridge/challenge', { artifact:config.artifact, instance, pid:process.pid, client_nonce:client }, null);
  const expected = Buffer.from(mac('server', client, challenge.server_nonce),'hex');
  const actual = Buffer.from(String(challenge.server_proof),'hex');
  if (actual.length !== expected.length || !crypto.timingSafeEqual(expected,actual)) throw Error('Controller identity proof failed');
  const registered = await request('/bridge/register', {challenge_id:challenge.challenge_id,client_proof:mac('client',client,challenge.server_nonce)},null);
  if (registered.instance !== instance) throw Error('Bridge instance mismatch');
  session = registered.session; receipts.length=0;logs.length=0; await flush();
}
async function poll() {
  if (polling || closing || config.mode !== 'development') return;
  polling = true;
  try { if (!session) await connect(); const result = await request('/bridge/poll'); if (result.command) await execute(result.command); }
  catch(error) { session=null; /* reconnect creates a fresh session; never replay commands */ }
  finally { polling=false; if(!closing) setTimeout(poll,session?10:1000).unref(); }
}
app.on('web-contents-created', (_event,wc) => track(wc));
app.whenReady().then(() => {for(const wc of webContents.getAllWebContents())track(wc);});
app.on('will-quit', () => {closing=true;if(session)request('/bridge/disconnect',{},session).catch(()=>{});});
try {
  if (config.version !== VERSION) throw Error('Unsupported Injectra bootstrap version');
  let current;
  if (config.mode === 'export') {
    current = JSON.parse(fs.readFileSync(path.join(__dirname,'export/current.json'),'utf8'));
    exportProgram = fs.readFileSync(path.join(__dirname,'export/program.js'),'utf8');
  } else {
    current = JSON.parse(fs.readFileSync(path.join(config.stage_root,'current.json'),'utf8'));
  }
  startup.revision = current.revision;
  if (current.startup) {
    const base = config.mode === 'export' ? path.join(__dirname,'export/workspace') : path.join(config.stage_root,current.revision,'workspace');
    const filename = path.resolve(base,current.startup);
    if (!filename.startsWith(path.resolve(base)+path.sep)) throw Error('Startup path escapes staged workspace');
    const bytes = fs.readFileSync(filename);
    if (crypto.createHash('sha256').update(bytes).digest('hex') !== current.startup_hash) throw Error('Staged startup digest mismatch');
    require(filename);
    startup.status = 'executed'; startup.source = 'injectra://' + config.workspace + '/' + current.revision + '/' + current.startup;
  }
} catch(error) {
  startup.status='failed';startup.error=String(error);startup.stack=String(error.stack||'');
  process.stderr.write('INJECTRA_STARTUP:' + JSON.stringify(startup) + '\n');
  app.exit(1);
  throw error;
}
if (config.mode === 'development') { setImmediate(poll); const heartbeat=setInterval(()=>flush().catch(()=>{session=null;}),1000);heartbeat.unref(); }
module.exports = Object.freeze({ version:VERSION, artifact:config.artifact });
