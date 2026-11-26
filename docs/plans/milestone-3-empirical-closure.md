# 第三个里程碑规划：商业宿主正式 CLI 实证闭环与准备器减重收口（M-003）

权威状态与验收见[M-003](../TODO.md#m-003)，长期边界决策见[D-016](../DECISIONS.md#d-016)与[D-015](../DECISIONS.md#d-015)。本文件记录承接关系、代表宿主回测矩阵、阶段顺序与排除边界，不重复维护第二份任务状态。

## 定位与承接关系

- **承接基线**：[M-002](../TODO.md#m-002)（`milestone/core-engine-cli-foundation`）已在受控夹具与双端单测上交付机器契约（`--machine`、`list`、`eval`）、跨宿主只读诊断（`inspect-app`、通用 `launch-plan`）、跨平台准备器（含 Electron ESM `esm-delegate`、Windows PE `INTEGRITY` 与 `session attach-v8`）以及声明式安全 DOM 挂载（`html_mode="dom"` 与 `ErrorKind::PolicyBlocked`）。
- **核心目标**：在不扩张核心职责边界的前提下，用真实商业桌面宿主检验 `M-002` 正式 CLI 原语能否替代早期实验脚本并穿透野生打包限制，刷新全项目唯一兼容性入口[current-matrix.md](../compatibility/current-matrix.md)，并完成 `preparation` 纯 Rust ASAR 切换以移除 `@electron/asar` 外部包树。
- **验证与证据纪律**：所有商业软件回测仅在仓库外受控隔离副本或显式隔离会话中执行，不强杀用户日常实例、不把应用品牌特判写入核心 crate；历史任务（`T-014`、`T-043`、`T-044` 等）原证据保持只读，新结果以独立 run 与新增分节记录。

## 代表商业宿主回测矩阵与验收标准

无需重复全量普查，选取覆盖四类关键机制边界的代表样本，全程仅使用 `injectra` 正式 CLI 子命令（零 `tests/windows/*` 旧脚本或跨机测试控制端介入）：

| 机制边界 | 代表样本类型 | 验证路径与关键断言 |
| --- | --- | --- |
| 严格 Trusted Types / CSP 页面挂载 | 此前在[Windows复验](../validation/windows-apps.md)中 HTML 字符串受限的宿主（如 `Cursor`、`Devin`、`Quark` 选 1–2 款） | 默认 `inner-html` 返回确定性 `NotExecuted`（`kind: "policy_blocked"`）且零残留；切换 `html_mode="dom"` 后 `apply`、`list`、`eval` 交互核对与 `remove` 清理全部返回退出码 `0` |
| Electron ESM 主入口 + Windows PE `INTEGRITY` | 含 `type: "module"` 与 PE 资源完整性校验的宿主（如 Windows `Folo` 隔离副本） | `inspect-app` 识别 `pe-asar-integrity` 与 `esm`；原生 `injectra prepare` 派生副本 → `injectra dev` 完成两版外置 JS/CSS/HTML 热更新与清理 |
| 无预留调试端口的 Windows GUI Electron | 此前依赖实验脚本接入的宿主（如[九项复验](../validation/windows-retries.md)中的 `QQ` 或 `WorkBuddy` 隔离会话） | 原生 `injectra session attach-v8 --bootstrap-output` 建立桥接 → `injectra dev` 或正式运行时子命令完成页面发现、挂载与清理 |
| 32 位 / 嵌套归档与不支持目标边界 | `ia32`、嵌套 ASAR 或重加壳/无控制层样本核对 | `inspect-app --machine` 与 `launch-plan` 输出架构、前置条件或明确 `Unsupported`（退出码 `3`），不静默降级 |

## 渐进推进阶段

| 顺序 | 任务 | 核心产出 | 阶段验收边界 |
| --- | --- | --- | --- |
| 阶段 1 | [T-088](../TODO.md#t-088) | 商业宿主 Trusted Types 与正式 CLI 真机回测 | 在 macOS 与 Lenovo Windows 受控隔离环境下完成上述四类代表宿主正式 CLI 闭环测试，沉淀结构化实测证据；若暴露通用宿主时序或路径缺陷则按正规分层修复并补回归测试 |
| 阶段 2 | [T-089](../TODO.md#t-089) | 兼容性总表刷新与正式 CLI 证据索引收口 | 更新[current-matrix.md](../compatibility/current-matrix.md)过时阅读口径（清理旧未推送与非正式 CLI 过期表述），增补 `M-002`/`M-003` 正式 CLI 路线、机器契约与实测边界索引，保持旧普查原貌 |
| 阶段 3 | [T-090](../TODO.md#t-090) | 准备器纯 Rust ASAR 统一切换与 npm 依赖收口 | 扩展纯 Rust [asar.rs](../../crates/preparation/src/asar.rs) 覆盖目录与 macOS 符号链接重建，移除准备工具集中的 `@electron/asar`（含 11 个间接包），同步[依赖减重评估](../architecture/dependency-reduction.md)并通过双端全布局实测 |

## 按需触发项与明确不纳入范围

- **显式子帧（`PageContext::SubFrame`：`iframe` / Electron `<webview>`）**：作为独立后续架构演进方向保留（契约位置见[执行域架构](../architecture/boundaries.md)与[多架构宿主机制](../architecture/host-mechanisms.md)），不在 `M-003` 默认铺开；仅当阶段 1 回测或上层实际插件消费遇到主业务界面位于子帧的目标时另行立项推进。
- **上层生态与 Agent 编排层**：`web-ui-plugins`、`UI Annotation Lab` 以及面向通用/弱模型的外部 Agent Skill / MCP 薄封装继续按[D-015](../DECISIONS.md#d-015)在独立仓库或外部目录维护，通过 `injectra --machine` 消费底座能力，不合入本仓库。
- **重加壳与二进制补丁**：V8 字节码（`.jsc`）、自定义加密加壳、原生二进制只读段补丁与系统安全机制绕过继续明确返回 `Unsupported`。
