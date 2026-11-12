import type { ApiFailure, StudioEvent } from './types';

export class StudioError extends Error {
  constructor(message: string, public readonly status: number, public readonly certainty?: string) { super(message); }
}
export function startupFailure(error: unknown, token: string): Error {
  let message = error instanceof Error ? error.message : '无法连接本地工作台服务。';
  if (token) for (const secret of new Set([token, encodeURIComponent(token)])) message = message.split(secret).join('[redacted]');
  return error instanceof StudioError ? new StudioError(message, error.status, error.certainty) : new Error(message);
}
export function takeToken(location: Pick<Location, 'hash' | 'pathname' | 'search'>, history: Pick<History, 'replaceState'>): string {
  const token = new URLSearchParams(location.hash.replace(/^#/, '')).get('token') ?? '';
  if (location.hash) history.replaceState(null, '', location.pathname + location.search);
  return token;
}
export class StudioApi {
  constructor(private readonly token: string, private readonly origin: string, private readonly fetcher: typeof fetch = fetch) {}
  async request<T>(path: string, body?: unknown, method = body === undefined ? 'GET' : 'POST'): Promise<T> {
    const url = new URL(path, this.origin);
    if (url.origin !== this.origin || !url.pathname.startsWith('/api/')) throw new Error('Only same-origin Studio API requests are allowed');
    // WebKit's native fetch requires its Window/global receiver, not this client.
    const response = await this.fetcher.call(globalThis, url.href, { method, credentials: 'same-origin', cache: 'no-store', headers: { Authorization: `Bearer ${this.token}`, ...(body === undefined ? {} : { 'Content-Type': 'application/json' }) }, ...(body === undefined ? {} : { body: JSON.stringify(body) }) });
    const content = await response.text();
    let data: unknown;
    try { data = content ? JSON.parse(content) : {}; } catch { throw new StudioError('服务返回了无法识别的响应。', response.status); }
    if (!response.ok) { const failure = data as ApiFailure; throw new StudioError(failure.message ?? failure.error ?? `请求失败 (${response.status})`, response.status, failure.certainty); }
    return data as T;
  }
  events(onEvent: (event: StudioEvent) => void, onStatus: (connected: boolean) => void): () => void {
    const url = new URL('/api/events', this.origin); url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
    const socket = new WebSocket(url.href);
    let authenticated = false;
    socket.addEventListener('open', () => { socket.send(JSON.stringify({ type: 'authenticate', token: this.token })); });
    socket.addEventListener('message', event => {
      try { const data = JSON.parse(String(event.data)); if (data.type === 'authenticated') { authenticated = true; onStatus(true); } else if (authenticated && ['state', 'run', 'log'].includes(data.type)) onEvent(data); } catch { /* Invalid events never trigger execution. */ }
    });
    socket.addEventListener('close', () => onStatus(false));
    socket.addEventListener('error', () => onStatus(false));
    return () => socket.close();
  }
}
