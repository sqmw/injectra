# 执行步骤

活跃块：1。任务状态见[TODO](TODO.md)。

<a id="t-032"></a>
## T-032 · macOS 十款应用顺序注入验证 @v1

所属[Task](TODO.md#t-032)，Parent/main，single-writer，2026-11-11。依用户给定顺序推进；每步以真实注入/清理成功或具体阻塞证据结束尝试，局部缺授权不推定其他步骤取消。所有实际启动串行，自有实例验证后退出；已运行用户实例不强杀。

```tracking-execution
{
  "id": "T-032/E-001",
  "name": "逐应用接入、注入与清理验证",
  "task_id": "T-032",
  "task_version": "v1",
  "status": "paused",
  "authorization": "2026-11-11用户指定阿里云盘、百度网盘、企业微信、千问、Anytype、Claude、Xmind、腾讯视频、网易云音乐、QQ依次尝试；允许必要的受控新实例测试，已有状态按全局保留约束处理。",
  "scope": "当前Apple Silicon本机应用；依次尝试CDP/已有原生页面后端，检查独立目录与已有实例；只用自有临时UI载荷验证并清理，保留账号/同步任务/文档；不改签名/系统保护或未授权结束已有进程。",
  "steps": [
    "T-032/v1/S-1",
    "T-032/v1/S-2",
    "T-032/v1/S-3",
    "T-032/v1/S-4",
    "T-032/v1/S-5",
    "T-032/v1/S-6",
    "T-032/v1/S-7",
    "T-032/v1/S-8",
    "T-032/v1/S-9",
    "T-032/v1/S-10",
    "T-032/v1/S-11"
  ],
  "evidence": [
    "基线8c041ce，工作区干净。已读旧矩阵和代码地图；元数据核对发现千问/Claude版本变化，Claude/Xmind/QQ已有用户实例。",
    "用户明确批准正常退出QQ后测试；CUA正常退出原PID653，再以新目录测试CDP及Node入口均未取得目标，随后重新打开普通QQ窗口。",
    "阿里云盘无调试参数对照adrive-no-debug-jumvlj3h保持运行12秒；后续Node入口试验在启动前检测到新普通实例PID69081而拒绝，未擅自结束它；用户授权待回复。",
    "首轮10款已依次尝试；五款在已选页面上通过同一CDP/共享runtime合同：百度、千问、Anytype、Xmind、网易云。应用范围和原始run见validation/macos-ten-apps.md。",
    "企业微信与腾讯视频复核本次进程树的实际监听端口，仍无CDP页面；QQ经用户明确批准正常退出后分别尝试CDP/Node，均无入口，已重新打开普通QQ窗口。",
    "阿里无调试参数对照可运行12秒；Node尝试在启动前发现新普通实例PID69081而拒绝，已询问用户是否允许正常退出，尚无回复。不以时间流逝当授权，不结束该实例。",
    "复查记录的测试PID均已退出；原Claude/Xmind用户PID680/674存在，普通QQ已重新打开为PID69729。千问新生成日志已保留在缓存并恢复严格签名；阿里/QQ/Xmind既有严格签名异常分开记录。",
    "自审确认成功只覆盖实际页面，指定profile未创建的CEF宿主不声称完整数据隔离。未修改产品源码、未新增按App注入器。原verify-live实测及新增诊断JS语法、文档/追踪/差异检查通过。"
  ],
  "stop_reason": "独立工作已推进，阿里Node补充验证等待用户对新普通实例的正常退出授权；原10项结果及具体接入限制已记录。"
}
```

### Step 1：阿里云盘

```tracking-step
{
  "id": "T-032/v1/S-1",
  "task_id": "T-032",
  "task_version": "v1",
  "number": 1,
  "name": "阿里云盘接入与注入验证",
  "status": "已完成",
  "evidence": [
    "阿里云盘6.8.0：adrive-hhdaz6y5和adrive-ls-2hlr6yg2分别直接/LaunchServices独立HOME及profile启动，CDP监听日志出现，但主进程在取得page前退出；未执行载荷。主文件哈希一致，严格签名检查报既有扩展属性错误（第二次前后均失败），未修改签名。接入未完成，不认定永久不兼容。"
  ]
}
```

### Step 2：百度网盘

```tracking-step
{
  "id": "T-032/v1/S-2",
  "task_id": "T-032",
  "task_version": "v1",
  "number": 2,
  "name": "百度网盘接入与注入验证",
  "status": "已完成",
  "evidence": [
    "百度网盘8.2.7：baidu-taufvr_h启动data页面合同通过；另起baidu-page-hsaszip3排除启动页，实际file应用页面通过JS/CSS/HTML、点击0→1、更新、remove及无节点残留，证据live-kgi4Iz。使用独立HOME/profile，自有实例退出，哈希/签名通过；不外推账号/同步业务。"
  ]
}
```

### Step 3：企业微信

```tracking-step
{
  "id": "T-032/v1/S-3",
  "task_id": "T-032",
  "task_version": "v1",
  "number": 3,
  "name": "企业微信接入与注入验证",
  "status": "已完成",
  "evidence": [
    "企业微信5.0.9：wecom-s_xhx2gr使用自身容器Caches中的新HOME/profile受控启动，日志确认CEF renderer启动，但35秒未取得指定CDP端口的可用page；未提交载荷。原包哈希/签名保持，退出测试主进程及其自有子进程；不将无页面推断为所有接入路线不支持。"
  ]
}
```

### Step 4：千问

```tracking-step
{
  "id": "T-032/v1/S-4",
  "task_id": "T-032",
  "task_version": "v1",
  "number": 4,
  "name": "千问接入与注入验证",
  "status": "已完成",
  "evidence": [
    "千问当前5.1.7.296：qianwen-or0rz76r真实chrome://newtab页面通过JS/CSS/HTML、交互、更新和清理，live-vcTKtm；应用启动在签名资源目录新增agent_runtime_debug.log导致初始最终签名校验失败。确认日志出生晚于run且无进程/打开句柄后移至该run保留，未删除数据；post-cleanup确认签名恢复，主程序哈希未变。"
  ]
}
```

### Step 5：Anytype

```tracking-step
{
  "id": "T-032/v1/S-5",
  "task_id": "T-032",
  "task_version": "v1",
  "number": 5,
  "name": "Anytype接入与注入验证",
  "status": "已完成",
  "evidence": [
    "Anytype0.55.4：anytype-6vg_d15u实际file页面经CDP通过CSS/HTML/JS、交互、更新和清理，live-emY5Bs；独立HOME/profile，现有实例起初为空；原文件哈希/签名保持，自有进程退出。"
  ]
}
```

### Step 6：Claude

```tracking-step
{
  "id": "T-032/v1/S-6",
  "task_id": "T-032",
  "task_version": "v1",
  "number": 6,
  "name": "Claude接入与注入验证",
  "status": "已完成",
  "evidence": [
    "Claude当前4.23118.1：claude-n0ney_g8独立HOME/user-data-dir尝试，应用明确输出refusing to start — a debugging or network-override switch is present并退出1，未获得CDP页面/未提交载荷。原用户PID680仍在，包哈希/签名保持。源码同样存在调试参数拒绝及开发者授权检查；未绕过该检查。"
  ]
}
```

### Step 7：Xmind

```tracking-step
{
  "id": "T-032/v1/S-7",
  "task_id": "T-032",
  "task_version": "v1",
  "number": 7,
  "name": "Xmind接入与注入验证",
  "status": "已完成",
  "evidence": [
    "Xmind24.01.13311：源码核对userData由传入profile派生Electron v3目录，单实例锁按该目录；xmind-n4miorqh并行新实例welcome.html通过CSS/HTML/JS、交互、更新、清理，live-Yxmns7。原PID674保留，测试新实例退出；主程序哈希保持，严格签名检查前后均失败（既有状态），不做包修复。"
  ]
}
```

### Step 8：腾讯视频

```tracking-step
{
  "id": "T-032/v1/S-8",
  "task_id": "T-032",
  "task_version": "v1",
  "number": 8,
  "name": "腾讯视频接入与注入验证",
  "status": "已完成",
  "evidence": [
    "腾讯视频2.164.0（QQLive）：tencentvideo-q4qo46_5在自身容器Caches新建HOME/profile启动，实际Web/未登录视图日志出现，但35秒未取得指定CDP入口可用page；未执行载荷。自有进程退出，主文件哈希/签名保持；不能据此断言CEF全部路线不可用。"
  ]
}
```

### Step 9：网易云音乐

```tracking-step
{
  "id": "T-032/v1/S-9",
  "task_id": "T-032",
  "task_version": "v1",
  "number": 9,
  "name": "网易云音乐接入与注入验证",
  "status": "已完成",
  "evidence": [
    "网易云音乐3.1.5：netease-a3gaqwn6在自身容器Caches新HOME/profile下，orpheus://orpheus实际页面通过CSS/HTML/JS、交互、更新和清理，live-5Kv77m；原包哈希/签名保持，自有实例退出。"
  ]
}
```

### Step 10：QQ

```tracking-step
{
  "id": "T-032/v1/S-10",
  "task_id": "T-032",
  "task_version": "v1",
  "number": 10,
  "name": "QQ接入与注入验证",
  "status": "已完成",
  "evidence": [
    "QQ6.9.86：原PID653已有9210/4001/4301/4310均未返回CDP，直接Frida attach PermissionDenied。用户随后明确批准正常退出QQ，CUA Cmd+Q确认App quit；qq-r_suqmov在自身容器Caches新HOME/profile重启测试40秒，仍无可用CDP页面，未执行载荷；新实例退出。主文件哈希未变，严格签名前后均为既有扩展属性错误。首轮10项顺序尝试完成，失败项按新证据作后续定位。"
  ]
}
```

### Step 11：剩余入口补查与整体验收

首轮完成后对失败项作有依据的补查，不重写原run；当前恢复动作是等待阿里普通实例退出授权，再核验身份并继续Node入口试验。

```tracking-step
{
  "id": "T-032/v1/S-11",
  "task_id": "T-032",
  "task_version": "v1",
  "number": 11,
  "name": "剩余入口补查与整体验收",
  "status": "受阻",
  "evidence": [
    "前10项首轮顺序不变，按首轮失败的新证据继续补查：阿里无调试对照、QQ Node入口、企业微信/腾讯视频实际监听核验已完成。",
    "剩余阿里Node入口试验因新普通实例占用需用户确认退出；授权问题已发出，不重复请求。其余结果已写入验证报告和兼容性总表，保留所有原始失败。"
  ]
}
```

<a id="t-017"></a>
## T-017 关闭入口

[macOS多引擎执行与验收](archive/T-017.md#t-017)。

历史：[T-008](archive/T-008.md#t-008) · [T-002](archive/T-002.md#t-002)。

<a id="t-019"></a>
## T-019 关闭入口

[控制端与目标设备边界解耦](archive/T-019.md#t-019)。
