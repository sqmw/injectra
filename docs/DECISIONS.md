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
