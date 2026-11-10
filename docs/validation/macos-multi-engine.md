# T-017 macOS多引擎验收与审阅

[任务](../TODO.md#t-017) · [当前矩阵](../compatibility/current-matrix.md) · [交付映射](../releases.md)。所有本轮结果对应当前窗口Mac工作树，基线62c1676；不代表main已集成，不改写历史MVP/Probe结果。

## 验收对象

| 层级/路线 | 已取得的证据 | 范围 |
|---|---|---|
| 核心/默认CDP | make check；真实Electron45.2.1/Chromium156的10项回归 | 原CLI参数/JSON保持；主页面 |
| Electron Gadget | 抖音8.7.0完整合同及CLI；独立Electron36.4.0合同/CLI/控制端退出清理 | 具体V8导出可用；不外推Electron45 |
| CEF/NW.js | cef-qdvrcqtq、nwjs-ypuq07xm官方独立宿主完整合同 | Mac arm64；不外推未验第三方应用 |
| WKWebView | webkit-i07adzl2、webkit-lg0f0zoc双页面合同及CLI/控制端退出清理 | 主frame/page content world |
| 真实Tauri/WKWebView | Codex Switcher0.2.11；t017/webkit-real-contract.log和webkit-real-result.json | 用户明确批准的空HOME/CODEX_HOME；主页面及托盘隔离；未执行账号切换 |
| 会话失败合同 | 6项Python测试及Rust错误映射测试 | 文件权限/符号链接、进程世代、端点、SDK截止、Unknown不重放 |

最终检查摘要保存于外部缓存t017/final-checks.json；原始stdout、失败run和profile继续在仓库外保存。临时测试仅创建自己的DOM，程序触发自有按钮事件；没有声称新增真实鼠标或截图验收。

## 自审及处理

- 核对core/native/application/Web契约和唯一Web runtime无结构性改动；三条Web后端并列，平台/宿主代码未进入core。未委派独立审核，不把自审当作独立审计。
- CLI参数缺省原先未正确触发必选项，已修正并增加回归，旧缺端口仍退出2。
- 结果读取I/O失败曾沿用准备期NotExecuted，已修正为Unknown；未知响应/过大响应同样不重放。
- 会话路径曾提前canonicalize导致符号链接检查丢失，已改为绝对化保留符号链接事实，由私有文件检查拒绝。
- WK completion参数nil在ObjC bridge中是null，已修正；首次失败证据保留。
- CEF/NW.js循环对象返回协议错误，新增测试不能擅自要求它一定Executed；按既有合同保留Protocol/Unknown，未改变旧产品错误语义。
- 原生回调在主线程排队和completion期间pin，确认关闭后卸载；CLI被测试终止时助手继续完成。SDK调用/清理设独立可取消截止，无法确认回调关闭时报告Unknown而不强制卸载。
- 下载来源、版本、校验值锁定，源码不含二进制/token/profile；可选依赖与官方宿主放外部缓存。第一方helper随Rust二进制嵌入，不依赖references或编译机绝对源码路径。
- 真实应用测试仅新建授权实例，进程退出前核对路径与启动时间。Easydict接入失败保留，Clash未作运行操作，没有重签名、修改安装包或更改系统保护。

## 剩余边界

本阶段不是所有Mac应用的兼容承诺：Gadget需要目标加载条件和具体ABI；Electron45当前缺所需导出；Easydict本次未取得完整接入。WebKit只枚举挂入NSApplication窗口树的WKWebView，未覆盖Safari专用进程体系或任意隐藏WebView。

同URL导航世代锁定、iframe/worker、自动重注入、通用原生内存接口、Windows/Gecko/Intel Mac均不在本轮验收。目标主线程永久阻塞或助手自身崩溃时不承诺自动安全回收；报告Unknown并保留诊断，不自动强杀用户进程。
