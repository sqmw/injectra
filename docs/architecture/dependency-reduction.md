# 外部运行时依赖评估与纯 Rust 减重边界

本文件对应 [T-086](../TODO.md#t-086) 与 [M-002 规划](../plans/milestone-2-core-engine.md)，盘点 Injectra 各路径对外部运行时与第三方包的依赖现状、纯 Rust 替换的收益/风险边界，以及本轮已落地的纯 Rust 替代实现。

## 依赖全景与零外部依赖边界

| 能力路径 | 运行期外部依赖 | 当前状态 |
| --- | --- | --- |
| Runtime CDP（`targets` / `apply` / `remove` / `list` / `eval`） | **无**（单二进制 Rust CLI） | 无需 Node、Python 或 Frida；共享 [web/runtime.js](../../web/runtime.js) 与纯 Rust 声明式 DOM 编译器 [dom.rs](../../crates/web/src/dom.rs) 均已编译内嵌 |
| 只读宿主诊断与规划（`inspect-app` / `launch-plan`） | **无**（macOS 仅调用系统 `/usr/bin/plutil`、`otool`、`codesign`；Windows 路径全纯 Rust） | 纯 Rust [asar_probe.rs](../../crates/hosts/src/asar_probe.rs) 与 [pe.rs](../../crates/platform-windows/src/pe.rs) 已替代 Node 探测依赖 |
| 非 CDP Frida 路由（`electron-gadget`、`webkit`、`session *`） | 可选 Python + Frida 虚拟环境（`make setup-frida`） | 由目标运行期插桩机制本身决定，不进入默认 CDP 依赖 |
| Pre-runtime 准备器（`prepare`） | Node.js + 外部锁定准备工具（`make setup-preparation`） | 本轮完成评估并落地部分纯 Rust 替换，见下表 |

## `preparation` 外部依赖逐项评估

| 外部依赖 | 职责 | 替代成本与风险 | 本轮结论与产出 |
| --- | --- | --- | --- |
| `chromium-pickle-js` | 编码 ASAR 16 字节 Chromium Pickle 头 | **极低**：格式固定（4 个 `u32` + UTF-8 JSON + 4 字节对齐填充） | **已去除运行期依赖**：[archive.mjs](../../crates/preparation/helper/archive.mjs) 与 [prepare.mjs](../../crates/preparation/helper/prepare.mjs) 改用原生 8 行 `Buffer` 编码；Rust 侧在 [asar.rs](../../crates/preparation/src/asar.rs) 提供零依赖实现 |
| `@electron/asar`（含 11 个间接依赖：`glob@7`、`inflight` 等） | 读取 ASAR 头/条目、增量追加改写（`appendRewrite`）、macOS 全量流重建（`rebuild`） | **中低**：只读解析已在 `asar_probe.rs` 实现；保留原数据段的增量追加改写与 `4 MiB` SHA-256 块完整性可用现有 `serde_json` + `sha2` 零新 crate 实现 | **已落地纯 Rust 模块**：新增 [crates/preparation/src/asar.rs](../../crates/preparation/src/asar.rs)（`read_header` / `extract_file` / `append_rewrite`）并通过 macOS 与 Windows 跨平台测试；保留 Node 侧 `@electron/asar` 以兼容 macOS `rebuild` 与符号链接完整流重建，后续可按需统一切换至纯 Rust 路径 |
| `acorn`（单包、零间接依赖） | 解析 CommonJS 顶层指令（`#!`、`'use strict'`、`function module()` 遮蔽检查）与校验 ESM 语法 | **高**：若在 Rust 中替代需引入完整 JS 解析器（如 `oxc_parser` / `swc_ecma_parser`，显著增加编译依赖树与体积）；手写正则易在注释/模板字面量/ASI 边界误判并破坏宿主入口 | **明确保留**：单包零间接依赖、语义精确，当前无引入重型 Rust JS 解析器 crate 的正向收益 |
| `resedit`（+ `pe-library`，仅 Windows 改写时使用） | 修改 Windows `.exe`/`.dll` 的 PE `.rsrc` 节中的 `INTEGRITY`/`ELECTRONASAR` 资源并重算节区布局 | **高**：只读 PE 解析已在 [pe.rs](../../crates/platform-windows/src/pe.rs) 纯 Rust 实现；但改写 PE 资源在 JSON 长度变化时涉及节区扩容、RVA 重定位与校验更新，自研维护风险高 | **读写分离**：只读解析保持纯 Rust，Windows 写路径继续使用锁定的 `resedit` |
| 声明式 HTML 安全解析（Trusted Types 场景） | 将外部 `html` 片段编译为无 `innerHTML` 沉槽的 DOM 节点树 | **低**：HTML 片段词法/树构建与 `<script>` / `on*` 静态校验可在 Rust 准备期一次性完成，零外部 crate | **已纯 Rust 落地**：[crates/web/src/dom.rs](../../crates/web/src/dom.rs) 在提交页面前完成编译与安全校验，页面侧 [web/runtime.js](../../web/runtime.js) 仅用 `createElement` / `createElementNS` / `createTextNode` 挂载 |

## 验收与演进边界

- **收益量化**：运行期 CDP、只读诊断（`inspect-app` / `launch-plan`）与声明式安全 DOM 编译全部零 Node / 零新增第三方 Rust crate；Windows 准备器不再强制要求 `chromium-pickle-js`，纯 Rust [asar.rs](../../crates/preparation/src/asar.rs) 为后续彻底移除 `@electron/asar` 的 12 个 npm 包提供了经跨平台测试验证的核心实现。
- **不以脆弱手写解析换取“零依赖”标签**：对完整 ECMAScript AST（`acorn`）与 PE `.rsrc` 二进制重排（`resedit`），在没有经验证的轻量级 Rust 替代方案前保留锁定外部工具，避免降低真实应用兼容性。
