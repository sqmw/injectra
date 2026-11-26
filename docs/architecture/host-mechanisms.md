# Web 桌面架构接入机制与控制层基线

本文件为 [M-002 阶段 3](../plans/milestone-2-core-engine.md) 与 [T-085](../TODO.md#t-085) 的跨平台架构调研基线，记录 macOS 与 Windows 下主流 Web 桌面宿主在运行期（runtime）和运行前（pre-runtime）两类控制面上的可核验机制、前置条件、核心 crate 映射及明确的 `Unsupported` 边界。

## 核心判据：控制层而非磁盘 JS 文件

判断一款 Web 桌面应用是否具备通用注入入口，依据两条独立维度：

1. **运行期协议／进程控制面（Runtime）**：宿主进程是否暴露受控页面调试协议（如回环 CDP），或允许在保持安全边界的前提下通过受控运行时桥接访问页面求值接口（如 macOS `FridaGadget` → V8/WKWebView，或 Windows MSVC V8 事件泵派发 → Bootstrap）。
2. **运行前磁盘控制层（Pre-runtime）**：在宿主启动到创建 `WebContents` / `WebView` 之间，是否存在宿主按公开模块契约加载的磁盘入口（如 Electron `package.json` 的 `main`），且派生副本在保持入口语义、归档完整性与平台签名策略后仍能正常启动。磁盘上存在静态前端 `.js` 文件本身不构成可执行控制层。

## 主流架构跨平台对比

| 架构 | macOS 结构与控制面 | Windows 结构与控制面 | 正式主路线 | Pre-runtime 边界 |
| --- | --- | --- | --- | --- |
| **Electron** | `.app/Contents/Frameworks/Electron Framework.framework`（`com.github.Electron.framework`），资源位于 `Contents/Resources/app[.asar]`；`Info.plist` 存 `ElectronAsarIntegrity`，41+ 框架二进制含 34 字节 Fuse 摘要槽；`CodeResources` 封存资源 | 主目录或版本子目录（如 `app-<ver>/`、`versions/<ver>/`）含主 `.exe`/宿主 `.dll` 与 `resources/app[.asar]`；Electron Fuse 标记与 PE 资源 `INTEGRITY` / `ELECTRONASAR` 校验 ASAR 头摘要；可能含包内 Junction | Runtime：`cdp`；macOS 可选 `electron-gadget`（需导出 `_ZN2v811HandleScopeC1EPNS_7IsolateE` 且签名允许）；Windows 可选 `windows-v8` 附加派发至 Bootstrap。<br>Pre-runtime：`prepare` / `dev` → `BootstrapBackend` | 支持目录、`app.asar`、目录内嵌套 `.asar` 及 CommonJS（`cjs-insert`、`main-delegate`）与 ESM（`esm-delegate`）。拒绝 `.jsc` 字节码、自定义加壳与无法保持签名/容器权限的原包 |
| **Tauri（v1 / v2）与系统 WebView** | 链接系统 `WebKit.framework`（`WKWebView`）；生产包通常将前端压缩编译进原生 Mach-O 只读段 | 携带 `WebView2Loader.dll`（Edge WebView2）；生产包通常将前端压缩编译进原生 PE 只读段 | Windows：`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` 开启回环 `cdp`。<br>macOS：`webkit` Gadget 路由（要求 `codesign` 允许动态库加载） | **不支持 pre-runtime 改包**：不修改原生二进制只读数据段或提取重打包自定义协议资源 |
| **CEF** | `Contents/Frameworks/Chromium Embedded Framework.framework`（`org.cef.framework`） | 携带 `libcef.dll`（可能位于版本子目录或独立子目录） | Runtime：标准回环 `--remote-debugging-port` 与 `--remote-debugging-address=127.0.0.1` → `cdp` | 宿主由原生 C/C++ 控制页面创建；停留在原生登录/欢迎窗（如 `DuiHostWnd`、`WeChatLogin`）且未初始化 CEF 页面时目标列表为空，不伪造 Web 目标；结构体偏移 Hook（`tests/windows/cef-init.js`）因强耦合版本布局保留在实验工具层 |
| **NW.js** | `Contents/Frameworks/nwjs Framework.framework`（`io.nwjs.nwjs.framework`） | 携带 `nw.dll` | Runtime：标准回环 `cdp`（已在 NW.js `0.119.2` 只读诊断与规划核验） | 当前不建专用 NW.js pre-runtime 包装，优先沿用标准 CDP |
| **Gecko（Firefox / Thunderbird）** | 协议驱动（BiDi / Marionette） | 协议驱动（BiDi / Marionette） | 当前由独立测试客户端验证，尚未纳入正式 CLI 后端 | 不改写浏览器内置资源包 |

## Electron 跨平台准备与完整性细则

### 1. 入口策略（无应用品牌分支）

- **`cjs-insert`**：明文 UTF-8 CommonJS 主入口，使用 Acorn 解析并保留 `#!` 与顶层指令（如 `'use strict'`），在其后插入 `module.require('./.injectra/bootstrap.cjs')`，保持原 `require.main === module`、`process.mainModule` 与相对模块解析。
- **`main-delegate`**：目录内嵌套 `.asar`（如 `application.asar/app_launcher/index.js`）或不透明 CommonJS 入口，原归档字节逐字节保留；改写外层 `package.json` 的 `main` 为 `./.injectra/entry.cjs`，由其先加载 `./bootstrap.cjs`，再以 `Module._load(filename, null, true)` 恢复原主模块身份与 `process.argv[1]`。
- **`esm-delegate`**：当 `package.json` 声明 `"type": "module"`（且入口非 `.cjs`）或主入口后缀为 `.mjs` 时，用 Acorn 按 `sourceType: "module"` 校验语法后保留原入口字节，改写 `package.json` 的 `main` 为 `./.injectra/entry.mjs`。该委托模块通过 `createRequire(import.meta.url)('./bootstrap.cjs')` 同步完成 Bootstrap 注册（含 `preflight` 提前退出，不执行原入口业务），再通过相对 ESM 路径 `await import('../<encoded_main>')` 加载原 ESM 入口，保留原模块的 `import.meta.url`、相对 `import` 与顶层 `await`。

### 2. 平台完整性与隔离差异

| 维度 | macOS `.app` | Windows 目录 / `.exe` |
| --- | --- | --- |
| 派生目录命名 | `<output>/<uuid>/Prepared.app` | `<output>/<uuid>/PreparedApp` |
| 复制过滤与链接安全 | 按 `CodeResources` 封存清单复制，排除未封存运行数据；拒绝逃出包的符号链接 | `copyTree` 预检全部符号链接与 Junction（处理尾部分隔符），拒绝外部/循环链接，将包内链接重建指向派生副本；跳过未包含包路径的顶层运行日志/缓存目录 |
| ASAR 改写策略 | 保留 `unpacked`/`link`/`executable` 标志并逐项校验摘要 | `appendRewrite` 保留原载荷字节与不透明/虚拟/缺失 unpacked 元数据，按 `4 MiB` 块计算 SHA-256 `integrity` 并追加新条目 |
| 完整性摘要更新 | 更新 `Info.plist` 的 `ElectronAsarIntegrity`；命中 `AGbevlPCksUGKNL8TSn7wGmJEuJsXb2A` 时在核对原基线后更新 34 字节摘要槽 | 扫描宿主目录 `.exe`/`.dll` 的 PE 资源 `INTEGRITY`/`ELECTRONASAR`，仅在 `sha256` 原基线匹配时更新对应 ASAR 头摘要 |
| 代码签名影响 | 本地 ad-hoc `codesign` 并附 `com.apple.security.cs.disable-library-validation`；`isolated` 模式剥离团队/钥匙串权限组；沙箱应用需显式 `--data-mode shared` | 修改 PE 资源会使派生副本上的原 Authenticode 签名失效；原安装目录保持只读不变，派生前后复核原关键文件 SHA-256 |
| 预检与隔离运行 | `HOME` / `CFFIXED_USER_HOME` / `--user-data-dir` / `--use-mock-keychain` | `USERPROFILE` / `APPDATA` / `LOCALAPPDATA` / `--user-data-dir` / `--no-first-run`；通过 `INJECTRA_PREFLIGHT_FILE` 兼容无控制台的 Windows GUI 子系统 `.exe` 预检回执 |

### 3. Windows 运行期 MSVC V8 附加（`windows-v8`）

针对忽略 CDP 开关且不便改包的已运行 Windows Electron / Node 进程（仅限显式指定 PID、镜像路径与进程创建世代）：

1. 先通过 Win32 `OpenProcess`、`GetProcessTimes`、`QueryFullProcessImageNameW` 核验 PID 对应的规范化可执行路径与创建时间戳，防止 PID 复用误附加。
2. 在显式目标模块（主 `.exe` 或宿主 `.dll`）上核对完整 MSVC C++ 导出符号（`x64` `win64` 或 `ia32` `thiscall`/`mscdecl` 下的 `v8::Isolate::GetCurrent`、`GetCurrentContext`、`HandleScope`、`String::NewFromUtf8`、`Script::Compile`、`Script::Run`）。
3. 挂钩 Win32 事件泵（`GetQueuedCompletionStatusEx`、`GetQueuedCompletionStatus`、`PeekMessageW`）的返回点，在目标 I/O 线程上执行单次受控派发（例如加载独立物化的 `.injectra/bootstrap.cjs`），随后立即卸载全部 Hook；仅当卸载确认零尝试时判定 `NotExecuted`，明确返回时判定 `Executed`，其余不确定状态保守判定 `Unknown` 且不自动重放。

## 核心 Crate 抽象映射

| 职责 | 正式 Crate | 依赖边界 |
| --- | --- | --- |
| macOS `.app` 元数据、Framework、`otool`、`codesign` | [platform-macos](../../crates/platform-macos/src/lib.rs) | 仅依赖 `injectra-core` |
| Windows 目录/版本子目录布局、PE 架构与 `INTEGRITY`/`ELECTRONASAR` 只读解析 | [platform-windows](../../crates/platform-windows/src/lib.rs) | 仅依赖 `injectra-core` |
| 跨宿主只读识别（`inspect-app`）、纯 Rust ASAR/入口探测与 `launch-plan` | [hosts](../../crates/hosts/src/inspect.rs) | 依赖 `core`、`platform-macos`、`platform-windows` |
| 跨平台 Electron 派生副本准备、ESM/CJS 入口处理、预检与派生实例启动/重连 | [preparation](../../crates/preparation/src/lib.rs) | 依赖 `core`、`bootstrap` |
| 可选 Frida 会话管理（macOS Gadget `launch/register/verify` 与 Windows `attach-v8`） | [frida](../../crates/frida/src/lib.rs) | 仅依赖 `core` |
| 认证回环 Bootstrap 桥接与共享页面后端 | [bootstrap](../../crates/bootstrap/src/lib.rs) | 依赖 `core`、`web` |

## 明确 `Unsupported` 边界

以下场景不纳入通用引擎支持范围，遇到时必须显式返回 `ErrorKind::Unsupported`：

- V8 预编译字节码入口（`.jsc` / bytenode）或自定义加密/混淆打包器；
- 原生可执行文件只读段（`.rodata` / Mach-O / PE 嵌入资源）二进制补丁；
- 绕过操作系统保护（SIP、Hardened Runtime 强制校验、沙箱容器/钥匙串权限绕过）；
- 针对特定应用品牌的硬编码路径、窗口名或业务规则分支；
- `iframe` / `<webview>` 子帧注入（不在 `M-002` 范围内，维持 `MainFrame` 契约）。
