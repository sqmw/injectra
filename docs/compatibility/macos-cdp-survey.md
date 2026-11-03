# macOS 本机应用 CDP 注入普查

2026-11-03；归档任务：[T-009](../TODO.md#t-009)。本报告归档已完成的SIDE-002/E-001普查，不改变产品契约或新增后端。现有Rust CLI直接复用，未为各应用修改注入核心。

## 结论与统计口径

macOS Chromium／CDP注入路径已获得跨应用复用证据：宿主开放兼容调试入口、指定页面上下文可达、页面已就绪且载荷符合页面策略时，同一核心能完成受控runtime注入。**不能据此宣称全部Chromium应用、全部页面或全部HTML注入方式均受支持。**

盘点系统应用目录、全局应用目录和用户应用目录，最多遍历三层文件夹，遇到`.app`不再进入其内部继续计数。共222个应用包，计数单位不是去重产品；未覆盖下载、备份及开发构建产物。通过应用包元数据、框架标识／资源、已有入口及公开资料筛出40个相关候选：标准Electron 29、定制Electron 1、Chromium浏览器2、定制Chromium 3、NW.js 1、CEF组件宿主4。框架存在只用于候选识别，不证明主界面全部由Web渲染，也不证明入口可达。

| 最终状态 | 应用数 | 含义 |
| --- | ---: | --- |
| 基线载荷完整通过 | 18 | CSS＋HTML＋JS，完成样式、交互、同ID更新及remove |
| JS构造DOM载荷完整通过 | 2 | HTML字符串被Trusted Types拒绝；改用通用JS DOM构造载荷完成同组验证 |
| 本次未获得CDP入口 | 4 | 停在连接／页面发现阶段，不能判定所有接入路线都不可行 |
| 隔离条件未确认 | 2 | 停在载荷执行前，不计为注入执行失败 |
| 未启动／仅查现有入口 | 14 | 保留现有工作或规避同步、网络、屏幕共享等启动副作用，未计为失败 |
| 合计 | 40 | 实际启动尝试26个应用 |

26个应用共31次尝试。首次完整通过16个；Super Productivity、Cursor、Devin、FlowUs分别复验后，最终完整通过20个。这里是便利样本的条件验证，协议还经历修正，**不计算或外推总体兼容率**。最终通过不覆盖首次失败记录，31次尝试逐行保存在[脱敏尝试清单](macos-cdp-survey-attempts.csv)。

## 方法与验收

1. 使用同一现有CLI及通用面板载荷；每次启动分配新的临时目录、用户数据目录和回环调试端口。浏览器从空白页启动，编辑器类使用独立扩展目录并禁用扩展。原有进程不重启。
2. 核对监听地址为`127.0.0.1`、端口归属新测试主进程及独立目录初始化。页面选择要求可明确定位，`head`／`body`存在、视口大于300×200；无法确认隔离则停止注入。
3. 注入自有400×150面板：CSS计算背景色为`rgb(17, 24, 39)`，中心误差小于2像素；仅有一个面板宿主和两个runtime标记节点；自有按钮使计数0→1。
4. 同ID再次应用返回`updated`，计数重置为0且不积累节点；`remove`返回`removed`，自有面板消失。每个通过项均满足整组断言，截图只裁剪自有面板。
5. 基线HTML字符串被页面策略拒绝时，尝试统一JS DOM构造模板（`createElement`／`textContent`）。这是载荷选择，没有放宽页面策略、修改安装包或加入应用专属选择器。

本轮只验证空白、启动或登录页上的自有节点，不涉及聊天、文件编辑、业务控件、账号动作，也未检验全部iframe、Shadow DOM或复杂页面策略。CSS／JS和HTML字符串入口分别报告，避免将一种入口的成功外推到其他入口。

## 完整通过清单

`baseline`表示基线CSS／HTML／JS；`fallback`表示JS构造DOM。引擎列为CDP实际报告值，不从宿主框架版本推算。尝试次数包含未完成的旧尝试。

| 应用 | 应用版本 | CDP报告 | 载荷 | 尝试次数 |
| --- | --- | --- | --- | ---: |
| Antigravity | 2.24.0 | Chrome/152.0.7977.78 | baseline | 1 |
| CocosDashboard | 2.2.1 | Chrome/126.0.6478.185 | baseline | 1 |
| Cursor | 3.7.36 | Chrome/142.0.7444.265 | fallback | 2 |
| Devin | 3.10.35 | Chrome/148.0.7778.97 | fallback | 2 |
| FlowUs | 1.2.1 | Chrome/108.0.5359.179 | baseline | 3 |
| Follow | 0.3.3-beta.0 | Chrome/130.0.6723.118 | baseline | 1 |
| Google Chrome | 156.0.8078.80 | Chrome/156.0.8078.80 | baseline | 1 |
| Logseq | 0.10.15 | Chrome/140.0.7339.240 | baseline | 1 |
| Manus | 1.7.7 | Chrome/148.0.7778.280 | baseline | 1 |
| Microsoft Edge | 156.0.4314.58 | Edg/156.0.4314.58 | baseline | 1 |
| Mockitt | 1.3.6 | Chrome/108.0.5359.215 | baseline | 1 |
| Notion | 7.35.0 | Chrome/150.0.7871.250 | baseline | 1 |
| Quark | 7.1.2.956 | Chrome/144.0.7559.86 | baseline | 1 |
| Super Productivity | 16.5.3 | Chrome/138.0.7204.251 | baseline | 2 |
| TRAE SOLO CN | 0.1.19 | Chrome/142.0.7444.235 | baseline | 1 |
| Trae CN | 3.4.11 | Chrome/142.0.7444.235 | baseline | 1 |
| Trae | 3.5.73 | Chrome/142.0.7444.235 | baseline | 1 |
| ZCode | 3.1.0 | Chrome/146.0.7680.80 | baseline | 1 |
| draw.io | 30.3.14 | Chrome/148.0.7778.280 | baseline | 1 |
| 哔哩哔哩 | 1.14.2 | Chrome/108.0.5359.215 | baseline | 1 |

## 未完成注入验证的清单

| 应用 | 应用版本 | 本次停止阶段 | 证据边界 |
| --- | --- | --- | --- |
| Figma | 126.5.6 | CDP入口不可达 | 本地`main.js`定向取证发现主动移除`remote-debugging-port`；未改包、启用内部测试开关或绕过限制 |
| CocosCreator | 3.8.8 | CDP入口不可达 | 本次参数启动在25秒探测窗口内未获得入口，原因未确认 |
| 抖音 | 7.9.0 | CDP入口不可达 | 本次参数启动在25秒探测窗口内未获得入口，原因未确认 |
| wechatwebdevtools | 1.06.2412050 | CDP入口不可达 | 已识别NW.js；本次参数启动在25秒探测窗口内未获得入口，原因未确认 |
| WorkBuddy | 5.9.3 | 隔离确认 | 请求的独立用户目录未确认初始化，未执行载荷 |
| 优酷 | 9.2.73 | 隔离确认 | 请求的独立用户目录未确认初始化，未执行载荷 |

## 未启动或仅检查现有入口

| 应用 | 应用版本 | 本轮处理及原因 |
| --- | --- | --- |
| Anytype | 0.55.4 | 已运行；本轮不重启，尚未确认可并行隔离的接入方式 |
| BaiduNetdisk_mac | 8.2.7 | 同步/下载客户端；未确认独立账号及数据目录，不启动后台同步 |
| Claude | 4.4212.0 | 已运行；本轮不重启，未新增调试入口 |
| Deskreen | 2.0.4 | 屏幕共享服务启动副作用不纳入仅UI注入测试 |
| Doubao | 2.34.6 | 现有9222端口可达并报告Chrome147；本轮仅查入口，未重启或向现有窗口注入 |
| EasyConnect | 7.6.7 | VPN/网络服务启动副作用不纳入仅UI注入测试 |
| NeteaseMusic | 3.1.5 | 带CEF；缺少已确认的隔离启动及CDP入口配置，未启动 |
| OBS Studio | 32.2.2 | CEF是嵌入浏览器组件，主UI并非纯Web；未新增宿主调试设置 |
| QQ | 6.9.86 | QQNT框架确认Electron37.1.0；现有候选端口未发现CDP列表，不重启 |
| QQLive | 2.164.0 | 带CEF；缺少已确认的隔离启动及CDP入口配置，未启动 |
| Qianwen | 5.1.5.293 | 定制Chromium应用正在运行；不重启或改写现有配置 |
| Xmind | 24.01.13311 | 已运行且有用户文档；保留现有实例 |
| aDrive | 6.8.0 | 同步/网盘客户端；未确认独立账号及数据目录，不启动后台同步 |
| 企业微信 | 5.0.9 | 带CEF；未确认独立用户数据与调试入口，不接入现有账号 |

Doubao 2.34.6本轮仅从现有入口读取到`Chrome/147.0.7727.149`和3个page目标，未向现有窗口注入；不能用[历史2.33.5实测](doubao.md)替代本版本注入证明。当前协作宿主及非候选应用未参与启动注入。

## 复验与协议变化

| 协议 | 调整及依据 | 对旧结果的处理 |
| --- | --- | --- |
| v1 | 首轮试跑；Chrome、Edge、draw.io通过，Super Productivity尚未就绪 | 原尝试保留 |
| v2 | 增加最长18秒DOM就绪等待及独立尝试目录；Super Productivity通过 | 未把v1改记成功 |
| v3 | 修正DOM回退模板重复声明造成的语法错误，并扩展进程参数读取宽度 | Cursor／Devin新尝试通过；v2错误属于测试载荷Bug，不归因于应用或注入核心 |
| v4 | FlowUs增加`readyState`、`timeOrigin`、自有节点及registry快照，断言未变 | 保留初次应用／交互通过而更新验证中断的事实 |
| v5 | FlowUs定向等待页面complete且URL／timeOrigin连续至少1.5秒不变，再执行相同载荷与断言 | 新尝试完整通过；先前两次尝试不覆盖 |

Cursor／Devin的基线HTML字符串均被Trusted Types拒绝；v2的回退模板还存在重复`const`声明。v3只修正测试载荷作用域，JS构造DOM随后通过；这证明一种可行载荷，并不证明HTML字符串入口兼容。

FlowUs v4快照显示：初始`https://flowus.cn/`可执行，点击后计数为1、自有节点2个；随后跳转`https://flowus.cn/login/`且`timeOrigin`变化，registry和节点均消失。因此该次中断来自真实页面导航，不能误写成CDP执行失败。v5稳定登录页复验通过；等待条件是本次验证方法，不增加产品自动恢复／重注入职责。

## 运行影响、证据与可复核性

所有完整通过项均已移除自有面板；最终审计确认所有记录在案的新测试主进程均已退出，未主动终止或重启用户原有进程。未直接修改安装包、Injectra源码或用户配置。应用正常启动可能自行写入偏好、日志和运行状态，本轮未做全局配置差异审计；通用隔离参数不能保证每个宿主都遵循。Figma的入口代码还覆盖了用户数据路径，说明仅传入参数不足以证明隔离。

原始证据保留在[开发说明](../dev.md)约定的缓存根下：`runs/app-survey-57d7Au/`。包括`summary.json`、`attempts.json`、`inventory-classified.json`、`cleanup-audit.json`、各版协议和脚本，以及各次`cases/`内的结果／日志／裁剪截图。这里不提交原始文件或222项完整盘点，避免将业务URL、账号数据、机器路径和无关应用信息纳入Git；提交的清单只含应用名、版本、时间、协议、状态及相对证据目录。

本轮使用的现有CLI二进制SHA-256为`51f808ede1d13cd6899c538fb9b84090640d4a87d8a82ec3da956d9ec80746fd`，修改时间早于首次尝试；归档开始时源码基线为`734ab6d`，工作区干净。该哈希标识实测工件，不冒充可重现构建证明。缓存证据未纳入版本控制，清理缓存后只能依据本报告和脱敏清单核对摘要，不能恢复逐次原始日志或截图。

## 公开资料依据

- [Electron命令行调试端口](https://www.electronjs.org/docs/latest/api/command-line-switches#--remote-debugging-portport)：说明调试入口机制，是否保留开关仍须检查具体宿主。
- [Chrome远程调试开关限制](https://developer.chrome.com/blog/remote-debugging-port)：Chrome 136起，调试开关需配合非默认用户数据目录。
- [VS Code高级命令行选项](https://code.visualstudio.com/docs/configure/command-line#_advanced-cli-options)：说明用户目录及扩展目录隔离；派生产品的行为仍以实测为准。
- [NW.js命令行选项](https://docs.nwjs.io/References/Command%20Line%20Options/)：基础运行时提供选项，不能据此断言具体打包宿主开放入口。
- [Clash Verge Rev官方仓库](https://github.com/clash-verge-rev/clash-verge-rev)及[Tauri WebView版本说明](https://v2.tauri.app/reference/webview-versions/)：macOS Tauri使用WKWebView，不归入本轮Chromium／CDP候选验证。
