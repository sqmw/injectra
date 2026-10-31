# 任务入口

本仓库 main 分支为当前唯一权威来源；Parent 为当前 Codex 主窗口，single-writer。
初始化采用轻量记录，不预建完整 MVP 的 Stage/Step；需求到位后按真实规模规划。

<a id="t-001"></a>
### T-001 · Injectra 项目初始化

```tracking-task
{
  "id": "T-001",
  "name": "Injectra 项目初始化",
  "version": "v1",
  "mode": "lightweight",
  "status": "blocked",
  "goal": "建立可追踪的项目基础并核对 macOS Electron MVP 的需求与架构输入。",
  "scope": "Git 初始化；项目文档与任务骨架；读取分享需求；确定 MVP 工程初始化边界。",
  "acceptance": "Git、README、docs 入口、AGENTS、gitignore 可用；架构明确共享 Web 与可替换适配边界；已核对分享中的功能与技术栈，工程初始化范围可据此确定；完成适配当前产物的检查与 review。",
  "execution_ref": "TODO.md#t-001",
  "acceptance_evidence": ["基础文档骨架已建立；8 份 Markdown 的 29 条本地链接检查通过；需求正文缺失，完整验收未通过。"],
  "stop_reason": "分享正文无法加载，功能、目标应用与技术栈缺失；等待需求文本后继续相同任务。",
  "unplanned_reason": "首次项目初始化，需求正文尚未取得，暂不建立无依据的 Milestone。"
}
```

```tracking-execution
{
  "id": "T-001/E-001",
  "name": "建立工程与需求核对基线",
  "task_id": "T-001",
  "task_version": "v1",
  "status": "paused",
  "authorization": "2026-10-31 用户要求初始化现有项目目录，MVP 为 macOS Electron，遵守全局架构规范并设计可插拔边界。",
  "scope": "读取规范与分享需求；创建 Git 与最小项目骨架；记录架构边界、验证与需求缺口。",
  "steps": [],
  "evidence": [
    "起始目录为空，非 Git 仓库；已执行 git init -b main",
    "分享网页读取超时；浏览器访问与一次重试均显示无法加载对话",
    "8 份 Markdown 的 29 条本地链接检查通过",
    "首次 tracking 校验发现 scope/acceptance 应为文本而非数组；修正后 project-tracking-check 通过：task=1, execution=1, decision=1",
    "首次 make check 发现 gitignore 尾部空行；修正后 make check 通过",
    "review：已检查暂存差异、范围、文档路由、相对路径、模式适用边界及未实现声明；没有产品源码或目标应用修改",
    "基础骨架保存于本记录所在提交；没有构建、单元测试或真实应用注入证据"
  ],
  "stop_reason": "可独立完成的基础初始化已落盘；缺少分享需求正文，等待用户补充后恢复，未开始产品实现。"
}
```

#### 恢复上下文 · r2

- 绑定：T-001@v1 / T-001/E-001；2026-10-31；起始无 HEAD、无现有文件与未提交用户改动。
- 必读：[需求](requirements.md)、[架构边界](architecture/boundaries.md)、[D-001](DECISIONS.md#d-001)。
- 当前重点：基础初始化已落盘并完成内容 review；取得分享正文后核对功能、应用和技术栈，再确定产品工程形态。当前证据见上方 E-001；核对日期 2026-10-31。
- 受影响边界：未读取的分享不构成已知需求；不修改目标应用、不安装运行时、不选择注入机制。
- 退出条件：基础材料检查完成；需求仍缺失则保留阻塞并明确未交付的产品范围。
