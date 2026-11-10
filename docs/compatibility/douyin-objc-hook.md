# 抖音 Frida Objective-C 分派 Hook 验证

2026-11-11，[T-026](../TODO.md#t-026)，基线d459ee9。目标为抖音8.7.0原包，macOS arm64、Frida17.25.2、Gadget required/QJS。此项是新方法分派Hook实验，不重复旧V8/Node页面注入套件。

## 结论与证据边界

**在真实抖音进程内，对专用测试对象的方法分派Hook通过。** 两次调用进入Frida回调，回调转发原实现且结果保持一致；恢复方法后第三次调用不再进入Hook，agent可卸载，进程保持存活直到测试器主动结束。

这证明该原包/运行模式下可以用Objective-C运行时替换测试类的实现指针（IMP），不证明抖音业务方法、任意C/C++函数、内存修改后端或正式CLI已实现。测试由探针主动调用自有对象，没有验证业务事件自然触发、并发调用或长时间存活。[T-025的Interceptor失败](douyin-native-hook.md)不被本结果覆盖。

## 机制与保护边界

调用链：原包受控启动 → Gadget required → Frida NativeFunction/NativeCallback → Objective-C方法表 → 自有对象hash方法。未使用Interceptor，也未走V8/Node或页面DOM执行。

- 创建唯一命名的NSObject子类，将继承的hash实现复制为该子类自己的方法，再替换这一方法的IMP；显式拒绝修改父类Method，测试结束核验NSObject原方法未变化。
- 先取基线，再Hook两次并调用原实现，最后恢复IMP、第三次调用、核对计数。测试不改变返回值。
- 回调被方法表引用期间pin脚本；确认恢复后才unpin/卸载。恢复异常或结果未知时先退出测试器拥有的进程，再释放连接，不自动重放。
- 自有对象release；已注册测试类的元数据保留至该自有进程退出。这里只提供可选测试，不新增通用撤销产品能力。
- 既有进程存在则拒绝启动；新HOME/CFFIXED_USER_HOME/Foundation home及profile核对一致，恢复目标运行前清除DYLD继承。不把这些检查宣称为完整系统沙箱或所有共享存储隔离。
- 没有重签名、改安装包、关闭系统保护、登录或读取业务数据。签名和主可执行文件哈希核验不能代替整个应用包的逐字节比较。

机制依据：[Apple class_addMethod](https://developer.apple.com/documentation/objectivec/class_addmethod(_:_:_:_:))允许增加子类方法覆盖；[method_setImplementation](https://developer.apple.com/documentation/objectivec/method_setimplementation(_:_:))设置方法实现。[Frida Gadget](https://frida.re/docs/gadget/)说明required模式限制Interceptor；[Frida API](https://frida.re/docs/javascript-api/)提供NativeFunction/NativeCallback及脚本pin/unpin。能在当前抖音原包运行是下列实测结果，不由API文档直接推定。

## 实机运行记录

缓存根遵循INJECTRA_CACHE_DIR，默认`${XDG_CACHE_HOME:-$HOME/.cache}/injectra`；原始资料在其runs目录，token只保存在0700 run中的0600配置，不写入本报告。

| run / PID | 变化与结果 |
| --- | --- |
| douyin-objc-hook-r2hmbl3w / 41296 | 初次实机通过；hash四次结果4355078240；Hook计数2，恢复后仍2 |
| douyin-objc-hook-eo1jroii / 41695 | runner补强“卸载成功且进程仍存活”验收后重测通过；四次结果4349835360，计数2/2；agentUnloaded、rootMethodUnchanged均true |

四次结果依次为基线、Hook第一次、Hook第二次、恢复后；hash是对象相关值，不要求不同进程之间相等。两个run均为真实主进程arm64、policy required、QJS；Bootstrap返回实际可执行路径和独立home。结果均记录主动结束后的exitCode -15、ownedProcessExited true。

- agent SHA256：9621d64fcffad88b5c5fa4e89620923b4d06bd200a8565e8fd8f610329143cfe。
- Gadget dylib SHA256：484c9bd24b106fd7dc4cbab5a84672d14267bd86bdef00c895cd706a9f39fa4c。
- 两次运行前后主可执行文件SHA256：223801802e24cf36c25c0a1667e3e8435129e9316586256f5b62d4762abbd466；codesign --verify --deep --strict通过。

## 复核入口

[测试器](../../scripts/objc-hook-smoke.py)复用现有Gadget启动器的Bootstrap核验；独立保留测试进程Popen句柄，不注册页面会话、不改正式后端。[Hook agent](../../tests/frida/objc-dispatch-hook.js)是最小测试夹具，不是插件Bundle。

DOUYIN_APP显式指定待验应用；只在明确要新建并退出测试实例时执行：

```sh
"${INJECTRA_FRIDA_PYTHON:-${INJECTRA_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/injectra}/frida/venv/bin/python3}" \
  -B scripts/objc-hook-smoke.py --app "$DOUYIN_APP" \
  --gadget "${INJECTRA_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/injectra}/frida/gadget/FridaGadget.dylib"
```

每阶段截止默认15秒，`--deadline 2`可做快速截止诊断；清理截止5秒。沿用项目可选uv/Frida环境，无依赖升级。失败保存独立result.json，不覆盖旧run；结果未知不会重试。

判定门测试见[test_objc_hook.py](../../tests/frida/test_objc_hook.py)：漏回执、无Hook、恢复后仍触发、返回值变化、父类被改均拒绝通过；已有用户实例在启动前拒绝。这些测试仅核验测试器，原生兼容性以实机run为准。最终检查与review见[执行记录](../archive/T-026.md#t-026)。
