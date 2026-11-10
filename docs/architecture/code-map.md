# 代码地图

先读本页，按关键词进入对应模块；不要批量读取整个源码树。

| 能力 | 源码入口与职责 | 上下游边界 | 测试与专题 |
| --- | --- | --- | --- |
| 核心契约 | [core](../../crates/core/src/lib.rs)：BackendId/TargetRef/Capability/Error | 无执行域/JSON依赖；能力检查与错误语义 | 身份隔离测试；[架构](boundaries.md) |
| Web载荷加载 | [bundle.rs](../../crates/web/src/bundle.rs)：版本、路径和资源限制 | 本地manifest → Bundle | 内联loader测试、[临时夹具](../../tests/support/bundle.rs)；[使用](../dev.md) |
| 程序生成 | [web](../../crates/web/src/lib.rs)：共享runtime嵌入、数据序列化 | Bundle → JS程序 | Electron端到端测试 |
| 共享注入执行 | [runtime.js](../../web/runtime.js)：拥有节点、清理、重复/并发注入 | 单次注入及既有辅助行为；无平台判断或导航恢复 | [runtime.test.mjs](../../tests/runtime.test.mjs) |
| CDP发现 | [discovery.rs](../../crates/cdp/src/discovery.rs)：本地HTTP发现 | CDP页面 → WebTarget | Electron发现实测 |
| CDP连接执行 | [session.rs](../../crates/cdp/src/session.rs)、[WebBackend](../../crates/cdp/src/lib.rs) | WebSocket/CDP → WebSession | [传输合同测试](../../crates/cdp/src/session/tests.rs)；Electron实测 |
| macOS平台 | [platform-macos](../../crates/platform-macos/src/lib.rs)：.app元数据 | 路径 → ApplicationInfo；无宿主识别或进程管理 | Electron.app规划实测 |
| Web执行契约 | [contracts](../../crates/web/src/contracts.rs)：WebBackend/WebSession/页面上下文 | 只声明页面执行，区别于通用JS环境 | [应用合同测试](../../crates/application/tests/web_contract.rs) |
| 应用用例 | [application](../../crates/application/src/lib.rs)：targets/apply/remove/execute | 注入后端；身份与能力检查，不固定CDP | 同上 |
| 原生域 | [target](../../crates/native/src/target.rs)、[memory](../../crates/native/src/memory.rs)：原生目标、地址、独立读写端口 | 无Web、无真实后端 | [受控缓冲区测试](../../crates/native/src/memory/tests.rs)、[契约](boundaries.md) |
| 宿主规划 | [hosts/electron](../../crates/hosts/src/electron.rs)：Electron识别与只读规划 | 平台元数据 → LaunchPlan | recipe单测、Electron实测 |
| CLI装配 | [main.rs](../../crates/cli/src/main.rs)、[args.rs](../../crates/cli/src/args.rs) | 参数 → 适配/载荷 → JSON | [smoke](../../scripts/electron-smoke.mjs) |
| 测试输入（非插件） | [临时夹具](../../tests/support/bundle.rs)、[Electron smoke](../../scripts/electron-smoke.mjs) | 仅在临时目录生成最小输入，结束后清理Rust夹具；不依赖插件库 | make check / make test-electron |
| 工程入口 | [Makefile](../../Makefile)、[cargo.sh](../../scripts/cargo.sh)、[测试环境](../../scripts/test-env.mjs) | 外部缓存，不污染同步工作区 | make check；[开发](../dev.md) |
| 软件参考源码（非产品模块） | [references](../../references/README.md)：apps按软件/路线归档，shared保留Electron/V8桥接及Web程序生成器 | 不加入根workspace；不参与默认构建/测试/发布；不复制Web runtime | make -C references check/stats；[抖音证据](../compatibility/douyin-gadget.md) |
| 测试工具 | [setup-tests](../../scripts/setup-tests.mjs)、[锁文件](../../tests/package-lock.json)、[fixture](../../tests/electron/main.cjs) | 自有测试进程与profile | make test-electron；[验证](../validation.md) |
| 真实应用验证 | [verify-live](../../scripts/verify-live.mjs)：精确目标上的CLI注入、断言、自有面板截图与清理 | 不管理用户进程，只接入显式端口/页面 | make verify-live；[豆包实测](../compatibility/doubao.md) |
| 架构依赖门 | [check-architecture](../../scripts/check-architecture.mjs) | Cargo metadata验证执行域/具体后端方向，core/native独立 | make architecture-check；[契约](boundaries.md) |
| 文档检查 | [check-docs](../../scripts/check-docs.mjs) | 相对链接/显式锚点/可达性 | make doc-check，不证明业务正确性 |

新增并列后端：共享[Frida通信](../../crates/frida/src/lib.rs)、[Electron Gadget](../../crates/electron-gadget/src/lib.rs)、[CLI装配](../../crates/cli/src/backend.rs)；[使用/边界](../frida-backends.md)。

[WKWebView后端](../../crates/webkit/src/lib.rs)复用Frida通信；原生主线程API、异步completion与范围见[WebKit说明](../webkit-backend.md)。

Frida内部边界：[控制端执行](../../crates/frida/src/controller.rs)、[会话模型](../../crates/frida/helper/session_model.py)、[目标接口](../../crates/frida/helper/contracts.py)、[Mac本机核验](../../crates/frida/helper/target_macos.py)、[连接器](../../crates/frida/helper/transport.py)；详见[设备边界](device-boundaries.md)。

WK宿主边界：[Mac AppKit发现/调度](../../crates/webkit/helper/macos_appkit.js)与[WK执行/回调](../../crates/webkit/helper/webkit_agent.js)由[入口](../../crates/webkit/helper/agent.js)装配；[宿主合同](../../tests/webkit/host-contract.test.mjs)不代表手机实现。

Gecko专项验证：[官方下载校验](../../scripts/setup-gecko-tests.py)、[Firefox实机测试](../../scripts/gecko-smoke.mjs)、[测试用BiDi连接](../../tests/gecko/bidi-client.mjs)；见[验证边界](../gecko-validation.md)，不属于正式CLI后端。

Thunderbird验证：[宿主与内容页测试](../../scripts/thunderbird-smoke.mjs)、[Marionette测试连接](../../tests/gecko/marionette-client.mjs)、[共用runtime断言](../../tests/gecko/runtime-checks.mjs)；[说明](../thunderbird-validation.md)，不作为公开产品后端。

Thunderbird宿主差异：[tabmail目标发现/世代核验](../../tests/gecko/thunderbird-host.mjs)、[原Window调试执行](../../tests/gecko/page-world.mjs)；仅测试用，标准BiDi发现限制及特权HTML过滤见上述验收。
