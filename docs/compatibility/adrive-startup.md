# 阿里云盘启动条件复核

2026-11-11，[T-034](../TODO.md#t-034)续接82018ad。用户要求继续深入。原[pre-runtime静态核查](adrive-preruntime.md)、[Frida结果](adrive-frida.md)及[T-032](../validation/macos-ten-apps.md)均保留，本轮没有通过修改保护来改变结果。

后续[T-035测试副本资源补丁](adrive-resource-patch.md)已验证本地登录壳页面执行；其构建/签名条件与本页原包不改动的实验分开，以下历史结果保留。

## 当前进展

新增证据将问题细化为启动/授权环境和实际调试响应；**仍没有新的页面注入成功**。用户已手动解锁，公开菜单/偏好设置复核及一次新的CDP启动观察已完成，普通首页已恢复。解锁后的测试仍快速退出；退出根因属于**证据不足待确认**，不能统一归因于锁屏、钥匙串或包内额外文件。

## 旧测试工具审查

只读子Agent确认：两个direct旧run在发现循环中确实观察到启动PID退出0，发生在finally清理之前，不能归因于清理代码先杀了该主进程。但旧工具随即清理已跟踪子树，缺少交接后的快照；过滤前/json/list和HTTP错误历史也未保存。因此旧结果不能证明所有后继进程均自行退出，或把“无可用page”一律解释成没有调试服务。

已新增[启动观察器](../validation/startup-observer.md)补齐这些证据。自有夹具只验证工具可靠性，实际阿里结果如下。

## 同版本新实验

均为aDrive6.9.3、macOS26.5.1、arm64；每次新建Chromium profile，沿用标准启动参数，不改签名、fuse、应用代码或系统保护。只查询CDP元数据，没有提交JS/HTML/CSS载荷。

| run（缓存根runs下） | 条件 | 实测 |
| --- | --- | --- |
| startup-observe-hhw0kuu3 | 隔离HOME/CFFIXED，CDP，45秒 | 主PID41886和回环监听49303持续存在；14次HTTP请求均达到3秒总截止。没有页面列表；观察器没有结束子树 |
| startup-observe-s_gnv4xu | 正常HOME、独立Chromium profile，CDP，45秒 | 取得一次HTTP200及原始空列表，主PID43244约1.09秒被观察到退出0；请求结束后监听归属未确认。随后观察至窗口结束，未发现可确认的接管宿主 |
| startup-observe-ci2dr51u | 正常HOME、独立Chromium profile，无调试参数，30秒 | 主PID44943保持运行，并创建多个renderer。没有CDP请求；不计注入成功 |
| startup-observe-yudc69f4 | 用户解锁后，正常HOME、独立Chromium profile，CDP，45秒 | 主PID74757约0.706秒被观察到退出0，约1.055秒后跟踪集合为空；完整观察45秒。日志有DevTools listening，但采样未捕获可探测的监听归属，HTTP回执0，无页面或载荷 |

这些run并非完整严格的单变量因果试验：临时目录不同，应用可能操作共享系统服务，观察器也有增量修正；不能据此断言HOME或调试参数单独解释了全部失败。首个run旧字段trackedLive包含过defunct条目；主PID及监听的持续证据仍成立，不能把所有被跟踪PID都当成可运行宿主。

新run与s_gnv4xu的主程序SHA、版本、启动环境生成逻辑和参数类别一致，但观察器整体源码不同，系统服务/缓存/网络状态也未完全控制。新run没有HTTP回执，不能写成超时、空列表或“从未监听”；短暂监听可能落在采样间隔内。新run未采线程栈，不能套用首个run的钥匙串等待解释退出。旧run的桌面状态未知，因此不是严格锁屏前后对照。

各run的request.json记录源哈希，source目录保存与当时哈希匹配的脚本/模块。后续修正了stat记录、响应完成与归属快照的时序、配置HOME命名；不把新字段补造到旧run。

## 只读线程采样

首个run的sample.txt于12:34:12采集，距45秒观察结束约150秒。该次主线程的91个样本位于SecKeychainAddGenericPassword → defaultKeychainUI/makeLoginAuthUI → AuthorizationCopyRights → XPC同步等待。它证明**采样当时**存在钥匙串添加操作的授权等待，不证明前面14次请求全过程都是同一原因，也不证明写入成功或具体钥匙串项。Agent没有读取密码/条目内容或修改钥匙串权限；不能因此说应用自身没有钥匙串操作。

普通启动对照后续sample.txt显示主线程在NSApplication事件循环等待，未在该次样本中看到前述授权栈。Electron/V8部分帧只有邻近符号和大偏移，不据此指认具体V8函数。首个PID在后续检查时已不存在，未取得其最终退出码或结束者。

## 安装包完整性与其他线索

Parent核对CodeResources的141个hash2资源，无差异或缺失。只读子Agent进一步核对10个nested cdhash、39个symlink以及CodeResources自身与CodeDirectory特殊槽-3，均匹配；槽含义参考[Apple TN3126](https://developer.apple.com/documentation/technotes/tn3126-inside-code-signing-hashes)。

当前codesign严格校验明确报告4个额外文件：包内sync_drive_app/data下的id_generator及其-shm/-wal文件，以及同步组件的syncapp.pid。它们是同步运行数据，未读取内容、未移动或删除。该发现解释了当前资源封存错误，**没有建立其与CDP退出的因果关系**。

当前Shell未设置所查代理/Node/DYLD变量；系统代理启用，但Python对127.0.0.1的绕过判断为true。只说明当前环境，不能回填旧run或据此把历史超时归因于代理。新观察器只直连已核对归属的回环端点，不修改系统代理。

安装包主入口编译资源的只读字面检索出现openDevTools、toggleDevTools各一次。没有解码控制流、执行或改写该资源；字面存在不能证明菜单已启用或有外部执行入口。无调试旧对照中的wss9227用途未确认，不将其当作CDP，也未向其发送未知操作。

## 解锁后的界面复核

用户回复先前钥匙串窗口“没注意到”，因此保留未知；用户随后明确已手动解锁。通过CUA正常打开原应用并看到首页。修改、Window菜单未见开发者工具；首页标题右键未观察到上下文菜单。此次实际打开偏好设置，看到普通锁定/隐私/传输/更新/账号设置，未见调试或外部扩展入口。只读检查后返回首页，没有更改选项、账号、缓存或系统权限；不能把所查界面范围外的入口一并排除。

随后沿用正常退出授权关闭普通实例，运行yudc69f4。没有确认新的钥匙串提示，也没有访问SecurityAgent或代替用户处理系统授权。当前缺口仍是可执行的目标侧入口：未发现已提供的外部早期加载接口，CDP也未取得稳定页面；不把内部preload、编译资源字面或未知服务算作入口。

## 现场与恢复点

原普通实例PID32254按已有授权正常退出后才启动实验。观察器本身不终止子树；首两个新run的已跟踪主进程已结束。普通启动测试PID44943经唯一profile及启动时间核对后，控制端仅发送SIGTERM并确认退出；没有清理整棵子树或结束原同步服务。

前次因Mac锁定未恢复普通窗口的现场记录保留。用户解锁后已补做恢复和界面复核；最新实验结束后再次正常打开阿里云盘，CUA确认首页/最近看过，不再是启动中。普通实例PID75682没有本次测试profile或调试参数。新run跟踪过的四个PID同世代均已不存在，未发送终止信号或清理同步服务；restoration-check.json保存复核结果。

完整性/环境摘要位于缓存根runs/adrive-entry-audit-iemq239j；四份启动观察及各自已有的采样/恢复记录按上表run保存。新run已保存与request.json哈希匹配的source快照。文档中的路径使用缓存根相对定位，不提交账号目录标识、签名运行数据或完整日志。
