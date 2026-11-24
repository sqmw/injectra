# 第二个里程碑规划：通用底层注入引擎与 CLI 底座夯实（M-002）

权威状态与验收见[M-002](../TODO.md#m-002)，长期边界决策见[D-015](../DECISIONS.md#d-015)。本文件记录架构划分、阶段顺序与工程纪律，不重复维护第二份任务状态。

## 定位与工程纪律

- **目标定位**：Injectra 定位为**通用底层注入引擎／SDK 与一等公民 CLI**。上层工具（UI Annotation Lab、插件管理器、MCP 封装、日常持久化 runner）基于 CLI 或 Rust 库契约构建，不进入 Injectra 核心。
- **核心代码优先**：新能力沉淀进正式 `crates/` 与统一 CLI 契约，不以临时脚本拼接或应用品牌补丁替代正式后端；按阶段渐进落地，每阶段单独闭环验收。
- **既有契约保持**：延续[产品边界](../requirements.md)、[执行域架构](../architecture/boundaries.md)（`core / web / native / application` 分域、唯一共享 `web/runtime.js`）与[dev 模式语义](../architecture/dev-mode.md)。

## 主流 Web 桌面架构的接入边界

判断应用能否走 pre-runtime 改造，关键在于**宿主从启动到创建 WebView 之间是否存在可保持语义与签名条件的磁盘控制层**（如主进程入口或 preload），而不在于磁盘上能否找到页面 JS 文件。

| 架构 | 磁盘控制层特征 | 推荐主路线 | 明确边界（不强做补丁） |
| --- | --- | --- | --- |
| Electron | 多数存在目录、ASAR 或嵌套 ASAR 主入口（CJS/ESM）与 preload | **Runtime（CDP/Gadget）＋ Pre-runtime（`prepare`/`dev`）互补** | V8 字节码（`.jsc`）、自定义加密加壳、以及重签导致系统钥匙串或共享容器拒权的原包直接报 `Unsupported` |
| NW.js | 存在 `package.json` 与本地入口 | **Runtime（CDP）为主，Pre-runtime 按需覆盖** | 不做应用品牌特判 |
| Tauri（v1 / v2） | 生产包通常将前端压缩编译进原生二进制只读段，磁盘无独立 JS 文件 | **Runtime 为主**（Windows WebView2 调试参数 → CDP；macOS WKWebView 桥接） | 不改原生二进制只读数据段 |
| CEF | 宿主为原生编译程序，页面可能是本地资源包或远端 URL | **Runtime（标准 CEF 调试端口 → CDP）为主** | 启动停留在原生登录窗且未初始化 CEF 时不伪造 Web 入口 |
| Gecko（Firefox / Thunderbird） | 协议驱动 | **Runtime（BiDi / Marionette）路线** | 仅在正式协议后端设计完成后纳入 CLI |

## 渐进推进阶段

| 顺序 | 任务 | 核心产出 | 阶段验收边界 |
| --- | --- | --- | --- |
| 阶段 1 | [T-083](../TODO.md#t-083) | CLI 机器契约与只读观测原语 | 进程边界保留 `ErrorKind / Phase / Certainty` 结构化错误与可区分退出码；提供 `list`（已注入 bundle 列表）与受控只读 `eval`；向后兼容既有成功 JSON |
| 阶段 2 | [T-084](../TODO.md#t-084) | 应用引擎只读诊断与非 CDP 会话收敛 | 新增跨平台只读 `inspect-app` 与扩展 `launch-plan`（覆盖 Electron/NW.js/CEF/Tauri/WebView2 识别与前置检查）；收敛 Gadget/WKWebView 外部脚本拼接为统一 CLI 会话边界 |
| 阶段 3 | [T-085](../TODO.md#t-085) | Web 桌面架构调研与跨平台核心正式化 | 产出可核验的多架构接入调研基线；按 `platform-* / hosts / preparation` 正规分层沉淀 Windows 正式后端/准备能力与 Electron ESM 入口，清理对 `tests/windows/` 实验脚本的依赖 |
| 阶段 4（按需） | [T-086](../TODO.md#t-086) | 共享 Web Runtime 策略容错与依赖减重 | 在共享 `web/runtime.js` 中为 Trusted Types / 严格 CSP 页面提供显式安全 DOM 挂载路径并与脚本异常分离报告；评估核心准备器去外部重工具链依赖 |

## 当前明确不纳入范围

- **`iframe` / `<webview>` 子帧注入**：渐进开发，本里程碑保持 MainFrame 单一页面语义。
- **重加壳 / 强对抗**：不实现脱壳、V8 字节码反编译、原生二进制段补丁或系统保护绕过。
- **上层产品能力**：内置 MCP Server、日常重启自动重注入 runner、图形工作台、内置编辑器与插件商店均不在本里程碑范围内。
