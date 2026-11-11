# 阿里云盘 Frida Gadget与原生Hook验证

2026-11-11，[T-033](../TODO.md#t-033)，基线45121ed。用户明确要求尝试Gadget或Hook，沿用已批准的正常退出阿里云盘后测试授权。本次为aDrive6.9.3、macOS arm64、Frida/Gadget17.25.2，不复用旧6.8.0结果。

## 结果

| 路线/阶段 | 实测 | 后续执行 |
| --- | --- | --- |
| 现有启动工具，Gadget required/QJS | Gadget监听未建立，启动器报listener unavailable；没有身份回执；lsof映像列表未列出Gadget | 未进入Hook或页面执行 |
| 对同一已核验测试PID的直接Frida attach | 独立保存的附加尝试返回PermissionDeniedError | 未加载用户Hook或页面脚本 |

当前阻塞在原生代理接入阶段，不是已执行Hook后函数行为错误。没有取得Gadget身份回执，不能仅凭环境变量或配置文件宣称代理已进入进程；lsof的未列出也只作辅助观察，不单独证明所有内存映射状态。

## 加载条件与解释边界

只读codesign信息显示：原包启用Hardened Runtime，disable-library-validation显式为false，未声明allow-dyld-environment-variables或get-task-allow；allow-jit及allow-unsigned-executable-memory为true。

[Apple的DYLD权限说明](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.allow-dyld-environment-variables)区分允许使用动态链接环境变量和库签名条件；[库验证说明](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.disable-library-validation)规定默认要求Apple签名或与主程序相同Team ID。这些条件与观察到的加载/附加受限一致。未取得具体dyld拒绝日志，因此不把静态权限和无监听合成唯一根因证明；允许应用自身JIT也不能作为外部代理加载成功的证据。

使用的是现有[生产启动入口](../../scripts/frida-launch.py)，保持required模式；启动前另行记录主程序和包的严格签名诊断（均已有sealed-resource失败）。没有修改启动器的检查逻辑、安装包、签名、fuse、系统权限或安全设置。

## 执行与清理

普通PID28887经应用菜单正常退出。启动器新建独立HOME/profile，测试PID31707，命令行及唯一profile均核对匹配。Gadget使用官方缓存中同一dylib，SHA256为484c9bd24b106fd7dc4cbab5a84672d14267bd86bdef00c895cd706a9f39fa4c；配置和状态保持私有，不向输出或文档复制token。

临时协调脚本在SIGTERM后等待5秒未能确认退出，抛错导致聚合回执及第一次附加结果未落盘。原launcher.log和映像列表已保存；第一次附加的结果不追补为成功或失败。随后重新核对同一PID、启动时间和profile，单独保存附加结果PermissionDeniedError；没有重复执行用户载荷。

测试进程未响应正常终止。确认它仍是本次独立目录、同一启动世代的测试PID后，以SIGKILL结束该测试进程；未强制结束原用户实例。复查无带本次profile的残留进程。随后重新打开普通阿里云盘，原生UI确认首页/文件导航可见且不再显示启动中。

观察结束后的主程序SHA256为34e63f81652fc8e1aed7debef2552dd6b17f70de4b42c077956c181e4949a481，与T-032的6.9.3记录一致。由于首个协调脚本未保存聚合回执，不把内存中未落盘的起始哈希补造为独立的本次前后证据；也不宣称完整安装包逐字节未变化。

## 证据定位

缓存根遵循项目约定，默认`${XDG_CACHE_HOME:-$HOME/.cache}/injectra`。

- runs/adrive-frida-48g069tu：launcher.log、mapped-images.txt、直接附加的direct-attach.json、清理cleanup.json、残留核对、恢复restoration.json、签名上下文及最终result.json。
- frida/runs/launch-1794365830699241000：启动器生成的Gadget、配置、独立home、startup.log与state.json；含凭据的文件保持本地私有。
- 原CDP/Node尝试：[十款应用验证](../validation/macos-ten-apps.md)。本次不把其失败理由套用到Frida。

本轮只做真实接入尝试与只读条件诊断，没有产品源码或公开接口变更。验证以实际启动器错误、已保存的附加回执、清理/恢复和文档检查为准；没有运行或宣称通过不可达的Hook/页面套件。自审见[执行记录](../archive/T-033.md#t-033)。
