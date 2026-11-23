# 豆包真实应用兼容性

> 本页保留单项/批次证据；当前跨版本、平台与路线的状态统一查看[兼容性总表](current-matrix.md)。


## 结论与身份

2026-10-31，macOS arm64，本机安装的豆包 **2.33.5**（bundle id `com.bot.pc.doubao`）。
安装结构与实际进程显示使用内嵌 `Doubao Browser.app` / `Doubao Browser Framework`，CDP报告 `Chrome/147.0.7727.149`。这是定制Chromium宿主，不应计作标准Electron应用覆盖。

**现有Injectra核心链路在豆包真实聊天页面验证通过**，未修改core、payload、cdp或平台源码。接入条件是先退出全部主进程，再给安装包主入口传入回环调试参数；已有进程驻留时，新实例请求未使调试参数生效。

## 探测与实测证据

| 检查 | 结果 |
| --- | --- |
| 标准Electron launch-plan | 明确拒绝custom bundle，符合当前设计 |
| 旧进程现有两个本地监听 | 未发现CDP入口 |
| 独立profile启动主入口、内嵌浏览器入口 | 两次均退出，未开放CDP；不推定具体退出根因 |
| 获用户批准后的正常重启 | 菜单退出后进程驻留；发送TERM并等待退出，再启动成功 |
| 调试端口归属 | lsof确认豆包浏览器监听127.0.0.1，非全网地址 |
| Injectra CLI targets | 找到豆包launcher、chat、background三个page，仅选择chat |
| 真实页面 | `doubao://doubao-chat/chat`；没有读取聊天正文或操作账号功能 |
| CSS / HTML | 样式与容器共2个受管理节点；按钮计算背景色rgb(76,85,213) |
| JS交互 | 测试按钮计数0→1，仅修改测试面板自己的状态 |
| 相同ID更新 | 返回updated；节点仍为2，计数重置为0 |
| remove / 重复remove | 返回removed，节点为0；随后返回absent |
| 恢复 | TERM退出本次调试主/浏览器进程，以无参数主入口重新启动；原调试端口不可达 |

所有apply/update/remove均调用已编译的Rust CLI；辅助CDP只断言自有节点、计算样式和捕获自有面板截图，不替代产品注入路径。

原始证据保存在仓库外默认缓存根下：

- `runs/doubao-fhSOUG/`：启动日志、发现元数据与恢复信息；包含本地运行信息，不提交Git。
- `runs/live-0obxkB/`：首次完整链路通过；报告的route为null，是JS URL对自定义scheme的origin表示，非产品注入失败。
- `runs/live-5yq0GM/`：修正验证报告的route表示后复测通过，`result.json`与`injected-panel.png`；截图只含自有面板，不含聊天内容。

## 可复现入口与安全恢复

对已授权且开放CDP的目标，先用CLI targets核对页面ID，再运行：

```sh
make verify-live ARGS="--port PORT --target EXACT_PAGE_ID"
```

脚本创建唯一bundle ID，在仓库外保存临时载荷与结果；自动移除自己的面板，异常路径也尝试清理。它不自动启动、退出或重新配置目标应用，不进入常规make check，避免意外接入真实会话。

本次豆包主入口相对安装bundle的路径为`Contents/MacOS/Doubao`；参数为`--remote-debugging-port=<PORT> --remote-debugging-address=127.0.0.1`。需确保旧进程实际退出，再核对监听地址与页面身份。标准launch-plan仍只支持标准Electron，不静默扩张其识别规则。

## 证据边界与review

结论仅覆盖上述版本和真实chat页面的独立面板注入；没有证明修改豆包原有控件、跨frame、导航恢复、所有窗口或全部Chromium应用兼容。未修改安装包、签名和账号数据，也未发送消息。

review核对：原CLI执行路径、端口进程归属、唯一ID/局部选择器、截图隐私范围、异常清理及无调试参数恢复。保留失败日志，不把接入前置失败改写为核心Bug。任务与执行见[任务入口](../TODO.md#t-003)。


<a id="windows-plugins"></a>
## Windows 双插件应用（2026-11-13）

[T-045](../TODO.md#t-045)，win-lenovo实际安装的豆包，定制Chromium147；不把内核版本当作豆包应用版本。用户明确允许正常退出并重启：关闭窗口仅隐藏托盘，随后调用豆包托盘“退出”菜单，确认旧进程结束，未强制结束。沿用原User Data重启，监听仅127.0.0.1，核对监听归属豆包。

使用Windows本机Injectra Rust CLI，对外部插件库eeaca6a的原Bundle执行apply，没有改注入器或插件源码：

| 插件 | 精确页面作用域 | 实测 |
| --- | --- | --- |
| doubao.quick-themes 0.1.1 | doubao-launcher/chat快捷窗口 | applied，API与自有控件存在，默认sakura；计算背景rgb(255,241,246)，文字rgb(74,36,53) |
| doubao.paper-focus 0.3.0 | doubao-chat/chat主聊天页 | applied，API与自有控件存在；mode=waiting-for-pdf，未打开PDF，不据此宣称Windows阅读布局验证通过 |

快捷窗口经当前Windows快捷操作打开。CDP发现报告doubao://，页面location规范化为chrome://；按精确目标与hostname分别选择，未把主题注入聊天/后台页。未读取聊天正文、账号凭据或发送消息；主题沿用自身已有配色偏好持久化。插件和App会话保留供用户使用；页面重建/刷新或App重启后需重新注入，不增加后台重注入服务。

部署Bundle四文件SHA256匹配源库；运行数据位于仓库外doubao-windows/apply-53d21ba01182，包含deployment.json、verification-final.json及会话恢复信息。Windows缓存是发布产物，不是源码第二份维护入口。Review检查页面选择、双插件互不串页、调试回环地址、原数据模式及仅正常退出的授权边界。代码未改，无新增代码测试；通过实际CLI结果/状态/计算样式、文档与任务检查验收。

同日E-002快速重注入未重启App，两项均返回applied；随后只读检查论文状态已为reading，主题沿用用户选择的薄荷（mint），背景rgb(234,246,240)。未读取文档内容，也未追加选区菜单/拖拽/布局全面验收；初次等待PDF的历史记录保留。

## macOS 双插件验证（2026-11-20）

| 目标页面 | 插件版本 | CLI 回执 | 应用后的实测结果 |
| --- | --- | --- | --- |
| 快捷窗口（launcher） | 主题 0.1.1 | `applied` | 当前主题为 `midnight`，页面计算背景色为 `rgb(22, 30, 48)`。 |
| 聊天页（chat） | 论文阅读 0.3.0 | `applied` | 插件状态为 `waiting-for-pdf`，Mid 尺寸为 386 × 537。本次未打开 PDF。 |
