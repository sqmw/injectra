# M-003 商业宿主正式 CLI 实证回测报告（T-088）

2026-11-27，[T-088@v1](../TODO.md#t-088) / [执行归档](../archive/T-088.md#t-088)，承接 [M-003 规划](../plans/milestone-3-empirical-closure.md) 与 [D-016](../DECISIONS.md#d-016)。

本轮在 **macOS（Apple Silicon）** 与 **Lenovo Windows（`DESKTOP-F4B6E6C`，`win32 x64`，`rustc 1.96.1 MSVC`，`node v24.18.0`，`frida 17.25.2`）** 上，全程仅使用 `injectra` / `injectra.exe` 正式 CLI 子命令（零 `tests/windows/*` 实验脚本、零跨机测试控制端），在仓库外受控隔离目录或显式隔离会话中完成四类代表商业宿主机制边界的实机闭环回测。历史任务（`T-014`、`T-043`、`T-044`）原证据保持只读不回填；运行中的用户日常实例（macOS `Cursor.app`、Windows `aDrive` 与 `WorkBuddy`）全程未触碰。

## 四类机制边界实测结果

### 1. 严格 Trusted Types / CSP 商业宿主（`html_mode="dom"` 与 `PolicyBlocked` 分离）

此前在 [Windows 首轮复验](windows-apps.md) 中，`Devin` 与 `Quark` 的页面均因 Trusted Types 拦截 `innerHTML` 字符串赋值。本轮在双端受控独立 `--user-data-dir` 实例上全部仅用 `injectra --machine` 正式子命令完成对照复验：

| 宿主与平台 | 选定目标页面 | 默认 `inner-html` 对照 | `html_mode="dom"` 全链路 | 仓库外运行证据 |
| --- | --- | --- | --- | --- |
| macOS `Devin.app`（Electron `42.2.0`） | `vscode-file://vscode-app/.../workbench/workbench.html` | 退出码 `4`，`kind: "PolicyBlocked"`，`certainty: "Executed"`，`list=[]`，`nodes=0` | `apply` v1/v2、`list`（`nodes=2, cleanups=1`）、`eval`（CSS `rgb(12, 148, 136)`、按钮点击 `0→1`、SVG `http://www.w3.org/2000/svg`）、`remove`（`nodes=0, cleanups=2`）均退出码 `0` | `t088-runs/macos-devin-1795718507/result.json` |
| Windows `Devin`（`D:\exe\Devin\Devin.exe`） | `vscode-file://vscode-app/d:/exe/Devin/resources/app/.../workbench.html` | 退出码 `4`，`kind: "PolicyBlocked"`，`certainty: "Executed"`，`list=[]`，`nodes=0` | 同上全链路退出码 `0`（CSS、按钮 `0→1`、SVG 命名空间、v2 更新 `count=1`、清理 `nodes=0, cleanups=2`） | `t088-runs/tt-20261127-024224/result.json`（`Devin`） |
| Windows `Quark`（`D:\exe\Quark\quark.exe`） | `chrome://new-tab-page/`（`新标签页`） | 退出码 `4`，`kind: "PolicyBlocked"`，`certainty: "Executed"`，`list=[]`，`nodes=0` | 同上全链路退出码 `0`（CSS、按钮 `0→1`、SVG 命名空间、v2 更新 `count=1`、清理 `nodes=0, cleanups=2`） | `t088-runs/tt-20261127-024224/result.json`（`Quark`） |

### 2. Electron ESM 主入口 + Windows PE `INTEGRITY`（原生 `prepare` → `dev`）

| 宿主 | 正式 CLI 路线 | 关键断言与实测结果 | 仓库外运行证据 |
| --- | --- | --- | --- |
| Windows `Folo`（`D:\exe\Folo\app-1.17.0\Folo.exe`，`pe-x64`、`pe-asar-integrity`、`app.asar`、`esm`） | `injectra.exe --machine prepare --data-mode isolated` → `injectra.exe dev` | 准备返回退出码 `0`、`entry_strategy: "esm-delegate"`、PE `INTEGRITY` 资源更新且原 `Folo.exe` 与 `app.asar` SHA-256 零变动；使用带 UTF-8 BOM 的 `manifest.json` + `html_mode="dom"` 驱动派生实例（`Folo - AI Reader | Follow Everything`）完成 `v1`（`applied`、`INJECTRA_T088_folo_v1_cleanups=0`）与 `v2` 免重打包热更新（`updated`、`INJECTRA_T088_folo_v2_cleanups=1`） | `t088-runs/formal-2026-11-26T18-50-50-176Z/result.json`（`foloPrepareDev`） |

### 3. 无预留调试端口的 Windows GUI Electron（原生 `session attach-v8 --bootstrap-output` → `dev`）

| 宿主 | 正式 CLI 路线 | 关键断言与实测结果 | 仓库外运行证据 |
| --- | --- | --- | --- |
| Windows `QQ`（`D:\exe\QQ\QQ.exe`，主进程加载 `versions\9.9.37-54395\QQNT.dll`） | `injectra.exe --machine session attach-v8 --pid <PID> --exe D:\exe\QQ\QQ.exe --module <QQNT.dll全路径> --bootstrap-output <DIR>` → `injectra.exe dev` | `attach-v8` 返回退出码 `0`、`entry_strategy: "windows-v8-attach"`、`hooks: 3`、`certainty: "Executed"`；随后 `dev` 直接绑定该已附加进程与受管页面，使用 UTF-8 BOM `manifest.json` + `html_mode="dom"` 完成 `v1`（`applied`、`INJECTRA_T088_qq_v1_cleanups=0`）与 `v2` 热更新（`updated`、`INJECTRA_T088_qq_v2_cleanups=1`） | `t088-runs/formal-2026-11-26T18-50-50-176Z/result.json`（`qqAttachV8Dev`） |

### 4. 多版本目录、`ia32` 与不支持目标只读诊断边界（`inspect-app` / `launch-plan`）

| 目标路径 | 识别结果与退出码 | 边界说明 |
| --- | --- | --- |
| Windows `D:\exe\Quark` | `inspect-app` / `launch-plan` 退出码 `0`，选定最新 `7.5.4.1068\quark.exe`（`pe-x64`、`app.asar`、`commonjs`） | 本轮修复多版本目录排序与辅助 exe 过滤后不再误选旧版 `7.5.2.1065\apollocmd.exe` |
| Windows `D:\exe\Douyin` | `inspect-app` / `launch-plan` 退出码 `0`，选定根 `douyin.exe`（`pe-x86`、`app.asar`、`commonjs`） | 准确识别 32 位 `ia32` PE 且不再误选 `8.8.0\app_shell_updater.exe` |
| Windows `D:\exe\QQ` | `inspect-app` 退出码 `0`，`prepare-dev: false`（`layout: "missing"`），`windows-v8: true` | 准确引导调用方走 `session attach-v8` 路线而非强做 `prepare` |
| Windows `C:\Windows\System32\notepad.exe` | `inspect-app` / `launch-plan` 退出码 `3`，`kind: "Unsupported"`，`certainty: "NotExecuted"` | 原生无 Web UI 宿主确定性拒绝 |
| macOS `/Applications/Quark.app` | `inspect-app` 退出码 `3`，`kind: "Unsupported"`，`certainty: "NotExecuted"` | 包内使用自定义 `Quark Framework.framework` 而非标准 Electron/CEF/NW.js/WKWebView 结构，保持明确 `Unsupported` 不做品牌特判 |

## 回测暴露并收敛的通用缺陷（零品牌硬编码）

1. **Windows 多版本目录与主可执行文件选择（`crates/platform-windows/src/lib.rs`）**：原有逻辑按字典序选择版本目录与 `.exe`，导致多版本目录先命中旧版本且把 `apollocmd.exe`、`app_shell_updater.exe` 等辅助程序排在主程序前。现改为按自然数字版本段降序排列版本目录，过滤通用 Windows 辅助/更新/崩溃/代理 `.exe`，并按“与根目录同名优先 → 最新版本目录优先 → 较大可执行文件优先”排序，补充两项单元测试。
2. **Windows UTF-8 BOM `manifest.json` 兼容（`crates/web/src/bundle.rs`、`crates/application/src/dev/mod.rs`）**：在 Windows PowerShell 5.1 或记事本保存的 UTF-8 BOM 文件会触发 `serde_json` 报错 `expected value at line 1 column 1`。现于 `Bundle::load` 与 `Revision::load` 统一剥离可选前导 UTF-8 BOM（`\u{feff}` / `EF BB BF`），补充两项单元测试并在真实 `Folo`/`QQ` `dev` 回测中实测通过。

## 约束与验证范围

- `session attach-v8 --module` 保持按已加载模块完整绝对路径匹配（如 `D:\exe\QQ\versions\9.9.37-54395\QQNT.dll`），防止同名 DLL 欺骗；传裸文件名时确定性返回退出码 `3`（`Unsupported` / `NotExecuted`）。
- 所有测试产生的临时进程在脚本 `finally` 阶段均按精确 `ExecutablePath` / PID 清理，双端复核无残留进程，原始安装包摘要无改动。
- 上述实测结果仅覆盖记录中的版本、受控页面与受控载荷，不外推为未测版本或任意业务页面兼容。
