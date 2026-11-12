import type { StudioApi } from './api';
import { StudioError } from './api';
import { createEditor, type SourceEditor } from './editor';
import { StudioModel, targetIdentity, canEdit, statusLabel, acknowledgeSave, isUnknown, type Draft } from './model';
import type { StudioState, FileContent, SourceFile, Run, StudioEvent } from './types';

function element<K extends keyof HTMLElementTagNameMap>(tag: K, className = '', text = ''): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag); node.className = className; node.textContent = text; return node;
}
function button(text: string, action: () => void, className = ''): HTMLButtonElement { const node = element('button', className, text); node.type = 'button'; node.addEventListener('click', action); return node; }
function field(label: string, input: HTMLElement): HTMLLabelElement { const node = element('label', 'field'); node.append(element('span', '', label), input); return node; }
function option(value: string, label: string): HTMLOptionElement { const node = element('option', '', label); node.value = value; return node; }
const format = (value: unknown): string => typeof value === 'string' ? value : JSON.stringify(value, null, 2) ?? '';
const short = (value?: string): string => value ? value.slice(0, 12) : '—';

export class StudioApp {
  readonly model = new StudioModel();
  private drafts = new Map<string, Draft>();
  private currentFile: SourceFile | null = null;
  private loadSequence = 0;
  private busy = false;
  private stopEvents?: () => void;
  private manualRequest: string | null = null;
  private editor: SourceEditor;
  private workspaceLabel = element('span', 'workspace-name', '正在连接…');
  private connectionLabel = element('span', 'connection-state', '连接中');
  private notice = element('div', 'notice');
  private files = element('nav', 'file-tree');
  private fileTitle = element('strong', '', '选择一个增量文件');
  private fileBadge = element('span', 'badge', '只读');
  private fileStatus = element('span', 'muted', '');
  private revisionLabel = element('span', 'revision', '版本 —');
  private saveButton = button('保存', () => { void this.save(); });
  private rereadButton = button('重新载入', () => { void this.reread(); }, 'quiet');
  private runButton = button('运行已保存版本', () => { void this.run(); }, 'primary');
  private auto = element('input');
  private target = element('select');
  private backend = element('select');
  private endpoint = element('input');
  private endpointLabel: HTMLLabelElement;
  private targetDetails = element('p', 'target-details');
  private startup = element('div', 'startup-card');
  private artifact = element('pre', 'artifact');
  private resultList = element('div', 'results-list');
  private logList = element('pre', 'log-list');
  private appPath = element('input');
  private dataMode = element('select');
  private destination = element('input');
  private importPath = element('input');
  private importButton = button('导入 manifest', () => { void this.operation('/api/import', { manifest: this.importPath.value.trim() }); });
  private prepareButton = button('准备开发副本', () => { if (!this.appPath.value.trim() || !this.dataMode.value) return this.notify('请填写应用路径并选择数据模式。', true); void this.operation('/api/prepare', { app: this.appPath.value.trim(), data_mode: this.dataMode.value }); });
  private devButton = button('启动开发实例', () => { void this.operation('/api/dev', {}); });
  private reloadButton = button('重载页面', () => { const target = this.model.target(); if (!target) return; this.model.pauseAuto('页面重载后请重新选择目标并手动运行。'); this.model.select(null); this.render(); void this.operation('/api/reload', { target: target.id, generation: target.generation }); });
  private restartButton = button('重启开发实例', () => { this.model.pauseAuto('开发实例重启后请重新选择页面。'); this.model.select(null); this.render(); void this.operation('/api/restart', {}); });
  private exportButton = button('导出固定版本', () => { if (!this.destination.value.trim()) return this.notify('请填写导出目录。', true); void this.operation('/api/export', { destination: this.destination.value.trim() }); });

