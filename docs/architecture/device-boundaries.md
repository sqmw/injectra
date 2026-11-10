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
