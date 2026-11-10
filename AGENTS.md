# Injectra · Agent 入口

全局协作规范继续生效；本文件只补充项目边界。

## 首读与导航

1. [项目说明](README.md)：用途、当前交付边界与检查命令。
2. [文档入口](docs/README.md) → [任务索引](docs/tasks.md)：绑定 Task/Execution，恢复当前重点。
3. 按任务读取一至两个专题；从[代码地图](docs/architecture/code-map.md)定位模块，之后定向检索。

## 项目约束

- 当前定位为代码注入工具／核心，仅做runtime注入：接入正确页面、执行用户提供的JS/HTML/CSS、明确结果与失败。使用者负责载荷及应用界面适配；pre-runtime、管理平台、Agent、自动恢复和任意JS撤销不属当前交付。完整责任边界见[需求](docs/requirements.md)。
- MVP 首个支持组合是 macOS + Electron 宿主；不据此推定 Injectra 自身使用 Electron 构建。
- 所有实际插件、示例插件及可复用插件Bundle归独立web-ui-plugins库；本仓库不得存放插件或其副本。仅保留注入runtime、接入实现和最小测试夹具，测试不得依赖插件库，详见[产品边界](docs/requirements.md)。
- 共享 Web 注入代码保留单一来源；平台、宿主体系、目标应用差异分别隔离，禁止复制成各平台版本。
- 当前采用Rust CLI，默认CDP，并提供可选Electron Gadget/WKWebView页面后端；需求和边界见docs/requirements.md。不得把接口设计或测试替身当成真实应用兼容性证明。
- 长期扩展边界现在建立：通用core不包含页面、JS、JSON或原生操作假设；Web/native分域，平台与宿主规则隔离，详见[执行域契约](docs/architecture/boundaries.md)。原生测试替身不能作为真实后端支持。
- 入口只负责装配；核心流程依赖契约，具体适配实现不得反向进入核心。
- 新适配必须说明支持能力、限制、失败行为与验证范围；不以空实现伪装跨平台支持。
- 常规编辑与验证在本机仓库完成。缓存、日志、备份及运行数据的位置在落地时独立确定，默认放仓库外。
- 冻结基线：无。架构候选与已确定边界分开记录，见[架构边界](docs/architecture/boundaries.md)。

## 检查入口

`make build`构建；`make setup-tests`准备仓库外测试依赖；`make check`运行格式、lint、Rust/Web测试与文档检查；`make test-electron`运行独立真实Electron验证。工具版本见mise.toml；禁止将测试通过外推为全部应用兼容。
