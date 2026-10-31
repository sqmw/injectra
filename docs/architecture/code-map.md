# 代码地图

先读本页，按关键词进入对应模块；不要批量读取整个源码树。

| 能力 | 源码入口与职责 | 上下游边界 | 测试与专题 |
| --- | --- | --- | --- |
| 核心契约 | [core](../../crates/core/src/lib.rs)：Engine/Session/Target/Capability | 不依赖具体适配；只编排能力检查 | 内联假Session测试；[架构](boundaries.md) |
| 载荷加载 | [bundle.rs](../../crates/payload/src/bundle.rs)：版本、路径和资源限制 | 本地manifest → Bundle | 内联loader测试、[负例](../../tests/bundles/escape.json)；[使用](../dev.md) |
| 程序生成 | [payload](../../crates/payload/src/lib.rs)：共享runtime嵌入、数据序列化 | Bundle → JS程序 | Electron端到端测试 |
| Web生命周期 | [runtime.js](../../web/runtime.js)：拥有节点、清理、重复/并发注入 | 只用Web能力，无平台判断 | [runtime.test.mjs](../../tests/runtime.test.mjs) |
| CDP发现 | [discovery.rs](../../crates/cdp/src/discovery.rs)：本地HTTP发现 | CDP页面 → 核心Target | Electron发现实测 |
| CDP连接执行 | [session.rs](../../crates/cdp/src/session.rs)、[Engine](../../crates/cdp/src/lib.rs) | WebSocket/CDP → Session | 内联地址/关联/错误测试；Electron实测 |
| macOS平台 | [platform-macos](../../crates/platform-macos/src/lib.rs)：.app启动规划 | 路径 → LaunchPlan；无进程管理 | Electron.app规划实测 |
| CLI装配 | [main.rs](../../crates/cli/src/main.rs)、[args.rs](../../crates/cli/src/args.rs) | 参数 → 适配/载荷 → JSON | [smoke](../../scripts/electron-smoke.mjs) |
| 示例载荷 | [manifest](../../examples/accent/manifest.json)、[behavior](../../examples/accent/behavior.js) | Web页面与清理回调 | 同上 |
| 工程入口 | [Makefile](../../Makefile)、[cargo.sh](../../scripts/cargo.sh)、[测试环境](../../scripts/test-env.mjs) | 外部缓存，不污染同步工作区 | make check；[开发](../dev.md) |
| 测试工具 | [setup-tests](../../scripts/setup-tests.mjs)、[锁文件](../../tests/package-lock.json)、[fixture](../../tests/electron/main.cjs) | 自有测试进程与profile | make test-electron；[验证](../validation.md) |
| 真实应用验证 | [verify-live](../../scripts/verify-live.mjs)：精确目标上的CLI注入、断言、自有面板截图与清理 | 不管理用户进程，只接入显式端口/页面 | make verify-live；[豆包实测](../compatibility/doubao.md) |
| 文档检查 | [check-docs](../../scripts/check-docs.mjs) | 相对链接/显式锚点/可达性 | make doc-check，不证明业务正确性 |
