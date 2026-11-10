# 执行步骤

活跃块：1。任务状态见[TODO](TODO.md)。

<a id="t-017"></a>
## T-017 @v1

所属任务：[T-017](TODO.md#t-017)。所有者：本窗口/root；single-writer；2026-11-10用户批准完整计划。
意图：在现有分层内实现Mac多引擎主页面注入。阶段顺序按用户批准计划；应用启动/系统保护边界保持。

### 1. 验收基线

基线make check与Electron实测通过，明确新旧证据和范围。

```tracking-step
{
  "id": "T-017/S-001",
  "name": "验收基线",
  "task_id": "T-017",
  "task_version": "v1",
  "number": 1,
  "status": "已完成",
  "evidence": [
    "独立缓存make check通过；真实Electron45.2.1/Chromium156的10项检查通过，run electron-3ZCwg9。文档跨仓相对链接已改为不依赖目录布局的引用。"
  ]
}
```

### 2. Gadget正式后端

CLI+独立Electron+抖音真实页面验收，资源/错误合同齐全。

```tracking-step
{
  "id": "T-017/S-002",
  "name": "Gadget正式后端",
  "task_id": "T-017",
  "task_version": "v1",
  "number": 2,
  "status": "已完成",
  "evidence": [
    "正式Gadget后端真实抖音8.7.0全组通过；独立Electron36.4.0全组通过，run gadget-electron-us6ccbwv；Electron45缺HandleScope符号在执行前拒绝。",
    "make check、4项Python会话校验及CDP Electron10项回归通过；自审发现并修正CLI缺省参数约束和结果读取失败确定性。",
    "CLI抖音apply/update/remove通过；独立fixture控制端退出后助手仍完成并关闭回调，run gadget-electron-c5qkiq_k。正式会话拒绝符号链接；自审与定向回归通过。"
  ]
}
```

### 3. Chromium宿主覆盖

CEF和NW.js独立宿主完整CDP验收。

```tracking-step
{
  "id": "T-017/S-003",
  "name": "Chromium宿主覆盖",
  "task_id": "T-017",
  "task_version": "v1",
  "number": 3,
  "status": "已完成",
  "evidence": [
    "CEF156.0.26和NW.js0.119.2官方Mac arm64独立宿主的共享主页面合同通过；run cef-qdvrcqtq/nwjs-ypuq07xm。",
    "首次循环对象断言过强失败保留；按既有CDP协议错误Unknown合同修正测试，未更改产品错误语义。来源与校验清单、恢复入口及自审完成。"
  ]
}
```

### 4. WebKit正式后端

独立WKWebView与至少一个真实应用通过。

```tracking-step
{
  "id": "T-017/S-004",
  "name": "WebKit正式后端",
  "task_id": "T-017",
  "task_version": "v1",
  "number": 4,
  "status": "已完成",
  "evidence": [
    "独立双WKWebView合同通过，run webkit-i07adzl2；CLI与控制端退出后助手清理通过，run webkit-lg0f0zoc。",
    "用户明确批准的空HOME/CODEX_HOME Codex Switcher0.2.11真实主页面合同、托盘隔离和CLI通过，t017/webkit-real-contract.log；未导入账号或切换，测试新实例按身份退出。",
    "Easydict接入尝试失败保留，未修改签名/系统保护/Clash运行；新增SDK可取消截止与6项会话单测通过，真实WebKit通过包含该截止实现。"
  ]
}
```

### 5. Mac交付验收

全套回归、review、文档和提交完成，不提前声称全平台支持。

```tracking-step
{
  "id": "T-017/S-005",
  "name": "Mac交付验收",
  "task_id": "T-017",
  "task_version": "v1",
  "number": 5,
  "status": "进行中",
  "evidence": []
}
```

```tracking-execution
{
  "id": "T-017/E-001",
  "name": "建立验收基线并完成Mac后端集成",
  "task_id": "T-017",
  "task_version": "v1",
  "status": "running",
  "authorization": "2026-11-10用户Implement the plan；已确认并列后端、Python/Frida可选依赖、Mac主页面、Chromium+WebKit。；用户另明确批准Codex Switcher独立空目录真实验收，不导入账号或切换。",
  "scope": "保留现有核心契约与共享runtime；新增并列后端、共享Frida桥接、CLI选择、锁定可选环境、CEF/NW.js/WKWebView验证；不推进Windows/Gecko/iframe/自动重注入，不改第三方安装包/系统保护，不自动重启用户应用。",
  "steps": [
    "T-017/S-001",
    "T-017/S-002",
    "T-017/S-003",
    "T-017/S-004",
    "T-017/S-005"
  ],
  "evidence": [
    "基线make check与Electron10项通过；日志位于外部缓存t017/baseline.log和electron-baseline.log。",
    "Gadget正式CLI、真实抖音/独立Electron36、控制端退出清理、默认CDP回归与4项会话单测通过。",
    "CEF/NW.js官方独立宿主已通过；开始WebKit并列后端。",
    "WebKit并列后端、独立双页与真实Tauri应用合同/CLI通过；进入Mac最终回归。"
  ],
  "stop_reason": ""
}
```

历史：[T-008](archive/T-008.md#t-008) · [T-002](archive/T-002.md#t-002)。
