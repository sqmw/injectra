# 应用平台与内核兼容性总表

更新时间：2026-11-11。历史应用矩阵来自[T-014](../TODO.md#t-014)（ee5640c）；后续T-017、T-021与T-022证据分别列在新增分节，不重算历史普查。**以后查看兼容性以本表为入口；旧普查和单项报告仅承担证据职责，不再各自表示全项目最新状态。**

## 阅读口径

- 当前macOS硬件范围仅Apple Silicon；2026-11-11用户明确排除Intel Mac，不再把Intel硬件列作后续验证缺口，见[D-011](../DECISIONS.md#d-011)。历史记录不改写。
- 每行是“应用＋平台＋应用版本＋运行时/内核＋接入路线”的独立组合，同软件不同版本或路线分别列出。优先展示豆包和抖音等后续成果，历史失败不覆盖新成功。
- 现有本机记录均为macOS，主机为arm64；未逐应用推断目标进程架构。Windows、Linux及其他平台没有实测，本表不编造其版本、内核或支持状态；取得证据后按同列新增行，不套用macOS结果。
- “通过”只覆盖该行版本/页面/受控载荷；“部分通过”不等于完整基线；“仅入口”不等于执行通过；“未验证/未取得页面”不等于不支持。
- Chromium内核列沿用协议报告或明确运行时读回；未连接的应用只列已识别引擎族，不从宿主版本推算内核版本。
- 本工作树的T-017已将Electron Gadget与WKWebView接入正式CLI；旧探针/参考实现证据仍单独保留，新增产品证据见下表。本地main已通过T-018接收本次成果，尚未推送。


## T-017新增宿主证据（已合入本地main，2026-11-10）

权威实现与阶段状态见[Mac交付](../macos-delivery.md)。以下为独立官方测试宿主，不冒充此前企业微信、微信开发者工具等真实应用的成功。

| 引擎/宿主 | 平台/版本 | 路线 | 本轮结果 |
|---|---|---|---|
| CEF官方cefclient | macOS arm64 / CEF156.0.26 / Chromium156.0.8078.88 | CDP | 主页面共享合同通过，run cef-qdvrcqtq |
| NW.js官方SDK宿主 | macOS arm64 / 0.119.2 | CDP | 主页面共享合同通过，run nwjs-ypuq07xm |
| Electron独立fixture | macOS arm64 / 36.4.0 | Gadget | 正式CLI及控制端退出清理通过，run gadget-electron-c5qkiq_k |
| 抖音现有隔离实例 | macOS arm64 / 8.7.0 | Gadget | 正式后端及CLI通过；不改写旧探针run |
| WKWebView独立Swift宿主 | macOS arm64 / 系统WebKit | Gadget→原生页面API | 双页合同、CLI和控制端退出清理通过，run webkit-lg0f0zoc |
| Codex Switcher真实Tauri应用 | macOS arm64 / 0.2.11 | Gadget→WKWebView | 明确授权的空目录实例主页面合同与CLI通过，托盘未污染；测试实例已退出 |

## T-021 Gecko专项证据（已合入本地main，验收于2026-11-10）

| 引擎/宿主 | 平台/版本 | 路线 | 结果与范围 |
|---|---|---|---|
| Gecko / 官方Firefox | macOS26.5.1、arm64主机 / Firefox160.0 | WebDriver BiDi测试客户端 + 唯一Web runtime | 15项真实页面检查通过，run firefox-bidi-Js5cnC；正式CLI尚无BiDi后端，CDP发现返回404；[详情与失败记录](../gecko-validation.md) |

这条证据按引擎/路线新增，不回填原普查计数，也不外推Thunderbird或其他平台；当前集成状态以[T-023](../TODO.md#t-023)为准。

## T-022 Thunderbird专项证据（已合入本地main，验收于2026-11-10）

| 引擎/宿主 | 平台/版本 | 路线 | 结果与限制 |
|---|---|---|---|
| Gecko / Thunderbird | macOS26.5.1、arm64主机 / 159.0 | Marionette + Thunderbird宿主枚举 + Gecko调试执行接口 | 已安装；内容页JS/CSS/HTML通过；主界面JS/CSS及DOM控件通过；主界面HTML按钮字符串未通过，整组partial；BiDi会话可建但不枚举其标签页。正式CLI尚未适配。[实测](../thunderbird-validation.md) |

## T-024 Thunderbird HTML补充验证（2026-11-11）

| 平台/版本 | 新路线与结果 | 仍保留的限制 |
|---|---|---|
| macOS arm64 / Thunderbird159.0 | 测试用DOMParser→importNode装配主界面HTML；37项实机检查通过（最终run thunderbird-gecko-DxFYST）；[验收](../validation/thunderbird-html.md) | 原runtime.html仍partial（默认合同复测fjBUxN），BiDi仍不枚举标签页；不是正式CLI能力，也不代表任意HTML/插件兼容 |

## T-025 抖音原生Interceptor验证（2026-11-11）

| 平台/版本 | 新路线 | 结果 |
|---|---|---|
| macOS / 抖音8.7.0原包 / Frida17.25.2 | 外部attach→Interceptor；Gadget optional→Interceptor | 未通过：前者PermissionDenied；后者在监听建立前被CODESIGNING/Invalid Page终止。Hook未进入；自有调试进程对照通过不计作抖音支持。[证据](douyin-native-hook.md) |

## T-026 抖音Objective-C分派Hook验证（2026-11-11）

| 平台/版本 | 新路线 | 结果与边界 |
|---|---|---|
| macOS arm64 / 抖音8.7.0原包 / Frida17.25.2 | Gadget required → Objective-C方法IMP替换 | 自有测试对象的两次回调、原结果保持、恢复及卸载通过；原包签名有效。未验证抖音业务方法、任意C/C++函数；不是正式CLI能力，不覆盖T-025的Interceptor限制。[证据](douyin-objc-hook.md) |

## T-027 抖音Interceptor授权重试（2026-11-11）

| 平台/版本 | 复测结果 | 证据边界 |
|---|---|---|
| macOS / 抖音8.7.0原包 / Frida17.25.2 | 外部attach两次仍PermissionDenied，首次用户确认无授权窗口；Gadget optional再次CODESIGNING/Invalid Page | 新的自有可调试进程外部attach→Interceptor通过，补齐旧对照缺口；不证明抖音原包支持，未改系统保护或应用签名。[复测](douyin-hook-auth-retry.md) |

## T-028 抖音真实宿主方法Hook（2026-11-11）

| 平台/版本 | 路线与实测 | 边界 |
|---|---|---|
| macOS arm64 / 抖音8.7.0原包 / Frida17.25.2 | Gadget required → AtomApplication.sendEvent:方法分派Hook；30秒无窗口操作复测360/360、最终默认命令147/147原调用完成，恢复/卸载通过 | 真实Electron宿主方法，未创建测试对象；不证明业务逻辑、任意原生函数或Interceptor支持。[证据](douyin-host-hook.md) |

## T-032 十款应用顺序验证（2026-11-11）

本批应用逐一尝试的最新结果见[详细证据](../validation/macos-ten-apps.md)，整体尝试已完成，五款已测页面通过、五款接入未完成，见[T-032](../TODO.md#t-032)。旧普查行保持历史口径。

| 应用/版本 | 结果 | 实测范围或接入缺口 |
|---|---|---|
| 阿里云盘6.8.0 | 未取得页面 | 两次CDP启动进程提前退出，无调试对照保持运行；不以此推断所有版本 |
| 阿里云盘6.9.3 | 未取得页面 | 获退出授权后补测：Node入口未出现且开关关闭，CDP监听后进程退出0；未执行载荷，普通窗口已恢复 |
| 百度网盘8.2.7 | 通过已测页面 | core.asar/index.html，CSS/HTML/JS、点击、更新与清理 |
| 企业微信5.0.9 | 未取得页面 | CEF renderer已启动；指定及其他实际监听端口未返回CDP页面 |
| 千问5.1.7.296 | 通过已测页面 | chrome://newtab完整合同；新增包内日志已备份移出，签名复核恢复 |
| Anytype0.55.4 | 通过已测页面 | dist/tabs.html宿主标签页合同；未外推全部内容页 |
| Claude4.23118.1 | 接入被应用拒绝 | 明确拒绝调试参数启动；原用户实例保留 |
| Xmind24.01.13311 | 通过已测页面 | 新profile欢迎页合同，原用户文档实例保留 |
| 腾讯视频2.164.0 | 未取得页面 | Web视图已运行；指定及其他实际监听端口未返回CDP页面 |
| 网易云音乐3.1.5 | 通过已测页面 | orpheus://orpheus/app.html合同；不外推登录或播放业务 |
| QQ6.9.86 | 未取得页面 | 原实例端口非CDP、attach拒绝；获准退出后CDP/Node启动均未取得目标，已重新打开普通QQ窗口 |

## T-033 阿里云盘Frida验证（2026-11-11）

| 应用/版本 | 路线 | 实测结果与范围 |
|---|---|---|
| 阿里云盘6.9.3 / macOS arm64 | Gadget required启动；直接Frida attach | Gadget监听及身份回执未建立；直接附加PermissionDenied；未进入Hook或页面代码。原包库验证开启等条件已读回，未改保护；测试实例结束，普通窗口恢复。[证据](adrive-frida.md) |

## 历史应用矩阵

pre-runtime后续探索：[T-034阿里云盘入口核查](adrive-preruntime.md)。初次静态核查阶段只有内部preload/组件清单等静态证据，尚无可确认的外部早期加载入口，该阶段不计作新注入成功或新实机run。

T-034继续的[启动条件复核](adrive-startup.md)累计4次无载荷观察：隔离HOME下监听持续但HTTP超时，后续采样见钥匙串授权等待；正常HOME下CDP得到空列表后退出，普通启动持续运行。用户解锁后新CDP测试约0.7秒退出，未取得页面；实际菜单/偏好设置未见调试入口，普通首页已恢复。钥匙串提示状态及退出根因仍未确认，不计注入成功。

T-035新增[独立测试副本资源补丁](adrive-resource-patch.md)：阿里6.9.3真实本地登录壳页面通过共享runtime的JS/CSS/HTML标记与按钮回执。条件为资源重打包、更新ASAR完整性、本地ad-hoc签名允许库加载、测试钥匙串和独立目录；不等于原包CDP/Hook支持，未登录或验证账号业务。原普通首页已恢复。

| 应用 | 平台 | 应用版本 | 宿主／运行时 | 内核／协议实测值 | 接入路线与条件 | 当前验证状态与范围 | 证据／代码入口 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 豆包 | macOS | 2.33.5 | 定制 Chromium（非标准Electron） | Chrome/147.0.7727.149 | CDP；完全退出后携带回环参数启动 | 通过（产品CLI，历史实测）；真实chat页CSS/HTML/JS、点击、更新、remove通过；独立论文/主题插件另有应用成果，不并入普查次数 | [豆包实测](doubao.md) |
| 豆包 | macOS | 2.34.6 | 定制 Chromium | Chrome/147.0.7727.149 | 现有CDP入口 | 仅入口可达；此版本未复验注入；普查仅查入口；不能据此否认同软件2.33.5已通过；见该版本独立记录 | [普查](macos-cdp-survey.md) |
| 抖音 | macOS | 8.7.0 | Electron 36.4.0-rs.33.release.pgo.0 / V8 13.6.233.8-electron.0 | Chromium 136.0.7103.59 | Gadget 17.25.2（QJS）→libuv/V8→原Node→页面 | 通过（独立探针，非CDP）；真实精选页可见、点击读回0→1及13项受控检查通过；参考源码已归档，Rust CLI未装配Frida后端；后续计数不满足的复验另存 | [实测/限制](douyin-gadget.md) · [代码](../../references/apps/douyin/gadget-v8/README.md) |
| 抖音 | macOS | 7.9.0 | Electron / Chromium | 版本未实测 | CDP尝试 | 历史：此路线未取得CDP页面；7.9.0普通CDP/Node调试参数未达；不替代8.7.0 Gadget新路线成功 | [CDP排查](macos-cdp-followup.md) |
| 抖音 | macOS | 7.9.0 | Electron / V8（完整版本未记录） | Chromium版本未实测 | 外部Frida直接attach | 历史：附加被权限拒绝；主进程和一个renderer PermissionDenied；不能外推Gadget不可用 | [版本及路线记录](douyin-gadget.md) |
| Figma | macOS | 126.5.6 | Electron 41.7.0 / Chromium | Chrome/146.0.7680.216 | CDP；独立配置＋FIGMA_TEST=1 | 部分通过（产品CLI）；外壳shell.html注入和首次点击通过；登录后编辑器及完整更新/remove未验；网络需单独核对 | [实测](figma.md) · [参考代码](../../references/apps/figma/cdp/README.md) |
| WorkBuddy | macOS | 5.9.3 | Electron / Chromium | Chrome/138.0.7204.251 | CDP；宿主配置/用户目录覆盖 | 通过（产品CLI）；独立宿主目录核对后，基线载荷整组通过；旧隔离未确认已由新尝试补齐 | [后续实测](macos-cdp-followup.md) · [参考代码](../../references/apps/workbuddy/cdp/README.md) |
| 优酷 | macOS | 9.2.73 | Electron / Chromium | Chrome/93.0.4577.82 | CDP；自身沙箱容器内新profile | 通过（产品CLI）；容器内profile下基线整组通过；不代表全部宿主状态隔离 | [后续实测](macos-cdp-followup.md) · [参考代码](../../references/apps/youku/cdp/README.md) |
| 阿里云盘 | macOS | 6.8.0 | Electron / Chromium | 版本未实测 | 未验证 | 未验证；同步/网盘客户端；未确认独立账号及数据目录，不启动后台同步 | [普查](macos-cdp-survey.md) |
| 百度网盘 | macOS | 8.2.7 | Electron / Chromium | 版本未实测 | 未验证 | 未验证；同步/下载客户端；未确认独立账号及数据目录，不启动后台同步 | [普查](macos-cdp-survey.md) |
| 哔哩哔哩 | macOS | 1.14.2 | Electron / Chromium | Chrome/108.0.5359.215 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| 企业微信 | macOS | 5.0.9 | CEF组件 / Chromium（主界面性质另核） | 版本未实测 | 未验证 | 未验证；带CEF；未确认独立用户数据与调试入口，不接入现有账号 | [普查](macos-cdp-survey.md) |
| 千问 | macOS | 5.1.5.293 | 定制 Chromium | 版本未实测 | 未验证 | 未验证；定制Chromium应用正在运行；不重启或改写现有配置 | [普查](macos-cdp-survey.md) |
| 腾讯视频 | macOS | 2.164.0 | CEF组件 / Chromium（主界面性质另核） | 版本未实测 | 未验证 | 未验证；带CEF；缺少已确认的隔离启动及CDP入口配置，未启动 | [普查](macos-cdp-survey.md) |
| 网易云音乐 | macOS | 3.1.5 | CEF组件 / Chromium（主界面性质另核） | 版本未实测 | 未验证 | 未验证；带CEF；缺少已确认的隔离启动及CDP入口配置，未启动 | [普查](macos-cdp-survey.md) |
| 微信开发者工具 | macOS | 1.06.2412050 | NW.js / Chromium | 版本未实测 | CDP尝试 | 未取得页面；按用户要求跳过；原主进程约1秒正常退出；不据此判NW.js不支持CDP | [勘误](macos-cdp-followup.md) |
| Antigravity | macOS | 2.24.0 | Electron / Chromium | Chrome/152.0.7977.78 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Anytype | macOS | 0.55.4 | Electron / Chromium | 版本未实测 | 未验证 | 未验证；已运行；本轮不重启，尚未确认可并行隔离的接入方式 | [普查](macos-cdp-survey.md) |
| Clash Verge Rev | macOS | 未记录 | Tauri / WKWebView | WebKit（版本未测） | 尚未验证WebKit接入 | 未验证；非CDP样本；仅运行时路线分类，不计作Chromium/CDP失败；未核对其他平台版本 | [分类依据](macos-cdp-survey.md) |
| Claude | macOS | 4.4212.0 | Electron / Chromium | 版本未实测 | 未验证 | 未验证；已运行；本轮不重启，未新增调试入口 | [普查](macos-cdp-survey.md) |
| CocosCreator | macOS | 3.8.8 | Electron / Chromium | 版本未实测 | CDP尝试 | 已有调试服务日志，未取得可用page；旧日志有DevTools listening；窗口/目标更替原因未确认，不再写“未开放CDP” | [勘误](macos-cdp-followup.md) |
| CocosDashboard | macOS | 2.2.1 | Electron / Chromium | Chrome/126.0.6478.185 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Cursor | macOS | 3.7.36 | Electron / Chromium | Chrome/142.0.7444.265 | CDP | 通过（产品CLI）；JS构造DOM整组通过；HTML字符串受Trusted Types限制 | [普查](macos-cdp-survey.md) |
| Deskreen | macOS | 2.0.4 | Electron / Chromium | 版本未实测 | 未验证 | 未验证；屏幕共享服务启动副作用不纳入仅UI注入测试 | [普查](macos-cdp-survey.md) |
| Devin | macOS | 3.10.35 | Electron / Chromium | Chrome/148.0.7778.97 | CDP | 通过（产品CLI）；JS构造DOM整组通过；HTML字符串受Trusted Types限制 | [普查](macos-cdp-survey.md) |
| draw.io | macOS | 30.3.14 | Electron / Chromium | Chrome/148.0.7778.280 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| EasyConnect | macOS | 7.6.7 | Electron / Chromium | 版本未实测 | 未验证 | 未验证；VPN/网络服务启动副作用不纳入仅UI注入测试 | [普查](macos-cdp-survey.md) |
| FlowUs | macOS | 1.2.1 | Electron / Chromium | Chrome/108.0.5359.179 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Follow | macOS | 0.3.3-beta.0 | Electron / Chromium | Chrome/130.0.6723.118 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Google Chrome | macOS | 156.0.8078.80 | Chromium 浏览器 | Chrome/156.0.8078.80 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Logseq | macOS | 0.10.15 | Electron / Chromium | Chrome/140.0.7339.240 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Manus | macOS | 1.7.7 | Electron / Chromium | Chrome/148.0.7778.280 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Microsoft Edge | macOS | 156.0.4314.58 | Chromium 浏览器 | Edg/156.0.4314.58（协议值；Chromium具体号未另测） | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Mockitt | macOS | 1.3.6 | Electron / Chromium | Chrome/108.0.5359.215 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Notion | macOS | 7.35.0 | Electron / Chromium | Chrome/150.0.7871.250 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| OBS Studio | macOS | 32.2.2 | CEF组件 / Chromium（主界面性质另核） | 版本未实测 | 未验证 | 未验证；CEF是嵌入浏览器组件，主UI并非纯Web；未新增宿主调试设置 | [普查](macos-cdp-survey.md) |
| QQ | macOS | 6.9.86 | QQNT / Electron 37.1.0 / Chromium | 版本未实测 | 未验证 | 未验证；QQNT框架确认Electron37.1.0；现有候选端口未发现CDP列表，不重启 | [普查](macos-cdp-survey.md) |
| Quark | macOS | 7.1.2.956 | 定制 Chromium | Chrome/144.0.7559.86 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Super Productivity | macOS | 16.5.3 | Electron / Chromium | Chrome/138.0.7204.251 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Trae | macOS | 3.5.73 | Electron / Chromium | Chrome/142.0.7444.235 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Trae CN | macOS | 3.4.11 | Electron / Chromium | Chrome/142.0.7444.235 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| TRAE SOLO CN | macOS | 0.1.19 | Electron / Chromium | Chrome/142.0.7444.235 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |
| Xmind | macOS | 24.01.13311 | Electron / Chromium | 版本未实测 | 未验证 | 未验证；已运行且有用户文档；保留现有实例 | [普查](macos-cdp-survey.md) |
| ZCode | macOS | 3.1.0 | Electron / Chromium | Chrome/146.0.7680.80 | CDP | 通过（产品CLI）；CSS/HTML/JS、点击、同ID更新及remove整组通过 | [普查](macos-cdp-survey.md) |

## 统计与更新

历史应用矩阵保留44个版本/路线组合、41个应用名（不含上述新增分节）：原40候选全部保留，补入豆包2.33.5、抖音7.9直接附加、抖音8.7 Gadget及一项WebKit路线分类。此计数是总表覆盖范围，不是新增实测次数或总体成功率。

原普查40候选、26款启动、31次尝试、20款完整通过仍只属于原run；后续WorkBuddy/优酷、Figma、抖音和豆包历史证据不回填该run。不同验证范围不合成一个“全平台通过率”。

新增结果只更新本表中对应组合，新增平台/版本/路线则新增行，记录日期、条件及证据链接。原始run、失败工件和逐次CSV保持原样；已完成报告只从本表证据列进入。

### 证据保留入口

[豆包](doubao.md) · [原普查及逐次清单](macos-cdp-survey.md) · [后续排查](macos-cdp-followup.md) · [Figma](figma.md) · [抖音Gadget](douyin-gadget.md)。

[产品边界](../requirements.md) · [参考源码与分组统计](../../references/README.md)。实际论文/主题Web插件属于独立插件项目，不与注入接入代码或兼容性尝试混算。
