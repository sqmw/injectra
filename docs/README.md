# 文档导航

默认读取链：[项目AGENTS](../AGENTS.md) → [任务入口](TODO.md) → 当前需要的一至两个专题 → [代码地图](architecture/code-map.md)。

| 关键词 | 权威文档 |
| --- | --- |
| 产品定位、runtime/pre-runtime、责任边界、需求来源、非目标 | [需求](requirements.md) |
| 分层、插件、设计模式、契约 | [架构](architecture/boundaries.md) |
| 模块、测试、符号检索 | [代码地图](architecture/code-map.md) |
| 构建、使用、Bundle、运行数据、恢复 | [开发说明](dev.md) |
| 验收、兼容性、review、测试边界 | [验证证据](validation.md) |
| 豆包、真实产品、Chromium接入 | [豆包兼容性](compatibility/doubao.md) |
| 任务与执行 | [TODO](TODO.md)、[STEPS](STEPS.md) |
| 长期约束与取舍 | [决策](DECISIONS.md) |
| MVP验收、版本与源码映射 | [交付记录](releases.md) |

`make doc-check`检查本地链接、显式锚点和文档可达性；任务结构通过全局project-tracking-check单独校验。历史从任务关闭索引进入，不默认加载。
