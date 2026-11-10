# 控制端与目标设备边界验收

[T-019@v1](../TODO.md#t-019)；基线74844e7。实现者自审，不作为独立审核。接口与发现见[设备边界](../architecture/device-boundaries.md)。

## 范围与处置

| 审查问题 | 已实现处置 | 验证 |
| --- | --- | --- |
| 共享连接代码把目标PID当成本机PID | TargetProvider持有身份来源；Mac本机ps/sysctl移到具体实现 | 未知设备先拒绝；测试provider无需本机PID查询 |
| 控制端、设备和连接参数混放 | 控制端执行独立；v2分开device/process/connection/routes | 23项Python测试覆盖旧会话、身份、连接器及输入拒绝 |
| 页面局部ID可跨设备/进程冲突 | Frida ID限定设备实例、进程世代及路由/profile | 同PID/局部页面号跨设备拒绝；旧未限定ID不回退 |
| WK执行直接依赖AppKit窗口枚举 | WK执行接收host；Mac发现/主线程调度独立 | 8项host合同及真实双WKWebView |
| 排队、回调异常的资源归属不明确 | 迟到未开始操作只排空；分配失败释放view；清理未确认不伪报closed | 调度拒绝、超时、分配失败及清理失败测试 |

core/Web/application/native/CDP源码与Cargo依赖相对74844e7无差异。没有新增crate、公开CLI命令或手机实现；CDP仍按显式协议端点接入，不据端口推断物理设备。测试替身只证明接口可替换，不能证明Android/iOS兼容。

## 实测证据

外部证据相对默认缓存根（INJECTRA_CACHE_DIR，默认~/.cache/injectra），不提交日志、会话凭据或二进制。

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| make check | 18 Rust、5 Web、8 WK host通过；格式/lint/架构/文档通过 | t019/review-unit.log |
| make test-frida | 23项通过 | t019/review-session.log |
| 默认CDP，独立Electron45.2.1 | 10核心检查通过 | t019/final-cdp.log；runs/electron-txLGCg |
| Gadget，独立Electron36.4.0 | 页面合同、CLI apply/update/remove、控制端退出清理通过 | t019/review-gadget.log；runs/gadget-electron-yd5f5ni7 |
| WK，独立双页面宿主 | 2页面合同、CLI及控制端退出清理通过 | t019/webkit-ready-check.log；runs/webkit-5jucn2z7 |
| 旧v1抖音会话只读发现 | 7页，源文件SHA未变 | t019/legacy-discovery-result.json |

最终共享helper诊断代码在WK和Gadget两个独立宿主上均通过。未重复启动真实Codex Switcher或其他用户应用；既有T-017真实应用结果保持历史口径。

## 保留的失败与限制

t019/final-checks.json中的WK失败及final-webkit.log原样保留，run webkit-2fclaktz：targets阶段15秒Frida截止，NotExecuted，尚无页面载荷提交。原日志粒度不足以确定是哪一次内部SDK调用，不能把推测写成根因。

后续增加不含凭据的阶段/耗时诊断，并让自有WK测试宿主在两个页面导航完成后显式发出ready；没有加长操作截止、弱化身份核验或重放载荷。新run webkit-5jucn2z7通过。ready排除了测试启动时序变量，不证明所有外部宿主的瞬态超时已消除；仍按Unknown/NotExecuted和无自动重放合同处理。

当前只有macos-local目标provider、认证回环gadget-tcp连接器、macos-electron-v8与macos-appkit宿主profile。未来设备需分别实现身份核验、连接方式和宿主生命周期；启动/授权、USB/ADB、UIKit及移动设备验证均未实现。

## 兼容与恢复

v1文件只读规范化，不自动改写；新注册为v2。旧Frida页面ID须重新targets发现。目标可执行路径与进程世代由provider解释，不要求存在于控制端文件系统；端口、token和控制端日志路径不纳入目标身份。

源码可按本任务提交逆序回退；旧工具不能读取新v2会话，应在目标仍受控时重新注册相应版本的描述，不删除旧证据或手工猜测PID。回退源码不会撤销已提交的用户载荷。长期阻塞/助手崩溃仍不能保证原生回调自动回收，未确认清理必须报告而非强制卸载。
