# T-019 关闭上下文

- [任务](../TODO.md#t-019)@v1；[执行](T-019.md#t-019) E-001；本窗口/root、single-writer。
- 输入74844e7；关闭修订r5，2026-11-10据最终日志和01cbf7f核对；实现提交24ca687、01cbf7f，任务关闭记录在本文件所在提交。
- 工作分支codex/chinami/3ceb631129fa4d73a322fab01d2f961e/7761403b62f6；没有合入main或推送。运行数据留仓库外；控制窗口恢复未复制或更换Mac工作树。
- 设计与扩展合同见[设备边界](../architecture/device-boundaries.md)，实测、保留失败和恢复见[验收](../validation/device-boundaries.md)。
- 现有crate/core/Web公开契约保持；v2会话、v1只读兼容；旧Frida页面ID须重新发现。手机/USB/ADB/自动恢复未实现。
- 本上下文只保留交付定位，任务和执行状态以上述权威记录为准；后续集成或移动端实施按新的实际授权记录。
