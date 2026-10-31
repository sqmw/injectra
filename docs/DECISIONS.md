# 决策记录

<a id="d-001"></a>
## D-001 · 首发范围与共享代码边界

```tracking-decision
{
  "id": "D-001",
  "date": "2026-10-31",
  "status": "accepted",
  "conclusion": "首个 MVP 限于 macOS + Electron；共享 Web 注入代码保持单一来源，平台、宿主与应用差异隔离。具体语言与注入机制未决。",
  "trigger": "用户要求初始化多平台、多体系项目并明确 MVP 范围与可插拔架构要求。",
  "context": "T-001；空仓库；分享需求正文尚未读取成功。",
  "affects": ["T-001", "后续 MVP 任务与架构设计"],
  "alternatives": ["为每个平台复制 Web 逻辑：不符合用户要求", "同时实现全部平台：超出首发范围"],
  "commit": "本条所在提交",
  "supersedes": []
}
```

来源：[需求证据](requirements.md)。候选模式及接口见[架构边界](architecture/boundaries.md)。
回退/演进：本次仅创建仓库与文档，无目标应用副作用；后续需求改变本决策时建立替代 Decision，保留原提交和证据。

<a id="d-002"></a>
## D-002 · CLI核心与适配边界

```tracking-decision
{
  "id": "D-002",
  "date": "2026-10-31",
  "status": "accepted",
  "conclusion": "在D-001首发/共享边界内采用Rust CLI、CDP本地运行时适配、独立共享Web载荷；仅实现显式页面发现、apply/remove及只读macOS启动规划。",
  "trigger": "已读取分享正文；用户明确MVP无需图形界面，先做好核心。",
  "context": "T-002@v1。D-001的语言/机制开放项在此收敛，其首发和隔离约束继续有效。",
  "affects": [
    "T-002",
    "workspace crate边界",
    "CLI与Bundle合同"
  ],
  "alternatives": [
    "完整GUI/管理平台：不是当前核心",
    "直接改ASAR或原生Hook：非必要且增加修改/恢复风险",
    "Node CLI：反馈更快，但本项目选择Rust承担未来原生接入与无Node运行时分发，接受编译成本"
  ],
  "commit": "本条所在提交",
  "supersedes": []
}
```

接口与限制见[架构](architecture/boundaries.md)。成本：Rust工具链与测试用Node并存；当前仅一个运行时实现，不承诺零成本新增引擎。恢复：停止CLI不撤销已注入页面内容，使用remove执行受管理清理；源码按独立提交回退，保留历史测试证据。
