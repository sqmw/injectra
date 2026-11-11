# macOS启动观察器

属于[T-034](../TODO.md#t-034)，用于修正旧实验的观测缺口，不属于正式注入后端或应用管理功能。[阿里实测](../compatibility/adrive-startup.md)与工具自测分别取证。

## 入口与行为

[observe-macos-startup.py](../../scripts/observe-macos-startup.py)启动用户明确授权的单个应用实例；[startup_observation.py](../../tests/support/startup_observation.py)提供进程归属和只读HTTP探测。已有同路径实例时拒绝，不自动退出用户应用。

```sh
"${INJECTRA_FRIDA_PYTHON:-${INJECTRA_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/injectra}/frida/venv/bin/python3}" \
  -B scripts/observe-macos-startup.py --app "$TARGET_APP" --mode cdp --seconds 45
```

- `--mode cdp`增加回环调试参数，只查询指定端口的/json/list；`plain`只观察普通启动，不探测本地未知服务。
- `--home-mode isolated`为默认值，设置新HOME/CFFIXED_USER_HOME。它可能使目标重新初始化系统服务；不能把这种启动阻塞误判成注入入口关闭。
- `--home-mode system`保留控制端正常HOME、移除测试CFFIXED覆盖，Chromium profile与XDG目录仍指向本次缓存。**这是部分隔离**，应用会依自身逻辑访问当前用户的系统服务。工具不读取钥匙串内容、不修改授权或密钥设置；系统授权交互由用户处理。
- 请求配置只证明传入值，不是目标进程内读回。新字段configuredHome采用这个含义；早期run中的effectiveHome同样仅来自环境配置，不能当作目标实测HOME。
- 观察期默认40秒，可设5～120秒；单次HTTP使用控制端子进程实施3秒总截止，最多读取64KiB。超时只结束探针子进程，不结束目标。
- 启动PID退出后继续观察到窗口结束；记录父子/进程组、PID世代、原始响应及目标过滤理由，区分空列表、启动页、非page、HTTP错误和超时。
- **不提交任何载荷，也不自动清理目标进程树。** 观察结束可能保留测试实例，控制方须重新核对身份后正常结束，并恢复用户原现场。

## 证据与限制

缓存根runs/startup-observe-*保存request.json、startup.log、events.jsonl、http.jsonl和result.json。每次记录脚本/模块SHA及共同monotonic时间基准；重要实验另存相匹配的源码快照。HTTP数据可能含目标URL，目录权限0700，不进入Git。

HTTP请求前核对监听归属，完成后用下一份时间晚于回执完成的快照核对PID/世代；若观察先结束则归属未确认。进程stat和present/live分开，僵尸PID不当作存活宿主。早期记录没有stat或存在较窄时序缺口，保留原数据并按实测说明解释，不回写旧run。

归属依据含本次新进程组、观测到的父子关系、同可执行文件及确切测试profile。进程组完整消失后的编号重用、含空格命令行解析仍有局限；这些记录用于观察，**不能直接替代正式后端的目标身份核验，也不能据此任意执行载荷或终止未知进程**。

## 验证

`make test-frida`包含[边界单测](../../tests/frida/test_startup_observation.py)与[真实进程交接夹具](../../tests/frida/test_startup_observation_process.py)。后者是自有HTTP协议夹具，模拟启动器退出、子进程保持监听；不冒充Electron或阿里兼容性。

覆盖原始启动页保留、HTTP403正文、慢滴流总截止、进程重挂/PID复用/相似路径拒绝、HOME配置不修改父环境、僵尸与存活区分，以及真实交接后的持续观察。夹具只退出自身创建的HTTP服务，源码和日志保留仓库外。本轮38项Frida测试通过；不据此声明阿里页面注入成功。
