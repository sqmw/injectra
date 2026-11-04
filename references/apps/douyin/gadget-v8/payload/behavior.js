const panel = context.root.querySelector('#injectra-douyin-proof');
const button = panel.querySelector('button');
let count = 0;
panel.dataset.executed = '1';
const click = () => { button.dataset.count = String(++count); button.textContent = `验证 JS：${count}`; };
button.addEventListener('click', click);
context.onCleanup(() => button.removeEventListener('click', click));
await Promise.resolve();