  constructor(private readonly root: HTMLElement, private readonly api: StudioApi, editorFactory: typeof createEditor = createEditor) {
    this.notice.setAttribute('role', 'status'); this.notice.setAttribute('aria-live', 'polite'); this.notice.hidden = true;
    this.auto.type = 'checkbox'; this.auto.addEventListener('change', () => { if (this.auto.checked && !this.model.autoPausedReason) this.model.enableAuto(); else this.model.autoApply = false; this.render(); });
    this.target.setAttribute('aria-label', '目标页面'); this.target.addEventListener('change', () => { this.model.select(this.target.value || null); this.render(); });
    this.backend.setAttribute('aria-label', '连接方式');
    for (const [id, name] of [['cdp', 'CDP'], ['electron-gadget', 'Electron Gadget'], ['webkit', 'WKWebView'], ['bootstrap', 'Bootstrap']]) this.backend.append(option(id, name));
    this.backend.addEventListener('change', () => this.renderEndpoint());
    this.endpoint.placeholder = '9222'; this.endpoint.value = '9222'; this.endpointLabel = field('回环端口', this.endpoint);
    this.appPath.placeholder = '/Applications/Example.app'; this.destination.placeholder = '选择一个新的输出目录'; this.importPath.placeholder = '现有 manifest.json 路径';
    this.dataMode.append(option('', '选择数据模式'), option('shared', '沿用原应用数据'), option('isolated', '使用独立数据目录'));
    const header = element('header', 'topbar');
    const brand = element('div', 'brand'); brand.append(element('span', 'brand-icon', '↗'), element('strong', '', 'Injectra'), element('span', 'studio-word', 'STUDIO'));
    header.append(brand, this.workspaceLabel, this.connectionLabel, button('刷新连接', () => { void this.refreshConnection(); }, 'quiet'));
    const sidebar = element('aside', 'sources'); sidebar.append(element('h2', '', '代码来源'), element('p', 'section-note', '增量可编辑，基线与引导保持只读。'), this.files);
    const editorColumn = element('section', 'editor-column');
    const fileHeader = element('div', 'file-header'); fileHeader.append(this.fileTitle, this.fileBadge, this.fileStatus);
    const editorHost = element('div', 'code-editor'); editorHost.setAttribute('aria-label', '增量代码编辑器');
    const toolbar = element('div', 'editor-toolbar'); const autoLabel = element('label', 'check-label'); autoLabel.append(this.auto, document.createTextNode('保存时应用'));
    toolbar.append(this.revisionLabel, autoLabel, this.rereadButton, this.saveButton, this.runButton);
    const emptyHint = element('p', 'preview-hint', '效果显示在真实 App 窗口中。Ctrl / ⌘ + S 保存，Ctrl / ⌘ + Enter 运行已保存版本。');
    editorColumn.append(fileHeader, editorHost, toolbar, emptyHint);
    this.editor = editorFactory(editorHost, content => this.edit(content), () => { void this.save(); }, () => { void this.run(); });
    const right = element('aside', 'target-panel'); right.append(element('h2', '', '运行目标'), field('连接方式', this.backend), this.endpointLabel, button('连接 / 发现页面', () => { void this.connect(); }), field('目标页面', this.target), this.targetDetails, this.reloadButton, this.startup);
    const deployment = element('details', 'deployment'); deployment.append(element('summary', '', '准备与导出'), field('原应用路径', this.appPath), field('数据模式', this.dataMode), this.prepareButton, this.devButton, this.restartButton, this.artifact, field('导出目录', this.destination), this.exportButton, field('导入现有增量', this.importPath), this.importButton); right.append(deployment);
    const body = element('main', 'workbench'); body.append(sidebar, editorColumn, right);
    const results = element('section', 'results'); const resultHeader = element('div', 'results-header'); resultHeader.append(element('h2', '', '执行记录'), element('span', 'muted', '保存与执行分别确认 · 结果未知时不自动重试'));
    results.append(resultHeader, this.resultList, this.logList);
    this.root.append(header, this.notice, body, results);
    this.render();
  }

