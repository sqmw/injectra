# macOS WKWebView 后端

属于[T-017](TODO.md#t-017)，复用WebBackend/WebSession与唯一Web runtime；[源入口](../crates/webkit/src/lib.rs)、[装配入口](../crates/webkit/helper/agent.js)、[WK执行](../crates/webkit/helper/webkit_agent.js)、[Mac宿主](../crates/webkit/helper/macos_appkit.js)、[桥接](../crates/webkit/helper/route.py)。

## 调用与条件

make setup-frida准备锁定Python3.13/Frida17.25.2及frida-objc-bridge8.0.6源码依赖；make setup-gadget获取并核对官方Gadget。Rust内嵌第一方helper源，首次在外部内容缓存编译ObjC agent，第三方包不复制进Git；正式运行不依赖references或编译机器的源目录。

通过显式、经身份核对的Gadget会话接入已加载WKWebView的Mac应用：

- injectra targets --backend webkit --session SESSION_JSON
- injectra apply --backend webkit --session SESSION_JSON --target TARGET_ID --bundle MANIFEST
- injectra remove --backend webkit --session SESSION_JSON --target TARGET_ID --id BUNDLE_ID

独立启动/既有会话注册与核验复用[Frida入口](frida-backends.md)（`injectra session launch/register/verify`），选择`--route webkit`。不会自动附加任意PID，也不会自动改变isInspectable、系统授权或目标签名。需要宿主允许加载Gadget；WebKit引擎存在不意味着加载条件满足。

## 页面执行与资源

在原生主线程遍历NSApplication窗口的视图树，仅选择WKWebView；页面ID先绑定设备/进程/route-profile作用域，再绑定视图指针和URL，每次执行重新查找当前视图并核对URL。只执行主frame的page content world，不承诺未挂入窗口树的WebView、iframe或同URL文档世代锁定。

使用WKWebView的callAsyncJavaScript，等待Promise，页面内封装JSON结果及原异常；原生API错误保守Unknown。异步等待使用4.5秒页面计时返回Unknown，不重放；原表达式可能稍后继续，计时返回不等于撤销。

主线程排队和原生completion均保持脚本pin与强引用，取得回调后在主线程释放；关闭前检查未完成计数。Frida SDK调用采用可取消截止，超过界限拒绝强行卸载并报告Unknown。若页面同步死循环、主线程永久阻塞或助手崩溃，不能保证自动回收；不将其伪报成功。控制端CLI退出时，助手继续正常收尾。

## 当前证据

独立Swift/AppKit/WKWebView宿主的两个非持久页面通过共享合同（run webkit-i07adzl2）；正式CLI的apply/update/remove与控制端退出后清理通过（run webkit-lg0f0zoc）。覆盖JS/CSS/HTML、真实页面计算样式、程序触发自有按钮点击读回、Promise/异常、循环结果编码失败、另页不变、精确目标拒绝、超时Unknown及恢复。

首次native completion空值适配错误保留于webkit-8e_qlir7，已按frida-objc-bridge的null语义修正，不把失败run改写为成功。

真实应用已通过：用户明确批准的Codex Switcher0.2.11（Tauri/WKWebView）在独立空HOME/CODEX_HOME与不含账号凭据的启动环境中通过主页面完整合同、托盘页面隔离及CLI apply/update/remove；证据在外部t017/webkit-real-contract.log与webkit-real-result.json。未导入原账号或执行账号切换，最终按可执行路径与启动时间退出本次创建的实例。

Easydict尝试未完成受控接入，保留launch-1794287157890869000并退出该测试新实例；不能据此断言所有Easydict/WebKit路线不支持，也不放宽签名/系统保护。Clash Verge仅作静态条件检查，未操作其运行状态。

## 开发检查

make test-webkit编译自有Swift宿主到外部缓存，创建独立HOME和非持久WebKit数据存储，验证两个页面，最后只退出自己的进程。Swift模块缓存、Gadget、日志和profile均在仓库外。make check默认不启动或接入真实应用。

首次下载官方Gadget约25MB压缩包；ObjC依赖版本和完整性见crates/frida/helper/package-lock.json；恢复依赖使用make setup-frida/setup-gadget，不恢复旧token或猜测旧PID。

T-019将Mac AppKit窗口发现/主线程调度与WK执行分开；只有macos-appkit profile已注册。宿主schedule同步抛出必须表示未入队；已入队回调必须保留资源直至回执。排队期间超过截止的回调只做排空，不再启动迟到载荷；已经开始的原生操作仍保持Unknown/回调清理语义。该接口通过测试替身和Mac真实宿主验证，不代表UIKit/iOS实现。

回调分配及清理调度失败也纳入宿主合同：没有提交载荷的分配失败为NotExecuted；仍持有原生资源时shutdown不得报告closed。

T-019最终回归与一次未定位到具体SDK调用的targets超时见[验收记录](validation/device-boundaries.md)。自有测试宿主在双页导航完成后发出ready，测试工具据此再开始发现；该等待只属于测试准备，不改变正式操作截止。
