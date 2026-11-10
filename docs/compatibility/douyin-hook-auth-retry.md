# 抖音 Frida Interceptor 授权重试

2026-11-11，[T-027](../TODO.md#t-027)，基线fb93d18。用户指出前次离开期间未输入系统授权密码，明确要求重试[T-025](douyin-native-hook.md)的两条失败路线。历史run保持原样，本轮未重跑旧页面注入或Objective-C分派方案。

## 本轮结果

| 路线 | 新结果 | 相较旧证据的变化 |
| --- | --- | --- |
| 抖音8.7.0原包，外部Frida attach → Interceptor | 前后两次均PermissionDeniedError，未加载agent；截止已延长到180秒，但实际为立即拒绝 | 用户确认首次重试没有授权窗口；在自有空进程外部attach成功后再试抖音，仍拒绝 |
| 抖音8.7.0原包，Gadget optional → Interceptor | 监听前进程退出-9；同PID新IPS记录CODESIGNING code2 / Invalid Page | 重现原签名终止，未进入Hook；不能等同于等待密码超时 |
| 自有可调试Swift空进程，外部attach → Interceptor | 通过；约1.38秒完成附加、Hook、卸载和自有进程退出 | 补齐此前外部attach对照的成功证据；不是重复旧Gadget控制正例 |

当前可以确认本机外部Frida附加在该可调试目标上工作；抖音原包的两条路线本轮仍未通过。不能把此前所有失败统一解释为未输入密码，也不能根据本次控制成功反推此前超时的唯一原因。

## 授权事实与解释边界

- 首次抖音重试后，用户明确回复“没有出现窗口”。电脑控制工具禁止访问SecurityAgent，未通过其他工具绕过该限制；未收集密码或代替用户操作授权窗口。
- 控制测试约1.38秒完成，无足够证据断言该过程曾重新弹窗、用户完成了新的授权或现有授权缓存的来源。对照成功是能力证据，不是一次密码授权事件的审计证明。
- 只读核验Developer mode当前disabled；未改DevToolsSecurity、authorizationdb、SIP或原包签名。抖音无get-task-allow，具有cs.debugger、allow-jit、allow-dyld-environment-variables及disable-library-validation。
- [Apple调试权限说明](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.debugger)区分调试器权限、管理员授权和目标get-task-allow；调试器可附加的能力不等于任意目标都允许被附加。当前目标限制的推断与这些条件及对照结果一致，但本轮未取得新的taskgated明确拒绝日志，不能声称穷尽全部拒绝原因。
- [Frida排障文档](https://frida.re/docs/troubleshooting/)也列出未完成认证可能导致失败。本轮采用等待和受控对照验证，没有照搬其中更改系统授权规则或SIP的操作。

## 运行证据

原始结果存于仓库外缓存根的runs目录，默认`${XDG_CACHE_HOME:-$HOME/.cache}/injectra`；本轮复用锁定Python/Frida环境，Frida17.25.2、主机arm64；控制进程实测arm64，抖音本轮未进入agent。

| run | PID | 证据 |
| --- | --- | --- |
| douyin-native-hook-b4fwdpix | 44160 | 原外部attach脚本--deadline 180；PermissionDenied；exit -9仅记录退出码，不单独归因为签名；authorization-context.json保存用户反馈及只读环境摘要 |
| douyin-optional-retry-bk6194ow | 44878 | Gadget optional/QJS、认证回环、独立HOME/profile；result.json与diagnostic-summary.json，原IPS为抖音-2026-11-11-012936.ips |
| frida-attach-auth-retry-te_8_dzm | 45117 | 自建Swift空进程，sleep300；仅该新fixture赋予get-task-allow；真实外部attach同一agent，abs(-7)/abs(-13)回调及7/13正确；卸载后abs(-19)=19且计数仍2；exit -15 |
| douyin-native-hook-slablh2e | 45302 | 正对照后再次外部attach，仍PermissionDenied，exit -15；无agent/Hook执行 |

四个自有PID均已退出，ps复核无残留；未结束用户已有实例。三次抖音运行前后主可执行文件SHA256均为223801802e24cf36c25c0a1667e3e8435129e9316586256f5b62d4762abbd466，codesign --verify --deep --strict均通过。不将其扩大为整个应用包逐字节比较证明。

仍使用原[abs Hook agent](../../tests/frida/native-hook.js)，SHA256为76f98a37659f91749261d0f9f647fb5b897488a5636d0f92debfbc73e6e4377e；Gadget dylib SHA256为484c9bd24b106fd7dc4cbab5a84672d14267bd86bdef00c895cd706a9f39fa4c。optional测试在listener前失败，故agent只是待加载而非实际执行。控制fixture源码/entitlements与运行结果保留缓存，不加入产品或插件目录。

## 验证与恢复

本轮产品和测试源码无修改；外部attach复用[现有脚本](../../scripts/frida-hook-smoke.py)，只把--deadline改为180。Gadget optional和自有调试fixture是同次本机实验，未新增正式命令或改变后端配置；实验中的绝对路径仅保存在短期缓存工件和回执中，长期文档使用缓存根与run名定位。

只同步本报告、兼容性入口和任务记录，验证使用make doc-check、tracking-check及git diff --check；没有把文档校验作为Hook成功证明，也未无条件重跑已通过的产品套件。[执行及review](../archive/T-027.md#t-027)记录准确关闭范围。
