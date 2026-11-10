# macOS多引擎交付

所属[T-017](TODO.md#t-017)；[执行证据](STEPS.md#t-017)。当前仅macOS arm64，主页面，不外推Intel/Windows/Gecko。

## 已批准实现边界

保留core/web/application/CDP；新增共享Frida通信、Electron Gadget和WKWebView并列后端。Rust CLI统一入口；可选Python/Frida环境锁定在仓库外。现有命令默认CDP并保持输出；新后端显式选择。不从references加载正式运行代码。

## 验收矩阵

| 引擎/宿主 | 路线 | 本轮状态 |
|---|---|---|
| Chromium浏览器/Electron/定制宿主 | CDP | 基线Rust/Web与Electron45的10项通过；历史应用见兼容性总表 |
| Electron/抖音 | Gadget→原Node→页面 | 正式后端及抖音/独立Electron36实测通过；45缺符号拒绝，详见Frida后端 |
| CEF/NW.js | CDP | CEF156.0.26与NW.js0.119.2独立宿主通过，原第三方应用记录不回填 |
| WKWebView | Frida→主线程原生页面API | 独立双WKWebView及真实Codex Switcher0.2.11、CLI检查通过 |

各后端统一验证目标隔离、JS/CSS/HTML、点击读回、Promise/异常、更新/清理、断线/超时Unknown不重放和资源释放。测试宿主、真实应用与历史run分别记证据。

## 操作边界

不自动重启用户应用，不修改安装包、重签名或调整系统保护。Clash Verge仅先做只读可行性检查；改变其运行状态须讨论。独立测试宿主由测试创建和退出。不能以fixture通过替代真实应用验收。
