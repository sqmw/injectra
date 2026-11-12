import type { StudioState, Target, SourceFile, Run } from './types';

export function targetIdentity(target: Target): string { return JSON.stringify([target.id, target.generation]); }
export function canEdit(file: SourceFile): boolean { return file.origin === 'user'; }
export function isInFlight(status: string): boolean { return ['pending', 'queued', 'running'].includes(status.toLowerCase()); }
export function isUnknown(run: Pick<Run, 'status' | 'certainty' | 'error'>): boolean {
  const error = run.error && typeof run.error === 'object' ? run.error as Record<string, unknown> : {};
  return run.status.toLowerCase() === 'unknown' || run.certainty?.toLowerCase() === 'unknown' || String(error.certainty ?? '').toLowerCase() === 'unknown';
}
export function statusLabel(status: string): string {
  return ({ saved: '已保存', pending: '待执行', queued: '排队中', running: '执行中', succeeded: '已执行', success: '已执行', executed: '已执行', failed: '失败', error: '失败', unknown: '结果未知', pending_restart: '等待重启', applied: '已生效', not_executed: '未执行', disconnected: '已断开' } as Record<string, string>)[status.toLowerCase()] ?? status;
}

export class StudioModel {
  state: StudioState | null = null;
  selectedTarget: string | null = null;
  autoApply = false;
  autoPausedReason = '';
  private unknownSeen = new Set<string>();
  target(): Target | undefined { return this.state?.targets.find(t => targetIdentity(t) === this.selectedTarget); }
  accept(state: StudioState): { targetInvalidated: boolean } {
    this.state = state;
    const targetInvalidated = this.selectedTarget !== null && !this.target();
    if (targetInvalidated) { this.selectedTarget = null; this.pauseAuto('目标已改变或关闭，请重新选择页面。'); }
    for (const run of state.runs) {
      if (isUnknown(run) && !this.unknownSeen.has(run.request_id)) { this.unknownSeen.add(run.request_id); this.pauseAuto('上次执行结果未知，已暂停保存时应用。请核对目标后手动运行。'); }
    }
    return { targetInvalidated };
  }
  select(identity: string | null): void { this.selectedTarget = identity; }
  pauseAuto(reason: string): void { this.autoApply = false; this.autoPausedReason = reason; }
  enableAuto(): void { this.autoApply = true; this.autoPausedReason = ''; }
  has(capability: string): boolean { return this.state?.capabilities?.includes(capability) ?? false; }
}

export interface Draft { path: string; content: string; baseHash: string; dirty: boolean; conflict: boolean; editSequence: number }
export function acknowledgeSave(draft: Draft, savedSequence: number, savedHash: string): Draft {
  return { ...draft, baseHash: savedHash, dirty: draft.editSequence !== savedSequence, conflict: false };
}
