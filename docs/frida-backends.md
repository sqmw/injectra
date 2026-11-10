# 可选 Frida 页面后端

当前实现Electron Gadget与[WKWebView](webkit-backend.md)并列主页面后端；整体进展见[T-017](TODO.md#t-017)。正式产品不加载 references；历史桥接源码只用于来源核对。

## 依赖与入口

先运行 make setup-frida，准备仓库外锁定 Python 3.13 / Frida 17.25.2。CDP无需该环境。INJECTRA_FRIDA_PYTHON可以选择已准备的同版本解释器；INJECTRA_CACHE_DIR控制缓存根。Rust可执行文件内嵌Python/agent源码，按内容标识展开到外部缓存，不依赖编译机器源路径。

现有已加载Gadget的受控实例：用可选环境解释器运行 scripts/frida-session.py --state STATE_JSON --output SESSION_JSON。必须显式选择历史state；不自动追随latest或猜测进程。注册只核对身份，不启动/恢复/结束应用。会话文件含凭据，0600保存，不提交、不输出正文。

CLI使用：
- injectra targets --backend electron-gadget --session SESSION_JSON
- injectra apply --backend electron-gadget --session SESSION_JSON --target TARGET_ID --bundle MANIFEST
- injectra remove --backend electron-gadget --session SESSION_JSON --target TARGET_ID --id BUNDLE_ID

默认仍为CDP；原 --port 方式及JSON字段不变。后端选择显式，不在失败时自动换路线。

新实例启动是独立显式命令：scripts/frida-launch.py --app APP --gadget LIBRARY，使用同一可选解释器。只有用户授权启动目标后使用；已存在同可执行文件实例时拒绝，不退出旧进程。Gadget来源为官方17.25.2 macos-universal，安装准备见make setup-gadget。启动创建独立home/profile，验证实际PID和可执行路径，并在恢复前清除DYLD对子进程的继承。不能把独立profile等同于账号、钥匙串或全部系统副作用隔离。

## 支持边界与身份

当前实测覆盖抖音8.7.0的Electron36.4.0-rs及独立Electron36.4.0。Electron45.2.1缺少HandleScope导出，明确Unsupported/NotExecuted，不宣称全Electron支持。

T-019将控制端与目标分开：会话v2分别保存device/process/connection/routes，新注册输出v2，旧v1按历史Mac本机会话只读兼容，不写回原文件。每次由Mac目标provider核对设备boot实例、进程世代、路径与架构，并用目标agent回执交叉确认。页面ID绑定设备/进程/路由作用域，再包含webContents身份、URL及主frame指纹。升级后旧未限定页面ID需重新运行targets，不提供不安全回退。执行前再次核对。只覆盖主页面，不承诺同URL同frame内的文档世代锁定或导航期间执行原子性。

返回值等待Promise并转JSON；脚本异常为Executed，超时/断线为Unknown且不重放，缺目标/ABI不支持为NotExecuted。消息按随机请求ID关联，读取Node日志只从本请求起点开始。

## 生命周期与证据

每次命令的Python助手负责连接和原生回调清理，取得uv_close回执后才卸载agent。初始化期间同样pin，关闭请求可取消尚未进入主线程的初始化。CLI异常退出不主动杀助手；助手继续有界收尾。

SDK操作15秒/清理5秒可取消截止，25秒助手截止后CLI报告Unknown并保留外部请求/诊断位置，不强杀可能持有原生回调的进程。若目标主线程永久阻塞或Python助手本身崩溃，不能保证无残留；此类状态必须检查，禁止把它报告为清理成功。正常重复连接、超时后再连接由实机测试覆盖；独立fixture已验证终止Rust CLI后助手仍完成请求与清理。

请求与回执在外部frida/requests，每次独立目录0700；可能包含载荷或结果，按运行数据管理，不入Git。会话内凭据不复制到请求日志。

## 检查

- make check：默认Rust/Web及WK宿主合同回归，不接入用户应用。
- make test-frida：可选Python会话校验测试，不接入应用。
- make setup-gadget-tests：独立Electron36.4.0测试依赖，锁文件在tests/gadget。
- make test-gadget：创建和退出自己的Electron实例，INJECTRA_TEST_GADGET指定已校验库文件。
- 显式真实页面测试：设置INJECTRA_TEST_SESSION和INJECTRA_TEST_TARGET后运行cargo test -p injectra-electron-gadget --test live -- --ignored。会短暂添加并移除测试自身DOM，不点击应用业务控件。

当前结果及阶段退出条件见[Mac交付](macos-delivery.md)、[执行证据](STEPS.md#t-017)。

内部扩展接口与当前支持范围见[控制端/设备边界](architecture/device-boundaries.md)：目前只注册Mac本机target provider及认证回环Gadget TCP connector，其他设备与连接器明确Unsupported。Rust平台检查仅表示本地Python控制端当前认证的平台，不代表手机端实现。

每次helper将阶段名和相对耗时写入外部诊断stderr，不含会话token；截止错误带当前阶段，保持15秒操作/5秒清理/25秒助手边界及无自动重放语义。
