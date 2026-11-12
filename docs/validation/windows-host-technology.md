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
