# 可插拔架构

## 当前职责

Injectra当前是runtime代码注入工具／核心；产品责任以[需求](../requirements.md)及[D-003](../DECISIONS.md#d-003)为准。核心只承担接入、目标选择、载荷执行和结果反馈，应用选择器、重渲染覆盖与业务行为由使用者负责。管理平台、Agent、pre-runtime及自动恢复不进入当前架构交付。

## 分层与依赖

实现选择见 [D-002](../DECISIONS.md#d-002)，首发和共享代码约束见 [D-001](../DECISIONS.md#d-001)。当前没有冻结基线。

```text
CLI（组合入口）
 ├── core：Target / Engine / Session / Capability / execute
 ├── payload：本地Bundle → 共享Web程序
 ├── cdp：Engine/Session的CDP实现 → 本地Electron页面
 └── platform-macos：标准.app识别 → 启动参数规划
payload ──→ web/runtime.js（同一份JS，编译时嵌入）
cdp / payload / platform-macos ──→ core
core 不依赖上述任何实现模块
```

| 边界 | 输入 → 输出 | 替换与限制 |
| --- | --- | --- |
| core | 目标身份、脚本源 → 能力检查与执行结果 | 定义窄契约，不识别Electron、OS路径或应用名称 |
| payload + Web runtime | manifest、CSS/HTML/JS → 一个可执行程序 | 所有未来适配消费同一程序；无需按平台复制 |
| cdp | 回环端口、页面ID → targets、Session、结果 | 仅实现当前声明的JS执行能力；拒绝浏览器主进程端点和跨地址跳转 |
| platform-macos | 标准.app路径、端口 → LaunchPlan | 只读规划，不管理用户进程；原生接入与协议能力分离 |
| CLI | 命令与文件参数 → JSON/退出码 | 仅装配与调用；无业务状态、应用名分支表 |
| 使用者载荷 | 局部选择器、样式和脚本 → 页面效果 | 由使用者维护；examples只用于示例与验收，不形成应用适配目录 |

未来 WebKit、协作式宿主接口或其他接入方式须实现契约并声明能力。不因“界面是Web”就承诺可连接，也不强迫不同平台实现虚假的同一组能力。

## 数据表示与单次注入行为

下列内容描述已有实现，不要求扩展自动恢复或任意JS撤销。web/runtime.js是共享注入执行程序；其节点记录与清理逻辑不代表应用生命周期管理平台。

- Target：id/title/url，是用户选择与错误定位依据；CDP WebSocket 地址留在适配器内部。
- Bundle：version/id/相对资源路径；CSS/HTML 以 JSON 编码作为数据，JS 明确作为可执行源码。
- Session：能力查询与JS执行；当前CLI一次操作建立一次连接，完成后释放。不会自动重试副作用操作。
- Web runtime：页面内按bundle id记录拥有的DOM节点和清理回调；同ID并发操作拒绝，不同ID分别记录。
- 更新：清理旧版 → 创建节点 → 执行JS → 登记成功。创建/执行失败时清理本次拥有的节点与已登记回调；不能保证撤销任意JS，不能保证恢复旧版。
- 超时/断线：结果未知；页面内脚本可能仍在执行，不因错误返回就宣称无副作用。

页面重载会清空页面内记录；同文档SPA重渲染也可能覆盖注入节点。使用者负责再次注入或在自己的载荷中处理页面变化；导航监听、DOM守护、持久化及后台重注入不属当前职责。

## 设计模式与取舍

| 模式/原则 | 当前落实 | 边界 |
| --- | --- | --- |
| Adapter | CdpEngine/CdpSession实现核心契约 | 协议和错误留在适配器；不返回假成功 |
| Bridge / 依赖倒置 | Web程序与执行宿主解耦，核心只依赖Session | 目前只有一个真实运行时，未来替换成本仍需第二适配验证 |
| Strategy | Engine作为可替换执行入口 | 当前只有CDP实现，不构造无用的策略注册系统 |
| 组合根 | CLI显式装配适配与载荷 | 不引入全局服务定位器或导入时副作用 |
| 单一职责 | 按核心、载荷、协议、平台、CLI拆crate | 不把.app识别、manifest或DOM代码塞进核心 |

当前无需抽象工厂层级、动态库ABI、插件市场或热加载；只有出现需成组选择的多套真实实现时，才有引入抽象工厂的依据。不是使用完23种模式，而是每个抽象都对应独立变化点。

## 验收与风险

依赖图可用Cargo metadata检查；假Session验证能力拒绝，真实Electron验证运行效果。共享代码单一来源已落实，但跨引擎可复用性只建立了边界，尚无WebKit实测。受信任JS/HTML拥有页面权限，不提供脚本隔离、跨模块样式隔离或系统级权限控制。

[代码地图](code-map.md)给出入口、测试和检索点；[验证记录](../validation.md)分别报告测试替身与真实应用证据。
