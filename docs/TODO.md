# 任务入口

main 分支为唯一权威来源；Parent 为当前 Codex 主窗口，single-writer。

## 当前任务

<a id="t-004"></a>
### T-004 · MVP 验收收口与仓库推送

```tracking-task
{
  "id": "T-004",
  "name": "MVP 验收收口与仓库推送",
  "version": "v1",
  "mode": "lightweight",
  "status": "in_progress",
  "goal": "记录用户已完成MVP验收，整理可追溯交付提交并推送用户建立的远端仓库。",
  "scope": "验收与版本映射文档、现有工作区和历史审阅、必要检查、提交和Git推送；无产品功能扩展。",
  "acceptance": "用户验收结论已落盘；review与检查通过；提交及MVP版本标识可追溯；远端目标正确且推送成功，确认远端提交与本地一致。",
  "execution_ref": "TODO.md#t-004",
  "acceptance_evidence": [],
  "unplanned_reason": "用户确认MVP通过并要求整理提交推送。"
}
```

```tracking-execution
{
  "id": "T-004/E-001",
  "name": "整理验收记录并提交推送",
  "task_id": "T-004",
  "task_version": "v1",
  "status": "running",
  "authorization": "2026-10-31 用户表示自己验证无问题、MVP通过，已建立仓库，要求整理提交并推送。",
  "scope": "验收与版本映射文档、现有工作区和历史审阅、必要检查、提交和Git推送；无产品功能扩展。",
  "steps": [],
  "evidence": [
    "起始bc4bff9，main工作区干净，无Git remote配置；后定位到用户公开仓库sqmw/injectra，并配置origin。"
  ],
  "stop_reason": ""
}
```

恢复上下文r2：2026-10-31，当前主窗口Parent，main/single-writer，起始bc4bff9；origin为sqmw/injectra；远端LICENSE初始化历史已合入。make check与tracking检查通过，review确认仅验收文档与许可证合并。当前动作：提交验收文档、标记v0.1.0-mvp并推送核验。

## 已关闭任务

<a id="t-003"></a>
### T-003 · 豆包 Electron 兼容性验证

```tracking-task
{
  "id": "T-003",
  "name": "豆包 Electron 兼容性验证",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "以本机已安装豆包验证Injectra核心链路，并如实识别其宿主和接入边界。",
  "scope": "应用版本/宿主识别、调试入口探测、真实页面CSS/HTML/JS注入与清理、兼容性记录；优先独立profile，不改包或账号操作。",
  "acceptance": "给出有证据的宿主类型和CDP可达性结论；可达则验证发现、注入、交互、重复应用与清理；不可达保留失败证据及具体阻塞；记录运行副作用和恢复状态。",
  "execution_ref": "archive/T-003.md#t-003",
  "acceptance_evidence": [
    "已识别为豆包2.33.5定制Chromium147，非标准Electron；现有CLI真实页面注入、交互、更新与清理通过，原调试端口关闭并恢复正常豆包。详见compatibility/doubao.md及归档执行。"
  ],
  "unplanned_reason": "用户指定真实豆包兼容性验证。"
}
```

名称沿用任务建立时的用户目标；实测宿主已纠正为定制Chromium，不计作标准Electron。
[实测报告](compatibility/doubao.md) · [执行记录](archive/T-003.md#t-003)

<a id="t-002"></a>
### T-002 · macOS Electron 注入核心 MVP

```tracking-task
{
  "id": "T-002",
  "name": "macOS Electron 注入核心 MVP",
  "version": "v1",
  "mode": "staged",
  "status": "done",
  "goal": "提供可运行CLI，向显式开放本地调试端口的Electron页面注入共享CSS/HTML/JS并移除受管理改动。",
  "scope": "Rust CLI；核心契约；共享Web载荷；CDP适配；macOS启动参数规划；独立Electron验证；文档与测试。",
  "acceptance": "页面发现和精确目标选择；CSS/HTML/JS注入与移除实测；重复应用不累积节点；脚本异常可见；拒绝非本地调试地址；核心不依赖具体适配；构建、测试、review与Electron闭环通过。",
  "priority": "P1",
  "unplanned_reason": "首次CLI垂直切片，暂不增加独立Milestone聚合。",
  "execution_ref": "archive/T-002.md#t-002",
  "acceptance_evidence": [
    "CLI核心验收通过：共享CSS/HTML/JS在真实macOS Electron fixture完成发现、注入、交互、更新、清理与失败路径；6项Rust、5项Web、10项Electron检查通过；review及文档完成，见validation.md和归档执行。"
  ]
}
```

[恢复上下文](archive/T-002-context.md) · [执行与步骤](archive/T-002.md#t-002)

<a id="t-001"></a>
### T-001 · Injectra 项目初始化

```tracking-task
{
  "id": "T-001",
  "name": "Injectra 项目初始化",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "建立可追踪的项目基础并核对 macOS Electron MVP 的需求与架构输入。",
  "scope": "Git 初始化；项目文档与任务骨架；读取分享需求；确定 MVP 工程初始化边界。",
  "acceptance": "Git、README、docs 入口、AGENTS、gitignore 可用；架构明确共享 Web 与可替换适配边界；已核对分享中的功能与技术栈，工程初始化范围可据此确定；完成适配当前产物的检查与 review。",
  "execution_ref": "archive/T-001.md#t-001",
  "acceptance_evidence": [
    "d55ffa5 保存基础骨架；make check、29条本地链接及tracking检查通过；分享正文已读取并写入requirements，用户确认CLI核心优先，实际实现归属T-002。"
  ],
  "unplanned_reason": "首次项目初始化，需求正文尚未取得，暂不建立无依据的 Milestone。"
}
```

[初始化证据](archive/T-001.md#t-001)