  async start(): Promise<void> { await this.refresh(); this.subscribe(); }
  hasDirtyDrafts(): boolean { return [...this.drafts.values()].some(draft => draft.dirty); }
  private notify(message: string, error = false): void { this.notice.textContent = message; this.notice.className = error ? 'notice error' : 'notice'; this.notice.hidden = !message; }
  reportError(error: unknown): void {
    const message = error instanceof Error ? error.message : '操作失败。';
    if (error instanceof StudioError && error.certainty?.toLowerCase() === 'unknown') this.model.pauseAuto('执行结果未知，自动应用已暂停。请核对目标后手动运行。');
    this.notify(message, true); this.render();
  }
  private async refresh(): Promise<void> { this.accept(await this.api.request<StudioState>('/api/state')); }
  private accept(state: StudioState): void {
    const invalidated = this.model.accept(state).targetInvalidated;
    for (const [path, draft] of this.drafts) { const file = state.files.find(file => file.path === path); if (!file || draft.baseHash !== file.hash) draft.conflict = true; }
    if (this.manualRequest) { const run = state.runs.find(run => run.request_id === this.manualRequest); if (run && !['pending', 'running'].includes(run.status.toLowerCase())) { if (!isUnknown(run)) this.model.autoPausedReason = ''; this.manualRequest = null; } }
    if (invalidated) this.notify(this.model.autoPausedReason, true);
    this.render();
  }
  private subscribe(): void {
    this.stopEvents?.();
    this.stopEvents = this.api.events(event => this.event(event), connected => { this.connectionLabel.textContent = connected ? '本机已连接' : '事件通道已断开'; this.connectionLabel.dataset.connected = String(connected); if (!connected) { this.model.pauseAuto('连接已断开，保存时应用已暂停。刷新连接后手动运行。'); this.render(); } });
  }
  private async refreshConnection(): Promise<void> { try { await this.refresh(); this.subscribe(); } catch (error) { this.reportError(error); } }
  private event(event: StudioEvent): void {
    if (event.type === 'state') this.accept(event.state);
    else if (event.type === 'run' && this.model.state) { const runs = this.model.state.runs.filter(run => run.request_id !== event.run.request_id); this.accept({ ...this.model.state, runs: [...runs, event.run] }); }
    else if (event.type === 'log') { const previous = this.logList.textContent ?? ''; this.logList.textContent = (previous + format(event) + '\n').slice(-24000); }
  }
  private renderEndpoint(): void {
    const mode = this.backend.value;
    this.endpointLabel.hidden = mode === 'bootstrap';
    this.endpointLabel.firstElementChild!.textContent = mode === 'cdp' ? '回环端口' : '会话文件';
    this.endpoint.placeholder = mode === 'cdp' ? '9222' : '会话描述文件路径';
    this.endpoint.value = mode === 'cdp' ? '9222' : '';
  }
  private async connect(): Promise<void> {
    const backend = this.backend.value;
    const config: Record<string, unknown> = { backend };
    if (backend === 'cdp') { const port = Number(this.endpoint.value); if (!Number.isInteger(port) || port < 1 || port > 65535) return this.notify('请输入 1–65535 的回环端口。', true); config.port = port; }
    else if (backend !== 'bootstrap') { if (!this.endpoint.value.trim()) return this.notify('请选择会话描述文件。', true); config.session = this.endpoint.value.trim(); }
    await this.operation('/api/connect', config);
  }
  private async operation(path: string, body: unknown): Promise<void> {
    if (this.busy) return;
    this.busy = true; this.render(); this.notify('正在处理…');
    try { const result = await this.api.request<unknown>(path, body); await this.refresh(); this.notify(path === '/api/export' ? `导出完成：${format(result)}` : '操作完成。'); }
    catch (error) { this.reportError(error); }
    finally { this.busy = false; this.render(); }
  }
  private edit(content: string): void {
    if (!this.currentFile || !canEdit(this.currentFile)) return;
    const draft = this.drafts.get(this.currentFile.path); if (!draft) return;
    draft.content = content; draft.dirty = true; draft.editSequence++; this.renderToolbar();
  }
  private async openFile(file: SourceFile): Promise<void> {
    const sequence = ++this.loadSequence;
    this.currentFile = file; this.fileTitle.textContent = file.relative_path ?? file.path; this.fileTitle.title = file.path;
    this.fileBadge.textContent = file.origin === 'user' ? (file.stage === 'startup' ? '启动增量' : '页面增量') : file.origin === 'application' ? '原 App · 只读' : 'Injectra · 只读';
    const existing = this.drafts.get(file.path);
    if (existing) { this.editor.load(file.path, existing.content, canEdit(file)); this.render(); return; }
    this.editor.load(file.path, '正在读取…', false);
    try { const content = await this.api.request<FileContent>('/api/files?path=' + encodeURIComponent(file.path)); if (sequence !== this.loadSequence) return; this.currentFile = { ...file, ...content }; this.drafts.set(file.path, { path: file.path, content: content.content, baseHash: content.hash, dirty: false, conflict: false, editSequence: 0 }); this.editor.load(file.path, content.content, canEdit(content)); this.render(); }
    catch (error) { if (sequence === this.loadSequence) { this.editor.load(file.path, '文件读取失败。', false); this.reportError(error); } }
  }
  private openProvenance(origin: 'application' | 'injectra'): void {
    ++this.loadSequence; this.currentFile = null;
    this.fileTitle.textContent = origin === 'application' ? '原 App 基线' : 'Injectra 引导与修改清单'; this.fileBadge.textContent = '只读';
    this.editor.load('provenance.json', format(this.model.state?.provenance?.[origin] ?? (origin === 'injectra' ? this.model.state?.artifact : null) ?? '尚未准备应用。'), false); this.renderToolbar();
  }
  private async reread(): Promise<void> {
    const file = this.currentFile; if (!file || this.busy) return;
    const draft = this.drafts.get(file.path);
    if (draft?.dirty && !window.confirm('放弃此文件未保存的草稿，并重新读取工作目录中的版本？')) return;
    const sequence = ++this.loadSequence;
    this.busy = true; this.renderToolbar();
    try {
      const content = await this.api.request<FileContent>('/api/files?path=' + encodeURIComponent(file.path));
      if (sequence !== this.loadSequence) return;
      this.currentFile = { ...file, ...content };
      this.drafts.set(file.path, { path: file.path, content: content.content, baseHash: content.hash, dirty: false, conflict: false, editSequence: 0 });
      this.editor.load(file.path, content.content, canEdit(this.currentFile)); this.notify('已重新读取文件。');
    } catch (error) { this.reportError(error); }
    finally { this.busy = false; this.render(); }
  }
  private async save(): Promise<void> {
    const file = this.currentFile, draft = file && this.drafts.get(file.path);
    if (this.busy || !file || !draft || !canEdit(file) || !draft.dirty) return;
    if (draft.conflict) return this.notify('文件已在外部改变。草稿已保留，请核对后重新载入，避免覆盖其他修改。', true);
    this.busy = true; this.render(); const savedSequence = draft.editSequence;
    try {
      const saved = await this.api.request<{ revision?: string; hash?: string }>('/api/files', { path: file.path, content: draft.content, expected_hash: draft.baseHash }, 'PUT');
      const state = await this.api.request<StudioState>('/api/state');
      const hash = saved.hash ?? state.files.find(candidate => candidate.path === file.path)?.hash ?? draft.baseHash;
      this.drafts.set(file.path, acknowledgeSave(draft, savedSequence, hash)); this.accept(state);
      this.notify(file.stage === 'startup' ? '启动增量已保存，重启开发实例后生效。' : '已保存。运行后才能确认目标中的效果。');
      this.busy = false; this.render();
      if (this.model.autoApply && file.stage !== 'startup') await this.run(false, saved.revision ?? state.workspace.revision);
    } catch (error) { if (error instanceof StudioError && error.status === 409) draft.conflict = true; this.reportError(error); }
    finally { this.busy = false; this.render(); }
  }
  private async run(manual = true, revision = this.model.state?.workspace.revision): Promise<void> {
    const target = this.model.target();
    if (this.busy || !target || !revision || !target.capabilities.includes('web.page.evaluate') || this.targetRunning(target.id)) return;
    if (!manual && this.model.autoPausedReason) return;
    this.busy = true; this.render();
    try { const result = await this.api.request<Partial<Run>>('/api/run', { target: target.id, generation: target.generation, revision }); if (manual && result.request_id) this.manualRequest = result.request_id; await this.refresh(); this.notify('已提交已保存版本。请在执行记录中确认结果，并查看真实 App 窗口。'); }
    catch (error) { this.reportError(error); }
    finally { this.busy = false; this.render(); }
  }
  private targetRunning(id: string): boolean { return this.model.state?.runs.some(run => run.target === id && ['pending', 'running'].includes(run.status.toLowerCase())) ?? false; }
  private renderToolbar(): void {
    const file = this.currentFile, draft = file && this.drafts.get(file.path), target = this.model.target();
    this.fileStatus.textContent = draft?.conflict ? '外部修改冲突 · 草稿保留' : draft?.dirty ? '未保存' : file ? '已保存' : '';
    this.saveButton.disabled = this.busy || !file || !canEdit(file) || !draft?.dirty || !!draft.conflict;
    this.rereadButton.disabled = this.busy || !file;
    this.runButton.disabled = this.busy || !target?.capabilities.includes('web.page.evaluate') || !this.model.state?.workspace.revision || !!target && this.targetRunning(target.id);
    this.auto.checked = this.model.autoApply; this.auto.disabled = this.busy || !target?.capabilities.includes('web.page.evaluate') || !!this.model.autoPausedReason;
    this.auto.title = this.model.autoPausedReason || '仅应用已保存的页面增量；启动代码需要重启。';
    this.revisionLabel.textContent = '版本 ' + short(this.model.state?.workspace.revision);
  }
  private render(): void {
    const state = this.model.state; this.workspaceLabel.textContent = state?.workspace.root ?? '尚未打开工作区'; this.workspaceLabel.title = state?.workspace.root ?? '';
    this.files.replaceChildren();
    for (const [origin, title] of [['application', '原 App 基线'], ['injectra', 'Injectra 引导'], ['user', '用户增量']] as const) {
      const group = element('div', 'source-group'); group.append(element('h3', '', title));
      if (origin !== 'user') group.append(button('查看来源与修改记录', () => this.openProvenance(origin), 'source-overview'));
      const files = state?.files.filter(file => file.origin === origin) ?? [];
      for (const file of files) { const row = button(file.relative_path ?? file.path, () => { void this.openFile(file); }, 'file-row'); row.title = file.path; row.dataset.selected = String(this.currentFile?.path === file.path); row.dataset.origin = origin; group.append(row); }
      if (origin === 'user' && !files.length) group.append(element('p', 'empty-small', '没有增量文件。可导入现有 manifest。'));
      this.files.append(group);
    }
    this.target.replaceChildren(option('', '选择实际页面…'));
    for (const target of state?.targets ?? []) this.target.append(option(targetIdentity(target), `${target.title || '未命名页面'} — ${target.url}`));
    this.target.value = this.model.selectedTarget ?? '';
    const selected = this.model.target(); this.targetDetails.textContent = selected ? `${selected.url}\n页面世代 ${short(selected.generation)}` : '请选择目标页面。连接成功不代表代码已经执行。';
    this.reloadButton.disabled = this.busy || !selected?.capabilities.includes('web.page.reload'); this.reloadButton.title = selected?.capabilities.includes('web.page.reload') ? '' : '当前后端未声明页面重载能力';
    this.startup.replaceChildren(element('strong', '', '启动增量'), element('p', '', state?.startup?.pending_restart ? '已保存 · 等待重启开发实例生效' : statusLabel(state?.startup?.status ?? '未配置')), element('small', 'muted', `已保存 ${short(state?.startup?.saved_revision)} / 已应用 ${short(state?.startup?.applied_revision)}`));
    for (const [control, capability] of [[this.prepareButton, 'prepare'], [this.devButton, 'dev'], [this.restartButton, 'restart'], [this.exportButton, 'export'], [this.importButton, 'workspace.import']] as const) { control.disabled = this.busy || !this.model.has(capability); control.title = this.model.has(capability) ? '' : '当前工作区暂不支持此操作'; }
    this.artifact.textContent = state?.artifact ? format(state.artifact) : '尚未准备开发产物';
    this.resultList.replaceChildren();
    if (!state?.runs.length) this.resultList.append(element('p', 'empty-small', '尚无执行记录。保存文件后，选择目标并运行。'));
    for (const run of [...(state?.runs ?? [])].reverse().slice(0, 100)) {
      const card = element('article', 'run-card'); const meta = element('div', 'run-meta'); const status = element('span', 'run-status', statusLabel(run.status)); status.dataset.status = isUnknown(run) ? 'unknown' : run.status;
      meta.append(status, element('code', '', short(run.revision)), element('span', 'muted', run.target), element('code', 'muted', run.request_id)); card.append(meta);
      if (run.error !== undefined || run.result !== undefined) card.append(element('pre', '', format(run.error ?? run.result)));
      const error = run.error && typeof run.error === 'object' ? run.error as Record<string, unknown> : null;
      if (error && typeof error.path === 'string' && typeof error.line === 'number') { const file = state?.files.find(file => file.path === error.path); if (file) card.append(button(`定位 ${file.path}:${error.line}`, () => { void this.openFile(file).then(() => this.editor.focusLine(error.line as number)); }, 'quiet')); }
      this.resultList.append(card);
    }
    if (this.model.autoPausedReason) this.notify(this.model.autoPausedReason, true);
    this.renderToolbar();
  }
}
