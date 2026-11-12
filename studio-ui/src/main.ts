import './style.css';
import { StudioApi, StudioError, takeToken } from './api';
import { StudioApp } from './studio';

const root = document.getElementById('studio')!;
const token = takeToken(window.location, window.history);
if (!token) {
  const page = document.createElement('main'); page.className = 'auth-empty';
  const title = document.createElement('h1'); title.textContent = '打开你的开发工作台';
  const text = document.createElement('p'); text.textContent = '请使用 injectra studio 输出的本机链接。访问凭据只保存在当前页面内存中。';
  page.append(title, text); root.append(page);
} else {
  const app = new StudioApp(root, new StudioApi(token, window.location.origin));
  app.start().catch(error => app.reportError(error instanceof StudioError ? error : new Error('无法连接本地工作台服务。')));
  window.addEventListener('beforeunload', event => { if (app.hasDirtyDrafts()) { event.preventDefault(); event.returnValue = ''; } });
}
