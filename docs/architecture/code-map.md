# 代码地图

当前方向见[dev模式](dev-mode.md)。代码存在、当前需求和完成验收是三件事；下列v1留存模块不作为继续开发工作台的指令。


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

Thunderbird HTML补充验证：[测试用装配器](../../tests/gecko/parsed-html.mjs)、[装配与套件隔离单测](../../tests/gecko/parsed-html.test.mjs)；[V2路线验收与原合同边界](../validation/thunderbird-html.md)。不注册正式CLI、不修改共享runtime。

原生函数Hook可行性验证：[可选受控启动/attach测试](../../scripts/frida-hook-smoke.py)、[纯函数Interceptor测试agent](../../tests/frida/native-hook.js)；[抖音失败与控制正例边界](../compatibility/douyin-native-hook.md)，不属于已交付页面后端或插件。

Objective-C分派Hook专项：[受控启动测试](../../scripts/objc-hook-smoke.py)、[自有对象IMP替换agent](../../tests/frida/objc-dispatch-hook.js)、[判定/所有权测试](../../tests/frida/test_objc_hook.py)；[抖音实机证据与限制](../compatibility/douyin-objc-hook.md)。不注册正式后端，不修改Web runtime。

真实宿主Hook：[受控入口](../../scripts/host-hook-smoke.py)、[宿主事件agent](../../tests/frida/host-dispatch-hook.js)、[判定测试](../../tests/frida/test_host_hook.py)；与自有对象实验共享[启动/身份/释放夹具](../../tests/frida/gadget_probe.py)，[抖音实测](../compatibility/douyin-host-hook.md)，仅测试、不注册产品后端。

启动诊断：[观察入口](../../scripts/observe-macos-startup.py)、[进程/HTTP证据模块](../../tests/support/startup_observation.py)、[边界测试](../../tests/frida/test_startup_observation.py)、[真实交接夹具](../../tests/frida/test_startup_observation_process.py)；[工具合同](../validation/startup-observer.md)与[阿里实验](../compatibility/adrive-startup.md)。只观察、无载荷，不自动终止目标子树。

资源补丁专项：[独立副本准备/启动](../../scripts/adrive-resource-smoke.mjs)、[ASAR重建](../../tests/resource-patch/archive.mjs)、[副本签名](../../tests/resource-patch/sign-copy.py)、[测试输入](../../tests/resource-patch/page-proof.mjs)、[回执判据](../../tests/resource-patch/receipts.mjs)；[阿里实机证据及签名条件](../compatibility/adrive-resource-patch.md)。make test-resource-patch仅验证工具边界，实机走可选prepare/test目标；不注册正式后端、不复制插件。

批次资源验证：[QQ/腾讯视频/Claude配方与控制](../../scripts/resource-patch-smoke.mjs)、[Electron41+摘要更新](../../tests/resource-patch/integrity-digest.py)及[格式测试](../../tests/resource-patch/integrity-digest.test.mjs)；[顺序实测、共享容器限制和当前失败](../compatibility/three-app-resource-patch.md)。仍是测试工具，不增加正式适配器或复制共享runtime。

QQ外部主题预览：[开发加载器](../../scripts/qq-theme-preview.mjs)复用资源准备和共享程序生成器，接受外部已编译程序；[共享数据授权、实机粉/紫/原样与预览边界](../compatibility/qq-pastel-preview.md)。配色、选择器与Bundle只在独立插件库。

## T-040 CLI收敛

工作台、Studio与完整workspace已移除；旧代码由Git与仓库外恢复快照保留。

| 能力 | 当前入口 |
| --- | --- |
| 页面增量版本与执行 | [application dev](../../crates/application/src/dev/mod.rs)，复用Web契约与共享runtime |
| 入口布局/委托 | [入口策略](../../crates/preparation/helper/entry.mjs)，明文插入或原加载器主模块委托；[QQ复验](../validation/qq-cli-dev.md) |
| 通用准备 | [preparation](../../crates/preparation/src/lib.rs)、[平台helper](../../crates/preparation/helper/platform.py)，probe/plan/prepare与派生实例启动；[验收](../validation/cli-dev.md) |
| CLI dev装配 | [dev](../../crates/cli/src/dev.rs)、[最小未决记录](../../crates/cli/src/dev/journal.rs)；[夹具](../../scripts/preparation-smoke.mjs) |
| App引导与认证通信 | [Bootstrap](../../crates/bootstrap/src/lib.rs)，没有浏览器工作台API |

当前范围见[dev模式](dev-mode.md)，旧方案见[历史入口](../implementation/platform-v1.md)。

Windows复验：[逐应用证据](../validation/windows-apps.md)、[交互启动/原生CLI编排](../../scripts/windows-cdp-smoke.py)、[共享ASAR格式](../../crates/preparation/helper/archive.mjs)。Windows Gecko/Hook/资源副本目前是测试工具，不额外宣称正式产品后端支持。

Windows九项复验：[实机与剩余入口](../validation/windows-retries.md)。实验[MSVC V8](../../tests/windows/v8-windows.js)/[受控Node验证](../../tests/windows/v8-fixture.py)、[CEF初始化](../../tests/windows/cef-init.js)、[包内链接复制](../../tests/windows/copy-tree.mjs)分别独立；都不进入公共core或正式适配方案注册。
