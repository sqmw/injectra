# Windows全应用注入复验

[T-043](../TODO.md#t-043)，2026-11-12开始，11-13续测。**安装列表38款均已实际尝试：29款取得页面正向证据，9款本轮未完成页面注入。** FlowUs、Manus跳过。Mac原候选中的Deskreen/OBS/EasyConnect/Clash原主要是静态枚举或未启动候选，额外单列同级核查，不冒充UI成功。

## 后续复验

[T-044@v2复验](windows-retries.md)新增QQ、WorkBuddy、Figma、抖音、Cocos Creator五款页面成功，累计34/38，尚余4款。下文29/9及逐run是T-043首轮历史口径，原证据不回填。

## 结果口径

| 类别 | 数量/结果 | 范围 |
| --- | --- | --- |
| Windows原生Rust CLI/CDP | 25款 | 包含WebView2等CDP入口；JS/CSS、交互、更新、清理通过；Devin/Quark/Cursor仅JS DOM路线通过，HTML字符串限制保留 |
| Windows Gecko测试客户端 | 2款 | Firefox BiDi；Thunderbird Marionette/宿主API，内容页与主界面parsed-html路线；不是正式CLI Gecko后端 |
| pre-runtime实验 | 2款 | Claude、阿里云盘私有副本→同一Bootstrap/WebBackend→单一runtime，两版JS/HTML/CSS、交互与清理通过；Mac Rust测试控制端，非正式Windows prepare/dev |
| 原生函数Hook实验 | 2款 | 阿里云盘x64 13/13、抖音ia32启动/宿主进程6/6，detach后不增长且原字节恢复；不计为页面注入成功 |

早期Mac控制端SSH转发结果独立保留；本表优先原生Windows CLI或测试客户端证据，不以安装、端口、Hook计数替代真实页面。

## 逐项证据

| 应用 | 当前入口/阶段结果 | 控制方式 | 证据 |
| --- | --- | --- | --- |
| Notion | 已测页面合同通过 | windows-native | native-runs/Notion-830073cb/result.json |
| Logseq | 已测页面合同通过 | windows-native | native-runs/Logseq-95c9758c/result.json |
| Folo | 已测页面合同通过 | windows-native | native-runs/Folo-2adea2d0/result.json |
| Anytype | 已测页面合同通过 | windows-native | native-runs/Anytype-0754da64/result.json |
| Xmind | 已测页面合同通过 | windows-native | native-runs/Xmind-602c34ad/result.json |
| QQ | CDP未取得页面 | windows-native | native-runs/QQ-64ebfc82/result.json |
| Mockitt | 已测页面合同通过 | windows-native | native-runs/Mockitt-0ba30e3b/result.json |
| SuperProductivity | 已测页面合同通过 | windows-native | native-runs/SuperProductivity-60dbbe93/result.json |
| CodexSwitcher | 已测页面合同通过 | windows-native | native-runs/CodexSwitcher-3dde684b/result.json |
| WorkBuddy | CDP未取得页面 | windows-native | native-runs/WorkBuddy-6575d05b/result.json |
| Qianwen | 已测页面合同通过 | windows-native | native-runs/Qianwen-26454369/result.json |
| Antigravity | 已测页面合同通过 | windows-native | native-runs/Antigravity-6b040d60/result.json |
| Devin | JS DOM/CSS通过，HTML字符串受限 | windows-native | native-runs/Devin-7d2dff02/result.json |
| Trae | 已测页面合同通过 | windows-native | native-runs/Trae-be1b86ea/result.json |
| TraeCN | 已测页面合同通过 | windows-native | native-runs/TraeCN-a13cc3c4/result.json |
| TraeSoloCN | 已测页面合同通过 | windows-native | native-runs/TraeSoloCN-9ae63782/result.json |
| ZCode | 已测页面合同通过 | windows-native | native-runs/ZCode-52025957/result.json |
| Quark | JS DOM/CSS通过，HTML字符串受限 | windows-native | native-runs/Quark-d1dd89b7/result.json |
| Claude | pre-runtime页面更新/清理通过（实验） | Mac Rust Bootstrap测试控制端 | bootstrap/Claude-b6509b7f/result.json |
| BaiduNetdisk | CDP未取得页面 | windows-native | native-runs/BaiduNetdisk-879f7c59/result.json |
| Youku | CDP未取得页面 | windows-native | native-runs/Youku-184dfa19/result.json |
| Bilibili | 已测页面合同通过 | windows-native | native-runs/Bilibili-c9342b8b/result.json |
| aDrive | pre-runtime页面更新/清理通过（实验） | Mac Rust Bootstrap测试控制端 | bootstrap/aDrive-3313a603/result.json |
| WeCom | CDP未取得页面 | windows-native | native-runs/WeCom-c90583d2/result.json |
| TencentVideo | 已测页面合同通过 | windows-native | native-runs/TencentVideo-6479aa27/result.json |
| Firefox | 已测页面合同通过 | windows-native | gecko-runs/Firefox-e50418f0/result.json |
| Thunderbird | 已测页面合同通过 | windows-native | gecko-runs/Thunderbird-975d6f47/result.json |
| WeixinDevTools | 已测页面合同通过 | windows-native | native-runs/WeixinDevTools-66804768/result.json |
| Douyin | 原生函数Hook通过，页面注入未证明 | windows-native | hook-runs/Douyin-348594da/result.json |
| CocosDashboard | 已测页面合同通过 | windows-native | native-runs/CocosDashboard-12b71bfa/result.json |
| drawio | 已测页面合同通过 | windows-native | native-runs/drawio-9b3736d4/result.json |
| Figma | CDP未取得页面 | windows-native | native-runs/Figma-9a3ed08c/result.json |
| Chrome | 已测页面合同通过 | windows-native | native-runs/Chrome-b386c7fe/result.json |
| Edge | 已测页面合同通过 | windows-native | native-runs/Edge-96cfb4e8/result.json |
| Doubao | 已测页面合同通过 | windows-native | native-runs/Doubao-61ab6f75/result.json |
| Cursor | JS DOM/CSS通过，HTML字符串受限 | windows-native | native-runs/Cursor-7925634e/result.json |
| CloudMusic | CDP未取得页面 | windows-native | native-runs/CloudMusic-4ed682e7/result.json |
| CocosCreator | CDP未取得页面 | windows-native | native-runs/CocosCreator-ea50eee9/result.json |
| OBS Studio | not-in-installation-list | — | — |
| FlowUs | skipped-user | — | — |
| Manus | skipped-user | — | — |
| Deskreen | not-in-installation-list | — | — |
| EasyConnect | mac-inventory-only-network-side-effects-not-started | — | — |
| ClashVergeRev | existing-network-service-preserved | — | — |


证据均为仓库外windows-validation-20261112下相对路径，不是仓库输入依赖；源码版本与控制/目标平台在各run中分开。原始run、工具失败、重试与截图超时均保留，不以最新网络失败抹掉旧成功。

[九项Windows技术栈只读核查](windows-host-technology.md)：九款均有Web相关组件证据，六款Electron明确；CEF/混合UI不等于全主界面Web。

## 尚未取得页面的9项

| 应用 | 已尝试/观察 | 本轮缺口 |
| --- | --- | --- |
| QQ | CDP、版本化资源根、目录主模块委托、Node预加载；原包fuse静态读取 | 改入口副本弹文件损坏；外置预加载标记未到达；不能据此判定QQ永久不支持 |
| WorkBuddy | CDP、ASAR入口、PE摘要、委托/插入、保留虚拟/缺失外置条目的写入 | 副本未注册可执行页面；需进一步区分启动/缓存/宿主拦截 |
| Figma | 更正实际版本化exe、FIGMA_TEST、CDP、ASAR委托 | 已启动外壳但未取得Bootstrap页面；编辑器/登录业务未测 |
| 百度网盘 | CDP、Qt/WebView2进程环境候选、静态libcef识别 | CEF＋WebView2加载器＋独立Electron浏览器模块并存，尚未绑定实际窗口/引擎 |
| 优酷 | CDP、Qt/WebView2回环环境探测 | 未取得页面，不能用Mac已通过外推Windows |
| 企业微信 | CDP、Qt/WebView2环境探测、安装结构核对 | 未取得页面，native/CEF宿主需进一步桥接 |
| 抖音 | CDP、版本化资源定位、真实Win32 Hook | Hook仅证明进程内插桩，未建立页面执行通道 |
| 网易云音乐 | CDP、Qt/WebView2环境探测、libcef识别 | CEF页面桥尚未建立 |
| Cocos Creator | CDP、正确版本exe、副本入口准备 | 来源含重解析项，受控复制拒绝；未绕过或修改原项目 |

本轮收口的是**全清单尝试与证据分类**，不是“所有软件兼容”或“所有Mac后端已移植”。剩余问题需按宿主/协议/资源格式泛化处理，不新增按品牌分支；未据失败修改用户全局安全、代理或调试策略。

## 安装与入口纠正

Folo根Folo.exe经7-Zip证实为Setup.exe，原便携部署只留下安装器。已保留原文件，从内嵌官方full.nupkg展开真实客户端到D:\exe\Folo\app-1.17.0\Folo.exe，新入口Windows页面合同通过。QQ资源实际在versions子目录，Figma真实exe在app-126.11.8子目录；旧启动器失败不算应用兼容失败。安装纠正独立记录，不覆盖原安装run。

## 工具、构建与恢复

- Mac维护源码，经Git放入仓库外Windows source-69ddebc验证checkout，不覆盖D:\language同步目录。WindowsRust1.96.1/MSVC编译CLI，Node24.18.0执行测试。core/native/web/application/cdp单测与doc tests通过；曾误用mise rustdoc的失败保留，后续仅在命令环境指定既有RUSTDOC修正。
- [交互启动与原生执行](../../scripts/windows-cdp-smoke.py)、[SSH传输](../../scripts/windows_remote.py)、[只核对自有进程的收尾](../../scripts/windows-clean-owned.py)、[结果汇总](../../scripts/summarize-windows-tests.py)。清理默认dry-run，用户已明确批准后才对核验过的本轮PID/创建时间/路径操作，初始会话排除。
- [Gecko测试客户端](../../scripts/windows-gecko-verify.mjs)、[Win32 Hook](../../scripts/windows-frida-hook.py)/[agent](../../tests/windows/dispatch-hook.js)、[副本准备](../../tests/windows/prepare-copy.mjs)、[PE完整性](../../tests/windows/pe-integrity.mjs)、[ASAR增量保留](../../tests/windows/append-asar.mjs)、[真实Bootstrap合同](../../crates/bootstrap/tests/remote_live.rs)。后三者是测试工具，不是正式Windows准备器。
- 新资源路径保持[Electron完整性格式](https://www.electronjs.org/docs/latest/tutorial/asar-integrity)，不关闭fuse；变更PE资源的副本不保证原Authenticode签名。仅源基线匹配的自有副本可修改，原安装程序/数据保留。
- Mac make check、三类Electron准备回归、四项入口测试、PE32/64与ASAR元数据保留格式测试通过。Windows特有探针已完成真实验证的范围按上表，其余格式测试不是App兼容性证明。

[历史阶段检查点](../archive/T-043-checkpoints.md) · [恢复上下文](../tasks/T-043-context.md)。仓库外原始证据、配对配置与派生副本保留，临时任务/SSH桥/Git源仅本轮使用；不要发布含配对密钥的文件。

## 最终复核

Parent按38个安装项逐项对账，补入Gecko/启动器纠正、source/control平台及pre-runtime证据；未用Hook替代页面，不删除失败run，不关闭用户初始会话。清理脚本先dry-run检查初始PID交集为零，后按用户明确授权核验启动时间/镜像路径/独有profile标识请求结束残留；进程查询竞态跳过而非推断归属。汇总工具只派生快照，不改raw结果或Task状态。源码、运行数据、安装修正分开保留。
