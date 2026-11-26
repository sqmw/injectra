# Windows九款未通过项的宿主技术核查

2026-11-13，[T-044](../TODO.md#t-044) / E-001。只读核查安装结构、包元数据和二进制引擎标识；没有重启或注入App。原[Windows测试结果](windows-apps.md)不回填。

## 结论

九款都存在Web相关技术证据，不能将页面注入未完成归因于“不是Web软件”。六款有Electron二进制fuse标识和对应应用包；另三款存在CEF或多引擎组件。静态组件存在不等于整个主界面由Web渲染，也不等于已取得可执行页面。

| 应用 | Windows已核实证据 | 判断及边界 |
| --- | --- | --- |
| QQ | 版本化QQNT.dll内Electron fuse；qq-chat包main为application.asar/app_launcher/index.js | Electron/QQNT宿主明确；不是普通根目录exe＋固定resources布局 |
| WorkBuddy | 主exe内Electron fuse；app.asar中@genie/workbuddy-desktop的main为main/index.js | Electron宿主明确；入口、缓存/完整性问题仍独立存在 |
| Figma | 实际app-126.11.8/Figma.exe内Electron fuse；figma-desktop包main.js | Electron宿主明确；启动器和实际程序不能混用 |
| 优酷 | 主exe内Electron fuse；youku-app包main为init.js | Electron宿主明确；此前笼统native/CEF推断应按此证据校准 |
| 抖音 | 8.8.0/main.dll内Electron fuse；主资源和tray各有app.asar，包名douyin | 版本化Electron运行时明确；根启动器的ia32 Hook不代表已接入实际页面 |
| Cocos Creator | 主exe内Electron fuse；CocosCreator包main为launch/dist/index | 编辑器Electron宿主明确；不能把原生预览/引擎部分与编辑器外壳合并判断 |
| 百度网盘 | libcef 86.0.24/Chromium86；WebView2Loader；module/BrowserEngine/app.asar包main为dist/electron/entry.js | 多引擎/混合技术，不能只归CEF或认定全UI是Web；需定位实际待改窗口使用哪个模块 |
| 企业微信 | compatible_web内CEF107/Chromium107 | 明确带CEF Web能力；主界面与原生部分边界尚未确认 |
| 网易云音乐 | libcef 91.2.2/Chromium91 | 明确带CEF Web能力；不能静态断言全主界面是CEF |

## 对后续验证的影响

Electron组优先统一实际资源根、主模块加载方式、缓存/完整性和页面上下文；不按品牌复制载荷。CEF/多引擎组先绑定窗口/进程与实际渲染组件，再决定CDP或原生Web桥，不能把原生UI操作包装为DOM。当前没有为此新增后端或实现。

验证目标是“具体窗口/页面的JS/HTML/CSS实际执行及更新/清理”，不能以Electron标识、DLL或ASAR存在替代成功。新平台技术栈必须独立取证，不能从Mac版本外推Windows版本。

## 证据

仓库外windows-validation-20261112/nine-host-technology-20261113.json记录组件路径/版本；nine-host-package-fuse-20261113.json记录包name/main和fuse标识。扫描未运行应用源码、解码不透明入口或读取用户文档/聊天数据。源代码及已安装资源均未修改。

## v2 复验工具边界

E-002按Windows实际入口复验。实验副本复制器新增[包内链接复制](../../tests/windows/copy-tree.mjs)：先验证链接最终目标位于源包内且属于复制范围，再重建指向派生副本的链接；外部目标、循环与既有目的目录明确拒绝。不得保留回指原安装的Junction。`make test-windows-format`覆盖链接隔离、外部拒绝、PE摘要和ASAR保留；格式测试不代表Cocos等实机通过。

CEF启动实验：[启动器](../../tests/windows/cef-spawn.py)、[初始化探针](../../tests/windows/cef-init.js)。仅在明确的CEF 86/91/107版本、目标架构及完整settings尺寸同时匹配时，替换本次cef_initialize参数为临时副本并设置调试端口，不修改原settings、安装文件或沙箱选项；未知版本/尺寸拒绝。布局依据CEF官方分支[4240](https://github.com/chromiumembedded/cef/blob/4240/include/internal/cef_types.h)、[4472](https://github.com/chromiumembedded/cef/blob/4472/include/internal/cef_types.h)、[5304](https://github.com/chromiumembedded/cef/blob/5304/include/internal/cef_types.h)。这是实机入口探索工具，不是正式后端或页面成功证明。

包内Junction先读取显式目标再规范化，规避Windows libuv对尾部分隔符Junction的UNKNOWN；目标仍须canonical化且限制在源包。实验prepare-copy的`--exclude <相对路径>`仅配置明确的运行数据/锁目录，并写入proof清单，不能排除应用package；不因EBUSY静默跳过未知文件。

Windows x64 V8 ABI实验：[agent](../../crates/frida/helper/v8_windows.js)、[自有Node隔离验证](../../tests/windows/v8-fixture.py)。只选择明确引擎模块和完整MSVC导出符号，在目标I/O线程尝试受控执行；必须先通过自有进程验证，不能将Mac ABI或符号存在当作执行证明。32位目标不进入此实验，尚不是正式WebBackend。

受控Node Windows x64 V8验证已取得目标进程内marker（42、实际PID、win32），随后显式目标[附加器](../../tests/windows/v8-attach.py)核对exe/PID世代，调用同一agent。只派发引导加载，不终止现有App；派发成功仍不等于页面成功，后者独立用Bootstrap共享Web合同验收。

Windows V8实验按MSVC ABI区分x64与ia32，32位先使用独立下载并核对官方SHA256的Node x86夹具验证，不对应用直接试错。真实Bootstrap验收增加目标世代连续稳定3秒的门槛，并记录选定目标的去查询参数URL；启动页重建拒绝仍保留旧run，后续显式连接属于新run，不重放Unknown。

V8实验同时观察Windows I/O完成与消息泵入口，不假定所有Electron版本使用同一种泵。卸载回执记录命中、实际尝试和取消的排队数；只有卸载明确确认零尝试，才把尚未派发的请求判为NotExecuted，其他不确定结果继续Unknown且不自动重放。

交互启动器在关闭前记录自有根进程是否已退出、退出码与时间，区分“没有调试页面”和“应用已结束”；根进程结束后仍保留其子页面发现机会。窗口类别/错误框只能在交互桌面任务中核查，SSH Session 0直接枚举不到窗口不能视为应用无UI。

后续E003已在活动网易云3.1.38主进程/renderer确认CEF91；用户正常退出后，标准CEF参数启动的新实例经现有Windows CLI/CDP通过完整页面合同。见[实机收口](windows-retries.md)，不以早期静态表代替新证据。
