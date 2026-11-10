# 执行步骤

活跃块：1。任务状态见[TODO](TODO.md)。

<a id="t-017"></a>
## T-017 关闭入口

[macOS多引擎执行与验收](archive/T-017.md#t-017)。

历史：[T-008](archive/T-008.md#t-008) · [T-002](archive/T-002.md#t-002)。

<a id="t-019"></a>
## T-019 @v1

[任务](TODO.md#t-019)；本窗口/root、single-writer；输入74844e7且工作区干净。用户授权review并解决扩展边界，排除手机实现。

### 1. 审查与边界

```tracking-step
{
  "id": "T-019/S-001",
  "name": "审查与边界",
  "task_id": "T-019",
  "task_version": "v1",
  "number": 1,
  "status": "已完成",
  "evidence": [
    "源码审查完成，见architecture/device-boundaries.md。"
  ]
}
```

### 2. 会话与目标解耦

```tracking-step
{
  "id": "T-019/S-002",
  "name": "会话与目标解耦",
  "task_id": "T-019",
  "task_version": "v1",
  "number": 2,
  "status": "已完成",
  "evidence": [
    "分域v2模型、Mac本机target provider与Gadget TCP connector完成；设备boot+进程世代及目标回执核验；不透明页面ID限定设备/进程/路由。",
    "18项会话/作用域/注入拒绝测试通过；真实旧v1抖音会话发现7页且原文件SHA未变；新v2独立Electron36全合同、CLI及控制端退出清理通过，run gadget-electron-g4_pknh3。",
    "补审拒绝v1/v2混合身份字段，目标路径/世代保持provider拥有；不加入手机实现。"
  ]
}
```

### 3. 宿主发现分离

```tracking-step
{
  "id": "T-019/S-003",
  "name": "宿主发现分离",
  "task_id": "T-019",
  "task_version": "v1",
  "number": 3,
  "status": "已完成",
  "evidence": [
    "WK入口只装配引擎和Mac宿主；macos_appkit提供available/schedule/views，webkit_agent不依赖NSApplication。",
    "6项宿主合同测试通过，覆盖非桌面测试宿主、主线程/page world、错误目标、调度拒绝、超时持有原生回调及迟到排队工作不启动载荷。",
    "真实独立双WKWebView v2会话、完整页面合同、CLI及控制端退出清理通过，run webkit-djwsgq6k；无手机profile注册。"
  ]
}
```

### 4. Mac回归与交付

```tracking-step
{
  "id": "T-019/S-004",
  "name": "Mac回归与交付",
  "task_id": "T-019",
  "task_version": "v1",
  "number": 4,
  "status": "进行中",
  "evidence": [
    "最终18 Rust/5 Web/8 WK host/23 Python测试通过；CDP Electron45、Gadget Electron36和双WKWebView独立宿主通过，详见validation/device-boundaries.md。",
    "保留webkit-2fclaktz的targets超时；新增阶段诊断和fixture ready，webkit-5jucn2z7通过，未加长截止或放宽身份。"
  ]
}
```

```tracking-execution
{
  "id": "T-019/E-001",
  "name": "边界审查、内部解耦与Mac回归",
  "task_id": "T-019",
  "task_version": "v1",
  "status": "running",
  "authorization": "用户本轮要求review并解决扩展边界，同时不实现手机端；现有分层保留。",
  "scope": "Frida内部控制端/会话/目标provider/连接器分离；v2会话并兼容v1；页面ID加入设备进程作用域；Mac宿主发现与WK执行分离；测试和文档。不新增crate、不改core/Web公开契约、CLI命令，不实现手机/USB/ADB或自动恢复。",
  "steps": [
    "T-019/S-001",
    "T-019/S-002",
    "T-019/S-003",
    "T-019/S-004"
  ],
  "evidence": [
    "确认本机ps/目标混用、扁平会话、页面ID跨会话歧义与WK/AppKit耦合。",
    "会话/目标分离完成并通过旧v1及新v2独立Electron验证，正在分离WK宿主发现。",
    "WK执行与Mac宿主发现分离；6项宿主合同和真实双WKWebView回归通过。",
    "最终回归通过；输入重复字段/格式失败、启动器Mac限定及WK回调失败防御完成，正在提交与关闭。"
  ],
  "stop_reason": ""
}
```
