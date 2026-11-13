# Windows 九项通用入口复验

2026-11-13，T-044@v2 / E-002；[Task与当前执行](../TODO.md#t-044)。承接[T-043首轮29/38](windows-apps.md)，E-002新增5款，E-003再补网易云音乐，共新增6款页面合同通过，累计35/38；剩余3款保留缺口，不判为永久不支持。FlowUs、Manus继续按用户要求跳过。

## 新通过的六款

| 应用/版本 | Windows实际入口与结果 | 仓库外证据相对路径 |
| --- | --- | --- |
| QQ 9.9.33.52230 | x64 QQNT.dll MSVC V8 → 共享Bootstrap/WebBackend；明确选择renderer/index.html主页面，双版JS/HTML/CSS、自己的按钮交互、更新/清理通过 | t044-reconnect/QQ-main-3d5c78b2/result.json；初次入口t044-v8/QQ-b7ccba22 |
| WorkBuddy 5.8.3 | x64主exe同一V8策略 → Bootstrap；renderer页面双版及清理通过 | t044-v8/WorkBuddy-9e5ac7f3/result.json |
| Figma 126.11.8 | 实际版本化x64主exe同一V8策略；shell先通过，另明确选取真实/login网页复验通过；未登录 | t044-reconnect/Figma-login-316f0a01/result.json；初次入口t044-v8/Figma-b3b117b9 |
| 抖音 8.8.0 | ia32版本化Electron，目录/ASAR准备 → 同一Bootstrap；初次启动页世代变化被拒绝，显式重新连接原副本后双版与清理通过 | t044-reconnect/Douyin-2246d139/result.json；初次bootstrap/Douyin-0a29255e |
| Cocos Creator 3.8.8 | 受控副本，配置自有空项目后，现有Windows Rust CLI/CDP完整页面合同通过；原无项目启动没有页面不能当作CDP不支持 | t044-runs/CocosCreator-project-8dd2bc41/result.json（首run ca66106b保留） |
| 网易云音乐 3.1.38.205386 | 用户正常退出后，标准CEF启动参数 → 原Windows CLI/CDP；真实Orpheus主页面JS/CSS/HTML、交互、更新、清理全部通过 | cloudmusic-e003/result-final.json；t044-cloud-verify-native-5198300e.json |

所有新Bootstrap成功run均执行两版HTML/CSS/JS、自己的按钮回调，移除后nodes=0、清理回调=2。QQ首个未指定目标的尝试可能选择内部窗口，因此另保留主页面明确选择的run；Figma壳与登录网页证据分开。受控Node夹具通过不能替代上述实机证据。

## E-002时未通过的四款（历史）

| 应用 | 新取证与尝试 | 当前缺口 |
| --- | --- | --- |
| 优酷 9.2.66.1001 | 目录CommonJS副本、Bootstrap、ia32 V8派发、交互桌面错误框核查；错误为Widevine组件下载404。配置包内同架构CDM后仍出现同一错误；有DevTools listening但/json请求超时 | 页面尚未建立，属于应用启动依赖缺口；旧V8排队结果Unknown不计成功、不自动重放。未修改DRM或系统安全策略 |
| 百度网盘 8.10.3.102 | 实际BrowserEngine/BaiduNetdiskUnite.exe独立启动、CEF初始化探针（含正确cwd）、宿主进程/交互窗口核查 | 当前是DuiHostWnd原生欢迎窗口且未加载CEF；独立Electron组件未取得页面。旧Unite PID已退出，错误31不能描述成活动页面附加拒绝 |
| 企业微信 5.0.13.6005 | CEF初始化探针、正确cwd、进程组件与交互桌面窗口核查 | 当前是WeChatLogin原生登录窗口，未观察CEF加载/页面；不把登录窗口分类外推为全客户端原生，也未自动登录账号 |
| 网易云音乐 | CEF启动插桩、普通带参数启动、无调试参数普通启动，检查进程存活与应用日志 | 无调试参数的新run CloudMusic-plain-77ab303b确认关闭请求前已退出，退出码/时间不可得；旧PID是已退出且零线程的残留对象。Frida spawn有未返回记录，启动/交接原因尚待确认，不能据CEF库存在声称页面注入完成 |

本轮新原始证据位于仓库外windows-validation-20261112：t044-cef、t044-runs、t044-preflight-runs、t044-v8、t044-reconnect和t044-*.json。配对配置含秘密，不进入Git。原run不覆盖，晚到页面、世代变化或Unknown均不改写为原尝试成功。

## 泛化边界与工具

- **接入由技术条件选择。** QQ/WorkBuddy/Figma共用[Windows MSVC V8实验](../../tests/windows/v8-windows.js)和[显式目标附加器](../../tests/windows/v8-attach.py)，只配置PID、世代、exe与引擎模块；随后复用[原Bootstrap](../../crates/bootstrap/helper/bootstrap.cjs)和共享Web runtime，没有按品牌的执行代码。x64与ia32分别通过自有Node进程验证；ia32夹具通过不代表优酷成功。
- **准备按资源格式处理。** [副本准备器](../../tests/windows/prepare-copy.mjs)共用CJS、目录/ASAR与PE摘要逻辑。包内Junction规范化并重建为指向副本；外部链接拒绝。抖音的已确认运行锁目录作为显式相对路径排除配置，写入proof，不静默跳过其他EBUSY文件。
- **CEF不是Electron。** [初始化实验](../../tests/windows/cef-init.js)只接受官方版本/结构尺寸匹配的设置副本；本轮未触达三款CEF页面，不把测试代码列作正式支持。登录前原生窗口不能包装成DOM。
- **目标必须明确。** [远程Web合同](../../crates/bootstrap/tests/remote_live.rs)记录选定目标、去查询参数URL，要求世代/URL连续稳定3秒。首次世代拒绝保留；另开明确连接run，不自动重放未知请求。新增启动退出元数据，区别应用退出、端口存在和真实页面；Windows缺失的退出码/退出时间保留null，不因诊断字段缺失中断收尾。默认附加启动参数也替换profile/port占位符，供空项目独立home配置使用。
- **产品边界未扩张。** Cocos使用现有Windows CLI/CDP；Windows V8、CEF与资源副本仍是实验工具，尚不是新增正式CLI后端。没有工作台、品牌适配器、自动登录或原安装改动。

## 验证与恢复

Mac make check、独立Electron合同及Windows格式测试通过；Windows x64和ia32自有Node ABI夹具通过，包内链接隔离测试通过。真正的应用范围只按上表计数。具体机型/工具链、初始进程保护和首轮恢复入口沿用[首轮报告](windows-apps.md)。

仅按用户已有授权结束核验过的本轮测试进程，核对image、创建时间和测试记录；原有用户实例保留。停止CLI/控制端不撤销引导或自动退出App；已加载的开发引导随App退出消失。停止时清理自有临时SSH转发、Git服务及测试编排，不删除原始证据和派生副本。

Review：检查未引入应用名分支、未复制共享runtime、未把进程派发/夹具通过当页面成功；针对QQ内部页选择另做主页面run，Figma壳另做登录网页run。首轮与失败run不覆盖；小步提交可单独回退，Git与仓库外proof保留恢复入口。


## E-003 · 网易云音乐再次复验

用户于同日要求再次执行。当前安装3.1.38.205386；开始时发现两个驻留的自动更新安装器，均存在于原始进程基线，没有终止。随后这两个安装器已结束，观测到新的活动主进程及GPU/renderer；普通Shell启动的短进程退出码为0。更新/实例交接是新线索，尚不据此宣称旧退出的唯一根因已查清。

这次在实际运行的主进程与renderer中确认libcef 91.2.2/Chromium91.0.4472.169，不再仅是文件存在。当前无调试监听。只读上下文观察请求超时、没有取得结果，没有执行页面载荷；后续两进程仍存活且无自有观察控制进程。

该启动阶段曾请求用户确认正常退出并以通用CEF入口重启，答复前保留窗口；当时尚未新增页面成功，34/38口径不变。后续完整通过见下方收口，旧失败不覆盖。新证据位于原缓存t044-cloud-*.json与cloudmusic-e003目录，Task/执行状态以[权威记录](../TODO.md#t-044)为准。


### E-003收口：网易云已通过

用户明确批准正常退出重启。普通关闭/不带强制参数的本机关闭请求仅让App驻留托盘，未强停；用户随后手动退出并回复“已退出”。确认原实例结束后，使用ShellExecute和正确工作目录启动新进程，只提供标准CEF调试端口/回环地址参数，沿用原数据，不提供独立profile或浏览器附加参数。

新主实例在127.0.0.1:58887开放真实`orpheus://orpheus/pub/app.html`；使用既有Windows本机Rust CLI和共享runtime，以html-string模式通过JS/CSS/HTML、自己的按钮0→1、同ID更新不重复、remove和清理归零（节点0）。结果passed=true、cleaned=true。无需新增网易云专用执行代码，也未使用Frida完成这次成功路线。

原始最终结果：cloudmusic-e003/result-final.json；原生控制端输出t044-cloud-verify-native-5198300e.json。App保持打开，原exe摘要与之前一致；没有安装包、沙箱或系统安全策略修改。新成功证明当前接入条件可行，不将全部旧失败强归为更新或单实例的唯一原因。当前累计35/38，仍未取得页面的仅优酷、百度网盘、企业微信；上方E-002的4项表格保留原历史。

Review核对真实页面身份、回环监听及进程归属、原数据模式、完整共享合同和仅用户正常退出的授权边界。本轮没有产品源码变更，不重跑无关后端；以实际页面验证及文档/任务检查验收。
