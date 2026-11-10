# 抖音 Frida 原生函数 Hook 验证

2026-11-11，[T-025](../TODO.md#t-025)，基线a92a55c。用户明确要求新方案，不重复旧Gadget/Node页面验证。本轮目标是原生Interceptor挂接、调用回调与卸载，目标函数abs只观测，不改参数或返回值。

## 结论与分层

当前抖音8.7.0原包下的新路线未通过，均在Hook执行前被阻止。不能写成Interceptor已经在抖音执行后失败，也不能用既有页面桥接成功替代本轮结果。

| 对象/路线 | 实际结果 | 证据边界 |
| --- | --- | --- |
| 抖音8.7.0，外部Frida attach | PermissionDeniedError：当前用户无法访问本次PID | 未加载agent，未执行Interceptor |
| 抖音8.7.0，启动加载Gadget optional | Gadget监听建立前进程被终止；SIGKILL/Code Signature Invalid，CODESIGNING code2 Invalid Page | 未进入Hook；属于系统代码签名终止证据，不推断为抖音主动反Frida |
| 自有Swift空进程，Gadget optional | 同一agent收到abs(-7)、abs(-13)两次回调，返回7/13；卸载后abs(-19)返回19且回调数不再增加 | 证明当前本机Interceptor/脚本在该自有调试目标可工作，不证明抖音支持 |
| 自有进程外部attach对照 | 操作截止取消；含已READY的Python、自建可调试Swift进程 | 本机外部attach链路仍有未定位问题；不能仅归因抖音某一个entitlement |

旧方案是Gadget required模式调用原V8/Node页面入口；按[Frida官方配置说明](https://frida.re/docs/gadget/)，required模式不提供Interceptor API。本轮没有重跑其页面注入合同，既有支持状态继续见[原证据](douyin-gadget.md)和[当前矩阵](current-matrix.md)。

## 实机身份与保护边界

目标安装位置为系统Applications中的抖音.app；8.7.0，bundle id com.bytedance.douyin.desktop，声明LSMultipleInstancesProhibited。启动前没有目标可执行文件实例，测试使用新HOME/CFFIXED_USER_HOME和Chromium profile；只结束Popen拥有的本次进程。

签名中有allow-dyld-environment-variables、allow-jit、cs.debugger和disable-library-validation，未发现get-task-allow。taskgated-helper同次日志记录对该bundle的no eligible provisioning profiles拒绝。这与[Apple调试权限说明](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.debugger)相符，但不据此声称已唯一定位全部本机attach截止原因。

两次抖音测试前后主可执行文件SHA256一致：223801802e24cf36c25c0a1667e3e8435129e9316586256f5b62d4762abbd466；测试后codesign --verify --deep --strict通过。这里只证明主可执行文件和签名状态，不宣称整个安装包逐字节全程不变。

未修改抖音签名、entitlements、安装包或系统保护；没有登录、读取账号/视频数据或点击业务控件。Gadget optional是本次独立代理的官方运行模式，不关闭macOS的代码签名执行保护；实际正被该保护终止。自建空Swift控制程序在仓库外签为可调试目标，未给抖音添加调试权限。

## 运行证据

原始资料位于仓库外默认缓存根runs目录，各结果含PID和自有进程退出信息：

| run | 结果 |
| --- | --- |
| douyin-native-hook-hvbjeole | 抖音外部attach被PermissionDenied拒绝；主可执行文件哈希与签名检查通过；自有PID28523退出 |
| frida-native-control-8wy7tczo | Python空进程对照15秒操作截止；未分段记录到唯一阶段，未取得Hook回执；PID29503退出 |
| frida-native-ready-control-3vzp6z_r | 已READY的Python空进程仍在attach截止；PID30084退出 |
| frida-native-debug-control-dkx_knph | 自建Swift空进程，get-task-allow仅赋予该fixture；attach仍截止；PID30979退出；fixture源码、签名配置和二进制保留 |
| frida-interceptor-control-qu11flea | 自有Swift目标经Gadget optional取得完整Interceptor回调和卸载；PID32020退出 |
| douyin-interceptor-gadget-ztqu45cw | 抖音Gadget optional在listener阶段退出，PID32972；系统报告抖音-2026-11-11-005803.ips对应同PID，CODESIGNING/Invalid Page；主文件哈希与签名检查通过 |

控制正例已执行的agent与抖音Gadget尝试记录的待加载agent具有同一SHA256（后者未实际加载）：76f98a37659f91749261d0f9f647fb5b897488a5636d0f92debfbc73e6e4377e。代理只Hook纯计算函数，不读取业务数据或改返回值；对照的codeSigningPolicy为optional、QJS、arm64。Gadget来源是锁定官方17.25.2，压缩包SHA256核对后解压；认证token只保存在0700 run内的0600配置中，不写文档。

## 复核入口与限制

[外部attach验证器](../../scripts/frida-hook-smoke.py)和[最小Interceptor agent](../../tests/frida/native-hook.js)作为可选测试工具保存，不注册成正式产品后端，不属于可复用插件Bundle。

DOUYIN_APP显式指定本机待验应用包路径。

```sh
# 仅在明确要新建并退出测试实例时运行；目标已经运行则拒绝。
"${INJECTRA_FRIDA_PYTHON:-${INJECTRA_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/injectra}/frida/venv/bin/python3}" \
  -B scripts/frida-hook-smoke.py --app "$DOUYIN_APP"
```

操作截止默认15秒，可用--deadline 2做快速截止验证；清理截止5秒。外部attach错误不会自动降级为Gadget，Gadget optional尝试独立记录。可选环境沿用项目锁定Python3.13/Frida17.25.2，没有升级或更换现有依赖。

自审确认本轮没有运行旧页面成功套件、修改产品crates或共享Web runtime；结果按接入、agent、Interceptor三阶段区分。新脚本语法及make check/test-frida回归单独记录，不能把回归通过写成抖音Hook通过。没有sudo、系统权限更改或已安装应用重签名；本轮可行性验证得到否定/受限结论，真实抖音Interceptor能力尚未交付。

最终自审与回归：make check通过18 Rust、5 Web、8 WK宿主、3 Gecko HTML；make test-frida通过23项Python。新runner的help/AST及agent语法通过；6个本次自有PID均已退出且ps复查不存在。审阅确认不改返回值、finally卸载listener、拒绝已有用户实例、证据与产品能力分开。抖音新方案依然未通过。
