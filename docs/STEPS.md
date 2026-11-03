# 执行步骤

活跃块：1。任务状态见[TODO](TODO.md)。

<a id="t-008"></a>
## T-008 · 长期注入架构基础重构 @v1

所有者：当前Parent；single-writer；2026-11-03。授权与任务定义见[任务入口](TODO.md#t-008)，恢复见[上下文](agent-context/tasks/T-008.md)。顺序为Web迁移→原生/平台契约→完整回归；每个阶段提交前可编译、review并同步文档。

### 迁移公共基础与Web执行路径

```tracking-step
{
  "id": "T-008/S-001",
  "name": "迁移公共基础与Web执行路径",
  "task_id": "T-008",
  "task_version": "v1",
  "status": "已完成",
  "evidence": [
    "core/Web/application/CDP迁移完成；离线cargo check和make check通过（11项Rust、5项Web）；review核对身份、页面能力、错误确定性与CLI投影；首次doc-check发现旧路径已同步，tracking缺number字段已补齐。"
  ],
  "number": 1
}
```

### 建立原生契约并拆分平台宿主

```tracking-step
{
  "id": "T-008/S-002",
  "name": "建立原生契约并拆分平台宿主",
  "task_id": "T-008",
  "task_version": "v1",
  "status": "未开始",
  "evidence": [],
  "number": 2
}
```

### 完整回归与架构验收收口

```tracking-step
{
  "id": "T-008/S-003",
  "name": "完整回归与架构验收收口",
  "task_id": "T-008",
  "task_version": "v1",
  "status": "未开始",
  "evidence": [],
  "number": 3
}
```

```tracking-execution
{
  "id": "T-008/E-001",
  "name": "迁移执行域并验证长期扩展契约",
  "task_id": "T-008",
  "task_version": "v1",
  "status": "running",
  "authorization": "2026-11-03用户PLEASE IMPLEMENT THIS PLAN，明确批准长期架构基础重构全部计划；选择骨架与契约验证，不做真实原生后端。",
  "scope": "core/Web/native/application契约及用例、CDP迁移、平台宿主拆分、兼容回归、架构与任务文档；不实现真实原生后端或跨平台兼容。",
  "steps": [
    "T-008/S-001",
    "T-008/S-002",
    "T-008/S-003"
  ],
  "evidence": [
    "基线990967b，工作区干净；现行MVP行为及历史证据保留，无冻结基线。"
  ],
  "stop_reason": ""
}
```

历史：[T-002执行与步骤](archive/T-002.md#t-002)。
