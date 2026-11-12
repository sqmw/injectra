export type Origin = 'application' | 'injectra' | 'user';
export interface SourceFile { path: string; relative_path?: string; origin: Origin; stage: string; hash: string }
export interface Target { id: string; title: string; url: string; generation: string; capabilities: string[] }
export interface Run { request_id: string; target: string; revision: string; status: string; result?: unknown; error?: unknown; certainty?: string }
export interface StudioState {
  workspace: { id: string; root: string; revision: string };
  files: SourceFile[];
  targets: Target[];
  runs: Run[];
  capabilities?: string[];
  artifact?: unknown;
  provenance?: { application?: unknown; injectra?: unknown; user?: unknown };
  startup?: { status: string; saved_revision?: string; applied_revision?: string; pending_restart?: boolean };
}
export interface FileContent extends SourceFile { content: string }
export interface ApiFailure { error?: string; message?: string; certainty?: string }
export type StudioEvent = { type: 'state'; state: StudioState } | { type: 'run'; run: Run } | { type: 'log'; [key: string]: unknown };
