# Injectra · Agent 入口

全局协作规范继续生效；本文件只补充项目边界。

## 首读与导航

1. [项目说明](README.md)：用途、当前交付边界与检查命令。
2. [文档入口](docs/README.md) → [任务入口](docs/TODO.md)：绑定 Task/Execution，恢复当前重点。
3. 按任务读取一至两个专题；从[代码地图](docs/architecture/code-map.md)定位模块，之后定向检索。

## 项目约束

- MVP 首个支持组合是 macOS + Electron 宿主；不据此推定 Injectra 自身使用 Electron 构建。
- 共享 Web 注入代码保留单一来源；平台、宿主体系、目标应用差异分别隔离，禁止复制成各平台版本。
- 当前采用Rust CLI + CDP；需求和边界见docs/requirements.md。不得把接口设计或测试替身当成真实应用兼容性证明。
- 入口只负责装配；核心流程依赖契约，具体适配实现不得反向进入核心。
- 新适配必须说明支持能力、限制、失败行为与验证范围；不以空实现伪装跨平台支持。
- 常规编辑与验证在本机仓库完成。缓存、日志、备份及运行数据的位置在落地时独立确定，默认放仓库外。
- 冻结基线：无。架构候选与已确定边界分开记录，见[架构边界](docs/architecture/boundaries.md)。

## 检查入口

`make build`构建；`make setup-tests`准备仓库外测试依赖；`make check`运行格式、lint、Rust/Web测试与文档检查；`make test-electron`运行独立真实Electron验证。工具版本见mise.toml；禁止将测试通过外推为全部应用兼容。
