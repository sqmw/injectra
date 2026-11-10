# 抖音实际宿主方法 Hook 验证

2026-11-11，[T-028](../TODO.md#t-028)，基线9678f7d。抖音8.7.0原包、macOS arm64、Frida17.25.2、Gadget required/QJS。

## 结果与范围

**真实宿主的Objective-C方法分派Hook成功。** 对抖音真实NSApp对象使用的`AtomApplication.sendEvent:`替换方法实现指针，保持原参数并转发原实现。在无窗口自动化操作的30秒复测中，捕获360次调用、360次原调用完成；恢复后IMP读回正确，计数保持360，agent卸载成功，测试进程退出。随后按最终默认命令再验，147/147通过。

这次没有创建测试类或测试对象，没有主动调用sendEvent:，没有生成测试事件，也没有走V8/Node/DOM路径。观察的是真实运行时自然到达宿主的方法调用；不读取NSEvent内容、键盘输入或账号/业务数据。计数随启动和系统事件变化，验收要求非零且原调用全部完成，不要求固定360次。

成功范围是该版本的宿主事件方法，**不等同于抖音业务逻辑Hook、任意C/C++函数、Interceptor或正式原生CLI支持**。不改原包签名、代码页、系统权限或SIP。[前序外部attach及optional失败](douyin-hook-auth-retry.md)保留。

## 身份与执行证据

| 核对项 | 实测值 |
| --- | --- |
| 实际接收对象类 | NSKVONotifying_AtomApplication |
| 方法定义类 | AtomApplication |
| 方法及编码 | sendEvent: / v24@0:8@16 |
| 原实现映像及相对偏移 | Electron Framework / 0x2c9c29c |
| Hook执行上下文 | 目标主线程，接收对象等于该进程NSApp |
| 第二次成功复测 | 360 calls / 360 completed，wrongReceiver、wrongThread、inFlight均0 |
| 最终默认命令复核 | 147 calls / 147 completed，同样无错误对象/线程/在途；恢复后计数147，卸载通过 |
| 恢复 | 当前IMP所有权核验后恢复原IMP；主线程上确认无在途回调，读回原指针，解除pin |
| 恢复后观察 | 等待1秒，计数仍360；NSApplication仍running，随后agent卸载且进程仍存活 |

最后一行证明指针恢复和回调停止，**没有另行验证恢复后新的sendEvent:调用**。原实现返回类型为void，因此“completed”表示原调用正常返回，不是业务效果完全等价证明。配置exceptions:propagate让目标处理原生异常，但本轮没有触发异常验证，不能宣称所有异常行为都已等价。

## 实验序列

原始证据在仓库外缓存根的runs目录；默认`${XDG_CACHE_HOME:-$HOME/.cache}/injectra`，可用INJECTRA_CACHE_DIR覆盖。

| run / PID | 代码与观察条件 | 结果 |
| --- | --- | --- |
| douyin-host-hook-r8bhu7tf / 48914 | agent初版，8秒静置 | 0调用，未通过；方法安装/恢复成功不计作Hook调用成功 |
| douyin-host-hook-zqb5uyap / 49279 | agent增加原生异常propagate；判定门补强全部回执完成数；30秒观察 | 61/61通过。期间两次CUA getApp分别procNotFound/timeout，无点击或键盘操作；不能凭此run声称完全没有观察工具影响 |
| douyin-host-hook-7d0_rtip / 49769 | 与上一run相同agent，30秒观察；不调用CUA或操作窗口 | 360/360通过，补齐不操作窗口的复现证据 |
| douyin-host-hook-5666xgvl / 51360 | 最终源码与默认30秒参数；不调用CUA或操作窗口 | 147/147通过；最终入口/agent/共享启动/Bootstrap/截止实现的源码快照与哈希另存source目录 |

初版agent SHA256为7f6e025853bebb80b9c281d1fdaee2cfd6546655e4dbc2eddf17561338beddb1，后三次为4a14f313657c84fc02a7d09b42777539e67108581b51973769eae343b8569d6d。不能将首个失败与后续成功表述为同代码仅延长时间；首版载荷已重建并与旧run哈希核对，四份agent.js快照均保留各自run中。

四次独立HOME/Foundation home与profile核对通过，恢复运行前清除DYLD继承；四个Popen自有PID退出并复查无残留。四次主可执行文件前后SHA256均为223801802e24cf36c25c0a1667e3e8435129e9316586256f5b62d4762abbd466，codesign --verify --deep --strict通过；不扩大为整个安装包的逐字节比较。

## 前序失败的进一步定位

并行只读诊断核验了T-027的PID44878 IPS：dyld初始化器 → 未符号化的FridaGadget帧 → sys_icache_invalidate(+40)，故障地址位于私有r-x区域。结合listener阶段结果，可定位为Gadget初始化时的CODESIGNING/Invalid Page，早于用户agent加载；不能归因为abs Hook执行失败或抖音主动反Frida。缺少符号，未进一步确定Gadget内部函数。

[Frida Gadget](https://frida.re/docs/gadget/)明确required模式限制Interceptor；本次使用[Apple method_setImplementation](https://developer.apple.com/documentation/objectivec/method_setimplementation(_:_:))及Frida NativeCallback的方法分派机制。未为避开代码签名检查获取可写可执行映射、重映射代码页或改变保护设置。若另行验证需改签名的调试副本，必须先明确独立副本、权限差异和影响并取得授权；本次未执行。

## 源码与复核

- [宿主Hook agent](../../tests/frida/host-dispatch-hook.js)：明确对象/方法/ABI，主线程安装与恢复，原实现转发，回调寿命及计数。
- [测试入口](../../scripts/host-hook-smoke.py)：自然调用、完成计数、恢复后静止及健康检查。
- [共用受控启动](../../tests/frida/gadget_probe.py)：从原自有对象测试入口提取的Gadget required启动/身份核对/清理；两个测试入口复用，未进入产品crates。
- [判定门测试](../../tests/frida/test_host_hook.py)：零调用拒绝、观察后新调用的完成性、错误对象/线程/在途、恢复失效、计数倒退或继续增长。

```sh
# DOUYIN_APP显式选择本机应用；已有实例则拒绝，不结束用户进程。
"${INJECTRA_FRIDA_PYTHON:-${INJECTRA_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/injectra}/frida/venv/bin/python3}" \
  -B scripts/host-hook-smoke.py --app "$DOUYIN_APP" \
  --gadget "${INJECTRA_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/injectra}/frida/gadget/FridaGadget.dylib"
```

默认每阶段deadline20秒、observe30秒；`--observe 1 --deadline 2`用于快速诊断，不保证产生真实事件，0调用必须失败。恢复未知时先退出本次自有进程，不释放仍可能被引用的回调，不自动重试。已存在的objc-hook-smoke命令保留原自有对象测试，载荷未变；共享启动器仅提取既有逻辑。

自审发现并修复了“snapshot之后原调用失败仍可能通过”的判定缺口：三份回执都核对completed==calls且计数不倒退。恢复后的保证按上文收窄。测试/文档回归及review见[执行记录](../archive/T-028.md#t-028)；本次没有改产品能力、Cargo依赖或共享Web runtime。

最终回归：make check通过18 Rust、5 Web、8 WK宿主、3 Gecko HTML；make test-frida通过31 Python。两轮只读子Agent review与Parent自审完成，判定门修复后复核无新增阻断问题；实际测试与产品兼容性边界保持分开。
