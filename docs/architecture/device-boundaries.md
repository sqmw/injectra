# 控制端与目标设备边界

属于[T-019](../TODO.md#t-019)，输入74844e7；自审，不冒充独立审核。

## 已确认发现

1. 共享connection.py直接执行本机ps，未来设备容易被当作控制端本机进程。
2. Rust Bridge的macOS条件未清楚区分控制端支持与目标支持。
3. 扁平v1会话混放pid、token、端口及ack_log，无设备/进程/路由配置边界。
4. 页面ID仅含局部编号/URL指纹，不同设备或进程可能产生相同ID。
5. WK执行直接依赖NSApplication枚举，通用引擎逻辑与Mac宿主绑定。

## 修复边界

保留crate依赖与core/Web公开契约。Frida内部拆分控制端运行、会话模型、目标核验provider、传输connector；当前仅注册Mac本机和认证回环Gadget TCP。Mac核对boot实例与进程世代，并验证目标Gadget回执；其他设备不得回退本机ps。

新会话v2分开device/process/connection/routes；v1按历史Mac本机生产者只读兼容，不修改原文件。页面ID绑定设备、进程及路由/profile，旧未限定ID须重新发现，不提供危险回退。

Mac启动工具保持显式Mac专用；WK单独提供Mac AppKit发现/调度适配器。Android/iOS、USB/ADB、原生界面动作、自动恢复不实现。

## 验收

未知设备/连接器在本机ps或连接前拒绝；不同设备同PID、不同进程同页面号不能互认；端口/token和控制端文件路径不充当目标身份；旧文件无损兼容；目标身份/回执不匹配拒绝。Mac独立Electron/WKWebView及默认CDP回归；测试替身仅存在于测试。[执行](../STEPS.md#t-019)。

会话/目标分离已实现：device包含provider/platform/instance_id，process为目标PID/不透明世代/目标路径/架构，connection为独立connector配置，routes为host profile与控制端回执选项。Mac实例以boot UUID摘要标识，目标agent独立回读核对。混合v1/v2身份声明拒绝；源v1文件不写回。

## 实际扩展接口

| 层 | 当前职责 | 后续新增设备需要实现 |
|---|---|---|
| controller.rs / session_store.py | 控制端Python执行、缓存及私有描述文件读取 | 不把控制端平台当作目标平台；新增控制端平台另验收 |
| TargetProvider | bind目标身份、核验认证通道及agent回执 | 提供目标设备自己的身份/进程世代来源，不能查询控制端PID冒充 |
| TransportConnector | 建立/关闭认证通道，负责连接地址 | 新通道单独注册，传输地址不当作设备身份 |
| RouteBinding/profile | 声明宿主实现及其控制端回执配置 | 新宿主明确实现和能力，不静默回退已有Mac代码 |
| WK host adapter | available、schedule、views；Mac实现独立 | 未来UIKit等实现自己的发现/调度，并满足同一回调所有权合同 |

Mac设备实例使用boot UUID摘要，进程世代是provider拥有的不透明值，目标可执行路径不在控制端做存在性解释。页面作用域不包含端口/token或控制端日志路径，连接参数变化不冒充目标身份变化。身份不匹配在用户载荷前拒绝。

CDP仍是显式协议端点适配器，不推定端口背后的物理设备，也不加入本机ps。未来设备接入需供给并验证其通道后再复用CDP；设备发现、手机授权/装载和生命周期策略仍需未来实现。本轮没有移动端枚举或启动命令。原生执行域已有目标地址/架构与不透明TargetRef，未发现需要把桌面假设写入core的理由。

WK宿主调度的同步异常表示未接受回调；已接受回调在超时后仍保持资源，尚未开始的迟到工作只排空、不执行载荷。回调分配失败会平衡已持有的view引用；已经提交到原生的操作仍保守Unknown。测试替身验证这些接口，不作为手机运行证据。

补充自审：重复JSON字段或混合v1/v2身份声明均在输入阶段拒绝，避免声明的设备被静默覆盖；损坏的UTF-8/JSON明确作为输入失败。原生回调分配失败平衡view持有；清理无法排队时不伪报closed。

最终检查、失败保留和恢复方案见[验收与自审](../validation/device-boundaries.md)。
