# 任务入口

main 分支为唯一权威来源；Parent 为当前 Codex 主窗口，single-writer。

## 已关闭任务

<a id="t-006"></a>
### T-006 · Xmind 架构图排版演示

```tracking-task
{
  "id": "T-006",
  "name": "Xmind 架构图排版演示",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "在用户打开的Xmind中，以保留原图的方式演示可读的模块结构与runtime注入路径。",
  "scope": "通过Xmind界面新建或复制演示画布并排版；仅新增副本/演示，不覆盖原图，不改产品代码。",
  "acceptance": "原画布保留；演示画布可见、结构及当前执行路径可读；核对界面结果并记录保存状态。",
  "execution_ref": "archive/T-006.md#t-006",
  "acceptance_evidence": [
    "已通过Xmind界面制作01·模块结构（22主题、六层逻辑图）及02·runtime路径（7主题、5条直线箭头）；原画布40主题保留；另存演示文件，UI无Edited标识且文件存在；review、文档及任务检查通过。"
  ],
  "unplanned_reason": "用户明确要求直接操作当前打开的Xmind进行演示。"
}
```

[界面操作与review证据](archive/T-006.md#t-006)

<a id="t-005"></a>
### T-005 · runtime 注入产品边界校准

```tracking-task
{
  "id": "T-005",
  "name": "runtime 注入产品边界校准",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "按用户批注与定位图片统一当前产品为代码注入工具／核心，仅交付runtime注入能力。",
  "scope": "项目定位、需求、架构职责、使用与验收说明、导航及决策记录；不修改源码、命令行为或历史验收证据。",
  "acceptance": "当前注入合同、使用者责任、后续候选方向明确且跨文档一致；已有清理接口仅记为实现事实；review、文档与任务结构检查通过。",
  "execution_ref": "archive/T-005.md#t-005",
  "acceptance_evidence": ["定位、责任及当前验收已统一，D-003记录用户纠正；review、make doc-check、任务结构检查与git diff --check通过；仅Markdown变更。"],
  "unplanned_reason": "用户纠正产品范围并明确要求同步文档与当前产品边界。"
}
```

[产品边界](requirements.md) · [执行与review证据](archive/T-005.md#t-005)

<a id="t-004"></a>
### T-004 · MVP 验收收口与仓库推送

```tracking-task
{
  "id": "T-004",
  "name": "MVP 验收收口与仓库推送",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "记录用户已完成MVP验收，整理可追溯交付提交并推送用户建立的远端仓库。",
  "scope": "验收与版本映射文档、现有工作区和历史审阅、必要检查、提交和Git推送；无产品功能扩展。",
  "acceptance": "用户验收结论已落盘；review与检查通过；提交及MVP版本标识可追溯；远端目标正确且推送成功，确认远端提交与本地一致。",
  "execution_ref": "archive/T-004.md#t-004",
  "acceptance_evidence": [
    "用户验收已记录；review、make check与任务检查通过；main与v0.1.0-mvp已推送sqmw/injectra，ls-remote确认交付提交a763157一致；远端初始化LICENSE历史保留。"
  ],
  "unplanned_reason": "用户确认MVP通过并要求整理提交推送。"
}
```

[交付记录](releases.md) · [执行证据](archive/T-004.md#t-004)

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
