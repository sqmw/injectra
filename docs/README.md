# 文档导航

默认读取链：[项目AGENTS](../AGENTS.md) → [任务索引](tasks.md) → 当前需要的一至两个专题 → [代码地图](architecture/code-map.md)。

| 关键词 | 权威文档 |
| --- | --- |
| 产品定位、runtime/pre-runtime、责任边界、需求来源、非目标 | [需求](requirements.md) |
| 第三个里程碑（M-003）、第二个里程碑（M-002）与核心演进路线 | [M-003规划](plans/milestone-3-empirical-closure.md) · [M-003状态](TODO.md#m-003) · [M-002规划](plans/milestone-2-core-engine.md) · [M-002状态](TODO.md#m-002) |
| 当前通用化方向、免重打包、外置增量、真实App反馈 | [dev模式边界](architecture/dev-mode.md) |
| 架构分层、执行域契约、目标身份、能力与错误 | [架构](architecture/boundaries.md) · [多架构宿主机制](architecture/host-mechanisms.md) · [依赖减重评估](architecture/dependency-reduction.md) |
| 模块、测试、符号检索 | [代码地图](architecture/code-map.md) |
| 构建、使用、Bundle、运行数据、恢复 | [开发说明](dev.md) |
| 应用、平台、版本、宿主/内核、CDP/非CDP、当前兼容性 | [兼容性总表（统一入口）](compatibility/current-matrix.md) |
| 验收入口、review与历史证据 | [验证证据](validation.md) |
| 按软件浏览参考源码、启动样例、共享桥接及代码统计 | [参考实现目录](../references/README.md) |
| 插件归属、集中迁移、测试解耦 | [迁移与边界](migrations/plugin-boundary.md) |
| GitHub双语介绍、协作模板、CI、软件引用 | [仓库与协作入口](github.md) |
| 任务与执行 | [任务索引](tasks.md)、[STEPS](STEPS.md) |
| 长期约束与取舍 | [决策](DECISIONS.md) |
| MVP验收、版本与源码映射 | [交付记录](releases.md) |

`make doc-check`检查本地链接、显式锚点和文档可达性；任务结构通过全局project-tracking-check单独校验。历史从任务关闭索引进入，不默认加载。

当前以[T-040@v3](TODO.md#t-040)为范围真相；工作台/内置编辑器退出范围。v1局部实现见[旧方案入口](implementation/platform-v1.md)，不作为继续开发指令。

当前Mac实施：[macOS多引擎交付](macos-delivery.md)。

可选正式接入：[Frida后端](frida-backends.md)。

宿主验证：[CEF/NW.js](host-validation.md)。

WebKit：[WKWebView正式后端与验收边界](webkit-backend.md)。

设备扩展边界：[控制端与目标设备](architecture/device-boundaries.md)。

Gecko：[macOS Firefox实机接入验证](gecko-validation.md)。

Gecko桌面宿主：[Thunderbird安装与实机验证](thunderbird-validation.md)。
